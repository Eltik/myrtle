//! General user account score calculations

use crate::core::local::types::GameData;
use crate::core::user::types::User;

use super::operators::{
    calculate_operator_score, helpers::is_token_or_trap, helpers::rarity_to_int,
    types::OperatorScore,
};
use super::roguelike::calculate_roguelike_score;
use super::sandbox::calculate_sandbox_score;
use super::stages::calculate_stage_score;
use super::types::{ScoreBreakdown, UserScore};

/// Calculate the total score for a user's account
///
/// Iterates through all operators in the user's roster, excluding tokens and traps,
/// and calculates individual scores plus aggregate statistics. Also calculates
/// stage completion scores across all zones.
pub fn calculate_user_score(user: &User, game_data: &GameData) -> UserScore {
    let mut operator_scores: Vec<OperatorScore> = Vec::new();
    let mut breakdown = ScoreBreakdown::default();
    let mut operator_total = 0.0;

    // Get user's owned skins
    let user_skins = &user.skin.character_skins;

    for character in user.troop.chars.values() {
        // Look up operator data from game_data
        let operator = match game_data.operators.get(&character.char_id) {
            Some(op) => op,
            None => continue, // Skip if operator not found in game data
        };

        // Skip tokens and traps
        if is_token_or_trap(&character.char_id, operator) {
            continue;
        }

        // Calculate individual operator score
        let score = calculate_operator_score(
            character,
            &operator.name,
            &operator.rarity,
            &game_data.favor,
            user_skins,
            &game_data.skins,
        );

        // Update breakdown statistics
        breakdown.total_operators += 1;

        let rarity_int = rarity_to_int(&operator.rarity);
        match rarity_int {
            6 => breakdown.six_star_count += 1,
            5 => breakdown.five_star_count += 1,
            4 => breakdown.four_star_count += 1,
            _ => breakdown.three_star_and_below_count += 1,
        }

        // Count M9 and M3+ operators
        if score.mastery_details.m3_count >= 3 {
            breakdown.m9_count += 1;
        }
        if score.mastery_details.m3_count >= 1 {
            breakdown.m3_count += 1;
        }

        // Count E2 operators
        if character.evolve_phase >= 2 {
            breakdown.e2_count += 1;
        }

        // Count skin statistics
        breakdown.total_skins_owned += score.skin_details.owned_count;
        if score.skin_details.total_available > 0
            && score.skin_details.completion_percentage >= 100.0
        {
            breakdown.full_skin_collection_count += 1;
        }

        operator_total += score.total_score;
        operator_scores.push(score);
    }

    // Calculate average score
    if breakdown.total_operators > 0 {
        breakdown.average_score_per_operator = operator_total / breakdown.total_operators as f32;
    }

    // Sort operator scores by total score descending
    operator_scores.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());

    // === STAGE SCORING ===
    let stage_result = calculate_stage_score(user, game_data);

    // Merge stage breakdown into main breakdown
    breakdown.mainline_completion = stage_result.breakdown.mainline_completion;
    breakdown.sidestory_completion = stage_result.breakdown.sidestory_completion;
    breakdown.activity_completion = stage_result.breakdown.activity_completion;
    breakdown.total_stages_completed = stage_result.breakdown.total_stages_completed;
    breakdown.total_stages_available = stage_result.breakdown.total_stages_available;
    breakdown.total_perfect_clears = stage_result.breakdown.total_perfect_clears;

    // === ROGUELIKE (INTEGRATED STRATEGIES) SCORING ===
    let roguelike_result = calculate_roguelike_score(user);

    // Merge roguelike breakdown into main breakdown
    breakdown.roguelike_themes_played = roguelike_result.breakdown.themes_played;
    breakdown.roguelike_total_endings = roguelike_result.breakdown.total_endings;
    breakdown.roguelike_total_bp_levels = roguelike_result.breakdown.total_bp_levels;
    breakdown.roguelike_total_buffs = roguelike_result.breakdown.total_buffs;
    breakdown.roguelike_total_collectibles = roguelike_result.breakdown.total_collectibles;
    breakdown.roguelike_total_runs = roguelike_result.breakdown.total_runs;
    breakdown.roguelike_grade_2_challenges = roguelike_result.breakdown.total_grade_2_challenges;
    breakdown.roguelike_themes_at_max_difficulty =
        roguelike_result.breakdown.themes_at_max_difficulty;

    // === SANDBOX (RECLAMATION ALGORITHM) SCORING ===
    let sandbox_result = calculate_sandbox_score(user);

    // Merge sandbox breakdown into main breakdown
    breakdown.sandbox_places_completed = sandbox_result.breakdown.places_completed;
    breakdown.sandbox_places_discovered = sandbox_result.breakdown.places_discovered;
    breakdown.sandbox_places_total = sandbox_result.breakdown.places_total;
    breakdown.sandbox_completion_percentage = sandbox_result.breakdown.places_completion_percentage;
    breakdown.sandbox_nodes_completed = sandbox_result.breakdown.total_nodes_completed;
    breakdown.sandbox_landmark_nodes = sandbox_result.breakdown.landmark_nodes_completed;
    breakdown.sandbox_special_nodes = sandbox_result.breakdown.special_nodes_completed;
    breakdown.sandbox_tech_trees_completed = sandbox_result.breakdown.tech_trees_completed;
    breakdown.sandbox_stories_unlocked = sandbox_result.breakdown.stories_unlocked;
    breakdown.sandbox_events_completed = sandbox_result.breakdown.events_completed;
    breakdown.sandbox_log_entries = sandbox_result.breakdown.log_entries_collected;
    breakdown.sandbox_chapters_with_logs = sandbox_result.breakdown.chapters_with_logs;

    // Combined total score
    let total_score = operator_total
        + stage_result.total_score
        + roguelike_result.total_score
        + sandbox_result.total_score;

    UserScore {
        total_score,
        operator_score: operator_total,
        stage_score: stage_result.total_score,
        roguelike_score: roguelike_result.total_score,
        sandbox_score: sandbox_result.total_score,
        operator_scores,
        zone_scores: stage_result.zone_scores,
        roguelike_theme_scores: roguelike_result.theme_scores.clone(),
        sandbox_area_scores: sandbox_result.area_scores.clone(),
        roguelike_details: roguelike_result,
        sandbox_details: sandbox_result,
        breakdown,
    }
}
