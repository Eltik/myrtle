//! What the GAME itself says this account has read, out of a raw `syncData`
//! payload.
//!
//! Pure over its inputs: JSON in, a [`GameStoryReadSet`] out, no database and
//! no clock. That is what lets [`super::store::reverdict`] run the same rule
//! again over stored rows, with no call to the game server.
//!
//! THREE SOURCES, and not one of them alone is the answer:
//!
//! - `user.status.flags`, the client's own played-script record, keyed by
//!   `story_review_table` `InfoUnlockDatas[].StoryTxt`. It is the half that
//!   covers the mainline, and it never carries a story-only stage
//!   (`main_08_st_01`, `spst_08-02`), which the game plays as a STAGE: 42 of
//!   those are invisible to it.
//! - `user.storyreview.groups.<groupId>`, the Archive block. PRESENCE is the
//!   signal: `rc` is a re-read counter, 0 on 1,036 of the measured account's
//!   1,038 rows, so gating on `rc > 0` yielded two stories out of a library the
//!   account has most of. No `main_*` group appears in it at all. And what it
//!   lists is what an event UNLOCKED, not what was opened: 94 of its gated
//!   stories had never been read.
//! - `user.dungeon.stages` read against `RequiredStages`, which settles both
//!   gaps. Satisfied gates mean the game played the story for the player; see
//!   [`stage_satisfies`] for why a story-only stage at state 3 with zero starts
//!   is an opening.
//!
//! THE VERDICT: played or cleared is read, and Archive presence counts only on
//! a story with NO gate (operator records, `USE_ITEM` mini vignettes), where it
//! is the only signal there is. On the measured EN account that reads 1,313 of
//! the 1,407 rows the three sources name.
//!
//! The payload census those numbers come from, and the two refutations that got
//! the rule here, are in `docs/story-reader.md`, section "4. Measured", under
//! "Progress sync from the GAME: two sources, measured against a real payload"
//! and "The game's read verdict, corrected: stage gates, a v2 document, a
//! DB-only re-derivation".
//!
//! Scale: the payload is 2.4 MB of JSON on the measured account and every walk
//! here is linear in it, so callers run this on the blocking pool under
//! `cpu::offload`, never on an async worker.

use serde_json::Value;

use crate::core::gamedata::types::story_review::StoryRequiredStage;

/// One story the game says this account has opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameStoryRead {
    pub story_id: String,
    /// The Archive's re-read counter (`rc`). 0 for almost every story, and
    /// never a condition for being read.
    pub reread_count: i32,
    /// Unix seconds the Archive recorded as the unlock time (`uts`), when it
    /// carries one. Null for a story known only from `status.flags`.
    pub unlocked_at: Option<i64>,
    /// `status.flags` carries this story's script path: the client played it.
    pub played: bool,
    /// The Archive block lists it: the Archive unlocked it.
    pub archived: bool,
    /// Every stage gate on it is satisfied by `dungeon.stages`: the game
    /// played it for the player.
    pub cleared: bool,
    /// The verdict the library shows: played, or cleared, or listed by the
    /// Archive on a story that has no gate.
    pub read: bool,
}

/// Both sources as parsed, plus the census the refresh logs.
#[derive(Debug, Clone, Default)]
pub struct GameStoryReadSet {
    /// Whether the payload said anything about reading at all.
    ///
    /// This is NOT `read.is_empty()`, and the difference decides whether a
    /// stored set is replaced or left alone: a payload that carries neither
    /// source says nothing about what was read, and must not erase what an
    /// earlier refresh imported.
    pub present: bool,
    /// Which top-level key the Archive block was found under, or, when none
    /// was, the key whose subtree carries a story-shaped id.
    pub key: Option<String>,
    /// Archive groups and the stories listed under them.
    pub groups: usize,
    pub stories: usize,
    /// The union of both sources, by story id.
    pub read: Vec<GameStoryRead>,
    /// One story id, for the log line that pins the id namespace.
    pub sample: Option<String>,
    /// Truthy keys in `status.flags`.
    pub flags: usize,
    /// Of those, how many are a script path the story index knows.
    pub flag_hits: usize,
    /// Of those, how many map to no story at all. Guide and tutorial paths
    /// are expected here.
    pub flag_misses: usize,
    /// Three of the misses, so the log line shows what they look like.
    pub flag_miss_sample: Vec<String>,
    /// Archive story ids the story index also knows.
    pub review_hits: usize,
    /// Stage records in `dungeon.stages`.
    pub stage_records: usize,
    /// Gated stories whose every gate those records satisfy.
    pub stage_hits: usize,
    /// Gated stories the Archive lists that neither the flags nor the gates
    /// reach: unlocked, never opened, and NOT read.
    pub archive_only_unread: usize,
}

