//! The chunk store every index is built from and resolved against.
//!
//! Row order is `chunks.jsonl` order, and it is the one identity shared by the
//! dense vectors (row i of `vectors.bin`), the BM25 index (stored `chunk_id`)
//! and the eval (line-span anchors resolved to rows). `chunks_sha` is the
//! sha256 of the file's bytes; every derived index records the sha it was
//! built from and refuses to load against a different one, because an index
//! over a stale chunk file returns plausible, wrong rows with no error.

use std::collections::HashMap;
use std::io::BufRead as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::corpus::chunk::Chunk;

pub struct ChunkStore {
    pub chunks: Vec<Chunk>,
    pub chunks_sha: String,
    by_id: HashMap<String, usize>,
    /// story id -> rows of that story, in chunk ordinal order.
    by_story: HashMap<String, Vec<usize>>,
}

impl ChunkStore {
    /// # Errors
    /// A missing or unparsable file, or a duplicated chunk id.
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let chunks_sha = format!("{:x}", Sha256::digest(&bytes));
        let mut chunks = Vec::new();
        for (i, line) in bytes.lines().enumerate() {
            let line = line?;
            chunks.push(
                serde_json::from_str::<Chunk>(&line)
                    .with_context(|| format!("{} line {}", path.display(), i + 1))?,
            );
        }
        Self::from_chunks(chunks, chunks_sha)
    }

    /// # Errors
    /// A duplicated chunk id.
    pub fn from_chunks(chunks: Vec<Chunk>, chunks_sha: String) -> Result<Self> {
        let mut by_id = HashMap::with_capacity(chunks.len());
        let mut by_story: HashMap<String, Vec<usize>> = HashMap::new();
        for (row, c) in chunks.iter().enumerate() {
            if by_id.insert(c.chunk_id.clone(), row).is_some() {
                bail!("chunk id {} appears twice", c.chunk_id);
            }
            by_story.entry(c.story_id.clone()).or_default().push(row);
        }
        for rows in by_story.values_mut() {
            rows.sort_by_key(|&r| chunks[r].ordinal);
        }
        Ok(Self {
            chunks,
            chunks_sha,
            by_id,
            by_story,
        })
    }

    #[must_use]
    pub fn row(&self, chunk_id: &str) -> Option<usize> {
        self.by_id.get(chunk_id).copied()
    }

    #[must_use]
    pub fn story_rows(&self, story_id: &str) -> &[usize] {
        self.by_story.get(story_id).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}
