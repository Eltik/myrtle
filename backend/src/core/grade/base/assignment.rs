use std::collections::{HashMap, HashSet};

use crate::core::{
    gamedata::types::building::{Buff, BuildingDataFile},
    grade::base::{
        buff_registry::BuffResolutionStrategy,
        evaluate::evaluate_buff,
        types::{
            BaseAssignment, EvalContext, OperatorBaseProfile, RoomAssignment, RoomRotation,
            RotationAssignment, RotationMember, RotationSet, RotationSetRoom, TeammateInfo,
            UserBuilding, UserRoom,
        },
        util::{is_production_room, max_stationed_at_level},
    },
};

/// Most candidate operators considered per room before exhaustively searching
/// team combinations. The optimal team is essentially always among the top
/// candidates by optimistic bound (synergy enablers are boosted into this set),
/// so 16 covers realistic optima while keeping C(16,3) combinations cheap.
const CANDIDATE_LIMIT: usize = 16;

/// A trading post's base order limit (orders it can bank) before operator buffs,
/// constant across levels. Operators that slash it (Degenbrecher: -6, "minimum 1")
/// strand the post far below capacity, throttling sustained output. The game
/// enforces a floor of 1.
const BASE_TRADING_ORDER_LIMIT: i32 = 6;
const MIN_TRADING_ORDER_LIMIT: i32 = 1;

/// Build a `char_id` → profile index for O(1) lookups in the hot inner loops.
fn build_op_index(operators: &[OperatorBaseProfile]) -> HashMap<&str, &OperatorBaseProfile> {
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

/// As `compute_optimal_assignment`, but with a set of `(char_id, room_type)` PINS - operators
/// the plan must station in a specific room and reserve from everything else. This is how the
/// base-wide resource economy (perception) wires into the real optimizer rather than living in
/// a side panel: its support generators (Mulberry -> Office, the dorm generators, Ling/Dusk ->
/// Control Center) and its morale-swap manager (Fiammetta -> dormitory) are reserved so
/// production and the rest of the Control Center are optimized AROUND them (never double-booking
/// an operator), and the rooms they occupy are surfaced in the plan. Empty pins reproduces the
/// plain optimum exactly, so non-243 / no-economy bases are unaffected.
pub fn compute_optimal_assignment_with_pins(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> BaseAssignment {
    optimal_inner(operators, building, building_data, registry, morale_drains, pins, false)
}

/// The shared optimizer body. `cap_aware` switches production-team scoring to the SUSTAINED
/// objective (factory product-buffer stall during AFK), so a high-capacity team is preferred
/// where it out-produces a denser team over a long unattended stretch; `false` is the plain
/// peak objective used everywhere else.
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
    let facility_counts = effective_facility_counts(building, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);

    // Reserve every pinned operator up front so neither production nor Control-Center padding
    // can claim one - the economy needs them where perception placed them.
    let pinned_ids: HashSet<String> = pins
        .iter()
        .filter(|(id, _)| op_index.contains_key(id.as_str()))
        .map(|(id, _)| id.clone())
        .collect();
    let assigned: HashSet<String> = pinned_ids.clone();

    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    // Only staff the Control Center if one is actually built - otherwise we'd
    // strip CC operators out of the production pool for a room that doesn't exist.
    let control_slots = if control_room.is_some() {
        max_stationed_for_room(building, building_data, "CONTROL")
    } else {
        0
    };
    // Control-Center pins (Ling/Dusk) are forced into the CC, so leave their seats out of the
    // bonus-greedy selection and prepend them afterwards.
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

    // Assign the Control Center, then production. If a CC operator turns out to be
    // dead weight - its only effect is a conditional the optimized teams never
    // satisfy (SilverAsh's Kjerag gate when no post fields 3 Kjerag) - drop it and
    // reselect, so its seat goes to an operator that actually contributes.
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
        );
        for id in &cc_room.operators {
            assigned.insert(id.clone());
        }
        // Reserve spare CC seats for global morale-recovery operators (Chongyue,
        // Wiš'adel) BEFORE production padding can claim them as zero-value fillers:
        // unlike a benchwarmer they lift every working operator's sustain.
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
    // Keep the dropped dead-weight operators benched - otherwise they would be
    // re-added to the Control Center as zero-value fillers, resurfacing the very
    // recommendation we just removed.
    assigned.extend(exclude_cc);

    let total_production_efficiency = room_assignments.iter().map(|r| r.total_efficiency).sum();

    let mut rooms = room_assignments;
    // Surface the Control Center so its operators (and the global bonuses they
    // provide to every production room) are visible in the recommendation.
    let mut cc_room = cc_room;
    // Force perception's Control-Center generators (Ling/Dusk) into the CC: their payoff is the
    // resource economy they feed, not a CC production bonus, so the bonus-greedy selector skips
    // them. They are already reserved, so prepend them and top up any remaining seats.
    for id in cc_pins.iter().rev() {
        if !cc_room.operators.contains(id) {
            cc_room.operators.insert(0, id.clone());
        }
    }
    // Fill any remaining CC slots with leftover operators - production has
    // already claimed everyone it benefits from, so these are true spares.
    fill_remaining_slots(
        &mut cc_room.operators,
        control_slots,
        "CONTROL",
        operators,
        building_data,
        &mut assigned,
    );
    if let Some(cc) = cc_room.into_room() {
        rooms.insert(0, cc);
    }

    // Surface the support generators and morale-swap manager in the rooms they occupy (Office,
    // dormitories) so the plan shows the full staffing the economy needs, not just the
    // production rooms its pool boosts. These rooms carry no production efficiency (their value
    // is already credited to the consumers they power), so they don't change the yield math.
    append_support_rooms(&mut rooms, pins, &pinned_ids, building, building_data);

    // Staff the auxiliary facilities (HR Office, Reception Room) with the best leftover
    // operators, so they aren't left empty in the plan.
    assign_auxiliary_rooms(&mut rooms, building, building_data, operators, registry, &mut assigned);

    BaseAssignment {
        rooms,
        total_production_efficiency,
    }
}

/// An operator's strength in a non-production facility (HR Office, Reception Room): the largest
/// value any of its base skills for that room resolves to (HR contacting speed, reception
/// clue/credit rate). Zero if it has no skill for the room.
fn aux_room_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    op.available_buffs
        .iter()
        .filter(|b| building_data.buffs.get(*b).is_some_and(|buff| buff.room_type == room_type))
        .filter_map(|b| match registry.get(b) {
            Some(BuffResolutionStrategy::NonProduction { value }) => Some(*value),
            _ => None,
        })
        .fold(0.0, f64::max)
}

/// Fill the auxiliary facilities (HR Office, Reception Room) with the best AVAILABLE operators -
/// the optimizer otherwise only staffs production and the Control Center, leaving these empty.
/// Operators are ranked by their room base-skill strength and reserved so the rest of the plan
/// doesn't double-book them; a room already staffed (e.g. an Office held by a perception support
/// generator) is left alone.
fn assign_auxiliary_rooms(
    rooms: &mut Vec<RoomAssignment>,
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    assigned: &mut HashSet<String>,
) {
    for room_type in ["HIRE", "MEETING"] {
        for room in building.rooms.iter().filter(|r| r.room_type == room_type) {
            if rooms.iter().any(|r| r.slot_id == room.slot_id) {
                continue; // already staffed (e.g. by a perception pin)
            }
            let cap = max_stationed_at_level(building_data, room_type, room.level).max(0) as usize;
            if cap == 0 {
                continue;
            }
            let mut ranked: Vec<(&str, f64)> = operators
                .iter()
                .filter(|op| !assigned.contains(&op.char_id))
                .map(|op| (op.char_id.as_str(), aux_room_value(op, room_type, registry, building_data)))
                .filter(|(_, v)| *v > 0.0)
                .collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            let crew: Vec<String> = ranked.iter().take(cap).map(|(id, _)| (*id).to_string()).collect();
            if crew.is_empty() {
                continue;
            }
            let eff: f64 = ranked.iter().take(cap).map(|(_, v)| *v).sum();
            for id in &crew {
                assigned.insert(id.clone());
            }
            rooms.push(RoomAssignment {
                slot_id: room.slot_id.clone(),
                room_type: room_type.to_string(),
                level: room.level,
                formula_type: None,
                operators: crew,
                total_efficiency: eff,
                order_value: 0.0,
                locked: false,
            });
        }
    }
}

/// Append a room per non-production, non-Control-Center pin group (Office, dormitories),
/// distributing the pinned operators across that type's actual rooms up to each room's capacity.
fn append_support_rooms(
    rooms: &mut Vec<RoomAssignment>,
    pins: &[(String, String)],
    pinned_ids: &HashSet<String>,
    building: &UserBuilding,
    building_data: &BuildingDataFile,
) {
    let mut by_type: Vec<(&str, Vec<String>)> = Vec::new();
    for (id, rt) in pins {
        if rt == "CONTROL" || is_production_room(rt) || !pinned_ids.contains(id) {
            continue;
        }
        match by_type.iter_mut().find(|(t, _)| *t == rt.as_str()) {
            Some((_, ops)) => ops.push(id.clone()),
            None => by_type.push((rt.as_str(), vec![id.clone()])),
        }
    }
    for (room_type, ops) in by_type {
        let mut ops = ops.into_iter();
        for room in building.rooms.iter().filter(|r| r.room_type == room_type) {
            let cap = max_stationed_at_level(building_data, room_type, room.level).max(0) as usize;
            let members: Vec<String> = ops.by_ref().take(cap).collect();
            if members.is_empty() {
                continue;
            }
            rooms.push(RoomAssignment {
                slot_id: room.slot_id.clone(),
                room_type: room_type.to_string(),
                level: room.level,
                formula_type: None,
                operators: members,
                total_efficiency: 0.0,
                order_value: 0.0,
                locked: false,
            });
        }
    }
}

// ── Morale / longevity model for staggered rotation ────────────────────────
// Each working operator burns morale and must occasionally rest. A STAGGERED
// rotation swaps only the lowest-morale operator at a time, so a room is almost
// always at full strength and just one backup covers a resting main. The 24/7
// output is therefore close to peak - lowered only while a backup (weaker)
// covers, and that coverage time shrinks for low-drain operators.
//
// Constants are calibrated estimates (drain modifiers come in ~±0.25/hr units):
/// Neutral per-hour morale burn for a working operator (before skill modifiers).
const BASE_MORALE_DRAIN: f64 = 0.5;
/// Per-hour morale recovery while resting in a FULLY-DEVELOPED dormitory (faster
/// than drain, so operators spend more time working than resting). The realized
/// rate scales down with dorm level - see `morale_recovery`.
const MORALE_RECOVERY: f64 = 1.0;
/// Dorm level at which recovery reaches the modeled maximum.
const MAX_DORM_LEVEL: f64 = 5.0;
/// Floor on the recovery factor so a barely-built base still rotates (slowly).
const MIN_RECOVERY_FACTOR: f64 = 0.3;
/// How much of a resting main's output a backup recovers while covering it (a
/// good rotation piece is close, but not equal, to the operator it replaces).
const BACKUP_COVERAGE: f64 = 0.8;

