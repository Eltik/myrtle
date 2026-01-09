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
        println!("  Total Skins Owned: {}", score.breakdown.total_skins_owned);
        println!(
            "  Full Skin Collections: {}",
            score.breakdown.full_skin_collection_count
        );
        println!(
            "  Average Score: {:.2}",
            score.breakdown.average_score_per_operator
        );

        // Stage completion stats
        println!();
        println!("Stage Completion:");
        println!("  Operator Score: {:.2}", score.operator_score);
        println!("  Stage Score: {:.2}", score.stage_score);
        println!(
            "  Mainline Completion: {:.1}%",
            score.breakdown.mainline_completion
        );
        println!(
            "  Sidestory Completion: {:.1}%",
            score.breakdown.sidestory_completion
        );
        println!(
            "  Activity Completion: {:.1}%",
            score.breakdown.activity_completion
        );
        println!(
            "  Stages Completed: {}/{}",
            score.breakdown.total_stages_completed, score.breakdown.total_stages_available
        );
        println!("  Perfect Clears: {}", score.breakdown.total_perfect_clears);

        // Print top zones by score
        println!();
        println!("Top 10 Zones by Score:");
        for (i, zone) in score.zone_scores.iter().take(10).enumerate() {
            println!(
                "  {}. {} ({:?}): {:.2} pts ({}/{} = {:.1}%)",
                i + 1,
                zone.zone_name,
                zone.zone_type,
                zone.score,
                zone.completed_stages,
                zone.total_stages,
                zone.completion_percentage
            );
        }

        // Roguelike (Integrated Strategies) stats
        println!();
        println!("Roguelike (Integrated Strategies):");
        println!("  Roguelike Score: {:.2}", score.roguelike_score);
        println!(
            "  Themes Played: {}",
            score.breakdown.roguelike_themes_played
        );
        println!(
            "  Total Endings: {}",
            score.breakdown.roguelike_total_endings
        );
        println!(
            "  Total BP Levels: {}",
            score.breakdown.roguelike_total_bp_levels
        );
        println!("  Total Buffs: {}", score.breakdown.roguelike_total_buffs);
        println!(
            "  Total Collectibles: {}",
            score.breakdown.roguelike_total_collectibles
        );
        println!("  Total Runs: {}", score.breakdown.roguelike_total_runs);
        println!(
            "  Grade 2 (Max Difficulty) Challenges: {}",
            score.breakdown.roguelike_grade_2_challenges
        );
        println!(
            "  Themes at Max Difficulty: {}",
            score.breakdown.roguelike_themes_at_max_difficulty
        );

        // Print roguelike theme scores
        if !score.roguelike_theme_scores.is_empty() {
            println!();
            println!("Roguelike Theme Scores:");
            for theme in &score.roguelike_theme_scores {
                println!("  {}: {:.2} pts", theme.theme_id, theme.total_score);
                println!(
                    "    Endings: {} ({:.0}), BP: {} ({:.0}), Buffs: {} ({:.0})",
                    theme.details.endings_unlocked,
                    theme.endings_score,
                    theme.details.bp_level,
                    theme.bp_score,
                    theme.details.buffs_unlocked,
                    theme.buffs_score
                );
                println!(
                    "    Collectibles: {} bands, {} relics, {} capsules ({:.0})",
                    theme.details.bands_unlocked,
                    theme.details.relics_unlocked,
                    theme.details.capsules_unlocked,
                    theme.collectibles_score
                );
                println!(
                    "    Runs: {} normal, {} challenge, {} monthly ({} total)",
                    theme.details.normal_runs,
                    theme.details.challenge_runs,
                    theme.details.month_team_runs,
                    theme.details.normal_runs
                        + theme.details.challenge_runs
                        + theme.details.month_team_runs
                );
                println!(
                    "    Difficulty: {} grade 2 clears, highest grade {} ({:.0} pts)",
                    theme.details.grade_2_challenges,
                    theme.details.highest_challenge_grade,
                    theme.difficulty_score
                );
            }
        }

        // Sandbox (Reclamation Algorithm) stats
        println!();
        println!("Sandbox (Reclamation Algorithm):");
        println!("  Sandbox Score: {:.2}", score.sandbox_score);
        println!(
            "  Places: {}/{} completed, {} discovered ({:.1}%)",
            score.breakdown.sandbox_places_completed,
            score.breakdown.sandbox_places_total,
            score.breakdown.sandbox_places_discovered,
            score.breakdown.sandbox_completion_percentage
        );
        println!(
            "  Nodes Completed: {} (landmarks: {}, special: {})",
            score.breakdown.sandbox_nodes_completed,
            score.breakdown.sandbox_landmark_nodes,
            score.breakdown.sandbox_special_nodes
        );
        println!(
            "  Tech Trees: {}",
            score.breakdown.sandbox_tech_trees_completed
        );
        println!(
            "  Stories Unlocked: {}",
            score.breakdown.sandbox_stories_unlocked
        );
        println!(
            "  Events Completed: {}",
            score.breakdown.sandbox_events_completed
        );
        println!(
            "  Log Entries: {} ({} chapters with logs)",
            score.breakdown.sandbox_log_entries, score.breakdown.sandbox_chapters_with_logs
        );

        // Print sandbox score breakdown
        println!();
        println!("Sandbox Score Breakdown:");
        println!(
            "  Places: {:.0}, Nodes: {:.0}, Tech Trees: {:.0}",
            score.sandbox_details.places_score,
            score.sandbox_details.nodes_score,
            score.sandbox_details.tech_tree_score
        );
        println!(
            "  Stories: {:.0}, Events: {:.0}, Logs: {:.0}",
            score.sandbox_details.stories_score,
            score.sandbox_details.events_score,
            score.sandbox_details.logs_score
        );

        // Print sandbox area scores
        if !score.sandbox_area_scores.is_empty() {
            println!();
            println!("Sandbox Area Scores:");
            for area in &score.sandbox_area_scores {
                println!(
                    "  Area {}: {:.2} pts ({}/{} places, {} nodes)",
                    area.area,
                    area.total_score,
                    area.places_completed,
                    area.places_total,
                    area.nodes_completed
                );
            }
        }

        // Medal stats
        println!();
        println!("Medals:");
        println!("  Medal Score: {:.2}", score.medal_score);
        println!(
            "  Medals Earned: {}/{} ({:.1}%)",
            score.breakdown.medal_total_earned,
            score.breakdown.medal_total_available,
            score.breakdown.medal_completion_percentage
        );
        println!(
            "  By Rarity: T1={}, T2={}, T3={}, T2D5={}",
            score.breakdown.medal_t1_earned,
            score.breakdown.medal_t2_earned,
            score.breakdown.medal_t3_earned,
            score.breakdown.medal_t2d5_earned
        );
        println!(
            "  Groups Complete: {}",
            score.breakdown.medal_groups_complete
        );

        // Print medal score breakdown
        println!();
        println!("Medal Score Breakdown:");
        println!(
            "  Rarity Score: {:.0}, Category Bonus: {:.0}, Group Bonus: {:.0}",
            score.medal_details.rarity_score,
            score.medal_details.category_bonus_score,
            score.medal_details.group_bonus_score
        );

        // Print top medal categories by score
        if !score.medal_category_scores.is_empty() {
            println!();
            println!("Medal Category Scores:");
            for cat in &score.medal_category_scores {
                println!(
                    "  {}: {:.2} pts ({}/{} = {:.1}%)",
                    cat.category_name,
                    cat.total_score,
                    cat.medals_earned,
                    cat.medals_available,
                    cat.completion_percentage
                );
            }
        }

        // Base (RIIC) Efficiency stats
        println!();
        println!("Base (RIIC) Efficiency:");
        println!("  Base Score: {:.2}", score.base_score);
        println!(
            "  Trading Posts: {} (avg {:.1}% efficiency)",
            score.breakdown.base_trading_post_count, score.breakdown.base_avg_trading_efficiency
        );
        println!(
            "  Factories: {} (avg {:.1}% efficiency)",
            score.breakdown.base_factory_count, score.breakdown.base_avg_factory_efficiency
        );
        println!("  Power Plants: {}", score.breakdown.base_power_plant_count);
        println!(
            "  Dormitories: {} (total comfort: {})",
            score.breakdown.base_dormitory_count, score.breakdown.base_total_comfort
        );
        println!(
            "  Electricity Balance: {}",
            score.breakdown.base_electricity_balance
        );
        println!(
            "  Max Level Buildings: {}",
            score.breakdown.base_max_level_buildings
        );

        // Print base score breakdown
        println!();
        println!("Base Score Breakdown:");
        println!(
            "  Trading: {:.0}, Factory: {:.0}, Power: {:.0}, Dormitory: {:.0}",
            score.base_details.trading_score,
            score.base_details.factory_score,
            score.base_details.power_score,
            score.base_details.dormitory_score
        );
        println!(
            "  Control Center: {:.0}, Reception: {:.0}, Office: {:.0}",
            score.base_details.control_center_score,
            score.base_details.reception_score,
            score.base_details.office_score
        );
        println!(
            "  Global Bonuses: {:.0}",
            score.base_details.global_bonus_score
        );

        // Print trading post details
        if !score.base_details.trading_posts.is_empty() {
            println!();
            println!("Trading Post Scores:");
            for tp in &score.base_details.trading_posts {
                println!(
                    "  {}: {:.2} pts (level {}, {:.0}% efficiency, {} orders)",
                    tp.slot_id,
                    tp.total_score,
                    tp.level,
                    tp.details.speed_multiplier * 100.0,
                    tp.details.stock_limit
                );
                if tp.details.preset_count > 0 {
                    let ops_per_preset: Vec<String> = tp
                        .details
                        .operators_per_preset
                        .iter()
                        .map(|n| n.to_string())
                        .collect();
                    println!(
                        "    Presets: {} rotations [{}] (+{:.0} pts)",
                        tp.details.preset_count,
                        ops_per_preset.join(", "),
                        tp.preset_score
                    );
                }
            }
        }

        // Print factory details
        if !score.base_details.factories.is_empty() {
            println!();
            println!("Factory Scores:");
            for f in &score.base_details.factories {
                println!(
                    "  {}: {:.2} pts (level {}, {:.0}% efficiency, {})",
                    f.slot_id,
                    f.total_score,
                    f.level,
                    f.details.speed_multiplier * 100.0,
                    f.details.production_type
                );
                if f.details.preset_count > 0 {
                    let ops_per_preset: Vec<String> = f
                        .details
                        .operators_per_preset
                        .iter()
                        .map(|n| n.to_string())
                        .collect();
                    println!(
                        "    Presets: {} rotations [{}] (+{:.0} pts)",
                        f.details.preset_count,
                        ops_per_preset.join(", "),
                        f.preset_score
                    );
                }
            }
        }

        // Print dormitory details
        if !score.base_details.dormitories.is_empty() {
            println!();
            println!("Dormitory Scores:");
            for d in &score.base_details.dormitories {
                println!(
                    "  {}: {:.2} pts (level {}, comfort {}/5000 = {:.1}%)",
                    d.slot_id,
                    d.total_score,
                    d.level,
                    d.details.comfort_level,
                    d.details.comfort_percentage
                );
            }
        }

        // Print control center details
        if let Some(ref cc) = score.base_details.control_center {
            println!();
            println!("Control Center:");
            println!(
                "  {}: {:.2} pts (level {}, {} operators)",
                cc.slot_id, cc.total_score, cc.level, cc.details.operators_stationed
            );
            println!(
                "  Buffs: Trading {:.2}%, Manufacture {:.2}%",
                cc.details.trading_buff, cc.details.manufacture_buff
            );
            println!("  AP Cost Reduction: {}", cc.details.ap_cost_reduction);
            if cc.details.preset_count > 0 {
                let ops_per_preset: Vec<String> = cc
                    .details
                    .operators_per_preset
                    .iter()
                    .map(|n| n.to_string())
                    .collect();
                println!(
                    "  Presets: {} rotations [{}]",
                    cc.details.preset_count,
                    ops_per_preset.join(", ")
                );
            }
        }

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
                "      Base: {:.0}, Level: {:.0}, Trust: {:.1}, Potential: {:.0}, Mastery: {:.0}, Module: {:.0}, Skin: {:.0}",
                op.base_score,
                op.level_score,
                op.trust_score,
                op.potential_score,
                op.mastery_score,
                op.module_score,
                op.skin_score
            );
            if op.skin_details.total_available > 0 {
                println!(
                    "      Skins: {}/{} ({:.0}% complete) [L2D: {}, Store: {}, Event: {}]",
                    op.skin_details.owned_count,
                    op.skin_details.total_available,
                    op.skin_details.completion_percentage,
                    op.skin_details.owned_l2d,
                    op.skin_details.owned_store,
                    op.skin_details.owned_event
                );
            }
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
                op.trust_score >= 0.0 && op.trust_score <= 50.0,
                "Trust score should be between 0 and 50 for {}",
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
            assert!(
                op.skin_score >= 0.0,
                "Skin score should be non-negative for {}",
                op.name
            );

            // Total should equal sum of components
            let expected_total = op.base_score
                + op.level_score
                + op.trust_score
                + op.potential_score
                + op.mastery_score
                + op.module_score
                + op.skin_score;
            assert!(
                (op.total_score - expected_total).abs() < 0.01,
                "Total score should equal sum of components for {} (expected {:.2}, got {:.2})",
                op.name,
                expected_total,
                op.total_score
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

    /// Export scores to JSON file for review
    /// Run with: cargo test --test score_calculation_test test_export_scores -- --nocapture
    #[test]
    fn test_export_scores() {
        let Some(game_data) = get_game_data() else {
            println!("DATA_DIR not set, skipping export test");
            return;
        };

        let Some(user) = get_user_data() else {
            println!("User data not loaded, skipping export test");
            return;
        };

        let score = calculate_user_score(user, game_data);

        // Create output directory if it doesn't exist
        let output_dir = std::path::Path::new("tests/score_calculation/output");
        std::fs::create_dir_all(output_dir).expect("Failed to create output directory");

        // Export full score to JSON
        let full_json = serde_json::to_string_pretty(&score).expect("Failed to serialize score");
        let full_path = output_dir.join("user_score_full.json");
        std::fs::write(&full_path, &full_json).expect("Failed to write full score JSON");
        println!("Exported full score to: {}", full_path.display());

        // Export just operator scores (sorted by total score)
        let operators_json = serde_json::to_string_pretty(&score.operator_scores)
            .expect("Failed to serialize operators");
        let operators_path = output_dir.join("operator_scores.json");
        std::fs::write(&operators_path, &operators_json)
            .expect("Failed to write operator scores JSON");
        println!("Exported operator scores to: {}", operators_path.display());

        // Export just zone scores
        let zones_json =
            serde_json::to_string_pretty(&score.zone_scores).expect("Failed to serialize zones");
        let zones_path = output_dir.join("zone_scores.json");
        std::fs::write(&zones_path, &zones_json).expect("Failed to write zone scores JSON");
        println!("Exported zone scores to: {}", zones_path.display());

        // Export breakdown summary
        let breakdown_json =
            serde_json::to_string_pretty(&score.breakdown).expect("Failed to serialize breakdown");
        let breakdown_path = output_dir.join("score_breakdown.json");
        std::fs::write(&breakdown_path, &breakdown_json).expect("Failed to write breakdown JSON");
        println!("Exported score breakdown to: {}", breakdown_path.display());

        // Export roguelike scores
        let roguelike_json = serde_json::to_string_pretty(&score.roguelike_details)
            .expect("Failed to serialize roguelike");
        let roguelike_path = output_dir.join("roguelike_scores.json");
        std::fs::write(&roguelike_path, &roguelike_json)
            .expect("Failed to write roguelike scores JSON");
        println!("Exported roguelike scores to: {}", roguelike_path.display());

        // Export sandbox scores
        let sandbox_json = serde_json::to_string_pretty(&score.sandbox_details)
            .expect("Failed to serialize sandbox");
        let sandbox_path = output_dir.join("sandbox_scores.json");
        std::fs::write(&sandbox_path, &sandbox_json).expect("Failed to write sandbox scores JSON");
        println!("Exported sandbox scores to: {}", sandbox_path.display());

        // Export medal scores
        let medal_json =
            serde_json::to_string_pretty(&score.medal_details).expect("Failed to serialize medals");
        let medal_path = output_dir.join("medal_scores.json");
        std::fs::write(&medal_path, &medal_json).expect("Failed to write medal scores JSON");
        println!("Exported medal scores to: {}", medal_path.display());

        // Export base scores
        let base_json =
            serde_json::to_string_pretty(&score.base_details).expect("Failed to serialize base");
        let base_path = output_dir.join("base_scores.json");
        std::fs::write(&base_path, &base_json).expect("Failed to write base scores JSON");
        println!("Exported base scores to: {}", base_path.display());

        // Summary
        println!();
        println!("=== Export Summary ===");
        println!("Total Score: {:.2}", score.total_score);
        println!("  Operator Score: {:.2}", score.operator_score);
        println!("  Stage Score: {:.2}", score.stage_score);
        println!("  Roguelike Score: {:.2}", score.roguelike_score);
        println!("  Sandbox Score: {:.2}", score.sandbox_score);
        println!("  Medal Score: {:.2}", score.medal_score);
        println!("  Base Score: {:.2}", score.base_score);
        println!("Operators: {}", score.operator_scores.len());
        println!("Zones: {}", score.zone_scores.len());
        println!("Roguelike Themes: {}", score.roguelike_theme_scores.len());
        println!("Sandbox Areas: {}", score.sandbox_area_scores.len());
        println!(
            "Medal Categories: {} ({} medals earned)",
            score.medal_category_scores.len(),
            score.breakdown.medal_total_earned
        );
        println!(
            "Base Buildings: {} trading, {} factory, {} power, {} dorm",
            score.breakdown.base_trading_post_count,
            score.breakdown.base_factory_count,
            score.breakdown.base_power_plant_count,
            score.breakdown.base_dormitory_count
        );
        println!();
        println!("Files exported to: {}", output_dir.display());
    }
}
