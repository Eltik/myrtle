use std::collections::{HashMap, HashSet};

use crate::core::{
    gamedata::types::building::BuildingDataFile,
    grade::base::{
        buff_registry::BuffResolutionStrategy,
        evaluate::evaluate_buff,
        types::{
            BaseAssignment, EvalContext, OperatorBaseProfile, RoomAssignment, TeammateInfo,
            UserBuilding, UserRoom,
        },
    },
};

pub fn compute_optimal_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> BaseAssignment {
    let mut facility_counts: HashMap<String, usize> = HashMap::new();
    for room in &building.rooms {
        *facility_counts.entry(room.room_type.clone()).or_insert(0) += 1;
    }
    let total_dorm_levels = building.total_dorm_levels();

    let mut assigned: HashSet<String> = HashSet::new();

    let control_slots = max_stationed_for_room(building, building_data, "CONTROL");
    let (cc_assigned, global_bonuses) =
        assign_control_center(operators, control_slots, registry, building_data);
    for id in &cc_assigned {
        assigned.insert(id.clone());
    }

    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .collect();

    let room_assignments = assign_production_rooms(
        &production_rooms,
        operators,
        &mut assigned,
        registry,
        building_data,
        &facility_counts,
        total_dorm_levels,
        &global_bonuses,
    );

    let room_assignments = check_automation_swaps(
        room_assignments,
        operators,
        &assigned,
        registry,
        &facility_counts,
    );

    let total_production_efficiency = room_assignments.iter().map(|r| r.total_efficiency).sum();

    BaseAssignment {
        rooms: room_assignments,
        total_production_efficiency,
    }
}

/// Get the max operator slots for the first room of a given type in the user's base.
/// Falls back to 1 if not found.
fn max_stationed_for_room(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    room_type: &str,
) -> i32 {
    // Find the user's room of this type (take highest level one for CC)
    let level = building
        .rooms
        .iter()
        .filter(|r| r.room_type == room_type)
        .map(|r| r.level)
        .max()
        .unwrap_or(1);

    // Look up in static data
    building_data
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1) as usize))
        .map(|phase| phase.max_stationed_num)
        .unwrap_or(1)
}

/// Get max stationed for a specific room at a specific level
fn max_stationed_at_level(building_data: &BuildingDataFile, room_type: &str, level: i32) -> i32 {
    building_data
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1) as usize))
        .map(|phase| phase.max_stationed_num)
        .unwrap_or(1)
}

