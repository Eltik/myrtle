//! The sync engine against `LocalStore`, a directory standing in for the
//! bucket.

use std::path::{Path, PathBuf};
use std::time::Duration;

use myrtle_storage::ObjectStore;
use myrtle_storage::providers::local::LocalStore;
use myrtle_storage::restore::{RestoreOptions, run_restore};
use myrtle_storage::sync::{
    LOCK_NAME, MANIFEST_VERSION, Manifest, STUB_CHANGED, STUB_GONE, SyncError, SyncOptions,
    SyncOutcome, SyncReport, run_sync,
};
use myrtle_storage::{EvictPolicy, PutBody, PutObject, StorageConfig};

struct Fixture {
    _dir: tempfile::TempDir,
    output: PathBuf,
    store: LocalStore,
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("output");
        let en = output.join("en");
        write(&en.join("textures/a.png"), b"png-a");
        write(&en.join("textures/sub/b.webp"), b"webp-b");
        write(&en.join("spine/DynIllust/x/[scene].json"), b"{\"scene\":1}");
        write(&en.join("audio/m.m4a"), b"m4a-bytes");
        write(&en.join("audio/s.ogg"), b"ogg-bytes");
        // Skipped: wrong extension, dotfile, temp file, outside the roots.
        write(&en.join("textures/c.bin"), b"bin");
        write(&en.join("textures/.DS_Store"), b"ds");
        write(&en.join("textures/d.png.tmp"), b"tmp");
        write(&en.join("textures/.hidden/e.png"), b"hidden");
        write(&en.join("gamedata/excel/x.json"), b"{}");
        let store = LocalStore::new(dir.path().join("bucket"));
        Self {
            _dir: dir,
            output,
            store,
        }
    }

    fn opts(&self) -> SyncOptions {
        SyncOptions {
            root: self.output.clone(),
            server: "en".into(),
            prefix: "assets".into(),
            roots: ["textures", "spine", "audio", "video", "portraits"]
                .iter()
                .map(|r| (*r).to_owned())
                .collect(),
            delete: true,
            dry_run: false,
            allow_shrink: false,
            concurrency: 4,
            hash_threads: 2,
            multipart_threshold: 1 << 30,
            part_size: 1 << 20,
            cache_control: "public, max-age=604800".into(),
            retry_base: Duration::from_millis(1),
            bucket: "bucket-a".into(),
            evict: EvictPolicy::default(),
        }
    }

    fn evicting(&self) -> SyncOptions {
        let mut opts = self.opts();
        opts.evict.enabled = true;
        opts
    }

    fn restore_opts(&self) -> RestoreOptions {
        let mut opts = RestoreOptions::new(
            self.output.clone(),
            "en".into(),
            "bucket-a".into(),
            "assets".into(),
            4,
        );
        opts.retry_base = Duration::from_millis(1);
        opts
    }

    fn len(&self, rel: &str) -> u64 {
        std::fs::metadata(self.en(rel)).unwrap().len()
    }

    fn manifest(&self) -> Manifest {
        Manifest::read(&Manifest::path(&self.output, "en")).unwrap()
    }

    /// Every file under `en/` except the manifest, as (rel, bytes), sorted.
    fn tree(&self) -> Vec<(String, Vec<u8>)> {
        let en = self.output.join("en");
        let mut files: Vec<(String, Vec<u8>)> = walkdir::WalkDir::new(&en)
            .into_iter()
            .map(Result::unwrap)
            .filter(|e| {
                e.file_type().is_file()
                    && e.file_name() != ".asset-sync.json"
                    && e.file_name() != ".asset-sync.lock"
            })
            .map(|e| {
                let rel = e.path().strip_prefix(&en).unwrap().display().to_string();
                (rel, std::fs::read(e.path()).unwrap())
            })
            .collect();
        files.sort();
        files
    }

    fn en(&self, rel: &str) -> PathBuf {
        self.output.join("en").join(rel)
    }

    async fn run(&self, opts: &SyncOptions) -> SyncReport {
        run_sync(&self.store, opts).await.unwrap()
    }

    async fn keys(&self) -> Vec<String> {
        use futures::StreamExt;
        let mut keys: Vec<String> = self.store.list("").map(|m| m.unwrap().key).collect().await;
        keys.sort();
        keys
    }
}

