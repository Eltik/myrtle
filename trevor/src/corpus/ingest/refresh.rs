//! Refresh: rebuild against an existing `chunks.jsonl`, keeping unchanged stories and recording what changed.

use std::collections::{HashMap, HashSet};

use anyhow::{Context, Result};
use serde::Serialize;

use super::resume::StoryIdOnly;

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
