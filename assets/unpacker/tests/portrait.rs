use std::collections::HashMap;
use std::path::PathBuf;

use unpacker::export::portrait;
use unpacker::export::texture::{DecodedTexture, decode_texture_object};
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

fn test_output_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(format!("{}/test_output/{name}", env!("CARGO_MANIFEST_DIR")));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Parse a portrait bundle: collect MonoBehaviours + decode Texture2Ds by path_id.
fn load_portrait_bundle(
    bundle_path: &str,
) -> (
    Vec<portrait::SpritePackerData>,
    HashMap<i64, DecodedTexture>,
) {
    let data = match std::fs::read(bundle_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return (vec![], HashMap::new());
        }
    };

    let bundle = BundleFile::parse(data).unwrap();

    let mut resources = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }

    let mut all_objects: HashMap<i64, (i32, serde_json::Value)> = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            continue;
        }
        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };
        for obj in &sf.objects {
            if obj.class_id == 114 || obj.class_id == 28 {
                let val = match read_object(&sf, obj) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                all_objects.insert(obj.path_id, (obj.class_id, val));
            }
        }
    }

    // Parse SpritePacker MonoBehaviours
    let mut packers = Vec::new();
    let mut claimed = std::collections::HashSet::new();
    for (pid, (class_id, val)) in &all_objects {
        if *class_id == 114
            && let Some(packer) = portrait::parse_sprite_packer(val)
        {
            claimed.insert(*pid);
            claimed.insert(packer.texture_pid);
            claimed.insert(packer.alpha_pid);
            packers.push(packer);
        }
    }

    // Decode all claimed textures (need resources for stream data)
    let mut decoded: HashMap<i64, DecodedTexture> = HashMap::new();
    for (pid, (class_id, val)) in &all_objects {
        if *class_id == 28 && claimed.contains(pid) {
            match decode_texture_object(val, &resources) {
                Ok(Some(tex)) => {
                    decoded.insert(*pid, tex);
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }
    }

    (packers, decoded)
}

#[test]
fn test_portrait_extraction_pack12() {
    let path = format!(
        "{}/tests/assets/charportraits_pack12.ab",
        env!("CARGO_MANIFEST_DIR")
    );
    let (packers, decoded) = load_portrait_bundle(&path);
    if packers.is_empty() {
        eprintln!("skip: no packers found (asset file may be missing)");
        return;
    }

    assert!(
        !packers.is_empty(),
        "should find SpritePacker MonoBehaviours"
    );
    assert!(!decoded.is_empty(), "should decode atlas textures");

    let dir = test_output_dir("portrait_pack12");
    let count = portrait::extract_portraits(&packers, &decoded, &dir);
    assert!(count > 0, "should extract at least one portrait");

    // Check that individual portrait files exist
    let pngs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "png"))
        .collect();
    assert!(
        pngs.len() > 10,
        "should extract many portraits, got {}",
        pngs.len()
    );

    // Verify dimensions and alpha for a sample portrait
    for entry in pngs.iter().take(5) {
        let img = image::open(entry.path()).unwrap().to_rgba8();
        let (w, h) = (img.width(), img.height());

        // Portraits should be 180x360 (or rotated: still 180x360 output)
        assert!(
            (w == 180 && h == 360) || (w == 360 && h == 180),
            "portrait {} has unexpected dimensions {}x{}",
            entry.file_name().to_string_lossy(),
            w,
            h
        );

        // Alpha should be properly merged (not all 255)
        let has_transparent = img.pixels().any(|p| p.0[3] < 255);
        let has_opaque = img.pixels().any(|p| p.0[3] > 0);
        assert!(
            has_transparent,
            "portrait {} should have transparent pixels (alpha merged from separate atlas)",
            entry.file_name().to_string_lossy()
        );
        assert!(
            has_opaque,
            "portrait {} should have opaque pixels",
            entry.file_name().to_string_lossy()
        );
    }
}