/// Effective per-hour morale recovery for this base. Operators rest in dormitories,
/// whose recovery rate rises with dorm level, so a base with low-level dorms (a
/// typical 252 layout) recovers its workers more slowly: they spend more time
/// resting and the base sustains further below peak. A fully-developed dorm wing
/// (L5) recovers at the modeled maximum, matching the original calibration.
pub fn morale_recovery(building: &UserBuilding) -> f64 {
    let levels: Vec<i32> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| r.level)
        .collect();
    if levels.is_empty() {
        return MORALE_RECOVERY * MIN_RECOVERY_FACTOR;
    }
    let avg = f64::from(levels.iter().sum::<i32>()) / levels.len() as f64;
    MORALE_RECOVERY * (avg / MAX_DORM_LEVEL).clamp(MIN_RECOVERY_FACTOR, 1.0)
}

/// Total operators the base can rest at once: the sum of its dormitory capacities.
/// A staggered rotation only keeps its workers fresh if the dorms can hold everyone
/// who needs to rest at a time (the "you can only rest 20, so you run 3 overlapping
/// sets" reality); when they can't, sustained output is throttled.
fn total_dorm_capacity(building: &UserBuilding, building_data: &BuildingDataFile) -> i32 {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| max_stationed_at_level(building_data, "DORMITORY", r.level))
        .sum()
}

/// How much a base's sustained output is throttled because its dormitories cannot
/// rest everyone who needs to at once. `rest_demand` is the expected number resting
/// at a time (summed per-operator rest fractions); if it exceeds capacity, the
/// rotation falls behind and output scales by `capacity / rest_demand`.
fn dorm_capacity_factor(rest_demand: f64, capacity: i32) -> f64 {
    let cap = f64::from(capacity.max(0));
    if rest_demand <= cap || rest_demand <= 0.0 {
        1.0
    } else {
        cap / rest_demand
    }
}

/// Morale per hour this operator restores to workers in OTHER buildings when seated
/// in the Control Center (Chongyue, Wiš'adel). Unlike the Control-Center-only morale
/// trickle, this lifts every working operator's recovery, so it is a base-wide
/// sustain boost - which is what makes a morale operator worth a CC seat over a
/// production buff whose gate never fires.
fn cc_global_morale_recovery(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            if buff.room_type != "CONTROL"
                || !buff.description.to_lowercase().contains("other building")
            {
                return None;
            }
            match registry.get(b) {
                Some(BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour, ..
                }) => Some(*recovery_per_hour),
                _ => None,
            }
        })
        .sum()
}

/// Fill the Control Center's spare seats with idle global morale-recovery operators
/// (strongest first), reserving them from production padding. Each lifts the whole
/// base's sustain, so it is a better use of a spare seat than a zero-value filler.
fn reserve_cc_morale_seats(
    cc_room: &mut ControlCenter,
    control_slots: i32,
    operators: &[OperatorBaseProfile],
    assigned: &mut HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) {
    let mut morale_ops: Vec<(&OperatorBaseProfile, f64)> = operators
        .iter()
        .filter(|op| !assigned.contains(&op.char_id))
        .filter_map(|op| {
            let boost = cc_global_morale_recovery(op, registry, building_data);
            (boost > 0.0).then_some((op, boost))
        })
        .collect();
    morale_ops.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (op, _) in morale_ops {
        if cc_room.operators.len() as i32 >= control_slots {
            break;
        }
        assigned.insert(op.char_id.clone());
        cc_room.operators.push(op.char_id.clone());
    }
}

/// Total base-wide morale recovery the Control Center's operators add on top of the
/// dormitories' rate.
fn cc_morale_boost(
    main: &BaseAssignment,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    main.rooms
        .iter()
        .filter(|r| r.room_type == "CONTROL")
        .flat_map(|r| r.operators.iter())
        .filter_map(|id| op_index.get(id.as_str()))
        .map(|op| cc_global_morale_recovery(op, registry, building_data))
        .sum()
}

