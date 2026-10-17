//! Resume: what an earlier, interrupted run already wrote.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};


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
pub(super) struct StoryIdOnly {
    pub(super) story_id: String,
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
