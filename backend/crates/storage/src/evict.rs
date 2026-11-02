//! Local eviction: clears files once uploaded to bucket

use std::collections::HashMap;
use std::fs::{OpenOptions, metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use futures::StreamExt;
use futures::stream::iter;
use hex::encode;
use serde::Serialize;
use tokio::task::spawn_blocking;
use tracing::{info, warn};

use crate::config::{ConfigError, StorageConfig, get as get_var};
use crate::key::{object_key, server_prefix};
use crate::store::{ObjectStore, md5_file};
use crate::sync::{
    LocalFile, MANIFEST_VERSION, Manifest, ManifestRecord, ServerLock, SyncError, in_roots,
    save_and_truncate, walk_local,
};

/// Rel-path prefixes (under `<root>/<server>/`) that are
/// never evicted. `STORAGE_EVICT_KEEP`
/// adds to this list and cannot remove from it.
pub const BUILTIN_KEEP: &[&str] = &[
    // Story sprites: hub.json, sprites.json, PNG headers, thumbnail renders,
    // gallery variants
    "textures/avg/",
    // Story art variants
    "textures/spritepack/cutin_char_",
    // `core/release/art.rs` copies these into `derived/release-art`
    "textures/spritepack/ui_home_act_banner_gacha_",
    "textures/spritepack/ui_zone_home_theme_",
    "textures/arts/ui/stage/[uc]homeentry/",
    // `event_banner.rs` composes generated event banners from Story
    // Collection key visuals and titles
    "textures/spritepack/mixstory_",
    // Unpacker reads this
    "audio/m4a_sources.json",
];

/// Whether `sync` evicts, and what it keeps on top of [`BUILTIN_KEEP`]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EvictPolicy {
    /// `STORAGE_EVICT_LOCAL`; off by default
    pub enabled: bool,
    /// `STORAGE_EVICT_KEEP`: extra rel-path prefixes to keep
    pub extra_keep: Vec<String>,
}

impl EvictPolicy {
    /// # Errors
    /// An `STORAGE_EVICT_KEEP` entry containing a backslash.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let enabled = get_var(lookup, "STORAGE_EVICT_LOCAL").is_some_and(|v| {
            matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
        });
        let extra_keep = match get_var(lookup, "STORAGE_EVICT_KEEP") {
            None => Vec::new(),
            Some(raw) => parse_keep(&raw)?,
        };
        Ok(Self {
            enabled,
            extra_keep,
        })
    }

    /// True when `rel` lies under a built-in or an extra keep prefix, or is a
    /// server-tagged home banner ([`is_server_home_banner`])
    #[must_use]
    pub fn keeps(&self, rel: &str) -> bool {
        BUILTIN_KEEP.iter().any(|p| rel.starts_with(p))
            || is_server_home_banner(rel)
            || self.extra_keep.iter().any(|p| rel.starts_with(p.as_str()))
    }
}

/// `textures/[[<anything>]]/arts/ui/homebanners/...`: gacha banners
/// under a server-tagged texture tree
#[must_use]
pub fn is_server_home_banner(rel: &str) -> bool {
    rel.strip_prefix("textures/[[")
        .and_then(|rest| rest.split_once("]]/"))
        .is_some_and(|(_, tail)| tail.starts_with("arts/ui/homebanners/"))
}

/// # Errors
/// An entry with a backslash: keys and rels use forward slashes only.
pub fn parse_keep(raw: &str) -> Result<Vec<String>, ConfigError> {
    let mut out = Vec::new();
    for entry in raw.split(',') {
        let entry = entry.trim().trim_start_matches('/');
        if entry.is_empty() {
            continue;
        }
        if entry.contains('\\') {
            return Err(ConfigError::Invalid {
                var: "STORAGE_EVICT_KEEP",
                value: raw.to_owned(),
                reason: "prefixes use forward slashes",
            });
        }
        out.push(entry.to_owned());
    }
    Ok(out)
}

