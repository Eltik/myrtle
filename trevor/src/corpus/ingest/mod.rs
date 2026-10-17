//! Phase 0 ingest: pull the story library and every scripted story from the
//! running backend, chunk each one, and write the corpus artifacts.
//!
//! The backend is the parser. `GET /api/story/{id}` already resolves the
//! script probe, folds the 16-entry typo table, joins backslash continuations
//! and decodes argument values, so nothing here re-reads `.txt.txt` files.
//!
//! Artifacts, under the output directory:
//!
//! * `chunks.jsonl`: one [`Chunk`] per line, stories in a fixed sorted order.
//!   Appended story by story and resumable.
//! * `spoiler.jsonl`: one line per scripted story with the fields the progress
//!   gate and any later ordering need. Rewritten whole each run.
//! * `unresolved.jsonl`: stories the index lists without a script, plus any
//!   fetch that failed this run.
//! * `manifest.p0.json`: what this run did, and the checks it ran.
//! * `changes.json` and `chunks.prev.jsonl`, with `refresh` only: which
//!   stories were added, changed, removed or unchanged, and the file as it was
//!   before the refresh replaced it.
//!
//! Every story is VERIFIED as it is written: its chunks must reproduce its
//! prose word for word, and the chunker's own word count must equal the
//! `wordCount` the API reports. Both are counted in the manifest, and the
//! binary exits nonzero when either is not zero.

use std::collections::HashSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};

use crate::util::sha_hex;
use crate::corpus::chunk::{
    Chunk, ChunkConfig, StoryCommand, chunk_story,
};

mod jobs;
mod resume;
mod refresh;
mod verify;
mod fetch;
#[cfg(test)]
mod tests;

pub use fetch::{Fetched, Fetcher};
pub use jobs::{Job, Source, SpoilerRow, plan};
pub use refresh::{ChangeCounts, ChangeReport, OldCorpus, Refresh};
pub use resume::{Recovery, recover};
pub use verify::reconstructs;
use verify::prose_words;

