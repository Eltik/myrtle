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
    /// Which ending IDs the user has unlocked
    #[serde(default)]
    ending: Option<serde_json::Value>,

    /// Relic IDs the user has seen/collected
    #[serde(default)]
    relic: Option<serde_json::Value>,

    /// Capsule pool unlocks (rogue_1 only)
    #[serde(default)]
    capsule: Option<serde_json::Value>,

    /// Band/token unlocks
    #[serde(default)]
    band: Option<serde_json::Value>,

    /// Challenge completions
    #[serde(default)]
    challenge: Option<serde_json::Value>,

    /// Current BP level
    #[serde(default)]
    bp: Option<BpProgress>,

    /// Highest difficulty grade cleared
    #[serde(default)]
    difficulty: Option<serde_json::Value>,

    /// Monthly squad data
    #[serde(default)]
    month_squad: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct BpProgress {
    #[serde(default)]
    level: i32,
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
        let count = count_object_keys(&progress.ending);
        let score = (count as f64 / theme.max_endings as f64).min(1.0);
        dimensions.push((WEIGHT_ENDINGS, score));
    }

    // 2. Difficulty (25%)
    if theme.max_difficulty_grade > 0 {
        let score = difficulty_milestone_score(progress, theme);
        dimensions.push((WEIGHT_DIFFICULTY, score));
    }

    // 3. Collectibles (20%) - relics + capsules + bands
    let max_collectibles = theme.max_relics + theme.max_capsules + theme.max_bands;
    if max_collectibles > 0 {
        let relics = count_object_keys(&progress.relic);
        let capsules = count_object_keys(&progress.capsule);
        let bands = count_object_keys(&progress.band);
        let total = relics + capsules + bands;
        let score = (total as f64 / max_collectibles as f64).min(1.0);

        let weight = if theme.max_challenges == 0 {
            WEIGHT_COLLECTIBLES + WEIGHT_CHALLENGES
        } else {
            WEIGHT_COLLECTIBLES
        };
        dimensions.push((weight, score));
    }

    // 4. BP Progress (15%)
    if theme.max_bp_levels > 0 {
        let bp_level = progress.bp.as_ref().map_or(0, |bp| bp.level);
        let raw = (bp_level as f64 / theme.max_bp_levels as f64).min(1.0);
        let score = log_curve_ratio(raw);
        dimensions.push((WEIGHT_BP, score));
    }

    // 5. Challenges (10%) - skip if theme has none (e.g. rogue_5)
    if theme.max_challenges > 0 {
        let count = count_object_keys(&progress.challenge);
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

    let highest = extract_highest_difficulty(progress);

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

fn extract_highest_difficulty(progress: &ThemeProgress) -> i32 {
    // TODO: Replace with actual logic, need to view game data
    let Some(diff) = &progress.difficulty else {
        return 0;
    };

    if let Some(obj) = diff.as_object() {
        // Try to find a "grade" or max key
        obj.values().filter_map(|v| v.as_i64()).max().unwrap_or(0) as i32
    } else {
        0
    }
}

fn count_object_keys(val: &Option<serde_json::Value>) -> usize {
    val.as_ref()
        .and_then(|v| v.as_object())
        .map_or(0, |obj| obj.len())
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}