/// An operator's total morale drain per hour (base + its skills' modifiers).
/// Lower = works longer before resting = better for sustained rotation.
fn op_morale_drain(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> f64 {
    let modifier: f64 = op
        .available_buffs
        .iter()
        .filter_map(|b| morale_drains.get(b))
        .sum();
    (BASE_MORALE_DRAIN + modifier).max(0.05)
}

/// Fraction of time an operator is working (vs resting) at steady state, given the
/// base's `recovery` rate (slower recovery -> more rest -> lower uptime).
pub(crate) fn op_uptime(
    op: &OperatorBaseProfile,
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
) -> f64 {
    let d = op_morale_drain(op, morale_drains);
    recovery / (d + recovery)
}

/// Operators a staggered rotation needs on its shared bench. Because only one
/// operator is swapped out at a time and the same filler can cover any room, the
/// bench is sized to the expected number resting at once (the summed per-operator
/// rest fractions across all mains), never fewer than one. Low-drain mains rest
/// less, so a strong roster needs only a handful of fillers - not a second team.
pub(crate) fn shared_bench_size(rest_demand: f64) -> usize {
    (rest_demand.ceil() as usize).max(1)
}

/// Hours a neutral-drain operator works before its morale runs low and you swap it
/// out (~a daily rotation); faster-draining operators last proportionally fewer.
const ROTATION_BASE_HOURS: f64 = 24.0;

/// Approximate hours an operator works before needing a rotation, rounded to the
/// hour - inversely proportional to its morale drain.
fn op_lasts_hours(op: &OperatorBaseProfile, morale_drains: &HashMap<String, f64>) -> f64 {
    (ROTATION_BASE_HOURS * BASE_MORALE_DRAIN / op_morale_drain(op, morale_drains)).round()
}

/// How close a room stays to its peak under a staggered rotation: 1.0 if its
/// operators never rest, lower as they rest more. A flexible room swaps a backup in
/// to cover a resting main (recovering `BACKUP_COVERAGE` of the gap); a LOCKED team
/// can't be staggered, so its rest gaps are uncovered (`coverage = 0`) and it falls
/// further from peak. Low-drain teams stay nearer peak either way.
fn room_sustain_factor(
    room_ops: &[String],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
    coverage: f64,
) -> f64 {
    let uptimes: Vec<f64> = room_ops
        .iter()
        .filter_map(|id| op_index.get(id.as_str()))
        .map(|op| op_uptime(op, morale_drains, recovery))
        .collect();
    if uptimes.is_empty() {
        return 1.0;
    }
    let avg_uptime = uptimes.iter().sum::<f64>() / uptimes.len() as f64;
    1.0 - (1.0 - avg_uptime) * (1.0 - coverage)
}

/// Sustained 24/7 efficiency of a staffing under a staggered rotation: each
/// production room's efficiency scaled by how well its team holds up to rotation.
pub fn sustained_efficiency_of(
    main: &BaseAssignment,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    recovery: f64,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    let op_index = build_op_index(operators);
    main.rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .map(|r| {
            // A locked synergy team can't be staggered, so a resting member's gap is
            // uncovered; a flexible room recovers `BACKUP_COVERAGE` of it.
            let coverage = if r.locked { 0.0 } else { BACKUP_COVERAGE };
            let rotation =
                room_sustain_factor(&r.operators, &op_index, morale_drains, recovery, coverage);
            // AFK product-buffer stall: a gold factory that overflows its buffer loses the excess,
            // so its sustained output is throttled unless its team carries enough capacity.
            let cap = team_capacity_bonus(
                &r.operators,
                &r.room_type,
                r.formula_type.as_deref(),
                &op_index,
                registry,
                building_data,
            );
            let cap_factor =
                factory_cap_factor(r.formula_type.as_deref(), r.level, r.total_efficiency, cap);
            r.total_efficiency * rotation * cap_factor
        })
        .sum()
}

pub fn compute_sustained_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> RotationAssignment {
    // The main staffing for the SUSTAINED view uses the cap-aware objective, so factories that
    // keep producing through long AFK (high product-buffer capacity, e.g. Vermeil teams) are
    // preferred over denser teams that would overflow and stall.
    let main = optimal_inner(operators, building, building_data, registry, morale_drains, &[], true);
    let facility_counts = effective_facility_counts(building, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);
    // Dormitories set the base recovery rate; a morale operator in the Control Center
    // (Chongyue, Wiš'adel) lifts it further for the whole base.
    let recovery =
        morale_recovery(building) + cc_morale_boost(&main, &op_index, registry, building_data);

    // Sustained 24/7 output: each production room's peak efficiency scaled by how
    // well its team holds up under rotation (low morale drain / low-level dorms ->
    // further below peak).
    let mut sustained_efficiency =
        sustained_efficiency_of(&main, operators, morale_drains, recovery, registry, building_data);

    // Dorm-capacity throttle: a base can only rest as many operators at once as its
    // dormitories hold. If more than that need to rest, the rotation falls behind and
    // workers run tired - scale output by how much of the rest demand the dorms cover.
    let rest_demand: f64 = main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .flat_map(|r| r.operators.iter())
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .map(|op| 1.0 - op_uptime(op, morale_drains, recovery))
        .sum();
    sustained_efficiency *=
        dorm_capacity_factor(rest_demand, total_dorm_capacity(building, building_data));

    let mains: HashSet<&str> = main
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter().map(String::as_str))
        .collect();

    // A staggered rotation swaps one operator at a time and reuses the same fillers
    // across rooms, so a single SHARED bench covers the whole base: one versatile
    // filler can back up every room of its kind, never needed in two at once.

    // Best filler for a room, ranked by the REAL value it adds ALONGSIDE the room's
    // mains - not an optimistic standalone bound. An operator whose buff is gated on
    // an absent teammate (Texas needs Lappland) adds ~0 and is rejected, since as a
    // staggered rotation piece it is swapped in to cover one resting main and its
    // enabler may not be present. Backups are not consumed per room (shared bench).
    let best_filler = |room_type: &str,
                       formula: Option<&str>,
                       room_mains: &[String]|
     -> Option<&OperatorBaseProfile> {
        let score = |ops: &[String]| -> f64 {
            let (s, v) = compute_team_efficiency(
                ops,
                room_type,
                formula,
                &op_index,
                registry,
                building_data,
                &facility_counts,
                total_dorm_levels,
                morale_drains,
                &[],
            );
            room_search_score(room_type, s, v)
        };
        let base = score(room_mains);
        operators
            .iter()
            .filter(|op| !mains.contains(op.char_id.as_str()))
            .filter(|op| {
                applicable_strategies(op, room_type, formula, registry, building_data)
                    .next()
                    .is_some()
            })
            .filter_map(|op| {
                let mut with = room_mains.to_vec();
                with.push(op.char_id.clone());
                let marginal = score(&with) - base;
                (marginal > 1e-6).then_some((op, marginal))
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(op, _)| op)
    };

    // Per-room rotation plan: who to swap first (fastest-draining) and the best
    // filler to rotate in. The same filler may back up several rooms - it is only
    // ever needed in one at a time - so backups are NOT consumed per room.
    let mut rooms = Vec::new();
    let mut bench: Vec<String> = Vec::new();
    for room in main
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .filter(|r| !r.operators.is_empty())
    {
        let formula = room.formula_type.as_deref();

        // Members ordered by who needs swapping first (shortest hours first).
        let mut members: Vec<RotationMember> = room
            .operators
            .iter()
            .filter_map(|id| op_index.get(id.as_str()).copied())
            .map(|op| RotationMember {
                operator: op.char_id.clone(),
                lasts_hours: op_lasts_hours(op, morale_drains),
            })
            .collect();
        members.sort_by(|a, b| {
            a.lasts_hours
                .partial_cmp(&b.lasts_hours)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // A LOCKED synergy team (its operators depend on each other) can't be
        // staggered one operator at a time without breaking the combo - swapping a
        // single member collapses the whole team's output. So a locked room gets NO
        // individual backup: it works as a fixed unit. Only flexible, individually
        // interchangeable rooms get a rotation backup.
        let backup = if room.locked {
            None
        } else {
            best_filler(&room.room_type, formula, &room.operators).map(|op| op.char_id.clone())
        };
        if let Some(id) = &backup
            && !bench.contains(id)
        {
            bench.push(id.clone());
        }

        rooms.push(RoomRotation {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            members,
            backup,
        });
    }

    // Express the rotation as a few OVERLAPPING sets: in set k each room rests its
    // k-th most-urgent main (covered by the backup), so consecutive sets share all
    // but one operator per room - you never break the whole base at once. Up to three
    // sets matches the usual 12h-swap / ~36h-recovery cadence.
    //
    // A room only rotates the operators that actually run their morale low within one
    // rotation cycle (~36h). A low-drain operator (Vermeil-type) works it almost
    // continuously and never needs a rest slot, so a room of them adds no sets - the
    // set count tracks REAL rest demand, not just seat count.
    const MAX_ROTATION_SETS: usize = 3;
    const ROTATION_CYCLE_HOURS: f64 = 36.0;
    let needs_rest = |m: &RotationMember| m.lasts_hours <= ROTATION_CYCLE_HOURS;
    // Only rooms with a backup rotate; a locked team (no backup) keeps its full crew
    // in every set, so it never drives the set count.
    let num_sets = rooms
        .iter()
        .map(|r| {
            if r.backup.is_some() {
                r.members.iter().filter(|m| needs_rest(m)).count()
            } else {
                0
            }
        })
        .max()
        .unwrap_or(0)
        .min(MAX_ROTATION_SETS);
    let sets: Vec<RotationSet> = (0..num_sets)
        .map(|k| RotationSet {
            rooms: rooms
                .iter()
                .map(|r| {
                    let mut working: Vec<String> =
                        r.members.iter().map(|m| m.operator.clone()).collect();
                    // Rest the k-th most-urgent main only if it genuinely needs it;
                    // otherwise this room keeps its full crew working this set.
                    let resting = match (r.members.get(k), &r.backup) {
                        (Some(m), Some(b)) if needs_rest(m) => {
                            working[k] = b.clone();
                            Some(m.operator.clone())
                        }
                        _ => None,
                    };
                    RotationSetRoom {
                        slot_id: r.slot_id.clone(),
                        room_type: r.room_type.clone(),
                        working,
                        resting,
                    }
                })
                .collect(),
        })
        .collect();

    RotationAssignment {
        main,
        rooms,
        shared_bench: bench,
        sets,
        sustained_efficiency,
    }
}

/// Which operators occupy a room for a given live view: the static stationed
/// crew (`shift = None`) or one of the player's planned preset rotation shifts.
fn room_ops_for_shift(room: &UserRoom, shift: Option<usize>) -> Vec<String> {
    match shift {
        Some(i) => room
            .preset_shifts
            .get(i)
            .cloned()
            .unwrap_or_else(|| room.current_operators.clone()),
        None => room.current_operators.clone(),
    }
}

/// Does the building define preset rotation shifts the player has planned?
pub fn has_preset_shifts(building: &UserBuilding) -> bool {
    building.rooms.iter().any(|r| r.preset_shifts.len() > 1)
}

/// Evaluate the player's CURRENT base as stationed right now (`shift = None`) or
/// for one of their planned preset rotation shifts (`shift = Some(i)`), so it can
/// be compared against the optimizer's output.
pub fn compute_current_assignment(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    shift: Option<usize>,
) -> BaseAssignment {
    let facility_counts = effective_facility_counts(building, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let op_index = build_op_index(operators);

    // Global bonuses from whoever the player has in the Control Center for this
    // shift (same non-stacking rule as the optimizer).
    let control_room = building.rooms.iter().find(|r| r.room_type == "CONTROL");
    let cc_ops = control_room
        .map(|cc| room_ops_for_shift(cc, shift))
        .unwrap_or_default();
    let mut acc = CcBonusAccumulator::default();
    for op in cc_ops
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
    {
        acc.add(&cc_bonuses(op, registry, building_data));
    }
    let (global_bonuses, cc_conditions) = acc.finish();

    let mut rooms: Vec<RoomAssignment> = Vec::new();

    // Show the current Control Center (with the bonuses it currently provides).
    if let Some(cc) = control_room
        && !cc_ops.is_empty()
    {
        rooms.push(RoomAssignment {
            slot_id: cc.slot_id.clone(),
            room_type: "CONTROL".to_string(),
            level: cc.level,
            formula_type: None,
            operators: cc_ops,
            total_efficiency: global_bonuses.values().sum(),
            ..Default::default()
        });
    }

    let mut total = 0.0;
    for room in building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        // Keep only operators that actually have base profiles (drop tokens etc.).
        let ops: Vec<String> = room_ops_for_shift(room, shift)
            .into_iter()
            .filter(|id| op_index.contains_key(id.as_str()))
            .collect();
        let formula = if room.room_type == "MANUFACTURE" {
            room.current_formula
                .clone()
                .or_else(|| Some("F_GOLD".to_string()))
        } else {
            None
        };
        let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);
        let (speed, value) = compute_team_efficiency(
            &ops,
            &room.room_type,
            formula.as_deref(),
            &op_index,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            morale_drains,
            &cc_conditions,
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
        rooms.push(RoomAssignment {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            level: room.level,
            formula_type: formula,
            operators: ops,
            total_efficiency: eff,
            order_value: value,
            locked,
        });
    }

    BaseAssignment {
        rooms,
        total_production_efficiency: total,
    }
}

fn count_facilities(building: &UserBuilding) -> HashMap<String, usize> {
    let mut facility_counts: HashMap<String, usize> = HashMap::new();
    for room in &building.rooms {
        *facility_counts.entry(room.room_type.clone()).or_insert(0) += 1;
    }
    facility_counts
}

/// The building's facility counts WITH any enabler bonuses the roster can field (Greyy the
/// Lightningbearer E2 "+1 Power Plant", Eunectes E2 "+2"): each distinct enabler buff family
/// contributes its amount once (the in-game gate is self-exclusive, so duplicate copies don't
/// stack), assuming it is stationed where it fires - optimal whenever the base runs the
/// per-facility automation it powers. Automation scalers (Weedy/Eunectes/Pudding) then score
/// against the boosted count, so those combos are correctly valued; bases without such operators
/// are unaffected (nothing reads the extra count).
fn effective_facility_counts(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> HashMap<String, usize> {
    let mut counts = count_facilities(building);
    let mut seen_families: HashSet<&str> = HashSet::new();
    for op in operators {
        for buff_id in &op.available_buffs {
            if let Some(BuffResolutionStrategy::FacilityCountModifier { target_room, amount }) =
                registry.get(buff_id)
            {
                let family = buff_id.split('[').next().unwrap_or(buff_id);
                if *amount > 0 && seen_families.insert(family) {
                    *counts.entry(target_room.clone()).or_insert(0) += *amount as usize;
                }
            }
        }
    }
    counts
}

/// Max stationed for the user's highest-level room of `room_type` (e.g. their CC).
fn max_stationed_for_room(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    room_type: &str,
) -> i32 {
    let level = building
        .rooms
        .iter()
        .filter(|r| r.room_type == room_type)
        .map(|r| r.level)
        .max()
        .unwrap_or(1);
    max_stationed_at_level(building_data, room_type, level)
}

/// A Control Center assignment plus the global production bonuses it grants.
struct ControlCenter {
    operators: Vec<String>,
    /// The user's CONTROL room (slot/level), if they have one built.
    slot_id: Option<String>,
    level: i32,
    /// Total global bonus % across all targeted production room types - shown
    /// as the CC room's "efficiency" so its impact is visible.
    total_global_pct: f64,
}

impl ControlCenter {
    fn into_room(self) -> Option<RoomAssignment> {
        let slot_id = self.slot_id?;
        if self.operators.is_empty() {
            return None;
        }
        Some(RoomAssignment {
            slot_id,
            room_type: "CONTROL".to_string(),
            level: self.level,
            formula_type: None,
            operators: self.operators,
            total_efficiency: self.total_global_pct,
            ..Default::default()
        })
    }
}

/// Append unused operators to `slots` until it reaches `max_slots`, marking each
/// as `assigned`. Fillers occupy seats but add no value, so we pick the operators
/// with the lowest *opportunity cost* - those with the fewest base skills for
/// OTHER rooms - keeping operators with useful Dormitory/Training/etc. skills
/// (e.g. Schwarz) available for the rooms where they actually help.
fn fill_remaining_slots(
    slots: &mut Vec<String>,
    max_slots: i32,
    room_type: &str,
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
    assigned: &mut HashSet<String>,
) {
    while (slots.len() as i32) < max_slots {
        let pick = operators
            .iter()
            .filter(|op| !assigned.contains(&op.char_id))
            .min_by_key(|op| other_room_skill_count(op, room_type, building_data));
        let Some(op) = pick else { break };
        assigned.insert(op.char_id.clone());
        slots.push(op.char_id.clone());
    }
}

/// How many *other* room types this operator has base skills for (a proxy for
/// how much value is lost by using them as a filler here). 0 = pure benchwarmer.
fn other_room_skill_count(
    op: &OperatorBaseProfile,
    room_type: &str,
    building_data: &BuildingDataFile,
) -> usize {
    let mut rooms: Vec<&str> = op
        .available_buffs
        .iter()
        .filter_map(|b| building_data.buffs.get(b))
        .map(|buff| buff.room_type.as_str())
        .filter(|rt| *rt != room_type)
        .collect();
    rooms.sort_unstable();
    rooms.dedup();
    rooms.len()
}

/// A single Control Center global bonus: which production room it boosts, the
/// "skill effect family", and the bonus %. The game's non-stacking rule is driven
/// by an explicit clause in the buff text ("only the most effective one will take
/// effect when assigned Operators have the same skill effect" / "Only the
/// strongest effect of this type takes place"): buffs carrying it dedup against
/// same-room clause-bearing buffs (only the strongest applies), while clause-less
/// buffs (Sakiko's Precious-Metal productivity, Viviana's faction buff, etc.)
/// always `stacks` on top.
struct CcBonus {
    room: String,
    family: String,
    bonus: f64,
    /// True when the buff has NO non-stacking clause and therefore always adds.
    stacks: bool,
    /// `Some` when this bonus only applies to production rooms whose team meets a
    /// faction condition - credited per-room, not flat. `bonus` above is then a
    /// discounted *selection* weight, not the value actually granted.
    conditional: Option<CcCondition>,
}

/// A Control Center bonus that is gated on a production room's team composition.
/// Resolved per-room in `compute_team_efficiency` rather than added flat.
#[derive(Clone)]
struct CcCondition {
    target_room: String,
    faction_token: String,
    required_count: usize,
    per_operator: bool,
    bonus_pct: f64,
}

impl CcCondition {
    /// This condition's contribution to a room of `room_type` staffed by `team`.
    fn contribution(&self, room_type: &str, team: &[&OperatorBaseProfile]) -> f64 {
        if self.target_room != room_type {
            return 0.0;
        }
        let count = team
            .iter()
            .filter(|op| op.match_tags.contains(&self.faction_token))
            .count();
        if self.per_operator {
            self.bonus_pct * count as f64
        } else if count >= self.required_count {
            self.bonus_pct
        } else {
            0.0
        }
    }
}

/// Folds Control Center bonuses together under the non-stacking rule (only the
/// strongest per skill-effect family applies), separating unconditional bonuses
/// (summed flat per room) from faction-gated ones (returned for per-room,
/// team-dependent evaluation). Shared by the optimizer and the live-base reader.
#[derive(Default)]
struct CcBonusAccumulator {
    /// Clause-bearing buffs: strongest-per-(room, family) wins (non-stacking).
    best: HashMap<(String, String), f64>,
    /// Clause-less buffs: summed flat per room (they always stack).
    stacked: HashMap<String, f64>,
    conditions: HashMap<(String, String), CcCondition>,
    cond_families: HashSet<(String, String)>,
}

impl CcBonusAccumulator {
    /// The marginal flat gain from adding `bonuses` - a stacking buff always counts
    /// fully; a non-stacking one counts only what it exceeds its family's strongest.
    fn marginal(&self, bonuses: &[CcBonus]) -> f64 {
        bonuses
            .iter()
            .map(|b| {
                if b.stacks && b.conditional.is_none() {
                    b.bonus
                } else {
                    let cur = self
                        .best
                        .get(&(b.room.clone(), b.family.clone()))
                        .copied()
                        .unwrap_or(0.0);
                    (b.bonus - cur).max(0.0)
                }
            })
            .sum()
    }

    /// Fold one operator's bonuses in: clause-less buffs add to the room total,
    /// clause-bearing ones keep only the strongest per family.
    fn add(&mut self, bonuses: &[CcBonus]) {
        for b in bonuses {
            if b.stacks && b.conditional.is_none() {
                *self.stacked.entry(b.room.clone()).or_insert(0.0) += b.bonus;
                continue;
            }
            let key = (b.room.clone(), b.family.clone());
            let slot = self.best.entry(key.clone()).or_insert(0.0);
            *slot = slot.max(b.bonus);
            if let Some(cond) = &b.conditional {
                self.cond_families.insert(key.clone());
                self.conditions
                    .entry(key)
                    .and_modify(|c| {
                        if cond.bonus_pct > c.bonus_pct {
                            *c = cond.clone();
                        }
                    })
                    .or_insert_with(|| cond.clone());
            }
        }
    }

    /// `(flat per-room totals, faction-gated conditions)`. Conditional families are
    /// kept out of the flat totals - they're applied per room against the team.
    fn finish(self) -> (HashMap<String, f64>, Vec<CcCondition>) {
        let mut global = self.stacked;
        for ((room, family), bonus) in &self.best {
            if !self.cond_families.contains(&(room.clone(), family.clone())) {
                *global.entry(room.clone()).or_insert(0.0) += bonus;
            }
        }
        (global, self.conditions.into_values().collect())
    }
}

/// Collect an operator's Control Center global production bonuses.
fn cc_bonuses(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> Vec<CcBonus> {
    let mut out = Vec::new();
    for buff_id in &op.available_buffs {
        let Some(buff) = building_data.buffs.get(buff_id) else {
            continue;
        };
        if buff.room_type != "CONTROL" {
            continue;
        }
        match registry.get(buff_id) {
            Some(BuffResolutionStrategy::GlobalEffect {
                target_room,
                bonus_pct,
            }) => {
                // Clause-bearing buffs of the same room+metric don't stack (e.g.
                // Amiya & Noir Corne Alter both "+7% all Trading Posts"); clause-less
                // ones (Sakiko's Precious-Metal productivity) always stack.
                let stacks = cc_buff_stacks(buff);
                out.push(CcBonus {
                    room: target_room.clone(),
                    family: format!("{target_room}#global"),
                    bonus: *bonus_pct,
                    stacks,
                    conditional: None,
                });
            }
            Some(BuffResolutionStrategy::TagBased {
                target_room,
                bonus_pct,
                ..
            }) => out.push(CcBonus {
                room: target_room.clone(),
                family: buff_id.split('[').next().unwrap_or(buff_id).to_string(),
                // Faction bonuses only reach matching operators - credit half. They
                // carry no non-stacking clause, so they stack on top of generics.
                bonus: bonus_pct * 0.5,
                stacks: true,
                conditional: None,
            }),
            Some(BuffResolutionStrategy::ConditionalGlobalEffect {
                target_room,
                faction_token,
                required_count,
                per_operator,
                bonus_pct,
            }) => out.push(CcBonus {
                room: target_room.clone(),
                family: buff_id.split('[').next().unwrap_or(buff_id).to_string(),
                // Selection weight: discounted since the gate may not be met. The
                // real value is granted per-room via `conditional`.
                bonus: bonus_pct * 0.5,
                stacks: false,
                conditional: Some(CcCondition {
                    target_room: target_room.clone(),
                    faction_token: faction_token.clone(),
                    required_count: *required_count,
                    per_operator: *per_operator,
                    bonus_pct: *bonus_pct,
                }),
            }),
            _ => {}
        }
    }
    out
}

/// Can the roster ever satisfy a conditional CC buff's gate? `SilverAsh`'s "+10% to
/// Trading Posts with 3 Kjerag Operators" is worthless unless the roster actually
/// holds `required_count` operators of that faction who can staff the target room.
/// Without this check the optimizer credits a selection weight (and a CC seat) to a
/// gate that can never trigger.
fn cc_condition_feasible(
    cond: &CcCondition,
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> bool {
    let eligible = operators
        .iter()
        .filter(|op| op.match_tags.contains(&cond.faction_token))
        .filter(|op| {
            op.available_buffs.iter().any(|b| {
                building_data
                    .buffs
                    .get(b)
                    .is_some_and(|buff| buff.room_type == cond.target_room)
            })
        })
        .count();
    eligible >= cond.required_count
}

/// Does a faction-gated CC condition actually fire given the FINAL production
/// teams? (`SilverAsh`'s "3 Kjerag in a Trading Post" only fires if a post really
/// ends up holding 3 Kjerag operators.)
fn cc_condition_fires(
    cond: &CcCondition,
    rooms: &[RoomAssignment],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
) -> bool {
    rooms
        .iter()
        .filter(|r| r.room_type == cond.target_room)
        .any(|r| {
            let count = r
                .operators
                .iter()
                .filter_map(|id| op_index.get(id.as_str()))
                .filter(|op| op.match_tags.contains(&cond.faction_token))
                .count();
            if cond.per_operator {
                count >= 1
            } else {
                count >= cond.required_count
            }
        })
}

/// Is a CC operator dead weight - i.e. every effect it brings is a conditional that
/// does NOT fire in the actual production teams? The production optimizer is already
/// rewarded for satisfying these gates (it credits the bonus per room), so if it
/// declined to, the gate is genuinely unmet and the operator should yield its seat.
fn cc_op_is_dead(
    op: &OperatorBaseProfile,
    rooms: &[RoomAssignment],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    let bonuses = cc_bonuses(op, registry, building_data);
    if bonuses.is_empty() {
        return false; // no CC value to begin with - a plain filler, leave it be
    }
    !bonuses.iter().any(|b| match &b.conditional {
        None => true, // an unconditional bonus always lands
        Some(c) => cc_condition_fires(c, rooms, op_index),
    })
}

/// Does this Control Center buff stack with same-type buffs? It does NOT (only the
/// strongest applies) when it carries the game's non-stacking clause.
fn cc_buff_stacks(buff: &Buff) -> bool {
    let d = buff.description.to_lowercase();
    !(d.contains("only the most effective")
        || d.contains("strongest effect of this type")
        || d.contains("only the strongest"))
}

/// Assign operators to the Control Center to maximize global production bonuses.
/// Returns the CC assignment and the global bonuses map (`room_type` -> total bonus %).
///
/// Selection is marginal-greedy and respects the non-stacking rule: filling a
/// slot with a second operator whose only effect duplicates one already present
/// adds nothing, so a complementary operator (different effect family / target
/// room) is preferred instead.
fn assign_control_center(
    operators: &[OperatorBaseProfile],
    control_room: Option<&UserRoom>,
    max_slots: i32,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    exclude: &HashSet<String>,
) -> (ControlCenter, HashMap<String, f64>, Vec<CcCondition>) {
    let candidates: Vec<(&OperatorBaseProfile, Vec<CcBonus>)> = operators
        .iter()
        .filter(|op| !exclude.contains(&op.char_id))
        .map(|op| {
            // Drop conditional bonuses whose gate the roster can never satisfy
            // (e.g. SilverAsh's "3 Kjerag in a Trading Post" with too few Kjerag
            // traders) so they earn neither a selection weight nor a CC seat.
            let bonuses = cc_bonuses(op, registry, building_data)
                .into_iter()
                .filter(|b| {
                    b.conditional
                        .as_ref()
                        .is_none_or(|c| cc_condition_feasible(c, operators, building_data))
                })
                .collect::<Vec<_>>();
            (op, bonuses)
        })
        .filter(|(_, b)| !b.is_empty())
        .collect();

    // Marginal-greedy selection: each slot takes the operator adding the most flat
    // gain over what's already chosen (the non-stacking rule lives in the accumulator).
    let mut acc = CcBonusAccumulator::default();
    let mut chosen: HashSet<String> = HashSet::new();
    let mut cc_assigned: Vec<String> = Vec::new();

    for _ in 0..max_slots.max(0) {
        let mut best_op: Option<&OperatorBaseProfile> = None;
        let mut best_bonuses: &[CcBonus] = &[];
        let mut best_gain = 0.0;
        for (op, bonuses) in &candidates {
            if chosen.contains(&op.char_id) {
                continue;
            }
            let gain = acc.marginal(bonuses);
            if gain > best_gain {
                best_gain = gain;
                best_op = Some(op);
                best_bonuses = bonuses;
            }
        }
        let Some(op) = best_op else { break };
        chosen.insert(op.char_id.clone());
        cc_assigned.push(op.char_id.clone());
        acc.add(best_bonuses);
    }

    let (global_bonuses, conditions) = acc.finish();
    let cc = ControlCenter {
        operators: cc_assigned,
        slot_id: control_room.map(|r| r.slot_id.clone()),
        level: control_room.map_or(1, |r| r.level),
        total_global_pct: global_bonuses.values().sum(),
    };
    (cc, global_bonuses, conditions)
}

#[allow(clippy::too_many_arguments)]
fn assign_production_rooms(
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
    let factory_rooms: Vec<&&UserRoom> = rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .collect();
    let trading_rooms: Vec<&&UserRoom> =
        rooms.iter().filter(|r| r.room_type == "TRADING").collect();

    let num_factories = factory_rooms.len();

    // Try all gold/EXP splits and pick the best
    let mut best_assignments: Vec<RoomAssignment> = Vec::new();
    let mut best_objective: f64 = f64::NEG_INFINITY;
    let mut best_assigned_snapshot: HashSet<String> = assigned.clone();

    // Gold factories must be >= trading post count (TPs need gold bars to trade).
    let min_gold = trading_rooms.len().min(num_factories);
    // The yield-coupled optimum is "just enough gold to feed the trading posts"
    // (excess gold is unsold and worth less than EXP), so only the splits at and
    // just above min_gold are ever competitive - no need to scan all the way up.
    let max_gold = (min_gold + 1).min(num_factories);

    for num_gold in min_gold..=max_gold {
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
                cc_conditions,
                morale_drains,
                cap_aware,
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
                cc_conditions,
                morale_drains,
                cap_aware,
            );
            trial_rooms.push(room_assignment);
        }

        // Balance operators across same-type/formula rooms (diminishing-returns
        // aware), then score this split by the soft-capped objective so stacking
        // everything into one room is penalized.
        rebalance_rooms(
            &mut trial_rooms,
            operators,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            global_bonuses,
            cc_conditions,
            morale_drains,
        );

        // Score the split by realized daily YIELD (LMD-equivalent), not by a sum
        // of efficiency %. This makes the gold/EXP split value-based: gold
        // factories only pay off up to the trading posts' selling capacity, so
        // excess gold factories are correctly switched to EXP.
        let trial_objective = assignment_value(&trial_rooms);
        if trial_objective > best_objective {
            best_objective = trial_objective;
            best_assignments = trial_rooms;
            best_assigned_snapshot = trial_assigned;
        }
    }

    // Apply the winning assignment's used operators to the real assigned set
    *assigned = best_assigned_snapshot;

    // Cross-formula cleanup: free a generic operator from a formula-specific room
    // for one that lacks a dedicated operator, seating an idle specialist in its
    // place. Runs before padding (EXP rooms still have open seats) and applies only
    // value-raising moves, so it never regresses the assignment.
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

    // Now that every room has its best *beneficial* team (and only now - so a
    // valuable operator is never wasted as a filler in a room where they do
    // nothing), top each room up to a full crew with leftover operators.
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

    best_assignments
}

/// Move a generic operator out of a formula-specific room into a different-formula
/// room that has an open seat, when doing so raises total value. A generic (`+30%`
/// to any product) in a Gold factory is worth the same in an EXP factory, so when
/// the Gold side already has enough output (excess gold is unsold) the generic is
/// better spent producing EXP; the seat it frees is later backfilled (by a Metalwork
/// specialist if one is idle). Gated on the full `assignment_value` - which couples
/// gold to trading - so it never regresses the assignment.
#[allow(clippy::too_many_arguments)]
fn reallocate_across_formulas(
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
    // Recompute a room as if staffed by `ops` (efficiency incl. global, order value).
    let scored = |tpl: &RoomAssignment, ops: Vec<String>| -> RoomAssignment {
        let (speed, value) = compute_team_efficiency(
            &ops,
            &tpl.room_type,
            tpl.formula_type.as_deref(),
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
            total_efficiency: speed + global,
            order_value: value,
            ..tpl.clone()
        }
    };

    for _ in 0..3 {
        let base_val = assignment_value(rooms);
        // Best strictly-improving move this pass: (room A, generic index, room B).
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
                // Only generics move - a specialist earns its keep right here.
                if op_is_formula_specialist(g_op, &rooms[ai].room_type, fa_ref, building_data) {
                    continue;
                }
                let mut a_ops = rooms[ai].operators.clone();
                a_ops.remove(gi);
                let a_new = scored(&rooms[ai], a_ops);
                for bi in 0..rooms.len() {
                    if bi == ai
                        || !is_production_room(&rooms[bi].room_type)
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
                        continue; // only rooms with an open seat (no displacement)
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

/// Fill every production room up to its slot count with leftover operators.
/// Runs only after all beneficial assignment is done, so the fillers are
/// operators no room benefits from. A normal room's efficiency is recomputed
/// (a filler with a small relevant buff still helps); an automation room keeps
/// its efficiency since extra operators are nullified.
#[allow(clippy::too_many_arguments)]
fn pad_production_rooms(
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
            // Extra operators are nullified - occupy the seats with the lowest
            // opportunity-cost benchwarmers available.
            fill_remaining_slots(
                &mut room.operators,
                max_slots,
                &room.room_type,
                operators,
                building_data,
                assigned,
            );
        } else {
            // Add the leftover that maximizes the room's efficiency (usually +0);
            // among equally-good picks, prefer the lowest opportunity-cost one so
            // operators useful elsewhere aren't burned as fillers.
            while (room.operators.len() as i32) < max_slots {
                // The seat may stay empty: a filler is only worth adding if it does
                // not LOWER the room's output. Degenbrecher (+25% speed but -6 order
                // limit) tanks a trading post, so she must never be padded in.
                let current_score = room_search_score(
                    &room.room_type,
                    room.total_efficiency - global,
                    room.order_value,
                );
                let mut best_id: Option<String> = None;
                let mut best_score = f64::NEG_INFINITY;
                let mut best_speed = 0.0;
                let mut best_value = 0.0;
                let mut best_cost = usize::MAX;
                for op in operators {
                    if assigned.contains(&op.char_id) {
                        continue;
                    }
                    // A nullifier (Weedy-type automation, Shamare) zeroes its
                    // teammates' output, so padding one into a working team would
                    // wreck it. These only belong in their own dedicated team.
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
                    let (speed, value) = compute_team_efficiency(
                        &trial,
                        &room.room_type,
                        room.formula_type.as_deref(),
                        &op_index,
                        registry,
                        building_data,
                        facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        cc_conditions,
                    );
                    let score = room_search_score(&room.room_type, speed, value);
                    let cost = other_room_skill_count(op, &room.room_type, building_data);
                    let better = best_id.is_none()
                        || score > best_score + 1e-9
                        || ((score - best_score).abs() <= 1e-9 && cost < best_cost);
                    if better {
                        best_score = score;
                        best_speed = speed;
                        best_value = value;
                        best_cost = cost;
                        best_id = Some(op.char_id.clone());
                    }
                }
                match best_id {
                    // Only add when it doesn't reduce output; otherwise leave the
                    // seat empty (a net-negative filler is worse than no filler).
                    Some(id) if best_score >= current_score - 1e-9 => {
                        assigned.insert(id.clone());
                        room.operators.push(id);
                        room.total_efficiency = best_speed + global;
                        room.order_value = best_value;
                    }
                    _ => break, // roster exhausted, or no non-harmful filler remains
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
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
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) -> RoomAssignment {
    let max_slots = max_stationed_at_level(building_data, &room.room_type, room.level);
    let global = *global_bonuses.get(&room.room_type).unwrap_or(&0.0);

    // Mode 1: best normal team (exhaustive search over top candidate combinations)
    let (normal_ops, normal_speed, normal_value) = best_team_for_room(
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

    // Mode 2: automation (factories only) - a single op's facility-count buff
    // nullifies teammates, so this team is scored independently. No order value.
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

    // Pick the mode by realized value (order value multiplies trading LMD).
    let normal_score = room_search_score(&room.room_type, normal_speed, normal_value);
    let auto_score = room_search_score(&room.room_type, auto_speed, 0.0);
    let (room_ops, speed, value) = if auto_score > normal_score && !auto_ops.is_empty() {
        (auto_ops, auto_speed, 0.0)
    } else {
        (normal_ops, normal_speed, normal_value)
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
        locked,
    }
}

/// A FIXED synergy squad: its operators depend on each other, so they can't be
/// swapped without breaking the combo. True when the team mixes a nullifier
/// (Shamare) with order-value partners, or any member's buff is gated on a
/// teammate / faction that the team actually satisfies. A team of independent
/// operators (each contributing standalone) is NOT locked - it's interchangeable.
fn team_is_locked(
    ops: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    if ops.len() < 2 {
        return false;
    }
    let present: HashSet<&str> = ops.iter().map(String::as_str).collect();

    // A nullifier in the team locks everyone to it (the only useful partners are
    // its order-value operators).
    let has_nullifier = ops.iter().any(|id| {
        building_data.chars.get(id).is_some_and(|c| {
            c.buff_char.iter().flat_map(|s| &s.buff_data).any(|e| {
                matches!(
                    registry.get(&e.buff_id),
                    Some(BuffResolutionStrategy::NullifyTeammatesSelfScaling { .. })
                )
            })
        })
    });
    if has_nullifier {
        return true;
    }

    // Any member whose buff is gated on a teammate present in this team.
    for id in ops {
        for strategy in profile_strategies(id, room_type, formula_type, registry, building_data) {
            if let BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                ..
            } = strategy
                && present.contains(req.as_str())
            {
                return true;
            }
        }
    }
    false
}

/// Resolved strategies for a `char_id`'s top buffs that apply in this room - the
/// `char_id` (E2-max) variant of `applicable_strategies`, for when we only have
/// the id rather than a profile with a specific elite level.
fn profile_strategies<'a>(
    char_id: &str,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> Vec<&'a BuffResolutionStrategy> {
    let Some(bc) = building_data.chars.get(char_id) else {
        return Vec::new();
    };
    bc.buff_char
        .iter()
        .filter_map(|s| s.buff_data.last())
        .filter_map(|e| {
            applicable_strategy(&e.buff_id, room_type, formula_type, registry, building_data)
        })
        .collect()
}

/// Does this buff apply to the given room type and (optional) formula?
fn buff_applies(buff: &Buff, room_type: &str, formula_type: Option<&str>) -> bool {
    if buff.room_type != room_type {
        return false;
    }
    if let Some(formula) = formula_type
        && !buff.targets.is_empty()
        && !buff.targets.iter().any(|t| t == formula)
    {
        return false;
    }
    true
}

/// A buff whose targets span more than one product family (both Gold and EXP)
/// serves any factory equally, so it is generic - not a formula specialist.
fn is_generic_targets(targets: &[String]) -> bool {
    targets.iter().any(|t| t == "F_GOLD") && targets.iter().any(|t| t == "F_EXP")
}

/// True if the operator has a buff that applies to this room/formula and is
/// formula-SPECIFIC (Metalwork-style: its targets name this product but not every
/// product). Generic productivity buffs (which serve every formula equally) are
/// not specialists, so when they tie a specialist they can be freed for whichever
/// formula has no dedicated operator (e.g. an EXP factory).
fn op_is_formula_specialist(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    building_data: &BuildingDataFile,
) -> bool {
    let Some(formula) = formula_type else {
        return false;
    };
    op.available_buffs.iter().any(|b| {
        building_data.buffs.get(b).is_some_and(|buff| {
            buff_applies(buff, room_type, formula_type)
                && buff.targets.iter().any(|t| t == formula)
                && !is_generic_targets(&buff.targets)
        })
    })
}

/// Resolve one `buff_id` to its strategy if the buff applies to this room/formula -
/// the shared "look up the buff, gate on room/formula, resolve" step.
fn applicable_strategy<'a>(
    buff_id: &str,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> Option<&'a BuffResolutionStrategy> {
    let buff = building_data.buffs.get(buff_id)?;
    buff_applies(buff, room_type, formula_type)
        .then(|| registry.get(buff_id))
        .flatten()
}

/// Resolved strategies for an operator's buffs that apply in the given room.
fn applicable_strategies<'a>(
    op: &'a OperatorBaseProfile,
    room_type: &'a str,
    formula_type: Option<&'a str>,
    registry: &'a HashMap<String, BuffResolutionStrategy>,
    building_data: &'a BuildingDataFile,
) -> impl Iterator<Item = &'a BuffResolutionStrategy> {
    op.available_buffs.iter().filter_map(move |b| {
        applicable_strategy(b, room_type, formula_type, registry, building_data)
    })
}

/// Sum of an operator's order-VALUE contributions (LMD per order) in this room.
/// An operator's order value that SURVIVES a Shamare-type speed-nullifier: flat-LMD
/// and Precious-Metal value count, but Pure-Gold value (Proviso) does not, because
/// Shamare shifts the post away from Pure-Gold orders. This is what makes an operator
/// a genuine Shamare partner versus a mere warm body.
fn op_surviving_order_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    applicable_strategies(op, room_type, formula_type, registry, building_data)
        .map(|s| match s {
            BuffResolutionStrategy::OrderValue {
                estimated_pct,
                pure_gold: false,
            } => *estimated_pct,
            _ => 0.0,
        })
        .sum()
}

/// Find the best NORMAL-mode team for a room by exhaustively evaluating
/// combinations of the top candidate operators.
///
/// Greedy slot-by-slot filling can't discover synergies that require *both*
/// members present (e.g. Texas's +65% only triggers with Lappland, and Lappland
/// alone scores ~0) - neither operator bootstraps the pair. Searching whole-team
/// combinations evaluates the pair together, so the true optimum is found.
#[allow(clippy::too_many_arguments)]
fn best_team_for_room(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    cap_aware: bool,
) -> (Vec<String>, f64, f64) {
    if max_slots <= 0 {
        return (Vec::new(), 0.0, 0.0);
    }
    let max_slots = max_slots as usize;
    let op_index = build_op_index(operators);

    // Candidate pool: operators with at least one applicable buff, excluding
    // automation operators (their nullify-others buff can't mix with a team).
    let candidates: Vec<&OperatorBaseProfile> = operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| {
            op.available_buffs.iter().any(|b| {
                building_data
                    .buffs
                    .get(b)
                    .is_some_and(|buff| buff_applies(buff, &room.room_type, formula_type))
            })
        })
        .filter(|op| !has_automation_buff(op, registry))
        .collect();

    if candidates.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }

    // Operators that *enable* another candidate's conditional buff get that
    // unlocked value folded into their ranking score, so synergy enablers (who
    // score ~0 standalone) survive the top-K cut.
    let mut enabler_boost: HashMap<String, f64> = HashMap::new();
    for op in &candidates {
        for b in &op.available_buffs {
            if let Some(BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                efficiency,
                ..
            }) = registry.get(b)
            {
                let slot = enabler_boost.entry(req.clone()).or_insert(0.0);
                *slot = slot.max(*efficiency);
            }
        }
    }

    // A nullifier (Shamare) zeroes teammates' SPEED and gains efficiency per
    // teammate, so its only worthwhile partners are operators whose order VALUE
    // survives the nullify (Tequila's flat LMD, Bibeak's Precious Metal). Every
    // other operator would merely be a warm body for the per-teammate scaling - and
    // that body slot is better filled by a true benchwarmer (during padding) than by
    // consuming a value operator (Proviso, who is wasted here) or a teammate-gated
    // operator (Lappland, reserved for when her partner is around). So when a
    // nullifier is present, only the nullifier itself and surviving-value operators
    // are eligible team members; the rest are left for padding to fill as bodies.
    let has_nullifier = candidates
        .iter()
        .any(|op| op_is_nullifier(op, &room.room_type, formula_type, registry, building_data));

    let candidates: Vec<&OperatorBaseProfile> = if has_nullifier {
        candidates
            .into_iter()
            .filter(|op| {
                op_is_nullifier(op, &room.room_type, formula_type, registry, building_data)
                    || op_surviving_order_value(
                        op,
                        &room.room_type,
                        formula_type,
                        registry,
                        building_data,
                    ) > 0.0
            })
            .collect()
    } else {
        candidates
    };

    let mut ranked: Vec<(&OperatorBaseProfile, f64, bool)> = candidates
        .iter()
        .map(|op| {
            // Fold surviving order value into the ranking so value partners (which
            // score ~0 on speed) survive the top-K cut alongside the nullifier.
            let value_boost = if has_nullifier {
                op_surviving_order_value(op, &room.room_type, formula_type, registry, building_data)
                    * 5.0
            } else {
                0.0
            };
            let bound = optimistic_bound(
                op,
                &room.room_type,
                formula_type,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                max_slots,
            ) + enabler_boost.get(&op.char_id).copied().unwrap_or(0.0)
                + value_boost;
            let specialist =
                op_is_formula_specialist(op, &room.room_type, formula_type, building_data);
            (*op, bound, specialist)
        })
        .collect();
    // Rank by upside, breaking ties toward formula specialists: a generic operator
    // that merely matches a specialist's value is freed for a formula that lacks
    // one (Metalwork ops take the Gold factory, a generic +30% goes to EXP).
    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.2.cmp(&a.2))
    });
    ranked.truncate(CANDIDATE_LIMIT);

    let pool: Vec<String> = ranked.iter().map(|(op, _, _)| op.char_id.clone()).collect();

    let mut best_team: Vec<String> = Vec::new();
    let mut best_score = 0.0;
    let mut best_speed = 0.0;
    let mut best_value = 0.0;

    for combo in combinations(&pool, max_slots) {
        let (speed, value) = compute_team_efficiency(
            &combo,
            &room.room_type,
            formula_type,
            &op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            cc_conditions,
        );
        let mut score = room_search_score(&room.room_type, speed, value);
        // Sustained objective: throttle a gold factory by its AFK buffer stall, so a high-capacity
        // team (Vermeil) that keeps producing across long AFK beats a denser team that overflows.
        if cap_aware {
            let cap = team_capacity_bonus(&combo, &room.room_type, formula_type, &op_index, registry, building_data);
            score *= factory_cap_factor(formula_type, room.level, speed, cap);
        }
        if score > best_score {
            best_score = score;
            best_speed = speed;
            best_value = value;
            best_team = combo;
        }
    }

    (best_team, best_speed, best_value)
}