const SEEDED: &[&str] = &[
    "assets/en/audio/m.m4a",
    "assets/en/audio/s.ogg",
    "assets/en/spine/DynIllust/x/[scene].json",
    "assets/en/textures/a.png",
    "assets/en/textures/sub/b.webp",
];

#[tokio::test]
async fn initial_seed_uploads_everything_with_headers() {
    let f = Fixture::new();
    let report = f.run(&f.opts()).await;
    assert_eq!(report.scanned, 5);
    assert_eq!(report.uploaded, 5);
    assert_eq!(report.unchanged, 0);
    assert_eq!(report.deleted, 0);
    assert_eq!(report.failed, 0);
    // c.bin and d.png.tmp; dot entries are never entered, so not counted.
    assert_eq!(report.skipped, 2);
    assert_eq!(report.uploaded_bytes, 5 + 6 + 11 + 9 + 9);
    assert_eq!(report.outcome, SyncOutcome::Ok);
    assert_eq!(report.exit_code(), 0);
    assert_eq!(f.keys().await, SEEDED);

    let m4a = f.store.meta("assets/en/audio/m.m4a").await.unwrap();
    assert_eq!(m4a.content_type, "audio/mp4");
    assert_eq!(m4a.cache_control.as_deref(), Some("public, max-age=604800"));
    assert_eq!(m4a.etag, hex_md5(b"m4a-bytes"));
    let scene = f
        .store
        .meta("assets/en/spine/DynIllust/x/[scene].json")
        .await
        .unwrap();
    assert_eq!(scene.content_type, "application/json");
    assert_eq!(
        f.store
            .meta("assets/en/textures/a.png")
            .await
            .unwrap()
            .content_type,
        "image/png"
    );

    let line = report.summary_line();
    assert!(
        line.starts_with("asset-sync: server=en scanned=5 unchanged=0 uploaded=5 (40 bytes) deleted=0 failed=0 elapsed="),
        "{line}"
    );
    assert!(Manifest::path(&f.output, "en").is_file());
}

fn hex_md5(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    hex::encode(Md5::digest(bytes))
}

#[tokio::test]
async fn a_second_run_is_all_unchanged() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    // Every extract rewrites every file: same bytes, new mtime.
    write(&f.en("textures/a.png"), b"png-a");
    let report = f.run(&f.opts()).await;
    assert_eq!(report.unchanged, 5);
    assert_eq!(report.uploaded, 0);
    assert_eq!(report.deleted, 0);
    assert!(!report.drifted());
}

#[tokio::test]
async fn a_changed_byte_reuploads_that_file_only() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    write(&f.en("textures/a.png"), b"png-A");
    let report = f.run(&f.opts()).await;
    assert_eq!(report.uploaded, 1);
    assert_eq!(report.unchanged, 4);
    assert_eq!(
        f.store.meta("assets/en/textures/a.png").await.unwrap().etag,
        hex_md5(b"png-A")
    );
}

#[tokio::test]
async fn a_removed_file_deletes_its_key() {
    let f = Fixture::new();
    // Ten more files so one removal stays above the 90% floor.
    for i in 0..10 {
        write(
            &f.en(&format!("portraits/p{i}.png")),
            format!("p{i}").as_bytes(),
        );
    }
    f.run(&f.opts()).await;
    std::fs::remove_file(f.en("textures/sub/b.webp")).unwrap();
    let report = f.run(&f.opts()).await;
    assert_eq!(report.deleted, 1);
    assert_eq!(report.outcome, SyncOutcome::Ok);
    assert!(
        !f.keys()
            .await
            .contains(&"assets/en/textures/sub/b.webp".to_owned())
    );
}

