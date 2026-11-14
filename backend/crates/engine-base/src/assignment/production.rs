use std::collections::{HashMap, HashSet};

use super::control::{CcCondition, fill_remaining_slots, padding_cost};
use super::teams::{
    CandidateTeam, best_automation_team, best_team_for_room, compute_team_efficiency,
    compute_team_totals, enumerate_candidate_teams, gold_split_candidates, has_automation_buff,
    has_facility_scaled_buff, op_is_formula_specialist, op_is_nullifier, room_search_score,
    score_operator_facility_only, team_is_locked,
};
use super::{CANDIDATE_LIMIT, assignment_value, build_op_index, rebalance_rooms};
use crate::{
    registry::BuffResolutionStrategy,
    types::{OperatorBaseProfile, RoomAssignment, UserRoom},
    util::{is_production_room, max_stationed_at_level},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

#[allow(clippy::too_many_arguments)]
pub(super) fn assign_production_rooms(
    rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) -> Vec<RoomAssignment> {
    assign_production_rooms_inner(
        rooms,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        global_bonuses,
        cc_conditions,
        morale_drains,
        cap_aware,
        true,
    )
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(super) fn assign_production_rooms_inner(
    rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
    retry_idle_leader: bool,
) -> Vec<RoomAssignment> {
    let assigned_before = assigned.clone();
    let op_index = build_op_index(operators);
    let mut fixed: Vec<RoomAssignment> = Vec::new();
    for room in rooms.iter().filter(|r| r.frozen) {
        let crew: Vec<String> = room
            .current_operators
            .iter()
            .filter(|id| op_index.contains_key(id.as_str()))
            .cloned()
            .collect();
        let formula = if room.room_type == "MANUFACTURE" {
            room.current_formula.as_deref()
        } else {
            None
        };
        let level = (room.room_type == "TRADING").then_some(room.level);
        let totals = compute_team_totals(
            &crew,
            &room.room_type,
            formula,
            level,
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        for id in &crew {
            assigned.insert(id.clone());
        }
        fixed.push(RoomAssignment {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            level: room.level,
            formula_type: formula.map(str::to_string),
            operators: crew,
            total_efficiency: totals.speed_pct + global,
            order_value: totals.order_value_pct,
            order_gold: totals.order_gold_pct,
            order_limit: totals.order_limit,
            locked: true,
            ledger: Vec::new(),
            fill: None,
        });
    }
    let frozen_gold = fixed
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" && r.formula_type.as_deref() == Some("F_GOLD"))
        .count();
    let total_posts = rooms.iter().filter(|r| r.room_type == "TRADING").count();

    let mut factory_rooms: Vec<&&UserRoom> = rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" && !r.frozen)
        .collect();
    factory_rooms.sort_by_key(|r| match r.current_formula.as_deref() {
        Some("F_GOLD") => 0,
        Some("F_EXP") => 1,
        _ => 2,
    });
    let trading_rooms: Vec<&&UserRoom> = rooms
        .iter()
        .filter(|r| r.room_type == "TRADING" && !r.frozen)
        .collect();

    let num_factories = factory_rooms.len();

    let mut best_assignments: Vec<RoomAssignment> = Vec::new();
    let mut best_objective: f64 = f64::NEG_INFINITY;
    let mut best_assigned_snapshot: HashSet<String> = assigned.clone();

    let fed = total_posts.saturating_sub(frozen_gold).min(num_factories);
    let splits = gold_split_candidates(&factory_rooms, fed, frozen_gold);

    for num_gold in splits {
        let mut trial_assigned = assigned.clone();
        let mut trial_rooms: Vec<RoomAssignment> = fixed.clone();

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
                cc_conditions,
                morale_drains,
                cap_aware,
            );
            trial_rooms.push(room_assignment);
        }

        let posts = assign_trading_rooms_by_yield(
            &trading_rooms,
            operators,
            &mut trial_assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            global_bonuses,
            cc_conditions,
            morale_drains,
            cap_aware,
            &trial_rooms,
        );
        trial_rooms.extend(posts);

        let mut free = trial_rooms.split_off(fixed.len());
        rebalance_rooms(
            &mut free,
            operators,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            global_bonuses,
            cc_conditions,
            morale_drains,
        );

        let mut scored = trial_rooms;
        scored.extend(free.iter().cloned());
        let trial_objective = assignment_value(&scored);
        if trial_objective > best_objective {
            best_objective = trial_objective;
            best_assignments = free;
            best_assigned_snapshot = trial_assigned;
        }
    }

    *assigned = best_assigned_snapshot;

    if retry_idle_leader {
        let op_index = build_op_index(operators);
        let facility_value = |op: &OperatorBaseProfile, formula: Option<&str>| {
            score_operator_facility_only(
                op,
                "MANUFACTURE",
                formula,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
            )
        };
        let leader = operators
            .iter()
            .filter(|op| !assigned.contains(&op.char_id) && has_automation_buff(op, registry))
            .map(|op| (op, facility_value(op, None)))
            .filter(|(_, v)| *v > 0.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(op, _)| op);
        let partner = best_assignments
            .iter()
            .filter(|r| r.room_type == "MANUFACTURE")
            .flat_map(|r| {
                r.operators
                    .iter()
                    .map(move |id| (r.slot_id.clone(), r.formula_type.clone(), id))
            })
            .filter_map(|(slot, formula, id)| {
                let op = op_index.get(id.as_str())?;
                (has_facility_scaled_buff(op, registry) && !has_automation_buff(op, registry)).then(
                    || {
                        (
                            slot,
                            formula.clone(),
                            id.clone(),
                            facility_value(op, formula.as_deref()),
                        )
                    },
                )
            })
            .filter(|(_, _, _, v)| *v > 0.0)
            .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        if let (Some(leader), Some((slot, formula, partner_id, _))) = (leader, partner)
            && let Some(forced_room) = rooms.iter().find(|r| r.slot_id == slot)
        {
            let mut seeded = assigned_before;
            seeded.insert(leader.char_id.clone());
            seeded.insert(partner_id.clone());
            let others: Vec<&UserRoom> = rooms
                .iter()
                .filter(|r| r.slot_id != slot)
                .copied()
                .collect();
            let mut trial = assign_production_rooms_inner(
                &others,
                operators,
                &mut seeded,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
                cap_aware,
                false,
            );
            let pair = vec![leader.char_id.clone(), partner_id];
            let (speed, _) = compute_team_efficiency(
                &pair,
                "MANUFACTURE",
                formula.as_deref(),
                None,
                &op_index,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                morale_drains,
                cc_conditions,
            );
            trial.push(RoomAssignment {
                slot_id: forced_room.slot_id.clone(),
                room_type: "MANUFACTURE".to_string(),
                level: forced_room.level,
                formula_type: formula,
                operators: pair,
                total_efficiency: speed + *global_bonuses.get("MANUFACTURE").unwrap_or(&0.0),
                order_value: 0.0,
                order_gold: 0.0,
                order_limit: None,
                locked: true,
                ledger: Vec::new(),
                fill: None,
            });
            let mut fixed_and_trial = fixed.clone();
            fixed_and_trial.extend(trial.iter().cloned());
            let mut fixed_and_best = fixed.clone();
            fixed_and_best.extend(best_assignments.iter().cloned());
            if assignment_value(&fixed_and_trial) > assignment_value(&fixed_and_best) + 1e-9 {
                trial.retain(|r| !fixed.iter().any(|f| f.slot_id == r.slot_id));
                best_assignments = trial;
                *assigned = seeded;
                let mut out = fixed;
                out.extend(best_assignments);
                return out;
            }
        }
    }

    poach_facility_partners(
        &mut best_assignments,
        rooms,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        global_bonuses,
        cc_conditions,
        morale_drains,
        cap_aware,
    );

    reallocate_across_formulas(
        &mut best_assignments,
        operators,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        global_bonuses,
        cc_conditions,
        morale_drains,
    );

    pad_production_rooms(
        &mut best_assignments,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        global_bonuses,
        cc_conditions,
        morale_drains,
    );

    let mut out = fixed;
    out.extend(best_assignments);
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn poach_facility_partners(
    rooms: &mut Vec<RoomAssignment>,
    user_rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) {
    let op_index = build_op_index(operators);
    let auto_rooms: Vec<usize> = rooms
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            r.room_type == "MANUFACTURE"
                && r.operators.iter().any(|id| {
                    op_index
                        .get(id.as_str())
                        .is_some_and(|op| has_automation_buff(op, registry))
                })
        })
        .map(|(i, _)| i)
        .collect();
    if auto_rooms.is_empty() {
        return;
    }
    for &ai in &auto_rooms {
        let room_max = user_rooms
            .iter()
            .find(|u| u.slot_id == rooms[ai].slot_id)
            .map_or(0, |u| {
                max_stationed_at_level(building_data, &u.room_type, u.level)
            });
        let partners: Vec<(usize, String)> = rooms
            .iter()
            .enumerate()
            .filter(|(di, r)| *di != ai && r.room_type == "MANUFACTURE" && !auto_rooms.contains(di))
            .flat_map(|(di, r)| r.operators.iter().map(move |id| (di, id.clone())))
            .filter(|(_, id)| {
                op_index.get(id.as_str()).is_some_and(|op| {
                    has_facility_scaled_buff(op, registry) && !has_automation_buff(op, registry)
                })
            })
            .collect();
        for (di, partner) in partners {
            let mut new_auto = rooms[ai].operators.clone();
            if (new_auto.len() as i32) >= room_max {
                let Some(weakest) = new_auto
                    .iter()
                    .enumerate()
                    .filter(|(_, id)| {
                        op_index
                            .get(id.as_str())
                            .is_some_and(|op| !has_automation_buff(op, registry))
                            || new_auto.len() > 1
                    })
                    .min_by(|(_, a), (_, b)| {
                        let va = op_index.get(a.as_str()).map_or(0.0, |op| {
                            score_operator_facility_only(
                                op,
                                "MANUFACTURE",
                                rooms[ai].formula_type.as_deref(),
                                registry,
                                building_data,
                                facility_counts,
                                total_dorm_levels,
                            )
                        });
                        let vb = op_index.get(b.as_str()).map_or(0.0, |op| {
                            score_operator_facility_only(
                                op,
                                "MANUFACTURE",
                                rooms[ai].formula_type.as_deref(),
                                registry,
                                building_data,
                                facility_counts,
                                total_dorm_levels,
                            )
                        });
                        va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                else {
                    continue;
                };
                new_auto.remove(weakest);
            }
            new_auto.push(partner.clone());
            let (auto_speed, _) = compute_team_efficiency(
                &new_auto,
                "MANUFACTURE",
                rooms[ai].formula_type.as_deref(),
                None,
                &op_index,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                morale_drains,
                cc_conditions,
            );
            let Some(donor_user) = user_rooms.iter().find(|u| u.slot_id == rooms[di].slot_id)
            else {
                continue;
            };
            let mut trial_assigned: HashSet<String> = assigned.clone();
            for id in &rooms[di].operators {
                trial_assigned.remove(id);
            }
            for id in &rooms[ai].operators {
                trial_assigned.remove(id);
            }
            for id in &new_auto {
                trial_assigned.insert(id.clone());
            }
            let donor = assign_single_room(
                donor_user,
                rooms[di].formula_type.as_deref(),
                operators,
                &mut trial_assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
                cap_aware,
            );
            let mut trial = rooms.clone();
            trial[ai].operators = new_auto.clone();
            trial[ai].total_efficiency =
                auto_speed + *global_bonuses.get("MANUFACTURE").unwrap_or(&0.0);
            trial[di] = donor;
            if assignment_value(&trial) > assignment_value(rooms) + 1e-9 {
                *rooms = trial;
                *assigned = trial_assigned;
                break;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn reallocate_across_formulas(
    rooms: &mut [RoomAssignment],
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) {
    let op_index = build_op_index(operators);
    let scored = |tpl: &RoomAssignment, ops: Vec<String>| -> RoomAssignment {
        let totals = compute_team_totals(
            &ops,
            &tpl.room_type,
            tpl.formula_type.as_deref(),
            Some(tpl.level),
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        let global = *global_bonuses.get(&tpl.room_type).unwrap_or(&0.0);
        RoomAssignment {
            operators: ops,
            total_efficiency: totals.speed_pct + global,
            order_value: totals.order_value_pct,
            order_gold: totals.order_gold_pct,
            order_limit: totals.order_limit,
            ..tpl.clone()
        }
    };

    for _ in 0..3 {
        let base_val = assignment_value(rooms);
        let mut best: Option<(usize, usize, usize)> = None;
        let mut best_delta = 1e-6;

        for ai in 0..rooms.len() {
            if !is_production_room(&rooms[ai].room_type) || rooms[ai].formula_type.is_none() {
                continue;
            }
            let fa = rooms[ai].formula_type.clone();
            let fa_ref = fa.as_deref();
            for gi in 0..rooms[ai].operators.len() {
                let g_id = rooms[ai].operators[gi].clone();
                let Some(g_op) = op_index.get(g_id.as_str()).copied() else {
                    continue;
                };
                if op_is_formula_specialist(g_op, &rooms[ai].room_type, fa_ref, building_data) {
                    continue;
                }
                let mut a_ops = rooms[ai].operators.clone();
                a_ops.remove(gi);
                let a_new = scored(&rooms[ai], a_ops);
                for bi in 0..rooms.len() {
                    if bi == ai
                        || rooms[bi].room_type != rooms[ai].room_type
                        || rooms[bi].formula_type.is_none()
                        || rooms[bi].formula_type == fa
                    {
                        continue;
                    }
                    let max_b = max_stationed_at_level(
                        building_data,
                        &rooms[bi].room_type,
                        rooms[bi].level,
                    );
                    if rooms[bi].operators.len() as i32 >= max_b {
                        continue; // open seat only
                    }
                    let mut b_ops = rooms[bi].operators.clone();
                    b_ops.push(g_id.clone());
                    let b_new = scored(&rooms[bi], b_ops);

                    let mut trial = rooms.to_vec();
                    trial[ai] = a_new.clone();
                    trial[bi] = b_new;
                    let delta = assignment_value(&trial) - base_val;
                    if delta > best_delta {
                        best_delta = delta;
                        best = Some((ai, gi, bi));
                    }
                }
            }
        }

        let Some((ai, gi, bi)) = best else {
            break;
        };
        let g_id = rooms[ai].operators[gi].clone();
        let mut a_ops = rooms[ai].operators.clone();
        a_ops.remove(gi);
        let a_tpl = rooms[ai].clone();
        rooms[ai] = scored(&a_tpl, a_ops);
        let mut b_ops = rooms[bi].operators.clone();
        b_ops.push(g_id);
        let b_tpl = rooms[bi].clone();
        rooms[bi] = scored(&b_tpl, b_ops);
    }
}

/// Fill every production room up to its slot count with leftover operators
#[allow(clippy::too_many_arguments)]
pub(super) fn pad_production_rooms(
    rooms: &mut [RoomAssignment],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
) {
    let op_index = build_op_index(operators);
    for room in rooms.iter_mut() {
        let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        let is_auto = room
            .operators
            .iter()
            .filter_map(|id| operators.iter().find(|o| o.char_id == *id))
            .any(|op| has_automation_buff(op, registry));

        if is_auto {
            fill_remaining_slots(
                &mut room.operators,
                max_slots,
                &room.room_type,
                operators,
                building_data,
                registry,
                assigned,
            );
        } else {
            let team_nullifies = room.operators.iter().any(|id| {
                op_index.get(id.as_str()).copied().is_some_and(|op| {
                    op_is_nullifier(
                        op,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        registry,
                        building_data,
                    )
                })
            });
            while (room.operators.len() as i32) < max_slots {
                let current_score = room_search_score(
                    &room.room_type,
                    room.total_efficiency - global,
                    room.order_value,
                ) * crate::scoring::yield_model::trading_cap_factor(
                    &room.room_type,
                    room.level,
                    room.total_efficiency - global,
                    room.operators.len(),
                    room.order_limit,
                );
                let mut best_id: Option<String> = None;
                let mut best_score = f64::NEG_INFINITY;
                let mut best_speed = 0.0;
                let mut best_value = 0.0;
                let mut best_gold = 0.0;
                let mut best_limit: Option<i32> = None;
                let mut best_cost = usize::MAX;
                for op in operators {
                    if assigned.contains(&op.char_id) {
                        continue;
                    }
                    if has_automation_buff(op, registry)
                        || op_is_nullifier(
                            op,
                            &room.room_type,
                            room.formula_type.as_deref(),
                            registry,
                            building_data,
                        )
                    {
                        continue;
                    }
                    let mut trial = room.operators.clone();
                    trial.push(op.char_id.clone());
                    let totals = compute_team_totals(
                        &trial,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        Some(room.level),
                        &op_index,
                        registry,
                        building_data,
                        facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        cc_conditions,
                    );
                    let (speed, value) = (totals.speed_pct, totals.order_value_pct);
                    let score = room_search_score(&room.room_type, speed, value)
                        * crate::scoring::yield_model::trading_cap_factor(
                            &room.room_type,
                            room.level,
                            speed,
                            trial.len(),
                            totals.order_limit,
                        );
                    let cost = padding_cost(
                        op,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        registry,
                        building_data,
                        team_nullifies,
                        false,
                    );
                    let better = best_id.is_none()
                        || score > best_score + 1e-9
                        || ((score - best_score).abs() <= 1e-9 && cost < best_cost);
                    if better {
                        best_score = score;
                        best_speed = speed;
                        best_value = value;
                        best_gold = totals.order_gold_pct;
                        best_limit = totals.order_limit;
                        best_cost = cost;
                        best_id = Some(op.char_id.clone());
                    }
                }
                match best_id {
                    Some(id) if best_score >= current_score - 1e-9 => {
                        assigned.insert(id.clone());
                        room.operators.push(id);
                        room.total_efficiency = best_speed + global;
                        room.order_value = best_value;
                        room.order_gold = best_gold;
                        room.order_limit = best_limit;
                    }
                    _ => break, // no harmless filler left
                }
            }
        }
    }
}

/// Top search candidates a trading post's yield pick compares
pub(super) const YIELD_PICK_WIDTH: usize = 8;

/// Trading post crews chosen together by realized yield beside `context`
#[allow(clippy::too_many_arguments)]
pub(super) fn assign_trading_rooms_by_yield(
    rooms: &[&&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
    context: &[RoomAssignment],
) -> Vec<RoomAssignment> {
    let options: Vec<Vec<RoomAssignment>> = rooms
        .iter()
        .map(|room| {
            let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
            let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
            let candidates = enumerate_candidate_teams(
                &room.room_type,
                room.level,
                None,
                operators,
                assigned,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                max_slots,
                cc_conditions,
                morale_drains,
                cap_aware,
                CANDIDATE_LIMIT,
                false,
                0,
            )
            .into_iter()
            .filter(|c| c.score > 0.0)
            .collect::<Vec<_>>();
            diverse_shortlist(candidates, YIELD_PICK_WIDTH)
                .into_iter()
                .map(|team| RoomAssignment {
                    slot_id: room.slot_id.clone(),
                    room_type: room.room_type.clone(),
                    level: room.level,
                    formula_type: None,
                    operators: team.ops,
                    total_efficiency: team.speed + global,
                    order_value: team.value,
                    order_gold: team.gold,
                    order_limit: team.order_limit,
                    locked: false,
                    ledger: Vec::new(),
                    fill: None,
                })
                .collect()
        })
        .collect();

    let chosen = best_post_combination(&options, context);
    let mut out: Vec<Option<RoomAssignment>> = vec![None; rooms.len()];
    for (i, pick) in chosen.iter().enumerate() {
        let Some(k) = pick else { continue };
        let mut post = options[i][*k].clone();
        post.locked = team_is_locked(
            &post.operators,
            &post.room_type,
            None,
            registry,
            building_data,
        );
        for id in &post.operators {
            assigned.insert(id.clone());
        }
        out[i] = Some(post);
    }
    out.into_iter()
        .enumerate()
        .map(|(i, post)| {
            post.unwrap_or_else(|| {
                assign_single_room(
                    rooms[i],
                    None,
                    operators,
                    assigned,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                    global_bonuses,
                    cc_conditions,
                    morale_drains,
                    cap_aware,
                )
            })
        })
        .collect()
}

pub(super) fn diverse_shortlist(
    candidates: Vec<CandidateTeam>,
    width: usize,
) -> Vec<CandidateTeam> {
    let mut shortlist: Vec<CandidateTeam> = candidates.iter().take(width).cloned().collect();
    let anchors: Vec<String> = candidates
        .first()
        .map(|best| best.ops.clone())
        .unwrap_or_default();
    for op in &anchors {
        if let Some(team) = candidates
            .iter()
            .find(|c| !c.ops.contains(op) && !shortlist.iter().any(|s| s.ops == c.ops))
        {
            shortlist.push(team.clone());
        }
    }
    shortlist
}

/// Coupled-value fraction within which post combinations tie
pub const POST_TIE_BAND: f64 = 0.01;

/// Per post, its crew index in the best disjoint combination; `None` where none fits
pub(super) fn best_post_combination(
    options: &[Vec<RoomAssignment>],
    context: &[RoomAssignment],
) -> Vec<Option<usize>> {
    struct Search<'a> {
        options: &'a [Vec<RoomAssignment>],
        context_len: usize,
        trial: Vec<RoomAssignment>,
        used: HashSet<String>,
        chosen: Vec<Option<usize>>,
        /// Every complete combination: picks, coupled value, posts' LMD
        complete: Vec<(Vec<Option<usize>>, f64, f64)>,
    }
    impl Search<'_> {
        fn go(&mut self, idx: usize) {
            if idx == self.options.len() {
                let value = assignment_value(&self.trial);
                let posts_lmd: f64 = self.trial[self.context_len..]
                    .iter()
                    .map(|r| {
                        crate::scoring::yield_model::room_yield(
                            &r.room_type,
                            r.formula_type.as_deref(),
                            r.level,
                            r.total_efficiency,
                            r.order_value,
                            r.operators.len(),
                            r.order_limit,
                        )
                        .lmd_per_day
                    })
                    .sum();
                self.complete.push((self.chosen.clone(), value, posts_lmd));
                return;
            }
            let mut any = false;
            for k in 0..self.options[idx].len() {
                let cand = &self.options[idx][k];
                if cand.operators.iter().any(|o| self.used.contains(o)) {
                    continue;
                }
                any = true;
                let cand = cand.clone();
                for o in &cand.operators {
                    self.used.insert(o.clone());
                }
                self.trial.push(cand.clone());
                self.chosen.push(Some(k));
                self.go(idx + 1);
                self.chosen.pop();
                self.trial.pop();
                for o in &cand.operators {
                    self.used.remove(o);
                }
            }
            if !any {
                self.chosen.push(None);
                self.go(idx + 1);
                self.chosen.pop();
            }
        }
    }
    let mut search = Search {
        options,
        context_len: context.len(),
        trial: context.to_vec(),
        used: HashSet::new(),
        chosen: Vec::new(),
        complete: Vec::new(),
    };
    search.go(0);
    let top = search
        .complete
        .iter()
        .map(|(_, v, _)| *v)
        .fold(f64::NEG_INFINITY, f64::max);
    let floor = top.abs().mul_add(-POST_TIE_BAND, top);
    search
        .complete
        .into_iter()
        .filter(|(_, v, _)| *v >= floor - 1e-9)
        .fold(
            None::<(Vec<Option<usize>>, f64)>,
            |best, (chosen, _, lmd)| {
                if best.as_ref().is_none_or(|(_, l)| lmd > *l + 1e-9) {
                    Some((chosen, lmd))
                } else {
                    best
                }
            },
        )
        .map_or_else(|| vec![None; options.len()], |(chosen, _)| chosen)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn assign_single_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) -> RoomAssignment {
    let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
    let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);

    let (normal_ops, normal_speed, normal_value, normal_gold, normal_limit) = best_team_for_room(
        room,
        formula_type,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        max_slots,
        cc_conditions,
        morale_drains,
        cap_aware,
    );

    let (auto_ops, auto_speed) = if room.room_type == "MANUFACTURE" {
        best_automation_team(
            room,
            formula_type,
            operators,
            assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            max_slots,
        )
    } else {
        (Vec::new(), f64::NEG_INFINITY)
    };

    let normal_score = room_search_score(&room.room_type, normal_speed, normal_value);
    let auto_score = room_search_score(&room.room_type, auto_speed, 0.0);
    let (room_ops, speed, value, gold, order_limit) =
        if auto_score > normal_score && !auto_ops.is_empty() {
            (auto_ops, auto_speed, 0.0, 0.0, None)
        } else {
            (
                normal_ops,
                normal_speed,
                normal_value,
                normal_gold,
                normal_limit,
            )
        };

    for id in &room_ops {
        assigned.insert(id.clone());
    }

    let locked = team_is_locked(
        &room_ops,
        &room.room_type,
        formula_type,
        registry,
        building_data,
    );
    RoomAssignment {
        slot_id: room.slot_id.clone(),
        room_type: room.room_type.clone(),
        level: room.level,
        formula_type: formula_type.map(std::string::ToString::to_string),
        operators: room_ops,
        total_efficiency: speed + global,
        order_value: value,
        order_gold: gold,
        order_limit,
        locked,
        ledger: Vec::new(),
        fill: None,
    }
}
