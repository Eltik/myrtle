//! Exact dense search over `vectors.bin`.
//!
//! Brute force on purpose: 13,837 x 768 is 10.6M multiply-adds per query,
//! about a millisecond, and exact. An approximate index would add a recall
//! loss to measure and a build step to keep in sync, for no latency a user
//! could notice at this size.

use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use super::store::ChunkStore;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorsMeta {
    pub model_file: String,
    pub model_sha: String,
    pub tokenizer_sha: String,
    pub chunks_sha: String,
    pub dim: usize,
    pub count: usize,
    pub normalized: bool,
    pub chunk_ids: Vec<String>,
}

pub struct DenseIndex {
    pub meta: VectorsMeta,
    vectors: Vec<f32>,
}

impl DenseIndex {
    /// Load `vectors.bin` and its meta from `dir`, refusing vectors built from
    /// different chunks or in a different row order.
    ///
    /// # Errors
    /// Missing files, a size mismatch, or stale vectors.
    pub fn load(dir: &Path, store: &ChunkStore) -> Result<Self> {
        let meta: VectorsMeta = serde_json::from_slice(
            &std::fs::read(dir.join("vectors.meta.json"))
                .context("reading vectors.meta.json; run embed-corpus")?,
        )?;
        if meta.chunks_sha != store.chunks_sha {
            bail!(
                "vectors were built from chunks {:.16}, but chunks.jsonl is {:.16}; run embed-corpus",
                meta.chunks_sha,
                store.chunks_sha
            );
        }
        if !meta.normalized {
            bail!(
                "vectors.meta.json says the vectors are not normalized; dot product is not cosine"
            );
        }
        if meta.count != store.len() || meta.chunk_ids.len() != store.len() {
            bail!(
                "vectors hold {} rows, chunks.jsonl {}",
                meta.count,
                store.len()
            );
        }
        if let Some((row, id)) = meta
            .chunk_ids
            .iter()
            .enumerate()
            .find(|(row, id)| store.chunks[*row].chunk_id != **id)
        {
            bail!(
                "vector row {row} is {id}, chunks.jsonl row {row} is {}",
                store.chunks[row].chunk_id
            );
        }
        let bytes = std::fs::read(dir.join("vectors.bin")).context("reading vectors.bin")?;
        let want = meta.count * meta.dim * 4;
        if bytes.len() != want {
            bail!("vectors.bin is {} bytes, expected {want}", bytes.len());
        }
        let vectors = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        Ok(Self { meta, vectors })
    }

    #[must_use]
    pub fn dim(&self) -> usize {
        self.meta.dim
    }

    #[must_use]
    pub fn row(&self, i: usize) -> &[f32] {
        &self.vectors[i * self.meta.dim..(i + 1) * self.meta.dim]
    }

    /// Top `k` rows by dot product with the unit-length `query`, best first,
    /// ties broken by row.
    ///
    /// # Errors
    /// A query of the wrong dimension.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>> {
        if query.len() != self.meta.dim {
            bail!("query has dim {}, index {}", query.len(), self.meta.dim);
        }
        let mut scored: Vec<(usize, f32)> = self
            .vectors
            .chunks_exact(self.meta.dim)
            .enumerate()
            .map(|(i, v)| (i, v.iter().zip(query).map(|(a, b)| a * b).sum::<f32>()))
            .collect();
        let by = |a: &(usize, f32), b: &(usize, f32)| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0));
        if k < scored.len() {
            scored.select_nth_unstable_by(k, by);
            scored.truncate(k);
        }
        scored.sort_by(by);
        Ok(scored)
    }
}