#[tokio::test]
async fn shrinking_below_90_percent_refuses_deletes_unless_allowed() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    std::fs::remove_file(f.en("textures/sub/b.webp")).unwrap();
    std::fs::remove_file(f.en("audio/s.ogg")).unwrap();

    // 3 local against 5 remote is 60%.
    let refused = f.run(&f.opts()).await;
    assert_eq!(
        refused.outcome,
        SyncOutcome::ShrinkRefused {
            local: 3,
            remote: 5
        }
    );
    assert_eq!(refused.exit_code(), 3);
    assert_eq!(refused.deleted, 0);
    assert_eq!(f.keys().await.len(), 5);

    let mut opts = f.opts();
    opts.allow_shrink = true;
    let allowed = f.run(&opts).await;
    assert_eq!(allowed.deleted, 2);
    assert_eq!(allowed.exit_code(), 0);
    assert_eq!(f.keys().await.len(), 3);
}

#[tokio::test]
async fn no_delete_skips_deletes_and_the_shrink_check() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    std::fs::remove_file(f.en("audio/s.ogg")).unwrap();
    let mut opts = f.opts();
    opts.delete = false;
    let report = f.run(&opts).await;
    assert_eq!(report.deleted, 0);
    assert_eq!(report.exit_code(), 0);
    assert_eq!(f.keys().await.len(), 5);
}

#[tokio::test]
async fn a_missing_root_keeps_its_remote_keys() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    std::fs::remove_dir_all(f.en("spine")).unwrap();
    let report = f.run(&f.opts()).await;
    assert_eq!(report.missing_roots, ["spine", "video", "portraits"]);
    assert_eq!(report.deleted, 0);
    assert_eq!(report.outcome, SyncOutcome::Ok);
    assert!(
        f.keys()
            .await
            .contains(&"assets/en/spine/DynIllust/x/[scene].json".to_owned())
    );
}

#[tokio::test]
async fn files_outside_the_extension_list_are_skipped() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    let keys = f.keys().await;
    for skipped in [
        "assets/en/textures/c.bin",
        "assets/en/textures/.DS_Store",
        "assets/en/textures/d.png.tmp",
        "assets/en/textures/.hidden/e.png",
        "assets/en/gamedata/excel/x.json",
    ] {
        assert!(!keys.contains(&skipped.to_owned()), "{skipped}");
    }
}

#[tokio::test]
async fn dry_run_changes_nothing() {
    let f = Fixture::new();
    let mut opts = f.opts();
    opts.dry_run = true;
    let report = f.run(&opts).await;
    assert_eq!(report.uploaded, 5);
    assert!(report.drifted());
    assert!(f.keys().await.is_empty());
    assert!(!Manifest::path(&f.output, "en").exists());

    f.run(&f.opts()).await;
    std::fs::remove_file(f.en("audio/s.ogg")).unwrap();
    write(&f.en("textures/a.png"), b"changed");
    // 4 of 5 is under the 90% floor; the dry run still reports the delete.
    opts.allow_shrink = true;
    let report = f.run(&opts).await;
    assert_eq!((report.uploaded, report.deleted), (1, 1));
    assert_eq!(f.keys().await.len(), 5);
    assert_eq!(
        f.store.meta("assets/en/textures/a.png").await.unwrap().etag,
        hex_md5(b"png-a")
    );
}

#[tokio::test]
async fn multipart_objects_compare_through_the_manifest() {
    let f = Fixture::new();
    write(&f.en("video/big.mp4"), &vec![3u8; 3000]);
    let mut opts = f.opts();
    opts.multipart_threshold = 1000;
    opts.part_size = 1000;
    let first = f.run(&opts).await;
    assert_eq!(first.uploaded, 6);
    let etag = f
        .store
        .head("assets/en/video/big.mp4")
        .await
        .unwrap()
        .unwrap()
        .etag
        .unwrap();
    assert!(etag.ends_with("-3"), "{etag}");

    let second = f.run(&opts).await;
    assert_eq!(second.unchanged, 6);

    // Same size, new content: the manifest MD5 no longer matches.
    write(&f.en("video/big.mp4"), &vec![4u8; 3000]);
    assert_eq!(f.run(&opts).await.uploaded, 1);

    // Without its manifest record a multipart object is re-uploaded.
    std::fs::remove_file(Manifest::path(&f.output, "en")).unwrap();
    assert_eq!(f.run(&opts).await.uploaded, 1);
}