/// Best automation team for a factory: greedily take the operators with the
/// highest facility-count-scaling value (these don't synergize with teammates).
#[allow(clippy::too_many_arguments)]
fn best_automation_team(
    room: &UserRoom,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    already_assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: i32,
) -> (Vec<String>, f64) {
    let mut scored: Vec<(&OperatorBaseProfile, f64)> = operators
        .iter()
        .filter(|op| !already_assigned.contains(&op.char_id))
        .filter(|op| has_automation_buff(op, registry))
        .map(|op| {
            let val = score_operator_facility_only(
                op,
                &room.room_type,
                formula_type,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
            );
            (op, val)
        })
        .filter(|(_, v)| *v > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut ops = Vec::new();
    let mut total = 0.0;
    for (op, val) in scored.into_iter().take(max_slots.max(0) as usize) {
        ops.push(op.char_id.clone());
        total += val;
    }
    (ops, total)
}

fn has_automation_buff(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    op.available_buffs.iter().any(|b| {
        matches!(
            registry.get(b),
            Some(BuffResolutionStrategy::FacilityCountScaling {
                nullifies_others: true,
                ..
            })
        )
    })
}

/// True if the operator nullifies teammates' output in this room (Shamare-type).
fn op_is_nullifier(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> bool {
    applicable_strategies(op, room_type, formula_type, registry, building_data).any(|s| {
        matches!(
            s,
            BuffResolutionStrategy::NullifyTeammatesSelfScaling { .. }
        )
    })
}

/// An optimistic upper bound on an operator's solo value in a room, used only
/// for ranking candidates (conditionals assumed satisfied, teammate-scaling
/// assumed fully matched). Never used as the final score.
#[allow(clippy::too_many_arguments)]
fn optimistic_bound(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    max_slots: usize,
) -> f64 {
    let mut total = 0.0;
    let teammates_assumed = max_slots.saturating_sub(1) as f64;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::DirectEfficiency { value }
            | BuffResolutionStrategy::Complex {
                estimated_pct: value,
            }
            | BuffResolutionStrategy::OrderValue {
                estimated_pct: value,
                ..
            }
            | BuffResolutionStrategy::MoraleDecayEfficiency {
                time_averaged_value: value,
            } => {
                total += value;
            }
            BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
                total += efficiency;
            }
            // Optimistic: assume every gating teammate / faction is present.
            BuffResolutionStrategy::ConditionalOnTeammate {
                base_efficiency,
                efficiency,
                ..
            }
            | BuffResolutionStrategy::ConditionalOnFaction {
                base_efficiency,
                efficiency,
                ..
            } => {
                total += base_efficiency + efficiency;
            }
            BuffResolutionStrategy::TeammateSkillScaling { per_match_pct, .. } => {
                total += per_match_pct * teammates_assumed;
            }
            BuffResolutionStrategy::MatchCountScaling {
                per_match_pct,
                cap_pct,
                bonus_pct,
                ..
            } => {
                let v = per_match_pct * teammates_assumed;
                total += cap_pct.map_or(v, |c| v.min(c)) + bonus_pct;
            }
            BuffResolutionStrategy::NullifyTeammatesSelfScaling { per_teammate_pct } => {
                total += per_teammate_pct * teammates_assumed;
            }
            BuffResolutionStrategy::TeammateOutputMirroring { cap_pct, .. } => total += cap_pct,
            // Bubble: optimistically assume every operator in the room (this one + teammates)
            // clears the capacity threshold and earns the high tier.
            BuffResolutionStrategy::CapacityTierScaling { high_pct, .. } => {
                total += high_pct * (teammates_assumed + 1.0);
            }
            // Output scales with teammates that ADD order/capacity limit. TRADING posts almost
            // never field those (Jaye/Degenbrecher virtually never fire there), so keep their
            // bound at 0. FACTORIES, though, routinely stack capacity operators - this is
            // Vermeil's whole comp - so credit an optimistic capacity-rich room there, otherwise
            // she'd rank at zero and get cut before the real scorer ever tries her with the
            // capacity teammates that make her strong.
            BuffResolutionStrategy::OrderLimitScaling {
                bonus_per_threshold,
                per_cap_threshold,
                cap_pct,
                ..
            } if room_type == "MANUFACTURE" => {
                const ASSUMED_CAP: f64 = 20.0;
                total +=
                    ((ASSUMED_CAP / per_cap_threshold).floor() * bonus_per_threshold).min(*cap_pct);
            }
            strategy @ BuffResolutionStrategy::FacilityCountScaling { .. } => {
                let ctx = EvalContext {
                    facility_counts,
                    total_dorm_levels,
                    room_teammates: Vec::new(),
                    self_order_limit: 0,
                };
                total += evaluate_buff(strategy, &ctx);
            }
            _ => {}
        }
    }
    total
}

