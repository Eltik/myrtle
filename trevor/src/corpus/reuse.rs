//! Vector reuse for `embed-corpus`: copy the stored vector of every chunk
//! whose embedded text is unchanged, and embed only the rest.
//!
//! A full embed is 28 min for 13,837 chunks; a monthly event adds about 150.
//! Reuse is exact, not approximate: every text is embedded alone in its own
//! ONNX call (see `embed.rs`), so a vector depends only on its text and the
//! model, and a stored vector is the bytes a fresh call would produce. Rows
//! are copied as raw bytes, never re-encoded.
//!
//! The key of a row is `sha16` of the text actually embedded: `text` alone in
//! P0 (where it equals the row's `contentSha`), `prefix + "\n\n" + text` when
//! a P1 prefix is embedded. Keying on `contentSha` alone would reuse a stale
//! vector when only the prefix changed. `vectors.meta.json` stores the keys
//! in row order as `contentShas`.
//!
//! A prior vector set is used only when its model sha, tokenizer sha, dtype,
//! layout, pooling, normalization and batch size all match this run's, and
//! its files agree with each other in size. Anything else is skipped with a
//! reason and the run embeds those rows; a bad prior never fails the run.
//!
//! Metas written before `contentShas` existed are bootstrapped: when the
//! `chunks.jsonl` (or `chunks.prev.jsonl`, the copy `build-corpus --refresh`
//! keeps) beside them hashes to the meta's `chunksSha` and pairs with its
//! `chunkIds` row for row, the keys are derived from it.

use std::collections::HashMap;
use std::io::BufRead as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::corpus::chunk::{indexed_text, sha16};

/// The values `embed-corpus` writes for the fields a prior must share.
pub const DTYPE: &str = "f32-le";
pub const LAYOUT: &str = "row-major [count, dim]";
pub const POOLING: &str = "cls";
pub const BATCH_SIZE: usize = 1;

