use std::collections::HashMap;

use crate::core::gamedata::types::{GameData, building::BuildingDataFile};
use crate::core::grade::base::assignment::compute_sustained_assignment;
use crate::database::models::roster::RosterEntry;

use super::{
    buff_registry::{BuffResolutionStrategy, build_registry},
    types::{OperatorBaseProfile, UserBuilding},
};

pub fn grade_base(
    roster: &[RosterEntry],
    building_json: Option<&serde_json::Value>,
    game_data: &GameData,
) -> f64 {
    let building_data = &game_data.building;

    // If no building data at all, can't score
    if building_data.buffs.is_empty() {
        return 0.0;
    }

    // Build the buff registry
    let registry = build_registry(&building_data.buffs);

    // Parse user building layout from JSONB
    let user_building = match building_json {
        Some(json) => UserBuilding::from_json(json),
        None => return 0.0, // No building data synced
    };
    if user_building.is_empty() {
        return 0.0;
    }

    // Build operator base profiles (combining roster + building char data)
    let profiles = build_operator_profiles(roster, game_data);

    // Dimension 1: Production Roster Potential (40%)
    let d1 = score_production_potential(&profiles, &user_building, building_data, &registry);

    // Dimension 2: Base Infrastructure (25%)
    let d2 = score_infrastructure(&user_building, building_data);

    // Dimension 3: Morale Sustainability (15%)
    let d3 = score_morale_sustainability(&profiles, &user_building, building_data, &registry);

    // Dimension 4: Secondary Facilities (10%)
    let d4 = score_secondary_facilities(&profiles, building_data);

    // Dimension 5: Production Flexibility (10%)
    let d5 = score_production_flexibility(&profiles, building_data);

    0.40 * d1 + 0.25 * d2 + 0.15 * d3 + 0.10 * d4 + 0.10 * d5
}

fn build_operator_profiles(
    roster: &[RosterEntry],
    game_data: &GameData,
) -> Vec<OperatorBaseProfile> {
    let mut profiles = Vec::new();
    for entry in roster {
        let building_char = game_data.building.chars.get(&entry.operator_id);

        if let Some(bc) = building_char {
            profiles.push(OperatorBaseProfile::build(entry, bc));
        }
    }
    profiles
}

