//! Score Calculation Tests
//!
//! Tests the user account scoring system against real user data.
//!
//! Run with: cargo test --test score_calculation_test
//!
//! Requires:
//!   - DATA_DIR environment variable pointing to game data
//!   - tests/score_calculation/user_eltik.json file

use std::sync::OnceLock;

use backend::core::local::handler::init_game_data;
use backend::core::local::types::GameData;
use backend::core::user::score::{CompletionStatus, calculate_user_score};
use backend::core::user::types::User as UserData;
use backend::database::models::user::User as DatabaseUser;
use backend::events::EventEmitter;

/// Loaded game data for scoring
static GAME_DATA: OnceLock<Option<GameData>> = OnceLock::new();

/// Loaded user data for testing
static USER_DATA: OnceLock<Option<UserData>> = OnceLock::new();

fn get_game_data() -> Option<&'static GameData> {
    GAME_DATA
        .get_or_init(|| {
            let data_dir = std::env::var("DATA_DIR").ok()?;
            let assets_dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "./assets".to_string());

            let events = std::sync::Arc::new(EventEmitter::new());

            match init_game_data(
                std::path::Path::new(&data_dir),
                std::path::Path::new(&assets_dir),
                &events,
            ) {
                Ok(data) => Some(data),
                Err(e) => {
                    eprintln!("Failed to load game data: {e:?}");
                    None
                }
            }
        })
        .as_ref()
}

