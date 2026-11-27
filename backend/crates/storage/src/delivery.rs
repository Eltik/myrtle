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
    #[allow(
        clippy::significant_drop_tightening,
        reason = "the guard spans the reload so concurrent first loads wait instead of parsing again"
    )]
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