#[tokio::test]
async fn keys_outside_the_roots_are_never_deleted() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    f.store
        .put(
            "assets/en/gamedata/x.json",
            PutObject {
                body: PutBody::Bytes(bytes::Bytes::from_static(b"{}")),
                content_type: "application/json".into(),
                cache_control: None,
                content_md5: None,
            },
        )
        .await
        .unwrap();
    let report = f.run(&f.opts()).await;
    assert_eq!(report.deleted, 0);
    assert!(
        f.keys()
            .await
            .contains(&"assets/en/gamedata/x.json".to_owned())
    );
}

#[tokio::test]
async fn a_missing_server_dir_is_an_error() {
    let f = Fixture::new();
    let mut opts = f.opts();
    opts.server = "kr".into();
    assert!(run_sync(&f.store, &opts).await.is_err());
}

#[tokio::test]
async fn eviction_truncates_in_place_and_marks_the_record() {
    let f = Fixture::new();
    let old = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
    std::fs::File::options()
        .write(true)
        .open(f.en("textures/a.png"))
        .unwrap()
        .set_modified(old)
        .unwrap();
    let report = f.run(&f.evicting()).await;
    // The stub keeps the mtime the extract gave it.
    assert_eq!(
        std::fs::metadata(f.en("textures/a.png"))
            .unwrap()
            .modified()
            .unwrap(),
        old
    );
    assert_eq!(
        (report.uploaded, report.evicted, report.evict_failed),
        (5, 5, 0)
    );
    assert_eq!(report.freed_bytes, 40);
    assert_eq!(report.exit_code(), 0);
    let line = report.summary_line();
    assert!(
        line.starts_with(
            "asset-sync: server=en scanned=5 unchanged=0 uploaded=5 (40 bytes) deleted=0 evicted=5 (40 bytes freed) failed=0 elapsed="
        ),
        "{line}"
    );
    for key in SEEDED {
        let rel = key.strip_prefix("assets/en/").unwrap();
        assert_eq!(f.len(rel), 0, "{rel}");
        assert!(f.manifest().objects[*key].evicted, "{key}");
    }
    assert_eq!(f.manifest().version, MANIFEST_VERSION);
    assert_eq!(f.manifest().evicted_count(), 5);
    // The bucket kept the bytes.
    assert_eq!(
        f.store.meta("assets/en/textures/a.png").await.unwrap().etag,
        hex_md5(b"png-a")
    );
}

#[tokio::test]
async fn a_second_sync_counts_stubs_as_unchanged_and_uploads_nothing() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let before = f.manifest().objects;
    let report = f.run(&f.evicting()).await;
    assert_eq!(
        (
            report.unchanged,
            report.stubs,
            report.uploaded,
            report.evicted,
            report.failed
        ),
        (5, 5, 0, 0, 0)
    );
    assert!(!report.drifted());
    assert_eq!(f.manifest().objects, before);
    assert_eq!(
        f.store.meta("assets/en/audio/m.m4a").await.unwrap().etag,
        hex_md5(b"m4a-bytes")
    );
    // With eviction off a stub is still a stub, never an empty upload.
    let report = f.run(&f.opts()).await;
    assert_eq!((report.unchanged, report.stubs, report.uploaded), (5, 5, 0));
}

