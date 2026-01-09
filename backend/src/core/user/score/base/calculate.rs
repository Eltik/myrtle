//! Base (RIIC) efficiency score calculations
//!
//! Calculates efficiency scores for trading posts, factories, power plants,
//! dormitories, control center, reception room, and office based on
//! operator skill buffs and room configuration.

use crate::core::user::types::User;

use super::types::{
    BaseBreakdown, BaseScore, ControlCenterDetails, ControlCenterScore, DormitoryDetails,
    DormitoryScore, FactoryDetails, FactoryScore, OfficeDetails, OfficeScore, PowerPlantDetails,
    PowerPlantScore, ReceptionRoomDetails, ReceptionRoomScore, TradingPostDetails,
    TradingPostScore,
};

/// Point values for base scoring
mod points {
    // === Trading Post ===
    pub const TRADING_BASE: f32 = 50.0;
    pub const TRADING_LEVEL: f32 = 25.0;
    pub const TRADING_EFFICIENCY_MULT: f32 = 100.0;
    pub const TRADING_MAX_EFFICIENCY_BONUS: f32 = 200.0;
    pub const TRADING_GOLD_STRATEGY: f32 = 10.0;
    pub const TRADING_ORDER_CAPACITY_MULT: f32 = 5.0;

    // === Factory ===
    pub const FACTORY_BASE: f32 = 50.0;
    pub const FACTORY_LEVEL: f32 = 25.0;
    pub const FACTORY_EFFICIENCY_MULT: f32 = 100.0;
    pub const FACTORY_MAX_EFFICIENCY_BONUS: f32 = 200.0;
    pub const FACTORY_CAPACITY_MULT: f32 = 2.0;

    // === Power Plant ===
    pub const POWER_BASE: f32 = 30.0;
    pub const POWER_LEVEL: f32 = 20.0;
    pub const POWER_ELECTRICITY_MULT: f32 = 0.5;

    // === Dormitory ===
    pub const DORM_BASE: f32 = 20.0;
    pub const DORM_LEVEL: f32 = 15.0;
    pub const DORM_COMFORT_MULT: f32 = 0.02;
    pub const DORM_MAX_COMFORT_BONUS: f32 = 50.0;
    pub const DORM_MAX_COMFORT: i32 = 5000;

    // === Control Center ===
    pub const CONTROL_BASE: f32 = 100.0;
    pub const CONTROL_LEVEL: f32 = 50.0;
    pub const CONTROL_BUFF_MULT: f32 = 200.0;
    pub const CONTROL_AP_REDUCTION_MULT: f32 = 2.0;

    // === Reception Room ===
    pub const RECEPTION_BASE: f32 = 40.0;
    pub const RECEPTION_LEVEL: f32 = 20.0;
    pub const RECEPTION_OPERATOR_BONUS: f32 = 10.0;

    // === Office ===
    pub const OFFICE_BASE: f32 = 40.0;
    pub const OFFICE_LEVEL: f32 = 20.0;
    pub const OFFICE_OPERATOR_BONUS: f32 = 10.0;

    // === Preset/Rotation Bonuses ===
    pub const PRESET_BASE: f32 = 25.0; // Per additional preset beyond the first
    pub const PRESET_FULL_ROTATION: f32 = 15.0; // Bonus if all presets have full operator slots

    // === Global Bonuses ===
    pub const ELECTRICITY_BALANCED_BONUS: f32 = 100.0;
    pub const HIGH_EFFICIENCY_BONUS: f32 = 300.0;
    pub const HIGH_EFFICIENCY_THRESHOLD: f32 = 150.0;
    pub const FULL_BASE_BONUS: f32 = 500.0;

    // === Electricity by level ===
    pub const POWER_LEVEL_1_OUTPUT: i32 = 60;
    pub const POWER_LEVEL_2_OUTPUT: i32 = 130;
    pub const POWER_LEVEL_3_OUTPUT: i32 = 270;
}