/// # Errors
/// The file's metadata cannot be read, it cannot be opened for writing, or
/// its mtime cannot be set.
pub fn truncate_in_place(path: &Path) -> io::Result<()> {
    let modified = metadata(path)?.modified()?;
    let file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.set_modified(modified)
}

#[derive(Debug, Clone)]
pub struct EvictOptions {
    /// The `assets/output` directory holding one tree per server
    pub root: PathBuf,
    pub server: String,
    /// `CLOUDFLARE_BUCKET`
    pub bucket: String,
    pub prefix: String,
    /// Served roots whose files may be stubbed
    pub roots: Vec<String>,
    pub backup_roots: Vec<String>,
    /// Keep list on top of [`BUILTIN_KEEP`]
    pub keep: EvictPolicy,
    pub hash_threads: usize,
    pub dry_run: bool,
}

impl EvictOptions {
    /// Options with every tunable taken from `config`
    #[must_use]
    pub fn from_config(config: &StorageConfig, root: PathBuf, server: String) -> Self {
        Self {
            root,
            server,
            bucket: config.bucket.clone(),
            prefix: config.prefix.clone(),
            roots: config.roots.clone(),
            backup_roots: config.backup_roots.clone(),
            keep: config.evict.clone(),
            hash_threads: config.hash_threads,
            dry_run: false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EvictReport {
    pub server: String,
    /// Files walked under the served roots
    pub scanned: usize,
    /// Files truncated to stubs
    pub stubbed: usize,
    /// Bytes those truncations freed
    pub freed_bytes: u64,
    /// Files whose size or MD5 differs from the stored object, or whose object
    /// has a multipart `ETag`
    pub mismatched: usize,
    /// Files with no object in the bucket
    pub missing: usize,
    /// Files the keep list or a backup root protects
    pub kept: usize,
    /// Files already 0 bytes
    pub empty: usize,
    /// Hashes or truncations that failed
    pub failed: usize,
    pub missing_roots: Vec<String>,
    pub dry_run: bool,
    pub elapsed_secs: f64,
}

impl EvictReport {
    /// 0 done, 2 one or more files failed
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        if self.failed > 0 { 2 } else { 0 }
    }

    /// `asset-sync: evict server=en scanned=N stubbed=N (B bytes freed) mismatched=N missing=N kept=N empty=N failed=N elapsed=S.s`
    #[must_use]
    pub fn summary_line(&self) -> String {
        format!(
            "asset-sync: evict server={} scanned={} stubbed={} ({} bytes freed) mismatched={} missing={} kept={} empty={} failed={} elapsed={:.1}s{}",
            self.server,
            self.scanned,
            self.stubbed,
            self.freed_bytes,
            self.mismatched,
            self.missing,
            self.kept,
            self.empty,
            self.failed,
            self.elapsed_secs,
            if self.dry_run { " (dry run)" } else { "" },
        )
    }
}

/// Stubs each file under `opts.roots` whose size and MD5 match the object the
/// store already holds
///
/// # Errors
/// The server directory is missing, another run holds the server's lock, the
/// local walk fails, the remote cannot be listed, or the manifest is
/// unreadable or was written for another bucket or prefix.
#[allow(clippy::too_many_lines)]
pub async fn run_evict(
    store: &dyn ObjectStore,
    opts: &EvictOptions,
) -> Result<EvictReport, SyncError> {
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

    let manifest_path = Manifest::path(&opts.root, &opts.server);
    let old_manifest = Manifest::read(&manifest_path).map_err(|error| SyncError::Manifest {
        path: manifest_path.clone(),
        error,
    })?;
    old_manifest
        .check_store(&opts.bucket, &opts.prefix, &opts.server)
        .map_err(SyncError::StoreMoved)?;

    let walk_dir = server_dir.clone();
    let roots: Vec<String> = opts
        .roots
        .iter()
        .filter(|r| !opts.backup_roots.contains(r))
        .cloned()
        .collect();
    let walked = spawn_blocking(move || walk_local(&walk_dir, &roots))
        .await
        .map_err(|e| SyncError::Walk(e.to_string()))?
        .map_err(SyncError::Walk)?;

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
    info!(server = %opts.server, files = walked.files.len(), objects = remote.len(), "evict: tree walked and remote listed");

    let mut report = EvictReport {
        server: opts.server.clone(),
        scanned: walked.files.len(),
        stubbed: 0,
        freed_bytes: 0,
        mismatched: 0,
        missing: 0,
        kept: 0,
        empty: 0,
        failed: 0,
        missing_roots: walked.missing_roots,
        dry_run: opts.dry_run,
        elapsed_secs: 0.0,
    };
    let key_of = |rel: &str| object_key(&opts.prefix, &opts.server, rel);
    let mut records = old_manifest.objects;
    let mut changed = false;
    let mut to_hash: Vec<(LocalFile, String)> = Vec::new();
    for file in walked.files {
        if file.size == 0 {
            report.empty += 1;
            continue;
        }
        let etag = if opts.keep.keeps(&file.rel) || in_roots(&file.rel, &opts.backup_roots) {
            report.kept += 1;
            None
        } else {
            match remote.get(&file.rel) {
                None => {
                    report.missing += 1;
                    None
                }
                Some((size, Some(etag))) if *size == file.size && !etag.contains('-') => {
                    Some(etag.clone())
                }
                Some(_) => {
                    report.mismatched += 1;
                    None
                }
            }
        };
        match etag {
            Some(etag) => to_hash.push((file, etag)),
            None => {
                if let Some(record) = records.get_mut(&key_of(&file.rel))
                    && record.evicted
                {
                    record.evicted = false;
                    changed = true;
                }
            }
        }
    }

    let mut hashed = iter(to_hash.into_iter().map(|(file, etag)| async move {
        let path = file.path.clone();
        let md5 = spawn_blocking(move || md5_file(&path))
            .await
            .map_err(io::Error::other)
            .and_then(|r| r)
            .map(encode);
        (file, etag, md5)
    }))
    .buffer_unordered(opts.hash_threads.max(1));
    let mut candidates: Vec<LocalFile> = Vec::new();
    while let Some((file, etag, md5)) = hashed.next().await {
        let key = key_of(&file.rel);
        match md5 {
            Ok(md5) if etag.eq_ignore_ascii_case(&md5) => {
                records.insert(
                    key,
                    ManifestRecord {
                        size: file.size,
                        md5,
                        etag,
                        evicted: false,
                    },
                );
                candidates.push(file);
            }
            other => {
                if let Err(e) = other {
                    warn!(rel = %file.rel, error = %e, "hashing failed; the file keeps its bytes");
                    report.failed += 1;
                } else {
                    report.mismatched += 1;
                }
                if let Some(record) = records.get_mut(&key)
                    && record.evicted
                {
                    record.evicted = false;
                    changed = true;
                }
            }
        }
    }
    drop(hashed);

    if opts.dry_run {
        report.stubbed = candidates.len();
        report.freed_bytes = candidates.iter().map(|f| f.size).sum();
    } else if changed || !candidates.is_empty() {
        let manifest = Manifest {
            version: MANIFEST_VERSION,
            bucket: opts.bucket.clone(),
            prefix: opts.prefix.clone(),
            objects: records,
        };
        let done = save_and_truncate(
            &manifest_path,
            manifest,
            &opts.prefix,
            &opts.server,
            candidates,
            opts.hash_threads,
        )
        .await;
        report.stubbed = done.evicted;
        report.freed_bytes = done.freed_bytes;
        report.failed += done.failed;
    }
    report.elapsed_secs = started.elapsed().as_secs_f64();
    Ok(report)
}
