//! Tier A: golden files. No statistics, no thresholds: the facts of the
//! corpus and the exact top-10 ids for a fixed query list, compared byte for
//! byte against a committed snapshot. Any difference is either an intended
//! change (re-record with `--update`) or a regression, and a human decides
//! which.
//!
//! Dense and reranked rankings depend on the model file and the ONNX Runtime
//! build (INT8 kernels differ between Apple Silicon and x86), so those lists
//! record the model shas they were made with and are compared only when the
//! shas match. BM25 is compared everywhere.

use std::collections::{BTreeMap, BTreeSet};
use std::io::BufRead as _;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::search::store::ChunkStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorpusFacts {
    pub chunks_sha: String,
    pub chunk_count: usize,
    pub story_count: usize,
    /// Token count at p0, p10, ..., p100.
    pub token_deciles: Vec<u32>,
    pub chunks_over_600: usize,
    pub distinct_speakers: usize,
    pub unresolved_rows: usize,
    pub spoiler_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedSnapshot {
    /// Every model sha this list depends on; empty for BM25.
    pub depends_on: BTreeMap<String, String>,
    /// (query, top-10 chunk ids) in query-file order.
    pub results: Vec<(String, Vec<String>)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Golden {
    pub corpus: CorpusFacts,
    /// Keyed by configuration name (`bm25`, `dense`, `hybrid`, `hybrid+rerank`).
    pub rankings: BTreeMap<String, RankedSnapshot>,
}

fn count_lines(path: &Path) -> Result<usize> {
    let f = std::fs::File::open(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(std::io::BufReader::new(f).lines().count())
}

/// # Errors
/// A missing `unresolved.jsonl` or `spoiler.jsonl`.
pub fn corpus_facts(store: &ChunkStore, corpus: &Path) -> Result<CorpusFacts> {
    let mut tokens: Vec<u32> = store.chunks.iter().map(|c| c.token_count).collect();
    tokens.sort_unstable();
    let deciles = (0..=10)
        .map(|d| {
            if tokens.is_empty() {
                return 0;
            }
            let i = (d * (tokens.len() - 1)) / 10;
            tokens[i]
        })
        .collect();
    let speakers: BTreeSet<&str> = store
        .chunks
        .iter()
        .flat_map(|c| c.speakers.iter().map(String::as_str))
        .collect();
    let stories: BTreeSet<&str> = store.chunks.iter().map(|c| c.story_id.as_str()).collect();
    Ok(CorpusFacts {
        chunks_sha: store.chunks_sha.clone(),
        chunk_count: store.len(),
        story_count: stories.len(),
        token_deciles: deciles,
        chunks_over_600: tokens.iter().filter(|&&t| t > 600).count(),
        distinct_speakers: speakers.len(),
        unresolved_rows: count_lines(&corpus.join("unresolved.jsonl"))?,
        spoiler_rows: count_lines(&corpus.join("spoiler.jsonl"))?,
    })
}

/// One line per difference; empty means identical. Rankings whose model
/// shas differ are reported as skipped, not as differences.
#[must_use]
pub fn diff(expected: &Golden, actual: &Golden) -> (Vec<String>, Vec<String>) {
    let mut diffs = Vec::new();
    let mut skipped = Vec::new();
    let (e, a) = (
        serde_json::to_value(&expected.corpus).unwrap_or_default(),
        serde_json::to_value(&actual.corpus).unwrap_or_default(),
    );
    if let (Some(e), Some(a)) = (e.as_object(), a.as_object()) {
        for (k, ev) in e {
            let av = a.get(k).cloned().unwrap_or_default();
            if *ev != av {
                diffs.push(format!("corpus.{k}: expected {ev}, got {av}"));
            }
        }
    }
    for (name, exp) in &expected.rankings {
        let Some(act) = actual.rankings.get(name) else {
            skipped.push(format!("{name}: not computed in this run"));
            continue;
        };
        if exp.depends_on != act.depends_on {
            skipped.push(format!(
                "{name}: recorded with {:?}, this run uses {:?}",
                exp.depends_on, act.depends_on
            ));
            continue;
        }
        for ((q, ids), (_, got)) in exp.results.iter().zip(&act.results) {
            if ids != got {
                let first = ids
                    .iter()
                    .zip(got)
                    .position(|(x, y)| x != y)
                    .unwrap_or(ids.len().min(got.len()));
                diffs.push(format!("{name}: \"{q}\" differs from rank {}", first + 1));
            }
        }
        if exp.results.len() != act.results.len() {
            diffs.push(format!(
                "{name}: {} queries recorded, {} run",
                exp.results.len(),
                act.results.len()
            ));
        }
    }
    (diffs, skipped)
}
