//! The spoiler horizon: which chunks a reader who has read up to one story may see.
//!
//! The release order is the reading guide's (`artifacts/chrono/reading_guide.json`, one position per story group in
//! EN release order, 85 groups) and the story table's (`artifacts/spoiler.jsonl`, each story's group and `storySort`
//! within it, 1,862 stories). A horizon is a story id (that story and everything released before it, including the
//! earlier stories of its own group) or a group id (the whole group). A chunk is visible when its group sits before
//! the horizon's group, or in it at or before the horizon story. Chunks of groups the reading guide does not place
//! are dated by `artifacts/chrono/source_dates.json` (scripts/source_dates.py, 2026-10-07: operator files and voice
//! lines by the operator's earliest record or module date, records by their set's date, modules, outfits, game text
//! naming a placed group; each an upper bound on the EN release) and visible when that date is at or before the
//! horizon's date, the latest guide date at or before the horizon's position. Undated sources (enemies, items, IS,
//! profiles, summaries, topics, 23 operators) stay hidden: the filter is strict, never permissive.
//! `TREVOR_HORIZON_DATES=0` (kill switch) ignores the source dates, so every unplaced chunk is hidden as before.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

/// The EN release order of story groups and the order of stories within each group.
#[derive(Debug, Default)]
pub struct ReleaseOrder {
    group_pos: HashMap<String, u32>,
    /// story id -> (group id, storySort)
    story: HashMap<String, (String, i64)>,
    /// guide position -> the latest guide release date (unix s) at or before it; absent before the first dated group.
    date_at: HashMap<u32, i64>,
    /// unplaced source story id -> its EN release date (unix s), an upper bound.
    source_date: HashMap<String, i64>,
}

impl ReleaseOrder {
    /// Read the reading guide and the story table under `root`.
    ///
    /// # Errors
    /// Either file missing or unreadable.
    pub fn load(root: &Path) -> Result<Self> {
        let guide: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join("artifacts/chrono/reading_guide.json"))
            .context("reading artifacts/chrono/reading_guide.json")?)?;
        let stories = std::fs::read_to_string(root.join("artifacts/spoiler.jsonl")).context("reading artifacts/spoiler.jsonl")?;
        let rows: Vec<serde_json::Value> = stories.lines().filter(|l| !l.trim().is_empty())
            .map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
        let mut order = Self::from_parts(&guide, &rows);
        let dates = root.join("artifacts/chrono/source_dates.json");
        if std::env::var("TREVOR_HORIZON_DATES").as_deref() != Ok("0") && dates.exists() {
            let d: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&dates)
                .context("reading artifacts/chrono/source_dates.json")?)?;
            order.set_source_dates(&d["dates"]);
        }
        Ok(order)
    }

    /// Date the unplaced sources from `{storyId: unix seconds}`.
    pub fn set_source_dates(&mut self, dates: &serde_json::Value) {
        self.source_date = dates.as_object().into_iter().flatten()
            .filter_map(|(k, v)| Some((k.clone(), v.as_i64()?))).collect();
    }

    /// How many unplaced sources carry a date.
    #[must_use]
    pub fn dated_sources(&self) -> usize {
        self.source_date.len()
    }

    /// The order from the parsed guide (`{"items": [{"groupId", "position"}]}`) and story rows
    /// (`{"storyId", "groupId", "storySort"}`).
    #[must_use]
    pub fn from_parts(guide: &serde_json::Value, stories: &[serde_json::Value]) -> Self {
        let group_pos = guide["items"].as_array().into_iter().flatten()
            .filter_map(|i| Some((i["groupId"].as_str()?.to_owned(), u32::try_from(i["position"].as_u64()?).ok()?)))
            .collect();
        let story = stories.iter()
            .filter_map(|s| Some((s["storyId"].as_str()?.to_owned(), (s["groupId"].as_str()?.to_owned(), s["storySort"].as_i64()?))))
            .collect();
        // The guide's "released" is "2020-01-22", "by 2020-01-22", "about 2020-07-01" or undated text; a position
        // takes the latest date at or before it, so an undated group never admits a source dated after its successors.
        let mut dated: Vec<(u32, Option<i64>)> = guide["items"].as_array().into_iter().flatten()
            .filter_map(|i| Some((u32::try_from(i["position"].as_u64()?).ok()?, i["released"].as_str().and_then(parse_day))))
            .collect();
        dated.sort_by_key(|(p, _)| *p);
        let mut date_at = HashMap::new();
        let mut latest: Option<i64> = None;
        for (p, d) in dated {
            latest = match (latest, d) { (Some(a), Some(b)) => Some(a.max(b)), (a, b) => a.or(b) };
            if let Some(l) = latest {
                date_at.insert(p, l);
            }
        }
        Self { group_pos, story, date_at, source_date: HashMap::new() }
    }

    /// The horizon at a story id or a group id.
    ///
    /// # Errors
    /// The id names no story or group the reading guide places.
    pub fn horizon(self: &Arc<Self>, id: &str) -> Result<Horizon> {
        let (group, sort) = match self.story.get(id) {
            Some((g, s)) => (g.as_str(), Some(*s)),
            None => (id, None),
        };
        let pos = *self.group_pos.get(group)
            .with_context(|| format!("horizon {id:?} is not a story or group in the EN release order"))?;
        Ok(Horizon { id: id.to_owned(), pos, sort, date: self.date_at.get(&pos).copied(), order: Arc::clone(self) })
    }
}

