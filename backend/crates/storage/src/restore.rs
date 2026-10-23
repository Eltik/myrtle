//! Brings evicted stubs back: the way out of eviction.
//!
//! For every manifest record marked evicted whose local file exists with 0
//! bytes, the object is fetched from the store into `<file>.<pid>.tmp` in the
//! same directory, its size and MD5 are checked against the record, and the
//! temp file is renamed over the stub, unless the stub gained bytes in the
//! meantime (an extract rewrote it), in which case the download is dropped.
//! A missing file is not restored: the orphan sweep deleted it on purpose,
//! and the next sync deletes its object. The manifest is saved once at the
//! end, under the same per-server lock `sync` takes; a run that dies part
//! way leaves the remaining records marked, so running it again picks up
//! where it stopped. It does not read `S3_EVICT_LOCAL`, so it works with
//! eviction switched off.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::StreamExt;
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

use crate::key::server_prefix;
use crate::store::{GetOptions, ObjectStore, StorageError};
use crate::sync::{MANIFEST_VERSION, Manifest, OBJECT_RETRIES, RETRY_BASE, ServerLock, SyncError};

#[derive(Debug, Clone)]
pub struct RestoreOptions {
    /// The `assets/output` directory holding one tree per server.
    pub root: PathBuf,
    pub server: String,
    /// `S3_BUCKET`, checked against the manifest's.
    pub bucket: String,
    pub prefix: String,
    pub concurrency: usize,
    pub dry_run: bool,
    /// Rel-path prefixes to restore; empty restores every stub.
    pub only: Vec<String>,
    pub retry_base: Duration,
}

