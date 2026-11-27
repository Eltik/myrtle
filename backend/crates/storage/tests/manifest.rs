//! `Manifest` parsing, stub counts and the store check.

use std::collections::BTreeMap;

use myrtle_storage::sync::{MANIFEST_VERSION, Manifest, ManifestRecord};

#[test]
fn evicted_count_counts_marked_records_and_v1_rows_parse() {
    let raw = br#"{"version":2,"objects":{
        "a/en/x.png":{"size":1,"md5":"m","etag":"e","evicted":true},
        "a/en/y.png":{"size":2,"md5":"m","etag":"e","evicted":false},
        "a/en/z.png":{"size":3,"md5":"m","etag":"e"}}}"#;
    let m: Manifest = serde_json::from_slice(raw).unwrap();
    assert_eq!(m.evicted_count(), 1);
    assert_eq!(Manifest::default().evicted_count(), 0);
    // `evicted: false` is left out, so an unevicted row reads as in v1.
    let row = serde_json::to_string(&m.objects["a/en/y.png"]).unwrap();
    assert_eq!(row, r#"{"size":2,"md5":"m","etag":"e"}"#);
    assert_eq!((m.bucket.as_str(), m.prefix.as_str()), ("", ""));
}

#[test]
fn evicted_outside_roots_counts_stubs_the_store_would_not_serve() {
    let raw = br#"{"version":2,"bucket":"b","prefix":"a","objects":{
        "a/en/textures/x.png":{"size":1,"md5":"m","etag":"e","evicted":true},
        "a/en/spine/y.json":{"size":1,"md5":"m","etag":"e","evicted":true},
        "a/en/video/z.mp4":{"size":1,"md5":"m","etag":"e","evicted":true},
        "a/en/video/kept.mp4":{"size":1,"md5":"m","etag":"e"},
        "old/en/textures/w.png":{"size":1,"md5":"m","etag":"e","evicted":true},
        "a/en/loose.png":{"size":1,"md5":"m","etag":"e","evicted":true}}}"#;
    let m: Manifest = serde_json::from_slice(raw).unwrap();
    let roots = ["textures".to_owned(), "spine".to_owned()];
    // video/z.mp4 (root not served), old/... (other prefix), loose.png (no root).
    assert_eq!(m.evicted_outside_roots("a/en/", &roots), 3);
    assert_eq!(m.evicted_outside_roots("a/en/", &[]), 5);
    assert_eq!(
        Manifest::default().evicted_outside_roots("a/en/", &roots),
        0
    );
}

#[test]
fn check_store_only_refuses_with_stubs_present() {
    let mut m = Manifest {
        version: MANIFEST_VERSION,
        bucket: "b".into(),
        prefix: "a".into(),
        objects: BTreeMap::new(),
    };
    let rec = |evicted| ManifestRecord {
        size: 1,
        md5: "m".into(),
        etag: "e".into(),
        evicted,
    };
    m.objects.insert("a/en/textures/x.png".into(), rec(false));
    assert!(m.check_store("other", "z", "en").is_ok());
    m.objects.insert("a/en/textures/y.png".into(), rec(true));
    assert!(m.check_store("b", "a", "en").is_ok());
    assert!(m.check_store("other", "a", "en").is_err());
    assert!(m.check_store("b", "z", "en").is_err());
    // An unrecorded bucket is unknown and passes; the keys still guard the prefix.
    m.bucket.clear();
    m.prefix.clear();
    assert!(m.check_store("any", "a", "en").is_ok());
    assert!(m.check_store("any", "z", "en").is_err());
}