impl GameStoryReadSet {
    /// Stories present in the Archive block, which is the smaller half of the
    /// union and the number the reader's progress line quotes in brackets.
    #[must_use]
    pub fn archived(&self) -> usize {
        self.read.iter().filter(|r| r.archived && r.read).count()
    }

    /// Stories the verdict marks read, which is what the library receives.
    #[must_use]
    pub fn read_count(&self) -> usize {
        self.read.iter().filter(|r| r.read).count()
    }
}

/// The block names to probe, in order.
const STORY_BLOCK_KEYS: [&str; 4] = ["storyreview", "storyReview", "story_review", "story"];

/// Read an integer that may arrive as a number, a numeric string or a bool.
fn loose_i64(entry: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<i64> {
    for key in keys {
        match entry.get(*key) {
            Some(Value::Number(n)) => return n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
            Some(Value::String(s)) => {
                if let Ok(n) = s.parse::<i64>() {
                    return Some(n);
                }
            }
            Some(Value::Bool(b)) => return Some(i64::from(*b)),
            _ => {}
        }
    }
    None
}

fn loose_str(entry: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| entry.get(*k))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// Whether a flag value means the flag is set. Every value in the measured
/// payload is the integer 1; the other spellings cost one match each.
fn truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        _ => false,
    }
}

/// Whether a string looks like an Archives story id, e.g.
/// `main_0_level_main_00-01_beg` or `1stact_level_a001_01_beg`.
fn story_shaped(s: &str) -> bool {
    s.contains("_level_") && s.len() > 12
}

/// The top-level `user` key whose subtree carries a story-shaped id.
///
/// Only runs when no named block was found, and only to a bounded depth. It is
/// instrumentation for one log line over a 2.4 MB payload, so the refresh runs
/// it on the blocking pool with the rest of the parse, never on an async
/// worker.
fn probe_story_shaped_key(user: &serde_json::Map<String, Value>) -> Option<String> {
    fn carries(v: &Value, depth: u32) -> bool {
        if depth == 0 {
            return false;
        }
        match v {
            Value::String(s) => story_shaped(s),
            Value::Array(items) => items.iter().take(64).any(|i| carries(i, depth - 1)),
            Value::Object(map) => map
                .iter()
                .take(256)
                .any(|(k, val)| story_shaped(k) || carries(val, depth - 1)),
            _ => false,
        }
    }
    user.iter()
        .find(|(_, v)| carries(v, 5))
        .map(|(k, _)| k.clone())
}

