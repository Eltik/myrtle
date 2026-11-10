use std::collections::{HashMap, HashSet};

use crate::scoring::yield_model::BaseFlows;
use crate::{
    registry::BuffResolutionStrategy,
    scoring::ledger::op_power_rank_value,
    types::{BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding, UserRoom},
    util::{is_production_room, max_stationed_at_level},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

mod auxiliary;
mod control;
mod facilities;
mod live;
mod morale;
mod production;
mod resolve;
pub mod scope;
mod teams;

use self::auxiliary::append_support_rooms;
pub(crate) use self::auxiliary::assign_auxiliary_rooms;
pub(crate) use self::control::{
    CcBonus, CcBonusAccumulator, assign_control_center, cc_bonus_for, cc_bonuses,
    cc_condition_reach, cc_marginal_over, cc_op_is_dead, cc_token_matches, other_room_skill_count,
    padding_cost, rotation_cc_plan,
};
pub use self::control::{
    CcCondition, cc_non_production_effects, cc_seated_coverage, cc_spare_seat_split,
    cc_spare_seat_value, fill_remaining_slots,
};
pub(crate) use self::facilities::{
    FUNCTIONAL_LEVEL_SUM, TRADING_MIN_LEVEL, max_stationed_for_room, parked_tokens,
};
pub use self::facilities::{
    GOLD_LINES, MEETING_LEVEL, TRAINING_LEVEL, drone_capacity, effective_facility_counts,
    working_crew,
};
pub use self::live::{
    align_rooms_to_current, compute_current_assignment, compute_current_assignment_over,
    compute_live_assignment, has_preset_shifts,
};
pub(crate) use self::morale::{
    INERT_MORALE, num_morale_swap_managers, reserve_cc_morale_seats, total_dorm_capacity,
};
pub use self::morale::{
    compute_sustained_assignment, has_drain_aura_immunity, morale_recovery,
    morale_sustained_beneficiaries, morale_swap_enabler, op_uptime, sustained_assignment_value,
    sustained_efficiency_of,
};
pub(crate) use self::production::POST_TIE_BAND;
use self::production::assign_production_rooms;
pub(crate) use self::resolve::{base_wide_relevant, dorm_residents, room_presence_relevant};
pub use self::resolve::{
    resolve_base_wide, resolve_global_pool, resolve_layout_branches, resolve_room_presence,
};
pub use self::teams::team_value;
pub(crate) use self::teams::{
    CandidateTeam, candidate_pool, compute_team_efficiency, compute_team_totals,
    enumerate_candidate_teams, gold_split_candidates, has_automation_buff, is_generic_targets,
    op_is_nullifier, room_search_score,
};
#[cfg(test)]
pub(crate) use self::teams::{SUSTAINED_AFK_DAYS, factory_cap_factor, gold_base_capacity};

/// Candidate cut per room before the exhaustive team search
const CANDIDATE_LIMIT: usize = 16;

/// `char_id` -> profile
pub fn build_op_index(operators: &[OperatorBaseProfile]) -> HashMap<&str, &OperatorBaseProfile> {
    operators.iter().map(|o| (o.char_id.as_str(), o)).collect()
}

pub fn compute_optimal_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> BaseAssignment {
    compute_optimal_assignment_with_pins(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        &[],
    )
}

/// As `compute_optimal_assignment`, plus `(char_id, room_type)` pins
pub fn compute_optimal_assignment_with_pins(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> BaseAssignment {
    optimal_inner(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        pins,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn optimal_inner(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
    cap_aware: bool,
) -> BaseAssignment {
    let pinned_seats: HashMap<String, String> = pins.iter().cloned().collect();
    if !base_wide_relevant(operators, registry) && !room_presence_relevant(operators, registry) {
        return optimal_inner_core(
            operators,
            building,
            building_data,
            registry,
            morale_drains,
            pins,
            cap_aware,
            &pinned_seats,
        );
    }
    let pinned_counts =
        effective_facility_counts(building, operators, registry, building_data, &pinned_seats);
    let layout_registry =
        resolve_layout_branches(registry, building, building_data, &pinned_counts);
    let registry = &layout_registry;
    let empty_rooms = HashMap::new();
    let pass1_registry = resolve_room_presence(
        &resolve_base_wide(registry, &HashSet::new(), &HashSet::new()),
        &empty_rooms,
        operators,
    );
    let pass1 = optimal_inner_core(
        operators,
        building,
        building_data,
        &pass1_registry,
        morale_drains,
        pins,
        cap_aware,
        &pinned_seats,
    );
    let deployed: HashSet<String> = pass1
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter().cloned())
        .collect();
    let anywhere: HashSet<String> = deployed
        .iter()
        .cloned()
        .chain(dorm_residents(building))
        .collect();
    let deployed_rooms: HashMap<String, String> = pass1
        .rooms
        .iter()
        .flat_map(|r| {
            r.operators
                .iter()
                .map(|op| (op.clone(), r.room_type.clone()))
        })
        .collect();
    let pass2_registry = resolve_room_presence(
        &resolve_base_wide(registry, &deployed, &anywhere),
        &deployed_rooms,
        operators,
    );
    let mut pass2_seats = deployed_rooms;
    pass2_seats.extend(pins.iter().cloned());
    let counts_unchanged =
        effective_facility_counts(building, operators, registry, building_data, &pass2_seats)
            == effective_facility_counts(
                building,
                operators,
                registry,
                building_data,
                &pinned_seats,
            );
    if pass2_registry == pass1_registry && counts_unchanged {
        return pass1; // nothing unlocked
    }
    optimal_inner_core(
        operators,
        building,
        building_data,
        &pass2_registry,
        morale_drains,
        pins,
        cap_aware,
        &pass2_seats,
    )
}

#[allow(clippy::too_many_arguments)]
fn optimal_inner_core(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
    cap_aware: bool,
    seats: &HashMap<String, String>,
) -> BaseAssignment {
    let facility_counts =
        effective_facility_counts(building, operators, registry, building_data, seats);
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);

    let pinned_ids: HashSet<String> = pins
        .iter()
        .filter(|(id, _)| op_index.contains_key(id.as_str()))
        .map(|(id, _)| id.clone())
        .collect();
    let assigned: HashSet<String> = pinned_ids.clone();

    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let control_slots = if control_room.is_some() {
        max_stationed_for_room(building, building_data, "CONTROL")
    } else {
        0
    };
    let cc_pins: Vec<String> = pins
        .iter()
        .filter(|(id, rt)| rt == "CONTROL" && pinned_ids.contains(id))
        .map(|(id, _)| id.clone())
        .collect();
    let bonus_slots = (control_slots - cc_pins.len() as i32).max(0);
    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .collect();

    let mut exclude_cc: HashSet<String> = HashSet::new();
    let (cc_room, room_assignments, assigned) = loop {
        let mut assigned = assigned.clone();
        let (mut cc_room, global_bonuses, cc_conditions) = assign_control_center(
            operators,
            control_room,
            bonus_slots,
            registry,
            building_data,
            &exclude_cc,
            &cc_pins,
        );
        for id in &cc_room.operators {
            assigned.insert(id.clone());
        }
        reserve_cc_morale_seats(
            &mut cc_room,
            bonus_slots,
            operators,
            &mut assigned,
            registry,
            building_data,
        );
        let room_assignments = assign_production_rooms(
            &production_rooms,
            operators,
            &mut assigned,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            &global_bonuses,
            &cc_conditions,
            morale_drains,
            cap_aware,
        );
        let newly_dead: Vec<String> = cc_room
            .operators
            .iter()
            .filter(|id| !exclude_cc.contains(*id))
            .filter(|id| {
                op_index.get(id.as_str()).is_some_and(|op| {
                    cc_op_is_dead(op, &room_assignments, &op_index, registry, building_data)
                })
            })
            .cloned()
            .collect();
        if newly_dead.is_empty() {
            break (cc_room, room_assignments, assigned);
        }
        exclude_cc.extend(newly_dead);
    };
    let mut assigned = assigned;
    assigned.extend(exclude_cc);

    let total_production_efficiency = room_assignments.iter().map(|r| r.total_efficiency).sum();

    let mut rooms = room_assignments;
    let mut cc_room = cc_room;
    for id in cc_pins.iter().rev() {
        if !cc_room.operators.contains(id) {
            cc_room.operators.insert(0, id.clone());
        }
    }
    let bench = fill_remaining_slots(
        &mut cc_room.operators,
        control_slots,
        "CONTROL",
        operators,
        building_data,
        registry,
        &mut assigned,
    );
    if let Some(cc) = cc_room.into_room() {
        rooms.insert(0, cc);
    }

    append_support_rooms(
        &mut rooms,
        pins,
        &pinned_ids,
        building,
        building_data,
        &op_index,
        registry,
    );

    assign_auxiliary_rooms(
        &mut rooms,
        building,
        building_data,
        operators,
        registry,
        morale_drains,
        &mut assigned,
        false,
    );

    {
        let mut ranked: Vec<(&OperatorBaseProfile, f64)> = operators
            .iter()
            .filter(|op| !assigned.contains(&op.char_id))
            .map(|op| {
                (
                    op,
                    op_power_rank_value(op, building_data, registry, &facility_counts),
                )
            })
            .filter(|(_, v)| *v > 0.0)
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut next = ranked.into_iter();
        for room in building.rooms.iter().filter(|r| r.room_type == "POWER") {
            if rooms.iter().any(|r| r.slot_id == room.slot_id) {
                continue;
            }
            let cap = max_stationed_at_level(building_data, "POWER", room.level).max(0) as usize;
            let crew: Vec<String> = (0..cap)
                .filter_map(|_| next.next())
                .map(|(op, _)| op.char_id.clone())
                .collect();
            if crew.is_empty() {
                continue;
            }
            let seats: HashMap<String, String> = rooms
                .iter()
                .flat_map(|r| r.operators.iter().map(|o| (o.clone(), r.room_type.clone())))
                .chain(crew.iter().map(|o| (o.clone(), "POWER".to_string())))
                .collect();
            let parked = parked_tokens(&seats, operators, registry);
            let (speed, _) = compute_team_efficiency(
                &working_crew(&crew, &parked),
                "POWER",
                None,
                None,
                &build_op_index(operators),
                registry,
                building_data,
                &facility_counts,
                building.total_dorm_levels(),
                morale_drains,
                &[],
            );
            assigned.extend(crew.iter().cloned());
            rooms.push(RoomAssignment {
                slot_id: room.slot_id.clone(),
                room_type: "POWER".to_string(),
                level: room.level,
                formula_type: None,
                total_efficiency: speed,
                operators: crew,
                ..Default::default()
            });
        }
    }

    BaseAssignment {
        rooms,
        total_production_efficiency,
        bench,
    }
}

/// Total realized daily output of a production assignment, in LMD
pub fn assignment_value(rooms: &[RoomAssignment]) -> f64 {
    base_flows(rooms).total_value()
}

fn base_flows(rooms: &[RoomAssignment]) -> BaseFlows {
    let mut flows = BaseFlows::default();
    for r in rooms {
        let speed = room_value(r.total_efficiency, &r.room_type);
        flows.add_room(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            speed,
            r.order_gold,
            r.order_value,
            r.operators.len(),
            r.order_limit,
        );
    }
    flows
}

/// Soft-capped value of a room's raw efficiency
fn room_value(raw_efficiency: f64, room_type: &str) -> f64 {
    let soft_cap = match room_type {
        "TRADING" | "MANUFACTURE" => 200.0,
        _ => return raw_efficiency,
    };
    if raw_efficiency <= soft_cap {
        raw_efficiency
    } else {
        soft_cap + (raw_efficiency - soft_cap) * 0.5
    }
}

#[allow(clippy::too_many_arguments)]
fn rebalance_rooms(
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
    let recompute = |room_type: &str, formula: Option<&str>, ops: &[String]| -> (f64, f64) {
        let (speed, value) = compute_team_efficiency(
            ops,
            room_type,
            formula,
            None,
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        (
            speed + *global_bonuses.get(room_type).unwrap_or(&0.0),
            value,
        )
    };
    let obj = |room_type: &str, speed: f64, value: f64| -> f64 {
        room_search_score(room_type, room_value(speed, room_type), value)
    };

    for _pass in 0..4 {
        let mut improved = false;
        for a in 0..rooms.len() {
            for b in (a + 1)..rooms.len() {
                if rooms[a].room_type != rooms[b].room_type
                    || rooms[a].formula_type != rooms[b].formula_type
                {
                    continue;
                }
                let base_obj = obj(
                    &rooms[a].room_type,
                    rooms[a].total_efficiency,
                    rooms[a].order_value,
                ) + obj(
                    &rooms[b].room_type,
                    rooms[b].total_efficiency,
                    rooms[b].order_value,
                );

                let mut best_delta = 0.0;
                let mut best_swap: Option<(usize, usize)> = None;

                for ia in 0..rooms[a].operators.len() {
                    for ib in 0..rooms[b].operators.len() {
                        let mut ops_a = rooms[a].operators.clone();
                        let mut ops_b = rooms[b].operators.clone();
                        std::mem::swap(&mut ops_a[ia], &mut ops_b[ib]);

                        let (sa, va) = recompute(
                            &rooms[a].room_type,
                            rooms[a].formula_type.as_deref(),
                            &ops_a,
                        );
                        let (sb, vb) = recompute(
                            &rooms[b].room_type,
                            rooms[b].formula_type.as_deref(),
                            &ops_b,
                        );
                        let new_obj =
                            obj(&rooms[a].room_type, sa, va) + obj(&rooms[b].room_type, sb, vb);
                        let delta = new_obj - base_obj;
                        if delta > best_delta + f64::EPSILON {
                            best_delta = delta;
                            best_swap = Some((ia, ib));
                        }
                    }
                }

                if let Some((ia, ib)) = best_swap {
                    let tmp = rooms[a].operators[ia].clone();
                    rooms[a].operators[ia] = rooms[b].operators[ib].clone();
                    rooms[b].operators[ib] = tmp;
                    let (sa, va) = recompute(
                        &rooms[a].room_type,
                        rooms[a].formula_type.as_deref(),
                        &rooms[a].operators,
                    );
                    let (sb, vb) = recompute(
                        &rooms[b].room_type,
                        rooms[b].formula_type.as_deref(),
                        &rooms[b].operators,
                    );
                    rooms[a].total_efficiency = sa;
                    rooms[a].order_value = va;
                    rooms[b].total_efficiency = sb;
                    rooms[b].order_value = vb;
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }
}
