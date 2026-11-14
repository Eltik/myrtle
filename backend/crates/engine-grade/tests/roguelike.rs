#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_grade::roguelike::*;
use myrtle_gamedata_raw::tables::roguelike::{RoguelikeGameData, RoguelikeThemeGameData};
use serde::Deserialize;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

fn theme(id: &str) -> RoguelikeThemeGameData {
    RoguelikeThemeGameData {
        theme_id: id.into(),
        max_endings: 4,
        max_relics: 2,
        relic_ids: ids(&["r1", "r2"]),
        max_bands: 2,
        band_ids: ids(&["b1", "b2"]),
        max_challenges: 2,
        max_difficulty_grade: 15,
        max_bp_levels: 10,
        ..RoguelikeThemeGameData::default()
    }
}

fn halfway() -> serde_json::Value {
    serde_json::json!({
        "record": { "endingCnt": {
            "NORMAL": { "ro_ending_1": 3, "ro_ending_2": 1 },
            "MONTH_TEAM": { "ro_ending_2": 1 },
        }},
        "collect": {
            "modeGrade": { "NORMAL": {
                "0": { "state": 2 },
                "8": { "state": 2 },
                "12": { "state": 1 },
            }},
            "relic": { "r1": { "state": 1 } },
            "band": { "b1": { "state": 1 }, "b2": { "state": 0 } },
        },
        "challenge": { "grade": { "c1": 1 } },
        "bp": { "reward": {
            "bp_level_1": 1, "bp_level_2": 1, "bp_level_3": 1, "bp_level_4": 1, "bp_level_5": 1,
        }},
    })
}

/// endings 2/4, difficulty 8/15 (-> 0.50 rung), collectibles 2/4, BP log2(1.5),
/// challenges 1/2, weighted 30/25/20/15/10.
const HALFWAY_SCORE: f64 =
    (30.0 * 0.5 + 25.0 * 0.5 + 20.0 * 0.5 + 15.0 * 0.584_962_500_721_156_2 + 10.0 * 0.5) / 100.0;

fn difficulty(progress: serde_json::Value, max: i32) -> f64 {
    let theme = RoguelikeThemeGameData {
        max_difficulty_grade: max,
        ..RoguelikeThemeGameData::default()
    };
    let p = ThemeProgress::deserialize(&progress).unwrap_or_default();
    difficulty_milestone_score(&p, &theme)
}

fn normal_grades(grades: &[(&str, i32)]) -> serde_json::Value {
    let normal: serde_json::Map<String, serde_json::Value> = grades
        .iter()
        .map(|(g, s)| ((*g).to_string(), serde_json::json!({ "state": s })))
        .collect();
    serde_json::json!({ "collect": { "modeGrade": { "NORMAL": normal } } })
}

#[test]
fn newer_themes_weigh_more() {
    let weights: Vec<f64> = [
        "rogue_1", "rogue_2", "rogue_3", "rogue_4", "rogue_5", "rogue_9",
    ]
    .iter()
    .map(|t| theme_weight(t))
    .collect();
    let expected = [0.12, 0.16, 0.20, 0.24, 0.28, 0.20];
    for (got, want) in weights.iter().zip(expected) {
        assert!(close(*got, want), "{got} vs {want}");
    }
}

#[test]
fn grades_every_dimension_of_a_theme() {
    let data = theme("rogue_3");
    let progress = halfway();
    let p = ThemeProgress::deserialize(&progress).unwrap_or_default();
    assert!(close(grade_theme(&p, &progress, &data), HALFWAY_SCORE));
}

