use std::collections::{HashMap, HashSet};

use super::auxiliary::support_room_figure;
use super::build_op_index;
use super::control::{
    CcBonusAccumulator, cc_bonuses, cc_non_production_effects, seat_grant_conditions,
};
use super::facilities::effective_facility_counts_with_inert;
use super::morale::INERT_MORALE;
use super::resolve::{
    base_wide_relevant, resolve_base_wide, resolve_global_pool, resolve_layout_branches,
    resolve_room_presence, room_presence_relevant,
};
use super::teams::{compute_team_totals, team_is_locked};
use crate::{
    registry::BuffResolutionStrategy,
    scoring::pools::{POOL_PREFIX, settle_current_pools},
    types::{BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding, UserRoom},
    util::is_production_room,
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

/// Every stationed operator's room type this shift
pub fn stationed_seats(building: &UserBuilding, shift: Option<usize>) -> HashMap<String, String> {
    building
        .rooms
        .iter()
        .flat_map(|r| {
            room_ops_for_shift(r, shift)
                .into_iter()
                .map(|op| (op, r.room_type.clone()))
        })
        .collect()
}

/// A plan's seats as `char id -> room type`
pub fn seats_of(asn: &BaseAssignment) -> HashMap<String, String> {
    asn.rooms
        .iter()
        .flat_map(|r| {
            r.operators
                .iter()
                .map(|op| (op.clone(), r.room_type.clone()))
        })
        .collect()
}

pub(super) fn room_ops_for_shift(room: &UserRoom, shift: Option<usize>) -> Vec<String> {
    match shift {
        Some(i) => room
            .preset_shifts
            .get(i)
            .cloned()
            .unwrap_or_else(|| room.current_operators.clone()),
        None => room.current_operators.clone(),
    }
}

#[must_use]
pub fn has_preset_shifts(building: &UserBuilding) -> bool {
    building.rooms.iter().any(|r| r.preset_shifts.len() > 1)
}

/// Evaluate the player's current base as stationed
#[must_use]
pub fn compute_current_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
) -> BaseAssignment {
    compute_live_assignment(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        shift,
        &HashMap::new(),
    )
}

#[must_use]
pub fn compute_current_assignment_over(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
    hours: f64,
) -> BaseAssignment {
    compute_live_assignment_over(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        shift,
        &HashMap::new(),
        Some(hours),
    )
}