/// Unix seconds (UTC midnight) of the first `YYYY-MM-DD` in `s`.
fn parse_day(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(9)).find_map(|i| {
        let w = &s[i..i + 10];
        let ok = w.bytes().enumerate().all(|(j, c)| if j == 4 || j == 7 { c == b'-' } else { c.is_ascii_digit() });
        if !ok {
            return None;
        }
        let (y, m, d): (i64, i64, i64) = (w[..4].parse().ok()?, w[5..7].parse().ok()?, w[8..].parse().ok()?);
        // Days from civil (Howard Hinnant's algorithm).
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        Some((era * 146_097 + doe - 719_468) * 86_400)
    })
}

/// A reader's position: what they may see.
#[derive(Debug, Clone)]
pub struct Horizon {
    /// The story or group id it was built from.
    pub id: String,
    pos: u32,
    /// The horizon story's `storySort`; None for a whole group.
    sort: Option<i64>,
    /// The latest guide date at or before the horizon's position; unplaced sources dated at or before it are visible.
    date: Option<i64>,
    order: Arc<ReleaseOrder>,
}

impl Horizon {
    /// Whether a chunk of `story_id` in `group_id` was released at or before the horizon.
    #[must_use]
    pub fn allows(&self, story_id: &str, group_id: &str) -> bool {
        match self.order.group_pos.get(group_id) {
            None => match (self.date, self.order.source_date.get(story_id)) {
                (Some(h), Some(&d)) => d <= h,
                _ => false,
            },
            Some(&p) if p < self.pos => true,
            Some(&p) if p > self.pos => false,
            Some(_) => match self.sort {
                None => true,
                Some(h) => self.order.story.get(story_id).is_some_and(|(_, s)| *s <= h),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn order() -> Arc<ReleaseOrder> {
        let guide = json!({"items": [{"groupId": "main_0", "position": 1, "released": "by 2020-01-22"},
                                      {"groupId": "act1", "position": 2, "released": "not dated in the game data"},
                                      {"groupId": "main_1", "position": 3, "released": "about 2020-07-01"}]});
        let stories = vec![
            json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
            json!({"storyId": "a1_a", "groupId": "act1", "storySort": 1}),
            json!({"storyId": "a1_b", "groupId": "act1", "storySort": 2}),
            json!({"storyId": "a1_c", "groupId": "act1", "storySort": 3}),
            json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1}),
        ];
        Arc::new(ReleaseOrder::from_parts(&guide, &stories))
    }

    #[test]
    fn story_horizon_keeps_earlier_groups_and_earlier_stories_of_its_own() {
        let h = order().horizon("a1_b").unwrap();
        assert!(h.allows("m0_a", "main_0"));
        assert!(h.allows("a1_a", "act1"));
        assert!(h.allows("a1_b", "act1"));
        assert!(!h.allows("a1_c", "act1"));
        assert!(!h.allows("m1_a", "main_1"));
        // Unplaced groups (operator files, voice lines, generated summaries) are hidden, never let through.
        assert!(!h.allows("archive_char_002_amiya", "archive"));
        assert!(!h.allows("unknown_story", "act1"));
    }

    #[test]
    fn dated_sources_follow_the_horizon_date() {
        let mut o = ReleaseOrder::from_parts(&json!({"items": [
            {"groupId": "main_0", "position": 1, "released": "by 2020-01-22"},
            {"groupId": "act1", "position": 2, "released": "not dated in the game data"},
            {"groupId": "main_1", "position": 3, "released": "about 2020-07-01"}]}), &[]);
        assert_eq!(parse_day("by 2020-01-22"), Some(1_579_651_200));
        assert_eq!(parse_day("not dated"), None);
        o.set_source_dates(&json!({"archive_char_002_amiya": 1_579_651_200, "voice_char_1": 1_579_651_201}));
        let o = Arc::new(o);
        // main_0's date admits Amiya's file (same day) but not a source one second later.
        let h = o.horizon("main_0").unwrap();
        assert!(h.allows("archive_char_002_amiya", "archive"));
        assert!(!h.allows("voice_char_1", "voice"));
        // An undated group takes the latest earlier date: the same as main_0.
        assert!(!o.horizon("act1").unwrap().allows("voice_char_1", "voice"));
        assert!(o.horizon("main_1").unwrap().allows("voice_char_1", "voice"));
        // Undated sources stay hidden at any horizon.
        assert!(!o.horizon("main_1").unwrap().allows("enemy_enemy_1007_slime", "enemy"));
    }

    #[test]
    fn group_horizon_keeps_the_whole_group() {
        let h = order().horizon("act1").unwrap();
        assert!(h.allows("a1_c", "act1"));
        assert!(!h.allows("m1_a", "main_1"));
        assert!(order().horizon("nope").is_err());
    }
}
