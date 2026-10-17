//! Everything a query needs, loaded once: the chunk store, both indexes, the
//! query embedder and the reranker. Shared by the `search` and `eval` bins so
//! the eval measures exactly the code path users run.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::Serialize;

use super::bm25::Bm25;
use super::dense::DenseIndex;
use super::horizon::Horizon;
use super::pipeline::{Hit, Indexes, Mode, Models, RetrievalConfig, retrieve};
use super::rerank::Reranker;
use super::store::ChunkStore;
use crate::corpus::embed::{QueryEmbedder, check_query_model, resolve_query_onnx};

#[derive(Debug, Clone, clap::Args)]
pub struct RuntimeArgs {
    /// Directory holding chunks.jsonl, vectors.bin and bm25/. Default: the P0 story corpus plus the
    /// operator archives (`build-archives`); `--corpus artifacts` is the stories alone.
    #[arg(long, default_value = "artifacts/p3a")]
    pub corpus: PathBuf,
    /// gte-modernbert-base directory.
    #[arg(long, default_value = "models/gte-modernbert-base")]
    pub model_dir: PathBuf,
    /// ONNX file for QUERIES. `auto` (the default, design/trevor-reembed.md section 7) is the file
    /// the corpus vectors were built with (`vectors.meta.json` `modelFile`): `model_int8.onnx` for the INT8 corpora,
    /// the old default, so nothing changes until an fp32 re-embed is swapped in. A query model whose sha differs from
    /// the vectors' is refused (INT8 drifts to cosine 0.94 on x86) unless `--allow-query-model-mismatch`.
    #[arg(long, default_value = "auto")]
    pub query_onnx: String,
    /// Allow a query model other than the one the vectors were built with (logged);
    /// `TREVOR_ALLOW_QUERY_MODEL_MISMATCH=1` does the same.
    #[arg(long)]
    pub allow_query_model_mismatch: bool,
    /// ettin-reranker-17m-v1 directory. Only loaded when reranking is on.
    #[arg(long, default_value = "models/ettin-reranker-17m-v1")]
    pub rerank_dir: PathBuf,
    #[arg(long, default_value = "onnx/model.onnx")]
    pub rerank_onnx: String,
    /// Tokens per (query, passage) pair; the passage is cut, never the query.
    #[arg(long, default_value_t = super::rerank::RERANK_MAX_TOKENS)]
    pub rerank_max_tokens: usize,
    #[arg(long)]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct ConfigArgs {
    #[arg(long, value_enum, default_value_t = Mode::Hybrid)]
    pub mode: Mode,
    #[arg(long, default_value_t = 100)]
    pub candidates: usize,
    #[arg(long, default_value_t = 20.0)]
    pub rrf_k: f32,
    /// Rerank this many of the fused top. 0 = reranker off.
    #[arg(long, default_value_t = 0)]
    pub rerank_top: usize,
    #[arg(long, default_value_t = 20)]
    pub k: usize,
}

impl From<&ConfigArgs> for RetrievalConfig {
    fn from(a: &ConfigArgs) -> Self {
        Self {
            mode: a.mode,
            candidates: a.candidates,
            rrf_k: a.rrf_k,
            rerank_top: a.rerank_top,
            k: a.k,
        }
    }
}

/// Identifies every input that can change a ranking, for the run manifest.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fingerprint {
    pub chunks_sha: String,
    pub chunk_count: usize,
    pub doc_model_sha: Option<String>,
    pub tokenizer_sha: Option<String>,
    pub query_model_file: Option<String>,
    pub query_model_sha: Option<String>,
    pub reranker_sha: Option<String>,
    pub rerank_max_tokens: Option<usize>,
    pub bm25_pattern: &'static str,
}

pub struct Runtime {
    pub store: ChunkStore,
    pub dense: Option<DenseIndex>,
    pub bm25: Option<Bm25>,
    embedder: Option<SharedEmbedder>,
    pub reranker: Option<Reranker>,
    /// The spoiler horizon (`ask --serve` jobs): `retrieve` returns only chunks released at or before it. None, the
    /// default, retrieves exactly as before.
    pub horizon: Option<Arc<Horizon>>,
    /// BM25-only serving (`ask --retrieval bm25`): loaded without the dense index and the embedder, and a
    /// dense or hybrid retrieval runs as BM25. False, the default, leaves a missing dense index an error as before.
    pub bm25_only: bool,
}