// ---------------------------------------------------------------------------
// Wire types. Only the fields this crate reads are modelled: modelling fields
// it does not use is how a sibling crate breaks on the next backend change.
// serde ignores the rest. Checked against backend/src/app/services/story.rs
// and backend/src/core/story/mod.rs.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryIndex {
    pub groups: Vec<StoryGroup>,
    #[serde(default)]
    pub records: Vec<OperatorRecordGroup>,
    pub totals: StoryTotals,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryTotals {
    pub stories: u32,
    pub with_script: u32,
    pub words: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryGroup {
    pub id: String,
    pub name: String,
    /// A `StoryCategory` enum on the backend. Kept as raw JSON so a new
    /// variant cannot fail the whole index parse.
    #[serde(default)]
    pub category: serde_json::Value,
    #[serde(default)]
    pub entry_type: String,
    #[serde(default)]
    pub act_type: String,
    /// Release time. `-1` on the mainline zones whose open time the tables do
    /// not carry, so it is NOT a chronology and is not used as one here.
    #[serde(default)]
    pub start_time: i64,
    #[serde(default)]
    pub chapter_number: Option<u32>,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorRecordGroup {
    pub char_id: String,
    pub name: String,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    pub sort: i32,
    #[serde(default)]
    pub avg_tag: Option<String>,
    pub group_id: String,
    pub has_script: bool,
    pub word_count: u32,
    #[serde(default)]
    pub required_stages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryScript {
    pub id: String,
    pub commands: Vec<StoryCommand>,
    pub word_count: u32,
}
// ---------------------------------------------------------------------------
// Run
// ---------------------------------------------------------------------------

struct Outcome {
    lossless: bool,
    words: usize,
    api_words: u32,
    chunks: Vec<Chunk>,
}

pub struct IngestConfig {
    pub base: String,
    pub out: PathBuf,
    pub concurrency: usize,
    pub limit: Option<usize>,
    pub service_key: Option<String>,
    pub chunk: ChunkConfig,
    /// sha256 of the tokenizer file, recorded so the embedder can refuse a
    /// corpus chunked with a different one.
    pub tokenizer_sha: String,
    /// Refetch every story and replace the chunks of those that changed,
    /// instead of skipping every story id already in `chunks.jsonl`. False is
    /// the resume behavior, unchanged.
    pub refresh: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unresolved {
    pub story_id: String,
    pub group_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkConfigRecord {
    pub target_min_tokens: u32,
    pub target_max_tokens: u32,
    pub max_scenes_per_chunk: u32,
    pub overlap_turns: u32,
    pub sequence_overhead_tokens: u32,
    pub joiner_tokens: u32,
}

impl From<&ChunkConfig> for ChunkConfigRecord {
    fn from(c: &ChunkConfig) -> Self {
        Self {
            target_min_tokens: c.target_min_tokens,
            target_max_tokens: c.target_max_tokens,
            max_scenes_per_chunk: c.max_scenes_per_chunk,
            overlap_turns: c.overlap_turns,
            sequence_overhead_tokens: c.sequence_overhead_tokens,
            joiner_tokens: c.joiner_tokens,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub phase: &'static str,
    pub built_at_unix: u64,
    pub backend_base: String,
    pub totals_from_api: StoryTotals,
    /// Distinct scripted stories after deduplicating record listings.
    pub scripted_stories: usize,
    pub unscripted_stories: usize,
    pub stories_skipped_as_done: usize,
    pub stories_fetched: usize,
    pub stories_failed: usize,
    pub chunks_written_this_run: usize,
    pub chunks_in_file: usize,
    pub lossless_failures: Vec<String>,
    pub word_count_mismatches: Vec<String>,
    pub recovery: Recovery,
    pub tokenizer_sha: String,
    pub chunk_config: ChunkConfigRecord,
    /// Set by a refresh run only; absent otherwise, so a resume run writes
    /// the manifest it always wrote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh: Option<ChangeCounts>,
}

impl Manifest {
    #[must_use]
    pub fn clean(&self) -> bool {
        self.lossless_failures.is_empty()
            && self.word_count_mismatches.is_empty()
            && self.stories_failed == 0
    }
}

/// Rename so a reader never sees a half-written derived file.
fn write_jsonl<T: Serialize>(path: &Path, rows: impl IntoIterator<Item = T>) -> Result<()> {
    crate::util::write_jsonl_atomic(path, "jsonl.tmp", rows)
}

/// Run the ingest end to end.
///
/// # Errors
/// Anything that stops the run as a whole: an unreachable index, an
/// unwritable output directory, a corrupt existing `chunks.jsonl`.
/// Per-story failures do not stop the run; they are recorded.
pub async fn run(
    cfg: &IngestConfig,
    count_tokens: Arc<dyn Fn(&str) -> u32 + Send + Sync>,
) -> Result<Manifest> {
    if cfg.refresh && cfg.limit.is_some() {
        // A refresh drops every old story the plan does not list, so a
        // truncated plan would delete the rest of the corpus.
        return Err(anyhow!("--refresh cannot be combined with --limit"));
    }
    std::fs::create_dir_all(&cfg.out)?;
    let fetcher = Arc::new(Fetcher::new(&cfg.base, cfg.service_key.clone())?);

    let index = fetcher.index().await.context("fetching the story index")?;
    let (mut jobs, unscripted) = plan(&index);
    if let Some(n) = cfg.limit {
        jobs.truncate(n);
    }

    // Spoiler metadata depends only on the index: rewrite it whole, sorted.
    write_jsonl(
        &cfg.out.join("spoiler.jsonl"),
        jobs.iter().map(|j| SpoilerRow {
            story_id: &j.entry.id,
            group_id: &j.entry.group_id,
            source: &j.source,
            story_sort: j.entry.sort,
            code: j.entry.code.as_deref(),
            avg_tag: j.entry.avg_tag.as_deref(),
            required_stages: &j.entry.required_stages,
            group_start_time: j.group_start_time,
            chapter_number: j.chapter_number,
            entry_type: &j.entry_type,
            act_type: &j.act_type,
            category: &j.category,
        }),
    )?;

    let chunks_path = cfg.out.join("chunks.jsonl");
    // Refresh reads the old file without repairing it and writes a new one
    // beside it; resume repairs the old file and appends to it.
    let refresh_tmp = cfg.out.join("chunks.jsonl.refresh");
    let (old_bytes, old) = if cfg.refresh {
        let b = match std::fs::read(&chunks_path) {
            Ok(b) => Some(b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e).context("reading chunks.jsonl"),
        };
        let old = OldCorpus::parse(b.as_deref().unwrap_or_default())?;
        (b, old)
    } else {
        (None, OldCorpus::default())
    };
    let mut refresh = cfg.refresh.then(|| Refresh::new(&old));
    let (done, recovery) = if cfg.refresh {
        let rec = Recovery {
            torn_bytes_discarded: old.torn_bytes,
            ..Recovery::default()
        };
        (HashSet::new(), rec)
    } else {
        recover(&chunks_path)?
    };
    let todo: Vec<Job> = jobs
        .iter()
        .filter(|j| !done.contains(&j.entry.id))
        .cloned()
        .collect();

    let mut file = if cfg.refresh {
        std::fs::File::create(&refresh_tmp)?
    } else {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&chunks_path)?
    };

    let chunk_cfg = cfg.chunk;
    // `buffered` keeps results in INPUT order while fetching `concurrency` at
    // once, so the file layout never depends on network timing.
    let mut results = futures::stream::iter(todo.into_iter().map(|job| {
        let fetcher = Arc::clone(&fetcher);
        let count = Arc::clone(&count_tokens);
        async move {
            let (id, gid) = (job.entry.id.clone(), job.entry.group_id.clone());
            let script = match fetcher.script(&id).await {
                Fetched::Script(s) => s,
                Fetched::Failed { status, error } => {
                    let why = status.map_or(error.clone(), |s| format!("http {s}: {error}"));
                    return (id, gid, Err(why));
                }
            };
            // Tokenizing is CPU work; keep it off the async workers.
            let gid2 = gid.clone();
            let done = tokio::task::spawn_blocking(move || {
                let chunks = chunk_story(&script.id, &gid2, &script.commands, &chunk_cfg, &|s| {
                    count(s)
                });
                Outcome {
                    lossless: reconstructs(&script.commands, &chunks),
                    words: prose_words(&script.commands).len(),
                    api_words: script.word_count,
                    chunks,
                }
            })
            .await
            .map_err(|e| format!("chunking panicked: {e}"));
            (id, gid, done)
        }
    }))
    .buffered(cfg.concurrency.max(1));

    let mut unresolved: Vec<Unresolved> = unscripted
        .iter()
        .map(|e| Unresolved {
            story_id: e.id.clone(),
            group_id: e.group_id.clone(),
            reason: "index reports hasScript=false".into(),
        })
        .collect();
    let (mut fetched, mut failed, mut written) = (0usize, 0usize, 0usize);
    let (mut lossless_failures, mut word_mismatches) = (Vec::new(), Vec::new());

    while let Some((id, gid, outcome)) = results.next().await {
        let o = match outcome {
            Ok(o) => o,
            Err(reason) => {
                failed += 1;
                if let Some(r) = refresh.as_mut() {
                    let keep = r.story(&id, None);
                    file.write_all(keep)?;
                }
                unresolved.push(Unresolved {
                    story_id: id,
                    group_id: gid,
                    reason,
                });
                continue;
            }
        };
        fetched += 1;
        if !o.lossless {
            lossless_failures.push(id.clone());
        }
        if u32::try_from(o.words).ok() != Some(o.api_words) {
            word_mismatches.push(format!("{id}: chunker {} vs api {}", o.words, o.api_words));
        }
        // One write per story, so a crash leaves at most one story torn,
        // which `recover` drops on the next run.
        let mut buf = Vec::new();
        for c in &o.chunks {
            serde_json::to_writer(&mut buf, c)?;
            buf.push(b'\n');
        }
        match refresh.as_mut() {
            Some(r) => {
                let out = r.story(&id, Some(&buf));
                file.write_all(out)?;
            }
            None => {
                file.write_all(&buf)?;
                file.flush()?;
                written += o.chunks.len();
            }
        }
    }
    file.flush()?;
    drop(file);

    let changes = match refresh {
        Some(r) => {
            let mut report = r.finish();
            written = report.counts.chunks_added_or_changed;
            let new_bytes = std::fs::read(&refresh_tmp).context("re-reading the refreshed file")?;
            report.old_chunks_sha = old_bytes.as_deref().map(sha_hex);
            report.new_chunks_sha = sha_hex(&new_bytes);
            if old_bytes.as_deref() == Some(new_bytes.as_slice()) {
                // Nothing changed: leave chunks.jsonl, its mtime included,
                // exactly as it was.
                std::fs::remove_file(&refresh_tmp)?;
            } else {
                if old_bytes.is_some() {
                    // Kept so embed-corpus can still bootstrap reuse from a
                    // meta that predates `contentShas`, and for diffing.
                    std::fs::rename(&chunks_path, cfg.out.join("chunks.prev.jsonl"))?;
                }
                std::fs::rename(&refresh_tmp, &chunks_path)?;
            }
            report.built_at_unix = crate::util::unix_now();
            let tmp = cfg.out.join("changes.json.tmp");
            std::fs::write(&tmp, serde_json::to_vec_pretty(&report)?)?;
            std::fs::rename(&tmp, cfg.out.join("changes.json"))?;
            Some(report.counts)
        }
        None => None,
    };

    write_jsonl(&cfg.out.join("unresolved.jsonl"), &unresolved)?;

    // A read failure here must not report an empty corpus as clean.
    let chunks_in_file = std::fs::read(&chunks_path)
        .context("re-reading chunks.jsonl to count it")?
        .iter()
        .filter(|&&c| c == b'\n')
        .count();
    let manifest = Manifest {
        phase: "p0-ingest",
        built_at_unix: crate::util::unix_now(),
        backend_base: cfg.base.clone(),
        totals_from_api: index.totals,
        scripted_stories: jobs.len(),
        unscripted_stories: unscripted.len(),
        stories_skipped_as_done: done.len(),
        stories_fetched: fetched,
        stories_failed: failed,
        chunks_written_this_run: written,
        chunks_in_file,
        lossless_failures,
        word_count_mismatches: word_mismatches,
        recovery,
        tokenizer_sha: cfg.tokenizer_sha.clone(),
        chunk_config: (&cfg.chunk).into(),
        refresh: changes,
    };
    std::fs::write(
        cfg.out.join("manifest.p0.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(manifest)
}