/// Generate all non-empty combinations of `pool` with size 1..=`max_size`.
fn combinations(pool: &[String], max_size: usize) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let max_size = max_size.min(pool.len());
    let mut current = Vec::new();
    for size in 1..=max_size {
        combos_of_size(pool, size, 0, &mut current, &mut result);
    }
    result
}

fn combos_of_size(
    pool: &[String],
    size: usize,
    start: usize,
    current: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    if current.len() == size {
        out.push(current.clone());
        return;
    }
    // Stop early if not enough remaining elements to reach `size`.
    let need = size - current.len();
    if pool.len() - start < need {
        return;
    }
    for i in start..pool.len() {
        current.push(pool[i].clone());
        combos_of_size(pool, size, i + 1, current, out);
        current.pop();
    }
}

/// Score only `FacilityCountScaling` buffs for an operator (used in automation rooms).
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
        room_teammates: Vec::new(),
        self_order_limit: 0,
    };

    let mut total = 0.0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        if matches!(
            strategy,
            BuffResolutionStrategy::FacilityCountScaling { .. }
        ) {
            total += evaluate_buff(strategy, &ctx);
        }
    }
    total
}

/// Compute only the `DirectEfficiency`-style contribution for an operator,
/// used as a teammate's `direct_efficiency` (feeds `TeammateOutputMirroring`).
/// Conditional efficiency is counted only when its required teammate is present.
fn compute_direct_efficiency(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> f64 {
    let mut total = 0.0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::DirectEfficiency { value } => total += value,
            BuffResolutionStrategy::EfficiencyWithOrderLimit { efficiency, .. } => {
                total += efficiency;
            }
            BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id,
                base_efficiency,
                efficiency,
                ..
            } => {
                total += base_efficiency;
                if let Some(req) = required_char_id
                    && present.contains(req)
                {
                    total += efficiency;
                }
            }
            // Faction-conditional: only the always-on base counts here - the gated
            // bonus needs teammate faction tags this mirroring source doesn't resolve.
            BuffResolutionStrategy::ConditionalOnFaction {
                base_efficiency, ..
            } => {
                total += base_efficiency;
            }
            _ => {}
        }
    }
    total
}