/// A query embedder that several runtimes share, so a serve job that loads another corpus (P4x, the art corpus) does
/// not spend seconds building a second one. It is reused only for the same model directory, ONNX file and thread count,
/// and only when the new corpus's vectors were made with the tokenizer it was checked against.
/// `TREVOR_SHARE_EMBEDDER=0` loads one per runtime.
#[derive(Clone)]
pub struct SharedEmbedder {
    inner: Arc<Mutex<QueryEmbedder>>,
    model_dir: PathBuf,
    onnx: String,
    threads: Option<usize>,
    tokenizer_sha: String,
}

impl SharedEmbedder {
    fn fits(&self, a: &RuntimeArgs, onnx: &str, tokenizer_sha: &str) -> bool {
        self.model_dir == a.model_dir && self.onnx == onnx && self.threads == a.threads && self.tokenizer_sha == tokenizer_sha
    }

    fn embed(&self, q: &str) -> Result<Vec<f32>> {
        self.inner.lock().map_err(|_| anyhow::anyhow!("query embedder lock poisoned"))?.embed(q)
    }
}

/// The fused candidate depth a horizon-filtered retrieval reads before dropping hidden chunks.
pub const HORIZON_DEPTH: usize = 2000;

impl Runtime {
    /// Load what `needs` requires and nothing else, so a BM25-only run does
    /// not pay for two ONNX sessions.
    ///
    /// # Errors
    /// Any missing or stale artifact.
    pub fn load(
        a: &RuntimeArgs,
        needs_dense: bool,
        needs_bm25: bool,
        needs_rerank: bool,
    ) -> Result<Self> {
        Self::load_sharing(a, needs_dense, needs_bm25, needs_rerank, None)
    }

    /// As [`Runtime::load`], reusing `share` (another runtime's query embedder) when it fits this corpus.
    ///
    /// # Errors
    /// Any missing or stale artifact.
    pub fn load_sharing(
        a: &RuntimeArgs,
        needs_dense: bool,
        needs_bm25: bool,
        needs_rerank: bool,
        share: Option<SharedEmbedder>,
    ) -> Result<Self> {
        let store = ChunkStore::load(&a.corpus.join("chunks.jsonl"))?;
        let (dense, embedder) = if needs_dense {
            let d = DenseIndex::load(&a.corpus, &store)?;
            let onnx = resolve_query_onnx(&a.query_onnx, &d.meta.model_file).to_owned();
            let share = share.filter(|s| crate::env::on_unless_0("TREVOR_SHARE_EMBEDDER")
                && s.fits(a, &onnx, &d.meta.tokenizer_sha));
            let e = match share {
                Some(s) => s,
                None => SharedEmbedder {
                    inner: Arc::new(Mutex::new(QueryEmbedder::load(
                        &a.model_dir,
                        &onnx,
                        a.threads,
                        &d.meta.tokenizer_sha,
                    )?)),
                    model_dir: a.model_dir.clone(),
                    onnx,
                    threads: a.threads,
                    tokenizer_sha: d.meta.tokenizer_sha.clone(),
                },
            };
            {
                let q = e.inner.lock().map_err(|_| anyhow::anyhow!("query embedder lock poisoned"))?;
                check_query_model(&d.meta.model_file, &d.meta.model_sha, &q, a.allow_query_model_mismatch)?;
            }
            (Some(d), Some(e))
        } else {
            (None, None)
        };
        let bm25 = if needs_bm25 {
            Some(Bm25::open(&a.corpus.join("bm25"), &store)?)
        } else {
            None
        };
        let reranker = if needs_rerank {
            Some(Reranker::load(
                &a.rerank_dir,
                &a.rerank_onnx,
                a.rerank_max_tokens,
                a.threads,
            )?)
        } else {
            None
        };
        Ok(Self {
            store,
            dense,
            bm25,
            embedder,
            reranker,
            horizon: None,
            bm25_only: false,
        })
    }

