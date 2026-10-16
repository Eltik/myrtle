//! Phase 0 embed: turn `chunks.jsonl` into `vectors.bin` plus
//! `vectors.meta.json`.
//!
//! Three decisions, each measured or read from source rather than assumed:
//!
//! * **One text per inference, always.** The INT8 export is dynamically
//!   quantized, and `DynamicQuantizeLinear` computes its scale over the WHOLE
//!   input tensor, so a text's embedding depends on which texts share its
//!   batch (and on their padding). Measured on a one-layer graph with
//!   onnxruntime 1.25: the same row embedded alone and beside a different
//!   batch-mate differs (cosine 0.999977; 0.999823 beside a 5x-scaled mate),
//!   while embedding it alone twice is bit-identical. A query is embedded
//!   alone at serve time, so documents are embedded alone here too, and the
//!   two sides are computed the same way. fastembed enforces the same rule:
//!   with `QuantizationMode::Dynamic` it refuses any batch smaller than its
//!   input.
//! * **CLS pooling.** `gte-modernbert-base/1_Pooling/config.json` sets
//!   `pooling_mode_cls_token: true`. Mean pooling would degrade retrieval
//!   without any error.
//! * **Unit-norm output, asserted.** fastembed normalizes every row in its
//!   default output transformer (`text_embedding/output.rs`), so cosine is a
//!   bare dot product. Each vector's norm is still checked here, because a
//!   silent change upstream would otherwise show up only as bad retrieval.
//!
//! The vectors are refused unless `tokenizer.json` in the model directory is
//! byte-identical to the one the chunks were counted with: a mismatched
//! tokenizer makes every stored `token_count` a lie.
//!
//! Rows whose embedded text is unchanged since a prior run are copied from
//! that run's `vectors.bin` instead of embedded (`corpus::reuse`). This is
//! exact only BECAUSE of the one-text-per-call rule above: a vector depends on
//! its text and the model alone, so the stored bytes are the bytes a fresh
//! call would return. Batching would break that silently. `reuse: false`
//! (`--no-reuse`) embeds every row, as before reuse existed.
//!
//! Long runs are paced and resumable (2026-10-16, design/trevor-reembed.md):
//!
//! * **Checkpoint.** Every embedded row is appended to `vectors.ckpt` in the
//!   corpus directory (16-byte key, u32 dim, the f32 row), flushed per row and
//!   fsynced every `checkpoint_every` rows; `vectors.ckpt.json` names the
//!   model and tokenizer it belongs to. A rerun after an interruption reads
//!   the valid prefix of that file (a torn or non-unit record ends it) and
//!   embeds only what is missing; the file goes once the run completes. With
//!   checkpointing on, the old `vectors.meta.json` stays until the new
//!   `vectors.bin` is renamed in, so the directory's own prior vectors serve
//!   and are reused across a resume. `checkpoint_every: 0` is the old run:
//!   no checkpoint written or read, the old meta removed first.
//! * **Guard.** Before an embedding call, at most every `Guard::every_secs`,
//!   the run pauses while the machine reports thermal or memory pressure, is
//!   on battery (macOS), or is short of memory or CPU (Linux).
//! * **Duty cycle.** After each embedding call taking `w`, the run sleeps
//!   `w * (1 - duty) / duty`, so inference takes at most `duty` of wall time.
//!   `duty: 1.0` never sleeps.
//!
//! None of the three changes a vector: they decide only when a row is
//! embedded, never how.
//!
//! The model file is part of a vector's identity: `vectors.meta.json` records
//! `modelFile` and `modelSha`, and [`check_query_model`] refuses a query
//! embedder whose sha differs, because the INT8 export's vectors move with the
//! CPU (cosine 0.94 to fp32 on x86, 0.997 on Apple Silicon).

