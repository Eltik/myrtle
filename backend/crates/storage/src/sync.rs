//! Mirrors one server's served roots into the store

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{File, OpenOptions, TryLockError, metadata, read, rename};
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::id;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures::StreamExt;
use futures::stream::iter;
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec_pretty};
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::task::spawn_blocking;
use tokio::time::sleep;
use tracing::{info, warn};
use walkdir::WalkDir;

use crate::config::StorageConfig;
use crate::evict::{EvictPolicy, truncate_in_place};
use crate::key::{object_key, server_prefix};
use crate::store::{ObjectStore, PutBody, PutObject, StorageError, content_type_for, md5_file};

/// Extensions the backend serves (`ALLOWED_EXTENSIONS` in
/// `routes/assets.rs`)
pub const SERVED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "svg", "mp3", "ogg", "wav", "m4a", "mp4", "webm", "skel",
    "atlas", "json", "txt",
];
/// Manifest file name under `<root>/<server>/`
pub const MANIFEST_NAME: &str = ".asset-sync.json";
/// Advisory lock file under `<root>/<server>/`
pub const LOCK_NAME: &str = ".asset-sync.lock";
/// Manifest format written: 2 adds `evicted` to each record. A version 1 file
/// loads unchanged
pub const MANIFEST_VERSION: u32 = 2;
/// The error a stub whose object has left the bucket fails with
pub const STUB_GONE: &str =
    "evicted stub but the object is gone from the store; re-extract to recover";
/// The error a stub whose object no longer matches its record fails with
pub const STUB_CHANGED: &str = "evicted stub but the stored object no longer matches its manifest record (size or etag); re-extract to recover";
/// Retries per object
pub const OBJECT_RETRIES: u32 = 3;
/// First retry waits 500 ms, doubling each time
pub const RETRY_BASE: Duration = Duration::from_millis(500);
/// Deletes refuse to run when the local count is under 90% of the remote
pub const SHRINK_FLOOR_PERCENT: usize = 90;
/// Progress is logged every 1000 files
const PROGRESS_EVERY: usize = 1000;
/// Keys per delete request
const DELETE_BATCH: usize = 1000;

#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// The `assets/output` directory holding one tree per server
    pub root: PathBuf,
    pub server: String,
    pub prefix: String,
    pub roots: Vec<String>,
    pub delete: bool,
    pub dry_run: bool,
    pub allow_shrink: bool,
    pub concurrency: usize,
    pub hash_threads: usize,
    pub multipart_threshold: u64,
    pub part_size: u64,
    pub cache_control: String,
    pub retry_base: Duration,
    /// `CLOUDFLARE_BUCKET`, recorded in the manifest beside the prefix
    pub bucket: String,
    /// Truncate files the bucket holds; off unless `STORAGE_EVICT_LOCAL` or `--evict`
    pub evict: EvictPolicy,
}

