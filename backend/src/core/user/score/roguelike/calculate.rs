//! Roguelike score calculation

use crate::core::local::types::roguelike::{RoguelikeGameData, RoguelikeThemeGameData};
use crate::core::user::types::User;

use super::types::{
    RoguelikeBreakdown, RoguelikeScore, RoguelikeThemeDetails, RoguelikeThemeScore,
};

/// Point values for roguelike scoring
mod points {
    pub const THEME_PLAYED: f32 = 50.0;
    pub const ENDING_UNLOCKED: f32 = 25.0;
    pub const BP_LEVEL: f32 = 5.0;
    pub const BUFF_UNLOCKED: f32 = 10.0;
    pub const BAND_UNLOCKED: f32 = 3.0;
    pub const RELIC_UNLOCKED: f32 = 2.0;
    pub const CAPSULE_UNLOCKED: f32 = 2.0;
    pub const CHALLENGE_GRADE: f32 = 15.0;
    /// Bonus per challenge cleared at max difficulty (grade 2)
    pub const GRADE_2_BONUS: f32 = 25.0;
    /// Bonus per theme with at least one grade 2 clear
    pub const MAX_DIFFICULTY_THEME: f32 = 100.0;
}

/// Calculate a percentage (0-100) from current and max values
fn calculate_percentage(current: i32, max: i32) -> f32 {
    if max > 0 {
        (current as f32 / max as f32) * 100.0
    } else {
        0.0
    }
}

/// Calculate roguelike score for a user
pub fn calculate_roguelike_score(user: &User, game_data: &RoguelikeGameData) -> RoguelikeScore {
    let mut theme_scores = Vec::new();

    // Initialize breakdown with max totals from game data
    let mut breakdown = RoguelikeBreakdown {
        total_themes_available: game_data.theme_count(),
        total_max_endings: game_data.total_max_endings(),
        total_max_collectibles: game_data.total_max_collectibles(),
        total_max_challenges: game_data.total_max_challenges(),
        total_max_monthly_squads: game_data.total_max_monthly_squads(),
        ..Default::default()
    };

    // Process each theme (rogue_1 through rogue_5)
    for i in 1..=5 {
        let theme_id = format!("rogue_{}", i);
        let theme_game_data = game_data.themes.get(&theme_id);

        if let Some(theme_data) = user.rlv2.outer.get(&theme_id)
            && let Some(theme_score) = calculate_theme_score(&theme_id, theme_data, theme_game_data)
        {
            // Update breakdown
            breakdown.themes_played += 1;
            breakdown.total_endings += theme_score.details.endings_unlocked;
            breakdown.total_bp_levels += theme_score.details.bp_level;
            breakdown.total_buffs += theme_score.details.buffs_unlocked;
            breakdown.total_collectibles += theme_score.details.bands_unlocked
                + theme_score.details.relics_unlocked
                + theme_score.details.capsules_unlocked;
            breakdown.total_runs += theme_score.details.normal_runs
                + theme_score.details.challenge_runs
                + theme_score.details.month_team_runs;
            breakdown.total_grade_2_challenges += theme_score.details.grade_2_challenges;
            if theme_score.details.grade_2_challenges > 0 {
                breakdown.themes_at_max_difficulty += 1;
            }

            theme_scores.push(theme_score);
        }
    }

    // Calculate breakdown completion percentages
    breakdown.themes_completion_percentage =
        calculate_percentage(breakdown.themes_played, breakdown.total_themes_available);
    breakdown.endings_completion_percentage =
        calculate_percentage(breakdown.total_endings, breakdown.total_max_endings);
    breakdown.collectibles_completion_percentage = calculate_percentage(
        breakdown.total_collectibles,
        breakdown.total_max_collectibles,
    );
    breakdown.challenges_completion_percentage = calculate_percentage(
        breakdown.total_grade_2_challenges,
        breakdown.total_max_challenges,
    );

    // Calculate overall completion percentage (weighted average)
    // Weight: endings (30%), collectibles (50%), challenges (20%)
    let endings_weight = 0.30;
    let collectibles_weight = 0.50;
    let challenges_weight = 0.20;
    breakdown.overall_completion_percentage = (breakdown.endings_completion_percentage
        * endings_weight)
        + (breakdown.collectibles_completion_percentage * collectibles_weight)
        + (breakdown.challenges_completion_percentage * challenges_weight);

    // Sort by total score descending
    theme_scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_score = theme_scores.iter().map(|t| t.total_score).sum();

    RoguelikeScore {
        total_score,
        theme_scores,
        breakdown,
    }
}