#[tokio::test]
async fn files_on_the_keep_list_are_never_truncated() {
    let f = Fixture::new();
    write(&f.en("textures/avg/characters/x.png"), b"avg-bytes");
    write(&f.en("audio/m4a_sources.json"), b"{}");
    write(
        &f.en("textures/[[en]]/arts/ui/homebanners/g.png"),
        b"banner",
    );
    let mut opts = f.evicting();
    opts.evict.extra_keep = vec!["spine/".into()];
    let report = f.run(&opts).await;
    assert_eq!(report.uploaded, 8);
    assert_eq!(report.evicted, 4);
    assert_eq!(
        std::fs::read(f.en("textures/[[en]]/arts/ui/homebanners/g.png")).unwrap(),
        b"banner"
    );
    assert_eq!(
        std::fs::read(f.en("textures/avg/characters/x.png")).unwrap(),
        b"avg-bytes"
    );
    assert_eq!(
        std::fs::read(f.en("audio/m4a_sources.json")).unwrap(),
        b"{}"
    );
    assert_eq!(f.len("spine/DynIllust/x/[scene].json"), 11);
    assert_eq!(f.len("textures/a.png"), 0);
    let m = f.manifest();
    assert!(!m.objects["assets/en/textures/avg/characters/x.png"].evicted);
    assert_eq!(m.evicted_count(), 4);
}

#[tokio::test]
async fn an_evicting_dry_run_touches_nothing() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    let tree = f.tree();
    let manifest = std::fs::read(Manifest::path(&f.output, "en")).unwrap();
    let mut opts = f.evicting();
    opts.dry_run = true;
    let report = f.run(&opts).await;
    assert_eq!(
        (report.unchanged, report.evicted, report.freed_bytes),
        (5, 5, 40)
    );
    assert_eq!(f.tree(), tree);
    assert_eq!(
        std::fs::read(Manifest::path(&f.output, "en")).unwrap(),
        manifest
    );
}

#[tokio::test]
async fn a_stub_whose_object_left_the_store_fails() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let gone = f
        .store
        .delete_many(&["assets/en/audio/s.ogg".to_owned()])
        .await;
    assert!(gone[0].1.is_ok());
    let report = f.run(&f.evicting()).await;
    assert_eq!((report.failed, report.stubs, report.uploaded), (1, 4, 0));
    assert_eq!(report.exit_code(), 2);
    assert_eq!(report.failures, [format!("audio/s.ogg: {STUB_GONE}")]);
    assert!(!f.keys().await.contains(&"assets/en/audio/s.ogg".to_owned()));
    // The record survives, so the next run fails again instead of uploading
    // the stub as an empty object.
    assert!(f.manifest().objects["assets/en/audio/s.ogg"].evicted);
    assert_eq!(f.run(&f.opts()).await.failed, 1);
}

#[tokio::test]
async fn a_stub_whose_object_changed_fails() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    f.store
        .put(
            "assets/en/textures/a.png",
            PutObject {
                body: PutBody::Bytes(bytes::Bytes::from_static(b"other")),
                content_type: "image/png".into(),
                cache_control: None,
                content_md5: None,
            },
        )
        .await
        .unwrap();
    let report = f.run(&f.opts()).await;
    assert_eq!(report.failures, [format!("textures/a.png: {STUB_CHANGED}")]);
}

#[tokio::test]
async fn restore_brings_the_bytes_back_and_clears_the_flag() {
    let f = Fixture::new();
    write(&f.en("video/big.mp4"), &vec![7u8; 3000]);
    let mut opts = f.evicting();
    opts.multipart_threshold = 1000;
    opts.part_size = 1000;
    let originals = f.tree();
    f.run(&opts).await;
    assert_eq!(f.len("video/big.mp4"), 0);

    let mut dry = f.restore_opts();
    dry.dry_run = true;
    let report = run_restore(&f.store, &dry).await.unwrap();
    assert_eq!((report.restored, report.restored_bytes), (6, 3040));
    assert_eq!(f.len("video/big.mp4"), 0);

    let mut only = f.restore_opts();
    only.only = vec!["audio/".into()];
    let report = run_restore(&f.store, &only).await.unwrap();
    assert_eq!((report.restored, report.failed), (2, 0));
    assert_eq!(f.manifest().evicted_count(), 4);

    let report = run_restore(&f.store, &f.restore_opts()).await.unwrap();
    assert_eq!(
        (report.restored, report.restored_bytes, report.failed),
        (4, 3022, 0)
    );
    assert_eq!(report.exit_code(), 0);
    assert!(
        report
            .summary_line()
            .starts_with("asset-sync: restore server=en restored=4 (3022 bytes) failed=0 elapsed="),
        "{}",
        report.summary_line()
    );
    assert_eq!(f.tree(), originals);
    assert_eq!(f.manifest().evicted_count(), 0);
    // Nothing left to do, and the restored tree syncs as unchanged.
    assert_eq!(
        run_restore(&f.store, &f.restore_opts())
            .await
            .unwrap()
            .restored,
        0
    );
    let mut plain = opts.clone();
    plain.evict.enabled = false;
    let report = f.run(&plain).await;
    assert_eq!((report.unchanged, report.uploaded, report.stubs), (6, 0, 0));
}

