use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::core::gamedata::types::roguelike::{RoguelikeGameData, RoguelikeThemeGameData};

use super::{Dimension, weighted_average};

const WEIGHT_ENDINGS: f64 = 30.0;
const WEIGHT_DIFFICULTY: f64 = 25.0;
const WEIGHT_COLLECTIBLES: f64 = 20.0;
const WEIGHT_BP: f64 = 15.0;
const WEIGHT_CHALLENGES: f64 = 10.0;

// Newer themes weigh slightly more. Might flatten all to 1.0.
fn theme_weight(theme_id: &str) -> f64 {
    match theme_id {
        "rogue_1" => 0.12,
        "rogue_2" => 0.16,
        "rogue_3" => 0.20,
        "rogue_4" => 0.24,
        "rogue_5" => 0.28,
        _ => 0.20, // future themes get neutral weight
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ThemeProgress {
    #[serde(default)]
    record: Option<Record>,
    #[serde(default)]
    collect: Option<Collect>,
    #[serde(default)]
    challenge: Option<Challenge>,
    #[serde(default)]
    bp: Option<Bp>,
}

/// progress.record
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Record {
    /// {MODE: {`ending_id`: count}} - ex. {"NORMAL": {"`ro_ending_1"`: 11}}
    #[serde(default)]
    ending_cnt: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// progress.collect
///
/// Relics, capsules and bands are counted by
/// [`RoguelikeThemeGameData::count_collected`] on the raw JSON, against the
/// archive id lists, so they are not modelled here.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Collect {
    /// {MODE: {`grade_str`: {state: 0/2, progress: ...}}}
    #[serde(default)]
    mode_grade: Option<HashMap<String, HashMap<String, CollectEntry>>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct CollectEntry {
    #[serde(default)]
    state: i32,
}

/// progress.challenge
#[derive(Debug, Deserialize, Default)]
struct Challenge {
    /// {`challenge_id`: `grade_value`}
    #[serde(default)]
    grade: Option<HashMap<String, serde_json::Value>>,
}

/// progress.bp
#[derive(Debug, Deserialize, Default)]
struct Bp {
    /// {`bp_level_N`: 1} - count of keys = BP level reached
    #[serde(default)]
    reward: Option<HashMap<String, serde_json::Value>>,
}

/// 0.0-1.0, themes weighted by `theme_weight`.
pub fn grade_roguelike(
    theme_progress: &[(String, serde_json::Value)],
    roguelike_data: &RoguelikeGameData,
) -> f64 {
    if theme_progress.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (theme_id, progress_json) in theme_progress {
        let Some(theme_data) = roguelike_data.themes.get(theme_id) else {
            continue;
        };

        let progress = ThemeProgress::deserialize(progress_json).unwrap_or_default();

        let score = grade_theme(&progress, progress_json, theme_data);
        let w = theme_weight(theme_id);

        weighted_sum += score * w;
        weight_total += w;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

fn grade_theme(
    progress: &ThemeProgress,
    progress_json: &serde_json::Value,
    theme: &RoguelikeThemeGameData,
) -> f64 {
    let mut dimensions: Vec<Dimension> = vec![];

    // 1. Endings (30%)
    if theme.max_endings > 0 {
        let count = progress
            .record
            .as_ref()
            .and_then(|r| r.ending_cnt.as_ref())
            .map_or(0, |ec| {
                let mut ids = HashSet::new();
                for mode_endings in ec.values() {
                    for ending_id in mode_endings.keys() {
                        ids.insert(ending_id.as_str());
                    }
                }
                ids.len()
            });
        let score = (count as f64 / f64::from(theme.max_endings)).min(1.0);
        dimensions.push((WEIGHT_ENDINGS, score));
    }

    // 2. Difficulty (25%)
    if theme.max_difficulty_grade > 0 {
        let score = difficulty_milestone_score(progress, theme);
        dimensions.push((WEIGHT_DIFFICULTY, score));
    }

    // 3. Collectibles (20%): relics + capsules + bands, counted against the archive
    //    id lists (`count_collected`), so each is bounded by its max already.
    let max_collectibles = theme.max_relics + theme.max_capsules + theme.max_bands;
    if max_collectibles > 0 {
        let collected = theme.count_collected(progress_json);
        let total = collected.relics + collected.capsules + collected.bands;
        let score = total as f64 / f64::from(max_collectibles);

        let weight = if theme.max_challenges == 0 {
            WEIGHT_COLLECTIBLES + WEIGHT_CHALLENGES
        } else {
            WEIGHT_COLLECTIBLES
        };
        dimensions.push((weight, score));
    }

    // 4. BP Progress (15%)
    if theme.max_bp_levels > 0 {
        let bp_level = progress
            .bp
            .as_ref()
            .and_then(|bp| bp.reward.as_ref())
            .map_or(0, |r| r.len() as i32);
        let raw = (f64::from(bp_level) / f64::from(theme.max_bp_levels)).min(1.0);
        let score = log_curve_ratio(raw);
        dimensions.push((WEIGHT_BP, score));
    }

    // 5. Challenges (10%); some themes have none (rogue_5)
    if theme.max_challenges > 0 {
        let count = progress
            .challenge
            .as_ref()
            .and_then(|c| c.grade.as_ref())
            .map_or(0, std::collections::HashMap::len);
        let score = (count as f64 / f64::from(theme.max_challenges)).min(1.0);
        dimensions.push((WEIGHT_CHALLENGES, score));
    }

    weighted_average(&dimensions)
}

fn difficulty_milestone_score(progress: &ThemeProgress, theme: &RoguelikeThemeGameData) -> f64 {
    let max = f64::from(theme.max_difficulty_grade);
    if max <= 0.0 {
        return 0.0;
    }

    let highest = progress
        .collect
        .as_ref()
        .and_then(|c| c.mode_grade.as_ref())
        .and_then(|mg| mg.get("NORMAL"))
        .map_or(-1, |normal| {
            normal
                .iter()
                .filter(|(_, entry)| entry.state >= 2)
                .filter_map(|(grade_str, _)| grade_str.parse::<i32>().ok())
                .max()
                .unwrap_or(-1)
        });

    if highest <= 0 {
        return 0.0;
    }

    let ratio = f64::from(highest) / max;
    match ratio {
        r if r >= 1.0 => 1.0, // Cleared max difficulty
        r if r >= 0.75 => 0.75,
        r if r >= 0.50 => 0.50,
        _ => 0.25, // Any clear at all
    }
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        (30.0 * 0.5 + 25.0 * 0.5 + 20.0 * 0.5 + 15.0 * 0.584_962_500_721_156_2 + 10.0 * 0.5)
            / 100.0;

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
}