use std::collections::HashMap;
use std::io::{BufRead, BufWriter, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use fastembed::{
    InitOptionsUserDefined, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde::{Deserialize, Serialize};

use crate::corpus::chunk::indexed_text;
use crate::corpus::reuse::{self, ReusePool};
use crate::util::sha_hex;

/// gte-modernbert-base's trained context. A chunk longer than this would be
/// truncated; the longest real chunk is 1,355 tokens.
pub const MODEL_MAX_TOKENS: u32 = 8192;

pub struct EmbedConfig {
    /// Directory holding the ONNX file and the four tokenizer files.
    pub model_dir: PathBuf,
    /// ONNX file name inside `model_dir`, e.g. `model_int8.onnx`.
    pub onnx_file: String,
    /// Directory holding `chunks.jsonl` and `manifest.p0.json`.
    pub corpus: PathBuf,
    pub intra_threads: Option<usize>,
    /// Embed `text` alone even when chunks carry a prefix, so BM25 can index
    /// the prefix while the vectors stay P0's. Off by default.
    pub ignore_prefix: bool,
    /// Copy the vector of every row whose embedded text is unchanged from
    /// `corpus`'s own prior vectors and from `reuse_from`. False embeds every
    /// row.
    pub reuse: bool,
    /// Further directories whose vectors may be reused, after `corpus`: the
    /// P0 directory for a derived corpus whose rows start with P0's.
    pub reuse_from: Vec<PathBuf>,
    /// Pause while the machine is under pressure. None never pauses.
    pub guard: Option<Guard>,
    /// Largest share of wall time spent inside embedding calls, in (0, 1].
    /// 1.0 never sleeps.
    pub duty: f64,
    /// Stop cleanly, checkpoint synced, once this much wall time has passed.
    pub max_minutes: Option<f64>,
    /// fsync the checkpoint every this many embedded rows. 0 = no checkpoint
    /// (written or read) and the old meta removed first, as before.
    pub checkpoint_every: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChunkLine {
    chunk_id: String,
    text: String,
    token_count: u32,
    /// P1 contextual prefix; absent in P0, where the embedded text is `text`.
    #[serde(default)]
    prefix: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct P0Manifest {
    tokenizer_sha: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorsMeta {
    pub model_file: String,
    pub model_sha: String,
    pub tokenizer_sha: String,
    pub chunks_sha: String,
    pub dim: usize,
    pub count: usize,
    pub dtype: &'static str,
    pub layout: &'static str,
    pub pooling: &'static str,
    pub normalized: bool,
    pub batch_size: usize,
    pub max_norm_error: f32,
    pub built_at_unix: u64,
    pub seconds: f64,
    /// Row order of `vectors.bin`.
    pub chunk_ids: Vec<String>,
    /// Reuse key of each row, in row order: sha16 of the text embedded, so
    /// the row's `contentSha` when no prefix was embedded.
    pub content_shas: Vec<String>,
    /// Rows copied from prior vectors and rows embedded this run.
    pub reused: usize,
    pub embedded: usize,
    /// Of `embedded`, the rows read back from an interrupted run's checkpoint.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub resumed: usize,
    /// True when chunks carried prefixes and they were not embedded.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub prefix_ignored: bool,
}

fn read(dir: &Path, name: &str) -> Result<Vec<u8>> {
    std::fs::read(dir.join(name)).with_context(|| format!("reading {}", dir.join(name).display()))
}

/// A loaded embedder and the hashes that identify it.
pub struct LoadedModel {
    pub embedding: TextEmbedding,
    pub model_sha: String,
    pub tokenizer_sha: String,
}

/// Load `onnx_file` from `model_dir` with the four tokenizer files beside it,
/// CLS pooling, and batching refused (see the module docs).
///
/// # Errors
/// Missing files or an ONNX Runtime load failure.
pub fn load_model(
    model_dir: &Path,
    onnx_file: &str,
    intra_threads: Option<usize>,
) -> Result<LoadedModel> {
    let tokenizer_file = read(model_dir, "tokenizer.json")?;
    let tokenizer_sha = sha_hex(&tokenizer_file);
    let onnx = read(model_dir, onnx_file)?;
    let model_sha = sha_hex(&onnx);
    let files = TokenizerFiles {
        tokenizer_file,
        config_file: read(model_dir, "config.json")?,
        special_tokens_map_file: read(model_dir, "special_tokens_map.json")?,
        tokenizer_config_file: read(model_dir, "tokenizer_config.json")?,
    };
    let model = UserDefinedEmbeddingModel::new(onnx, files)
        .with_pooling(Pooling::Cls)
        // Declared Dynamic so fastembed itself refuses to batch, which is the
        // property this module depends on.
        .with_quantization(QuantizationMode::Dynamic);
    let mut opts = InitOptionsUserDefined::new().with_max_length(MODEL_MAX_TOKENS as usize);
    if let Some(n) = intra_threads {
        opts = opts.with_intra_threads(n);
    }
    let embedding = TextEmbedding::try_new_from_user_defined(model, opts)
        .map_err(|e| anyhow!("loading the embedder: {e}"))?;
    Ok(LoadedModel {
        embedding,
        model_sha,
        tokenizer_sha,
    })
}

/// Embeds queries the way the corpus was embedded: one text per call, CLS,
/// unit length.
///
/// The tokenizer may not differ from the corpus's; that is refused here. The
/// ONNX file was allowed to differ until 2026-10-16 (Mac INT8 corpus, fp32
/// queries on x86, cosine 0.997); since then every corpus moves to the fp32
/// export and [`check_query_model`] refuses a query model other than the
/// vectors' own.
pub struct QueryEmbedder {
    embedding: TextEmbedding,
    pub model_sha: String,
    pub onnx_file: String,
}

impl QueryEmbedder {
    /// # Errors
    /// Load failures, or a tokenizer other than the one the vectors used.
    pub fn load(
        model_dir: &Path,
        onnx_file: &str,
        intra_threads: Option<usize>,
        vectors_tokenizer_sha: &str,
    ) -> Result<Self> {
        let m = load_model(model_dir, onnx_file, intra_threads)?;
        if m.tokenizer_sha != vectors_tokenizer_sha {
            bail!(
                "query tokenizer {:.16} differs from the one the vectors used ({:.16})",
                m.tokenizer_sha,
                vectors_tokenizer_sha
            );
        }
        Ok(Self {
            embedding: m.embedding,
            model_sha: m.model_sha,
            onnx_file: onnx_file.to_owned(),
        })
    }

    /// # Errors
    /// An inference failure or an empty result.
    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>> {
        self.embedding
            .embed([text], None)
            .map_err(|e| anyhow!("embedding the query: {e}"))?
            .pop()
            .ok_or_else(|| anyhow!("no vector for the query"))
    }
}

/// Setting this to `1` turns [`check_query_model`]'s refusal into a warning
/// (the kill switch: query and corpus models may differ, as before
/// 2026-10-16).
pub const ALLOW_QUERY_MODEL_MISMATCH_ENV: &str = "TREVOR_ALLOW_QUERY_MODEL_MISMATCH";

/// The query ONNX file to load: `requested`, or the vectors' own `modelFile`
/// when `requested` is `auto`.
#[must_use]
pub fn resolve_query_onnx<'a>(requested: &'a str, vectors_model_file: &'a str) -> &'a str {
    if requested == "auto" { vectors_model_file } else { requested }
}

/// Refuse a query embedder whose model sha differs from the one the corpus
/// vectors were built with (`vectors.meta.json` `modelFile`, `modelSha`):
/// INT8 vectors move with the CPU, so mixing models or machines silently
/// degrades dense retrieval. `allow_mismatch`, or
/// [`ALLOW_QUERY_MODEL_MISMATCH_ENV`]`=1`, logs the mismatch and allows it.
///
/// # Errors
/// The two shas differ and no override is set.
pub fn check_query_model(
    vectors_model_file: &str,
    vectors_model_sha: &str,
    query: &QueryEmbedder,
    allow_mismatch: bool,
) -> Result<()> {
    if query.model_sha == vectors_model_sha {
        return Ok(());
    }
    let msg = format!(
        "query model {} ({:.16}) is not the model the vectors were built with, {} ({:.16})",
        query.onnx_file, query.model_sha, vectors_model_file, vectors_model_sha
    );
    let env = std::env::var(ALLOW_QUERY_MODEL_MISMATCH_ENV).is_ok_and(|v| v == "1");
    if allow_mismatch || env {
        eprintln!("warning: {msg}; allowed by override");
        return Ok(());
    }
    bail!("{msg}; pass the vectors' model file (or `auto`), or set {ALLOW_QUERY_MODEL_MISMATCH_ENV}=1")
}

/// Pauses a long embedding run while the machine is under pressure. Every
/// probe reads only what the OS already publishes (no sudo, no
/// `powermetrics`); a probe that cannot run is skipped, never a pause.
#[derive(Debug, Clone)]
pub struct Guard {
    /// Linux: pause while `MemAvailable` (/proc/meminfo) is below this, GiB.
    pub min_available_gib: f64,
    /// Linux: pause while /proc/pressure/cpu `some avg10` is above this, %.
    pub max_cpu_pressure: f64,
    /// Linux: pause while the 1-minute load average is above the number of
    /// online CPUs (the machine's, not this process's affinity mask).
    pub check_load: bool,
    /// macOS: pause while on battery power (`pmset -g batt`).
    pub pause_on_battery: bool,
    /// Seconds between probes while running.
    pub every_secs: f64,
    /// Seconds between probes while paused.
    pub pause_secs: f64,
}

impl Default for Guard {
    fn default() -> Self {
        Self {
            min_available_gib: 1.5,
            max_cpu_pressure: 40.0,
            check_load: true,
            pause_on_battery: true,
            every_secs: 10.0,
            pause_secs: 30.0,
        }
    }
}

fn probe(prog: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(prog).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A recorded warning in `pmset -g therm` output, or None. Apple Silicon
/// prints "Note: No thermal warning level has been recorded" (and the same for
/// performance warnings and CPU power status) when nothing is recorded; any
/// other line naming a warning level or power status, or a `*_Limit` below
/// 100 (Intel's CPU_Speed_Limit), is a reason to pause.
#[must_use]
pub fn therm_warning(text: &str) -> Option<String> {
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("Note: No") {
            continue;
        }
        let lower = line.to_ascii_lowercase();
        if lower.contains("warning level") || lower.contains("power status") {
            return Some(line.to_owned());
        }
        if let Some((k, v)) = line.split_once('=')
            && k.trim().ends_with("_Limit")
            && v.trim().parse::<f64>().is_ok_and(|x| x < 100.0)
        {
            return Some(line.to_owned());
        }
    }
    None
}

/// `MemAvailable` from /proc/meminfo, in GiB.
#[must_use]
pub fn mem_available_gib(meminfo: &str) -> Option<f64> {
    let line = meminfo.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / (1024.0 * 1024.0))
}

/// `some avg10` from /proc/pressure/cpu, in percent.
#[must_use]
pub fn cpu_some_avg10(psi: &str) -> Option<f64> {
    let line = psi.lines().find(|l| l.starts_with("some "))?;
    line.split_whitespace().find_map(|f| f.strip_prefix("avg10=")?.parse().ok())
}

/// The online CPUs counted in /proc/stat (`cpu0`, `cpu1`, ...).
#[must_use]
pub fn online_cpus(stat: &str) -> usize {
    stat.lines()
        .filter(|l| l.strip_prefix("cpu").is_some_and(|r| r.starts_with(|c: char| c.is_ascii_digit())))
        .count()
}

impl Guard {
    /// Every reason to pause right now; empty when the run may continue.
    #[must_use]
    pub fn reasons(&self) -> Vec<String> {
        let mut why = Vec::new();
        if cfg!(target_os = "macos") {
            if let Some(t) = probe("/usr/bin/pmset", &["-g", "therm"]).as_deref().and_then(therm_warning) {
                why.push(format!("pmset therm: {t}"));
            }
            if let Some(level) = probe("/usr/sbin/sysctl", &["-n", "kern.memorystatus_vm_pressure_level"])
                .and_then(|s| s.trim().parse::<u32>().ok())
                && level >= 2
            {
                why.push(format!("memory pressure level {level} (2 warn, 4 critical)"));
            }
            if self.pause_on_battery
                && probe("/usr/bin/pmset", &["-g", "batt"]).is_some_and(|s| s.contains("'Battery Power'"))
            {
                why.push("on battery power".to_owned());
            }
        } else if cfg!(target_os = "linux") {
            let read = |p: &str| std::fs::read_to_string(p).ok();
            if let Some(g) = read("/proc/meminfo").as_deref().and_then(mem_available_gib)
                && g < self.min_available_gib
            {
                why.push(format!("MemAvailable {g:.2} GiB < {:.2}", self.min_available_gib));
            }
            if let Some(p) = read("/proc/pressure/cpu").as_deref().and_then(cpu_some_avg10)
                && p > self.max_cpu_pressure
            {
                why.push(format!("cpu pressure some avg10 {p:.1} > {:.1}", self.max_cpu_pressure));
            }
            if self.check_load {
                let n = read("/proc/stat").as_deref().map_or(0, online_cpus);
                let load = read("/proc/loadavg")
                    .and_then(|s| s.split_whitespace().next().and_then(|x| x.parse::<f64>().ok()));
                #[allow(clippy::cast_precision_loss)]
                if let Some(l) = load
                    && n > 0
                    && l > n as f64
                {
                    why.push(format!("load average {l:.2} > {n} CPUs"));
                }
            }
        }
        why
    }
}

const CKPT_FILE: &str = "vectors.ckpt";
const CKPT_HEAD: &str = "vectors.ckpt.json";
const KEY_LEN: usize = 16;

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CkptHead {
    model_sha: String,
    tokenizer_sha: String,
    dtype: String,
    pooling: String,
    batch_size: usize,
}

/// Parse checkpoint records from `bytes`: the rows by key, the length of the
/// valid prefix, and the dim. The first record that is cut short, has a key
/// that is not 16 lowercase hex characters, a dim other than the first
/// record's, or a vector that is not unit length ends the prefix.
fn parse_checkpoint(bytes: &[u8]) -> (HashMap<String, Vec<u8>>, usize, Option<usize>) {
    let mut rows = HashMap::new();
    let mut at = 0usize;
    let mut dim: Option<usize> = None;
    while bytes.len() >= at + KEY_LEN + 4 {
        let key = &bytes[at..at + KEY_LEN];
        if !key.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)) {
            break;
        }
        let d = u32::from_le_bytes([
            bytes[at + KEY_LEN],
            bytes[at + KEY_LEN + 1],
            bytes[at + KEY_LEN + 2],
            bytes[at + KEY_LEN + 3],
        ]) as usize;
        if d == 0 || dim.is_some_and(|x| x != d) {
            break;
        }
        let start = at + KEY_LEN + 4;
        let Some(row) = bytes.get(start..start + d * 4) else { break };
        let norm = row
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt();
        if !norm.is_finite() || (norm - 1.0).abs() > 1e-3 {
            break;
        }
        dim = Some(d);
        rows.insert(String::from_utf8_lossy(key).into_owned(), row.to_vec());
        at = start + d * 4;
    }
    (rows, at, dim)
}

/// The append side of the checkpoint.
struct Checkpoint {
    dir: PathBuf,
    file: BufWriter<std::fs::File>,
    every: usize,
    since_sync: usize,
}

impl Checkpoint {
    /// Open `dir`'s checkpoint for this model: the rows a previous run left
    /// (a checkpoint of another model or tokenizer is discarded), with the
    /// file cut back to its valid prefix and opened for appending.
    fn open(dir: &Path, head: &CkptHead, every: usize) -> Result<(Self, HashMap<String, Vec<u8>>)> {
        let head_path = dir.join(CKPT_HEAD);
        let ckpt_path = dir.join(CKPT_FILE);
        let same = crate::util::read_json_opt::<CkptHead>(&head_path).is_some_and(|h| h == *head);
        let mut rows = HashMap::new();
        if same {
            let mut bytes = Vec::new();
            match std::fs::File::open(&ckpt_path) {
                Ok(mut f) => {
                    f.read_to_end(&mut bytes)?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("reading vectors.ckpt"),
            }
            let (r, valid, _) = parse_checkpoint(&bytes);
            if valid < bytes.len() {
                eprintln!(
                    "checkpoint: kept {} rows, cut {} trailing bytes",
                    r.len(),
                    bytes.len() - valid
                );
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(&ckpt_path)?
                    .set_len(valid as u64)?;
            }
            rows = r;
        } else {
            if head_path.exists() {
                eprintln!("checkpoint: discarding one built by another model or tokenizer");
            }
            match std::fs::remove_file(&ckpt_path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).context("removing vectors.ckpt"),
            }
            let tmp = dir.join("vectors.ckpt.json.tmp");
            std::fs::write(&tmp, serde_json::to_vec_pretty(head)?)?;
            std::fs::rename(&tmp, &head_path)?;
        }
        let f = std::fs::OpenOptions::new().create(true).append(true).open(&ckpt_path)?;
        Ok((
            Self {
                dir: dir.to_path_buf(),
                file: BufWriter::new(f),
                every: every.max(1),
                since_sync: 0,
            },
            rows,
        ))
    }

    /// Append one row; it reaches the OS at once (a killed process loses
    /// nothing) and the disk every `every` rows.
    fn append(&mut self, key: &str, v: &[f32]) -> Result<()> {
        debug_assert_eq!(key.len(), KEY_LEN);
        let dim = u32::try_from(v.len()).context("dim over u32")?;
        self.file.write_all(key.as_bytes())?;
        self.file.write_all(&dim.to_le_bytes())?;
        for x in v {
            self.file.write_all(&x.to_le_bytes())?;
        }
        self.file.flush()?;
        self.since_sync += 1;
        if self.since_sync >= self.every {
            self.sync()?;
        }
        Ok(())
    }

    fn sync(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.get_ref().sync_data()?;
        self.since_sync = 0;
        Ok(())
    }

    /// The run is complete: the checkpoint is no longer needed.
    fn finish(self) -> Result<()> {
        drop(self.file);
        for name in [CKPT_FILE, CKPT_HEAD] {
            match std::fs::remove_file(self.dir.join(name)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("removing {name}")),
            }
        }
        Ok(())
    }
}

/// What a run did besides its vectors.
#[derive(Debug, Default, Clone)]
pub struct RunStats {
    /// Rows written so far and rows in the corpus.
    pub done: usize,
    pub total: usize,
    /// Seconds inside embedding calls, asleep for the duty cycle, and paused
    /// by the guard; the number of pauses.
    pub busy_secs: f64,
    pub duty_secs: f64,
    pub paused_secs: f64,
    pub pauses: usize,
    /// Why the run stopped early (`max_minutes`), when it did.
    pub stopped: Option<String>,
}

/// A run's result: `meta` is None when it stopped early (`stats.stopped`); a
/// rerun resumes from the checkpoint.
pub struct RunResult {
    pub meta: Option<VectorsMeta>,
    pub sources: Vec<reuse::SourceReport>,
    pub stats: RunStats,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

fn remove_if_present(path: &Path, what: &str) -> Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("removing {what}")),
    }
}