/// Assign operators to the Control Center to maximize global production bonuses.
/// Returns (assigned char_ids, global bonuses map: room_type -> total bonus %).
fn assign_control_center(
    operators: &[OperatorBaseProfile],
    max_slots: i32,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> (Vec<String>, HashMap<String, f64>) {
    // Score each operator by how much global production bonus they provide
    let mut scored: Vec<(&OperatorBaseProfile, f64)> = Vec::new();

    for op in operators {
        let mut total_global = 0.0;
        for buff_id in &op.available_buffs {
            // Only consider buffs that target CONTROL room
            if let Some(buff) = building_data.buffs.get(buff_id)
                && buff.room_type != "CONTROL"
            {
                continue;
            }
            if let Some(strategy) = registry.get(buff_id) {
                match strategy {
                    BuffResolutionStrategy::GlobalEffect { bonus_pct, .. } => {
                        total_global += bonus_pct;
                    }
                    BuffResolutionStrategy::TagBased { bonus_pct, .. } => {
                        // Count tag-based as partial value (not all ops will match)
                        total_global += bonus_pct * 0.5;
                    }
                    _ => {}
                }
            }
        }
        if total_global > 0.0 {
            scored.push((op, total_global));
        }
    }

    // Sort descending by global value
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top N operators
    let mut cc_assigned: Vec<String> = Vec::new();
    let mut global_bonuses: HashMap<String, f64> = HashMap::new();

    for (op, _score) in scored.iter().take(max_slots as usize) {
        cc_assigned.push(op.char_id.clone());

        // Accumulate their actual global bonuses by target room
        for buff_id in &op.available_buffs {
            if let Some(buff) = building_data.buffs.get(buff_id)
                && buff.room_type != "CONTROL"
            {
                continue;
            }
            if let Some(strategy) = registry.get(buff_id) {
                match strategy {
                    BuffResolutionStrategy::GlobalEffect {
                        target_room,
                        bonus_pct,
                    } => {
                        *global_bonuses.entry(target_room.clone()).or_insert(0.0) += bonus_pct;
                    }
                    // TODO: Skip TagBased for now — would need to know which ops end up in which rooms (chicken-and-egg)
                    _ => {}
                }
            }
        }
    }

    (cc_assigned, global_bonuses)
}

fn assign_production_rooms(
    rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
) -> Vec<RoomAssignment> {
    let factory_rooms: Vec<&&UserRoom> = rooms.iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .collect();
    let trading_rooms: Vec<&&UserRoom> = rooms.iter()
        .filter(|r| r.room_type == "TRADING")
        .collect();

    let num_factories = factory_rooms.len();

    // Try all gold/EXP splits and pick the best
    let mut best_assignments: Vec<RoomAssignment> = Vec::new();
    let mut best_total: f64 = f64::NEG_INFINITY;
    let mut best_assigned_snapshot: HashSet<String> = assigned.clone();

    // Gold factories must be >= trading post count (TPs need gold bars to trade)
    let min_gold = trading_rooms.len().min(num_factories);

    for num_gold in min_gold..=num_factories {
        let mut trial_assigned = assigned.clone();
        let mut trial_rooms: Vec<RoomAssignment> = Vec::new();

        // Assign factories: first num_gold get F_GOLD, rest get F_EXP
        for (i, factory) in factory_rooms.iter().enumerate() {
            let formula = if i < num_gold { "F_GOLD" } else { "F_EXP" };
            let room_assignment = assign_single_room(
                factory,
                Some(formula),
                operators,
                &mut trial_assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
            );
            trial_rooms.push(room_assignment);
        }

        // Assign trading posts (no formula filter)
        for tp in &trading_rooms {
            let room_assignment = assign_single_room(
                tp,
                None,
                operators,
                &mut trial_assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
            );
            trial_rooms.push(room_assignment);
        }

        let trial_total: f64 = trial_rooms.iter().map(|r| r.total_efficiency).sum();
        if trial_total > best_total {
            best_total = trial_total;
            best_assignments = trial_rooms;
            best_assigned_snapshot = trial_assigned;
        }
    }

    // Apply the winning assignment's used operators to the real assigned set
    *assigned = best_assigned_snapshot;
    best_assignments
}

fn assign_single_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
) -> RoomAssignment {
    let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
    let mut room_ops: Vec<String> = Vec::new();
    let mut room_teammates: Vec<TeammateInfo> = Vec::new();

    for _slot in 0..max_slots {
        let mut best_id: Option<String> = None;
        let mut best_score: f64 = f64::NEG_INFINITY;

        for op in operators {
            if assigned.contains(&op.char_id) { continue; }

            let score = score_operator_in_room(
                op,
                &room.room_type,
                formula_type,
                &room_teammates,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
            );

            if score > best_score {
                best_score = score;
                best_id = Some(op.char_id.clone());
            }
        }

        if let Some(ref id) = best_id {
            if best_score <= 0.0 { break; }

            assigned.insert(id.clone());
            room_ops.push(id.clone());

            let op = operators.iter().find(|o| o.char_id == *id).unwrap();
            let direct_eff = compute_direct_efficiency(
                op, &room.room_type, formula_type, registry, building_data,
            );
            room_teammates.push(TeammateInfo {
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: direct_eff,
            });
        } else {
            break;
        }
    }

    let total_efficiency = compute_room_efficiency(
        &room_ops,
        &room.room_type,
        formula_type,
        operators,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
    ) + global_bonuses.get(&room.room_type).unwrap_or(&0.0);

    RoomAssignment {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: formula_type.map(|s| s.to_string()),
        operators: room_ops,
        total_efficiency,
    }
}

