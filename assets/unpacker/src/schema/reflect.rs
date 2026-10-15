//! Verify a buffer against a `.fbs` without generating any Rust: compile the
//! schema to a binary schema (`flatc -b --schema`) once, then walk the buffer
//! with the vendored `flatbuffers-reflection` verifier under
//! [`verifier_opts`](super::opts::verifier_opts), the options the generated
//! decoder uses.

use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};

use super::opts::verifier_opts;
use super::store::normalize_fbs;

/// Lower-case hex sha256.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// The `flatc` version string (`25.12.19`), or `None` when flatc is missing.
#[must_use]
pub fn flatc_version() -> Option<String> {
    let out = Command::new("flatc").arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace().last().map(str::to_string)
}

/// The default binary-schema cache: `unpacker-bfbs-cache` in the system temp
/// directory. Entries are keyed by flatc version and normalised source, so a
/// shared cache never serves a stale schema.
#[must_use]
pub fn default_cache_dir() -> PathBuf {
    std::env::temp_dir().join("unpacker-bfbs-cache")
}

/// A compiled binary schema, or flatc's complaint about the source.
pub type Compiled = Result<Arc<Vec<u8>>, String>;

/// Compiles schemas to `.bfbs` once per distinct normalised source.
///
/// The key is the sha256 of `flatc <version>\n` + the normalised schema, so two
/// blobs that differ only in comments share one entry and a flatc upgrade
/// cannot serve a stale binary schema. Entries live as `<key>.bfbs` in the
/// cache directory; a source flatc rejects is remembered as `<key>.flatc-error`.
pub struct BfbsCache {
    dir: PathBuf,
    flatc: String,
    memo: Mutex<HashMap<String, Compiled>>,
    /// Held while flatc runs. Two tables whose schemas normalise to the same text
    /// (`init_text` and `main_text`) share a key; compiled in parallel they shared a
    /// work directory too, and the first to finish deleted the second's input under
    /// flatc, which was then cached as a rejection (measured 2026-10-15: `main_text`
    /// "verifies under no candidate" on all four servers in one run of three).
    compile_lock: Mutex<()>,
    /// Makes every work directory unique even under the lock.
    work_seq: AtomicU64,
}

impl BfbsCache {
    pub fn new(dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let flatc = flatc_version().ok_or_else(|| anyhow::anyhow!("flatc not found on PATH"))?;
        Ok(Self {
            dir: dir.to_path_buf(),
            flatc,
            memo: Mutex::new(HashMap::new()),
            compile_lock: Mutex::new(()),
            work_seq: AtomicU64::new(0),
        })
    }

    #[must_use]
    pub fn flatc(&self) -> &str {
        &self.flatc
    }

    /// The cache key of a schema source.
    #[must_use]
    pub fn key(&self, src: &[u8]) -> String {
        let norm = normalize_fbs(&String::from_utf8_lossy(src));
        sha256_hex(format!("flatc {}\n{norm}", self.flatc).as_bytes())
    }

    /// The binary schema for `src`, a `.fbs` whose file stem is `stem`.
    pub fn compile(&self, stem: &str, src: &[u8]) -> Compiled {
        let key = self.key(src);
        if let Some(hit) = self.memo.lock().expect("bfbs memo poisoned").get(&key) {
            return hit.clone();
        }
        let _serial = self.compile_lock.lock().expect("compile lock poisoned");
        // A parallel caller may have compiled this key while we waited.
        if let Some(hit) = self.memo.lock().expect("bfbs memo poisoned").get(&key) {
            return hit.clone();
        }
        let result = self.compile_uncached(&key, stem, src);
        self.memo
            .lock()
            .expect("bfbs memo poisoned")
            .insert(key, result.clone());
        result
    }

    fn compile_uncached(&self, key: &str, stem: &str, src: &[u8]) -> Compiled {
        let bfbs_path = self.dir.join(format!("{key}.bfbs"));
        let err_path = self.dir.join(format!("{key}.flatc-error"));
        if let Ok(bytes) = std::fs::read(&bfbs_path) {
            return Ok(Arc::new(bytes));
        }
        if let Ok(err) = std::fs::read_to_string(&err_path) {
            return Err(err);
        }
        let seq = self.work_seq.fetch_add(1, Ordering::Relaxed);
        let work = self
            .dir
            .join(format!("tmp-{}-{seq}-{key}", std::process::id()));
        let run = || -> Result<Vec<u8>, String> {
            std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
            let fbs = work.join(format!("{stem}.fbs"));
            std::fs::write(&fbs, src).map_err(|e| e.to_string())?;
            let out = Command::new("flatc")
                .arg("-b")
                .arg("--schema")
                .arg("-o")
                .arg(&work)
                .arg(&fbs)
                .output()
                .map_err(|e| format!("flatc: {e}"))?;
            if !out.status.success() {
                let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
                return Err(format!("flatc rejected the schema: {msg}"));
            }
            std::fs::read(work.join(format!("{stem}.bfbs"))).map_err(|e| e.to_string())
        };
        let result = run();
        let _ = std::fs::remove_dir_all(&work);
        match result {
            Ok(bytes) => {
                let tmp = self
                    .dir
                    .join(format!("{key}.bfbs.tmp-{}", std::process::id()));
                if std::fs::write(&tmp, &bytes).is_ok() {
                    let _ = std::fs::rename(&tmp, &bfbs_path);
                }
                Ok(Arc::new(bytes))
            }
            Err(e) => {
                // Only a REJECTION is cached; an I/O failure is retried next run.
                if e.starts_with("flatc rejected") {
                    let _ = std::fs::write(&err_path, &e);
                }
                Err(e)
            }
        }
    }
}

/// Verify `data` against a compiled schema. `Err` carries the first line of the
/// verifier's complaint. A verifier panic counts as a failure.
pub fn verify(bfbs: &[u8], data: &[u8]) -> Result<(), String> {
    let schema = flatbuffers_reflection::reflection::root_as_schema(bfbs)
        .map_err(|e| format!("bad bfbs: {e}"))?;
    let opts = verifier_opts();
    match panic::catch_unwind(AssertUnwindSafe(|| {
        flatbuffers_reflection::SafeBuffer::new_with_options(data, &schema, &opts).map(|_| ())
    })) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e.to_string().lines().next().unwrap_or_default().to_string()),
        Err(_) => Err("reflection verifier panicked".to_string()),
    }
}