/// Compute the total order limit contribution for an operator in a room.
/// Conditional order limit counts only when its required teammate is present.
fn compute_order_limit(
    op: &OperatorBaseProfile,
    room_type: &str,
    formula_type: Option<&str>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    present: &HashSet<String>,
) -> i32 {
    let mut total: i32 = 0;
    for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data) {
        match strategy {
            BuffResolutionStrategy::EfficiencyWithOrderLimit { order_limit, .. } => {
                total += order_limit;
            }
            BuffResolutionStrategy::ConditionalOnTeammate {
                required_char_id: Some(req),
                order_limit,
                ..
            } if present.contains(req) => total += order_limit,
            // A pure capacity skill (Vermeil's "+8") feeds the room's order/capacity limit,
            // which capacity-scaling operators convert into productivity.
            BuffResolutionStrategy::CapacityOnly { order_limit } => total += order_limit,
            _ => {}
        }
    }
    total
}

/// Room efficiency resolved against the full team (conditionals, teammate- and
/// faction-scaling, output mirroring, order-limit scaling). Returns
/// `(speed_pct, value_pct)`: order-acquisition SPEED (the productivity % the game
/// shows) and order VALUE (LMD per order, Proviso etc.), kept separate so value
/// doesn't inflate the displayed efficiency.
#[allow(clippy::too_many_arguments)]
fn compute_team_efficiency(
    member_ids: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
) -> (f64, f64) {
    // Resolve members once via the index (O(1) each) instead of scanning.
    let members: Vec<&OperatorBaseProfile> = member_ids
        .iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .collect();
    let present: HashSet<String> = members.iter().map(|o| o.char_id.clone()).collect();

    // Collect any skill-type converters in the room (Highmore-style). They
    // reclassify teammates' tags so e.g. Rhine operators count for a
    // Standardization scaler.
    let converters: Vec<(&[String], &str)> = members
        .iter()
        .flat_map(|op| {
            applicable_strategies(op, room_type, formula_type, registry, building_data).filter_map(
                |s| match s {
                    BuffResolutionStrategy::SkillTypeConversion {
                        from_tokens,
                        to_token,
                    } => Some((from_tokens.as_slice(), to_token.as_str())),
                    _ => None,
                },
            )
        })
        .collect();

    let teammates: Vec<TeammateInfo> = members
        .iter()
        .map(|op| {
            // Precomputed tags; only clone (no recomputation) and augment for converters.
            let mut match_tags = op.match_tags.clone();
            for (from, to) in &converters {
                if match_tags.iter().any(|t| from.contains(t))
                    && !match_tags.iter().any(|t| t == to)
                {
                    match_tags.push((*to).to_string());
                }
            }
            TeammateInfo {
                char_id: op.char_id.clone(),
                match_tags,
                buff_ids: op.available_buffs.clone(),
                direct_efficiency: compute_direct_efficiency(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                    &present,
                ),
                order_limit_contribution: compute_order_limit(
                    op,
                    room_type,
                    formula_type,
                    registry,
                    building_data,
                    &present,
                ),
            }
        })
        .collect();

    // Shamare-type operators zero out every teammate's output. If one is in the
    // room, only the nullifier(s) contribute (their own value still scales with
    // the number of zeroed teammates).
    let has_nullifier = members
        .iter()
        .any(|op| op_is_nullifier(op, room_type, formula_type, registry, building_data));

    // Weedy-type automation (Weedy, Passenger, Windflit, Eunectes) sets the
    // productivity of all OTHER operators to 0 EXCEPT the part granted by facility
    // count - so several automation operators stack with one another, but ANY normal
    // operator (and a normal operator's own non-facility buffs) contributes nothing.
    // This is why a normal operator is dead weight in an automation team, and why
    // dropping an automation operator into a normal team wrecks it.
    let has_automation = members.iter().any(|op| has_automation_buff(op, registry));

    let mut speed = 0.0;
    // Order VALUE (extra LMD per Pure-Gold order) does NOT stack across operators:
    // they target the same orders by disjoint rules, so two value operators don't
    // combine (Proviso boosts the low/"defaulted" orders, while Tequila's bonus
    // EXPLICITLY excludes defaulted orders). Only the strongest value operator
    // applies, so a second value operator is wasted - the best partner for a value
    // operator is order-acquisition SPEED, not another value operator.
    let mut value: f64 = 0.0;

    for (i, op) in members.iter().enumerate() {
        // A nullifier (Shamare) zeroes teammates' *speed/efficiency*, but their
        // *order-value* buffs survive. So a nullified teammate still contributes its
        // OrderValue (subject to the non-stacking rule above).
        let nullified =
            has_nullifier && !op_is_nullifier(op, room_type, formula_type, registry, building_data);

        let others: Vec<&TeammateInfo> = teammates
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, t)| t)
            .collect();

        let ctx = EvalContext {
            facility_counts,
            total_dorm_levels,
            room_teammates: others,
            self_order_limit: teammates[i].order_limit_contribution,
        };

        let mut op_value: f64 = 0.0;
        for strategy in applicable_strategies(op, room_type, formula_type, registry, building_data)
        {
            // Order VALUE is tracked separately (non-stacking, see above); everything
            // else is order SPEED, which a nullifier zeroes.
            if let BuffResolutionStrategy::OrderValue {
                estimated_pct,
                pure_gold,
            } = strategy
            {
                // A Shamare-type nullifier shifts the post toward Precious-Metal
                // orders, so a Pure-Gold value (Proviso) no longer applies in its
                // team - only flat-LMD value (Tequila) survives. This is why Proviso,
                // unlike Tequila, does not benefit from Shamare.
                if !(has_nullifier
                    && *pure_gold
                    && !op_is_nullifier(op, room_type, formula_type, registry, building_data))
                {
                    op_value += estimated_pct;
                }
            } else if has_automation {
                // Only facility-count productivity survives an automation operator.
                if matches!(
                    strategy,
                    BuffResolutionStrategy::FacilityCountScaling { .. }
                ) {
                    speed += evaluate_buff(strategy, &ctx);
                }
            } else if !nullified {
                speed += evaluate_buff(strategy, &ctx);
            }
        }
        // Non-stacking: keep only the strongest order-value operator's contribution.
        value = value.max(op_value);
    }
    // Faction-gated Control Center bonuses: order SPEED, but only for the posts
    // whose team actually satisfies the faction condition.
    speed += cc_conditions
        .iter()
        .map(|c| c.contribution(room_type, &members))
        .sum::<f64>();

    // Trading posts can only bank so many orders. An operator that slashes the
    // order limit (Degenbrecher: -6) strands the post far below capacity, so its
    // throughput is throttled no matter how fast it acquires orders. Fold that
    // bottleneck into the speed multiplier. Surplus limit is NOT rewarded (active
    // collection, not buffer size, drives throughput), so the factor caps at 1.0.
    if room_type == "TRADING" {
        let net_limit: i32 = teammates.iter().map(|t| t.order_limit_contribution).sum();
        let effective = (BASE_TRADING_ORDER_LIMIT + net_limit).max(MIN_TRADING_ORDER_LIMIT);
        let capacity = (f64::from(effective) / f64::from(BASE_TRADING_ORDER_LIMIT)).min(1.0);
        speed = ((1.0 + speed / 100.0) * capacity - 1.0) * 100.0;
    }

    let _ = morale_drains;
    (speed, value)
}