/// Pull the stories the game says the account has read out of a raw `syncData`
/// payload, from both sources, and union them.
///
/// `by_txt` is the story index's script path -> story id map
/// (`story::StoryIndexCache::by_txt`), per server. An EMPTY map is legal and
/// means only that the index could not be reached: the Archive half still
/// parses and the census reports `flag_hits` 0, which is how that degradation
/// is visible.
///
/// Never fails: an absent or unexpected block yields an empty set.
#[must_use]
pub fn parse_game_story_read(
    raw: &Value,
    by_txt: &std::collections::HashMap<String, String>,
    gates: &std::collections::HashMap<String, Vec<StoryRequiredStage>>,
) -> GameStoryReadSet {
    use std::collections::BTreeMap;

    let mut set = GameStoryReadSet::default();
    let Some(user) = raw.get("user").and_then(Value::as_object) else {
        return set;
    };
    let mut merged: BTreeMap<String, GameStoryRead> = BTreeMap::new();

    // Source (a): the client's played-script flags, mapped through the index.
    if let Some(flags) = user
        .get("status")
        .and_then(|s| s.get("flags"))
        .and_then(Value::as_object)
    {
        for (path, value) in flags {
            if !truthy(value) {
                continue;
            }
            set.flags += 1;
            let Some(story_id) = by_txt.get(path) else {
                set.flag_misses += 1;
                if set.flag_miss_sample.len() < 3 {
                    set.flag_miss_sample.push(path.clone());
                }
                continue;
            };
            set.flag_hits += 1;
            merged
                .entry(story_id.clone())
                .or_insert_with(|| GameStoryRead {
                    story_id: story_id.clone(),
                    reread_count: 0,
                    unlocked_at: None,
                    played: false,
                    archived: false,
                    cleared: false,
                    read: false,
                })
                .played = true;
        }
    }

    // Source (b): the Archive block. Presence is the signal; `rc` is a
    // re-read counter and gates nothing.
    if let Some((key, block)) = STORY_BLOCK_KEYS
        .iter()
        .find_map(|k| user.get(*k).map(|v| ((*k).to_owned(), v)))
    {
        set.present = true;
        set.key = Some(key);
        // The groups sit under `groups` where the client wraps them, and are
        // the block itself where it does not.
        if let Some(groups) = block.get("groups").unwrap_or(block).as_object() {
            for group in groups.values() {
                set.groups += 1;
                let Some(group) = group.as_object() else {
                    continue;
                };
                // `stories` is an array in the measured payload, but a client
                // that keys it by story id is the same data and costs one
                // match to accept.
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
                    let entry = merged
                        .entry(story_id.clone())
                        .or_insert_with(|| GameStoryRead {
                            story_id: story_id.clone(),
                            reread_count: 0,
                            unlocked_at: None,
                            played: false,
                            archived: false,
                            cleared: false,
                            read: false,
                        });
                    entry.archived = true;
                    entry.reread_count = i32::try_from(
                        loose_i64(story, &["rc", "readCount", "read_count"])
                            .unwrap_or(0)
                            .max(0),
                    )
                    .unwrap_or(i32::MAX);
                    entry.unlocked_at =
                        loose_i64(story, &["uts", "ts", "readTs"]).filter(|t| *t > 0);
                }
            }
        }
    } else {
        set.key = probe_story_shaped_key(user);
    }

    // Source (c): the stage gates. `story_review_table` puts `RequiredStages`
    // on every mainline and event story, and the client's `dungeon.stages`
    // records per stage `state` (0 locked, 1 unlocked, 2 pass, 3 complete),
    // `startTimes` and `completeTimes`; `dungeon.cowLevel` keeps the first
    // open of a special story stage. Satisfied gates mean the game played the
    // story for the player, skipped or not; see `stage_satisfies` for what
    // counts as an entry and why `state` alone does not.
    let cow_level = user
        .get("dungeon")
        .and_then(|d| d.get("cowLevel"))
        .and_then(Value::as_object);
    if let Some(stages) = user
        .get("dungeon")
        .and_then(|d| d.get("stages"))
        .and_then(Value::as_object)
    {
        set.stage_records = stages.len();
        for (story_id, required) in gates {
            if required.is_empty()
                || !required.iter().all(|g| {
                    stage_satisfies(
                        stages.get(&g.stage_id),
                        cow_level.and_then(|c| c.get(&g.stage_id)),
                        &g.min_state,
                    )
                })
            {
                continue;
            }
            set.stage_hits += 1;
            merged
                .entry(story_id.clone())
                .or_insert_with(|| GameStoryRead {
                    story_id: story_id.clone(),
                    reread_count: 0,
                    unlocked_at: None,
                    played: false,
                    archived: false,
                    cleared: false,
                    read: false,
                })
                .cleared = true;
        }
    }

    // THE VERDICT. Played or cleared is read. Archive presence is read only
    // on a story with no gate: on a gated one the Archive lists what the
    // event UNLOCKED, 94 never-opened stories on the measured account.
    for r in merged.values_mut() {
        let gated = gates.get(&r.story_id).is_some_and(|g| !g.is_empty());
        r.read = r.played || r.cleared || (r.archived && !gated);
        if r.archived && gated && !r.read {
            set.archive_only_unread += 1;
        }
    }

    // A payload with flags or stage records but no Archive block still said
    // something.
    set.present = set.present || set.flag_hits > 0 || set.stage_hits > 0;
    let known: std::collections::HashSet<&str> = by_txt.values().map(String::as_str).collect();
    set.review_hits = merged
        .values()
        .filter(|r| r.archived && known.contains(r.story_id.as_str()))
        .count();
    set.read = merged.into_values().collect();
    set.sample = set.read.first().map(|r| r.story_id.clone());
    set
}

