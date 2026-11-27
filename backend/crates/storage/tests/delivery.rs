//! `DeliveryResolver` over a local tree and its `.asset-sync.json` manifests.

use std::collections::BTreeMap;
use std::fs::{File, create_dir_all, metadata, write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use myrtle_storage::config::{DeliveryMode, DeliveryOracle, StorageConfig, StorageKind};
use myrtle_storage::delivery::{Delivery, DeliveryResolver};
use myrtle_storage::evict::EvictPolicy;
use myrtle_storage::key::{object_key, public_url};
use myrtle_storage::sync::{MANIFEST_VERSION, Manifest, ManifestRecord};
use tempfile::TempDir;

const BASE: &str = "https://cdn.test";

fn config(bucket: &str, prefix: &str) -> StorageConfig {
    StorageConfig {
        kind: StorageKind::Cloudflare,
        bucket: bucket.to_owned(),
        access_key_id: None,
        secret_access_key: None,
        account_id: None,
        endpoint: None,
        prefix: prefix.to_owned(),
        public_url: Some(BASE.to_owned()),
        local_dir: None,
        roots: vec!["textures".to_owned(), "audio".to_owned()],
        backup_roots: Vec::new(),
        cache_control: String::new(),
        sync_concurrency: 1,
        hash_threads: 1,
        multipart_threshold: 1,
        part_size: 1,
        evict: EvictPolicy::default(),
    }
}

fn put(root: &Path, server: &str, rel: &str, len: usize) {
    let path = root.join(server).join(rel);
    create_dir_all(path.parent().unwrap()).unwrap();
    write(path, vec![b'x'; len]).unwrap();
}

fn manifest(root: &Path, server: &str, bucket: &str, prefix: &str, records: &[(&str, u64)]) {
    let objects: BTreeMap<String, ManifestRecord> = records
        .iter()
        .map(|(rel, size)| {
            (
                object_key(prefix, server, rel),
                ManifestRecord {
                    size: *size,
                    md5: String::new(),
                    etag: String::new(),
                    evicted: false,
                },
            )
        })
        .collect();
    create_dir_all(root.join(server)).unwrap();
    Manifest {
        version: MANIFEST_VERSION,
        bucket: bucket.to_owned(),
        prefix: prefix.to_owned(),
        objects,
    }
    .save(&Manifest::path(root, server))
    .unwrap();
}

fn resolver(root: &Path, mode: DeliveryMode, oracle: DeliveryOracle) -> DeliveryResolver {
    DeliveryResolver::new(
        root.to_path_buf(),
        Some(&config("bkt", "pre")),
        mode,
        oracle,
    )
}

fn redirect(server: &str, rel: &str) -> Delivery {
    Delivery::Redirect {
        url: public_url(BASE, &object_key("pre", server, rel)),
    }
}

fn local(root: &Path, server: &str, rel: &str) -> Delivery {
    Delivery::Local {
        path: root.join(server).join(rel),
    }
}

const ART: &str = "textures/a.png";

#[test]
fn manifest_oracle_per_file() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    put(root, "en", "textures/stub_rec.png", 0);
    put(root, "en", "textures/same.png", 4);
    put(root, "en", "textures/diff.png", 4);
    put(root, "en", "textures/full.png", 4);
    put(root, "en", "textures/stub.png", 0);
    manifest(
        root,
        "en",
        "bkt",
        "pre",
        &[
            ("textures/stub_rec.png", 9),
            ("textures/same.png", 4),
            ("textures/diff.png", 7),
        ],
    );
    let r = resolver(root, DeliveryMode::Redirect, DeliveryOracle::Manifest);
    let en = ["en"];
    assert_eq!(
        r.resolve(&en, "textures/stub_rec.png"),
        redirect("en", "textures/stub_rec.png")
    );
    assert_eq!(
        r.resolve(&en, "textures/same.png"),
        redirect("en", "textures/same.png")
    );
    assert_eq!(
        r.resolve(&en, "textures/diff.png"),
        local(root, "en", "textures/diff.png")
    );
    assert_eq!(
        r.resolve(&en, "textures/full.png"),
        local(root, "en", "textures/full.png")
    );
    assert_eq!(r.resolve(&en, "textures/stub.png"), Delivery::NotFound);
    assert_eq!(r.resolve(&en, "textures/none.png"), Delivery::NotFound);
}

#[test]
fn falls_back_to_the_next_server() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    put(root, "cn", ART, 4);
    put(root, "cn", "textures/b.png", 4);
    put(root, "en", "textures/b.png", 0);
    manifest(root, "en", "bkt", "pre", &[]);
    manifest(root, "cn", "bkt", "pre", &[(ART, 4), ("textures/b.png", 4)]);
    let r = resolver(root, DeliveryMode::Redirect, DeliveryOracle::Manifest);
    assert_eq!(r.resolve(&["en", "cn"], ART), redirect("cn", ART));
    assert_eq!(
        r.resolve(&["en", "cn"], "textures/b.png"),
        redirect("cn", "textures/b.png")
    );
}

#[test]
fn unusable_manifest_redirects_what_exists() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    for server in ["missing", "corrupt", "bucket", "prefix", "keys"] {
        put(root, server, ART, 4);
        put(root, server, "textures/stub.png", 0);
    }
    write(root.join("corrupt").join(".asset-sync.json"), b"{ not json").unwrap();
    manifest(root, "bucket", "other", "pre", &[]);
    manifest(root, "prefix", "bkt", "other", &[]);
    manifest(root, "keys", "", "", &[(ART, 4)]);
    let r = resolver(root, DeliveryMode::Redirect, DeliveryOracle::Manifest);
    for server in ["missing", "corrupt", "bucket", "prefix", "keys"] {
        assert_eq!(r.resolve(&[server], ART), redirect(server, ART), "{server}");
        assert_eq!(
            r.resolve(&[server], "textures/stub.png"),
            redirect(server, "textures/stub.png"),
            "{server}"
        );
    }
}

