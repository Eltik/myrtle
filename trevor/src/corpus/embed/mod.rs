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
//! Long runs are paced and resumable (design/trevor-reembed.md):
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
use std::io::{BufRead, BufWriter, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::corpus::chunk::indexed_text;
use crate::corpus::reuse::{self, ReusePool};
use crate::util::sha_hex;

mod checkpoint;
mod model;
mod pacing;
#[cfg(test)]
mod tests;

use checkpoint::{Checkpoint, CkptHead};
pub use model::{
    ALLOW_QUERY_MODEL_MISMATCH_ENV, LoadedModel, QueryEmbedder, check_query_model, load_model, resolve_query_onnx,
};
pub use pacing::Guard;
use pacing::Pacer;

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
    let manifest: P0Manifest =
        serde_json::from_slice(&read(&cfg.corpus, "manifest.p0.json")?).context("manifest.p0.json")?;

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
        deadline: cfg
            .max_minutes
            .map(|m| started + Duration::from_secs_f64(m.max(0.0) * 60.0)),
        next_probe: Instant::now(),
        stats: RunStats {
            total,
            ..RunStats::default()
        },
    };
    for (i, c) in lines.iter().enumerate() {
        let stored = pool
            .get(&keys[i])
            .map(|b| (b, true))
            .or_else(|| resumable.get(&keys[i]).map(|b| (b.as_slice(), false)));
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
                return Ok(RunResult {
                    meta: None,
                    sources: pool.sources,
                    stats: pacer.stats,
                });
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
        built_at_unix: crate::util::unix_now(),
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
    Ok(RunResult {
        meta: Some(meta),
        sources: pool.sources,
        stats: pacer.stats,
    })
}
