use downloader::manifest::Manifest;
use downloader::types::HotFile;
use std::fs;

fn make_file(name: &str, md5: &str) -> HotFile {
    HotFile {
        name: name.to_string(),
        total_size: 100,
        md5: md5.to_string(),
    }
}

#[test]
fn load_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::load(dir.path()).unwrap();

    let files = vec![make_file("a.ab", "abc123")];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 1);
}

#[test]
fn load_existing() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("persistent_res_list.json");
    fs::write(&manifest_path, r#"{"a.ab": "abc123", "b.ab": "def456"}"#).unwrap();

    let manifest = Manifest::load(dir.path()).unwrap();
    let files = vec![make_file("a.ab", "abc123"), make_file("c.ab", "ghi789")];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 1);
    assert_eq!(needed[0].name, "c.ab");
}

#[test]
fn filter_needed_all_new() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = Manifest::load(dir.path()).unwrap();

    let files = vec![
        make_file("a.ab", "aaa"),
        make_file("b.ab", "bbb"),
        make_file("c.ab", "ccc"),
    ];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 3);
}

#[test]
fn filter_needed_all_cached() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("persistent_res_list.json");
    fs::write(&manifest_path, r#"{"a.ab": "aaa", "b.ab": "bbb"}"#).unwrap();

    let manifest = Manifest::load(dir.path()).unwrap();
    let files = vec![make_file("a.ab", "aaa"), make_file("b.ab", "bbb")];
    let needed = manifest.filter_needed(&files);
    assert!(needed.is_empty());
}

#[test]
fn filter_needed_md5_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("persistent_res_list.json");
    fs::write(&manifest_path, r#"{"a.ab": "old_md5"}"#).unwrap();

    let manifest = Manifest::load(dir.path()).unwrap();
    let files = vec![make_file("a.ab", "new_md5")];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 1);
    assert_eq!(needed[0].md5, "new_md5");
}

#[test]
fn filter_needed_mixed() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("persistent_res_list.json");
    fs::write(
        &manifest_path,
        r#"{"cached.ab": "match", "stale.ab": "old"}"#,
    )
    .unwrap();

    let manifest = Manifest::load(dir.path()).unwrap();
    let files = vec![
        make_file("cached.ab", "match"),
        make_file("stale.ab", "new"),
        make_file("fresh.ab", "brand_new"),
    ];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 2);

    let names: Vec<&str> = needed.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"stale.ab"));
    assert!(names.contains(&"fresh.ab"));
}

#[test]
fn update_and_save_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = Manifest::load(dir.path()).unwrap();

    manifest.update("test.ab", "abc123");
    manifest.update("test2.ab", "def456");
    manifest.save().unwrap();

    // Reload and verify
    let reloaded = Manifest::load(dir.path()).unwrap();
    let files = vec![
        make_file("test.ab", "abc123"),
        make_file("test2.ab", "def456"),
    ];
    let needed = reloaded.filter_needed(&files);
    assert!(needed.is_empty(), "saved entries should match");

    // Verify a file with different md5 IS needed
    let files2 = vec![make_file("test.ab", "changed")];
    let needed2 = reloaded.filter_needed(&files2);
    assert_eq!(needed2.len(), 1);
}

#[test]
fn load_corrupted_json() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("persistent_res_list.json");
    fs::write(&manifest_path, "this is not valid json!!!").unwrap();

    let manifest = Manifest::load(dir.path()).unwrap();
    // Should default to empty entries
    let files = vec![make_file("a.ab", "aaa")];
    let needed = manifest.filter_needed(&files);
    assert_eq!(needed.len(), 1);
}
