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

use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};

use crate::corpus::chunk::{
    Chunk, ChunkConfig, PROSE_ARG_KINDS, PROSE_KINDS, StoryCommand, chunk_story,
};

// ---------------------------------------------------------------------------
// Wire types. Only the fields this crate reads are modelled: modelling fields
// it does not use is how a sibling crate breaks on the next backend change.
// serde ignores the rest. Verified against backend/src/app/services/story.rs
// and backend/src/core/story/mod.rs on 2026-09-24.
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
// Jobs
// ---------------------------------------------------------------------------

/// Where a story was listed. Record stories appear both in the index's
/// `record` groups and under their operator; the first listing wins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Source {
    Group,
    Record { char_id: String },
}

#[derive(Debug, Clone)]
pub struct Job {
    pub entry: StoryEntry,
    pub source: Source,
    pub group_start_time: i64,
    pub chapter_number: Option<u32>,
    pub entry_type: String,
    pub act_type: String,
    pub category: serde_json::Value,
}

/// One line of `spoiler.jsonl`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerRow<'a> {
    pub story_id: &'a str,
    pub group_id: &'a str,
    pub source: &'a Source,
    pub story_sort: i32,
    pub code: Option<&'a str>,
    pub avg_tag: Option<&'a str>,
    pub required_stages: &'a [String],
    pub group_start_time: i64,
    pub chapter_number: Option<u32>,
    pub entry_type: &'a str,
    pub act_type: &'a str,
    pub category: &'a serde_json::Value,
}

/// Turn the index into a deduplicated job list in a FIXED order.
///
/// The backend builds its group list by walking a map, so the order it
/// serves is not something to depend on: two runs against two backend starts
/// could lay the corpus out differently. The order here is groups by
/// `(startTime, id)`, their stories by `(sort, id)`, then records by
/// `(charId)` with the same story order. It is stable, not a chronology.
#[must_use]
pub fn plan(index: &StoryIndex) -> (Vec<Job>, Vec<StoryEntry>) {
    let mut groups: Vec<&StoryGroup> = index.groups.iter().collect();
    groups.sort_by(|a, b| a.start_time.cmp(&b.start_time).then(a.id.cmp(&b.id)));
    let mut records: Vec<&OperatorRecordGroup> = index.records.iter().collect();
    records.sort_by(|a, b| a.char_id.cmp(&b.char_id));

    let mut seen: HashSet<String> = HashSet::new();
    let mut jobs = Vec::new();
    let mut unscripted = Vec::new();

    let mut take = |entries: &[StoryEntry], mk: &dyn Fn(StoryEntry) -> Job| {
        let mut sorted: Vec<&StoryEntry> = entries.iter().collect();
        sorted.sort_by(|a, b| a.sort.cmp(&b.sort).then(a.id.cmp(&b.id)));
        for e in sorted {
            if !seen.insert(e.id.clone()) {
                continue;
            }
            if e.has_script {
                jobs.push(mk(e.clone()));
            } else {
                unscripted.push(e.clone());
            }
        }
    };

    for g in groups {
        take(&g.stories, &|entry| Job {
            entry,
            source: Source::Group,
            group_start_time: g.start_time,
            chapter_number: g.chapter_number,
            entry_type: g.entry_type.clone(),
            act_type: g.act_type.clone(),
            category: g.category.clone(),
        });
    }
    for r in records {
        take(&r.stories, &|entry| Job {
            entry,
            source: Source::Record {
                char_id: r.char_id.clone(),
            },
            group_start_time: -1,
            chapter_number: None,
            entry_type: String::new(),
            act_type: String::new(),
            category: serde_json::Value::Null,
        });
    }
    (jobs, unscripted)
}