    /// This runtime's query embedder, for another runtime to share ([`Runtime::load_sharing`]).
    #[must_use]
    pub fn shared_embedder(&self) -> Option<SharedEmbedder> {
        self.embedder.clone()
    }

    /// # Errors
    /// See [`retrieve`].
    pub fn retrieve(&mut self, query: &str, cfg: &RetrievalConfig) -> Result<Vec<Hit>> {
        let Some(h) = self.horizon.clone() else { return self.retrieve_all(query, cfg) };
        // Under a horizon: a fused list of at least HORIZON_DEPTH candidates per retriever, the hidden chunks dropped,
        // then the first `k`. A depth of 8 x k alone can leave a question whose top candidates are nearly all later
        // stories with one or two passages; dense and BM25 score every chunk anyway, so the depth is cheap.
        // A reranked config reranks before the drop (opt-in paths only; `ask --rerank-add`).
        let depth = cfg.candidates.max(cfg.k * 8).max(HORIZON_DEPTH);
        let wide = RetrievalConfig { candidates: depth, k: depth, ..cfg.clone() };
        let mut hits = self.retrieve_all(query, &wide)?;
        hits.retain(|x| self.allows_row(&h, x.row));
        hits.truncate(cfg.k);
        Ok(hits)
    }

    /// Whether row `row` is visible under horizon `h`.
    #[must_use]
    pub fn allows_row(&self, h: &Horizon, row: usize) -> bool {
        self.store.chunks.get(row).is_some_and(|c| h.allows(&c.story_id, &c.group_id))
    }

    fn retrieve_all(&mut self, query: &str, cfg: &RetrievalConfig) -> Result<Vec<Hit>> {
        let bm25_cfg;
        let cfg = if self.bm25_only && self.dense.is_none() && cfg.mode != Mode::Bm25 {
            bm25_cfg = RetrievalConfig { mode: Mode::Bm25, ..cfg.clone() };
            &bm25_cfg
        } else { cfg };
        let ix = Indexes {
            store: &self.store,
            dense: self.dense.as_ref(),
            bm25: self.bm25.as_ref(),
        };
        let embedder = &self.embedder;
        let mut embed = |q: &str| -> Result<Vec<f32>> {
            match embedder.as_ref() {
                Some(e) => e.embed(q),
                None => anyhow::bail!("no query embedder loaded"),
            }
        };
        let rr = self.reranker.as_ref();
        let rerank_fn = |q: &str, docs: &[&str]| -> Result<Vec<f32>> {
            match rr {
                Some(r) => r.score(q, docs),
                None => anyhow::bail!("no reranker loaded"),
            }
        };
        let mut models = Models {
            embed_query: &mut embed,
            rerank: rr.map(|_| &rerank_fn as &super::pipeline::RerankFn<'_>),
        };
        retrieve(query, cfg, &ix, &mut models)
    }

    /// The query vector the dense stage would use.
    ///
    /// # Errors
    /// No embedder loaded, or an inference failure.
    pub fn embed_query(&mut self, q: &str) -> Result<Vec<f32>> {
        match self.embedder.as_ref() {
            Some(e) => e.embed(q),
            None => anyhow::bail!("no query embedder loaded"),
        }
    }

    #[must_use]
    pub fn fingerprint(&self) -> Fingerprint {
        Fingerprint {
            chunks_sha: self.store.chunks_sha.clone(),
            chunk_count: self.store.len(),
            doc_model_sha: self.dense.as_ref().map(|d| d.meta.model_sha.clone()),
            tokenizer_sha: self.dense.as_ref().map(|d| d.meta.tokenizer_sha.clone()),
            query_model_file: self.embedder.as_ref().and_then(|e| e.inner.lock().ok().map(|e| e.onnx_file.clone())),
            query_model_sha: self.embedder.as_ref().and_then(|e| e.inner.lock().ok().map(|e| e.model_sha.clone())),
            reranker_sha: self.reranker.as_ref().map(|r| r.model_sha.clone()),
            rerank_max_tokens: self.reranker.as_ref().map(|r| r.max_tokens),
            bm25_pattern: super::tokenizer::LORE_PATTERN,
        }
    }
}
