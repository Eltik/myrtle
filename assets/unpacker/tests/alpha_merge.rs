use std::collections::HashMap;
use std::path::PathBuf;

use unpacker::export::alpha_merge::merge_and_export;
use unpacker::export::texture::{DecodedTexture, decode_texture_object, save_decoded_texture};
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

/// Helper: parse a bundle and decode all Texture2D objects into a HashMap.
fn decode_all_textures(bundle_path: &str) -> HashMap<String, DecodedTexture> {
    let data = match std::fs::read(bundle_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return HashMap::new();
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

    let mut textures = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            continue;
        }
        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };
        for obj in &sf.objects {
            if obj.class_id != 28 {
                continue;
            }
            let val = match read_object(&sf, obj) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Ok(Some(tex)) = decode_texture_object(&val, &resources) {
                textures.insert(tex.name.clone(), tex);
            }
        }
    }
    textures
}

fn test_output_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(format!("{}/test_output/{name}", env!("CARGO_MANIFEST_DIR")));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_alpha_merge_kalts_bundle() {
    let path = format!(
        "{}/tests/assets/chararts_char_003_kalts.ab",
        env!("CARGO_MANIFEST_DIR")
    );
    let textures = decode_all_textures(&path);
    if textures.is_empty() {
        eprintln!("skip: no textures decoded (asset file may be missing)");
        return;
    }

    // Verify we have the expected alpha pairs in the raw data
    let alpha_names: Vec<&String> = textures.keys().filter(|n| n.ends_with("[alpha]")).collect();
    assert!(
        !alpha_names.is_empty(),
        "bundle should contain [alpha] textures, got names: {:?}",
        textures.keys().collect::<Vec<_>>()
    );

    let dir = test_output_dir("alpha_merge_kalts");
    let count = merge_and_export(textures, &dir);
    assert!(count > 0, "should export at least one texture");

    // Check that merged files exist for known pairs
    for base in &["char_003_kalts", "char_003_kalts_1", "char_003_kalts_1b"] {
        let merged_path = dir.join(format!("{base}.png"));
        let alpha_path = dir.join(format!("{base}[alpha].png"));

        assert!(merged_path.exists(), "merged {base}.png should exist");
        assert!(alpha_path.exists(), "{base}[alpha].png should be preserved");

        // Verify merged image has proper alpha channel (not all 255)
        let img = image::open(&merged_path).unwrap().to_rgba8();
        let has_transparent = img.pixels().any(|p| p.0[3] < 255);
        let has_opaque = img.pixels().any(|p| p.0[3] > 0);
        assert!(
            has_transparent,
            "merged {base}.png should have some transparent pixels"
        );
        assert!(
            has_opaque,
            "merged {base}.png should have some opaque pixels"
        );
    }

    // Textures without alpha pair should still be exported
    let unpaired_path = dir.join("char_003_kalts_2.png");
    assert!(
        unpaired_path.exists(),
        "unpaired texture char_003_kalts_2.png should be exported as-is"
    );
}

#[test]
fn test_no_merge_exports_all_raw() {
    let path = format!(
        "{}/tests/assets/chararts_char_003_kalts.ab",
        env!("CARGO_MANIFEST_DIR")
    );
    let textures = decode_all_textures(&path);
    if textures.is_empty() {
        eprintln!("skip: no textures decoded");
        return;
    }

    let dir = test_output_dir("alpha_no_merge_kalts");

    // Export without merging (simulates --no-merge)
    let mut count = 0;
    for tex in textures.values() {
        if save_decoded_texture(tex, &dir).is_ok() {
            count += 1;
        }
    }
    assert!(count > 0);

    // Both base and alpha files should exist independently
    assert!(
        dir.join("char_003_kalts.png").exists(),
        "base texture should exist"
    );
    assert!(
        dir.join("char_003_kalts[alpha].png").exists(),
        "alpha texture should exist separately"
    );
}