// ---------------------------------------------------------------------------
// Resume
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recovery {
    /// Bytes of a torn final line discarded.
    pub torn_bytes_discarded: u64,
    /// The story whose lines were last in the file, dropped for refetch.
    pub dropped_last_story: Option<String>,
    pub lines_dropped: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoryIdOnly {
    story_id: String,
}

/// Prepare `chunks.jsonl` for appending and return the stories already done.
///
/// Two failure shapes are repaired. A process killed mid-write leaves a torn
/// final line, which is cut back to the last newline. And a story's chunks
/// are written in one write per story, so the LAST story in the file may be
/// only partly present with no way to tell from the lines alone. It is
/// always dropped and refetched: one story of redundant work, against the
/// alternative of silently keeping a story with missing chunks.
///
/// A story that yields ZERO chunks leaves no line, so it is never marked done
/// and is refetched on every run. In EN that is exactly one story,
/// `main_14_level_main_14-20_beg`, which is a single `[Video]`. Measured: a
/// rerun over a complete file fetches 2 stories (that one and the dropped
/// last story) and leaves `chunks.jsonl` byte-identical. Priced at one
/// redundant fetch, it does not earn a separate done-log.
///
/// # Errors
/// I/O failures, or a complete line that is not a chunk.
pub fn recover(path: &Path) -> Result<(HashSet<String>, Recovery)> {
    let mut rec = Recovery::default();
    let Ok(bytes) = std::fs::read(path) else {
        return Ok((HashSet::new(), rec));
    };

    let keep_to = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    rec.torn_bytes_discarded = (bytes.len() - keep_to) as u64;
    let body = &bytes[..keep_to];

    let mut offsets: Vec<(usize, String)> = Vec::new();
    let mut pos = 0usize;
    for line in body.split_inclusive(|&b| b == b'\n') {
        let row: StoryIdOnly = serde_json::from_slice(line)
            .with_context(|| format!("unparseable complete line at byte {pos}"))?;
        offsets.push((pos, row.story_id));
        pos += line.len();
    }

    let mut cut = keep_to;
    if let Some((_, last)) = offsets.last().cloned() {
        let first_of_last = offsets
            .iter()
            .find(|(_, id)| *id == last)
            .map_or(keep_to, |(off, _)| *off);
        rec.lines_dropped = offsets.iter().filter(|(_, id)| *id == last).count() as u64;
        rec.dropped_last_story = Some(last);
        cut = first_of_last;
    }

    if cut != bytes.len() {
        let f = std::fs::OpenOptions::new().write(true).open(path)?;
        f.set_len(cut as u64)?;
    }

    let done: HashSet<String> = offsets
        .into_iter()
        .filter(|(off, _)| *off < cut)
        .map(|(_, id)| id)
        .collect();
    Ok((done, rec))
}

// ---------------------------------------------------------------------------
// Refresh
// ---------------------------------------------------------------------------

/// An existing `chunks.jsonl`, split by story, as raw bytes.
///
/// Resume trusts a story id already in the file, so an asset update that
/// edits a story's text is never seen. Refresh refetches every story instead
/// and compares the new lines with these, byte for byte.
#[derive(Debug, Default)]
pub struct OldCorpus {
    by_story: HashMap<String, Vec<u8>>,
    order: Vec<String>,
    /// Bytes of a torn final line, ignored.
    pub torn_bytes: u64,
    pub lines: usize,
}

impl OldCorpus {
    /// Split a `chunks.jsonl` by story. A torn final line is ignored; a story
    /// left partial by a killed run then reads as changed and is replaced.
    ///
    /// # Errors
    /// A complete line that is not a chunk.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let keep_to = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        let mut old = Self {
            torn_bytes: (bytes.len() - keep_to) as u64,
            ..Self::default()
        };
        let mut pos = 0usize;
        for line in bytes[..keep_to].split_inclusive(|&b| b == b'\n') {
            let row: StoryIdOnly = serde_json::from_slice(line)
                .with_context(|| format!("unparseable complete line at byte {pos}"))?;
            pos += line.len();
            old.lines += 1;
            match old.by_story.get_mut(&row.story_id) {
                Some(buf) => buf.extend_from_slice(line),
                None => {
                    old.order.push(row.story_id.clone());
                    old.by_story.insert(row.story_id, line.to_vec());
                }
            }
        }
        Ok(old)
    }
}