/// Score an arbitrary team stationed in a production room with the same speed/value objective
/// the room search uses - so a caller can tell whether a player's CURRENT team is already at
/// least as good as a recommended one (and the plan needn't nag them to swap). `operators` is
/// the full roster, needed to resolve in-room synergies (e.g. Dorothy's bonus to a Rhine
/// teammate). Faction-gated Control-Center conditions are ignored - a small approximation that
/// keeps this a pure team-vs-team comparison.
#[allow(clippy::too_many_arguments)]
pub fn team_value(
    team: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let op_index = build_op_index(operators);
    let facility_counts = effective_facility_counts(building, operators, registry);
    let (speed, value) = compute_team_efficiency(
        team,
        room_type,
        formula_type,
        &op_index,
        registry,
        building_data,
        &facility_counts,
        building.total_dorm_levels(),
        morale_drains,
        &[],
    );
    room_search_score(room_type, speed, value)
}

/// The optimizer's per-room objective: a room's LMD-equivalent output, with order
/// VALUE multiplying trading LMD (not adding to speed). Within a room the base
/// rate is constant, so maximizing this multiplier maximizes the room's yield.
fn room_search_score(room_type: &str, speed_pct: f64, value_pct: f64) -> f64 {
    let speed_mult = 1.0 + speed_pct / 100.0;
    if room_type == "TRADING" {
        speed_mult * (1.0 + value_pct / 100.0)
    } else {
        speed_mult
    }
}

