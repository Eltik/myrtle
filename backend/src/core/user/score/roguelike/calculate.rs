//! Roguelike score calculation

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
}

/// Calculate roguelike score for a user
pub fn calculate_roguelike_score(user: &User) -> RoguelikeScore {
    let mut theme_scores = Vec::new();
    let mut breakdown = RoguelikeBreakdown::default();

    // Process each theme (rogue_1 through rogue_5)
    for i in 1..=5 {
        let theme_id = format!("rogue_{}", i);
        if let Some(theme_data) = user.rlv2.outer.get(&theme_id)
            && let Some(theme_score) = calculate_theme_score(&theme_id, theme_data)
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

            theme_scores.push(theme_score);
        }
    }

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

fn calculate_theme_score(theme_id: &str, data: &serde_json::Value) -> Option<RoguelikeThemeScore> {
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
        // Count challenges with grade > 0
        details.challenge_grades_achieved = obj
            .values()
            .filter(|v| v.as_i64().map(|g| g > 0).unwrap_or(false))
            .count() as i32;

        // Find highest grade
        details.highest_challenge_grade =
            obj.values().filter_map(|v| v.as_i64()).max().unwrap_or(0) as i32;
    }

    // Check if theme has been played
    let total_runs = details.normal_runs + details.challenge_runs + details.month_team_runs;
    if total_runs == 0 && details.bp_level == 0 && details.buffs_unlocked == 0 {
        return None; // Theme not played
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

    let total_score =
        base_score + endings_score + bp_score + buffs_score + collectibles_score + challenge_score;

    Some(RoguelikeThemeScore {
        theme_id: theme_id.to_string(),
        total_score,
        endings_score,
        bp_score,
        buffs_score,
        collectibles_score,
        challenge_score,
        details,
    })
}