/// Calculate the total base efficiency score for a user's account
pub fn calculate_base_score(user: &User) -> BaseScore {
    let building = &user.building;
    let mut breakdown = BaseBreakdown {
        labor_buff_speed: building.status.labor.buff_speed as f32,
        labor_max_value: building.status.labor.max_value,
        ..Default::default()
    };

    // Calculate individual room scores
    let trading_posts = calculate_trading_scores(building, &mut breakdown);
    let factories = calculate_factory_scores(building, &mut breakdown);
    let power_plants = calculate_power_scores(building, &mut breakdown);
    let dormitories = calculate_dormitory_scores(building, &mut breakdown);
    let control_center = calculate_control_center_score(building, &mut breakdown);
    let reception_room = calculate_reception_score(building, &mut breakdown);
    let office = calculate_office_score(building, &mut breakdown);

    // Sum scores
    let trading_score: f32 = trading_posts.iter().map(|t| t.total_score).sum();
    let factory_score: f32 = factories.iter().map(|f| f.total_score).sum();
    let power_score: f32 = power_plants.iter().map(|p| p.total_score).sum();
    let dormitory_score: f32 = dormitories.iter().map(|d| d.total_score).sum();
    let control_center_score = control_center
        .as_ref()
        .map(|c| c.total_score)
        .unwrap_or(0.0);
    let reception_score = reception_room
        .as_ref()
        .map(|r| r.total_score)
        .unwrap_or(0.0);
    let office_score = office.as_ref().map(|o| o.total_score).unwrap_or(0.0);

    // Calculate global bonuses
    let global_bonus_score = calculate_global_bonuses(&breakdown);

    let total_score = trading_score
        + factory_score
        + power_score
        + dormitory_score
        + control_center_score
        + reception_score
        + office_score
        + global_bonus_score;

    BaseScore {
        total_score,
        trading_score,
        factory_score,
        power_score,
        dormitory_score,
        control_center_score,
        reception_score,
        office_score,
        global_bonus_score,
        trading_posts,
        factories,
        power_plants,
        dormitories,
        control_center,
        reception_room,
        office,
        breakdown,
    }
}

/// Get room level from room_slots by slot_id
fn get_room_level(
    room_slots: &std::collections::HashMap<String, crate::core::user::types::RoomSlot>,
    slot_id: &str,
) -> i32 {
    room_slots.get(slot_id).map(|slot| slot.level).unwrap_or(1)
}

/// Count operators stationed in a room slot
fn count_operators_in_slot(
    room_slots: &std::collections::HashMap<String, crate::core::user::types::RoomSlot>,
    slot_id: &str,
) -> i32 {
    room_slots
        .get(slot_id)
        .map(|slot| slot.char_inst_ids.len() as i32)
        .unwrap_or(0)
}

/// Parse preset queue from room data and return (preset_count, operators_per_preset)
fn parse_presets(room_data: &serde_json::Value) -> (i32, Vec<i32>) {
    let preset_queue = room_data.get("presetQueue");

    if let Some(presets) = preset_queue.and_then(|p| p.as_array()) {
        let preset_count = presets.len() as i32;
        let operators_per_preset: Vec<i32> = presets
            .iter()
            .map(|preset| preset.as_array().map(|arr| arr.len() as i32).unwrap_or(0))
            .collect();
        (preset_count, operators_per_preset)
    } else {
        (0, Vec::new())
    }
}

/// Calculate preset score based on number of presets and their completeness
fn calculate_preset_score(
    preset_count: i32,
    operators_per_preset: &[i32],
    max_operators: i32,
) -> f32 {
    if preset_count <= 1 {
        return 0.0;
    }

    // Base score for additional presets (beyond the first)
    let additional_presets = preset_count - 1;
    let base_preset_score = additional_presets as f32 * points::PRESET_BASE;

    // Bonus if all presets have full operator slots
    let full_presets = operators_per_preset
        .iter()
        .filter(|&&count| count >= max_operators)
        .count() as i32;

    let full_rotation_bonus = if full_presets == preset_count {
        points::PRESET_FULL_ROTATION * preset_count as f32
    } else {
        0.0
    };

    base_preset_score + full_rotation_bonus
}

