use std::path::PathBuf;
use unity_rs::files::bundle_file::FileType;
use unity_rs::helpers::import_helper::FileSource;
use unity_rs::ClassIDType;
use unity_rs::Environment;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("samples")
}

#[test]
fn test_debug_sprite_json() {
    let sample_files: Vec<PathBuf> = std::fs::read_dir(samples_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    for file_path in sample_files {
        if !file_path.is_file() {
            continue;
        }

        println!("Loading file: {}", file_path.display());
        let mut env = Environment::new();
        env.load_file(
            FileSource::Path(file_path.to_string_lossy().to_string()),
            None,
            false,
        );

        println!("Files loaded: {}", env.files.len());
        for (_name, file_rc) in &env.files {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(serialized_file_rc) => {
                    let serialized_file = serialized_file_rc.borrow();
                    println!(
                        "Found SerializedFile with {} objects",
                        serialized_file.objects.len()
                    );
                    for obj in serialized_file.objects.values() {
                        println!("Object type: {:?}", obj.obj_type);
                        if obj.obj_type == ClassIDType::Sprite {
                            println!("\n=== Found Sprite ===");
                            println!("Path ID: {}", obj.path_id);

                            let mut reader = obj.clone();
                            match reader.read(false) {
                                Ok(json) => {
                                    println!(
                                        "JSON (pretty):\n{}",
                                        serde_json::to_string_pretty(&json).unwrap()
                                    );
                                    return; // Just show the first one
                                }
                                Err(e) => {
                                    println!("Error reading: {}", e);
                                }
                            }
                        }
                    }
                }
                FileType::BundleFile(bundle) => {
                    println!("Found BundleFile with {} files", bundle.files.len());
                    for (_bundle_name, bundle_file_rc) in &bundle.files {
                        let bundle_file_ref = bundle_file_rc.borrow();
                        if let FileType::SerializedFile(serialized_file_rc) = &*bundle_file_ref {
                            let serialized_file = serialized_file_rc.borrow();
                            println!(
                                "  Found SerializedFile in bundle with {} objects",
                                serialized_file.objects.len()
                            );
                            for obj in serialized_file.objects.values() {
                                if obj.obj_type == ClassIDType::Sprite {
                                    println!("\n=== Found Sprite ===");
                                    println!("Path ID: {}", obj.path_id);

                                    let mut reader = obj.clone();
                                    match reader.read(false) {
                                        Ok(json) => {
                                            println!(
                                                "JSON (pretty):\n{}",
                                                serde_json::to_string_pretty(&json).unwrap()
                                            );
                                            return; // Just show the first one
                                        }
                                        Err(e) => {
                                            println!("Error reading: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    println!("Skipping unknown FileType");
                }
            }
        }
    }
}

#[test]
fn test_debug_audioclip_json() {
    let sample_files: Vec<PathBuf> = std::fs::read_dir(samples_dir())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    for file_path in sample_files {
        if !file_path.is_file() {
            continue;
        }

        let mut env = Environment::new();
        env.load_file(
            FileSource::Path(file_path.to_string_lossy().to_string()),
            None,
            false,
        );

        for (_name, file_rc) in &env.files {
            let file_ref = file_rc.borrow();
            match &*file_ref {
                FileType::SerializedFile(serialized_file_rc) => {
                    let serialized_file = serialized_file_rc.borrow();
                    for obj in serialized_file.objects.values() {
                        if obj.obj_type == ClassIDType::AudioClip {
                            println!("\n=== Found AudioClip ===");
                            println!("Path ID: {}", obj.path_id);

                            let mut reader = obj.clone();
                            match reader.read(false) {
                                Ok(json) => {
                                    println!(
                                        "JSON (pretty):\n{}",
                                        serde_json::to_string_pretty(&json).unwrap()
                                    );
                                    return; // Just show the first one
                                }
                                Err(e) => {
                                    println!("Error reading: {}", e);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
