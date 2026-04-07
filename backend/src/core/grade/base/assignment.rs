use std::collections::{HashMap, HashSet};

use crate::core::{
    gamedata::types::building::BuildingDataFile,
    grade::base::{
        buff_registry::BuffResolutionStrategy,
        evaluate::evaluate_buff,
        types::{
            BaseAssignment, EvalContext, OperatorBaseProfile, RoomAssignment, ShiftAssignment,
            TeammateInfo, UserBuilding, UserRoom,
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

    let total_production_efficiency = room_assignments.iter().map(|r| r.total_efficiency).sum();

    BaseAssignment {
        rooms: room_assignments,
        total_production_efficiency,
    }
}

pub fn compute_sustained_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> ShiftAssignment {
    let mut facility_counts: HashMap<String, usize> = HashMap::new();
    for room in &building.rooms {
        *facility_counts.entry(room.room_type.clone()).or_insert(0) += 1;
    }
    let total_dorm_levels = building.total_dorm_levels();

    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" || r.room_type == "TRADING")
        .collect();

    let control_slots = max_stationed_for_room(building, building_data, "CONTROL");
    let (cc_assigned, global_bonuses) =
        assign_control_center(operators, control_slots, registry, building_data);

    // --- Shift A: best possible team ---
    let mut assigned_a: HashSet<String> = cc_assigned.iter().cloned().collect();
    let rooms_a = assign_production_rooms(
        &production_rooms,
        operators,
        &mut assigned_a,
        registry,
        building_data,
        &facility_counts,
        total_dorm_levels,
        &global_bonuses,
    );

    // --- Shift B: best team from remaining operators ---
    // CC operators stay assigned (shared), but production ops from Shift A are excluded
    let mut assigned_b: HashSet<String> = assigned_a.clone();
    let rooms_b = assign_production_rooms(
        &production_rooms,
        operators,
        &mut assigned_b,
        registry,
        building_data,
        &facility_counts,
        total_dorm_levels,
        &global_bonuses,
    );

    let total_a: f64 = rooms_a.iter().map(|r| r.total_efficiency).sum();
    let total_b: f64 = rooms_b.iter().map(|r| r.total_efficiency).sum();

    ShiftAssignment {
        shift_a: BaseAssignment {
            rooms: rooms_a,
            total_production_efficiency: total_a,
        },
        shift_b: BaseAssignment {
            rooms: rooms_b,
            total_production_efficiency: total_b,
        },
        sustained_efficiency: (total_a + total_b) / 2.0,
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
    let factory_rooms: Vec<&&UserRoom> = rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .collect();
    let trading_rooms: Vec<&&UserRoom> =
        rooms.iter().filter(|r| r.room_type == "TRADING").collect();

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
    let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);

    // Mode 1: Normal assignment (no automation)
    let normal = greedy_fill_room(
        room,
        formula_type,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        max_slots,
        false,
    );
    let normal_eff = normal.2 + global; // (ops, teammates, efficiency)

    // Mode 2: Automation assignment (only for factories)
    let auto_eff;
    let auto_result;
    if room.room_type == "MANUFACTURE" {
        auto_result = greedy_fill_room(
            room,
            formula_type,
            operators,
            assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            max_slots,
            true,
        );
        auto_eff = auto_result.2 + global;
    } else {
        auto_result = (Vec::new(), Vec::new(), 0.0);
        auto_eff = f64::NEG_INFINITY;
    }

    // Pick the better mode
    let (room_ops, _teammates, _eff) = if auto_eff > normal_eff {
        auto_result
    } else {
        normal
    };

    let best_eff = normal_eff.max(auto_eff);

    // Mark operators as assigned
    for id in &room_ops {
        assigned.insert(id.clone());
    }

    RoomAssignment {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: formula_type.map(|s| s.to_string()),
        operators: room_ops,
        total_efficiency: best_eff,
    }
}

/// Fill a room greedily. Returns (operator_ids, teammate_infos, total_efficiency).
/// If automation_only is true, only considers FacilityCountScaling buffs.
fn greedy_fill_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
    automation_only: bool,
) -> (Vec<String>, Vec<TeammateInfo>, f64) {
    let mut room_ops: Vec<String> = Vec::new();
    let mut room_teammates: Vec<TeammateInfo> = Vec::new();
    let mut local_assigned: HashSet<String> = HashSet::new();

    for _slot in 0..max_slots {
        let mut best_id: Option<String> = None;
        let mut best_score: f64 = f64::NEG_INFINITY;

        for op in operators {
            if already_assigned.contains(&op.char_id) || local_assigned.contains(&op.char_id) {
                continue;
            }

            let score = if automation_only {
                // Only count FacilityCountScaling buffs (the ones that survive automation)
                score_operator_facility_only(
                    op,
                    &room.room_type,
                    formula_type,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                )
            } else {
                // Normal scoring — but skip operators that have automation buffs
                // (they'd nullify teammates, which isn't modeled in normal mode)
                let has_automation = op.available_buffs.iter().any(|b| {
                    matches!(
                        registry.get(b),
                        Some(BuffResolutionStrategy::FacilityCountScaling {
                            nullifies_others: true,
                            ..
                        })
                    )
                });
                if has_automation {
                    continue;
                } // Don't mix automation ops into normal teams

                score_operator_in_room(
                    op,
                    &room.room_type,
                    formula_type,
                    &room_teammates,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                )
            };

            if score > best_score {
                best_score = score;
                best_id = Some(op.char_id.clone());
            }
        }

        if let Some(ref id) = best_id {
            if best_score <= 0.0 {
                break;
            }

            local_assigned.insert(id.clone());
            room_ops.push(id.clone());

            let op = operators.iter().find(|o| o.char_id == *id).unwrap();
            let direct_eff = compute_direct_efficiency(
                op,
                &room.room_type,
                formula_type,
                registry,
                building_data,
            );
            let order_limit =
                compute_order_limit(op, &room.room_type, formula_type, registry, building_data);
            room_teammates.push(TeammateInfo {
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: direct_eff,
                order_limit_contribution: order_limit,
            });
        } else {
            break;
        }
    }

    // Compute final efficiency
    let total = if automation_only {
        // In automation mode, sum all operators' FacilityCountScaling values
        room_ops
            .iter()
            .map(|id| {
                let op = operators.iter().find(|o| o.char_id == *id).unwrap();
                score_operator_facility_only(
                    op,
                    &room.room_type,
                    formula_type,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                )
            })
            .sum()
    } else {
        compute_room_efficiency(
            &room_ops,
            &room.room_type,
            formula_type,
            operators,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
        )
    };

    (room_ops, room_teammates, total)
}

/// Score only FacilityCountScaling buffs for an operator (used in automation rooms).
fn score_operator_facility_only(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
) -> f64 {
    let ctx = EvalContext {
        facility_counts,
        total_dorm_levels,
        room_teammates: Vec::new(), // No teammate context needed for facility-count buffs
    };

    let mut total = 0.0;
    for buff_id in &op.available_buffs {
        if let Some(buff) = building_data.buffs.get(buff_id) {
            if buff.room_type != room_type {
                continue;
            }
            if let Some(formula) = formula_type
                && !buff.targets.is_empty()
                && !buff.targets.iter().any(|t| t == formula)
            {
                continue;
            }
        } else {
            continue;
        }
        if let Some(strategy) = registry.get(buff_id)
            && matches!(
                strategy,
                BuffResolutionStrategy::FacilityCountScaling { .. }
            )
        {
            total += evaluate_buff(strategy, &ctx);
        }
    }
    total
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
            if let Some(formula) = formula_type
                && !buff.targets.is_empty()
                && !buff.targets.iter().any(|t| t == formula)
            {
                continue; // This buff doesn't work for this formula
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
            if let Some(formula) = formula_type
                && !buff.targets.is_empty()
                && !buff.targets.iter().any(|t| t == formula)
            {
                continue; // This buff doesn't work for this formula
            }
        } else {
            continue;
        }
        match registry.get(buff_id) {
            Some(BuffResolutionStrategy::DirectEfficiency { value }) => {
                total += value;
            }
            Some(BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. }) => {
                total += efficiency;
            }
            _ => {}
        }
    }
    total
}

/// Compute the total order limit contribution for an operator in a room.
fn compute_order_limit(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> i32 {
    let mut total: i32 = 0;
    for buff_id in &op.available_buffs {
        if let Some(buff) = building_data.buffs.get(buff_id) {
            if buff.room_type != room_type {
                continue;
            }
            if let Some(formula) = formula_type
                && !buff.targets.is_empty()
                && !buff.targets.iter().any(|t| t == formula)
            {
                continue;
            }
        } else {
            continue;
        }
        if let Some(BuffResolutionStrategy::EfficiencyWithOrderLimit { order_limit, .. }) =
            registry.get(buff_id)
        {
            total += order_limit;
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
            let direct_eff =
                compute_direct_efficiency(op, room_type, formula_type, registry, building_data);
            TeammateInfo {
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: direct_eff,
                order_limit_contribution: compute_order_limit(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                ),
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
                if let Some(formula) = formula_type
                    && !buff.targets.is_empty()
                    && !buff.targets.iter().any(|t| t == formula)
                {
                    continue; // This buff doesn't work for this formula
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