/// Calculate trading post scores
fn calculate_trading_scores(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Vec<TradingPostScore> {
    let mut scores = Vec::new();

    for (slot_id, room_data) in &building.rooms.trading {
        breakdown.trading_post_count += 1;

        // Parse room data from JSON
        let speed = room_data
            .get("buff")
            .and_then(|b| b.get("speed"))
            .and_then(|s| s.as_f64())
            .unwrap_or(1.0) as f32;

        let stock_limit = room_data
            .get("stockLimit")
            .and_then(|s| s.as_i64())
            .unwrap_or(6) as i32;

        let strategy = room_data
            .get("strategy")
            .and_then(|s| s.as_str())
            .unwrap_or("O_GOLD")
            .to_string();

        // Get level from room_slots
        let level = get_room_level(&building.room_slots, slot_id);
        let operators_stationed = count_operators_in_slot(&building.room_slots, slot_id);

        // Parse presets
        let (preset_count, operators_per_preset) = parse_presets(room_data);

        // Max operators for trading post by level (1/2/2 for levels 1/2/3)
        let max_operators = match level {
            1 => 1,
            2 => 2,
            _ => 2,
        };

        // Update breakdown
        breakdown.total_building_levels += level;
        if level >= 3 {
            breakdown.max_level_buildings += 1;
        }
        breakdown.operators_in_production += operators_stationed;
        breakdown.total_stationed_operators += operators_stationed;

        // Calculate electricity consumption (10 for level 1, 30 for level 2, 60 for level 3)
        let electricity_consumption = match level {
            1 => 10,
            2 => 30,
            _ => 60,
        };
        breakdown.total_electricity_consumption += electricity_consumption;

        // Calculate scores
        let efficiency_bonus = (speed - 1.0).max(0.0) * points::TRADING_EFFICIENCY_MULT;
        let level_score = level as f32 * points::TRADING_LEVEL;
        let max_efficiency_bonus = if speed >= 2.0 {
            points::TRADING_MAX_EFFICIENCY_BONUS
        } else {
            0.0
        };

        let efficiency_score =
            points::TRADING_BASE + level_score + efficiency_bonus + max_efficiency_bonus;
        let order_score = (stock_limit - 6).max(0) as f32 * points::TRADING_ORDER_CAPACITY_MULT;
        let strategy_score = if strategy == "O_GOLD" {
            points::TRADING_GOLD_STRATEGY
        } else {
            0.0
        };
        let preset_score =
            calculate_preset_score(preset_count, &operators_per_preset, max_operators);

        let total_score = efficiency_score + order_score + strategy_score + preset_score;

        // Update average efficiency
        let current_count = breakdown.trading_post_count;
        breakdown.avg_trading_efficiency =
            ((breakdown.avg_trading_efficiency * (current_count - 1) as f32) + speed * 100.0)
                / current_count as f32;

        scores.push(TradingPostScore {
            slot_id: slot_id.clone(),
            level,
            efficiency_score,
            order_score,
            strategy_score,
            preset_score,
            total_score,
            details: TradingPostDetails {
                speed_multiplier: speed,
                stock_limit,
                strategy,
                operators_stationed,
                preset_count,
                operators_per_preset,
            },
        });
    }

    // Sort by total score descending
    scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// Calculate factory scores
fn calculate_factory_scores(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Vec<FactoryScore> {
    let mut scores = Vec::new();

    for (slot_id, room_data) in &building.rooms.manufacture {
        breakdown.factory_count += 1;

        // Parse room data from JSON
        let speed = room_data
            .get("buff")
            .and_then(|b| b.get("speed"))
            .and_then(|s| s.as_f64())
            .unwrap_or(1.0) as f32;

        let capacity = room_data
            .get("buff")
            .and_then(|b| b.get("capacity"))
            .and_then(|c| c.as_i64())
            .unwrap_or(0) as i32;

        let formula_id = room_data
            .get("formulaId")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();

        // Determine production type from formula ID
        let production_type = match formula_id.as_str() {
            "1" | "2" => "EXP".to_string(),
            "3" | "4" => "Gold".to_string(),
            "5" | "6" => "Originium".to_string(),
            _ => "Unknown".to_string(),
        };

        // Get level from room_slots
        let level = get_room_level(&building.room_slots, slot_id);
        let operators_stationed = count_operators_in_slot(&building.room_slots, slot_id);

        // Parse presets
        let (preset_count, operators_per_preset) = parse_presets(room_data);

        // Max operators for factory by level (1/2/3 for levels 1/2/3)
        let max_operators = match level {
            1 => 1,
            2 => 2,
            _ => 3,
        };

        // Update breakdown
        breakdown.total_building_levels += level;
        if level >= 3 {
            breakdown.max_level_buildings += 1;
        }
        breakdown.operators_in_production += operators_stationed;
        breakdown.total_stationed_operators += operators_stationed;

        // Calculate electricity consumption
        let electricity_consumption = match level {
            1 => 10,
            2 => 30,
            _ => 60,
        };
        breakdown.total_electricity_consumption += electricity_consumption;

        // Base capacity by level (24, 36, 54)
        let base_capacity = match level {
            1 => 24,
            2 => 36,
            _ => 54,
        };

        // Calculate scores
        let efficiency_bonus = (speed - 1.0).max(0.0) * points::FACTORY_EFFICIENCY_MULT;
        let level_score = level as f32 * points::FACTORY_LEVEL;
        let max_efficiency_bonus = if speed >= 2.0 {
            points::FACTORY_MAX_EFFICIENCY_BONUS
        } else {
            0.0
        };

        let efficiency_score =
            points::FACTORY_BASE + level_score + efficiency_bonus + max_efficiency_bonus;
        let capacity_score =
            (capacity - base_capacity).max(0) as f32 * points::FACTORY_CAPACITY_MULT;
        let preset_score =
            calculate_preset_score(preset_count, &operators_per_preset, max_operators);

        let total_score = efficiency_score + capacity_score + preset_score;

        // Update average efficiency
        let current_count = breakdown.factory_count;
        breakdown.avg_factory_efficiency =
            ((breakdown.avg_factory_efficiency * (current_count - 1) as f32) + speed * 100.0)
                / current_count as f32;

        scores.push(FactoryScore {
            slot_id: slot_id.clone(),
            level,
            efficiency_score,
            capacity_score,
            preset_score,
            total_score,
            details: FactoryDetails {
                speed_multiplier: speed,
                capacity,
                production_type,
                operators_stationed,
                preset_count,
                operators_per_preset,
            },
        });
    }

    // Sort by total score descending
    scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// Calculate power plant scores
fn calculate_power_scores(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Vec<PowerPlantScore> {
    let mut scores = Vec::new();

    for slot_id in building.rooms.power.keys() {
        breakdown.power_plant_count += 1;

        // Get level from room_slots
        let level = get_room_level(&building.room_slots, slot_id);
        let operators_stationed = count_operators_in_slot(&building.room_slots, slot_id);

        // Update breakdown
        breakdown.total_building_levels += level;
        if level >= 3 {
            breakdown.max_level_buildings += 1;
        }
        breakdown.operators_in_support += operators_stationed;
        breakdown.total_stationed_operators += operators_stationed;

        // Electricity output by level
        let electricity_output = match level {
            1 => points::POWER_LEVEL_1_OUTPUT,
            2 => points::POWER_LEVEL_2_OUTPUT,
            _ => points::POWER_LEVEL_3_OUTPUT,
        };
        breakdown.total_electricity_output += electricity_output;

        // Calculate scores
        let level_score = level as f32 * points::POWER_LEVEL;
        let electricity_score = electricity_output as f32 * points::POWER_ELECTRICITY_MULT;

        // Drone recovery score (simplified - based on level)
        let drone_score = level as f32 * 10.0;

        let total_score = points::POWER_BASE + level_score + electricity_score + drone_score;

        scores.push(PowerPlantScore {
            slot_id: slot_id.clone(),
            level,
            electricity_score: points::POWER_BASE + level_score + electricity_score,
            drone_score,
            total_score,
            details: PowerPlantDetails {
                electricity_output,
                operators_stationed,
            },
        });
    }

    // Sort by total score descending
    scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// Calculate dormitory scores
fn calculate_dormitory_scores(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Vec<DormitoryScore> {
    let mut scores = Vec::new();

    for (slot_id, room_data) in &building.rooms.dormitory {
        breakdown.dormitory_count += 1;

        // Parse comfort from JSON
        let comfort = room_data
            .get("comfort")
            .and_then(|c| c.as_i64())
            .unwrap_or(0) as i32;

        // Get level from room_slots
        let level = get_room_level(&building.room_slots, slot_id);
        let operators_stationed = count_operators_in_slot(&building.room_slots, slot_id);

        // Update breakdown
        breakdown.total_building_levels += level;
        if level >= 5 {
            breakdown.max_level_buildings += 1;
        }
        breakdown.total_comfort += comfort;
        if comfort >= points::DORM_MAX_COMFORT {
            breakdown.max_comfort_dorms += 1;
        }
        breakdown.operators_in_rest += operators_stationed;
        breakdown.total_stationed_operators += operators_stationed;

        // Calculate electricity consumption for dormitory
        let electricity_consumption = match level {
            1 => 10,
            2 => 20,
            3 => 30,
            4 => 45,
            _ => 65,
        };
        breakdown.total_electricity_consumption += electricity_consumption;

        // Calculate comfort percentage
        let comfort_percentage = (comfort as f32 / points::DORM_MAX_COMFORT as f32) * 100.0;

        // Calculate scores
        let level_score = level as f32 * points::DORM_LEVEL;
        let comfort_score_value = comfort as f32 * points::DORM_COMFORT_MULT;
        let max_comfort_bonus = if comfort >= points::DORM_MAX_COMFORT {
            points::DORM_MAX_COMFORT_BONUS
        } else {
            0.0
        };

        let comfort_score =
            points::DORM_BASE + level_score + comfort_score_value + max_comfort_bonus;
        let morale_recovery_score = 0.0; // Simplified for now

        let total_score = comfort_score + morale_recovery_score;

        scores.push(DormitoryScore {
            slot_id: slot_id.clone(),
            level,
            comfort_score,
            morale_recovery_score,
            total_score,
            details: DormitoryDetails {
                comfort_level: comfort,
                comfort_percentage,
            },
        });
    }

    // Calculate average comfort per dorm
    if breakdown.dormitory_count > 0 {
        breakdown.avg_comfort_per_dorm =
            breakdown.total_comfort as f32 / breakdown.dormitory_count as f32;
    }

    // Sort by total score descending
    scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// Calculate control center score
fn calculate_control_center_score(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Option<ControlCenterScore> {
    // Get the first (and only) control center
    let (slot_id, room_data) = building.rooms.control.iter().next()?;

    // Get level from room_slots
    let level = get_room_level(&building.room_slots, slot_id);
    let operators_stationed = count_operators_in_slot(&building.room_slots, slot_id);

    // Parse presets
    let (preset_count, operators_per_preset) = parse_presets(room_data);

    // Max operators for control center by level (1-5 for levels 1-5)
    let max_operators = level;

    // Update breakdown
    breakdown.total_building_levels += level;
    if level >= 5 {
        breakdown.max_level_buildings += 1;
    }
    breakdown.operators_in_support += operators_stationed;
    breakdown.total_stationed_operators += operators_stationed;

    // Parse buff data
    let trading_buff = room_data
        .get("buff")
        .and_then(|b| b.get("trading"))
        .and_then(|t| {
            // Trading buff can be a number or an object with values
            if let Some(n) = t.as_f64() {
                Some(n as f32)
            } else if let Some(obj) = t.as_object() {
                // Sum all trading buff values
                let sum: f64 = obj.values().filter_map(|v| v.as_f64()).sum();
                Some(sum as f32)
            } else {
                None
            }
        })
        .unwrap_or(0.0);

    let manufacture_buff = room_data
        .get("buff")
        .and_then(|b| b.get("manufacture"))
        .and_then(|m| {
            if let Some(n) = m.as_f64() {
                Some(n as f32)
            } else if let Some(obj) = m.as_object() {
                let sum: f64 = obj.values().filter_map(|v| v.as_f64()).sum();
                Some(sum as f32)
            } else {
                None
            }
        })
        .unwrap_or(0.0);

    let ap_cost = room_data
        .get("apCost")
        .and_then(|a| a.as_i64())
        .unwrap_or(0) as i32;

    // AP cost is negative (reduction), so we use absolute value
    let ap_cost_reduction = ap_cost.abs();

    // Calculate scores
    let level_score = level as f32 * points::CONTROL_LEVEL;
    let buff_score = (trading_buff + manufacture_buff) * points::CONTROL_BUFF_MULT;
    let ap_cost_score = ap_cost_reduction as f32 * points::CONTROL_AP_REDUCTION_MULT;
    let preset_score = calculate_preset_score(preset_count, &operators_per_preset, max_operators);

    let global_buff_score = points::CONTROL_BASE + level_score + buff_score;
    let total_score = global_buff_score + ap_cost_score + preset_score;

    Some(ControlCenterScore {
        slot_id: slot_id.clone(),
        level,
        global_buff_score,
        ap_cost_score,
        total_score,
        details: ControlCenterDetails {
            trading_buff,
            manufacture_buff,
            ap_cost_reduction,
            operators_stationed,
            preset_count,
            operators_per_preset,
        },
    })
}

/// Calculate reception room score (from room_slots with roomId="MEETING")
fn calculate_reception_score(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Option<ReceptionRoomScore> {
    // Find MEETING room in room_slots
    let (slot_id, slot) = building
        .room_slots
        .iter()
        .find(|(_, slot)| slot.room_id == "MEETING")?;

    let level = slot.level;
    let operators_stationed = slot.char_inst_ids.len() as i32;

    // Update breakdown
    breakdown.total_building_levels += level;
    if level >= 3 {
        breakdown.max_level_buildings += 1;
    }
    breakdown.operators_in_support += operators_stationed;
    breakdown.total_stationed_operators += operators_stationed;

    // Electricity consumption for meeting room
    let electricity_consumption = match level {
        1 => 10,
        2 => 30,
        _ => 60,
    };
    breakdown.total_electricity_consumption += electricity_consumption;

    // Calculate scores (level-only since we don't have buff data)
    let level_score = points::RECEPTION_BASE + (level as f32 * points::RECEPTION_LEVEL);
    let operators_score = operators_stationed as f32 * points::RECEPTION_OPERATOR_BONUS;
    let total_score = level_score + operators_score;

    Some(ReceptionRoomScore {
        slot_id: slot_id.clone(),
        level,
        level_score,
        operators_score,
        total_score,
        details: ReceptionRoomDetails {
            operators_stationed,
        },
    })
}

/// Calculate office score (from room_slots with roomId="HIRE")
fn calculate_office_score(
    building: &crate::core::user::types::Building,
    breakdown: &mut BaseBreakdown,
) -> Option<OfficeScore> {
    // Find HIRE room in room_slots
    let (slot_id, slot) = building
        .room_slots
        .iter()
        .find(|(_, slot)| slot.room_id == "HIRE")?;

    let level = slot.level;
    let operators_stationed = slot.char_inst_ids.len() as i32;

    // Update breakdown
    breakdown.total_building_levels += level;
    if level >= 3 {
        breakdown.max_level_buildings += 1;
    }
    breakdown.operators_in_support += operators_stationed;
    breakdown.total_stationed_operators += operators_stationed;

    // Electricity consumption for office
    let electricity_consumption = match level {
        1 => 10,
        2 => 30,
        _ => 60,
    };
    breakdown.total_electricity_consumption += electricity_consumption;

    // Calculate scores (level-only since we don't have buff data)
    let level_score = points::OFFICE_BASE + (level as f32 * points::OFFICE_LEVEL);
    let operators_score = operators_stationed as f32 * points::OFFICE_OPERATOR_BONUS;
    let total_score = level_score + operators_score;

    Some(OfficeScore {
        slot_id: slot_id.clone(),
        level,
        level_score,
        operators_score,
        total_score,
        details: OfficeDetails {
            operators_stationed,
        },
    })
}

/// Calculate global bonuses based on overall base efficiency
fn calculate_global_bonuses(breakdown: &BaseBreakdown) -> f32 {
    let mut bonus = 0.0;

    // Electricity balance bonus - check if output >= consumption
    if breakdown.total_electricity_output >= breakdown.total_electricity_consumption {
        bonus += points::ELECTRICITY_BALANCED_BONUS;
    }

    // High efficiency bonus (average of trading and factory > 150%)
    let avg_efficiency =
        (breakdown.avg_trading_efficiency + breakdown.avg_factory_efficiency) / 2.0;
    if avg_efficiency > points::HIGH_EFFICIENCY_THRESHOLD {
        bonus += points::HIGH_EFFICIENCY_BONUS;
    }

    // Full base bonus (all production buildings at max level)
    // Trading posts (max 5), factories (max 5), power plants (max 3) at level 3
    // This is a simplification - we check if we have enough max-level buildings
    let production_buildings =
        breakdown.trading_post_count + breakdown.factory_count + breakdown.power_plant_count;
    if production_buildings > 0 && breakdown.max_level_buildings >= production_buildings {
        bonus += points::FULL_BASE_BONUS;
    }

    bonus
}
