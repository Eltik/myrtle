use crate::enrich::stage_class::{activity_for_stage_id, is_hard_path, is_topic_key, node_code};
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;

#[test]
fn hard_paths_are_spotted_case_insensitively() {
    for hard in [
        "Obt/Main/level_tough_12-01",
        "Obt/Hard/level_hard_15-01",
        "obt/hard/level_x",
        "Activities/ACT_HARD_01/level_a",
        "Obt/Main/level_main_hard_10-01",
    ] {
        assert!(is_hard_path(hard), "{hard}");
    }
    for normal in [
        "Obt/Main/level_main_01-07",
        "obt/hardware/level_x",
        "level_hard",
        "",
    ] {
        assert!(!is_hard_path(normal), "{normal}");
    }
}

#[test]
fn topic_keys_come_from_the_season_number() {
    assert_eq!(is_topic_key("rogue_4").as_deref(), Some("rogue_4"));
    assert_eq!(is_topic_key("rogue4").as_deref(), Some("rogue_4"));
    assert_eq!(is_topic_key("rogue_10").as_deref(), Some("rogue_10"));
    assert_eq!(is_topic_key("rogue_04").as_deref(), Some("rogue_4"));
    assert_eq!(is_topic_key("rogue_"), None);
    assert_eq!(is_topic_key("sandbox"), None);
    // Anything after the digits fails the parse.
    assert_eq!(is_topic_key("rogue4_b-4-c"), None);
}

#[test]
fn node_code_is_everything_after_the_first_underscore() {
    assert_eq!(node_code("rogue4_b-4-c"), "b-4-c");
    assert_eq!(node_code("sandbox2_04"), "04");
    assert_eq!(node_code("crisis_v2_01-02"), "v2_01-02");
    assert_eq!(node_code("plain"), "plain");
    assert_eq!(node_code("trailing_"), "");
}

fn act(id: &str, name: &str) -> ActivityBasicInfo {
    ActivityBasicInfo {
        id: id.to_owned(),
        name: name.to_owned(),
        ..Default::default()
    }
}

#[test]
fn a_stage_id_resolves_to_its_activity_at_a_segment_boundary() {
    let (long, short, unnamed) = (
        act("act4d0", "Operational Intelligence"),
        act("act4", "Short"),
        act("act6d5", ""),
    );
    let acts = [&long, &short, &unnamed];
    assert_eq!(
        activity_for_stage_id(&acts, "act4d0_01").map(|a| a.id.as_str()),
        Some("act4d0")
    );
    assert_eq!(
        activity_for_stage_id(&acts, "act4d0_01#f#").map(|a| a.id.as_str()),
        Some("act4d0")
    );
    assert_eq!(
        activity_for_stage_id(&acts, "act4_02").map(|a| a.id.as_str()),
        Some("act4")
    );
    // `act4` must not claim `act40side_01`, and a nameless activity groups nothing.
    assert!(activity_for_stage_id(&acts, "act40side_01").is_none());
    assert!(activity_for_stage_id(&acts, "act6d5_01").is_none());
    assert!(activity_for_stage_id(&acts, "main_01-07").is_none());
}
