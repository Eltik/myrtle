use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::core::gamedata::types::roguelike::{RoguelikeGameData, RoguelikeThemeGameData};

const WEIGHT_ENDINGS: f64 = 30.0;
const WEIGHT_DIFFICULTY: f64 = 25.0;
const WEIGHT_COLLECTIBLES: f64 = 20.0;
const WEIGHT_BP: f64 = 15.0;
const WEIGHT_CHALLENGES: f64 = 10.0;

// Newer themes weigh slightly more
// Might consider making all of them a weight of 1.0
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

type Dimension = (f64, f64);

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
    /// {MODE: {ending_id: count}} - ex. {"NORMAL": {"ro_ending_1": 11}}
    #[serde(default)]
    ending_cnt: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// progress.collect
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Collect {
    /// {relic_id: {state: 0/1/2/, progress: ...}}
    #[serde(default)]
    relic: Option<HashMap<String, CollectEntry>>,
    #[serde(default)]
    capsule: Option<HashMap<String, CollectEntry>>,
    #[serde(default)]
    band: Option<HashMap<String, CollectEntry>>,
    /// {MODE: {grade_str: {state: 0/2, progress: ...}}}
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
    /// {challenge_id: grade_value}
    #[serde(default)]
    grade: Option<HashMap<String, serde_json::Value>>,
}

/// progress.bp
#[derive(Debug, Deserialize, Default)]
struct Bp {
    /// {bp_level_N: 1} — count of keys = BP level reached
    #[serde(default)]
    reward: Option<HashMap<String, serde_json::Value>>,
}

/// Grades a user's roguelike progress across all themes.
/// Returns 0.0–1.0.
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

        let progress: ThemeProgress =
            serde_json::from_value(progress_json.clone()).unwrap_or_default();

        let score = grade_theme(&progress, theme_data);
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

fn grade_theme(progress: &ThemeProgress, theme: &RoguelikeThemeGameData) -> f64 {
    let mut dimensions: Vec<Dimension> = vec![];

    // 1. Endings (30%)
    if theme.max_endings > 0 {
        let count = progress
            .record
            .as_ref()
            .and_then(|r| r.ending_cnt.as_ref())
            .map(|ec| {
                let mut ids = HashSet::new();
                for mode_endings in ec.values() {
                    for ending_id in mode_endings.keys() {
                        ids.insert(ending_id.as_str());
                    }
                }
                ids.len()
            })
            .unwrap_or(0);
        let score = (count as f64 / theme.max_endings as f64).min(1.0);
        dimensions.push((WEIGHT_ENDINGS, score));
    }

    // 2. Difficulty (25%)
    if theme.max_difficulty_grade > 0 {
        let score = difficulty_milestone_score(progress, theme);
        dimensions.push((WEIGHT_DIFFICULTY, score));
    }

    // 3. Collectibles (20%) - relics + capsules + bands
    //    Cap per-category: user data includes non-catalog items (event relics, etc.)
    //    that exceed archiveComp max. Cap each so overflow in one category
    //    doesn't compensate for missing items in another.
    let max_collectibles = theme.max_relics + theme.max_capsules + theme.max_bands;
    if max_collectibles > 0 {
        let relics = count_unlocked(&progress.collect.as_ref().and_then(|c| c.relic.clone()))
            .min(theme.max_relics as usize);
        let capsules = count_unlocked(&progress.collect.as_ref().and_then(|c| c.capsule.clone()))
            .min(theme.max_capsules as usize);
        let bands = count_unlocked(&progress.collect.as_ref().and_then(|c| c.band.clone()))
            .min(theme.max_bands as usize);
        let total = relics + capsules + bands;
        let score = total as f64 / max_collectibles as f64;

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
            .map(|r| r.len() as i32)
            .unwrap_or(0);
        let raw = (bp_level as f64 / theme.max_bp_levels as f64).min(1.0);
        let score = log_curve_ratio(raw);
        dimensions.push((WEIGHT_BP, score));
    }

    // 5. Challenges (10%) - skip if theme has none (e.g. rogue_5)
    if theme.max_challenges > 0 {
        let count = progress
            .challenge
            .as_ref()
            .and_then(|c| c.grade.as_ref())
            .map(|g| g.len())
            .unwrap_or(0);
        let score = (count as f64 / theme.max_challenges as f64).min(1.0);
        dimensions.push((WEIGHT_CHALLENGES, score));
    }

    // Weighted average
    let total_weight: f64 = dimensions.iter().map(|(w, _)| w).sum();
    if total_weight > 0.0 {
        dimensions.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight
    } else {
        0.0
    }
}

fn difficulty_milestone_score(progress: &ThemeProgress, theme: &RoguelikeThemeGameData) -> f64 {
    let max = theme.max_difficulty_grade as f64;
    if max <= 0.0 {
        return 0.0;
    }

    let highest = progress
        .collect
        .as_ref()
        .and_then(|c| c.mode_grade.as_ref())
        .and_then(|mg| mg.get("NORMAL"))
        .map(|normal| {
            normal
                .iter()
                .filter(|(_, entry)| entry.state >= 2)
                .filter_map(|(grade_str, _)| grade_str.parse::<i32>().ok())
                .max()
                .unwrap_or(-1)
        })
        .unwrap_or(-1);

    if highest <= 0 {
        return 0.0;
    }

    let ratio = highest as f64 / max;
    match ratio {
        r if r >= 1.0 => 1.0,   // Cleared max difficulty
        r if r >= 0.75 => 0.75, // 75%+ of max
        r if r >= 0.50 => 0.50, // 50%+ of max
        _ => 0.25,              // Any clear at all
    }
}

fn count_unlocked(items: &Option<HashMap<String, CollectEntry>>) -> usize {
    items
        .as_ref()
        .map(|m| m.values().filter(|e| e.state >= 1).count())
        .unwrap_or(0)
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}
