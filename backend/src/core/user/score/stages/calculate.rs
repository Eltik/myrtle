use std::collections::HashMap;

use crate::core::local::types::{GameData, zone::ZoneType};
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
                if user_stage.complete_times > 0 {
                    completed_stages += 1;
                    breakdown.total_stages_completed += 1;
                }

                if user_stage.state >= 2 {
                    perfect_stages += 1;
                    breakdown.total_perfect_clears += 1;
                }
            }
        }

        breakdown.total_stages_available += total_stages;

        match zone.zone_type {
            ZoneType::Mainline => {
                mainline_completed += completed_stages;
                mainline_total += total_stages;
            }
            ZoneType::Sidestory | ZoneType::Branchline => {
                sidestory_completed += completed_stages;
                sidestory_total += total_stages;
            }
            ZoneType::Activity | ZoneType::MainlineActivity => {
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
            &zone.zone_type,
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
            zone_type: zone.zone_type.clone(),
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