#[tokio::test]
async fn restore_fails_for_an_object_gone_from_the_store() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    f.store
        .delete_many(&["assets/en/textures/a.png".to_owned()])
        .await;
    let report = run_restore(&f.store, &f.restore_opts()).await.unwrap();
    assert_eq!((report.restored, report.failed), (4, 1));
    assert_eq!(report.exit_code(), 1);
    assert_eq!(f.len("textures/a.png"), 0);
    assert!(f.manifest().objects["assets/en/textures/a.png"].evicted);
    let leftovers: Vec<_> = std::fs::read_dir(f.en("textures"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .filter(|n| n.to_string_lossy().ends_with(".tmp") && n != "d.png.tmp")
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[tokio::test]
async fn verify_accepts_stubs() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let mut verify = f.opts();
    verify.dry_run = true;
    let report = f.run(&verify).await;
    assert!(!report.drifted(), "{report:?}");
    assert_eq!(report.stubs, 5);
    // A 0-byte file with no evicted record is drift, as before.
    write(&f.en("portraits/empty.png"), b"");
    assert!(f.run(&verify).await.drifted());
}

#[tokio::test]
async fn a_narrower_run_keeps_the_records_of_other_roots() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let mut opts = f.opts();
    opts.roots = vec!["audio".into()];
    f.run(&opts).await;
    assert_eq!(f.manifest().evicted_count(), 5);
    let report = f.run(&f.opts()).await;
    assert_eq!((report.stubs, report.uploaded), (5, 0));
}

#[tokio::test]
async fn an_unreadable_manifest_stops_the_sync() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    std::fs::write(Manifest::path(&f.output, "en"), b"{not json").unwrap();
    let err = run_sync(&f.store, &f.opts()).await.unwrap_err();
    assert!(err.to_string().contains("evicted stubs"), "{err}");
    assert!(run_restore(&f.store, &f.restore_opts()).await.is_err());
}

#[tokio::test]
async fn a_version_1_manifest_loads_unevicted() {
    let f = Fixture::new();
    f.run(&f.opts()).await;
    let v1 = format!(
        "{{\"version\":1,\"objects\":{{\"assets/en/textures/a.png\":{{\"size\":5,\"md5\":\"{0}\",\"etag\":\"{0}\"}}}}}}",
        hex_md5(b"png-a")
    );
    std::fs::write(Manifest::path(&f.output, "en"), v1).unwrap();
    let m = f.manifest();
    assert_eq!(m.evicted_count(), 0);
    assert!(!m.objects["assets/en/textures/a.png"].evicted);
    // The next run completes it: every unchanged key gets a record.
    let report = f.run(&f.opts()).await;
    assert_eq!(report.unchanged, 5);
    let m = f.manifest();
    assert_eq!((m.version, m.objects.len()), (MANIFEST_VERSION, 5));
}