/// Paces a run: the guard's pauses, the duty cycle's sleeps and the
/// `max_minutes` stop, with the seconds each took.
struct Pacer<'a> {
    guard: Option<&'a Guard>,
    duty: f64,
    deadline: Option<Instant>,
    next_probe: Instant,
    stats: RunStats,
}

impl Pacer<'_> {
    /// Before an embedding call: Err(reason) when the run must stop now, after
    /// syncing the checkpoint; otherwise waits out any pause and returns.
    fn before(&mut self, ckpt: &mut Option<Checkpoint>, done: usize, total: usize) -> Result<std::result::Result<(), String>> {
        let over = |d: Option<Instant>| d.is_some_and(|d| Instant::now() >= d);
        if over(self.deadline) {
            return Ok(Err("max-minutes reached".to_owned()));
        }
        let Some(g) = self.guard else { return Ok(Ok(())) };
        if Instant::now() < self.next_probe {
            return Ok(Ok(()));
        }
        let mut last = String::new();
        let mut since_log = Instant::now();
        let mut paused_at: Option<Instant> = None;
        loop {
            let why = g.reasons();
            if why.is_empty() {
                break;
            }
            if let Some(c) = ckpt.as_mut() {
                c.sync()?;
            }
            let now = why.join("; ");
            if paused_at.is_none() {
                paused_at = Some(Instant::now());
                self.stats.pauses += 1;
            }
            if now != last || since_log.elapsed() >= Duration::from_secs(300) {
                eprintln!("guard: paused at {done}/{total}: {now}");
                last = now;
                since_log = Instant::now();
            }
            if over(self.deadline) {
                self.stats.paused_secs += paused_at.map_or(0.0, |t| t.elapsed().as_secs_f64());
                return Ok(Err("max-minutes reached while paused".to_owned()));
            }
            std::thread::sleep(Duration::from_secs_f64(g.pause_secs.max(0.1)));
        }
        if let Some(t) = paused_at {
            let s = t.elapsed().as_secs_f64();
            self.stats.paused_secs += s;
            eprintln!("guard: resumed after {s:.0}s");
        }
        self.next_probe = Instant::now() + Duration::from_secs_f64(g.every_secs.max(0.0));
        Ok(Ok(()))
    }

    /// After an embedding call that took `work`.
    fn after(&mut self, work: Duration) {
        self.stats.busy_secs += work.as_secs_f64();
        if self.duty < 1.0 {
            let s = work.mul_f64((1.0 - self.duty) / self.duty);
            self.stats.duty_secs += s.as_secs_f64();
            std::thread::sleep(s);
        }
    }
}