impl SyncOptions {
    /// Options with every tunable taken from `config`
    #[must_use]
    pub fn from_config(config: &StorageConfig, root: PathBuf, server: String) -> Self {
        Self {
            root,
            server,
            prefix: config.prefix.clone(),
            roots: config.roots.clone(),
            delete: true,
            dry_run: false,
            allow_shrink: false,
            concurrency: config.sync_concurrency,
            hash_threads: config.hash_threads,
            multipart_threshold: config.multipart_threshold,
            part_size: config.part_size,
            cache_control: config.cache_control.clone(),
            retry_base: RETRY_BASE,
            bucket: config.bucket.clone(),
            evict: config.evict.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SyncOutcome {
    Ok,
    /// One or more objects failed to upload or delete
    Failed,
    /// Deletes were skipped because the local tree looks truncated
    ShrinkRefused {
        local: usize,
        remote: usize,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub server: String,
    pub scanned: usize,
    pub unchanged: usize,
    pub uploaded: usize,
    pub uploaded_bytes: u64,
    pub deleted: usize,
    pub failed: usize,
    /// 0-byte stubs whose object is in the bucket
    pub stubs: usize,
    /// Whether eviction was on for this run
    pub evict: bool,
    /// Files truncated to stubs
    pub evicted: usize,
    /// Bytes those truncations freed
    pub freed_bytes: u64,
    /// Truncations that failed
    pub evict_failed: usize,
    /// Files under the roots left out by the skip rules
    pub skipped: usize,
    pub missing_roots: Vec<String>,
    pub dry_run: bool,
    pub outcome: SyncOutcome,
    pub elapsed_secs: f64,
    /// The first failures, `key: error` for the log
    pub failures: Vec<String>,
}

/// At most this many failure messages ride in the report
const MAX_REPORTED_FAILURES: usize = 20;

impl SyncReport {
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self.outcome {
            SyncOutcome::Ok => 0,
            SyncOutcome::Failed => 2,
            SyncOutcome::ShrinkRefused { .. } => 3,
        }
    }

    /// True when the remote differs from the local tree
    #[must_use]
    pub const fn drifted(&self) -> bool {
        self.uploaded > 0
            || self.deleted > 0
            || self.failed > 0
            || !matches!(self.outcome, SyncOutcome::Ok)
    }

    /// `asset-sync: server=en scanned=N unchanged=N uploaded=N (B bytes) deleted=N failed=N elapsed=S.s`,
    /// with `evicted=N (B bytes freed)` before `failed` when eviction is on
    #[must_use]
    pub fn summary_line(&self) -> String {
        let evicted = if self.evict {
            format!(
                " evicted={} ({} bytes freed)",
                self.evicted, self.freed_bytes
            )
        } else {
            String::new()
        };
        format!(
            "asset-sync: server={} scanned={} unchanged={} uploaded={} ({} bytes) deleted={}{evicted} failed={} elapsed={:.1}s{}",
            self.server,
            self.scanned,
            self.unchanged,
            self.uploaded,
            self.uploaded_bytes,
            self.deleted,
            self.failed,
            self.elapsed_secs,
            if self.dry_run { " (dry run)" } else { "" },
        )
    }
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("{0} is not a directory; check --root and --server")]
    MissingServerDir(PathBuf),
    #[error("listing the remote failed: {0}")]
    List(StorageError),
    #[error("walking the local tree failed: {0}")]
    Walk(String),
    #[error(
        "manifest {path} is unreadable ({error}); it is what marks 0-byte files as evicted stubs, so syncing without it could upload stubs as empty objects. Repair it, or remove it only if no file was ever evicted"
    )]
    Manifest { path: PathBuf, error: String },
    #[error(
        "another asset-sync is running on this server: the lock {0} is held; wait for it to finish"
    )]
    Locked(PathBuf),
    #[error("{0}")]
    StoreMoved(String),
}

/// What the bucket holds for a key
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestRecord {
    pub size: u64,
    /// MD5 of the content
    pub md5: String,
    pub etag: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub evicted: bool,
}

/// `<root>/<server>/.asset-sync.json`: last run's record of what the
/// bucket holds
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    /// `CLOUDFLARE_BUCKET` the records belong to
    #[serde(default)]
    pub bucket: String,
    /// `CLOUDFLARE_PREFIX` the record keys were built with
    #[serde(default)]
    pub prefix: String,
    pub objects: BTreeMap<String, ManifestRecord>,
}

impl Manifest {
    #[must_use]
    pub fn path(root: &Path, server: &str) -> PathBuf {
        root.join(server).join(MANIFEST_NAME)
    }

    /// Records whose local file is an evicted stub
    #[must_use]
    pub fn evicted_count(&self) -> usize {
        self.objects.values().filter(|r| r.evicted).count()
    }

    /// Evicted records whose key is not under `key_prefix` (`[prefix/]server/`)
    /// or whose rel's first component is not in `roots`
    #[must_use]
    pub fn evicted_outside_roots(&self, key_prefix: &str, roots: &[String]) -> usize {
        self.objects
            .iter()
            .filter(|(_, r)| r.evicted)
            .filter(|(key, _)| {
                key.strip_prefix(key_prefix)
                    .and_then(|rel| rel.split_once('/'))
                    .is_none_or(|(first, _)| !roots.iter().any(|r| r == first))
            })
            .count()
    }

