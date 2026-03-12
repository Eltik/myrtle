use std::path::PathBuf;
use unpacker::export::gamedata::export_gamedata;

#[test]
fn test_export_gamedata() {
    let base = env!("CARGO_MANIFEST_DIR");
    let bundle_dir = PathBuf::from(format!("{base}/tests/assets/anon"));
    let idx_path = PathBuf::from(format!("{base}/tests/assets/manifest.idx"));

    if !bundle_dir.exists() || !idx_path.exists() {
        eprintln!("skip: anon dir or idx file not found");
        return;
    }

    let output_dir = PathBuf::from(format!("{base}/test_output/gamedata"));
    std::fs::create_dir_all(&output_dir).unwrap();

    match export_gamedata(&bundle_dir, &idx_path, &output_dir) {
        Ok(count) => {
            assert!(count > 0, "should export at least one gamedata file");
        }
        Err(e) => panic!("export_gamedata failed: {e}"),
    }
}
