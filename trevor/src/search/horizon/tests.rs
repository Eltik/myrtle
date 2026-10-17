use super::*;
use serde_json::json;
use super::build::{build_gate_order, parse_day};

fn guide() -> serde_json::Value {
    json!({"items": [
        {"groupId": "main_0", "position": 1, "released": "by 2020-01-22", "kind": "main", "name": "Ep 0"},
        {"groupId": "act1", "position": 2, "released": "not dated in the game data", "kind": "event", "name": "A1"},
        {"groupId": "main_1", "position": 3, "released": "about 2020-07-01", "kind": "main", "name": "Ep 1"}
    ]})
}

fn order() -> Arc<ReleaseOrder> {
    let stories = vec![
        json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
        json!({"storyId": "a1_a", "groupId": "act1", "storySort": 1}),
        json!({"storyId": "a1_b", "groupId": "act1", "storySort": 2}),
        json!({"storyId": "a1_c", "groupId": "act1", "storySort": 3}),
        json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1}),
    ];
    Arc::new(ReleaseOrder::from_parts(&guide(), &stories))
}

#[test]
fn story_horizon_keeps_earlier_groups_and_earlier_stories_of_its_own() {
    let h = order().horizon("a1_b").unwrap();
    assert_eq!(h.pos, 3);
    assert!(h.allows("m0_a", "main_0"));
    assert!(h.allows("a1_a", "act1"));
    assert!(h.allows("a1_b", "act1"));
    assert!(!h.allows("a1_c", "act1"));
    assert!(!h.allows("m1_a", "main_1"));
    // Unplaced groups (operator files, voice lines, generated summaries) are hidden, never let through.
    assert!(!h.allows("archive_char_002_amiya", "archive"));
    assert!(!h.allows("unknown_story", "act1"));
    // The same horizon by gatePos; gatePos 0 hides everything.
    let o = order();
    let g = o.horizon_at(3).unwrap();
    assert!(g.allows("a1_b", "act1") && !g.allows("a1_c", "act1"));
    assert!(!o.horizon_at(0).unwrap().allows("m0_a", "main_0"));
}

#[test]
fn dated_sources_follow_the_horizon_date() {
    assert_eq!(parse_day("by 2020-01-22"), Some(1_579_651_200));
    assert_eq!(parse_day("not dated"), None);
    let rows = [json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
                json!({"storyId": "a1_a", "groupId": "act1", "storySort": 1}),
                json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1})];
    let mut o = ReleaseOrder::from_parts(&guide(), &rows);
    let dates = json!({"dates": {"archive_char_002_amiya": 1_579_651_200, "voice_char_1": 1_579_651_201}});
    o.set_source_dates(&dates["dates"]);
    o.set_gate(build_gate_order(&guide(), &rows, Some(&dates), None));
    let o = Arc::new(o);
    // Amiya's file sits before main_0's stories (same day); the voice line before main_1's.
    assert_eq!(o.gate().source_pos.get("archive_char_002_amiya"), Some(&1));
    assert_eq!(o.gate().source_pos.get("voice_char_1"), Some(&3));
    for (h, amiya, voice) in [("main_0", true, false), ("act1", true, false), ("main_1", true, true)] {
        for hz in [o.horizon(h).unwrap(), o.legacy_horizon(h).unwrap()] {
            assert_eq!(hz.allows("archive_char_002_amiya", "archive"), amiya, "{h} {:?}", hz.legacy);
            assert_eq!(hz.allows("voice_char_1", "voice"), voice, "{h}");
            // Undated sources stay hidden at any horizon.
            assert!(!hz.allows("enemy_enemy_1007_slime", "enemy"));
        }
    }
}

#[test]
fn group_horizon_keeps_the_whole_group() {
    let h = order().horizon("act1").unwrap();
    assert!(h.allows("a1_c", "act1"));
    assert!(!h.allows("m1_a", "main_1"));
    assert!(order().horizon("nope").is_err());
}

#[test]
fn story_sort_ties_break_by_file_order_and_eras_sit_at_the_last_story() {
    let stories = vec![
        json!({"storyId": "m0_a", "groupId": "main_0", "storySort": 1}),
        json!({"storyId": "a1_x", "groupId": "act1", "storySort": 7}),
        json!({"storyId": "a1_y", "groupId": "act1", "storySort": 7}),
        json!({"storyId": "a1_w", "groupId": "act1", "storySort": 2}),
        json!({"storyId": "m1_a", "groupId": "main_1", "storySort": 1}),
        json!({"storyId": "rec_1", "groupId": "story_x_set_1", "storySort": 0}),
    ];
    let o = Arc::new(ReleaseOrder::from_parts(&guide(), &stories));
    assert_eq!(o.gate().story_order, ["m0_a", "a1_w", "a1_x", "a1_y", "m1_a", "rec_1"]);
    assert_eq!(o.gate().never, ["rec_1"]);
    // The tie: the old rule let a1_x see a1_y; the gatePos rule does not, and a1_y still sees a1_x.
    assert!(o.legacy_horizon("a1_x").unwrap().allows("a1_y", "act1"));
    assert!(!o.horizon("a1_x").unwrap().allows("a1_y", "act1"));
    assert!(o.horizon("a1_y").unwrap().allows("a1_x", "act1"));
    // Never-visible stories stay hidden even past their rank.
    assert!(!o.horizon_at(99).unwrap().allows("rec_1", "story_x_set_1"));
    let eras: Vec<(&str, u32)> = o.gate().eras.iter().map(|e| (e.id.as_str(), e.position)).collect();
    assert_eq!(eras, [("main_0", 1), ("main_1", 5)]);
    assert_eq!(o.era_at(4).map(|e| e.id.as_str()), Some("main_0"));
    assert_eq!(o.era_at(0), None);
}
