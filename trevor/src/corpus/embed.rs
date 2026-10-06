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

use std::io::{BufRead, BufWriter, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use fastembed::{
    InitOptionsUserDefined, Pooling, QuantizationMode, TextEmbedding, TokenizerFiles,
    UserDefinedEmbeddingModel,
};
use serde::{Deserialize, Serialize};

use crate::corpus::chunk::indexed_text;
use crate::corpus::reuse::{self, ReusePool};
use sha2::{Digest, Sha256};

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
    /// True when chunks carried prefixes and they were not embedded.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub prefix_ignored: bool,
}

fn sha_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
/// The ONNX file may differ from the one the corpus used, deliberately: on
/// x86 the INT8 export drifts to cosine 0.94 from fp32 while the Mac's INT8
/// vectors sit at 0.997, so an x86 host embeds queries with the fp32 export.
/// The tokenizer may not differ; that is refused.
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

/// # Errors
/// Missing files, a tokenizer that does not match the corpus, a chunk longer
/// than the model's context, a failed inference, or a vector that is not unit
/// length.
pub fn run(
    cfg: &EmbedConfig,
    progress: &dyn Fn(usize, usize),
) -> Result<(VectorsMeta, Vec<reuse::SourceReport>)> {
    let started = std::time::Instant::now();
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

    // A vectors.bin without a matching meta file must never look complete, so
    // the old meta goes first and the new one lands last, by rename.
    let meta_path = cfg.corpus.join("vectors.meta.json");
    match std::fs::remove_file(&meta_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e).context("removing the old vectors.meta.json"),
    }
    let tmp = cfg.corpus.join("vectors.bin.tmp");
    let mut out = BufWriter::new(std::fs::File::create(&tmp)?);
    let mut dim = pool.dim().unwrap_or(0);
    let mut max_norm_error = 0f32;
    let (mut reused, mut embedded) = (0usize, 0usize);
    let total = lines.len();
    for (i, c) in lines.iter().enumerate() {
        let v = if let Some(bytes) = pool.get(&keys[i]) {
            reused += 1;
            // Decoded only for the dim and norm checks below; the bytes
            // written are the stored ones.
            bytes
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect::<Vec<f32>>()
        } else {
            embedded += 1;
            embedder
                .embed([texts[i].as_ref()], None)
                .map_err(|e| anyhow!("embedding {}: {e}", c.chunk_id))?
                .pop()
                .ok_or_else(|| anyhow!("no vector for {}", c.chunk_id))?
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
        for x in &v {
            out.write_all(&x.to_le_bytes())?;
        }
        if (i + 1) % 500 == 0 || i + 1 == total {
            progress(i + 1, total);
        }
    }
    out.flush()?;
    drop(out);
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
    };
    let meta_tmp = cfg.corpus.join("vectors.meta.json.tmp");
    std::fs::write(&meta_tmp, serde_json::to_vec_pretty(&meta)?)?;
    std::fs::rename(&meta_tmp, &meta_path)?;
    Ok((meta, pool.sources))
}
