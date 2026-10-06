//! The gold set: questions anchored on line spans, resolved to chunk rows.
//!
//! Anchors are `(story_id, line_start, line_end)`, never chunk ids, so a gold
//! set survives rechunking: a chunk is gold for an item iff its line span
//! overlaps one of the item's anchors.

use std::collections::{BTreeSet, HashSet};
use std::io::BufRead as _;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::search::store::ChunkStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stratum {
    SingleFact,
    MultiHop,
    Entity,
    Temporal,
    Causal,
    Aggregation,
    Unanswerable,
    Ambiguous,
    Spoiler,
}

impl Stratum {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::SingleFact => "single_fact",
            Self::MultiHop => "multi_hop",
            Self::Entity => "entity",
            Self::Temporal => "temporal",
            Self::Causal => "causal",
            Self::Aggregation => "aggregation",
            Self::Unanswerable => "unanswerable",
            Self::Ambiguous => "ambiguous",
            Self::Spoiler => "spoiler",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Review {
    Accepted,
    #[default]
    Pending,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub story_id: String,
    pub line_start: u32,
    pub line_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedBook {
    pub model: String,
    pub correct: bool,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoldItem {
    pub qid: String,
    pub question: String,
    #[serde(default)]
    pub anchors: Vec<Anchor>,
    pub stratum: Stratum,
    #[serde(default)]
    pub source_chunk_id: Option<String>,
    #[serde(default)]
    pub generated_by: Option<String>,
    #[serde(default)]
    pub reviewed_by: Option<String>,
    #[serde(default)]
    pub review: Review,
    #[serde(default)]
    pub closed_book: Option<ClosedBook>,
    #[serde(default)]
    pub notes: Option<String>,
}

pub struct GoldSet {
    pub items: Vec<GoldItem>,
    /// sha256 of the file bytes. Runs are only comparable on the same sha.
    pub sha: String,
}

/// # Errors
/// I/O or a line that does not parse.
pub fn load(path: &Path) -> Result<GoldSet> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    let mut items = Vec::new();
    for (i, line) in bytes.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        items.push(
            serde_json::from_str::<GoldItem>(&line)
                .with_context(|| format!("{} line {}", path.display(), i + 1))?,
        );
    }
    Ok(GoldSet { items, sha })
}

/// Rows whose line span overlaps any anchor.
#[must_use]
pub fn resolve(item: &GoldItem, store: &ChunkStore) -> BTreeSet<usize> {
    let mut out = BTreeSet::new();
    for a in &item.anchors {
        for &row in store.story_rows(&a.story_id) {
            let c = &store.chunks[row];
            if c.line_start <= a.line_end && a.line_start <= c.line_end {
                out.insert(row);
            }
        }
    }
    out
}

/// Every problem in the set at once, so one pass fixes a file.
///
/// # Errors
/// A list of every invalid item.
pub fn validate(set: &GoldSet, store: &ChunkStore) -> Result<()> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for it in &set.items {
        if !seen.insert(it.qid.as_str()) {
            problems.push(format!("{}: duplicate qid", it.qid));
        }
        if it.question.trim().is_empty() {
            problems.push(format!("{}: empty question", it.qid));
        }
        match (it.stratum == Stratum::Unanswerable, it.anchors.is_empty()) {
            (true, false) => {
                problems.push(format!("{}: unanswerable items take no anchors", it.qid))
            }
            (false, true) => problems.push(format!("{}: no anchors", it.qid)),
            _ => {}
        }
        for a in &it.anchors {
            if a.line_start > a.line_end {
                problems.push(format!(
                    "{}: anchor {} {}>{}",
                    it.qid, a.story_id, a.line_start, a.line_end
                ));
            } else if store.story_rows(&a.story_id).is_empty() {
                problems.push(format!("{}: story {} has no chunks", it.qid, a.story_id));
            }
        }
        if !it.anchors.is_empty() && resolve(it, store).is_empty() {
            problems.push(format!("{}: anchors overlap no chunk", it.qid));
        }
        if let Some(src) = &it.source_chunk_id {
            if store.row(src).is_none() {
                // Not fatal to scoring (anchors are authoritative), but it
                // means the item was written against a different chunking.
                problems.push(format!(
                    "{}: source_chunk_id {src} is not in the store",
                    it.qid
                ));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        bail!(
            "{} problem(s) in the gold set:\n  {}",
            problems.len(),
            problems.join("\n  ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::chunk::Chunk;

    fn store() -> ChunkStore {
        let c = |id: &str, ord: u32, a: u32, b: u32| Chunk {
            chunk_id: id.to_owned(),
            story_id: "s".to_owned(),
            group_id: "g".to_owned(),
            ordinal: ord,
            scene_ordinal: 0,
            line_start: a,
            line_end: b,
            text: String::new(),
            speakers: vec![],
            on_screen: vec![],
            background: None,
            token_count: 1,
            content_sha: String::new(),
            prefix: None,
        };
        ChunkStore::from_chunks(
            vec![c("s#0", 0, 1, 10), c("s#1", 1, 12, 20), c("s#2", 2, 22, 30)],
            "x".into(),
        )
        .expect("store")
    }

    fn item(anchors: Vec<(u32, u32)>, stratum: Stratum) -> GoldItem {
        GoldItem {
            qid: "q".into(),
            question: "?".into(),
            anchors: anchors
                .into_iter()
                .map(|(a, b)| Anchor {
                    story_id: "s".into(),
                    line_start: a,
                    line_end: b,
                })
                .collect(),
            stratum,
            source_chunk_id: None,
            generated_by: None,
            reviewed_by: None,
            review: Review::Accepted,
            closed_book: None,
            notes: None,
        }
    }

    #[test]
    fn overlap_is_inclusive_and_can_span_chunks() {
        let s = store();
        assert_eq!(
            resolve(&item(vec![(10, 12)], Stratum::SingleFact), &s),
            [0, 1].into()
        );
        assert_eq!(
            resolve(&item(vec![(11, 11)], Stratum::SingleFact), &s),
            BTreeSet::new(),
            "gap line"
        );
        assert_eq!(
            resolve(&item(vec![(25, 25)], Stratum::SingleFact), &s),
            [2].into()
        );
    }

    #[test]
    fn validation_reports_every_problem() {
        let s = store();
        let set = GoldSet {
            items: vec![
                item(vec![], Stratum::Causal),
                item(vec![(1, 2)], Stratum::Unanswerable),
                item(vec![(11, 11)], Stratum::SingleFact),
            ],
            sha: String::new(),
        };
        let err = validate(&set, &s).expect_err("invalid").to_string();
        assert!(
            err.contains("no anchors")
                && err.contains("take no anchors")
                && err.contains("overlap no chunk")
        );
        assert!(err.contains("duplicate qid"));
    }
}