    /// Refuses when stubs exist and the manifest was written for another
    /// bucket or prefix
    ///
    /// # Errors
    /// Stubs exist and the manifest names another bucket or prefix
    pub fn check_store(&self, bucket: &str, prefix: &str, server: &str) -> Result<(), String> {
        let evicted = self.evicted_count();
        if evicted == 0 {
            return Ok(());
        }
        let key_prefix = server_prefix(prefix, server);
        let bucket_moved = !self.bucket.is_empty() && self.bucket != bucket;
        let prefix_moved = (!self.prefix.is_empty() && self.prefix != prefix)
            || self
                .objects
                .iter()
                .any(|(key, r)| r.evicted && !key.starts_with(&key_prefix));
        if !bucket_moved && !prefix_moved {
            return Ok(());
        }
        let shown = |v: &str| {
            if v.is_empty() {
                "(unset)".to_owned()
            } else {
                v.to_owned()
            }
        };
        Err(format!(
            "{evicted} evicted stubs were synced with CLOUDFLARE_BUCKET={} CLOUDFLARE_PREFIX={}, but the settings now are CLOUDFLARE_BUCKET={} CLOUDFLARE_PREFIX={}; run asset-sync restore under the old settings first",
            shown(&self.bucket),
            shown(&self.prefix),
            shown(bucket),
            shown(prefix),
        ))
    }

    /// The manifest at `path`, empty when absent
    ///
    /// # Errors
    /// The file exists but cannot be read or parsed.
    pub fn read(path: &Path) -> Result<Self, String> {
        match read(path) {
            Ok(raw) => from_slice(&raw).map_err(|e| e.to_string()),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// The manifest at `path`, empty when absent or unreadable
    #[must_use]
    pub fn load(path: &Path) -> Self {
        read(path).map_or_else(
            |_| Self::default(),
            |raw| {
                from_slice(&raw).unwrap_or_else(|e| {
                    warn!(path = %path.display(), error = %e, "manifest unreadable, starting empty");
                    Self::default()
                })
            },
        )
    }

    /// Writes through a temp file, fsync and rename
    ///
    /// # Errors
    /// The temp file cannot be written or renamed.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        use std::io::Write;
        let tmp = path.with_file_name(format!("{MANIFEST_NAME}.{}.tmp", id()));
        let raw = to_vec_pretty(self).map_err(Error::other)?;
        let mut file = File::create(&tmp)?;
        file.write_all(&raw)?;
        file.sync_all()?;
        drop(file);
        rename(&tmp, path)?;
        sync_parent_dir(path);
        Ok(())
    }
}

/// Flushes the directory holding `path`
pub fn sync_parent_dir(path: &Path) {
    if let Some(dir) = path.parent()
        && let Ok(handle) = File::open(dir)
    {
        let _ = handle.sync_all();
    }
}

/// Per-server advisory lock
#[derive(Debug)]
pub struct ServerLock {
    _file: File,
}

impl ServerLock {
    /// Takes `<server_dir>/.asset-sync.lock` without waiting
    ///
    /// # Errors
    /// [`SyncError::Locked`] when another process holds it, or
    /// [`SyncError::Walk`] when the lock file cannot be opened.
    pub fn acquire(server_dir: &Path) -> Result<Self, SyncError> {
        let path = server_dir.join(LOCK_NAME);
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| SyncError::Walk(format!("opening the lock {}: {e}", path.display())))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(SyncError::Locked(path)),
            Err(TryLockError::Error(e)) => {
                Err(SyncError::Walk(format!("locking {}: {e}", path.display())))
            }
        }
    }
}

#[derive(Debug, Clone)]
struct LocalFile {
    /// Path under `<root>/<server>/`
    rel: String,
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
}

struct Walked {
    files: Vec<LocalFile>,
    skipped: usize,
    missing_roots: Vec<String>,
    roots: Vec<String>,
}

fn is_skipped_name(name: &str) -> bool {
    name.starts_with('.')
        || Path::new(name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("tmp"))
}

fn has_served_extension(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SERVED_EXTENSIONS.contains(&e))
}

/// Every syncable file under `server_dir/<root>` for each root
fn walk_local(server_dir: &Path, roots: &[String]) -> Result<Walked, String> {
    let mut walked = Walked {
        files: Vec::new(),
        skipped: 0,
        missing_roots: Vec::new(),
        roots: Vec::new(),
    };
    for root in roots {
        let dir = server_dir.join(root);
        if !dir.is_dir() {
            warn!(root = %dir.display(), "root missing locally; its remote keys are kept");
            walked.missing_roots.push(root.clone());
            continue;
        }
        walked.roots.push(root.clone());
        let entries = WalkDir::new(&dir)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || !e.file_name().to_string_lossy().starts_with('.'));
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(name) = entry.file_name().to_str() else {
                warn!(path = %entry.path().display(), "skipping a non-UTF-8 file name");
                walked.skipped += 1;
                continue;
            };
            if is_skipped_name(name) || !has_served_extension(name) {
                walked.skipped += 1;
                continue;
            }
            let rel_path = entry
                .path()
                .strip_prefix(server_dir)
                .map_err(|e| e.to_string())?;
            let Some(rel) = rel_path
                .components()
                .map(|c| c.as_os_str().to_str())
                .collect::<Option<Vec<_>>>()
                .map(|parts| parts.join("/"))
            else {
                warn!(path = %entry.path().display(), "skipping a non-UTF-8 path");
                walked.skipped += 1;
                continue;
            };
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            walked.files.push(LocalFile {
                rel,
                path: entry.path().to_path_buf(),
                size: meta.len(),
                modified: meta.modified().ok(),
            });
        }
    }
    Ok(walked)
}