// ── Cap-aware sustained model (AFK product-buffer stall) ──────────────────────
// A factory makes products into a fixed buffer; once full, it STOPS. A player who
// leaves the base running and collects on a cadence loses everything a factory
// makes past its buffer, so high-throughput / low-capacity factories stall and a
// LARGE capacity (Vermeil's "+8" etc.) keeps a factory producing across long AFK.
// Peak/active play never stalls (you collect as you go), so this only shapes the
// SUSTAINED objective.
/// Assumed days between product collections for the sustained metric - the buffer
/// stalls a factory that overproduces within this window. Tunable; ~1.5 days is the
/// "set it and check back" cadence where factory capacity starts to matter without
/// over-penalising a normal base that collects more often.
const SUSTAINED_AFK_DAYS: f64 = 1.5;
/// Gold bars a Pure-Gold factory produces per day at +0% (mirrors `yield_model`'s base).
const GOLD_PRODUCTS_PER_DAY_BASE: f64 = 20.0;

/// A factory level's base product-buffer capacity (`OutputCapacity` in gamedata).
const fn gold_base_capacity(level: i32) -> f64 {
    match level {
        1 => 24.0,
        2 => 36.0,
        _ => 54.0,
    }
}

/// Fraction of a GOLD factory's peak output that survives the AFK buffer stall (<= 1.0): the
/// buffer (base capacity + operators' capacity bonuses) caps how much it can make before
/// stopping, so a factory whose peak production over `SUSTAINED_AFK_DAYS` exceeds its buffer is
/// throttled to buffer/window. Capacity operators raise the buffer and lift this toward 1.0.
/// Non-gold factories don't stall in realistic AFK (an EXP buffer dwarfs its card output), so
/// they return 1.0.
fn factory_cap_factor(formula: Option<&str>, level: i32, efficiency: f64, capacity_bonus: i32) -> f64 {
    if formula != Some("F_GOLD") {
        return 1.0;
    }
    let buffer = gold_base_capacity(level) + f64::from(capacity_bonus.max(0));
    let peak_over_window =
        GOLD_PRODUCTS_PER_DAY_BASE * (1.0 + efficiency / 100.0) * SUSTAINED_AFK_DAYS;
    if peak_over_window <= 0.0 {
        return 1.0;
    }
    (buffer / peak_over_window).min(1.0)
}

/// A team's total capacity-limit bonus in a room (sum of its operators' order/capacity-limit
/// contributions) - the buffer headroom it adds for the cap-aware sustained model.
fn team_capacity_bonus(
    team: &[String],
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> i32 {
    let present: HashSet<String> = team.iter().cloned().collect();
    team.iter()
        .filter_map(|id| op_index.get(id.as_str()).copied())
        .map(|op| compute_order_limit(op, room_type, formula_type, registry, building_data, &present))
        .sum()
}

/// Total realized daily output of a production assignment, as a single
/// LMD-equivalent value. Each room's efficiency is soft-capped (per-room
/// throughput ceiling) and converted to its resource yield; the gold→trade loop
/// is then coupled so LMD = min(gold made, gold sold) × 500 and EXP adds at 1:1.
fn assignment_value(rooms: &[RoomAssignment]) -> f64 {
    let mut flows = crate::core::grade::base::yield_model::BaseFlows::default();
    for r in rooms {
        let speed = room_value(r.total_efficiency, &r.room_type);
        flows.add_room(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            speed,
            r.order_value,
        );
    }
    flows.total_value()
}

/// Soft-capped value of a room's raw efficiency. Production rooms have a
/// practical throughput ceiling (orders/credits accumulate only up to the order
/// limit between collections), so efficiency stacked far past that point is
/// worth progressively less. Modeling this is what makes spreading operators
/// across multiple rooms beat dumping every strong operator into one.
fn room_value(raw_efficiency: f64, room_type: &str) -> f64 {
    // Beyond this much total efficiency, extra % is discounted.
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

/// Balance operators across rooms that share the same type and formula by
/// swapping pairs whenever it raises the soft-capped objective. With multiple
/// trading posts / gold factories this spreads efficiency instead of stacking it
/// all in one room (diminishing returns). Only same-type/formula rooms are
/// considered so buff applicability is identical on both sides of a swap.
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
    // (speed incl. global, value) for a room's operator set.
    let recompute = |room_type: &str, formula: Option<&str>, ops: &[String]| -> (f64, f64) {
        let (speed, value) = compute_team_efficiency(
            ops,
            room_type,
            formula,
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
    // Balancing objective: soft-capped speed (diminishing returns) × order value.
    let obj = |room_type: &str, speed: f64, value: f64| -> f64 {
        room_search_score(room_type, room_value(speed, room_type), value)
    };

    // Up to a few passes; each pass tries all same-type/formula room pairs.
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

#[cfg(test)]
mod cap_tests {
    use super::{factory_cap_factor, gold_base_capacity, SUSTAINED_AFK_DAYS};

    #[test]
    fn cap_factor_throttles_high_throughput_low_capacity_gold() {
        // A high-productivity gold factory with no extra capacity overflows its buffer during
        // AFK and is throttled below 1.0; piling on capacity (Vermeil-style) lifts it back to 1.0.
        let lvl = 3;
        let high_eff = 110.0;
        let no_cap = factory_cap_factor(Some("F_GOLD"), lvl, high_eff, 0);
        assert!(no_cap < 1.0, "a +110% gold factory with no capacity should stall, got {no_cap}");
        let with_cap = factory_cap_factor(Some("F_GOLD"), lvl, high_eff, 40);
        assert!(with_cap > no_cap, "adding capacity should reduce the stall ({no_cap} -> {with_cap})");
        // Enough capacity to hold the whole AFK window's production -> no stall.
        assert!((with_cap - 1.0).abs() < 1e-9 || with_cap >= no_cap);
    }

    #[test]
    fn cap_factor_is_neutral_for_low_throughput_and_non_gold() {
        // A modest gold factory doesn't fill its buffer in the window -> no throttle.
        assert!((factory_cap_factor(Some("F_GOLD"), 3, 10.0, 0) - 1.0).abs() < 1e-9);
        // EXP and trading don't stall in realistic AFK -> always 1.0.
        assert_eq!(factory_cap_factor(Some("F_EXP"), 3, 300.0, 0), 1.0);
        assert_eq!(factory_cap_factor(None, 3, 300.0, 0), 1.0);
        // Sanity on the constants the model rests on.
        assert!(SUSTAINED_AFK_DAYS > 0.0 && gold_base_capacity(3) > gold_base_capacity(1));
    }
}