/// The reuse key of one row: `sha16` of the text the embedder sees.
#[must_use]
pub fn embed_key(prefix: Option<&str>, text: &str) -> String {
    sha16(&indexed_text(prefix, text))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PriorMeta {
    model_sha: String,
    tokenizer_sha: String,
    chunks_sha: String,
    dim: usize,
    count: usize,
    dtype: String,
    layout: String,
    pooling: String,
    normalized: bool,
    batch_size: usize,
    chunk_ids: Vec<String>,
    #[serde(default)]
    content_shas: Option<Vec<String>>,
    #[serde(default)]
    prefix_ignored: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyLine {
    chunk_id: String,
    text: String,
    #[serde(default)]
    prefix: Option<String>,
}

/// What happened to one prior directory, for the run's log.
#[derive(Debug, Clone)]
pub struct SourceReport {
    pub dir: PathBuf,
    /// `Ok(how)` with the number of rows added, or `Err(reason)` when skipped.
    pub outcome: std::result::Result<(usize, &'static str), String>,
}

/// Stored vectors by reuse key, pooled from one or more prior runs.
#[derive(Default)]
pub struct ReusePool {
    dim: Option<usize>,
    rows: HashMap<String, Vec<u8>>,
    pub sources: Vec<SourceReport>,
}

impl ReusePool {
    #[must_use]
    pub fn dim(&self) -> Option<usize> {
        self.dim
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The stored row for `key`, as the little-endian f32 bytes to write.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.rows.get(key).map(Vec::as_slice)
    }

    /// Add the vectors in `dir` when they were built by the same model and
    /// tokenizer. Never fails: a skipped directory is recorded in `sources`.
    /// The first directory added wins on a duplicate key; identical keys
    /// carry identical vectors anyway, by the one-text-per-call rule.
    pub fn add_dir(&mut self, dir: &Path, model_sha: &str, tokenizer_sha: &str) {
        let outcome = self
            .load(dir, model_sha, tokenizer_sha)
            .map_err(|e| format!("{e:#}"));
        self.sources.push(SourceReport {
            dir: dir.to_path_buf(),
            outcome,
        });
    }

    fn load(
        &mut self,
        dir: &Path,
        model_sha: &str,
        tokenizer_sha: &str,
    ) -> Result<(usize, &'static str)> {
        let meta_path = dir.join("vectors.meta.json");
        let meta_bytes = match std::fs::read(&meta_path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => bail!("no vectors.meta.json"),
            Err(e) => return Err(e).context("reading vectors.meta.json"),
        };
        let meta: PriorMeta =
            serde_json::from_slice(&meta_bytes).context("parsing vectors.meta.json")?;
        if meta.model_sha != model_sha {
            bail!("model sha {:.16} differs from this run's", meta.model_sha);
        }
        if meta.tokenizer_sha != tokenizer_sha {
            bail!("tokenizer sha {:.16} differs from this run's", meta.tokenizer_sha);
        }
        if meta.dtype != DTYPE
            || meta.layout != LAYOUT
            || meta.pooling != POOLING
            || !meta.normalized
            || meta.batch_size != BATCH_SIZE
        {
            bail!(
                "built as {} {} {} normalized={} batch={}, not this run's settings",
                meta.dtype,
                meta.layout,
                meta.pooling,
                meta.normalized,
                meta.batch_size
            );
        }
        if meta.dim == 0 {
            bail!("dim 0");
        }
        if let Some(d) = self.dim
            && d != meta.dim
        {
            bail!("dim {} differs from an earlier source's {d}", meta.dim);
        }
        if meta.chunk_ids.len() != meta.count {
            bail!("{} chunk ids for count {}", meta.chunk_ids.len(), meta.count);
        }

        let (keys, how) = match meta.content_shas.clone() {
            Some(k) => (k, "contentShas"),
            None => (bootstrap_keys(dir, &meta)?, "bootstrapped from chunks"),
        };
        if keys.len() != meta.count {
            bail!("{} contentShas for count {}", keys.len(), meta.count);
        }

        let bin = std::fs::read(dir.join("vectors.bin")).context("reading vectors.bin")?;
        let row_bytes = meta.dim * 4;
        if bin.len() != meta.count * row_bytes {
            bail!(
                "vectors.bin is {} bytes, expected {} x {} x 4",
                bin.len(),
                meta.count,
                meta.dim
            );
        }
        let before = self.rows.len();
        for (key, row) in keys.into_iter().zip(bin.chunks_exact(row_bytes)) {
            self.rows.entry(key).or_insert_with(|| row.to_vec());
        }
        self.dim = Some(meta.dim);
        Ok((self.rows.len() - before, how))
    }
}

/// Derive the keys of a meta that predates `contentShas` from the chunks file
/// it was built from, found by sha among `chunks.jsonl` and
/// `chunks.prev.jsonl`.
fn bootstrap_keys(dir: &Path, meta: &PriorMeta) -> Result<Vec<String>> {
    for name in ["chunks.jsonl", "chunks.prev.jsonl"] {
        let Ok(bytes) = std::fs::read(dir.join(name)) else {
            continue;
        };
        if crate::util::sha_hex(&bytes) != meta.chunks_sha {
            continue;
        }
        let mut keys = Vec::with_capacity(meta.count);
        for (i, line) in bytes.lines().enumerate() {
            let row: KeyLine = serde_json::from_str(&line?)
                .with_context(|| format!("{name} line {}", i + 1))?;
            if meta.chunk_ids.get(i) != Some(&row.chunk_id) {
                bail!("{name} row {i} is {}, the meta says otherwise", row.chunk_id);
            }
            // The prefix the prior run embedded: none when it ran with
            // `--ignore-prefix`.
            let prefix = row.prefix.as_deref().filter(|_| !meta.prefix_ignored);
            keys.push(embed_key(prefix, &row.text));
        }
        if keys.len() != meta.count {
            bail!("{name} has {} rows, the meta {}", keys.len(), meta.count);
        }
        return Ok(keys);
    }
    bail!(
        "meta has no contentShas and no chunks file beside it hashes to its chunksSha {:.16}",
        meta.chunks_sha
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::chunk::{ChunkConfig, StoryCommand, chunk_story};

    fn tmpdir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("trevor-reuse-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("mkdir");
        p
    }

    /// Row `i` of a fake dim-2 vector set, distinct per row.
    fn row(i: usize) -> Vec<u8> {
        #[allow(clippy::cast_precision_loss)]
        let x = i as f32;
        [x, -x].iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    fn chunk_line(id: &str, text: &str, prefix: Option<&str>) -> String {
        let mut v = serde_json::json!({ "chunkId": id, "text": text, "contentSha": sha16(text) });
        if let Some(p) = prefix {
            v["prefix"] = p.into();
        }
        format!("{v}\n")
    }

    /// Write a prior run into `dir`: chunks.jsonl, vectors.bin, and a meta
    /// with or without `contentShas`.
    fn write_prior(dir: &Path, rows: &[(&str, &str, Option<&str>)], with_keys: bool) {
        let chunks: String = rows.iter().map(|(i, t, p)| chunk_line(i, t, *p)).collect();
        std::fs::write(dir.join("chunks.jsonl"), &chunks).expect("chunks");
        let bin: Vec<u8> = (0..rows.len()).flat_map(row).collect();
        std::fs::write(dir.join("vectors.bin"), bin).expect("bin");
        let mut meta = serde_json::json!({
            "modelSha": "m", "tokenizerSha": "t",
            "chunksSha": crate::util::sha_hex(chunks.as_bytes()),
            "dim": 2, "count": rows.len(), "dtype": DTYPE, "layout": LAYOUT,
            "pooling": POOLING, "normalized": true, "batchSize": 1,
            "chunkIds": rows.iter().map(|r| r.0).collect::<Vec<_>>(),
        });
        if with_keys {
            meta["contentShas"] = rows
                .iter()
                .map(|(_, t, p)| embed_key(*p, t))
                .collect::<Vec<_>>()
                .into();
        }
        std::fs::write(dir.join("vectors.meta.json"), meta.to_string()).expect("meta");
    }

    #[test]
    fn key_equals_content_sha_without_a_prefix() {
        let cmds = vec![StoryCommand {
            kind: "name".into(),
            args: [("name".to_owned(), "Amiya".to_owned())].into_iter().collect(),
            text: Some("Doctor, are you awake?".into()),
            line: 1,
        }];
        let words = |s: &str| u32::try_from(s.split_whitespace().count()).unwrap_or(0);
        let c = &chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words)[0];
        assert_eq!(embed_key(None, &c.text), c.content_sha);
        assert_ne!(embed_key(Some("Prefix"), &c.text), c.content_sha);
    }

    #[test]
    fn pool_reuses_rows_by_stored_keys() {
        let d = tmpdir("keys");
        write_prior(&d, &[("a", "alpha", None), ("b", "beta", None)], true);
        let mut pool = ReusePool::default();
        pool.add_dir(&d, "m", "t");
        assert_eq!(pool.sources[0].outcome, Ok((2, "contentShas")));
        assert_eq!(pool.dim(), Some(2));
        assert_eq!(pool.get(&embed_key(None, "beta")), Some(row(1).as_slice()));
        assert_eq!(pool.get(&embed_key(None, "gamma")), None);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn bootstrap_derives_keys_from_the_matching_chunks_file() {
        let d = tmpdir("boot");
        write_prior(&d, &[("a", "alpha", Some("P")), ("b", "beta", None)], false);
        let mut pool = ReusePool::default();
        pool.add_dir(&d, "m", "t");
        assert_eq!(pool.sources[0].outcome, Ok((2, "bootstrapped from chunks")));
        // The prior embedded the prefix, so the prefixed key is the one stored.
        assert_eq!(pool.get(&embed_key(Some("P"), "alpha")), Some(row(0).as_slice()));
        assert_eq!(pool.get(&embed_key(None, "alpha")), None);

        // After a refresh rewrote chunks.jsonl, the kept previous copy still
        // bootstraps.
        std::fs::rename(d.join("chunks.jsonl"), d.join("chunks.prev.jsonl")).expect("mv");
        std::fs::write(d.join("chunks.jsonl"), chunk_line("c", "new", None)).expect("w");
        let mut pool = ReusePool::default();
        pool.add_dir(&d, "m", "t");
        assert_eq!(pool.sources[0].outcome, Ok((2, "bootstrapped from chunks")));

        // With neither file matching the recorded sha, nothing is reused.
        std::fs::remove_file(d.join("chunks.prev.jsonl")).expect("rm");
        let mut pool = ReusePool::default();
        pool.add_dir(&d, "m", "t");
        assert!(pool.sources[0].outcome.is_err());
        assert!(pool.is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn mismatched_priors_are_skipped_not_fatal() {
        let d = tmpdir("skip");
        write_prior(&d, &[("a", "alpha", None)], true);
        for (model, tok) in [("other", "t"), ("m", "other")] {
            let mut pool = ReusePool::default();
            pool.add_dir(&d, model, tok);
            assert!(pool.sources[0].outcome.is_err(), "{model} {tok}");
            assert!(pool.is_empty());
        }
        // A truncated vectors.bin is refused rather than read short.
        std::fs::write(d.join("vectors.bin"), [0u8; 4]).expect("w");
        let mut pool = ReusePool::default();
        pool.add_dir(&d, "m", "t");
        assert!(pool.sources[0].outcome.is_err());
        // A missing directory is a skip too.
        let mut pool = ReusePool::default();
        pool.add_dir(&d.join("absent"), "m", "t");
        assert!(pool.sources[0].outcome.is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