/// Whether one `dungeon.stages` record meets a gate's `MinState`.
///
/// `state` is 0 locked, 1 unlocked, 2 pass, 3 complete. A PASS IS AN ENTRY:
/// a story-only stage (`st_*`, `spst_*`, `act*_st0N`) never counts a start or
/// a completion, so the 159 of them the account has opened all read state 3
/// with `startTimes` 0 and `completeTimes` 0, and the four it has not
/// (`st_16-01`, `act12side_st01`, `act13side_st01`, `act15side_st01`) read
/// state 0. A record moves off 0 only when the player completes the stage,
/// so state 2 or 3 is the opening the counters do not record. The 2026-09-24
/// cut that required a counter above zero marked all 159 unread and was
/// refuted on chapter 7 within the hour; the report that led to it was the
/// baked document, not this rule. `PLAYED` is any of: a start, a completion,
/// state 2 or above, or a `dungeon.cowLevel` first open (`fts` above zero,
/// which the special story stages `spst_*` carry and the refresh copies onto
/// the stored record as `cowFirstTs`); `PASS` is state 2 or above, a
/// completion, or a cowLevel open; `COMPLETE` is state 3; `UNLOCKED` is state
/// 1 or above. An absent record is a stage never unlocked. An unknown word is
/// held to PASS.
pub fn stage_satisfies(record: Option<&Value>, cow: Option<&Value>, min_state: &str) -> bool {
    let record = record.and_then(Value::as_object);
    let num = |k: &str| {
        record
            .and_then(|r| r.get(k))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };
    let cow_opened = cow
        .and_then(Value::as_object)
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

#[cfg(test)]
mod tests {
    use super::*;
    // The round-trip test below rebuilds the refresh's payload from stored rows
    // and runs it back through the rule here, which is the one place a verdict
    // test reaches into the store half.
    use super::super::store::{StoredRead, stored_payload};

    // -----------------------------------------------------------------------
    // The game's own read marks, from BOTH sources.
    //
    // Every payload below is HAND-WRITTEN. The shapes and the field semantics
    // are read off a real EN `account/syncData` dump, but nothing captured from
    // an account is reproduced here: the fragments are the smallest thing that
    // exercises each rule.
    // -----------------------------------------------------------------------

    use std::collections::HashMap;

    /// The two-source form: no gates, which is every test written before the
    /// stage rule and still the shape of an index that failed to build.
    fn parse(raw: &Value, by_txt: &HashMap<String, String>) -> GameStoryReadSet {
        parse_game_story_read(raw, by_txt, &HashMap::new())
    }

    fn gate(stage: &str, min_state: &str) -> Vec<StoryRequiredStage> {
        vec![StoryRequiredStage {
            stage_id: stage.to_owned(),
            min_state: min_state.to_owned(),
            max_state: "COMPLETE".to_owned(),
        }]
    }

    #[test]
    fn a_stage_gate_reads_the_stage_record_and_outranks_archive_presence() {
        let mut gates: HashMap<String, Vec<StoryRequiredStage>> = HashMap::new();
        // Entered once, never cleared: a `_beg`-style PLAYED gate is met.
        gates.insert(
            "main_8_level_st_08-01".into(),
            gate("main_08_st_01", "PLAYED"),
        );
        // Entered three times, never cleared: a PASS gate is not.
        gates.insert(
            "main_8_level_main_08-02_end".into(),
            gate("main_08-02", "PASS"),
        );
        // No stage record at all, but the Archive lists it: unlocked, unread.
        gates.insert("1stact_level_a001_01_end".into(), gate("a001_01", "PASS"));
        // Same missing stage, but the flags say it was played.
        gates.insert("1stact_level_a001_01_beg".into(), gate("a001_01", "PLAYED"));
        // Complete by state with zero counters: the shape of every OPENED story-only stage.
        gates.insert(
            "act3d0_level_act3d0_st01".into(),
            gate("act3d0_st01", "PLAYED"),
        );
        // And the shape of one never opened.
        gates.insert(
            "act12side_level_act12side_st01".into(),
            gate("act12side_st01", "PLAYED"),
        );
        let payload = serde_json::json!({ "user": {
            "status": { "flags": { "activities/a001/level_a001_01_beg": 1 } },
            "storyreview": { "groups": {
                "1stact": { "stories": [ { "id": "1stact_level_a001_01_end", "rc": 0 }, { "id": "1stact_level_a001_01_beg", "rc": 0 } ] },
                "rec": { "stories": [ { "id": "story_kroos_set_1_story_1", "rc": 0 } ] }
            } },
            "dungeon": { "stages": {
                "main_08_st_01": { "state": 1, "startTimes": 1, "completeTimes": 0 },
                "main_08-02": { "state": 1, "startTimes": 3, "completeTimes": 0 },
                "act3d0_st01": { "state": 3, "startTimes": 0, "completeTimes": 0 },
                "act12side_st01": { "state": 0, "startTimes": 0, "completeTimes": 0 }
            } }
        } });
        let set = parse_game_story_read(&payload, &index(), &gates);
        assert_eq!(set.stage_records, 4);
        assert_eq!(set.stage_hits, 2);
        let by_id: HashMap<&str, &GameStoryRead> =
            set.read.iter().map(|r| (r.story_id.as_str(), r)).collect();
        let st = by_id["main_8_level_st_08-01"];
        assert!(st.cleared && st.read && !st.played && !st.archived);
        assert!(!by_id.contains_key("main_8_level_main_08-02_end"));
        assert!(
            by_id["act3d0_level_act3d0_st01"].cleared && by_id["act3d0_level_act3d0_st01"].read
        );
        assert!(!by_id.contains_key("act12side_level_act12side_st01"));
        let unlocked = by_id["1stact_level_a001_01_end"];
        assert!(unlocked.archived && !unlocked.cleared && !unlocked.read);
        assert!(by_id["1stact_level_a001_01_beg"].read);
        assert!(
            by_id["story_kroos_set_1_story_1"].read,
            "an ungated Archive entry is read"
        );
        assert_eq!(set.archive_only_unread, 1);
        assert_eq!(set.archived(), 2);
        assert_eq!(set.read_count(), 4);
        assert_eq!(set.read.len(), 5);
    }

    #[test]
    fn a_stored_set_rebuilds_to_the_refresh_payload_and_the_same_verdict() {
        use crate::app::services::story::StoryRef;
        let lookup: HashMap<String, StoryRef> = [
            (
                "main_0_level_main_00-01_beg",
                "obt/main/level_main_00-01_beg",
            ),
            (
                "1stact_level_a001_01_end",
                "activities/a001/level_a001_01_end",
            ),
        ]
        .into_iter()
        .map(|(id, txt)| {
            (
                id.to_owned(),
                StoryRef {
                    story_txt: txt.to_owned(),
                    name: String::new(),
                    group_id: String::new(),
                },
            )
        })
        .collect();
        let rows = vec![
            StoredRead {
                story_id: "main_0_level_main_00-01_beg".into(),
                reread_count: 0,
                unlocked_at: None,
                played: true,
                archived: false,
            },
            StoredRead {
                story_id: "1stact_level_a001_01_end".into(),
                reread_count: 2,
                unlocked_at: chrono::DateTime::from_timestamp(1_700_000_200, 0),
                played: false,
                archived: true,
            },
            // Known to no index entry: its flag cannot be rebuilt, as a path the index does not know would not map.
            StoredRead {
                story_id: "gone_level_x_beg".into(),
                reread_count: 0,
                unlocked_at: None,
                played: true,
                archived: false,
            },
        ];
        let stages = serde_json::json!({ "main_08_st_01": { "state": 1, "startTimes": 1, "completeTimes": 0 } });
        let raw = stored_payload(&rows, Some(stages), &lookup);
        assert_eq!(
            raw["user"]["status"]["flags"]["obt/main/level_main_00-01_beg"],
            1
        );
        assert_eq!(
            raw["user"]["storyreview"]["groups"]["stored"]["stories"][0]["id"],
            "1stact_level_a001_01_end"
        );
        assert_eq!(
            raw["user"]["storyreview"]["groups"]["stored"]["stories"][0]["uts"],
            1_700_000_200
        );
        let mut gates: HashMap<String, Vec<StoryRequiredStage>> = HashMap::new();
        gates.insert(
            "main_8_level_st_08-01".into(),
            gate("main_08_st_01", "PLAYED"),
        );
        gates.insert("1stact_level_a001_01_end".into(), gate("a001_01", "PASS"));
        let set = parse_game_story_read(&raw, &index(), &gates);
        let by_id: HashMap<&str, &GameStoryRead> =
            set.read.iter().map(|r| (r.story_id.as_str(), r)).collect();
        assert!(
            by_id["main_0_level_main_00-01_beg"].played
                && by_id["main_0_level_main_00-01_beg"].read
        );
        assert!(by_id["main_8_level_st_08-01"].cleared && by_id["main_8_level_st_08-01"].read);
        let unlocked = by_id["1stact_level_a001_01_end"];
        assert!(
            unlocked.archived
                && !unlocked.read
                && unlocked.reread_count == 2
                && unlocked.unlocked_at == Some(1_700_000_200)
        );
        assert!(!by_id.contains_key("gone_level_x_beg"));
        assert_eq!(set.read_count(), 2);
        assert_eq!(set.flag_misses, 0);
    }

    #[test]
    fn stage_state_words() {
        let rec = |state: i64, started: i64, done: i64| serde_json::json!({ "state": state, "startTimes": started, "completeTimes": done });
        let sat = |r: &Value, word: &str| stage_satisfies(Some(r), None, word);
        assert!(sat(&rec(1, 1, 0), "PLAYED"));
        assert!(!sat(&rec(1, 0, 0), "PLAYED"));
        assert!(!sat(&rec(1, 5, 0), "PASS"));
        assert!(sat(&rec(2, 5, 1), "PASS"));
        assert!(!sat(&rec(2, 5, 1), "COMPLETE"));
        assert!(sat(&rec(3, 5, 1), "COMPLETE"));
        assert!(!stage_satisfies(None, None, "PLAYED"));
        assert!(!sat(&serde_json::json!(7), "PLAYED"));
        // A PASS IS AN ENTRY: the shape of every opened story-only stage is
        // state 3 with zero starts and zero completions, and an unopened one
        // is state 0.
        assert!(sat(&rec(3, 0, 0), "PLAYED"));
        assert!(sat(&rec(3, 0, 0), "PASS"));
        assert!(sat(&rec(3, 0, 0), "COMPLETE"));
        assert!(!sat(&rec(0, 0, 0), "PLAYED"));
        assert!(!sat(&rec(0, 0, 0), "UNLOCKED"));
        assert!(sat(&rec(1, 0, 0), "UNLOCKED"));
        // A cowLevel first-open is an entry AND a pass, on the payload's own
        // record or copied onto the stored one.
        let cow = serde_json::json!({ "id": "spst_08-01", "fts": 1_708_906_788 });
        assert!(stage_satisfies(Some(&rec(3, 0, 0)), Some(&cow), "PLAYED"));
        assert!(stage_satisfies(Some(&rec(3, 0, 0)), Some(&cow), "PASS"));
        assert!(stage_satisfies(None, Some(&cow), "PLAYED"));
        assert!(!stage_satisfies(
            Some(&rec(0, 0, 0)),
            Some(&serde_json::json!({ "fts": -1 })),
            "PLAYED"
        ));
        let stored = serde_json::json!({ "state": 3, "startTimes": 0, "completeTimes": 0, "cowFirstTs": 1_708_906_788 });
        assert!(sat(&stored, "PASS"));
        // An unknown word is held to PASS.
        assert!(sat(&rec(2, 0, 0), "WHATEVER"));
        assert!(!sat(&rec(1, 9, 0), "WHATEVER"));
    }

    /// A three-entry script path -> story id table, standing in for the story
    /// index's `by_txt`. One mainline path, one activity path, and nothing for
    /// the guide path the flags below also carry.
    fn index() -> HashMap<String, String> {
        [
            (
                "obt/main/level_main_00-01_beg",
                "main_0_level_main_00-01_beg",
            ),
            (
                "activities/a001/level_a001_01_beg",
                "1stact_level_a001_01_beg",
            ),
            (
                "activities/a001/level_a001_01_end",
                "1stact_level_a001_01_end",
            ),
        ]
        .into_iter()
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect()
    }

    /// Three set flags: two script paths the index knows and one guide path it
    /// does not.
    fn flags_payload() -> Value {
        serde_json::json!({
            "user": { "status": { "flags": {
                "obt/main/level_main_00-01_beg": 1,
                "activities/a001/level_a001_01_beg": 1,
                "obt/guide/l0-0/0_home_ui": 1
            } } }
        })
    }

    /// A two-story Archive group. Both carry `rc` 0, which is what the real
    /// block carries for 1,036 of its 1,038 stories.
    fn archive_payload() -> Value {
        serde_json::json!({
            "user": { "storyreview": { "groups": { "1stact": {
                "rts": 1_700_000_000,
                "stories": [
                    { "id": "1stact_level_a001_01_beg", "uts": 1_700_000_100, "rc": 0 },
                    { "id": "1stact_level_a001_01_end", "uts": 1_700_000_200, "rc": 0 }
                ],
                "trailRewards": []
            } } } }
        })
    }

    #[test]
    fn the_flags_source_maps_script_paths_through_the_index() {
        let set = parse(&flags_payload(), &index());
        assert!(set.present, "flags alone are a source");
        assert_eq!(set.flags, 3);
        assert_eq!(set.flag_hits, 2);
        assert_eq!(set.flag_misses, 1);
        assert_eq!(set.flag_miss_sample, vec!["obt/guide/l0-0/0_home_ui"]);
        let ids: Vec<&str> = set.read.iter().map(|r| r.story_id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["1stact_level_a001_01_beg", "main_0_level_main_00-01_beg"]
        );
        assert!(set.read.iter().all(|r| r.played && !r.archived));
        assert_eq!(set.groups, 0);
    }

    #[test]
    fn a_zero_reread_count_is_still_read() {
        let set = parse(&archive_payload(), &index());
        assert!(set.present);
        assert_eq!(set.key.as_deref(), Some("storyreview"));
        assert_eq!(set.groups, 1);
        assert_eq!(set.stories, 2);
        // The old rule was `rc > 0` and would have yielded NOTHING here.
        assert_eq!(set.read.len(), 2);
        assert!(set.read.iter().all(|r| r.archived && r.reread_count == 0));
        assert_eq!(set.read[0].unlocked_at, Some(1_700_000_100));
        assert_eq!(set.read[1].unlocked_at, Some(1_700_000_200));
        assert_eq!(set.review_hits, 2);
        assert_eq!(set.flags, 0);
    }

    #[test]
    fn the_union_carries_both_sources_and_marks_which_named_it() {
        let mut payload = flags_payload();
        payload["user"]["storyreview"] = archive_payload()["user"]["storyreview"].clone();
        let set = parse(&payload, &index());
        // beg: both. end: Archive only. main_00-01_beg: flags only.
        assert_eq!(set.read.len(), 3);
        assert_eq!(set.flag_hits, 2);
        assert_eq!(set.stories, 2);
        assert_eq!(set.archived(), 2);
        let by_id: std::collections::HashMap<&str, &GameStoryRead> =
            set.read.iter().map(|r| (r.story_id.as_str(), r)).collect();
        let both = by_id["1stact_level_a001_01_beg"];
        assert!(both.played && both.archived);
        assert_eq!(both.unlocked_at, Some(1_700_000_100));
        let archive_only = by_id["1stact_level_a001_01_end"];
        assert!(!archive_only.played && archive_only.archived);
        let flags_only = by_id["main_0_level_main_00-01_beg"];
        assert!(flags_only.played && !flags_only.archived);
        assert_eq!(flags_only.unlocked_at, None);
        assert_eq!(flags_only.reread_count, 0);
    }

    #[test]
    fn an_empty_index_costs_the_flags_half_and_keeps_the_archive_half() {
        let mut payload = flags_payload();
        payload["user"]["storyreview"] = archive_payload()["user"]["storyreview"].clone();
        let set = parse(&payload, &HashMap::new());
        assert_eq!(set.flag_hits, 0);
        assert_eq!(set.flag_misses, 3);
        assert_eq!(set.read.len(), 2);
        assert!(set.present);
    }

    #[test]
    fn neither_source_is_an_empty_set_and_not_present() {
        let set = parse(&serde_json::json!({ "user": { "troop": {} } }), &index());
        assert!(!set.present);
        assert!(set.read.is_empty());
        assert_eq!(set.groups, 0);
        assert!(!parse(&serde_json::json!(null), &index()).present);
        assert!(!parse(&serde_json::json!({}), &index()).present);
    }

    #[test]
    fn accepts_the_other_block_spellings_and_an_unwrapped_group_map() {
        let camel = serde_json::json!({
            "user": { "storyReview": { "groups": { "g": { "stories": [ { "id": "g_level_s_beg" } ] } } } }
        });
        assert_eq!(parse(&camel, &index()).read.len(), 1);
        // No `groups` wrapper: the block IS the map.
        let flat = serde_json::json!({
            "user": { "storyreview": { "g": { "stories": [ { "id": "g_level_s_beg" } ] } } }
        });
        let set = parse(&flat, &index());
        assert_eq!(set.groups, 1);
        assert_eq!(set.read.len(), 1);
    }

    #[test]
    fn tolerates_junk_without_failing() {
        let junk = serde_json::json!({
            "user": {
                "status": { "flags": { "obt/main/level_main_00-01_beg": 0, "activities/a001/level_a001_01_beg": true } },
                "storyreview": { "groups": {
                    "ok":      { "stories": [ { "id": "ok_level_1_beg", "rc": "2", "uts": "1700000000" } ] },
                    "keyed":   { "stories": { "keyed_level_1_beg": { "rc": true } } },
                    "noid":    { "stories": [ { "rc": 5 } ] },
                    "notlist": { "stories": 7 },
                    "scalar":  3,
                    "empty":   {}
                } }
            }
        });
        let set = parse(&junk, &index());
        assert!(set.present);
        assert_eq!(set.groups, 6);
        assert_eq!(set.stories, 2);
        // The 0-valued flag is not set and is not counted at all.
        assert_eq!(set.flags, 1);
        assert_eq!(set.flag_hits, 1);
        let ids: Vec<&str> = set.read.iter().map(|r| r.story_id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "1stact_level_a001_01_beg",
                "keyed_level_1_beg",
                "ok_level_1_beg"
            ]
        );
        let ok = set
            .read
            .iter()
            .find(|r| r.story_id == "ok_level_1_beg")
            .unwrap();
        assert_eq!(ok.reread_count, 2);
        assert_eq!(ok.unlocked_at, Some(1_700_000_000));
    }

    #[test]
    fn a_negative_reread_count_is_clamped_and_the_story_still_counts() {
        let set = parse(
            &serde_json::json!({
                "user": { "storyreview": { "groups": { "g": {
                    "rts": 1, "trailRewards": { "a": 1 }, "somethingNew": [1, 2],
                    "stories": [
                        { "id": "g_level_1_beg", "rc": 0, "uts": 5, "extra": "x" },
                        { "id": "g_level_1_end", "rc": -1 }
                    ]
                } } } }
            }),
            &index(),
        );
        assert_eq!(set.stories, 2);
        assert_eq!(set.read.len(), 2);
        assert!(set.read.iter().all(|r| r.reread_count == 0));
        assert_eq!(set.review_hits, 0);
    }

    #[test]
    fn names_the_key_whose_subtree_carries_a_story_shaped_id() {
        let set = parse(
            &serde_json::json!({
                "user": {
                    "troop": { "chars": { "1": { "charId": "char_002_amiya" } } },
                    "avg": { "groups": { "main_0": { "stories": ["main_0_level_main_00-01_beg"] } } }
                }
            }),
            &index(),
        );
        assert!(!set.present);
        assert_eq!(set.key.as_deref(), Some("avg"));
    }
}
