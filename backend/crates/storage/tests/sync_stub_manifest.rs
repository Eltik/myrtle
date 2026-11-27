//! `run_sync` and evicted stubs whose manifest is gone.

use std::fs;
use std::path::Path;
use std::time::Duration;

use myrtle_storage::evict::EvictPolicy;
use myrtle_storage::providers::local::LocalStore;
use myrtle_storage::store::ObjectStore;
use myrtle_storage::sync::{EMPTY_OVER_OBJECT, MANIFEST_NAME, SyncOptions, SyncOutcome, run_sync};

fn opts(root: &Path, evict: bool) -> SyncOptions {
    SyncOptions {
        root: root.to_path_buf(),
        server: "en".into(),
        prefix: String::new(),
        roots: vec!["textures".into()],
        backup_roots: vec![],
        delete: true,
        dry_run: false,
        allow_shrink: false,
        concurrency: 2,
        hash_threads: 2,
        multipart_threshold: u64::MAX,
        part_size: 8 << 20,
        cache_control: "public, max-age=60".into(),
        retry_base: Duration::from_millis(1),
        bucket: "b".into(),
        evict: EvictPolicy {
            enabled: evict,
            extra_keep: vec![],
        },
    }
}

#[tokio::test]
async fn a_stub_without_its_manifest_is_not_uploaded_over_the_object() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("output");
    let file = root.join("en/textures/chars/a.png");
    fs::create_dir_all(file.parent().expect("parent")).expect("mkdir");
    fs::write(&file, vec![7u8; 4096]).expect("write");
    let store = LocalStore::new(tmp.path().join("bucket"));
    let key = "en/textures/chars/a.png";

    let first = run_sync(&store, &opts(&root, true)).await.expect("sync");
    assert_eq!((first.uploaded, first.evicted), (1, 1));
    assert_eq!(fs::metadata(&file).expect("stat").len(), 0);

    let second = run_sync(&store, &opts(&root, true)).await.expect("sync");
    assert_eq!((second.stubs, second.uploaded), (1, 0));

    fs::remove_file(root.join("en").join(MANIFEST_NAME)).expect("rm manifest");
    let third = run_sync(&store, &opts(&root, false)).await.expect("sync");
    assert_eq!(third.uploaded, 0);
    assert_eq!(third.outcome, SyncOutcome::Failed);
    assert!(third.failures[0].contains(EMPTY_OVER_OBJECT));
    let head = store.head(key).await.expect("head").expect("object");
    assert_eq!(head.size, 4096, "the bucket keeps the only full copy");
}
