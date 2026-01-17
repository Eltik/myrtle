use std::collections::HashMap;

use crate::core::local::types::{GameData, stage::StageType, zone::ZoneType};
use crate::core::user::types::User;

use super::types::{StageBreakdown, StageScore, ZoneScore};

pub fn calculate_stage_score(user: &User, game_data: &GameData) -> StageScore {
    let mut stages_by_zone: HashMap<String, Vec<&str>> = HashMap::new();
    for (stage_id, stage) in &game_data.stages {
        if stage.is_story_only {
            continue; // tutorial/guide stages
        }

        stages_by_zone
            .entry(stage.zone_id.clone())
            .or_default()
            .push(stage_id);
    }

    let mut zone_scores: Vec<ZoneScore> = Vec::new();
    let mut breakdown = StageBreakdown::default();

    let mut mainline_completed = 0;
    let mut mainline_total = 0;
    let mut sidestory_completed = 0;
    let mut sidestory_total = 0;
    let mut activity_completed = 0;
    let mut activity_total = 0;

    for (zone_id, stage_ids) in &stages_by_zone {
        let zone = match game_data.zones.get(zone_id) {
            Some(z) => z,
            None => continue,
        };

        let total_stages = stage_ids.len() as i32;
        let mut completed_stages = 0;
        let mut perfect_stages = 0;

        for stage_id in stage_ids {
            if let Some(user_stage) = user.dungeon.stages.get(*stage_id) {
                // Use state >= 2 for completion check (not complete_times)
                // Easy/tough mode stages have complete_times=0 but state=3 when cleared
                // state=2: cleared (2-star), state=3: perfect (3-star)
                if user_stage.state >= 2 {
                    completed_stages += 1;
                    breakdown.total_stages_completed += 1;
                }

                // state >= 3 means 3-star (perfect) clear
                if user_stage.state >= 3 {
                    perfect_stages += 1;
                    breakdown.total_perfect_clears += 1;
                }
            }
        }

        breakdown.total_stages_available += total_stages;

        // Determine effective zone type
        // Only apply "permanent_sidestory" heuristic for zones with Unknown type
        // to handle cases where stage zone_ids don't match zone_table entries.
        // IMPORTANT: Do NOT override Activity type zones even if they have "side"
        // in the name (e.g., "act43side_zone1" is time-limited, not permanent sidestory)
        let effective_zone_type = match zone.zone_type {
            ZoneType::Unknown if zone_id.contains("permanent_sidestory") => ZoneType::Sidestory,
            _ => zone.zone_type.clone(),
        };

        match effective_zone_type {
            ZoneType::Mainline | ZoneType::MainlineActivity => {
                // MainlineActivity = former events that became permanent mainline content
                // (e.g., Episode 15 "Dissociative Recombination")
                //
                // Exclude from mainline completion count:
                // 1. "act" prefixed stages - anniversary/collaboration event stages
                // 2. SUB type stages - optional side content (tr_s_X, sub_XX-X-X)
                let mainline_stage_ids: Vec<&&str> = stage_ids
                    .iter()
                    .filter(|s| {
                        // Exclude act-prefixed stages
                        if s.starts_with("act") {
                            return false;
                        }
                        // Exclude SUB type stages
                        if let Some(stage) = game_data.stages.get(**s)
                            && stage.stage_type == StageType::Sub
                        {
                            return false;
                        }
                        true
                    })
                    .collect();

                let mut mainline_zone_completed = 0;
                let mainline_zone_total = mainline_stage_ids.len() as i32;

                for stage_id in &mainline_stage_ids {
                    if let Some(user_stage) = user.dungeon.stages.get(**stage_id)
                        && user_stage.state >= 2
                    {
                        mainline_zone_completed += 1;
                    }
                }

                mainline_completed += mainline_zone_completed;
                mainline_total += mainline_zone_total;
                // Mainline is permanent content (excluding act-prefixed and SUB stages)
                breakdown.permanent_stages_completed += mainline_zone_completed;
                breakdown.permanent_stages_available += mainline_zone_total;
            }
            ZoneType::Sidestory | ZoneType::Branchline => {
                sidestory_completed += completed_stages;
                sidestory_total += total_stages;
                // Sidestory/Branchline is permanent content
                breakdown.permanent_stages_completed += completed_stages;
                breakdown.permanent_stages_available += total_stages;
            }
            ZoneType::Activity => {
                // Activity stages are time-limited, NOT counted as permanent
                activity_completed += completed_stages;
                activity_total += total_stages;
            }
            _ => {}
        }

        let completion_percentage = if total_stages > 0 {
            (completed_stages as f32 / total_stages as f32) * 100.0
        } else {
            0.0
        };

        let score = calculate_zone_score(
            &effective_zone_type,
            completed_stages,
            perfect_stages,
            total_stages,
        );

        let zone_name = zone
            .zone_name_first
            .clone()
            .or(zone.zone_name_second.clone())
            .unwrap_or_else(|| zone_id.clone());

        zone_scores.push(ZoneScore {
            zone_id: zone_id.clone(),
            zone_name,
            zone_type: effective_zone_type,
            zone_index: zone.zone_index,
            total_stages,
            completed_stages,
            perfect_stages,
            completion_percentage,
            score,
        });
    }

    breakdown.mainline_completion = if mainline_total > 0 {
        (mainline_completed as f32 / mainline_total as f32) * 100.0
    } else {
        0.0
    };

    breakdown.sidestory_completion = if sidestory_total > 0 {
        (sidestory_completed as f32 / sidestory_total as f32) * 100.0
    } else {
        0.0
    };

    breakdown.activity_completion = if activity_total > 0 {
        (activity_completed as f32 / activity_total as f32) * 100.0
    } else {
        0.0
    };

    // Sort by score descending (highest score first)
    zone_scores.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_score: f32 = zone_scores.iter().map(|z| z.score).sum();

    StageScore {
        total_score,
        zone_scores,
        breakdown,
    }
}

/// Calculate score for a single zone based on type and completion
fn calculate_zone_score(zone_type: &ZoneType, completed: i32, perfect: i32, total: i32) -> f32 {
    if total == 0 {
        return 0.0;
    }

    let base_per_stage = match zone_type {
        ZoneType::Mainline => 2.0,
        ZoneType::Sidestory | ZoneType::Branchline => 1.5,
        ZoneType::Activity | ZoneType::MainlineActivity => 1.0,
        ZoneType::Campaign => 1.5,
        ZoneType::Weekly | ZoneType::ClimbTower => 0.5,
        _ => 0.25,
    };

    // Bonus for perfect/3-star clears
    let perfect_bonus = 0.5;

    (completed as f32 * base_per_stage) + (perfect as f32 * perfect_bonus)
}
