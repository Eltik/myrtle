#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::float_cmp)] // the overlap rule is exact arithmetic on the same deltas

use std::collections::{HashMap, HashSet};

use myrtle_engine_grade::GradeData;
use myrtle_engine_grade::improvements::{
    MedalGap, SandboxTopic, StageGap, StagePoolImprovements, medal_improvements,
    operator_improvements, roguelike_improvements, sandbox_improvements, stage_pool,
};
use myrtle_engine_grade::operators::ScoreModel;
use myrtle_engine_grade::stages::{PoolStage, StageClear};
use myrtle_gamedata_raw::tables::medal::{EventWindow, MedalDefinition, OperatorLock};
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorRarity, Phase};
use myrtle_gamedata_raw::tables::roguelike::{RoguelikeThemeGameData, SpecialEnding};
use myrtle_gamedata_raw::tables::sandbox_universe::{
    SandboxTemplate, SandboxTopicUniverse, SandboxUniverse,
};
use myrtle_types::roster::RosterEntry;
use serde_json::json;

const fn record(state: i16) -> StageClear {
    StageClear {
        state,
        state_max: state,
        inferred: false,
        complete_times: 1,
        practice_times: 0,
    }
}

fn pool_of(rows: &[(&'static str, f64, Option<&StageClear>, bool)]) -> StagePoolImprovements {
    let stages = rows
        .iter()
        .map(|&(stage_id, weight, clear, live_only)| PoolStage {
            stage_id,
            weight,
            decay: 1.0,
            clear,
            live_only,
        });
    stage_pool(stages, &HashMap::new(), |_: &str| None)
}

fn ids(gaps: &[StageGap]) -> Vec<&str> {
    gaps.iter().map(|g| g.stage_id.as_str()).collect()
}

#[test]
fn stages_bucket_by_state() {
    let (three, two) = (record(3), record(2));
    let pool = pool_of(&[
        ("main_01-01", 1.0, Some(&three), false),
        ("main_01-02", 1.0, Some(&two), false),
        ("main_01-03", 1.0, None, false),
    ]);
    assert_eq!((pool.total, pool.cleared, pool.three_starred), (3, 2, 1));
    assert_eq!(ids(&pool.missing), ["main_01-03"]);
    assert_eq!(ids(&pool.not_three_starred), ["main_01-02"]);
    assert_eq!(pool.not_three_starred[0].state, 2);
}

#[test]
fn a_two_star_live_only_clear_is_not_a_three_star_gap() {
    let two = record(2);
    let pool = pool_of(&[("act21side_04_m1", 1.0, Some(&two), true)]); // the hub closed with the event
    assert_eq!((pool.total, pool.cleared, pool.three_starred), (1, 1, 1));
    assert!(pool.not_three_starred.is_empty());
}

#[test]
fn gaps_sort_by_weight_desc() {
    let pool = pool_of(&[
        ("light", 0.85, None, false),
        ("heavy", 1.5, None, false),
        ("mid", 1.0, None, false),
    ]);
    assert_eq!(ids(&pool.missing), ["heavy", "mid", "light"]);
}

#[test]
fn a_gap_carries_stage_meta_and_rotation_only_when_given() {
    let stages = HashMap::from([(
        "camp_r_01".to_owned(),
        myrtle_gamedata_raw::tables::stage::Stage {
            code: "R-1".into(),
            zone_id: "zone_camp".into(),
            name: Some("Rotating".into()),
            ..Default::default()
        },
    )]);
    let stage = PoolStage {
        stage_id: "camp_r_01",
        weight: 1.0,
        decay: 1.0,
        clear: None,
        live_only: false,
    };
    let pool = stage_pool([stage].into_iter(), &stages, |_: &str| None);
    let gap = &pool.missing[0];
    assert_eq!(
        (gap.code.as_str(), gap.zone_id.as_str()),
        ("R-1", "zone_camp")
    );
    let wire = serde_json::to_value(gap).unwrap();
    assert!(wire.get("rotation").is_none(), "omitted, not null: {wire}");
}

fn theme(special: Vec<SpecialEnding>) -> RoguelikeThemeGameData {
    RoguelikeThemeGameData {
        theme_name: "Theme".into(),
        max_endings: 4,
        max_difficulty_grade: 15,
        max_bp_levels: 10,
        max_challenges: 8,
        special_endings: special,
        ..Default::default()
    }
}

#[test]
fn roguelike_reads_endings_difficulty_bp_and_challenges() {
    let mut data = GradeData::default();
    data.roguelike
        .themes
        .insert("rogue_2".into(), theme(Vec::new()));
    data.roguelike.themes.insert(
        "rogue_1".into(),
        theme(vec![SpecialEnding {
            id: "ro_ending_5".into(),
            name: "Hidden".into(),
            mode_name: None,
        }]),
    );
    let progress = json!({
        "record": { "endingCnt": { "NORMAL": { "e1": 1, "e2": 3 }, "MONTH": { "e2": 1, "e3": 1 } } },
        "collect": { "modeGrade": { "NORMAL": {
            "3": { "state": 2 }, "7": { "state": 1 }, "5": { "state": 3 }
        } } },
        "bp": { "reward": { "1": 1, "2": 1 } },
        "challenge": { "grade": { "c1": 1 } }
    });
    let themes = roguelike_improvements(&data, &[("rogue_2".into(), progress)]);
    assert_eq!(
        themes
            .iter()
            .map(|t| t.theme_id.as_str())
            .collect::<Vec<_>>(),
        ["rogue_1", "rogue_2"]
    );

    let untouched = &themes[0];
    assert_eq!(untouched.endings.current, 0);
    assert_eq!(untouched.difficulty.highest_cleared, -1);
    assert_eq!(untouched.special_endings.as_ref().unwrap().len(), 1);

    let played = &themes[1];
    assert_eq!((played.endings.current, played.endings.max), (3, 4));
    assert_eq!(played.difficulty.highest_cleared, 5);
    assert_eq!((played.bp.current, played.challenges.current), (2, 1));
    let wire = serde_json::to_value(played).unwrap();
    assert!(wire.get("special_endings").is_none(), "{wire}");
}

fn medal(id: &str, rarity: &str) -> MedalDefinition {
    serde_json::from_value(json!({
        "MedalId": id, "MedalName": id, "MedalType": "playerMedal", "SlotId": 0, "Rarity": rarity
    }))
    .unwrap()
}

#[test]
fn medals_split_earned_locked_and_permanent_with_community_share() {
    let mut data = GradeData::default();
    for (id, rarity) in [
        ("m_t1", "T1"),
        ("m_t3", "T3"),
        ("m_got", "T2"),
        ("m_collab", "T2"),
    ] {
        data.medals.medals.insert(id.into(), medal(id, rarity));
    }
    data.medals.operator_locked.insert(
        "m_collab".into(),
        OperatorLock {
            operator_id: "char_collab".into(),
            operator_name: "Collab".into(),
        },
    );
    let rows = vec![("m_got".to_owned(), None, Some(1), Some(5))];
    let owners = HashMap::from([("m_t3".to_owned(), 1)]);

    let out = medal_improvements(&data, &rows, &HashSet::new(), 3, &owners, 1_000);
    assert_eq!(
        out.permanent_missing
            .iter()
            .map(|m| m.medal_id.as_str())
            .collect::<Vec<_>>(),
        ["m_t3", "m_t1"],
        "rarest first, the earned medal left out"
    );
    assert_eq!(out.permanent_missing[0].owned_pct, Some(33.3));
    assert_eq!(out.permanent_missing[1].owned_pct, Some(0.0));
    assert_eq!(out.operator_locked.len(), 1);
    let lock = serde_json::to_value(&out.operator_locked[0]).unwrap();
    assert_eq!(lock["operator_lock"]["operatorId"], "char_collab");
    assert_eq!(lock["operator_lock"]["reason"], "collab");

    let owned = HashSet::from(["char_collab"]);
    let out = medal_improvements(&data, &rows, &owned, 0, &owners, 1_000);
    assert!(
        out.operator_locked.is_empty(),
        "owning the operator makes it a normal gap"
    );
    assert_eq!(out.permanent_missing.len(), 3);
    assert!(out.permanent_missing.iter().all(|m| m.owned_pct.is_none()));
}

fn phase(max_level: i32) -> Phase {
    Phase {
        max_level,
        ..Default::default()
    }
}

fn entry(operator_id: &str, elite: i16, level: i16, potential: i16) -> RosterEntry {
    RosterEntry {
        user_id: uuid::Uuid::nil(),
        operator_id: operator_id.into(),
        elite,
        level,
        exp: 0,
        potential,
        skill_level: 1,
        favor_point: 0,
        skin_id: None,
        default_skill: None,
        voice_lan: None,
        current_equip: None,
        current_tmpl: None,
        obtained_at: None,
        masteries: json!([]),
        modules: json!([]),
    }
}

#[test]
fn operator_gaps_count_elite_and_max_level_once() {
    let mut data = GradeData::default();
    let op = |name: &str, rarity| Operator {
        name: name.into(),
        rarity,
        phases: vec![phase(50), phase(80)],
        ..Default::default()
    };
    data.operators
        .insert("char_a".into(), op("A", OperatorRarity::SixStar));
    data.operators
        .insert("char_b".into(), op("B", OperatorRarity::FiveStar));
    data.operators
        .insert("char_done".into(), op("Done", OperatorRarity::SixStar));
    let roster = [
        entry("char_b", 0, 10, 0),
        entry("char_a", 0, 10, 0),
        entry("char_done", 1, 80, 5),
        entry("char_unknown", 0, 1, 0),
    ];

    let out = operator_improvements(&roster, &data, &HashSet::new(), ScoreModel::SHIPPED);
    let ids: Vec<&str> = out
        .below_milestone
        .iter()
        .map(|g| g.operator_id.as_str())
        .collect();
    assert_eq!(
        ids,
        ["char_a", "char_b"],
        "rarest first; complete and unknown skipped"
    );

    let a = &out.below_milestone[0];
    assert_eq!(a.missing, ["ELITE", "MAX_LEVEL", "POT6"]);
    assert_eq!(a.deltas.len(), 3);
    let delta = |tag: &str| {
        a.deltas
            .iter()
            .find(|d| d.tag == tag)
            .unwrap()
            .operator_grade_delta
    };
    assert_eq!(
        a.subscore_potential_gain,
        delta("ELITE").max(delta("MAX_LEVEL")) + delta("POT6")
    );
    assert!(a.subscore_potential_gain > 0.0);
    assert_eq!((a.max_mastery, a.max_module_level), (-1, -1));
}

#[test]
fn invested_only_skips_an_unraised_pull() {
    let mut data = GradeData::default();
    data.operators.insert(
        "char_a".into(),
        Operator {
            name: "A".into(),
            phases: vec![phase(50), phase(80)],
            ..Default::default()
        },
    );
    let roster = [entry("char_a", 0, 1, 0)];
    let model = ScoreModel {
        invested_only: true,
        ..ScoreModel::SHIPPED
    };
    assert!(
        operator_improvements(&roster, &data, &HashSet::new(), model)
            .below_milestone
            .is_empty()
    );
    assert_eq!(
        operator_improvements(&roster, &data, &HashSet::new(), ScoreModel::SHIPPED)
            .below_milestone
            .len(),
        1
    );
}

fn medal_ids(gaps: &[MedalGap]) -> Vec<&str> {
    gaps.iter().map(|m| m.medal_id.as_str()).collect()
}

#[test]
fn medal_windows_split_open_closed_upcoming_and_gone() {
    let mut data = GradeData::default();
    let window = |start, close, one_time| EventWindow {
        start,
        close,
        one_time,
    };
    for (id, rarity, w) in [
        ("m_closed", "T2", window(0, 500, false)),
        ("m_gone", "T2", window(0, 500, true)),
        ("m_upcoming", "T3", window(5_000, 9_000, true)),
        ("m_late", "T1", window(0, 5_000, false)),
        ("m_soon", "T1", window(0, 2_000, false)),
        ("m_open_ended", "T3", window(0, 0, false)),
        ("m_tie_t1", "T1", window(0, 3_000, false)),
        ("m_tie_t3", "T3", window(0, 3_000, false)),
        ("m_tie_b", "T2", window(0, 3_000, false)),
        ("m_tie_a", "T2", window(0, 3_000, false)),
        ("m_tie_c", "T2", window(0, 3_000, false)),
    ] {
        data.medals.medals.insert(id.into(), medal(id, rarity));
        data.medals.event_windows.insert(id.into(), w);
    }

    let out = medal_improvements(&data, &[], &HashSet::new(), 0, &HashMap::new(), 1_000);
    assert_eq!(
        medal_ids(&out.event_in_window_missing),
        [
            "m_soon",
            "m_tie_t3",
            "m_tie_a",
            "m_tie_b",
            "m_tie_c",
            "m_tie_t1",
            "m_late",
            "m_open_ended"
        ],
        "closing first, ties rarest then by id, no close last"
    );
    let open: Vec<Option<i64>> = out
        .event_in_window_missing
        .iter()
        .map(|m| m.end_time)
        .collect();
    assert_eq!(open[0], Some(2_000));
    assert_eq!(
        open[7], None,
        "a window without a close carries no end time"
    );
    assert_eq!(
        medal_ids(&out.unobtainable_missing),
        ["m_closed", "m_gone"],
        "the closed window first, with its close time"
    );
    assert_eq!(out.unobtainable_missing[0].end_time, Some(500));
    assert_eq!(out.unobtainable_missing[1].end_time, None);
    assert!(out.permanent_missing.is_empty());
    assert!(
        !medal_ids(&out.event_in_window_missing).contains(&"m_upcoming")
            && !medal_ids(&out.unobtainable_missing).contains(&"m_upcoming"),
        "a medal not yet earnable is no gap"
    );
}

fn sandbox_counts(max_band_levels: usize) -> SandboxUniverse {
    SandboxUniverse {
        max_achievements: 4,
        max_nodes: 4,
        max_zones: 2,
        max_tech_nodes: 4,
        max_quests: 2,
        max_base_level: 4,
        max_blueprints: 4,
        max_recipes: 2,
        max_music: 2,
        max_rifts: 4,
        max_band_levels,
        ..Default::default()
    }
}

fn sandbox_topic(
    id: &str,
    template: SandboxTemplate,
    counts: SandboxUniverse,
) -> SandboxTopicUniverse {
    SandboxTopicUniverse {
        topic_id: id.into(),
        name: format!("name of {id}"),
        template,
        sort_id: 0,
        counts,
    }
}

fn parts(topic: &SandboxTopic, key: &str) -> Vec<(&'static str, usize, usize)> {
    topic
        .categories
        .iter()
        .find(|c| c.key == key)
        .unwrap()
        .parts
        .iter()
        .map(|p| (p.label, p.current, p.max))
        .collect()
}

#[test]
fn sandbox_topics_show_blueprints_and_rifts_on_v2_and_bands_on_v3() {
    let data = GradeData {
        sandbox_universe: SandboxUniverse {
            topics: vec![
                sandbox_topic("sandbox_1", SandboxTemplate::V2, sandbox_counts(0)),
                sandbox_topic("sandbox_2", SandboxTemplate::V3, sandbox_counts(6)),
            ],
            ..sandbox_counts(0)
        },
        ..GradeData::default()
    };
    let progress = json!({ "template": {
        "SANDBOX_V2": { "sandbox_1": {
            "base": { "baseLv": 2 },
            "build": { "book": { "b_1": 1 } },
            "riftInfo": { "fixFinish": ["r_1", "r_2"] },
        } },
        "SANDBOX_V3": { "sandbox_2": {
            "base": { "level": 3 },
            "band": { "band_a": { "level": 2 } },
        } },
    } });

    let out = sandbox_improvements(&data, Some(&progress));
    assert_eq!(out.topics.len(), 2);
    assert!(out.topics.iter().all(|t| t.weight == 0.5));
    assert_eq!(
        out.total,
        f64::midpoint(out.topics[0].total, out.topics[1].total)
    );
    let (v2, v3) = (&out.topics[0], &out.topics[1]);
    assert_eq!(
        (v2.topic_id.as_str(), v2.name.as_str()),
        ("sandbox_1", "name of sandbox_1")
    );
    assert_eq!(
        v2.categories.iter().map(|c| c.key).collect::<Vec<_>>(),
        [
            "achievements",
            "exploration",
            "tech",
            "quests",
            "base",
            "content"
        ]
    );
    assert_eq!(
        parts(v2, "base"),
        [("Base level", 2, 4), ("Blueprints", 1, 4)]
    );
    assert_eq!(parts(v2, "content").last(), Some(&("Rifts cleared", 2, 4)));
    assert_eq!(
        parts(v3, "base"),
        [("Base level", 3, 4), ("Band levels", 2, 6)]
    );
    assert!(
        parts(v3, "content").iter().all(|p| p.0 != "Rifts cleared"),
        "V3 has no rifts"
    );
}

#[test]
fn sandbox_without_progress_still_lists_every_topic() {
    let data = GradeData {
        sandbox_universe: SandboxUniverse {
            topics: vec![sandbox_topic(
                "sandbox_1",
                SandboxTemplate::V2,
                sandbox_counts(0),
            )],
            ..sandbox_counts(0)
        },
        ..GradeData::default()
    };
    let out = sandbox_improvements(&data, None);
    assert_eq!(out.total, 0.0);
    assert_eq!(out.topics.len(), 1);
    assert_eq!(out.topics[0].weight, 1.0);
}
