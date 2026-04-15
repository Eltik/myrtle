use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use unpacker::export::spine::{SpineCategory, collect_spine_assets};
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

type ObjectMap = HashMap<i64, (i32, Value)>;
type ResourceMap = HashMap<String, Vec<u8>>;

fn load_all_objects(bundle_path: &Path) -> (ObjectMap, ResourceMap) {
    let data = std::fs::read(bundle_path).expect("failed to read bundle");
    let bundle = BundleFile::parse(data).expect("failed to parse bundle");

    let mut all_objects = HashMap::new();
    let mut resources = HashMap::new();

    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
            continue;
        }
        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };
        for obj in &sf.objects {
            if let Ok(val) = read_object(&sf, obj) {
                all_objects.insert(obj.path_id, (obj.class_id, val));
            }
        }
    }

    (all_objects, resources)
}

#[test]
fn collect_spine_via_monobehaviour() {
    let bundle_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets/chararts_char_002_amiya.ab");

    if !bundle_path.exists() {
        eprintln!("SKIP: bundle not found at {}", bundle_path.display());
        return;
    }

    let (all_objects, _resources) = load_all_objects(&bundle_path);
    let (spine_assets, claimed) = collect_spine_assets(&all_objects);

    assert!(
        spine_assets.len() >= 3,
        "char_002_amiya should have 3 spine instances, got {}",
        spine_assets.len()
    );
    assert!(!claimed.is_empty());

    let categories: Vec<_> = spine_assets.iter().map(|sa| sa.category).collect();
    assert!(
        categories.contains(&SpineCategory::BattleFront),
        "should have BattleFront"
    );
    assert!(
        categories.contains(&SpineCategory::Building),
        "should have Building"
    );
}

#[test]
fn debug_material_textures() {
    let bundle_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/assets/chararts_char_002_amiya.ab");

    if !bundle_path.exists() {
        eprintln!("SKIP: bundle not found at {}", bundle_path.display());
        return;
    }

    let (all_objects, _resources) = load_all_objects(&bundle_path);

    // Verify materials exist in the bundle
    let materials: Vec<_> = all_objects
        .iter()
        .filter(|(_, (cid, _))| *cid == 21)
        .collect();
    assert!(
        !materials.is_empty(),
        "should have Material objects in bundle"
    );
}
