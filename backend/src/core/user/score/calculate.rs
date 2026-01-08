//! General user account score calculations

use crate::core::local::types::GameData;
use crate::core::user::types::User;

use super::operators::{
    calculate_operator_score,
    helpers::is_token_or_trap,
    helpers::rarity_to_int,
    types::{OperatorScore, ScoreBreakdown, UserScore},
};

/// Calculate the total score for a user's account
///
/// Iterates through all operators in the user's roster, excluding tokens and traps,
/// and calculates individual scores plus aggregate statistics.
pub fn calculate_user_score(user: &User, game_data: &GameData) -> UserScore {
    let mut operator_scores: Vec<OperatorScore> = Vec::new();
    let mut breakdown = ScoreBreakdown::default();
    let mut total_score = 0.0;

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

        total_score += score.total_score;
        operator_scores.push(score);
    }

    // Calculate average score
    if breakdown.total_operators > 0 {
        breakdown.average_score_per_operator = total_score / breakdown.total_operators as f32;
    }

    // Sort operator scores by total score descending
    operator_scores.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());

    UserScore {
        total_score,
        operator_scores,
        breakdown,
    }
}
