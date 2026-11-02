//! Where a request for a synced asset is answered from

use std::collections::HashMap;
use std::fs::{metadata, read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock, TryLockError};
use std::time::{Duration, Instant, SystemTime};

use serde::Deserialize;
use serde_json::from_slice;
use tokio::task::{JoinError, spawn_blocking};
use tracing::{info, warn};

use crate::config::{DeliveryMode, DeliveryOracle, StorageConfig};
use crate::key::{is_under_roots, normalize_rel, object_key, public_url, server_prefix};
use crate::sync::Manifest;

/// How often a manifest file is checked for changes
pub const MANIFEST_RECHECK: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// `302 Found` to the object's public URL
    Redirect {
        url: String,
    },
    /// Stream `key` from the store
    Proxy {
        key: String,
        path: PathBuf,
        stub: bool,
    },
    /// Serve the bytes at `path`
    Local {
        path: PathBuf,
    },
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexEntry {
    pub size: u64,
    pub evicted: bool,
}

/// A server's manifest keyed by rel path, without the hashes
#[derive(Debug, Default)]
pub struct ManifestIndex {
    entries: HashMap<Box<str>, IndexEntry>,
}

#[derive(Deserialize)]
struct SlimManifest {
    #[serde(default)]
    bucket: String,
    #[serde(default)]
    prefix: String,
    objects: HashMap<String, SlimRecord>,
}

#[derive(Deserialize)]
struct SlimRecord {
    size: u64,
    #[serde(default)]
    evicted: bool,
}

