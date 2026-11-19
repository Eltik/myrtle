use std::collections::{BTreeMap, HashMap, HashSet};

use myrtle_gamedata_raw::tables::story::StoryRequiredStage;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameStoryRead {
    pub story_id: String,
    pub reread_count: i32,
    pub unlocked_at: Option<i64>,
    pub played: bool,
    pub archived: bool,
    pub cleared: bool,
    pub read: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GameStoryReadSet {
    pub present: bool,
    pub key: Option<String>,
    pub groups: usize,
    pub stories: usize,
    pub read: Vec<GameStoryRead>,
    pub sample: Option<String>,
    pub flags: usize,
    pub flag_hits: usize,
    pub flag_misses: usize,
    pub flag_miss_sample: Vec<String>,
    pub review_hits: usize,
    pub stage_records: usize,
    pub stage_hits: usize,
    pub archive_only_unread: usize,
}

impl GameStoryReadSet {
    #[must_use]
    pub fn archived(&self) -> usize {
        self.read.iter().filter(|r| r.archived && r.read).count()
    }

    #[must_use]
    pub fn read_count(&self) -> usize {
        self.read.iter().filter(|r| r.read).count()
    }
}

const STORY_BLOCK_KEYS: [&str; 4] = ["storyreview", "storyReview", "story_review", "story"];

fn loose_i64(entry: &Map<String, Value>, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| match entry.get(*key)? {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
        Value::String(s) => s.parse().ok(),
        Value::Bool(b) => Some(i64::from(*b)),
        _ => None,
    })
}

fn loose_str(entry: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| entry.get(*k))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        _ => false,
    }
}

fn story_shaped(s: &str) -> bool {
    s.contains("_level_") && s.len() > 12
}

fn carries(v: &Value, depth: u32) -> bool {
    depth > 0
        && match v {
            Value::String(s) => story_shaped(s),
            Value::Array(items) => items.iter().take(64).any(|i| carries(i, depth - 1)),
            Value::Object(map) => map
                .iter()
                .take(256)
                .any(|(k, val)| story_shaped(k) || carries(val, depth - 1)),
            _ => false,
        }
}

fn entry<'a>(merged: &'a mut BTreeMap<String, GameStoryRead>, id: &str) -> &'a mut GameStoryRead {
    merged
        .entry(id.to_owned())
        .or_insert_with(|| GameStoryRead {
            story_id: id.to_owned(),
            ..GameStoryRead::default()
        })
}

#[must_use]
pub fn parse_game_story_read(
    raw: &Value,
    by_txt: &HashMap<String, String>,
    gates: &HashMap<String, Vec<StoryRequiredStage>>,
) -> GameStoryReadSet {
    let mut set = GameStoryReadSet::default();
    let Some(user) = raw.get("user").and_then(Value::as_object) else {
        return set;
    };
    let mut merged: BTreeMap<String, GameStoryRead> = BTreeMap::new();

    if let Some(flags) = user
        .get("status")
        .and_then(|s| s.get("flags"))
        .and_then(Value::as_object)
    {
        for (path, _) in flags.iter().filter(|(_, v)| truthy(v)) {
            set.flags += 1;
            if let Some(story_id) = by_txt.get(path) {
                set.flag_hits += 1;
                entry(&mut merged, story_id).played = true;
            } else {
                set.flag_misses += 1;
                if set.flag_miss_sample.len() < 3 {
                    set.flag_miss_sample.push(path.clone());
                }
            }
        }
    }

    if let Some((key, block)) = STORY_BLOCK_KEYS
        .iter()
        .find_map(|k| user.get(*k).map(|v| (*k, v)))
    {
        set.present = true;
        set.key = Some(key.to_owned());
        for group in block
            .get("groups")
            .unwrap_or(block)
            .as_object()
            .into_iter()
            .flat_map(Map::values)
        {
            set.groups += 1;
            let stories: Vec<(Option<&str>, &Value)> = match group.get("stories") {
                Some(Value::Array(items)) => items.iter().map(|v| (None, v)).collect(),
                Some(Value::Object(map)) => {
                    map.iter().map(|(k, v)| (Some(k.as_str()), v)).collect()
                }
                _ => continue,
            };
            for (keyed_id, story) in stories {
                let Some(story) = story.as_object() else {
                    continue;
                };
                let Some(story_id) = loose_str(story, &["id", "storyId", "story_id"])
                    .or_else(|| keyed_id.map(str::to_owned))
                else {
                    continue;
                };
                set.stories += 1;
                let e = entry(&mut merged, &story_id);
                e.archived = true;
                e.reread_count = i32::try_from(
                    loose_i64(story, &["rc", "readCount", "read_count"])
                        .unwrap_or(0)
                        .max(0),
                )
                .unwrap_or(i32::MAX);
                e.unlocked_at = loose_i64(story, &["uts", "ts", "readTs"]).filter(|t| *t > 0);
            }
        }
    } else {
        set.key = user
            .iter()
            .find(|(_, v)| carries(v, 5))
            .map(|(k, _)| k.clone());
    }

    let dungeon = user.get("dungeon");
    let cow_level = dungeon
        .and_then(|d| d.get("cowLevel"))
        .and_then(Value::as_object);
    if let Some(stages) = dungeon
        .and_then(|d| d.get("stages"))
        .and_then(Value::as_object)
    {
        set.stage_records = stages.len();
        for (story_id, required) in gates {
            let met = !required.is_empty()
                && required.iter().all(|g| {
                    stage_satisfies(
                        stages.get(&g.stage_id),
                        cow_level.and_then(|c| c.get(&g.stage_id)),
                        &g.min_state,
                    )
                });
            if met {
                set.stage_hits += 1;
                entry(&mut merged, story_id).cleared = true;
            }
        }
    }

    for r in merged.values_mut() {
        let gated = gates.get(&r.story_id).is_some_and(|g| !g.is_empty());
        r.read = r.played || r.cleared || (r.archived && !gated);
        if r.archived && gated && !r.read {
            set.archive_only_unread += 1;
        }
    }

    set.present = set.present || set.flag_hits > 0 || set.stage_hits > 0;
    let known: HashSet<&str> = by_txt.values().map(String::as_str).collect();
    set.review_hits = merged
        .values()
        .filter(|r| r.archived && known.contains(r.story_id.as_str()))
        .count();
    set.read = merged.into_values().collect();
    set.sample = set.read.first().map(|r| r.story_id.clone());
    set
}

pub fn stage_satisfies(record: Option<&Value>, cow: Option<&Value>, min_state: &str) -> bool {
    let record = record.and_then(Value::as_object);
    let num = |k: &str| {
        record
            .and_then(|r| r.get(k))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };
    let cow_opened = cow
        .and_then(|c| c.get("fts"))
        .and_then(Value::as_i64)
        .is_some_and(|fts| fts > 0)
        || num("cowFirstTs") > 0;
    let state = num("state");
    let passed = cow_opened || state >= 2 || num("completeTimes") > 0;
    match min_state {
        "PLAYED" => passed || num("startTimes") > 0,
        "UNLOCKED" => passed || state >= 1 || num("startTimes") > 0,
        "COMPLETE" => state >= 3,
        _ => passed,
    }
}