#[must_use]
pub fn compute_live_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
    live_morale: &HashMap<String, f64>,
) -> BaseAssignment {
    compute_live_assignment_over(
        operators,
        building,
        building_data,
        registry,
        morale_drains,
        shift,
        live_morale,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn compute_live_assignment_over(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
    live_morale: &HashMap<String, f64>,
    hours: Option<f64>,
) -> BaseAssignment {
    let inert: HashSet<String> = live_morale
        .iter()
        .filter(|(_, m)| **m < INERT_MORALE)
        .map(|(id, _)| id.clone())
        .collect();
    let mut facility_counts = effective_facility_counts_with_inert(
        building,
        operators,
        registry,
        building_data,
        &stationed_seats(building, shift),
        &inert,
    );
    if let Some(hours) = hours {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        facility_counts.insert(
            crate::scoring::clause::TENURE_HOURS.to_string(),
            hours.round().max(1.0) as usize,
        );
    }
    let settled = settle_current_pools(building, operators, registry, building_data, live_morale);
    for (resource, points) in &settled {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        facility_counts.insert(
            format!("{POOL_PREFIX}{resource}"),
            points.round().max(0.0) as usize,
        );
    }
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);

    let live_counts = effective_facility_counts_with_inert(
        building,
        operators,
        registry,
        building_data,
        &stationed_seats(building, shift),
        &inert,
    );
    let layout_registry = resolve_layout_branches(registry, building, building_data, &live_counts);
    let registry = &layout_registry;
    let needs_presence =
        base_wide_relevant(operators, registry) || room_presence_relevant(operators, registry);
    let needs_global_pool = registry
        .values()
        .any(|s| matches!(s, BuffResolutionStrategy::GlobalPoolScaling { .. }));
    let resolved_registry;
    let registry: &HashMap<String, BuffResolutionStrategy> = if needs_presence || needs_global_pool
    {
        let mut resolved = resolve_global_pool(registry, &settled);
        if needs_presence {
            let working: HashSet<String> = building
                .rooms
                .iter()
                .filter(|r| r.room_type != "DORMITORY")
                .flat_map(|r| room_ops_for_shift(r, shift))
                .collect();
            let stationed: HashMap<String, String> = building
                .rooms
                .iter()
                .flat_map(|r| {
                    room_ops_for_shift(r, shift)
                        .into_iter()
                        .map(|op| (op, r.room_type.clone()))
                })
                .collect();
            let anywhere: HashSet<String> = stationed.keys().cloned().collect();
            resolved = resolve_room_presence(
                &resolve_base_wide(&resolved, &working, &anywhere),
                &stationed,
                operators,
            );
        }
        resolved_registry = resolved;
        &resolved_registry
    } else {
        registry
    };

    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let cc_ops = control_room
        .map(|cc| room_ops_for_shift(cc, shift))
        .unwrap_or_default();
    let mut acc = CcBonusAccumulator::default();
    for op in cc_ops
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
    {
        acc.add_from(op, &cc_bonuses(op, registry, building_data));
    }
    let (global_bonuses, mut cc_conditions) = acc.finish();
    cc_conditions.extend(seat_grant_conditions(registry));

    let mut rooms: Vec<RoomAssignment> = Vec::new();

    let ledger_ctx = crate::scoring::skill_ledger::LedgerCtx {
        op_index: &op_index,
        registry,
        building_data,
        facility_counts: &facility_counts,
        total_dorm_levels,
        morale_drains,
    };

    let mut total = 0.0;
    for room in building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let seated: Vec<String> = room_ops_for_shift(room, shift)
            .into_iter()
            .filter(|id| op_index.contains_key(id.as_str()))
            .collect();
        let ops: Vec<String> = seated
            .iter()
            .filter(|id| !inert.contains(*id))
            .cloned()
            .collect();
        let formula = if room.room_type == "MANUFACTURE" {
            room.current_formula.clone()
        } else {
            None
        };
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        let totals = compute_team_totals(
            &ops,
            &room.room_type,
            formula.as_deref(),
            Some(room.level),
            &op_index,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            morale_drains,
            &cc_conditions,
        );
        let (speed, value, gold) = (
            totals.speed_pct,
            totals.order_value_pct,
            totals.order_gold_pct,
        );
        let eff = speed + global;
        total += eff;
        let locked = team_is_locked(
            &ops,
            &room.room_type,
            formula.as_deref(),
            registry,
            building_data,
        );
        let ledger = crate::scoring::skill_ledger::production_room_ledger(
            &ledger_ctx,
            &ops,
            &room.room_type,
            formula.as_deref(),
            &cc_ops,
            &global_bonuses,
            &cc_conditions,
        );
        #[allow(clippy::cast_possible_truncation)]
        let capacity_bonus: i32 = totals.capacity_delta.round() as i32;
        let fill = crate::scoring::yield_model::room_fill(
            &room.room_type,
            formula.as_deref(),
            room.level,
            eff,
            capacity_bonus,
            building_data,
        );
        rooms.push(RoomAssignment {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            level: room.level,
            formula_type: formula,
            operators: seated,
            total_efficiency: eff,
            order_value: value,
            order_gold: gold,
            order_limit: totals.order_limit,
            locked,
            ledger,
            fill,
        });
    }

    let cc_nonprod = cc_non_production_effects(&cc_ops, operators, registry, &rooms);
    for room in building
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type.as_str(), "POWER" | "MEETING" | "HIRE"))
    {
        let seated: Vec<String> = room_ops_for_shift(room, shift)
            .into_iter()
            .filter(|id| op_index.contains_key(id.as_str()))
            .collect();
        if seated.is_empty() {
            continue;
        }
        let ops: Vec<String> = seated
            .iter()
            .filter(|id| !inert.contains(*id))
            .cloned()
            .collect();
        let from_cc = cc_nonprod
            .iter()
            .find(|(rt, _)| rt == &room.room_type)
            .map_or(0.0, |(_, v)| *v);
        let eff = match room.room_type.as_str() {
            "POWER" => {
                compute_team_totals(
                    &ops,
                    "POWER",
                    None,
                    None,
                    &op_index,
                    registry,
                    building_data,
                    &facility_counts,
                    total_dorm_levels,
                    morale_drains,
                    &cc_conditions,
                )
                .speed_pct
            }
            _ => support_room_figure(
                &room.room_type,
                &seated,
                &ops,
                &op_index,
                registry,
                building_data,
                from_cc,
            ),
        };
        let eff = if eff == 0.0 { 0.0 } else { eff };
        rooms.push(RoomAssignment {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            level: room.level,
            formula_type: None,
            operators: seated,
            total_efficiency: eff,
            ..Default::default()
        });
    }

    if let Some(cc) = control_room
        && !cc_ops.is_empty()
    {
        let cc_row = RoomAssignment {
            slot_id: cc.slot_id.clone(),
            room_type: "CONTROL".to_string(),
            level: cc.level,
            formula_type: None,
            total_efficiency: global_bonuses.values().sum(),
            ledger: crate::scoring::skill_ledger::control_room_ledger(&ledger_ctx, &cc_ops, &rooms),
            operators: cc_ops.clone(),
            ..Default::default()
        };
        rooms.insert(0, cc_row);
    }

    BaseAssignment {
        rooms,
        total_production_efficiency: total,
        bench: Vec::new(),
    }
}

/// Re-slot a proposal's rooms onto the slots the player already runs the same crews in
pub fn align_rooms_to_current(building: &UserBuilding, proposal: &mut BaseAssignment) {
    let mut by_kind: HashMap<(String, i32), Vec<usize>> = HashMap::new();
    for (i, r) in proposal.rooms.iter().enumerate() {
        by_kind
            .entry((r.room_type.clone(), r.level))
            .or_default()
            .push(i);
    }
    for ((room_type, level), idxs) in by_kind {
        if idxs.len() < 2 {
            continue;
        }
        let mut free_slots: Vec<&UserRoom> = building
            .rooms
            .iter()
            .filter(|u| u.room_type == room_type && u.level == level)
            .collect();
        let mut pending = idxs.clone();
        while !pending.is_empty() && !free_slots.is_empty() {
            let mut best: Option<(usize, usize, usize, bool)> = None;
            for (pi, &i) in pending.iter().enumerate() {
                for (si, slot) in free_slots.iter().enumerate() {
                    let overlap = proposal.rooms[i]
                        .operators
                        .iter()
                        .filter(|o| slot.current_operators.contains(o))
                        .count();
                    let same = proposal.rooms[i].slot_id == slot.slot_id;
                    if best.is_none_or(|(_, _, o, s)| overlap > o || (overlap == o && same && !s)) {
                        best = Some((pi, si, overlap, same));
                    }
                }
            }
            let Some((pi, si, _, _)) = best else { break };
            let i = pending.remove(pi);
            let slot = free_slots.remove(si);
            proposal.rooms[i].slot_id = slot.slot_id.clone();
        }
    }
}
