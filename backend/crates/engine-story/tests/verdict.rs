#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;

use myrtle_engine_story::verdict::{
    GameStoryRead, GameStoryReadSet, parse_game_story_read, stage_satisfies,
};
use myrtle_gamedata_raw::tables::story::StoryRequiredStage;
use serde_json::{Value, json};

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

fn flags_payload() -> Value {
    json!({ "user": { "status": { "flags": {
        "obt/main/level_main_00-01_beg": 1,
        "activities/a001/level_a001_01_beg": 1,
        "obt/guide/l0-0/0_home_ui": 1
    } } } })
}

fn archive_payload() -> Value {
    json!({ "user": { "storyreview": { "groups": { "1stact": {
        "rts": 1_700_000_000,
        "stories": [
            { "id": "1stact_level_a001_01_beg", "uts": 1_700_000_100, "rc": 0 },
            { "id": "1stact_level_a001_01_end", "uts": 1_700_000_200, "rc": 0 }
        ],
        "trailRewards": []
    } } } } })
}

fn by_id(set: &GameStoryReadSet) -> HashMap<&str, &GameStoryRead> {
    set.read.iter().map(|r| (r.story_id.as_str(), r)).collect()
}

#[test]
fn a_stage_gate_reads_the_stage_record_and_outranks_archive_presence() {
    let gates: HashMap<String, Vec<StoryRequiredStage>> = [
        ("main_8_level_st_08-01", gate("main_08_st_01", "PLAYED")),
        ("main_8_level_main_08-02_end", gate("main_08-02", "PASS")),
        ("1stact_level_a001_01_end", gate("a001_01", "PASS")),
        ("1stact_level_a001_01_beg", gate("a001_01", "PLAYED")),
        ("act3d0_level_act3d0_st01", gate("act3d0_st01", "PLAYED")),
        (
            "act12side_level_act12side_st01",
            gate("act12side_st01", "PLAYED"),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    let payload = json!({ "user": {
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
    let by_id = by_id(&set);
    let st = by_id["main_8_level_st_08-01"];
    assert!(st.cleared && st.read && !st.played && !st.archived);
    assert!(!by_id.contains_key("main_8_level_main_08-02_end"));
    assert!(by_id["act3d0_level_act3d0_st01"].cleared && by_id["act3d0_level_act3d0_st01"].read);
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
fn stage_state_words() {
    let rec = |state: i64, started: i64, done: i64| json!({ "state": state, "startTimes": started, "completeTimes": done });
    let sat = |r: &Value, word: &str| stage_satisfies(Some(r), None, word);
    assert!(sat(&rec(1, 1, 0), "PLAYED"));
    assert!(!sat(&rec(1, 0, 0), "PLAYED"));
    assert!(!sat(&rec(1, 5, 0), "PASS"));
    assert!(sat(&rec(2, 5, 1), "PASS"));
    assert!(!sat(&rec(2, 5, 1), "COMPLETE"));
    assert!(sat(&rec(3, 5, 1), "COMPLETE"));
    assert!(!stage_satisfies(None, None, "PLAYED"));
    assert!(!sat(&json!(7), "PLAYED"));
    assert!(sat(&rec(3, 0, 0), "PLAYED"));
    assert!(sat(&rec(3, 0, 0), "PASS"));
    assert!(sat(&rec(3, 0, 0), "COMPLETE"));
    assert!(!sat(&rec(0, 0, 0), "PLAYED"));
    assert!(!sat(&rec(0, 0, 0), "UNLOCKED"));
    assert!(sat(&rec(1, 0, 0), "UNLOCKED"));
    let cow = json!({ "id": "spst_08-01", "fts": 1_708_906_788 });
    assert!(stage_satisfies(Some(&rec(3, 0, 0)), Some(&cow), "PLAYED"));
    assert!(stage_satisfies(Some(&rec(3, 0, 0)), Some(&cow), "PASS"));
    assert!(stage_satisfies(None, Some(&cow), "PLAYED"));
    assert!(!stage_satisfies(
        Some(&rec(0, 0, 0)),
        Some(&json!({ "fts": -1 })),
        "PLAYED"
    ));
    let stored =
        json!({ "state": 3, "startTimes": 0, "completeTimes": 0, "cowFirstTs": 1_708_906_788 });
    assert!(sat(&stored, "PASS"));
    assert!(sat(&rec(2, 0, 0), "WHATEVER"));
    assert!(!sat(&rec(1, 9, 0), "WHATEVER"));
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
        ["1stact_level_a001_01_beg", "main_0_level_main_00-01_beg"]
    );
    assert!(set.read.iter().all(|r| r.played && !r.archived));
    assert_eq!(set.groups, 0);
}

#[test]
fn a_zero_reread_count_is_still_read() {
    let set = parse(&archive_payload(), &index());
    assert!(set.present);
    assert_eq!(set.key.as_deref(), Some("storyreview"));
    assert_eq!((set.groups, set.stories), (1, 2));
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
    assert_eq!(set.read.len(), 3);
    assert_eq!(set.flag_hits, 2);
    assert_eq!(set.stories, 2);
    assert_eq!(set.archived(), 2);
    let by_id = by_id(&set);
    let both = by_id["1stact_level_a001_01_beg"];
    assert!(both.played && both.archived);
    assert_eq!(both.unlocked_at, Some(1_700_000_100));
    let archive_only = by_id["1stact_level_a001_01_end"];
    assert!(!archive_only.played && archive_only.archived);
    let flags_only = by_id["main_0_level_main_00-01_beg"];
    assert!(flags_only.played && !flags_only.archived);
    assert_eq!((flags_only.unlocked_at, flags_only.reread_count), (None, 0));
}

#[test]
fn an_empty_index_costs_the_flags_half_and_keeps_the_archive_half() {
    let mut payload = flags_payload();
    payload["user"]["storyreview"] = archive_payload()["user"]["storyreview"].clone();
    let set = parse(&payload, &HashMap::new());
    assert_eq!((set.flag_hits, set.flag_misses), (0, 3));
    assert_eq!(set.read.len(), 2);
    assert!(set.present);
}

#[test]
fn neither_source_is_an_empty_set_and_not_present() {
    let set = parse(&json!({ "user": { "troop": {} } }), &index());
    assert!(!set.present);
    assert!(set.read.is_empty());
    assert_eq!(set.groups, 0);
    assert!(!parse(&json!(null), &index()).present);
    assert!(!parse(&json!({}), &index()).present);
}

#[test]
fn accepts_the_other_block_spellings_and_an_unwrapped_group_map() {
    let camel = json!({ "user": { "storyReview": { "groups": { "g": { "stories": [ { "id": "g_level_s_beg" } ] } } } } });
    assert_eq!(parse(&camel, &index()).read.len(), 1);
    let flat =
        json!({ "user": { "storyreview": { "g": { "stories": [ { "id": "g_level_s_beg" } ] } } } });
    let set = parse(&flat, &index());
    assert_eq!((set.groups, set.read.len()), (1, 1));
}

#[test]
fn tolerates_junk_without_failing() {
    let junk = json!({ "user": {
        "status": { "flags": { "obt/main/level_main_00-01_beg": 0, "activities/a001/level_a001_01_beg": true } },
        "storyreview": { "groups": {
            "ok":      { "stories": [ { "id": "ok_level_1_beg", "rc": "2", "uts": "1700000000" } ] },
            "keyed":   { "stories": { "keyed_level_1_beg": { "rc": true } } },
            "noid":    { "stories": [ { "rc": 5 } ] },
            "notlist": { "stories": 7 },
            "scalar":  3,
            "empty":   {}
        } }
    } });
    let set = parse(&junk, &index());
    assert!(set.present);
    assert_eq!((set.groups, set.stories), (6, 2));
    assert_eq!((set.flags, set.flag_hits), (1, 1));
    let ids: Vec<&str> = set.read.iter().map(|r| r.story_id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "1stact_level_a001_01_beg",
            "keyed_level_1_beg",
            "ok_level_1_beg"
        ]
    );
    let ok = by_id(&set)["ok_level_1_beg"];
    assert_eq!((ok.reread_count, ok.unlocked_at), (2, Some(1_700_000_000)));
}

#[test]
fn a_negative_reread_count_is_clamped_and_the_story_still_counts() {
    let set = parse(
        &json!({ "user": { "storyreview": { "groups": { "g": {
            "rts": 1, "trailRewards": { "a": 1 }, "somethingNew": [1, 2],
            "stories": [
                { "id": "g_level_1_beg", "rc": 0, "uts": 5, "extra": "x" },
                { "id": "g_level_1_end", "rc": -1 }
            ]
        } } } } }),
        &index(),
    );
    assert_eq!((set.stories, set.read.len()), (2, 2));
    assert!(set.read.iter().all(|r| r.reread_count == 0));
    assert_eq!(set.review_hits, 0);
}

#[test]
fn names_the_key_whose_subtree_carries_a_story_shaped_id() {
    let set = parse(
        &json!({ "user": {
            "troop": { "chars": { "1": { "charId": "char_002_amiya" } } },
            "avg": { "groups": { "main_0": { "stories": ["main_0_level_main_00-01_beg"] } } }
        } }),
        &index(),
    );
    assert!(!set.present);
    assert_eq!(set.key.as_deref(), Some("avg"));
}