#[test]
fn difficulty_snaps_to_milestones() {
    assert!(close(difficulty(normal_grades(&[("15", 2)]), 15), 1.0));
    assert!(close(difficulty(normal_grades(&[("18", 2)]), 15), 1.0));
    assert!(close(difficulty(normal_grades(&[("12", 2)]), 15), 0.75));
    assert!(close(difficulty(normal_grades(&[("8", 2)]), 15), 0.50));
    assert!(close(difficulty(normal_grades(&[("1", 2)]), 15), 0.25));
    // Only cleared grades (state >= 2) count.
    assert!(close(
        difficulty(normal_grades(&[("15", 1), ("1", 2)]), 15),
        0.25
    ));
    // Other modes are ignored.
    let other_mode =
        serde_json::json!({ "collect": { "modeGrade": { "HARD": { "15": { "state": 2 } } } } });
    assert!(close(difficulty(other_mode, 15), 0.0));
    assert!(close(difficulty(serde_json::json!({}), 15), 0.0));
    assert!(close(difficulty(normal_grades(&[("15", 2)]), 0), 0.0));
}

#[test]
fn a_clear_on_grade_zero_scores_nothing() {
    // CURRENT BEHAVIOR, suspected bug: the match arm says "Any clear at all"
    // earns 0.25, but `highest <= 0` returns 0.0 first, so a player whose only
    // clear is the base difficulty (grade 0) gets no difficulty credit.
    assert!(close(difficulty(normal_grades(&[("0", 2)]), 15), 0.0));
}

#[test]
fn endings_are_unique_ids_across_modes_and_capped() {
    let data = RoguelikeThemeGameData {
        max_endings: 2,
        ..RoguelikeThemeGameData::default()
    };
    let progress = serde_json::json!({ "record": { "endingCnt": {
        "NORMAL": { "e1": 1, "e2": 1 },
        "MONTH_TEAM": { "e3": 1, "e1": 4 },
    }}});
    let p = ThemeProgress::deserialize(&progress).unwrap_or_default();
    assert!(close(grade_theme(&p, &progress, &data), 1.0));
}

#[test]
fn collectibles_absorb_the_challenge_weight_when_a_theme_has_none() {
    let mut data = RoguelikeThemeGameData {
        max_endings: 1,
        max_relics: 1,
        relic_ids: ids(&["r1"]),
        ..RoguelikeThemeGameData::default()
    };
    let progress = serde_json::json!({ "record": { "endingCnt": { "NORMAL": { "e1": 1 } } } });
    let p = ThemeProgress::deserialize(&progress).unwrap_or_default();
    // Endings 30 x 1.0, collectibles 30 x 0.0.
    assert!(close(grade_theme(&p, &progress, &data), 0.5));
    data.max_challenges = 1;
    // Endings 30 x 1.0, collectibles 20 x 0.0, challenges 10 x 0.0.
    assert!(close(grade_theme(&p, &progress, &data), 0.5));
}

#[test]
fn a_theme_with_no_dimensions_scores_zero() {
    let progress = halfway();
    let p = ThemeProgress::deserialize(&progress).unwrap_or_default();
    assert!(close(
        grade_theme(&p, &progress, &RoguelikeThemeGameData::default()),
        0.0
    ));
}

#[test]
fn themes_blend_by_theme_weight_and_unknown_themes_are_skipped() {
    let mut data = RoguelikeGameData::default();
    data.themes.insert("rogue_3".into(), theme("rogue_3"));
    data.themes.insert("rogue_1".into(), theme("rogue_1"));
    let progress = vec![
        ("rogue_3".to_string(), halfway()),
        ("rogue_1".to_string(), serde_json::json!({})),
        ("rogue_404".to_string(), halfway()),
    ];
    let expected = HALFWAY_SCORE * 0.20 / (0.20 + 0.12);
    assert!(close(grade_roguelike(&progress, &data), expected));
}

#[test]
fn no_progress_or_only_unknown_themes_grade_zero() {
    let data = RoguelikeGameData::default();
    assert!(close(grade_roguelike(&[], &data), 0.0));
    assert!(close(
        grade_roguelike(&[("rogue_1".into(), halfway())], &data),
        0.0
    ));
}

#[test]
fn malformed_progress_falls_back_to_empty() {
    let mut data = RoguelikeGameData::default();
    data.themes.insert("rogue_2".into(), theme("rogue_2"));
    let bad = serde_json::json!({ "record": "not an object" });
    assert!(close(
        grade_roguelike(&[("rogue_2".into(), bad)], &data),
        0.0
    ));
}