fn calculate_theme_score(
    theme_id: &str,
    data: &serde_json::Value,
    theme_game_data: Option<&RoguelikeThemeGameData>,
) -> Option<RoguelikeThemeScore> {
    let mut details = RoguelikeThemeDetails::default();

    // Parse record.endingCnt - count unique endings across all modes
    if let Some(record) = data.get("record") {
        // Count endings from endingCnt
        if let Some(ending_cnt) = record.get("endingCnt")
            && let Some(obj) = ending_cnt.as_object()
        {
            let mut unique_endings = std::collections::HashSet::new();
            for (_mode, endings) in obj {
                if let Some(endings_obj) = endings.as_object() {
                    for (ending_id, _count) in endings_obj {
                        unique_endings.insert(ending_id.clone());
                    }
                }
            }
            details.endings_unlocked = unique_endings.len() as i32;
        }

        // Parse modeCnt for run counts
        if let Some(mode_cnt) = record.get("modeCnt")
            && let Some(obj) = mode_cnt.as_object()
        {
            details.normal_runs = obj.get("NORMAL").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            details.challenge_runs =
                obj.get("CHALLENGE").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            details.month_team_runs =
                obj.get("MONTH_TEAM").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        }
    }

    // Parse bp.reward to count BP levels
    if let Some(bp) = data.get("bp")
        && let Some(reward) = bp.get("reward")
        && let Some(obj) = reward.as_object()
    {
        // Count bp_level_* entries
        details.bp_level = obj.keys().filter(|k| k.starts_with("bp_level_")).count() as i32;
    }

    // Parse buff data
    if let Some(buff) = data.get("buff") {
        details.total_accumulated_score = buff.get("score").and_then(|v| v.as_i64()).unwrap_or(0);

        if let Some(unlocked) = buff.get("unlocked")
            && let Some(obj) = unlocked.as_object()
        {
            details.buffs_unlocked = obj.len() as i32;
        }
    }

    // Parse collectibles
    if let Some(collect) = data.get("collect") {
        // Count bands with state > 0
        if let Some(band) = collect.get("band")
            && let Some(obj) = band.as_object()
        {
            details.bands_unlocked = obj
                .values()
                .filter(|v| {
                    v.get("state")
                        .and_then(|s| s.as_i64())
                        .map(|s| s > 0)
                        .unwrap_or(false)
                })
                .count() as i32;
        }

        // Count relics
        if let Some(relic) = collect.get("relic")
            && let Some(obj) = relic.as_object()
        {
            details.relics_unlocked = obj
                .values()
                .filter(|v| {
                    v.get("state")
                        .and_then(|s| s.as_i64())
                        .map(|s| s > 0)
                        .unwrap_or(false)
                })
                .count() as i32;
        }

        // Count capsules
        if let Some(capsule) = collect.get("capsule")
            && let Some(obj) = capsule.as_object()
        {
            details.capsules_unlocked = obj
                .values()
                .filter(|v| {
                    v.get("state")
                        .and_then(|s| s.as_i64())
                        .map(|s| s > 0)
                        .unwrap_or(false)
                })
                .count() as i32;
        }
    }

    // Parse challenge data
    if let Some(challenge) = data.get("challenge")
        && let Some(grade) = challenge.get("grade")
        && let Some(obj) = grade.as_object()
    {
        for grade_val in obj.values() {
            if let Some(g) = grade_val.as_i64() {
                if g > 0 {
                    details.challenge_grades_achieved += 1;
                }
                if g >= 2 {
                    details.grade_2_challenges += 1;
                }
                if g > details.highest_challenge_grade as i64 {
                    details.highest_challenge_grade = g as i32;
                }
            }
        }
    }

    // Check if theme has been played
    let total_runs = details.normal_runs + details.challenge_runs + details.month_team_runs;
    if total_runs == 0 && details.bp_level == 0 && details.buffs_unlocked == 0 {
        return None; // Theme not played
    }

    // Populate max values from game data
    if let Some(gd) = theme_game_data {
        details.max_endings = gd.max_endings;
        details.max_relics = gd.max_relics;
        details.max_capsules = gd.max_capsules;
        details.max_bands = gd.max_bands;
        details.max_challenges = gd.max_challenges;
        details.max_monthly_squads = gd.max_monthly_squads;
        details.max_difficulty_grade = gd.max_difficulty_grade;

        // Calculate completion percentages
        details.endings_completion_percentage =
            calculate_percentage(details.endings_unlocked, details.max_endings);
        details.relics_completion_percentage =
            calculate_percentage(details.relics_unlocked, details.max_relics);
        details.capsules_completion_percentage =
            calculate_percentage(details.capsules_unlocked, details.max_capsules);
        details.bands_completion_percentage =
            calculate_percentage(details.bands_unlocked, details.max_bands);

        // Total collectibles completion
        let total_unlocked =
            details.relics_unlocked + details.capsules_unlocked + details.bands_unlocked;
        let total_max = details.max_relics + details.max_capsules + details.max_bands;
        details.collectibles_completion_percentage =
            calculate_percentage(total_unlocked, total_max);

        // Challenges at max grade (grade 2) completion
        details.challenges_at_max_grade_percentage =
            calculate_percentage(details.grade_2_challenges, details.max_challenges);

        // Overall theme completion (weighted average)
        // Weight: endings (30%), collectibles (50%), challenges (20%)
        let endings_weight = 0.30;
        let collectibles_weight = 0.50;
        let challenges_weight = 0.20;
        details.overall_completion_percentage = (details.endings_completion_percentage
            * endings_weight)
            + (details.collectibles_completion_percentage * collectibles_weight)
            + (details.challenges_at_max_grade_percentage * challenges_weight);
    }

    // Calculate scores
    let base_score = points::THEME_PLAYED;
    let endings_score = details.endings_unlocked as f32 * points::ENDING_UNLOCKED;
    let bp_score = details.bp_level as f32 * points::BP_LEVEL;
    let buffs_score = details.buffs_unlocked as f32 * points::BUFF_UNLOCKED;
    let collectibles_score = (details.bands_unlocked as f32 * points::BAND_UNLOCKED)
        + (details.relics_unlocked as f32 * points::RELIC_UNLOCKED)
        + (details.capsules_unlocked as f32 * points::CAPSULE_UNLOCKED);
    let challenge_score = details.challenge_grades_achieved as f32 * points::CHALLENGE_GRADE;

    // Difficulty score: bonus for grade 2 clears + bonus for having any grade 2 on this theme
    let difficulty_score = (details.grade_2_challenges as f32 * points::GRADE_2_BONUS)
        + if details.highest_challenge_grade >= 2 {
            points::MAX_DIFFICULTY_THEME
        } else {
            0.0
        };

    let total_score = base_score
        + endings_score
        + bp_score
        + buffs_score
        + collectibles_score
        + challenge_score
        + difficulty_score;

    Some(RoguelikeThemeScore {
        theme_id: theme_id.to_string(),
        total_score,
        endings_score,
        bp_score,
        buffs_score,
        collectibles_score,
        challenge_score,
        difficulty_score,
        details,
    })
}