/// `changes.json`: what a refresh changed, story by story. Every list is in
/// plan order (removed: in the old file's order), so two refreshes of the
/// same data write the same report apart from `builtAtUnix`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeReport {
    pub built_at_unix: u64,
    /// sha256 of `chunks.jsonl` before and after; equal when nothing changed,
    /// and the value every derived index records.
    pub old_chunks_sha: Option<String>,
    pub new_chunks_sha: String,
    pub counts: ChangeCounts,
    /// New story ids.
    pub added: Vec<String>,
    /// Stories whose chunk lines differ in any byte: text, chunking, or
    /// metadata. Their old chunks are replaced.
    pub changed: Vec<String>,
    /// Stories in the old file that the index no longer lists; dropped.
    pub removed: Vec<String>,
    pub unchanged: Vec<String>,
    /// Fetch failures. A failed story keeps its old chunks, if it had any.
    pub failed: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeCounts {
    pub added: usize,
    pub changed: usize,
    pub removed: usize,
    pub unchanged: usize,
    pub failed: usize,
    pub old_chunks: usize,
    pub new_chunks: usize,
    /// Lines of added and changed stories: what a reusing embedder embeds, at
    /// most.
    pub chunks_added_or_changed: usize,
}

/// Merge refetched stories into an old corpus, one story at a time in plan
/// order, and classify each.
pub struct Refresh<'a> {
    old: &'a OldCorpus,
    report: ChangeReport,
    seen: HashSet<String>,
}

fn line_count(b: &[u8]) -> usize {
    b.iter().filter(|&&c| c == b'\n').count()
}

impl<'a> Refresh<'a> {
    #[must_use]
    pub fn new(old: &'a OldCorpus) -> Self {
        Self {
            old,
            report: ChangeReport::default(),
            seen: HashSet::new(),
        }
    }

    /// Classify one story and return the bytes to write for it: the fetched
    /// lines, or on a failed fetch (`None`) the old lines, so a network error
    /// never deletes a story.
    pub fn story<'b>(&mut self, id: &str, fetched: Option<&'b [u8]>) -> &'b [u8]
    where
        'a: 'b,
    {
        self.seen.insert(id.to_owned());
        let old = self.old.by_story.get(id).map(Vec::as_slice);
        let out = match fetched {
            None => {
                self.report.failed.push(id.to_owned());
                old.unwrap_or_default()
            }
            Some(new) => {
                match old {
                    // A story that chunks to nothing (one in EN, a single
                    // `[Video]`) is absent from the file both times.
                    None if new.is_empty() => self.report.unchanged.push(id.to_owned()),
                    None => self.report.added.push(id.to_owned()),
                    Some(o) if o == new => self.report.unchanged.push(id.to_owned()),
                    Some(_) => self.report.changed.push(id.to_owned()),
                }
                if old != Some(new) {
                    self.report.counts.chunks_added_or_changed += line_count(new);
                }
                new
            }
        };
        self.report.counts.new_chunks += line_count(out);
        out
    }

    /// Close the report: every old story not seen this run is removed.
    #[must_use]
    pub fn finish(mut self) -> ChangeReport {
        self.report.removed = self
            .old
            .order
            .iter()
            .filter(|id| !self.seen.contains(*id))
            .cloned()
            .collect();
        let r = &mut self.report;
        r.counts.added = r.added.len();
        r.counts.changed = r.changed.len();
        r.counts.removed = r.removed.len();
        r.counts.unchanged = r.unchanged.len();
        r.counts.failed = r.failed.len();
        r.counts.old_chunks = self.old.lines;
        self.report
    }
}

// ---------------------------------------------------------------------------
// Verification
// ---------------------------------------------------------------------------

/// Prose words of a script by the backend's own rule
/// (`parser::word_count`, parser.rs:427).
fn prose_words(commands: &[StoryCommand]) -> Vec<&str> {
    commands
        .iter()
        .filter_map(|c| {
            if PROSE_KINDS.contains(&c.kind.as_str()) {
                c.text.as_deref()
            } else if PROSE_ARG_KINDS.contains(&c.kind.as_str()) {
                c.args.get("text").map(String::as_str)
            } else {
                None
            }
        })
        .filter(|t| !t.trim().is_empty())
        .flat_map(str::split_whitespace)
        .collect()
}