enum FileResult {
    Unchanged {
        rel: String,
        record: ManifestRecord,
        evict: Option<LocalFile>,
    },
    /// An evicted stub whose object is in the bucket
    Stub { rel: String, record: ManifestRecord },
    Uploaded {
        rel: String,
        bytes: u64,
        record: ManifestRecord,
        evict: Option<LocalFile>,
    },
    Failed {
        rel: String,
        error: String,
        record: Option<ManifestRecord>,
    },
}

/// `file` when the policy evicts it
fn eviction_candidate(opts: &SyncOptions, file: &LocalFile) -> Option<LocalFile> {
    (opts.evict.enabled && file.size > 0 && !opts.evict.keeps(&file.rel)).then(|| file.clone())
}

/// Truncates `file` in place unless it changed since walk
fn truncate_unchanged(file: &LocalFile) -> io::Result<()> {
    let meta = metadata(&file.path)?;
    if meta.len() != file.size || meta.modified().ok() != file.modified {
        return Err(Error::other("changed since it was hashed"));
    }
    truncate_in_place(&file.path)
}

/// Result for 0-byte file whose record says it is an evicted stub
fn stub_result(
    rel: String,
    record: ManifestRecord,
    remote: Option<&(u64, Option<String>)>,
) -> FileResult {
    match remote {
        Some((size, etag))
            if *size == record.size
                && etag
                    .as_deref()
                    .is_some_and(|e| e.eq_ignore_ascii_case(&record.etag)) =>
        {
            FileResult::Stub { rel, record }
        }
        Some(_) => FileResult::Failed {
            rel,
            error: STUB_CHANGED.into(),
            record: Some(record),
        },
        None => FileResult::Failed {
            rel,
            error: STUB_GONE.into(),
            record: Some(record),
        },
    }
}

async fn hash(path: &Path, hash_sem: &Semaphore) -> Result<[u8; 16], StorageError> {
    let _permit = hash_sem
        .acquire()
        .await
        .map_err(|e| StorageError::Request(e.to_string()))?;
    let p = path.to_path_buf();
    spawn_blocking(move || md5_file(&p))
        .await
        .map_err(|e| StorageError::Request(e.to_string()))?
        .map_err(StorageError::from)
}

async fn upload_once(
    store: &dyn ObjectStore,
    opts: &SyncOptions,
    key: &str,
    file: &LocalFile,
    md5: &[u8; 16],
) -> Result<String, StorageError> {
    let content_type = content_type_for(&file.rel);
    if file.size >= opts.multipart_threshold {
        store
            .put_multipart_file(
                key,
                &file.path,
                file.size,
                opts.part_size,
                &content_type,
                Some(&opts.cache_control),
            )
            .await
    } else {
        store
            .put(
                key,
                PutObject {
                    body: PutBody::File {
                        path: file.path.clone(),
                        size: file.size,
                    },
                    content_type,
                    cache_control: Some(opts.cache_control.clone()),
                    content_md5: Some(STANDARD.encode(md5)),
                },
            )
            .await
    }
}