/// Embed `cfg.corpus`, reusing unchanged rows and resuming from a checkpoint,
/// paced by the guard, the duty cycle and `max_minutes`.
///
/// # Errors
/// Missing files, a tokenizer that does not match the corpus, a chunk longer
/// than the model's context, a failed inference, a vector that is not unit
/// length, or a duty outside (0, 1].
#[allow(clippy::too_many_lines)]
pub fn run(cfg: &EmbedConfig, progress: &dyn Fn(usize, usize)) -> Result<RunResult> {
    let started = Instant::now();
    if !(cfg.duty > 0.0 && cfg.duty <= 1.0) {
        bail!("duty {} is outside (0, 1]", cfg.duty);
    }
    let manifest: P0Manifest = serde_json::from_slice(&read(&cfg.corpus, "manifest.p0.json")?)
        .context("manifest.p0.json")?;

    let tokenizer_sha = sha_hex(&read(&cfg.model_dir, "tokenizer.json")?);
    if tokenizer_sha != manifest.tokenizer_sha {
        bail!(
            "tokenizer mismatch: chunks were counted with {} but {} is {}",
            &manifest.tokenizer_sha[..16.min(manifest.tokenizer_sha.len())],
            cfg.model_dir.join("tokenizer.json").display(),
            &tokenizer_sha[..16]
        );
    }

    let chunks_bytes = read(&cfg.corpus, "chunks.jsonl")?;
    let chunks_sha = sha_hex(&chunks_bytes);
    let lines: Vec<ChunkLine> = chunks_bytes
        .lines()
        .enumerate()
        .map(|(i, l)| -> Result<ChunkLine> {
            let l = l?;
            serde_json::from_str(&l).with_context(|| format!("chunks.jsonl line {}", i + 1))
        })
        .collect::<Result<_>>()?;
    if let Some(c) = lines.iter().find(|c| c.token_count > MODEL_MAX_TOKENS) {
        bail!(
            "{} is {} tokens, over the model's {MODEL_MAX_TOKENS}; it would be truncated",
            c.chunk_id,
            c.token_count
        );
    }

    let loaded = load_model(&cfg.model_dir, &cfg.onnx_file, cfg.intra_threads)?;
    let model_sha = loaded.model_sha;
    let mut embedder = loaded.embedding;

    let texts: Vec<std::borrow::Cow<'_, str>> = lines
        .iter()
        .map(|c| indexed_text(c.prefix.as_deref().filter(|_| !cfg.ignore_prefix), &c.text))
        .collect();
    let keys: Vec<String> = texts.iter().map(|t| crate::corpus::chunk::sha16(t)).collect();
    // Read every prior BEFORE the old meta is removed below: the target
    // directory's own last run is the main source.
    let mut pool = ReusePool::default();
    if cfg.reuse {
        pool.add_dir(&cfg.corpus, &model_sha, &tokenizer_sha);
        for d in &cfg.reuse_from {
            pool.add_dir(d, &model_sha, &tokenizer_sha);
        }
    }
    // An interrupted run's rows, read whether or not `reuse` is on: they were
    // embedded by this model for this run.
    let (mut ckpt, resumable) = if cfg.checkpoint_every > 0 {
        let head = CkptHead {
            model_sha: model_sha.clone(),
            tokenizer_sha: tokenizer_sha.clone(),
            dtype: reuse::DTYPE.to_owned(),
            pooling: reuse::POOLING.to_owned(),
            batch_size: reuse::BATCH_SIZE,
        };
        let (c, rows) = Checkpoint::open(&cfg.corpus, &head, cfg.checkpoint_every)?;
        if !rows.is_empty() {
            eprintln!("checkpoint: {} rows from an interrupted run", rows.len());
        }
        (Some(c), rows)
    } else {
        (None, HashMap::new())
    };

    // A vectors.bin without a matching meta file must never look complete, so
    // the old meta goes before the new vectors.bin and the new meta lands
    // last, by rename. Without a checkpoint it goes first, as it always did;
    // with one it stays until the rename, so the old vectors serve meanwhile
    // and a resumed run still reuses them.
    let meta_path = cfg.corpus.join("vectors.meta.json");
    if ckpt.is_none() {
        remove_if_present(&meta_path, "the old vectors.meta.json")?;
    }
    let tmp = cfg.corpus.join("vectors.bin.tmp");
    let mut out = BufWriter::new(std::fs::File::create(&tmp)?);
    let mut dim = pool.dim().unwrap_or(0);
    let mut max_norm_error = 0f32;
    let (mut reused, mut embedded, mut resumed) = (0usize, 0usize, 0usize);
    let total = lines.len();
    let mut pacer = Pacer {
        guard: cfg.guard.as_ref(),
        duty: cfg.duty,
        deadline: cfg.max_minutes.map(|m| started + Duration::from_secs_f64(m.max(0.0) * 60.0)),
        next_probe: Instant::now(),
        stats: RunStats { total, ..RunStats::default() },
    };
    for (i, c) in lines.iter().enumerate() {
        let stored = pool.get(&keys[i]).map(|b| (b, true)).or_else(|| resumable.get(&keys[i]).map(|b| (b.as_slice(), false)));
        let v = if let Some((bytes, from_pool)) = stored {
            if from_pool {
                reused += 1;
            } else {
                embedded += 1;
                resumed += 1;
            }
            // Decoded only for the dim and norm checks below; the bytes
            // written are the stored ones.
            bytes
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect::<Vec<f32>>()
        } else {
            if let Err(why) = pacer.before(&mut ckpt, i, total)? {
                drop(out);
                remove_if_present(&tmp, "vectors.bin.tmp")?;
                if let Some(mut k) = ckpt {
                    k.sync()?;
                }
                eprintln!("stopped at {i}/{total}: {why}; rerun the same command to resume");
                pacer.stats.done = i;
                pacer.stats.stopped = Some(why);
                return Ok(RunResult { meta: None, sources: pool.sources, stats: pacer.stats });
            }
            embedded += 1;
            let t = Instant::now();
            let v = embedder
                .embed([texts[i].as_ref()], None)
                .map_err(|e| anyhow!("embedding {}: {e}", c.chunk_id))?
                .pop()
                .ok_or_else(|| anyhow!("no vector for {}", c.chunk_id))?;
            pacer.after(t.elapsed());
            v
        };
        if dim == 0 {
            dim = v.len();
        } else if v.len() != dim {
            bail!("{} has dim {}, expected {dim}", c.chunk_id, v.len());
        }
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        max_norm_error = max_norm_error.max((norm - 1.0).abs());
        if (norm - 1.0).abs() > 1e-3 {
            bail!("{} has norm {norm}; expected unit length", c.chunk_id);
        }
        // Freshly embedded rows only: reused and resumed rows are on disk.
        if stored.is_none()
            && let Some(k) = ckpt.as_mut()
        {
            k.append(&keys[i], &v)?;
        }
        for x in &v {
            out.write_all(&x.to_le_bytes())?;
        }
        if (i + 1) % 500 == 0 || i + 1 == total {
            progress(i + 1, total);
        }
    }
    out.flush()?;
    drop(out);
    if ckpt.is_some() {
        remove_if_present(&meta_path, "the old vectors.meta.json")?;
    }
    std::fs::rename(&tmp, cfg.corpus.join("vectors.bin"))?;

    let meta = VectorsMeta {
        model_file: cfg.onnx_file.clone(),
        model_sha,
        tokenizer_sha,
        chunks_sha,
        dim,
        count: total,
        dtype: reuse::DTYPE,
        layout: reuse::LAYOUT,
        pooling: reuse::POOLING,
        normalized: true,
        batch_size: reuse::BATCH_SIZE,
        max_norm_error,
        built_at_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        seconds: started.elapsed().as_secs_f64(),
        prefix_ignored: cfg.ignore_prefix && lines.iter().any(|c| c.prefix.is_some()),
        chunk_ids: lines.iter().map(|c| c.chunk_id.clone()).collect(),
        content_shas: keys,
        reused,
        embedded,
        resumed,
    };
    let meta_tmp = cfg.corpus.join("vectors.meta.json.tmp");
    std::fs::write(&meta_tmp, serde_json::to_vec_pretty(&meta)?)?;
    std::fs::rename(&meta_tmp, &meta_path)?;
    if let Some(k) = ckpt {
        k.finish()?;
    }
    pacer.stats.done = total;
    Ok(RunResult { meta: Some(meta), sources: pool.sources, stats: pacer.stats })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(key: &str, v: &[f32]) -> Vec<u8> {
        let mut b = key.as_bytes().to_vec();
        b.extend_from_slice(&u32::try_from(v.len()).unwrap().to_le_bytes());
        for x in v {
            b.extend_from_slice(&x.to_le_bytes());
        }
        b
    }

    #[test]
    fn checkpoint_keeps_the_valid_prefix() {
        let a = record("0123456789abcdef", &[0.6, 0.8]);
        let b = record("fedcba9876543210", &[1.0, 0.0]);
        let mut bytes = [a.clone(), b.clone()].concat();
        let (rows, valid, dim) = parse_checkpoint(&bytes);
        assert_eq!((rows.len(), valid, dim), (2, bytes.len(), Some(2)));
        assert_eq!(rows["fedcba9876543210"], b[20..]);
        // A torn last record is cut, not read.
        bytes.extend_from_slice(&record("00000000000000aa", &[0.0, 1.0])[..23]);
        let (rows, valid, _) = parse_checkpoint(&bytes);
        assert_eq!((rows.len(), valid), (2, a.len() + b.len()));
        // Zeros (a crash after the size grew, before the data) end it too.
        let z = [a.clone(), vec![0u8; 28]].concat();
        assert_eq!(parse_checkpoint(&z).1, a.len());
        // So do a non-unit row, a non-hex key and a changed dim.
        assert_eq!(parse_checkpoint(&[a.clone(), record("1111111111111111", &[0.5, 0.5])].concat()).1, a.len());
        assert_eq!(parse_checkpoint(&[a.clone(), record("XYZ1111111111111", &[1.0, 0.0])].concat()).1, a.len());
        assert_eq!(parse_checkpoint(&[a.clone(), record("1111111111111111", &[1.0, 0.0, 0.0])].concat()).1, a.len());
    }

    #[test]
    fn guard_parsers() {
        let none = "Note: No thermal warning level has been recorded\nNote: No performance warning level has been recorded\nNote: No CPU power status has been recorded\n";
        assert_eq!(therm_warning(none), None);
        assert!(therm_warning("Thermal warning level: 2\n").is_some());
        assert!(therm_warning("CPU_Speed_Limit \t= 80\n").is_some());
        assert_eq!(therm_warning("CPU_Speed_Limit \t= 100\n"), None);
        let mi = "MemTotal:       11534336 kB\nMemFree:  100 kB\nMemAvailable:    1572864 kB\n";
        assert!((mem_available_gib(mi).unwrap() - 1.5).abs() < 1e-9);
        let psi = "some avg10=41.50 avg60=3.00 avg300=1.00 total=1\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        assert!((cpu_some_avg10(psi).unwrap() - 41.5).abs() < 1e-9);
        let stat = "cpu  1 2 3\ncpu0 1 2 3\ncpu1 1 2 3\ncpu2 1 2 3\nintr 5\nctxt 6\n";
        assert_eq!(online_cpus(stat), 3);
    }

    #[test]
    fn query_onnx_auto_is_the_vectors_file() {
        assert_eq!(resolve_query_onnx("auto", "model.onnx"), "model.onnx");
        assert_eq!(resolve_query_onnx("model_int8.onnx", "model.onnx"), "model_int8.onnx");
    }
}