#[test]
fn reloads_a_changed_manifest_after_the_interval() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    put(root, "en", ART, 4);
    manifest(root, "en", "bkt", "pre", &[]);
    let start = Instant::now();
    let elapsed = Arc::new(AtomicU64::new(0));
    let clock = Arc::clone(&elapsed);
    let r = resolver(root, DeliveryMode::Redirect, DeliveryOracle::Manifest)
        .with_recheck(Duration::from_secs(5))
        .with_clock(move || start + Duration::from_secs(clock.load(Ordering::SeqCst)));
    assert_eq!(r.resolve(&["en"], ART), local(root, "en", ART));

    manifest(root, "en", "bkt", "pre", &[(ART, 4)]);
    let path = Manifest::path(root, "en");
    File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(SystemTime::now() + Duration::from_secs(60))
        .unwrap();
    elapsed.store(4, Ordering::SeqCst);
    assert_eq!(r.resolve(&["en"], ART), local(root, "en", ART));
    elapsed.store(5, Ordering::SeqCst);
    assert_eq!(r.resolve(&["en"], ART), redirect("en", ART));
}

/// `main`'s `deliver_with`
fn legacy(root: &Path, mode: DeliveryMode, server: &str, rel: &str) -> Option<Delivery> {
    let path = root.join(server).join(rel);
    let len = metadata(&path)
        .ok()
        .filter(std::fs::Metadata::is_file)?
        .len();
    if mode == DeliveryMode::Local || !rel.starts_with("textures/") {
        return Some(Delivery::Local { path });
    }
    Some(match mode {
        DeliveryMode::Redirect => redirect(server, rel),
        _ => Delivery::Proxy {
            key: object_key("pre", server, rel),
            path,
            stub: len == 0,
        },
    })
}

#[test]
fn exists_oracle_matches_legacy() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let rels = [
        "textures/stub_rec.png",
        "textures/same.png",
        "textures/diff.png",
        "textures/full.png",
        "textures/stub.png",
        "textures/none.png",
        "gamedata/full.json",
        "gamedata/stub.json",
    ];
    for (server, sizes) in [("en", [0, 4, 4, 4, 0]), ("cn", [4, 0, 4, 0, 4])] {
        for (rel, len) in rels.iter().zip(sizes) {
            put(root, server, rel, len);
        }
        put(root, server, "gamedata/full.json", 4);
        put(root, server, "gamedata/stub.json", 0);
        manifest(
            root,
            server,
            "bkt",
            "pre",
            &[
                ("textures/stub_rec.png", 9),
                ("textures/same.png", 4),
                ("textures/diff.png", 7),
            ],
        );
    }
    for mode in [
        DeliveryMode::Redirect,
        DeliveryMode::Proxy,
        DeliveryMode::Local,
    ] {
        let r = resolver(root, mode, DeliveryOracle::Exists);
        for chain in [&["en", "cn"][..], &["cn"], &["en"], &["jp", "cn"]] {
            for rel in rels {
                let expected = chain
                    .iter()
                    .find_map(|s| legacy(root, mode, s, rel))
                    .unwrap_or(Delivery::NotFound);
                assert_eq!(r.resolve(chain, rel), expected, "{mode} {chain:?} {rel}");
            }
        }
    }
}

#[test]
fn local_delivery_never_serves_a_stub_under_the_manifest_oracle() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    put(root, "en", ART, 0);
    put(root, "cn", ART, 4);
    manifest(root, "en", "bkt", "pre", &[(ART, 4)]);
    let r = resolver(root, DeliveryMode::Local, DeliveryOracle::Manifest);
    assert_eq!(r.resolve(&["en", "cn"], ART), local(root, "cn", ART));
    assert_eq!(r.resolve(&["en"], ART), Delivery::NotFound);
}

#[test]
fn encodes_keys_like_encode_uri_component() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let rel = "textures/a b/#1[x]%2.png";
    put(root, "en", rel, 4);
    manifest(root, "en", "bkt", "pre", &[(rel, 4)]);
    let r = resolver(root, DeliveryMode::Redirect, DeliveryOracle::Manifest);
    assert_eq!(
        r.resolve(&["en"], rel),
        Delivery::Redirect {
            url: "https://cdn.test/pre/en/textures/a%20b/%231%5Bx%5D%252.png".to_owned()
        }
    );
}

#[test]
fn parses_the_oracle() {
    let storage = config("bkt", "pre");
    let parse = |v: Option<&str>| {
        let v = v.map(ToOwned::to_owned);
        DeliveryOracle::from_lookup(
            &move |k| (k == "ASSET_DELIVERY_ORACLE").then(|| v.clone()).flatten(),
            Some(&storage),
        )
    };
    assert_eq!(parse(None), Ok(DeliveryOracle::Manifest));
    assert_eq!(parse(Some(" ")), Ok(DeliveryOracle::Manifest));
    assert_eq!(parse(Some("EXISTS")), Ok(DeliveryOracle::Exists));
    assert_eq!(parse(Some("manifest")), Ok(DeliveryOracle::Manifest));
    assert!(parse(Some("bucket")).is_err());
    assert_eq!(
        DeliveryOracle::from_lookup(&|_| Some("bogus".to_owned()), None),
        Ok(DeliveryOracle::Exists)
    );
}