/// True when the chunks reproduce the script's prose exactly, word for word,
/// with no loss and no duplication. Each rendered line's `Speaker: ` prefix
/// is removed using the story's own speaker strings, longest first, so a
/// multi-word speaker is stripped whole.
#[must_use]
pub fn reconstructs(commands: &[StoryCommand], chunks: &[Chunk]) -> bool {
    let speakers: BTreeSet<&str> = commands
        .iter()
        .filter(|c| c.kind == "name")
        .filter_map(|c| c.args.get("name").map(String::as_str))
        .filter(|s| !s.trim().is_empty())
        .collect();
    let mut by_len: Vec<&str> = speakers.into_iter().collect();
    by_len.sort_by_key(|s| std::cmp::Reverse(s.len()));

    let want = prose_words(commands);
    let mut got: Vec<&str> = Vec::with_capacity(want.len());
    for c in chunks {
        for line in c.text.split('\n') {
            let body = by_len
                .iter()
                .find_map(|s| line.strip_prefix(s).and_then(|r| r.strip_prefix(": ")))
                .unwrap_or(line);
            got.extend(body.split_whitespace());
        }
    }
    got == want
}

// ---------------------------------------------------------------------------
// Fetch
// ---------------------------------------------------------------------------

pub struct Fetcher {
    client: reqwest::Client,
    base: reqwest::Url,
    service_key: Option<String>,
}

pub enum Fetched {
    Script(StoryScript),
    /// The index said `hasScript` but the fetch did not return one.
    Failed {
        status: Option<u16>,
        error: String,
    },
}

impl Fetcher {
    /// # Errors
    /// An unparseable base URL.
    pub fn new(base: &str, service_key: Option<String>) -> Result<Self> {
        let base = reqwest::Url::parse(base).with_context(|| format!("bad base url {base}"))?;
        if base.cannot_be_a_base() {
            return Err(anyhow!("base url cannot take a path: {base}"));
        }
        let client = reqwest::Client::builder()
            // The backend's own handler timeout is 30 s; anything past that is
            // not coming back.
            .timeout(Duration::from_secs(45))
            .build()?;
        Ok(Self {
            client,
            base,
            service_key,
        })
    }

    fn url(&self, segments: &[&str]) -> reqwest::Url {
        let mut u = self.base.clone();
        {
            let mut p = u.path_segments_mut().expect("checked in new");
            p.pop_if_empty();
            p.push("api");
            for s in segments {
                p.push(s);
            }
        }
        u
    }

    fn get(&self, url: reqwest::Url) -> reqwest::RequestBuilder {
        let r = self.client.get(url);
        match &self.service_key {
            // The backend exempts this header from its rate limiter.
            Some(k) => r.header("x-service-key", k),
            None => r,
        }
    }

    /// # Errors
    /// Transport failure or a non-success status on the index.
    pub async fn index(&self) -> Result<StoryIndex> {
        let resp = self.get(self.url(&["story", "index"])).send().await?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("GET /api/story/index -> {status}"));
        }
        Ok(resp.json().await?)
    }

    /// Fetch one script. Transport errors and 5xx retry three times with
    /// backoff; 4xx do not, because a 404 on a story the index calls scripted
    /// is a data problem that retrying cannot fix.
    pub async fn script(&self, id: &str) -> Fetched {
        let url = self.url(&["story", id]);
        let mut last = String::new();
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << (attempt - 1))).await;
            }
            match self.get(url.clone()).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return match resp.json::<StoryScript>().await {
                            Ok(s) => Fetched::Script(s),
                            Err(e) => Fetched::Failed {
                                status: Some(status.as_u16()),
                                error: format!("decode: {e}"),
                            },
                        };
                    }
                    if status.is_client_error() {
                        return Fetched::Failed {
                            status: Some(status.as_u16()),
                            error: status.to_string(),
                        };
                    }
                    last = status.to_string();
                }
                Err(e) => last = e.to_string(),
            }
        }
        Fetched::Failed {
            status: None,
            error: format!("after 4 attempts: {last}"),
        }
    }
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

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn sha_hex(bytes: &[u8]) -> String {
    use sha2::Digest as _;
    format!("{:x}", sha2::Sha256::digest(bytes))
}

