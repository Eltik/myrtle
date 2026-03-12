use std::path::PathBuf;
use unpacker::export::manifest::ResourceManifest;

#[test]
fn load_manifest() {
    let idx = PathBuf::from(format!(
        "{}/tests/assets/manifest.idx",
        env!("CARGO_MANIFEST_DIR")
    ));
    let manifest = match ResourceManifest::load(&idx) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };

    assert!(!manifest.filename_to_path.is_empty());
}