/// Score how valuable an operator would be if added to a room with existing teammates.
fn score_operator_in_room(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    current_teammates: &[TeammateInfo],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    let ctx = EvalContext {
        facility_counts,
        total_dorm_levels,
        room_teammates: current_teammates.to_vec(),
    };

    let mut total = 0.0;
    for buff_id in &op.available_buffs {
        // Check this buff targets the right room type
        if let Some(buff) = building_data.buffs.get(buff_id) {
            if buff.room_type != room_type {
                continue;
            }
            if let Some(formula) = formula_type {
                if !buff.targets.is_empty()
                    && !buff.targets.iter().any(|t| t == formula)
                {
                    continue; // This buff doesn't work for this formula
                }
            }
        } else {
            continue;
        }
        if let Some(strategy) = registry.get(buff_id) {
            total += evaluate_buff(strategy, &ctx);
        }
    }
    total
}

/// Compute only the DirectEfficiency sum for an operator in a room type.
/// Used for TeammateInfo.direct_efficiency, which feeds into
/// TeammateOutputMirroring and TeammateSkillScaling calculations.
fn compute_direct_efficiency(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    let mut total = 0.0;
    for buff_id in &op.available_buffs {
        if let Some(buff) = building_data.buffs.get(buff_id) {
            if buff.room_type != room_type {
                continue;
            }
            if let Some(formula) = formula_type {
                if !buff.targets.is_empty()
                    && !buff.targets.iter().any(|t| t == formula)
                {
                    continue; // This buff doesn't work for this formula
                }
            }
        } else {
            continue;
        }
        if let Some(BuffResolutionStrategy::DirectEfficiency { value }) = registry.get(buff_id) {
            total += value;
        }
    }
    total
}

/// Re-evaluate total room efficiency with complete team context.
fn compute_room_efficiency(
    room_ops: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    all_operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    // Build full TeammateInfo for everyone in the room
    let teammates: Vec<TeammateInfo> = room_ops
        .iter()
        .map(|id| {
            let op = all_operators.iter().find(|o| o.char_id == *id).unwrap();
            let direct_eff = compute_direct_efficiency(op, room_type, formula_type, registry, building_data);
            TeammateInfo {
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: direct_eff,
            }
        })
        .collect();

    let mut total = 0.0;

    for (i, op_id) in room_ops.iter().enumerate() {
        let op = all_operators.iter().find(|o| o.char_id == *op_id).unwrap();

        // Context excludes self from teammates
        let others: Vec<TeammateInfo> = teammates
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, t)| t.clone())
            .collect();

        let ctx = EvalContext {
            facility_counts,
            total_dorm_levels,
            room_teammates: others,
        };

        for buff_id in &op.available_buffs {
            if let Some(buff) = building_data.buffs.get(buff_id) {
                if buff.room_type != room_type {
                    continue;
                }
                if let Some(formula) = formula_type {
                    if !buff.targets.is_empty()
                        && !buff.targets.iter().any(|t| t == formula)
                    {
                        continue; // This buff doesn't work for this formula
                    }
                }
            } else {
                continue;
            }
            if let Some(strategy) = registry.get(buff_id) {
                total += evaluate_buff(strategy, &ctx);
            }
        }
    }

    total
}

fn check_automation_swaps(
    mut room_assignments: Vec<RoomAssignment>,
    operators: &[OperatorBaseProfile],
    assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> Vec<RoomAssignment> {
    let power_count = facility_counts.get("POWER").copied().unwrap_or(0);

    for room in &mut room_assignments {
        if room.room_type != "MANUFACTURE" {
            continue;
        }

        // Find the best automation operator (assigned or not)
        let mut best_auto_value: f64 = 0.0;
        let mut best_auto_id: Option<String> = None;

        for op in operators {
            for buff_id in &op.available_buffs {
                if let Some(strategy) = registry.get(buff_id)
                    && let BuffResolutionStrategy::FacilityCountScaling {
                        nullifies_others: true,
                        per_unit_pct,
                        ..
                    } = strategy
                {
                    let value = per_unit_pct * power_count as f64;
                    if value > best_auto_value {
                        best_auto_value = value;
                        best_auto_id = Some(op.char_id.clone());
                    }
                }
            }
        }

        // If solo automation beats the current team, swap
        if let Some(auto_id) = best_auto_id
            && best_auto_value > room.total_efficiency
            && !assigned.contains(&auto_id)
        {
            room.operators = vec![auto_id];
            room.total_efficiency = best_auto_value;
        }
    }

    room_assignments
}
