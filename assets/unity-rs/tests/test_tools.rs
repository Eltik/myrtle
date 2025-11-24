/// Tools module tests matching Python's tools functionality
///
/// These tests verify asset extraction functionality.
use std::path::PathBuf;
use unity_rs::tools::extract_assets;

/// Get the path to test samples directory
fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("samples")
}

/// Test 1: Basic asset extraction with container organization
#[test]
fn test_extract_assets_with_container() {
    let samples = samples_dir();
    let test_file = samples.join("char_118_yuki.ab");

    if !test_file.exists() {
        eprintln!("Test file not found: {:?}", test_file);
        return;
    }

    // Create temporary output directory
    let output_dir = std::env::temp_dir().join("unity_rs_test_extract");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).ok();
    }
    std::fs::create_dir_all(&output_dir).unwrap();

    // Extract assets with container organization
    let result = extract_assets(&test_file, &output_dir, true);

    assert!(
        result.is_ok(),
        "Failed to extract assets: {:?}",
        result.err()
    );

    let count = result.unwrap();
    println!("Extracted {} files to {:?}", count, output_dir);

    // Cleanup
    std::fs::remove_dir_all(&output_dir).ok();
}

/// Test 2: Asset extraction without container organization
#[test]
fn test_extract_assets_without_container() {
    let samples = samples_dir();
    let test_file = samples.join("char_118_yuki.ab");

    if !test_file.exists() {
        eprintln!("Test file not found: {:?}", test_file);
        return;
    }

    // Create temporary output directory
    let output_dir = std::env::temp_dir().join("unity_rs_test_extract_no_container");
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).ok();
    }
    std::fs::create_dir_all(&output_dir).unwrap();

    // Extract assets without container organization
    let result = extract_assets(&test_file, &output_dir, false);

    assert!(
        result.is_ok(),
        "Failed to extract assets: {:?}",
        result.err()
    );

    let count = result.unwrap();
    println!("Extracted {} files to {:?}", count, output_dir);

    // Cleanup
    std::fs::remove_dir_all(&output_dir).ok();
}

/// Test 3: Extract from multiple sample files
#[test]
#[ignore] // Slow test - only run when needed
fn test_extract_all_samples() {
    let samples = samples_dir();

    if !samples.exists() {
        eprintln!("Samples directory not found: {:?}", samples);
        return;
    }

    // Get all Unity files in samples
    let unity_files: Vec<_> = std::fs::read_dir(&samples)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file()
                && (path.extension().map(|e| e == "unity3d").unwrap_or(false)
                    || path.extension().map(|e| e == "ab").unwrap_or(false))
        })
        .collect();

    println!("Found {} Unity files in samples", unity_files.len());

    for entry in unity_files {
        let path = entry.path();
        println!("Testing extraction from: {:?}", path.file_name());

        let output_dir = std::env::temp_dir()
            .join("unity_rs_test_extract_all")
            .join(path.file_name().unwrap());

        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir).ok();
        }
        std::fs::create_dir_all(&output_dir).unwrap();

        let result = extract_assets(&path, &output_dir, true);

        if let Ok(count) = result {
            println!("  Extracted {} files", count);
        } else {
            eprintln!("  Failed: {:?}", result.err());
        }

        // Cleanup
        std::fs::remove_dir_all(&output_dir).ok();
    }
}

/// Test 4: Verify MONOBEHAVIOUR_TYPETREES storage is accessible
#[test]
fn test_monobehaviour_typetrees_storage() {
    use unity_rs::tools::MONOBEHAVIOUR_TYPETREES;

    // Verify we can access the global storage
    let storage = MONOBEHAVIOUR_TYPETREES.read().unwrap();

    // Should be empty initially
    assert_eq!(
        storage.len(),
        0,
        "MONOBEHAVIOUR_TYPETREES should be empty initially"
    );
}
#[test]
fn test_extract_debug() {
    use unity_rs::files::bundle_file::FileType;
    use unity_rs::helpers::import_helper::FileSource;
    use unity_rs::Environment;

    let mut env = Environment::new();
    env.load_file(
        FileSource::Path("tests/fixtures/samples/char_118_yuki.ab".to_string()),
        None,
        false,
    );

    println!("=== Checking container ===");
    let container = env.container();
    println!("Container has {} entries", container.len());

    for (path, obj) in container.iter().take(5) {
        println!("  {} - {:?}", path, obj.obj_type);
    }

    println!("\n=== Checking files directly ===");
    for (name, file_rc) in &env.files {
        let file_ref = file_rc.borrow();
        println!("File: {}", name);
        match &*file_ref {
            FileType::SerializedFile(sf_rc) => {
                let sf = sf_rc.borrow();
                println!("  SerializedFile with {} objects", sf.objects.len());
                for (_id, obj) in sf.objects.iter().take(5) {
                    println!("    {:?} - path_id: {}", obj.obj_type, obj.path_id);
                }
            }
            FileType::BundleFile(bundle) => {
                println!("  BundleFile with {} files", bundle.files.len());
                for (bundle_name, bundle_file_rc) in bundle.files.iter().take(3) {
                    println!("    Bundle file: {}", bundle_name);
                    let bundle_file_ref = bundle_file_rc.borrow();
                    if let FileType::SerializedFile(sf_rc) = &*bundle_file_ref {
                        let sf = sf_rc.borrow();
                        println!("      SerializedFile with {} objects", sf.objects.len());
                        for (_id, obj) in sf.objects.iter().take(3) {
                            println!("        {:?} - path_id: {}", obj.obj_type, obj.path_id);
                        }
                    }
                }
            }
            _ => println!("  Other file type"),
        }
    }
}