impl RestoreOptions {
    #[must_use]
    pub const fn new(
        root: PathBuf,
        server: String,
        bucket: String,
        prefix: String,
        concurrency: usize,
    ) -> Self {
        Self {
            root,
            server,
            bucket,
            prefix,
            concurrency,
            dry_run: false,
            only: Vec::new(),
            retry_base: RETRY_BASE,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreReport {
    pub server: String,
    /// Restored, or in a dry run, would be restored.
    pub restored: usize,
    pub restored_bytes: u64,
    pub failed: usize,
    /// Evicted records whose local file is gone: left marked, not restored.
    pub missing: usize,
    /// Stubs that gained bytes while their object downloaded: left alone.
    pub skipped: usize,
    pub dry_run: bool,
    pub elapsed_secs: f64,
    /// The first failures, `rel: error`, for the log.
    pub failures: Vec<String>,
}

/// At most this many failure messages ride in the report.
const MAX_REPORTED_FAILURES: usize = 20;

impl RestoreReport {
    /// 0 all restored, 1 one or more failed.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        if self.failed > 0 { 1 } else { 0 }
    }

    /// `asset-sync: restore server=en restored=N (B bytes) failed=N elapsed=S.s`,
    /// with ` missing=N skipped=N` appended when either is above 0.
    #[must_use]
    pub fn summary_line(&self) -> String {
        let extra = if self.missing > 0 || self.skipped > 0 {
            format!(" missing={} skipped={}", self.missing, self.skipped)
        } else {
            String::new()
        };
        format!(
            "asset-sync: restore server={} restored={} ({} bytes) failed={} elapsed={:.1}s{extra}{}",
            self.server,
            self.restored,
            self.restored_bytes,
            self.failed,
            self.elapsed_secs,
            if self.dry_run { " (dry run)" } else { "" },
        )
    }
}

struct Stub {
    key: String,
    rel: String,
    path: PathBuf,
    size: u64,
    md5: String,
}

/// The stubs `opts` selects (records marked evicted, under the server's key
/// prefix and an `--only` prefix, whose local file exists with 0 bytes), and
/// how many selected records have no local file at all.
fn stubs(manifest: &Manifest, opts: &RestoreOptions, server_dir: &Path) -> (Vec<Stub>, usize) {
    let key_prefix = server_prefix(&opts.prefix, &opts.server);
    let mut missing = 0usize;
    let mut out = Vec::new();
    for (key, r) in manifest.objects.iter().filter(|(_, r)| r.evicted) {
        let Some(rel) = key.strip_prefix(&key_prefix) else {
            warn!(key, prefix = %key_prefix, "evicted record outside the key prefix; skipped");
            continue;
        };
        let selected = opts.only.is_empty()
            || opts
                .only
                .iter()
                .any(|p| rel.starts_with(p.trim_start_matches('/')));
        if !selected {
            continue;
        }
        let path = server_dir.join(rel);
        match std::fs::metadata(&path) {
            Ok(m) if m.is_file() && m.len() == 0 => out.push(Stub {
                key: key.clone(),
                rel: rel.to_owned(),
                path,
                size: r.size,
                md5: r.md5.clone(),
            }),
            Ok(_) => {}
            Err(_) => {
                // Deleted on purpose by the orphan sweep: the next sync
                // deletes the object, and the record goes with it.
                info!(rel, "evicted record with no local file; not restored");
                missing += 1;
            }
        }
    }
    (out, missing)
}

/// What one fetch ended as, when it did not fail.
enum Fetched {
    Restored,
    /// The stub gained bytes before the rename; the download was dropped.
    Skipped,
}

/// Downloads `stub` into a temp file beside it, checks size and MD5 against
/// the record, and renames it over the stub.
async fn fetch_once(store: &dyn ObjectStore, stub: &Stub) -> Result<Fetched, StorageError> {
    use md5::{Digest, Md5};
    let got = store.get(&stub.key, GetOptions::default()).await?;
    if got.status != 200 {
        return Err(StorageError::Request(format!(
            "store answered {} for a plain GET",
            got.status
        )));
    }
    let file_name = stub
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = stub
        .path
        .with_file_name(format!("{file_name}.{}.tmp", std::process::id()));
    let result = async {
        let mut file = tokio::fs::File::create(&tmp).await?;
        let mut hasher = Md5::new();
        let mut written = 0u64;
        let mut body = got.body;
        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            hasher.update(&chunk);
            written += chunk.len() as u64;
            file.write_all(&chunk).await?;
        }
        file.sync_all().await?;
        drop(file);
        if written != stub.size {
            return Err(StorageError::Request(format!(
                "got {written} bytes, the manifest records {}",
                stub.size
            )));
        }
        let md5 = hex::encode(hasher.finalize());
        if !md5.eq_ignore_ascii_case(&stub.md5) {
            return Err(StorageError::Request(format!(
                "got MD5 {md5}, the manifest records {}",
                stub.md5
            )));
        }
        // An extract may have rewritten the stub while the object
        // downloaded; its bytes win over the bucket's.
        match tokio::fs::metadata(&stub.path).await {
            Ok(m) if m.is_file() && m.len() == 0 => {}
            _ => return Ok(Fetched::Skipped),
        }
        tokio::fs::rename(&tmp, &stub.path).await?;
        if let Some(parent) = stub.path.parent()
            && let Ok(dir) = tokio::fs::File::open(parent).await
        {
            let _ = dir.sync_all().await;
        }
        Ok(Fetched::Restored)
    }
    .await;
    if !matches!(result, Ok(Fetched::Restored)) {
        let _ = tokio::fs::remove_file(&tmp).await;
    }
    result
}

async fn fetch_with_retry(
    store: &dyn ObjectStore,
    stub: &Stub,
    retry_base: Duration,
) -> Result<Fetched, StorageError> {
    let mut attempt = 0;
    loop {
        match fetch_once(store, stub).await {
            Ok(done) => return Ok(done),
            // A missing object will not come back by waiting.
            Err(e @ StorageError::NotFound(_)) => return Err(e),
            Err(e) if attempt < OBJECT_RETRIES => {
                let wait = retry_base * 2u32.pow(attempt);
                warn!(key = %stub.key, attempt = attempt + 1, error = %e, ?wait, "restore failed, retrying");
                tokio::time::sleep(wait).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Restores the evicted stubs of `<opts.root>/<opts.server>`.
///
/// # Errors
/// The server directory is missing, another run holds the server's lock, the
/// manifest exists but cannot be read, or it was written for another bucket
/// or prefix. Per-object failures are counted in the report instead.
pub async fn run_restore(
    store: &dyn ObjectStore,
    opts: &RestoreOptions,
) -> Result<RestoreReport, SyncError> {
    let started = Instant::now();
    let server_dir = opts.root.join(&opts.server);
    if !server_dir.is_dir() {
        return Err(SyncError::MissingServerDir(server_dir));
    }
    // Held until the manifest is saved, so a sync cannot interleave.
    let _lock = if opts.dry_run {
        None
    } else {
        Some(ServerLock::acquire(&server_dir)?)
    };
    let manifest_path = Manifest::path(&opts.root, &opts.server);
    let mut manifest = Manifest::read(&manifest_path).map_err(|error| SyncError::Manifest {
        path: manifest_path.clone(),
        error,
    })?;
    manifest
        .check_store(&opts.bucket, &opts.prefix, &opts.server)
        .map_err(SyncError::StoreMoved)?;
    let (todo, missing) = stubs(&manifest, opts, &server_dir);
    info!(server = %opts.server, stubs = todo.len(), missing, "stubs to restore");

    let mut report = RestoreReport {
        server: opts.server.clone(),
        restored: 0,
        restored_bytes: 0,
        failed: 0,
        missing,
        skipped: 0,
        dry_run: opts.dry_run,
        elapsed_secs: 0.0,
        failures: Vec::new(),
    };
    if opts.dry_run {
        report.restored = todo.len();
        report.restored_bytes = todo.iter().map(|s| s.size).sum();
        report.elapsed_secs = started.elapsed().as_secs_f64();
        return Ok(report);
    }

    let mut results = futures::stream::iter(todo.into_iter().map(|stub| async move {
        let result = fetch_with_retry(store, &stub, opts.retry_base).await;
        (stub, result)
    }))
    .buffer_unordered(opts.concurrency.max(1));
    while let Some((stub, result)) = results.next().await {
        match result {
            Ok(Fetched::Skipped) => {
                info!(rel = %stub.rel, "stub was rewritten during the download; left as is");
                report.skipped += 1;
            }
            Ok(Fetched::Restored) => {
                report.restored += 1;
                report.restored_bytes += stub.size;
                if let Some(record) = manifest.objects.get_mut(&stub.key) {
                    record.evicted = false;
                }
            }
            Err(e) => {
                report.failed += 1;
                warn!(rel = %stub.rel, error = %e, "restore failed");
                if report.failures.len() < MAX_REPORTED_FAILURES {
                    report.failures.push(format!("{}: {e}", stub.rel));
                }
            }
        }
    }
    drop(results);

    if report.restored > 0 {
        manifest.version = MANIFEST_VERSION;
        manifest.bucket.clone_from(&opts.bucket);
        manifest.prefix.clone_from(&opts.prefix);
        if let Err(e) = manifest.save(&manifest_path) {
            // The restored files have their bytes; their records still say
            // evicted, which sync treats as ordinary files and a re-run of
            // restore skips.
            warn!(path = %manifest_path.display(), error = %e, "manifest not written");
        }
    }
    report.elapsed_secs = started.elapsed().as_secs_f64();
    Ok(report)
}
