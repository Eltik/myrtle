use std::path::PathBuf;

fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("samples")
}

#[test]
fn test_extractor() {
    use std::fs;
    use unity_rs::tools::extract_assets;

    // Create output directory in current directory
    let output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("extracted_assets");

    // Clean up any existing output directory
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap();
    }

    // Create fresh output directory
    fs::create_dir_all(&output_dir).unwrap();

    let extracted = extract_assets(samples_dir(), &output_dir, true).unwrap();

    println!("extract_assets reported {} files extracted", extracted);
    println!("Output directory: {}", output_dir.display());

    let mut file_count = 0;
    for entry in walkdir::WalkDir::new(&output_dir) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            println!("  Found file: {}", entry.path().display());
            file_count += 1;
        }
    }

    println!("Extracted {} files", file_count);

    // Note: The test samples contain 53 total objects (46 in container):
    // - 35 AudioClips ✓ (successfully extracted from internal CAB resources)
    // - 9 Sprites ✓ (all 9 successfully extracted with SpriteAtlas support)
    // - 4 AssetBundles (metadata, not exportable)
    // - 3 Texture2D (have StreamData references, exported via Sprites)
    // - 1 SpriteAtlas ✓ (packed texture exported to atlases directory)
    // - 1 Mesh ✓ (successfully extracted)
    //
    // Successfully extracted: 46 files (35 AudioClips + 9 Sprites + 1 SpriteAtlas + 1 Mesh)
    // SpriteAtlas support: 7 watertower sprites load textures from the packed atlas
    assert!(
        file_count >= 46,
        "Expected at least 46 files, got {}",
        file_count
    );
}