/// With no eviction variable set, the env path yields eviction off and a
/// sync leaves every local byte as it was.
#[tokio::test]
async fn eviction_is_off_by_default_and_the_tree_is_untouched() {
    let f = Fixture::new();
    let bucket = f.output.parent().unwrap().join("bucket");
    let vars = [
        ("S3_TYPE", "local".to_owned()),
        ("S3_LOCAL_DIR", bucket.display().to_string()),
        ("S3_PREFIX", "assets".to_owned()),
    ];
    let config =
        StorageConfig::from_lookup(&|k| vars.iter().find(|(n, _)| *n == k).map(|(_, v)| v.clone()))
            .unwrap()
            .unwrap();
    assert_eq!(config.evict, EvictPolicy::default());
    let opts = SyncOptions::from_config(&config, f.output.clone(), "en".into());
    let before = f.tree();
    for _ in 0..2 {
        let report = f.run(&opts).await;
        assert_eq!(
            (report.evicted, report.freed_bytes, report.evict_failed),
            (0, 0, 0)
        );
        assert!(
            !report.summary_line().contains("evicted"),
            "{}",
            report.summary_line()
        );
        assert_eq!(f.tree(), before);
    }
    assert_eq!(f.manifest().evicted_count(), 0);
}

/// Holds the server lock the way a second process would: a separate open
/// file description, so the flock conflicts even inside this process.
fn hold_lock(f: &Fixture) -> std::fs::File {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(f.en(LOCK_NAME))
        .unwrap();
    file.try_lock().unwrap();
    file
}

#[tokio::test]
async fn a_second_writer_on_the_same_server_is_refused() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let held = hold_lock(&f);

    let err = run_sync(&f.store, &f.evicting()).await.unwrap_err();
    assert!(
        matches!(&err, SyncError::Locked(p) if p.ends_with(LOCK_NAME)),
        "{err}"
    );
    let msg = err.to_string();
    assert!(msg.contains("another asset-sync is running"), "{msg}");
    assert!(msg.contains(LOCK_NAME), "{msg}");
    let err = run_restore(&f.store, &f.restore_opts()).await.unwrap_err();
    assert!(matches!(err, SyncError::Locked(_)), "{err}");
    // Read-only runs neither take nor need it.
    let mut verify = f.opts();
    verify.dry_run = true;
    assert!(!f.run(&verify).await.drifted());
    // The marks the first run saved are intact.
    assert_eq!(f.manifest().evicted_count(), 5);

    drop(held);
    assert_eq!(f.run(&f.evicting()).await.stubs, 5);
    // The lock file is a dot file and never synced.
    assert!(!f.keys().await.iter().any(|k| k.contains(LOCK_NAME)));
}

#[tokio::test]
async fn a_bucket_or_prefix_change_with_stubs_is_refused() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let m = f.manifest();
    assert_eq!(
        (m.bucket.as_str(), m.prefix.as_str()),
        ("bucket-a", "assets")
    );

    let mut moved = f.opts();
    moved.prefix = "other".into();
    let err = run_sync(&f.store, &moved).await.unwrap_err();
    let msg = err.to_string();
    assert!(matches!(err, SyncError::StoreMoved(_)), "{msg}");
    for part in [
        "S3_PREFIX=assets",
        "S3_PREFIX=other",
        "S3_BUCKET=bucket-a",
        "restore",
    ] {
        assert!(msg.contains(part), "{part}: {msg}");
    }
    let mut restore = f.restore_opts();
    restore.prefix = "other".into();
    assert!(matches!(
        run_restore(&f.store, &restore).await,
        Err(SyncError::StoreMoved(_))
    ));

    let mut other_bucket = f.opts();
    other_bucket.bucket = "bucket-b".into();
    assert!(matches!(
        run_sync(&f.store, &other_bucket).await,
        Err(SyncError::StoreMoved(_))
    ));
    // Nothing was uploaded under the new prefix.
    assert!(!f.keys().await.iter().any(|k| k.starts_with("other/")));

    // A manifest from before the header still refuses, through its keys.
    let path = Manifest::path(&f.output, "en");
    let raw = std::fs::read_to_string(&path).unwrap();
    let legacy = raw
        .replace("\"bucket\": \"bucket-a\",", "")
        .replace("\"prefix\": \"assets\",", "");
    assert_ne!(raw, legacy);
    std::fs::write(&path, legacy).unwrap();
    assert!(matches!(
        run_sync(&f.store, &moved).await,
        Err(SyncError::StoreMoved(_))
    ));

    // Restored first, the move is allowed.
    run_restore(&f.store, &f.restore_opts()).await.unwrap();
    assert_eq!(f.run(&moved).await.uploaded, 5);
}

