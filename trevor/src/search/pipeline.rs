//! The retrieval pipeline: candidates from dense and/or BM25, fused by RRF,
//! optionally reranked.
//!
//! Every stage is a switch in [`RetrievalConfig`], and each switch's "off"
//! value reproduces the pipeline without that stage exactly: `Mode::Dense`
//! and `Mode::Bm25` return that retriever's own ranking untouched, and
//! `rerank_top = 0` returns the fused ranking untouched. That is what lets
//! the eval attribute a change to one stage.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use super::bm25::Bm25;
use super::dense::DenseIndex;
use super::store::ChunkStore;
use super::tokenizer::rrf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Dense,
    Bm25,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalConfig {
    pub mode: Mode,
    /// Candidates taken from each retriever before fusion.
    pub candidates: usize,
    /// RRF damping. Not evidence-backed at 20; tune on the gold set.
    pub rrf_k: f32,
    /// How many of the fused top to rerank. 0 turns the reranker off.
    pub rerank_top: usize,
    /// Results returned.
    pub k: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Hybrid,
            candidates: 100,
            rrf_k: 20.0,
            rerank_top: 0,
            k: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hit {
    pub row: usize,
    pub score: f32,
    /// 1-based rank in each stage, when the chunk appeared there.
    pub dense_rank: Option<usize>,
    pub bm25_rank: Option<usize>,
    pub fused_rank: Option<usize>,
}

/// Scores passages for a query, one logit each, in input order.
pub type RerankFn<'f> = dyn Fn(&str, &[&str]) -> Result<Vec<f32>> + 'f;

/// What the pipeline needs from outside: a query vector and reranker scores.
/// Kept as closures so the pipeline has no model types in it and tests can
/// drive it with fakes.
pub struct Models<'a> {
    pub embed_query: &'a mut dyn FnMut(&str) -> Result<Vec<f32>>,
    pub rerank: Option<&'a RerankFn<'a>>,
}

pub struct Indexes<'a> {
    pub store: &'a ChunkStore,
    pub dense: Option<&'a DenseIndex>,
    pub bm25: Option<&'a Bm25>,
}

/// # Errors
/// A stage the config asks for whose index or model is missing, or a stage
/// failure.
pub fn retrieve(
    query: &str,
    cfg: &RetrievalConfig,
    ix: &Indexes<'_>,
    m: &mut Models<'_>,
) -> Result<Vec<Hit>> {
    let dense = if matches!(cfg.mode, Mode::Dense | Mode::Hybrid) {
        let Some(d) = ix.dense else {
            bail!("mode {:?} needs the dense index", cfg.mode)
        };
        let q = (m.embed_query)(query)?;
        d.search(&q, cfg.candidates)?
    } else {
        Vec::new()
    };
    let lexical = if matches!(cfg.mode, Mode::Bm25 | Mode::Hybrid) {
        let Some(b) = ix.bm25 else {
            bail!("mode {:?} needs the BM25 index", cfg.mode)
        };
        b.search(query, cfg.candidates, ix.store)?
    } else {
        Vec::new()
    };
    let rank_in =
        |list: &[(usize, f32)], row: usize| list.iter().position(|h| h.0 == row).map(|p| p + 1);

    let first: Vec<(usize, f32)> = match cfg.mode {
        Mode::Dense => dense.clone(),
        Mode::Bm25 => lexical.clone(),
        Mode::Hybrid => {
            let lists: Vec<Vec<u64>> = [&dense, &lexical]
                .iter()
                .map(|l| l.iter().map(|h| h.0 as u64).collect())
                .collect();
            rrf(&lists, cfg.rrf_k)
                .into_iter()
                .map(|(row, s)| (usize::try_from(row).unwrap_or(usize::MAX), s))
                .collect()
        }
    };
    let mut hits: Vec<Hit> = first
        .iter()
        .enumerate()
        .map(|(i, &(row, score))| Hit {
            row,
            score,
            dense_rank: rank_in(&dense, row),
            bm25_rank: rank_in(&lexical, row),
            fused_rank: Some(i + 1),
        })
        .collect();

    if cfg.rerank_top > 0 {
        let Some(rerank) = m.rerank else {
            bail!("rerank_top is {} but no reranker is loaded", cfg.rerank_top)
        };
        let n = cfg.rerank_top.min(hits.len());
        let texts: Vec<&str> = hits[..n]
            .iter()
            .map(|h| ix.store.chunks[h.row].text.as_str())
            .collect();
        let scores = rerank(query, &texts)?;
        for (h, s) in hits[..n].iter_mut().zip(scores) {
            h.score = s;
        }
        // Reranked head, then the unreranked tail in fused order. Ties keep
        // the fused order, so the sort is stable by construction.
        hits[..n].sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then(a.fused_rank.cmp(&b.fused_rank))
        });
    }
    hits.truncate(cfg.k);
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::chunk::Chunk;

    fn store() -> ChunkStore {
        let c = |id: &str, text: &str| Chunk {
            chunk_id: id.to_owned(),
            story_id: "s".to_owned(),
            group_id: "g".to_owned(),
            ordinal: 0,
            scene_ordinal: 0,
            line_start: 1,
            line_end: 1,
            text: text.to_owned(),
            speakers: vec![],
            on_screen: vec![],
            background: None,
            token_count: 1,
            content_sha: String::new(),
            prefix: None,
        };
        ChunkStore::from_chunks(
            vec![c("s#0", "alpha"), c("s#1", "beta"), c("s#2", "gamma")],
            "x".into(),
        )
        .expect("store")
    }

    #[test]
    fn rerank_off_is_the_fused_order_and_rerank_on_reorders_only_the_head() {
        let st = store();
        let dir = std::env::temp_dir().join(format!("trevor-pipe-{}", std::process::id()));
        crate::search::bm25::build(&st, &dir).expect("build");
        let bm = Bm25::open(&dir, &st).expect("open");
        let ix = Indexes {
            store: &st,
            dense: None,
            bm25: Some(&bm),
        };
        let cfg = RetrievalConfig {
            mode: Mode::Bm25,
            k: 3,
            ..RetrievalConfig::default()
        };
        let mut no_embed = |_: &str| -> Result<Vec<f32>> { bail!("not used") };
        let base = retrieve(
            "alpha beta",
            &cfg,
            &ix,
            &mut Models {
                embed_query: &mut no_embed,
                rerank: None,
            },
        )
        .expect("bm25");
        assert_eq!(base.len(), 2);

        // A reranker that prefers "beta" flips the head.
        let fake = |_: &str, docs: &[&str]| -> Result<Vec<f32>> {
            Ok(docs
                .iter()
                .map(|d| if *d == "beta" { 9.0 } else { 1.0 })
                .collect())
        };
        let cfg_r = RetrievalConfig {
            rerank_top: 2,
            ..cfg.clone()
        };
        let rr = retrieve(
            "alpha beta",
            &cfg_r,
            &ix,
            &mut Models {
                embed_query: &mut no_embed,
                rerank: Some(&fake),
            },
        )
        .expect("rerank");
        assert_eq!(st.chunks[rr[0].row].text, "beta");
        assert!(rr[0].fused_rank.is_some());

        // Asking for a stage without its index is an error, not a silent skip.
        let cfg_d = RetrievalConfig {
            mode: Mode::Dense,
            ..cfg
        };
        assert!(
            retrieve(
                "x",
                &cfg_d,
                &ix,
                &mut Models {
                    embed_query: &mut no_embed,
                    rerank: None
                }
            )
            .is_err()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