impl ManifestIndex {
    /// Rejects a manifest written for another bucket, prefix or server
    pub fn load(path: &Path, bucket: &str, prefix: &str, server: &str) -> Result<Self, String> {
        let raw = read(path).map_err(|e| e.to_string())?;
        let slim: SlimManifest = from_slice(&raw).map_err(|e| e.to_string())?;
        drop(raw);
        if !slim.bucket.is_empty() && slim.bucket != bucket {
            return Err(format!(
                "written for CLOUDFLARE_BUCKET={}, configured {bucket:?}",
                slim.bucket
            ));
        }
        if !slim.prefix.is_empty() && slim.prefix != prefix {
            return Err(format!(
                "written for CLOUDFLARE_PREFIX={}, configured {prefix:?}",
                slim.prefix
            ));
        }
        let key_prefix = server_prefix(prefix, server);
        let mut entries = HashMap::with_capacity(slim.objects.len());
        for (key, record) in slim.objects {
            let Some(rel) = key.strip_prefix(&key_prefix) else {
                return Err(format!("records {key}, outside {key_prefix}"));
            };
            entries.insert(
                rel.into(),
                IndexEntry {
                    size: record.size,
                    evicted: record.evicted,
                },
            );
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn get(&self, rel: &str) -> Option<IndexEntry> {
        self.entries.get(rel).copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stamp {
    modified: Option<SystemTime>,
    len: u64,
}

fn stamp_of(path: &Path) -> Option<Stamp> {
    let meta = metadata(path).ok()?;
    Some(Stamp {
        modified: meta.modified().ok(),
        len: meta.len(),
    })
}

#[derive(Debug, Default)]
struct Snapshot {
    checked: bool,
    stamp: Option<Stamp>,
    /// `None` falls back to the existence oracle
    index: Option<ManifestIndex>,
}

#[derive(Debug, Default)]
struct Slot {
    current: RwLock<Arc<Snapshot>>,
    checked_at: Mutex<Option<Instant>>,
}

impl Slot {
    fn current(&self) -> Arc<Snapshot> {
        Arc::clone(&self.current.read().unwrap_or_else(PoisonError::into_inner))
    }
}

#[derive(Debug, Clone)]
struct Store {
    bucket: String,
    prefix: String,
    public_url: Option<String>,
    roots: Vec<String>,
}

type Clock = Arc<dyn Fn() -> Instant + Send + Sync>;

/// Picks the [`Delivery`] for an asset, caching each server's manifest
pub struct DeliveryResolver {
    /// The `assets/output` directory holding one tree per server
    root: PathBuf,
    store: Option<Store>,
    mode: DeliveryMode,
    oracle: DeliveryOracle,
    recheck: Duration,
    clock: Clock,
    slots: RwLock<HashMap<String, Arc<Slot>>>,
}

impl DeliveryResolver {
    #[must_use]
    pub fn new(
        root: PathBuf,
        storage: Option<&StorageConfig>,
        mode: DeliveryMode,
        oracle: DeliveryOracle,
    ) -> Self {
        Self {
            root,
            store: storage.map(|c| Store {
                bucket: c.bucket.clone(),
                prefix: c.prefix.clone(),
                public_url: c.public_url.clone(),
                roots: c.roots.clone(),
            }),
            mode,
            oracle,
            recheck: MANIFEST_RECHECK,
            clock: Arc::new(Instant::now),
            slots: RwLock::new(HashMap::new()),
        }
    }

    #[must_use]
    pub const fn with_recheck(mut self, recheck: Duration) -> Self {
        self.recheck = recheck;
        self
    }

    #[must_use]
    pub fn with_clock(mut self, clock: impl Fn() -> Instant + Send + Sync + 'static) -> Self {
        self.clock = Arc::new(clock);
        self
    }

    /// The first server in `servers` that can answer; blocks on disk
    pub fn resolve(&self, servers: &[&str], rel: &str) -> Delivery {
        let Some(rel) = normalize_rel(rel) else {
            return Delivery::NotFound;
        };
        servers
            .iter()
            .find_map(|server| self.resolve_server(server, &rel))
            .unwrap_or(Delivery::NotFound)
    }

    /// [`DeliveryResolver::resolve`] on the blocking pool
    pub async fn resolve_async(
        self: &Arc<Self>,
        servers: Vec<String>,
        rel: String,
    ) -> Result<Delivery, JoinError> {
        let this = Arc::clone(self);
        spawn_blocking(move || {
            let servers: Vec<&str> = servers.iter().map(String::as_str).collect();
            this.resolve(&servers, &rel)
        })
        .await
    }

    /// `None` tries the next server
    fn resolve_server(&self, server: &str, rel: &str) -> Option<Delivery> {
        let path = self.root.join(server).join(rel);
        let len = match metadata(&path) {
            Ok(m) if m.is_file() => m.len(),
            _ => return None,
        };
        let stub = len == 0;
        let local = || {
            (self.oracle == DeliveryOracle::Exists || !stub)
                .then(|| Delivery::Local { path: path.clone() })
        };
        let Some(store) = &self.store else {
            return local();
        };
        if !is_under_roots(rel, &store.roots) {
            return local();
        }
        let key = object_key(&store.prefix, server, rel);
        match self.mode {
            DeliveryMode::Local => local(),
            DeliveryMode::Proxy => Some(Delivery::Proxy { key, path, stub }),
            DeliveryMode::Redirect => {
                let Some(base) = &store.public_url else {
                    return local();
                };
                let redirect = || Delivery::Redirect {
                    url: public_url(base, &key),
                };
                if self.oracle == DeliveryOracle::Exists {
                    return Some(redirect());
                }
                let snapshot = self.snapshot(server, store);
                let Some(index) = &snapshot.index else {
                    return Some(redirect());
                };
                match index.get(rel) {
                    Some(entry) if stub || entry.size == len => Some(redirect()),
                    _ if stub => None,
                    _ => Some(Delivery::Local { path }),
                }
            }
        }
    }

    fn slot(&self, server: &str) -> Arc<Slot> {
        if let Some(slot) = self
            .slots
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(server)
        {
            return Arc::clone(slot);
        }
        Arc::clone(
            self.slots
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .entry(server.to_owned())
                .or_default(),
        )
    }

    /// Reloads `server`'s manifest when its file changed, at most once per recheck
    fn snapshot(&self, server: &str, store: &Store) -> Arc<Snapshot> {
        let slot = self.slot(server);
        let mut checked_at = match slot.checked_at.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(e)) => e.into_inner(),
            Err(TryLockError::WouldBlock) => {
                let current = slot.current();
                if current.checked {
                    return current;
                }
                slot.checked_at
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
            }
        };
        let current = slot.current();
        let now = (self.clock)();
        if current.checked
            && checked_at.is_some_and(|at| now.saturating_duration_since(at) < self.recheck)
        {
            return current;
        }
        *checked_at = Some(now);
        let path = Manifest::path(&self.root, server);
        let stamp = stamp_of(&path);
        if current.checked && current.stamp == stamp {
            return current;
        }
        let index = if stamp.is_none() {
            warn!(server, path = %path.display(), "no sync manifest; redirecting every existing file");
            None
        } else {
            match ManifestIndex::load(&path, &store.bucket, &store.prefix, server) {
                Ok(index) => {
                    info!(server, records = index.len(), "sync manifest loaded");
                    Some(index)
                }
                Err(error) => {
                    warn!(server, path = %path.display(), %error, "sync manifest unusable; redirecting every existing file");
                    None
                }
            }
        };
        let next = Arc::new(Snapshot {
            checked: true,
            stamp,
            index,
        });
        *slot.current.write().unwrap_or_else(PoisonError::into_inner) = Arc::clone(&next);
        next
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs::{File, create_dir_all, write};
    use std::sync::atomic::{AtomicU64, Ordering};

    use tempfile::TempDir;

    use super::*;
    use crate::config::StorageKind;
    use crate::evict::EvictPolicy;
    use crate::sync::{MANIFEST_VERSION, ManifestRecord};

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
}