fn write_jsonl<T: Serialize>(path: &Path, rows: impl IntoIterator<Item = T>) -> Result<()> {
    let tmp = path.with_extension("jsonl.tmp");
    {
        let mut w = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
        for r in rows {
            serde_json::to_writer(&mut w, &r)?;
            w.write_all(b"\n")?;
        }
        w.flush()?;
    }
    // Rename so a reader never sees a half-written derived file.
    std::fs::rename(&tmp, path)?;
    Ok(())
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
            report.built_at_unix = now_unix();
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
        built_at_unix: now_unix(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, group: &str, sort: i32, scripted: bool) -> StoryEntry {
        StoryEntry {
            id: id.into(),
            name: id.into(),
            code: None,
            sort,
            avg_tag: None,
            group_id: group.into(),
            has_script: scripted,
            word_count: 0,
            required_stages: vec![],
        }
    }

    fn group(id: &str, start: i64, stories: Vec<StoryEntry>) -> StoryGroup {
        StoryGroup {
            id: id.into(),
            name: id.into(),
            category: serde_json::Value::Null,
            entry_type: String::new(),
            act_type: String::new(),
            start_time: start,
            chapter_number: None,
            stories,
        }
    }

    fn index(groups: Vec<StoryGroup>, records: Vec<OperatorRecordGroup>) -> StoryIndex {
        StoryIndex {
            groups,
            records,
            totals: StoryTotals {
                stories: 0,
                with_script: 0,
                words: 0,
            },
        }
    }

    #[test]
    fn plan_is_stable_under_shuffled_input() {
        let a = index(
            vec![
                group(
                    "g2",
                    20,
                    vec![entry("b", "g2", 2, true), entry("a", "g2", 1, true)],
                ),
                group("g1", 10, vec![entry("c", "g1", 1, true)]),
            ],
            vec![],
        );
        let b = index(
            vec![
                group("g1", 10, vec![entry("c", "g1", 1, true)]),
                group(
                    "g2",
                    20,
                    vec![entry("a", "g2", 1, true), entry("b", "g2", 2, true)],
                ),
            ],
            vec![],
        );
        let ids = |i: &StoryIndex| {
            plan(i)
                .0
                .into_iter()
                .map(|j| j.entry.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&a), vec!["c", "a", "b"]);
        assert_eq!(ids(&a), ids(&b));
    }

    #[test]
    fn records_dedupe_against_groups_and_unscripted_are_set_aside() {
        let i = index(
            vec![group(
                "record",
                0,
                vec![
                    entry("r1", "record", 1, true),
                    entry("x", "record", 2, false),
                ],
            )],
            vec![OperatorRecordGroup {
                char_id: "char_002_amiya".into(),
                name: "Amiya".into(),
                stories: vec![entry("r1", "record", 1, true), entry("r2", "set", 1, true)],
            }],
        );
        let (jobs, unscripted) = plan(&i);
        let ids: Vec<_> = jobs.iter().map(|j| j.entry.id.as_str()).collect();
        assert_eq!(ids, vec!["r1", "r2"]);
        assert_eq!(jobs[0].source, Source::Group, "first listing wins");
        assert!(matches!(jobs[1].source, Source::Record { .. }));
        assert_eq!(unscripted.len(), 1);
    }

    fn tmpfile(name: &str, contents: &[u8]) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("trevor-ingest-{}-{name}", std::process::id()));
        std::fs::write(&p, contents).expect("write");
        p
    }

    #[test]
    fn recover_cuts_a_torn_line_and_drops_the_last_story() {
        let body = concat!(
            "{\"storyId\":\"s1\",\"x\":1}\n",
            "{\"storyId\":\"s1\",\"x\":2}\n",
            "{\"storyId\":\"s2\",\"x\":1}\n",
            "{\"storyId\":\"s2\",\"x\":2}\n",
            "{\"storyId\":\"s3\",\"x\":1}\n",
            "{\"storyId\":\"s3\",\"x\""
        );
        let p = tmpfile("torn", body.as_bytes());
        let (done, rec) = recover(&p).expect("recover");
        assert_eq!(
            rec.torn_bytes_discarded,
            "{\"storyId\":\"s3\",\"x\"".len() as u64
        );
        assert_eq!(rec.dropped_last_story.as_deref(), Some("s3"));
        assert_eq!(rec.lines_dropped, 1);
        let mut d: Vec<_> = done.into_iter().collect();
        d.sort();
        assert_eq!(d, vec!["s1", "s2"]);
        let after = std::fs::read_to_string(&p).expect("read");
        assert!(after.ends_with("{\"storyId\":\"s2\",\"x\":2}\n"));
        assert!(!after.contains("s3"));
    }

    #[test]
    fn recover_on_a_missing_file_is_empty() {
        let p = std::env::temp_dir().join("trevor-ingest-definitely-absent.jsonl");
        let _ = std::fs::remove_file(&p);
        let (done, rec) = recover(&p).expect("recover");
        assert!(done.is_empty());
        assert!(rec.dropped_last_story.is_none());
    }

    fn lines(story: &str, n: usize, tag: &str) -> Vec<u8> {
        (0..n)
            .map(|i| format!("{{\"storyId\":\"{story}\",\"n\":{i},\"t\":\"{tag}\"}}\n"))
            .collect::<String>()
            .into_bytes()
    }

    #[test]
    fn refresh_classifies_every_story_and_keeps_plan_order() {
        let old_bytes = [
            lines("a", 2, "x"),
            lines("b", 1, "x"),
            lines("gone", 3, "x"),
            lines("c", 2, "x"),
            lines("d", 1, "x"),
        ]
        .concat();
        let old = OldCorpus::parse(&old_bytes).expect("parse");
        assert_eq!((old.lines, old.torn_bytes), (9, 0));

        let mut r = Refresh::new(&old);
        let mut out = Vec::new();
        // Plan order puts the new story first; the file follows the plan.
        let (new, a, b, empty) = (
            lines("new", 2, "x"),
            lines("a", 2, "x"),
            lines("b", 2, "y"),
            Vec::new(),
        );
        out.extend_from_slice(r.story("new", Some(&new)));
        out.extend_from_slice(r.story("a", Some(&a)));
        out.extend_from_slice(r.story("b", Some(&b)));
        out.extend_from_slice(r.story("c", None));
        out.extend_from_slice(r.story("video", Some(&empty)));
        out.extend_from_slice(r.story("d", Some(&empty)));
        let rep = r.finish();

        assert_eq!(rep.added, ["new"]);
        assert_eq!(rep.changed, ["b", "d"], "d now chunks to nothing");
        assert_eq!(rep.unchanged, ["a", "video"]);
        assert_eq!(rep.failed, ["c"]);
        assert_eq!(rep.removed, ["gone"]);
        assert_eq!(
            rep.counts,
            ChangeCounts {
                added: 1,
                changed: 2,
                removed: 1,
                unchanged: 2,
                failed: 1,
                old_chunks: 9,
                new_chunks: 8,
                chunks_added_or_changed: 4,
            }
        );
        // A failed fetch keeps the story's old lines.
        let want = [new, a, b, lines("c", 2, "x")].concat();
        assert_eq!(out, want);
    }

    #[test]
    fn refresh_of_unchanged_data_reproduces_the_file_and_ignores_a_torn_tail() {
        let body = [lines("a", 2, "x"), lines("b", 3, "x")].concat();
        let mut torn = body.clone();
        torn.extend_from_slice(b"{\"storyId\":\"b\",\"n\"");
        let old = OldCorpus::parse(&torn).expect("parse");
        assert_eq!(old.torn_bytes, 18);
        let mut r = Refresh::new(&old);
        let mut out = Vec::new();
        out.extend_from_slice(r.story("a", Some(&lines("a", 2, "x"))));
        out.extend_from_slice(r.story("b", Some(&lines("b", 3, "x"))));
        let rep = r.finish();
        assert_eq!(out, body);
        assert_eq!(rep.unchanged, ["a", "b"]);
        assert_eq!(rep.counts.chunks_added_or_changed, 0);
    }

    // ---- end to end against a fake backend -------------------------------

    type Routes = Arc<std::sync::Mutex<HashMap<String, String>>>;

    /// A one-thread HTTP/1.1 server answering GETs from `routes`, 404
    /// otherwise. Enough for `Fetcher`; it never sees a real backend.
    fn fake_backend(routes: Routes) -> String {
        use std::io::{BufRead as _, BufReader};
        let l = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = l.local_addr().expect("addr");
        std::thread::spawn(move || {
            for conn in l.incoming() {
                let Ok(mut conn) = conn else { continue };
                let mut reader = BufReader::new(conn.try_clone().expect("clone"));
                let mut first = String::new();
                if reader.read_line(&mut first).is_err() {
                    continue;
                }
                let mut h = String::new();
                while reader.read_line(&mut h).is_ok_and(|n| n > 2) {
                    h.clear();
                }
                let path = first.split_whitespace().nth(1).unwrap_or("").to_owned();
                let body = routes.lock().expect("lock").get(&path).cloned();
                let (status, body) = body.map_or(("404 Not Found", String::new()), |b| ("200 OK", b));
                let _ = write!(
                    conn,
                    "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
        });
        format!("http://{addr}")
    }

    /// Serve an index of one group with `stories` (id, lines of dialogue).
    fn serve(routes: &Routes, stories: &[(&str, &[&str])]) {
        let mut m = routes.lock().expect("lock");
        m.clear();
        let entries: Vec<serde_json::Value> = stories
            .iter()
            .enumerate()
            .map(|(i, (id, _))| {
                serde_json::json!({"id": id, "name": id, "sort": i, "groupId": "g",
                    "hasScript": true, "wordCount": 0})
            })
            .collect();
        let index = serde_json::json!({
            "groups": [{"id": "g", "name": "g", "startTime": 1, "stories": entries}],
            "totals": {"stories": stories.len(), "withScript": stories.len(), "words": 0},
        });
        m.insert("/api/story/index".into(), index.to_string());
        for (id, said) in stories {
            let commands: Vec<serde_json::Value> = said
                .iter()
                .enumerate()
                .map(|(i, t)| serde_json::json!({"kind": "name", "args": {"name": "Amiya"},
                    "text": t, "line": i + 1}))
                .collect();
            let words: usize = said.iter().map(|t| t.split_whitespace().count()).sum();
            let script = serde_json::json!({"id": id, "commands": commands, "wordCount": words});
            m.insert(format!("/api/story/{id}"), script.to_string());
        }
    }

    fn test_cfg(base: &str, out: &Path, refresh: bool) -> IngestConfig {
        IngestConfig {
            base: base.into(),
            out: out.to_path_buf(),
            concurrency: 2,
            limit: None,
            service_key: None,
            chunk: ChunkConfig::default(),
            tokenizer_sha: "t".into(),
            refresh,
        }
    }

    fn words() -> Arc<dyn Fn(&str) -> u32 + Send + Sync> {
        Arc::new(|s: &str| u32::try_from(s.split_whitespace().count()).unwrap_or(0))
    }

    fn scratch(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("trevor-refresh-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn refresh_matches_a_fresh_build_and_leaves_resume_alone() {
        let routes: Routes = Arc::default();
        let base = fake_backend(Arc::clone(&routes));
        let v1: &[(&str, &[&str])] = &[
            ("s1", &["Doctor, are you awake?", "We land at dawn."]),
            ("s2", &["Kal'tsit is waiting."]),
            ("s3", &["This story will be cut."]),
        ];
        serve(&routes, v1);
        let dir = scratch("main");
        let m = run(&test_cfg(&base, &dir, false), words()).await.expect("build");
        assert!(m.clean() && m.refresh.is_none());
        let built = std::fs::read(dir.join("chunks.jsonl")).expect("read");

        // Unchanged data: the file is untouched and every story unchanged.
        let m = run(&test_cfg(&base, &dir, true), words()).await.expect("refresh");
        let c = m.refresh.expect("counts");
        assert_eq!((c.unchanged, c.added, c.changed, c.removed), (3, 0, 0, 0));
        assert_eq!(std::fs::read(dir.join("chunks.jsonl")).expect("read"), built);
        assert!(!dir.join("chunks.prev.jsonl").exists());

        // An asset update: s2 edited, s3 gone, s4 new. Resume misses the
        // edit (the reason refresh exists) and keeps the file as it was.
        let v2: &[(&str, &[&str])] = &[
            ("s1", &["Doctor, are you awake?", "We land at dawn."]),
            ("s2", &["Kal'tsit is still waiting."]),
            ("s4", &["A new story."]),
        ];
        serve(&routes, v2);
        let resumed = scratch("resume");
        std::fs::create_dir_all(&resumed).expect("mkdir");
        std::fs::write(resumed.join("chunks.jsonl"), &built).expect("w");
        run(&test_cfg(&base, &resumed, false), words()).await.expect("resume");
        let r = std::fs::read_to_string(resumed.join("chunks.jsonl")).expect("read");
        assert!(r.contains("Kal'tsit is waiting.") && !r.contains("still waiting"));

        let m = run(&test_cfg(&base, &dir, true), words()).await.expect("refresh");
        let c = m.refresh.expect("counts");
        assert_eq!((c.unchanged, c.added, c.changed, c.removed), (1, 1, 1, 1));
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dir.join("changes.json")).expect("changes"))
                .expect("json");
        assert_eq!(report["changed"], serde_json::json!(["s2"]));
        assert_eq!(report["added"], serde_json::json!(["s4"]));
        assert_eq!(report["removed"], serde_json::json!(["s3"]));
        assert_eq!(std::fs::read(dir.join("chunks.prev.jsonl")).expect("prev"), built);

        // The refreshed file is byte for byte a from-scratch build of v2.
        let fresh = scratch("fresh");
        run(&test_cfg(&base, &fresh, false), words()).await.expect("fresh");
        let refreshed = std::fs::read(dir.join("chunks.jsonl")).expect("read");
        assert_eq!(refreshed, std::fs::read(fresh.join("chunks.jsonl")).expect("read"));
        assert_eq!(report["newChunksSha"], serde_json::json!(sha_hex(&refreshed)));

        // Refresh with --limit would drop the rest of the corpus: refused.
        let mut limited = test_cfg(&base, &dir, true);
        limited.limit = Some(1);
        assert!(run(&limited, words()).await.is_err());
        for d in [dir, resumed, fresh] {
            let _ = std::fs::remove_dir_all(d);
        }
    }

    fn cmd(kind: &str, args: &[(&str, &str)], text: Option<&str>, line: u32) -> StoryCommand {
        StoryCommand {
            kind: kind.into(),
            args: args
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
            text: text.map(Into::into),
            line,
        }
    }

    #[test]
    fn reconstruction_catches_loss_and_accepts_multiword_speakers() {
        let cmds = vec![
            cmd(
                "name",
                &[("name", "Rhodes Island Operator")],
                Some("Doctor: stand by."),
                1,
            ),
            cmd("name", &[("name", "Amiya")], Some("Understood."), 2),
        ];
        let words = |s: &str| u32::try_from(s.split_whitespace().count()).unwrap_or(0);
        let chunks = chunk_story("s", "g", &cmds, &ChunkConfig::default(), &words);
        assert!(reconstructs(&cmds, &chunks));

        let mut lossy = chunks.clone();
        lossy[0].text = lossy[0].text.replace("stand by", "stand");
        assert!(!reconstructs(&cmds, &lossy));
    }
}