fn score_production_potential(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> f64 {
    let user = compute_sustained_assignment(profiles, building, building_data, registry);
    let user_efficiency = user.sustained_efficiency;

    let all_ops = build_max_profiles(building_data);
    let max = compute_sustained_assignment(&all_ops, building, building_data, registry);
    let max_efficiency = max.sustained_efficiency;

    if max_efficiency <= 0.0 {
        return 0.0;
    }

    let raw = user_efficiency / max_efficiency;
    log_curve_ratio(raw.clamp(0.0, 1.0))
}

fn build_max_profiles(building_data: &BuildingDataFile) -> Vec<OperatorBaseProfile> {
    building_data
        .chars
        .values()
        .map(|bc| {
            let available_buffs: Vec<String> = bc
                .buff_char
                .iter()
                .filter_map(|slot| slot.buff_data.last().map(|entry| entry.buff_id.clone()))
                .collect();

            OperatorBaseProfile {
                char_id: bc.char_id.clone(),
                available_buffs,
            }
        })
        .collect()
}

fn score_infrastructure(building: &UserBuilding, building_data: &BuildingDataFile) -> f64 {
    // Sub-score A: Room levels vs max
    let mut level_sum: f64 = 0.0;
    let mut max_level_sum: f64 = 0.0;

    for room in &building.rooms {
        level_sum += room.level as f64;
        let max_level = building_data
            .rooms
            .get(&room.room_type)
            .map(|def| def.phases.len() as f64)
            .unwrap_or(3.0);
        max_level_sum += max_level;
    }

    let level_score = if max_level_sum > 0.0 {
        level_sum / max_level_sum
    } else {
        0.0
    };

    // Sub-score B: Power sufficiency
    let mut power_generated: i32 = 0;
    let mut power_consumed: i32 = 0;

    for room in &building.rooms {
        if let Some(def) = building_data.rooms.get(&room.room_type)
            && let Some(phase) = def.phases.get((room.level - 1) as usize)
        {
            if phase.electricity > 0 {
                power_generated += phase.electricity;
            } else {
                power_consumed += phase.electricity.abs();
            }
        }
    }

    let power_score = if power_consumed > 0 {
        (power_generated as f64 / power_consumed as f64).min(1.0)
    } else {
        1.0
    };

    // Weighted combination
    0.7 * level_score + 0.3 * power_score
}

fn score_morale_sustainability(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> f64 {
    // Count how many operators have production-relevant buffs
    let production_ops = profiles
        .iter()
        .filter(|p| {
            p.available_buffs.iter().any(|b| {
                building_data
                    .buffs
                    .get(b)
                    .map(|buff| buff.room_type == "MANUFACTURE" || buff.room_type == "TRADING")
                    .unwrap_or(false)
            })
        })
        .count();

    // Total production slots needed
    let production_slots: i32 = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .map(|r| {
            building_data
                .rooms
                .get(&r.room_type)
                .and_then(|def| def.phases.get((r.level - 1) as usize))
                .map(|p| p.max_stationed_num)
                .unwrap_or(1)
        })
        .sum();

    // Rotation depth: need 2x slots for shift rotation
    let needed = (production_slots * 2) as f64;
    let rotation_score = if needed > 0.0 {
        (production_ops as f64 / needed).min(1.0)
    } else {
        0.0
    };

    // Count dorm recovery operators
    let dorm_ops = profiles
        .iter()
        .filter(|p| {
            p.available_buffs.iter().any(|b| {
                registry
                    .get(b)
                    .map(|s| matches!(s, BuffResolutionStrategy::MoraleModifier { .. }))
                    .unwrap_or(false)
            })
        })
        .count();

    let dorm_slots: i32 = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| {
            building_data
                .rooms
                .get("DORMITORY")
                .and_then(|def| def.phases.get((r.level - 1) as usize))
                .map(|p| p.max_stationed_num)
                .unwrap_or(5)
        })
        .sum();

    let recovery_score = if dorm_slots > 0 {
        (dorm_ops as f64 / dorm_slots as f64).min(1.0)
    } else {
        0.0
    };

    0.6 * rotation_score + 0.4 * recovery_score
}

fn score_secondary_facilities(
    profiles: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> f64 {
    let secondary_rooms = ["WORKSHOP", "MEETING", "HIRE", "TRAINING"];
    let mut coverage_sum = 0.0;

    for room_type in &secondary_rooms {
        // Count operators with buffs for this room
        let ops_with_buffs = profiles
            .iter()
            .filter(|p| {
                p.available_buffs.iter().any(|b| {
                    building_data
                        .buffs
                        .get(b)
                        .map(|buff| buff.room_type == *room_type)
                        .unwrap_or(false)
                })
            })
            .count();

        // At least 1-2 operators per secondary room is good
        let coverage = (ops_with_buffs as f64 / 2.0).min(1.0);
        coverage_sum += coverage;
    }

    coverage_sum / secondary_rooms.len() as f64
}

fn score_production_flexibility(
    profiles: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> f64 {
    let mut gold_ops = 0;
    let mut exp_ops = 0;
    let mut general_ops = 0;

    for profile in profiles {
        for buff_id in &profile.available_buffs {
            if let Some(buff) = building_data.buffs.get(buff_id) {
                if buff.room_type != "MANUFACTURE" {
                    continue;
                }
                if buff.targets.contains(&"F_GOLD".to_string())
                    && !buff.targets.contains(&"F_EXP".to_string())
                {
                    gold_ops += 1;
                } else if buff.targets.contains(&"F_EXP".to_string())
                    && !buff.targets.contains(&"F_GOLD".to_string())
                {
                    exp_ops += 1;
                } else if buff.targets.is_empty() || buff.targets.len() > 1 {
                    general_ops += 1; // General productivity (no target restriction)
                }
            }
        }
    }

    // Score based on balance — penalize if heavily skewed to one type
    let gold_total = gold_ops + general_ops;
    let exp_total = exp_ops + general_ops;

    if gold_total == 0 && exp_total == 0 {
        return 0.0;
    }

    let max_side = gold_total.max(exp_total) as f64;
    let min_side = gold_total.min(exp_total) as f64;

    if max_side > 0.0 {
        min_side / max_side
    } else {
        0.0
    }
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}
