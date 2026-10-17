//! Jobs: the stories to fetch, from the library index, in a fixed sorted order.

use std::collections::HashSet;

use serde::Serialize;

use super::{OperatorRecordGroup, StoryEntry, StoryGroup, StoryIndex};

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