fn get_user_data() -> Option<&'static UserData> {
    USER_DATA
        .get_or_init(|| {
            let user_path = std::env::var("USER_DATA_PATH")
                .unwrap_or_else(|_| "tests/score_calculation/user_eltik.json".to_string());

            match std::fs::read_to_string(&user_path) {
                Ok(content) => {
                    // Try parsing as DatabaseUser first (full row with id, uid, server, data)
                    if let Ok(db_user) = serde_json::from_str::<DatabaseUser>(&content) {
                        // DatabaseUser.data is serde_json::Value, need to convert to UserData
                        match serde_json::from_value::<UserData>(db_user.data) {
                            Ok(user_data) => return Some(user_data),
                            Err(e) => {
                                eprintln!("Failed to parse user data from DatabaseUser: {e}");
                            }
                        }
                    }
                    // Fall back to parsing as direct UserData
                    match serde_json::from_str::<UserData>(&content) {
                        Ok(user) => Some(user),
                        Err(e) => {
                            eprintln!("Failed to parse user data: {e}");
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("User data file not found at {user_path}: {e}");
                    None
                }
            }
        })
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that user data file exists and can be loaded
    #[test]
    fn test_user_data_loaded() {
        let user = get_user_data();
        assert!(user.is_some(), "User data should be loadable");

        if let Some(user) = user {
            println!("Loaded user: {}", user.status.nick_name);
            println!("User level: {}", user.status.level);
            println!("Total characters in roster: {}", user.troop.chars.len());
        }
    }

    /// Test that game data can be loaded
    #[test]
    fn test_game_data_loaded() {
        let game_data = get_game_data();

        if game_data.is_none() {
            println!("DATA_DIR not set, skipping game data test");
            println!("Set DATA_DIR environment variable to enable full tests");
            return;
        }

        let game_data = game_data.unwrap();
        println!(
            "Loaded {} operators from game data",
            game_data.operators.len()
        );
        assert!(
            !game_data.operators.is_empty(),
            "Game data should have operators"
        );
    }

    /// Main test: calculate score for user Eltik
    #[test]
    fn test_calculate_user_score() {
        let Some(game_data) = get_game_data() else {
            println!("DATA_DIR not set, skipping score calculation test");
            return;
        };

        let Some(user) = get_user_data() else {
            println!("User data not loaded, skipping score calculation test");
            return;
        };

        // Calculate score
        let score = calculate_user_score(user, game_data);

        // Print results
        println!();
        println!(
            "=== Score Calculation Results for {} ===",
            user.status.nick_name
        );
        println!("Total Score: {:.2}", score.total_score);
        println!();
        println!("Breakdown:");
        println!("  Total Operators: {}", score.breakdown.total_operators);
        println!("  6-Star Count: {}", score.breakdown.six_star_count);
        println!("  5-Star Count: {}", score.breakdown.five_star_count);
        println!("  4-Star Count: {}", score.breakdown.four_star_count);
        println!(
            "  3-Star and Below: {}",
            score.breakdown.three_star_and_below_count
        );
        println!("  E2 Count: {}", score.breakdown.e2_count);
        println!("  M3+ Operators: {}", score.breakdown.m3_count);
        println!("  M9 Operators: {}", score.breakdown.m9_count);
        println!(
            "  Average Score: {:.2}",
            score.breakdown.average_score_per_operator
        );

        // Basic assertions
        assert!(score.total_score > 0.0, "Total score should be positive");
        assert!(score.breakdown.total_operators > 0, "Should have operators");
        assert!(
            score.operator_scores.len() == score.breakdown.total_operators as usize,
            "Operator scores count should match breakdown"
        );

        // Print top 10 operators by score
        println!();
        println!("Top 10 Operators by Score:");
        for (i, op) in score.operator_scores.iter().take(10).enumerate() {
            println!(
                "  {}. {} ({}-star): {:.2} pts [{:?}]",
                i + 1,
                op.name,
                op.rarity,
                op.total_score,
                op.completion_status
            );
            println!(
                "      Base: {:.0}, Level: {:.0}, Potential: {:.0}, Mastery: {:.0}, Module: {:.0}",
                op.base_score,
                op.level_score,
                op.potential_score,
                op.mastery_score,
                op.module_score
            );
        }

        // Verify M9 operators are marked as AbsolutelyCompleted
        let m9_operators: Vec<_> = score
            .operator_scores
            .iter()
            .filter(|op| op.mastery_details.m3_count >= 3)
            .collect();

        println!();
        println!("M9 Operators ({}):", m9_operators.len());
        for op in &m9_operators {
            println!("  - {} [{:?}]", op.name, op.completion_status);
            assert_eq!(
                op.completion_status,
                CompletionStatus::AbsolutelyCompleted,
                "M9 operator {} should be AbsolutelyCompleted",
                op.name
            );
        }

        // Verify the M9 count matches
        assert_eq!(
            m9_operators.len(),
            score.breakdown.m9_count as usize,
            "M9 count in breakdown should match actual M9 operators"
        );
    }

    /// Test that tokens and traps are excluded from scoring
    #[test]
    fn test_tokens_excluded() {
        let Some(game_data) = get_game_data() else {
            println!("DATA_DIR not set, skipping token exclusion test");
            return;
        };

        let Some(user) = get_user_data() else {
            println!("User data not loaded, skipping token exclusion test");
            return;
        };

        let score = calculate_user_score(user, game_data);

        // Check that no scored operators have token_ or trap_ prefixes
        for op in &score.operator_scores {
            assert!(
                !op.char_id.starts_with("token_"),
                "Token {} should not be scored",
                op.char_id
            );
            assert!(
                !op.char_id.starts_with("trap_"),
                "Trap {} should not be scored",
                op.char_id
            );
        }

        println!(
            "Verified: No tokens or traps in {} scored operators",
            score.operator_scores.len()
        );
    }

    /// Test score components are reasonable
    #[test]
    fn test_score_components_reasonable() {
        let Some(game_data) = get_game_data() else {
            println!("DATA_DIR not set, skipping score components test");
            return;
        };

        let Some(user) = get_user_data() else {
            println!("User data not loaded, skipping score components test");
            return;
        };

        let score = calculate_user_score(user, game_data);

        for op in &score.operator_scores {
            // All score components should be non-negative
            assert!(
                op.base_score >= 0.0,
                "Base score should be non-negative for {}",
                op.name
            );
            assert!(
                op.level_score >= 0.0,
                "Level score should be non-negative for {}",
                op.name
            );
            assert!(
                op.potential_score >= 0.0,
                "Potential score should be non-negative for {}",
                op.name
            );
            assert!(
                op.mastery_score >= 0.0,
                "Mastery score should be non-negative for {}",
                op.name
            );
            assert!(
                op.module_score >= 0.0,
                "Module score should be non-negative for {}",
                op.name
            );

            // Total should equal sum of components
            let expected_total = op.base_score
                + op.level_score
                + op.potential_score
                + op.mastery_score
                + op.module_score;
            assert!(
                (op.total_score - expected_total).abs() < 0.01,
                "Total score should equal sum of components for {}",
                op.name
            );

            // Base score should match rarity
            let expected_base = match op.rarity {
                6 => 500.0,
                5 => 400.0,
                4 => 150.0,
                3 => 30.0,
                2 => 10.0,
                1 => 5.0,
                _ => 0.0,
            };
            assert_eq!(
                op.base_score, expected_base,
                "Base score should match rarity for {}",
                op.name
            );
        }

        println!(
            "Verified: All score components are reasonable for {} operators",
            score.operator_scores.len()
        );
    }
}