async fn upload_with_retry(
    store: &dyn ObjectStore,
    opts: &SyncOptions,
    key: &str,
    file: &LocalFile,
    md5: &[u8; 16],
) -> Result<String, StorageError> {
    let mut attempt = 0;
    loop {
        match upload_once(store, opts, key, file, md5).await {
            Ok(etag) => return Ok(etag),
            Err(e) if attempt < OBJECT_RETRIES => {
                let wait = opts.retry_base * 2u32.pow(attempt);
                warn!(key, attempt = attempt + 1, error = %e, ?wait, "upload failed, retrying");
                sleep(wait).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn process_file(
    store: &dyn ObjectStore,
    opts: &SyncOptions,
    file: LocalFile,
    remote: Option<(u64, Option<String>)>,
    record: Option<ManifestRecord>,
    hash_sem: &Semaphore,
    upload_sem: &Semaphore,
) -> FileResult {
    if file.size == 0
        && let Some(stub) = record.as_ref().filter(|r| r.evicted)
    {
        return stub_result(file.rel, stub.clone(), remote.as_ref());
    }
    let key = object_key(&opts.prefix, &opts.server, &file.rel);
    let md5 = match hash(&file.path, hash_sem).await {
        Ok(d) => d,
        Err(e) => {
            return FileResult::Failed {
                rel: file.rel,
                error: format!("hashing: {e}"),
                record: None,
            };
        }
    };
    let md5_hex = encode(md5);

    if let Some((size, Some(etag))) = &remote
        && *size == file.size
    {
        let same = if etag.contains('-') {
            record
                .as_ref()
                .is_some_and(|r| r.size == file.size && r.etag == *etag && r.md5 == md5_hex)
        } else {
            etag.eq_ignore_ascii_case(&md5_hex)
        };
        if same {
            let record = ManifestRecord {
                size: file.size,
                md5: md5_hex,
                etag: etag.clone(),
                evicted: false,
            };
            let evict = eviction_candidate(opts, &file);
            return FileResult::Unchanged {
                rel: file.rel,
                record,
                evict,
            };
        }
    }

    if opts.dry_run {
        let evict = eviction_candidate(opts, &file);
        return FileResult::Uploaded {
            bytes: file.size,
            record: ManifestRecord {
                size: file.size,
                md5: md5_hex.clone(),
                etag: md5_hex,
                evicted: false,
            },
            rel: file.rel,
            evict,
        };
    }

    let upload = {
        let Ok(_permit) = upload_sem.acquire().await else {
            return FileResult::Failed {
                rel: file.rel,
                error: "upload pool closed".into(),
                record: None,
            };
        };
        upload_with_retry(store, opts, &key, &file, &md5).await
    };
    match upload {
        Ok(etag) => {
            let evict = eviction_candidate(opts, &file);
            FileResult::Uploaded {
                bytes: file.size,
                record: ManifestRecord {
                    size: file.size,
                    md5: md5_hex,
                    etag,
                    evicted: false,
                },
                rel: file.rel,
                evict,
            }
        }
        Err(e) => FileResult::Failed {
            rel: file.rel,
            error: e.to_string(),
            record: None,
        },
    }
}

async fn delete_with_retry(
    store: &dyn ObjectStore,
    keys: Vec<String>,
    retry_base: Duration,
) -> Vec<(String, Result<(), StorageError>)> {
    let mut pending = keys;
    let mut done = Vec::new();
    let mut attempt = 0;
    loop {
        let results = store.delete_many(&pending).await;
        let mut failed = Vec::new();
        for (key, result) in results {
            match result {
                Ok(()) => done.push((key, Ok(()))),
                Err(e) if attempt < OBJECT_RETRIES => {
                    warn!(key, attempt = attempt + 1, error = %e, "delete failed, retrying");
                    failed.push(key);
                }
                Err(e) => done.push((key, Err(e))),
            }
        }
        if failed.is_empty() {
            return done;
        }
        sleep(retry_base * 2u32.pow(attempt)).await;
        attempt += 1;
        pending = failed;
    }
}

/// Mirrors `opts.roots` of `<opts.root>/<opts.server>` into `store`.
///
/// # Errors
/// The server directory is missing, the local walk fails, or the remote
/// cannot be listed.
#[allow(clippy::too_many_lines)]
pub async fn run_sync(
    store: &dyn ObjectStore,
    opts: &SyncOptions,
) -> Result<SyncReport, SyncError> {
    let started = Instant::now();
    let server_dir = opts.root.join(&opts.server);
    if !server_dir.is_dir() {
        return Err(SyncError::MissingServerDir(server_dir));
    }
    let _lock = if opts.dry_run {
        None
    } else {
        Some(ServerLock::acquire(&server_dir)?)
    };

    let walk_dir = server_dir.clone();
    let roots = opts.roots.clone();
    let walked = spawn_blocking(move || walk_local(&walk_dir, &roots))
        .await
        .map_err(|e| SyncError::Walk(e.to_string()))?
        .map_err(SyncError::Walk)?;
    info!(
        server = %opts.server,
        files = walked.files.len(),
        skipped = walked.skipped,
        "local tree walked"
    );

    let remote_prefix = server_prefix(&opts.prefix, &opts.server);
    let mut remote: HashMap<String, (u64, Option<String>)> = HashMap::new();
    let mut listing = store.list(&remote_prefix);
    while let Some(item) = listing.next().await {
        let meta = item.map_err(SyncError::List)?;
        if let Some(rel) = meta.key.strip_prefix(&remote_prefix) {
            remote.insert(rel.to_owned(), (meta.size, meta.etag));
        }
    }
    drop(listing);
    info!(server = %opts.server, objects = remote.len(), "remote listed");

    let manifest_path = Manifest::path(&opts.root, &opts.server);
    let old_manifest = Manifest::read(&manifest_path).map_err(|error| SyncError::Manifest {
        path: manifest_path.clone(),
        error,
    })?;
    old_manifest
        .check_store(&opts.bucket, &opts.prefix, &opts.server)
        .map_err(SyncError::StoreMoved)?;

    let hash_sem = Arc::new(Semaphore::new(opts.hash_threads.max(1)));
    let upload_sem = Arc::new(Semaphore::new(opts.concurrency.max(1)));
    let local_rels: HashSet<String> = walked.files.iter().map(|f| f.rel.clone()).collect();
    let scanned = walked.files.len();

    let mut report = SyncReport {
        server: opts.server.clone(),
        scanned,
        unchanged: 0,
        uploaded: 0,
        uploaded_bytes: 0,
        deleted: 0,
        failed: 0,
        stubs: 0,
        evict: opts.evict.enabled,
        evicted: 0,
        freed_bytes: 0,
        evict_failed: 0,
        skipped: walked.skipped,
        missing_roots: walked.missing_roots.clone(),
        dry_run: opts.dry_run,
        outcome: SyncOutcome::Ok,
        elapsed_secs: 0.0,
        failures: Vec::new(),
    };
    let walked_roots: HashSet<&str> = walked.roots.iter().map(String::as_str).collect();
    let in_walked_root = |rel: &str| {
        rel.split_once('/')
            .is_some_and(|(first, _)| walked_roots.contains(first))
    };
    let mut new_records: BTreeMap<String, ManifestRecord> = old_manifest
        .objects
        .iter()
        .filter(|(key, _)| {
            key.strip_prefix(&remote_prefix)
                .is_none_or(|rel| !in_walked_root(rel))
        })
        .map(|(k, r)| (k.clone(), r.clone()))
        .collect();

    let in_flight = opts.concurrency.max(1) + opts.hash_threads.max(1);
    let mut results = iter(walked.files.into_iter().map(|file| {
        let remote_entry = remote.get(&file.rel).cloned();
        let key = object_key(&opts.prefix, &opts.server, &file.rel);
        let record = old_manifest.objects.get(&key).cloned();
        let (hash_sem, upload_sem) = (Arc::clone(&hash_sem), Arc::clone(&upload_sem));
        async move {
            process_file(
                store,
                opts,
                file,
                remote_entry,
                record,
                &hash_sem,
                &upload_sem,
            )
            .await
        }
    }))
    .buffer_unordered(in_flight);

    let mut candidates: Vec<LocalFile> = Vec::new();
    let mut processed = 0usize;
    while let Some(result) = results.next().await {
        processed += 1;
        let (rel, record) = match result {
            FileResult::Unchanged { rel, record, evict } => {
                candidates.extend(evict);
                report.unchanged += 1;
                (rel, Some(record))
            }
            FileResult::Stub { rel, record } => {
                report.unchanged += 1;
                report.stubs += 1;
                (rel, Some(record))
            }
            FileResult::Uploaded {
                rel,
                bytes,
                record,
                evict,
            } => {
                candidates.extend(evict);
                report.uploaded += 1;
                report.uploaded_bytes += bytes;
                (rel, Some(record))
            }
            FileResult::Failed { rel, error, record } => {
                report.failed += 1;
                warn!(rel, error, "object failed");
                if report.failures.len() < MAX_REPORTED_FAILURES {
                    report.failures.push(format!("{rel}: {error}"));
                }
                (rel, record)
            }
        };
        if let Some(record) = record {
            new_records.insert(object_key(&opts.prefix, &opts.server, &rel), record);
        }
        if processed.is_multiple_of(PROGRESS_EVERY) {
            info!(
                server = %opts.server,
                processed,
                total = scanned,
                unchanged = report.unchanged,
                uploaded = report.uploaded,
                failed = report.failed,
                "progress"
            );
        }
    }
    drop(results);

    let remote_in_roots = remote.keys().filter(|rel| in_walked_root(rel)).count();
    let mut doomed: Vec<String> = remote
        .keys()
        .filter(|rel| in_walked_root(rel) && !local_rels.contains(*rel))
        .map(|rel| object_key(&opts.prefix, &opts.server, rel))
        .collect();
    doomed.sort();

    if opts.delete && !doomed.is_empty() {
        let floor_breached = scanned * 100 < remote_in_roots * SHRINK_FLOOR_PERCENT;
        if floor_breached && !opts.allow_shrink {
            warn!(
                server = %opts.server,
                local = scanned,
                remote = remote_in_roots,
                would_delete = doomed.len(),
                "local tree is under 90% of the remote; refusing to delete (pass --allow-shrink to override)"
            );
            report.outcome = SyncOutcome::ShrinkRefused {
                local: scanned,
                remote: remote_in_roots,
            };
        } else if opts.dry_run {
            report.deleted = doomed.len();
        } else {
            let mut deleted = iter(
                doomed
                    .chunks(DELETE_BATCH)
                    .map(|batch| delete_with_retry(store, batch.to_vec(), opts.retry_base)),
            )
            .buffer_unordered(opts.concurrency.max(1));
            while let Some(results) = deleted.next().await {
                for (key, result) in results {
                    match result {
                        Ok(()) => {
                            report.deleted += 1;
                            new_records.remove(&key);
                        }
                        Err(e) => {
                            report.failed += 1;
                            warn!(key, error = %e, "delete failed");
                            if report.failures.len() < MAX_REPORTED_FAILURES {
                                report.failures.push(format!("{key}: delete: {e}"));
                            }
                        }
                    }
                }
            }
        }
    }

    if report.failed > 0 && report.outcome == SyncOutcome::Ok {
        report.outcome = SyncOutcome::Failed;
    }

    if opts.dry_run {
        report.evicted = candidates.len();
        report.freed_bytes = candidates.iter().map(|f| f.size).sum();
    } else {
        evict_and_save(opts, &manifest_path, new_records, candidates, &mut report).await;
    }

    report.elapsed_secs = started.elapsed().as_secs_f64();
    Ok(report)
}

/// Saves the manifest, then truncates `candidates`
async fn evict_and_save(
    opts: &SyncOptions,
    manifest_path: &Path,
    mut records: BTreeMap<String, ManifestRecord>,
    candidates: Vec<LocalFile>,
    report: &mut SyncReport,
) {
    let key_of = |file: &LocalFile| object_key(&opts.prefix, &opts.server, &file.rel);
    for file in &candidates {
        if let Some(record) = records.get_mut(&key_of(file)) {
            record.evicted = true;
        }
    }
    let mut manifest = Manifest {
        version: MANIFEST_VERSION,
        bucket: opts.bucket.clone(),
        prefix: opts.prefix.clone(),
        objects: records,
    };
    if let Err(e) = manifest.save(manifest_path) {
        warn!(path = %manifest_path.display(), error = %e, "manifest not written; nothing evicted");
        report.evict_failed += candidates.len();
        return;
    }
    if candidates.is_empty() {
        return;
    }

    let mut truncations = iter(candidates.into_iter().map(|file| async move {
        let blocking = file.clone();
        let result = spawn_blocking(move || truncate_unchanged(&blocking))
            .await
            .map_err(Error::other)
            .and_then(|r| r);
        (file, result)
    }))
    .buffer_unordered(opts.hash_threads.max(1));
    let mut unmarked = 0usize;
    while let Some((file, result)) = truncations.next().await {
        match result {
            Ok(()) => {
                report.evicted += 1;
                report.freed_bytes += file.size;
            }
            Err(e) => {
                warn!(rel = %file.rel, error = %e, "eviction failed; the file keeps its bytes");
                report.evict_failed += 1;
                if let Some(record) = manifest.objects.get_mut(&key_of(&file)) {
                    record.evicted = false;
                    unmarked += 1;
                }
            }
        }
    }
    drop(truncations);
    if unmarked > 0
        && let Err(e) = manifest.save(manifest_path)
    {
        warn!(path = %manifest_path.display(), error = %e, "manifest not rewritten after failed evictions");
    }
}