#[tokio::test]
async fn restore_leaves_a_missing_stub_marked() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    std::fs::remove_file(f.en("audio/s.ogg")).unwrap();
    let report = run_restore(&f.store, &f.restore_opts()).await.unwrap();
    assert_eq!(
        (
            report.restored,
            report.missing,
            report.skipped,
            report.failed
        ),
        (4, 1, 0, 0)
    );
    assert_eq!(report.exit_code(), 0);
    let line = report.summary_line();
    assert!(line.ends_with(" missing=1 skipped=0"), "{line}");
    assert!(!f.en("audio/s.ogg").exists());
    assert!(f.manifest().objects["assets/en/audio/s.ogg"].evicted);
}

/// A store whose GET rewrites the local stub first, as an extract running
/// during the download would.
struct RewritingStore {
    inner: LocalStore,
    stub: PathBuf,
}

#[async_trait::async_trait]
impl ObjectStore for RewritingStore {
    fn kind(&self) -> myrtle_storage::StorageKind {
        self.inner.kind()
    }
    async fn head(
        &self,
        key: &str,
    ) -> Result<Option<myrtle_storage::ObjectMeta>, myrtle_storage::StorageError> {
        self.inner.head(key).await
    }
    async fn get(
        &self,
        key: &str,
        opts: myrtle_storage::GetOptions,
    ) -> Result<myrtle_storage::GetResult, myrtle_storage::StorageError> {
        std::fs::write(&self.stub, b"fresh-extract").unwrap();
        self.inner.get(key, opts).await
    }
    async fn put(
        &self,
        key: &str,
        object: PutObject,
    ) -> Result<String, myrtle_storage::StorageError> {
        self.inner.put(key, object).await
    }
    async fn put_multipart_file(
        &self,
        key: &str,
        path: &Path,
        size: u64,
        part_size: u64,
        content_type: &str,
        cache_control: Option<&str>,
    ) -> Result<String, myrtle_storage::StorageError> {
        self.inner
            .put_multipart_file(key, path, size, part_size, content_type, cache_control)
            .await
    }
    async fn delete_many(
        &self,
        keys: &[String],
    ) -> Vec<(String, Result<(), myrtle_storage::StorageError>)> {
        self.inner.delete_many(keys).await
    }
    fn list<'a>(
        &'a self,
        prefix: &'a str,
    ) -> futures::stream::BoxStream<
        'a,
        Result<myrtle_storage::ObjectMeta, myrtle_storage::StorageError>,
    > {
        self.inner.list(prefix)
    }
    async fn put_bucket_cors(
        &self,
        origins: &[String],
    ) -> Result<(), myrtle_storage::StorageError> {
        self.inner.put_bucket_cors(origins).await
    }
}

#[tokio::test]
async fn restore_never_overwrites_a_stub_rewritten_during_the_download() {
    let f = Fixture::new();
    f.run(&f.evicting()).await;
    let store = RewritingStore {
        inner: LocalStore::new(f.output.parent().unwrap().join("bucket")),
        stub: f.en("textures/a.png"),
    };
    let mut only = f.restore_opts();
    only.only = vec!["textures/a.png".into()];
    let report = run_restore(&store, &only).await.unwrap();
    assert_eq!((report.restored, report.skipped, report.failed), (0, 1, 0));
    assert!(report.summary_line().ends_with(" missing=0 skipped=1"));
    assert_eq!(
        std::fs::read(f.en("textures/a.png")).unwrap(),
        b"fresh-extract"
    );
    let leftovers: Vec<_> = std::fs::read_dir(f.en("textures"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .filter(|n| n.to_string_lossy().ends_with(".tmp") && n != "d.png.tmp")
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}
