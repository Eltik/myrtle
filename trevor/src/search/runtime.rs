//! Everything a query needs, loaded once: the chunk store, both indexes, the
//! query embedder and the reranker. Shared by the `search` and `eval` bins so
//! the eval measures exactly the code path users run.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use serde::Serialize;

use super::bm25::Bm25;
use super::dense::DenseIndex;
use super::horizon::Horizon;
use super::pipeline::{Hit, Indexes, Mode, Models, RetrievalConfig, retrieve};
use super::rerank::Reranker;
use super::store::ChunkStore;
use crate::corpus::embed::QueryEmbedder;

#[derive(Debug, Clone, clap::Args)]
pub struct RuntimeArgs {
    /// Directory holding chunks.jsonl, vectors.bin and bm25/. Default: the P0 story corpus plus the
    /// operator archives (`build-archives`, adopted 2026-09-27); `--corpus artifacts` is the stories alone.
    #[arg(long, default_value = "artifacts/p3a")]
    pub corpus: PathBuf,
    /// gte-modernbert-base directory.
    #[arg(long, default_value = "models/gte-modernbert-base")]
    pub model_dir: PathBuf,
    /// ONNX file for QUERIES. INT8 is right on Apple Silicon; on x86 use the
    /// fp32 `model.onnx`, since INT8 drifts to cosine 0.94 there.
    #[arg(long, default_value = "model_int8.onnx")]
    pub query_onnx: String,
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
    embedder: Option<QueryEmbedder>,
    pub reranker: Option<Reranker>,
    /// The spoiler horizon (`ask --serve` jobs): `retrieve` returns only chunks released at or before it. None, the
    /// default, retrieves exactly as before.
    pub horizon: Option<Arc<Horizon>>,
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
        let store = ChunkStore::load(&a.corpus.join("chunks.jsonl"))?;
        let (dense, embedder) = if needs_dense {
            let d = DenseIndex::load(&a.corpus, &store)?;
            let e = QueryEmbedder::load(
                &a.model_dir,
                &a.query_onnx,
                a.threads,
                &d.meta.tokenizer_sha,
            )?;
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
        })
    }

    /// # Errors
    /// See [`retrieve`].
    pub fn retrieve(&mut self, query: &str, cfg: &RetrievalConfig) -> Result<Vec<Hit>> {
        let Some(h) = self.horizon.clone() else { return self.retrieve_all(query, cfg) };
        // Under a horizon: a fused list of at least HORIZON_DEPTH candidates per retriever, the hidden chunks dropped,
        // then the first `k`. The 2026-10-07 first cut took 8 x k (64) and kept 1, 5 and 2 passages on three questions
        // whose top 64 were nearly all later stories; dense and BM25 score every chunk anyway, so the depth is cheap.
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
        let ix = Indexes {
            store: &self.store,
            dense: self.dense.as_ref(),
            bm25: self.bm25.as_ref(),
        };
        let embedder = &mut self.embedder;
        let mut embed = |q: &str| -> Result<Vec<f32>> {
            match embedder.as_mut() {
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
        match self.embedder.as_mut() {
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
            query_model_file: self.embedder.as_ref().map(|e| e.onnx_file.clone()),
            query_model_sha: self.embedder.as_ref().map(|e| e.model_sha.clone()),
            reranker_sha: self.reranker.as_ref().map(|r| r.model_sha.clone()),
            rerank_max_tokens: self.reranker.as_ref().map(|r| r.max_tokens),
            bm25_pattern: super::tokenizer::LORE_PATTERN,
        }
    }
}
