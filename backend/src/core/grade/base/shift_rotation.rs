//! Recommended 3-shift base rotation, for comparing the player's in-game presets
//! against an optimized rotation.
//!
//! EVERY room follows the same work / rest / work cycle: its MAIN team (the best
//! deployment) works shifts 1 & 3 and rests the MIDDLE shift, where a reserve
//! OFF-team covers it (or the room rests dark when there's no reserve):
//!
//! ```text
//!   Shift 1: main      Shift 2: off-team      Shift 3: main
//! ```
//!
//! So no operator works two shifts back-to-back, and nobody is held 24/7 - with one carve-out: an
//! operator the player explicitly sustains with a morale-swap manager (Fiammetta), detected from
//! their saved presets, keeps working the middle shift too (and is labelled as such).
//! Resting the WHOLE base on the same middle shift keeps the primary team together
//! on shifts 1 & 3, which is what cross-room buff synergies need - e.g. Viviana's
//! Control-Center buff only reaches full strength while her Knight operators are
//! working in the production rooms, so they must share a rest cycle. This is also
//! the player's two-preset swap: shifts 1 & 3 are "preset A", the middle is "preset B".

use std::collections::{HashMap, HashSet, VecDeque};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::types::UserRoom;
use super::{
    assignment::{
        compute_optimal_assignment, morale_recovery, num_morale_swap_managers, op_uptime,
    },
    buff_registry::BuffResolutionStrategy,
    evaluate::evaluate_buff,
    types::{BaseAssignment, EvalContext, OperatorBaseProfile, UserBuilding},
    util::{is_production_room, max_stationed_at_level},
};

/// The number of shifts in a recommended rotation (each team rests one of three).
pub const SHIFT_COUNT: usize = 3;

/// A full recommended rotation: `SHIFT_COUNT` shifts, each listing every room's
/// recommended crew alongside the player's preset for that room and shift.
pub struct ShiftRotation {
    pub shifts: Vec<Shift>,
    /// Operators the player runs 24/7 with a morale-swap manager (Fiammetta) - kept working every
    /// shift instead of resting the middle one. The frontend badges these as "24/7 - Fiammetta".
    pub sustained: Vec<String>,
}

pub struct Shift {
    /// 1-indexed shift number.
    pub index: usize,
    pub rooms: Vec<ShiftRoom>,
}

pub struct ShiftRoom {
    pub slot_id: String,
    pub room_type: String,
    /// Production formula for factories (`F_GOLD`/`F_EXP`/...), else `None`.
    pub formula_type: Option<String>,
    /// Recommended operators (`char_id`s) for this room in this shift.
    pub recommended: Vec<String>,
    /// The player's preset operators for this room in this shift, if they have one.
    pub current: Vec<String>,
    /// False when the room is deliberately UNSTAFFED this shift (the Control Center's
    /// off shift). The crew still listed is "who staffs it the other two shifts".
    pub active: bool,
}

/// A production room's rotation plan: its MAIN team (the best deployment) works shifts 1 & 3 and
/// rests the MIDDLE shift, where its reserve OFF-team covers it (or the room rests dark). So no
/// operator ever works two shifts back-to-back - shifts 1 & 3 are the player's "preset A" and the
/// middle shift is "preset B".
struct ProdRoom {
    slot_id: String,
    room_type: String,
    formula_type: Option<String>,
    main: Vec<String>,
    off: Option<Vec<String>>,
    /// Operators in `main` that a morale-swap manager holds 24/7 - they stay working on the off
    /// shift (filling slots ahead of the reserve team) instead of resting with the rest of `main`.
    kept: Vec<String>,
}

/// Operators the player runs 24/7 - present in EVERY saved preset of a production room, so they're
/// kept in across both presets and never rotated out - that a morale-swap manager (Fiammetta) can
/// hold at full morale around the clock. Capped at the number of managers owned, preferring the
/// operators with the LOWEST natural uptime (highest morale drain): those are the ones that actually
/// need a manager to sustain 24/7, whereas a low-drain operator sustains itself and shouldn't spend
/// a manager. Empty when the roster owns no manager or runs nobody across every preset.
fn preset_sustained_operators(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    building_data: &BuildingDataFile,
) -> HashSet<String> {
    let managers = num_morale_swap_managers(operators, building_data);
    if managers == 0 {
        return HashSet::new();
    }
    // 24/7 candidates: operators present in every (non-empty) saved preset of a production room. Two
    // distinct presets are required - a single saved preset is just "the current team", not a swap
    // the player deliberately holds around the clock.
    let mut candidates: HashSet<String> = HashSet::new();
    for room in building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let presets: Vec<&Vec<String>> = room
            .preset_shifts
            .iter()
            .filter(|p| !p.is_empty())
            .collect();
        if presets.len() < 2 {
            continue;
        }
        for op in presets[0] {
            if presets.iter().all(|p| p.contains(op)) {
                candidates.insert(op.clone());
            }
        }
    }
    // A manager holds the operators that benefit most - lowest natural uptime (highest drain) first.
    let recovery = morale_recovery(building);
    let op_index: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();
    let mut ranked: Vec<String> = candidates.into_iter().collect();
    ranked.sort_by(|a, b| {
        let ua = op_index
            .get(a.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        let ub = op_index
            .get(b.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        ua.partial_cmp(&ub)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });
    ranked.truncate(managers);
    ranked.into_iter().collect()
}

/// A room's off-shift team: the manager-sustained operators stay (they never rest), and the reserve
/// off-team fills the remaining slots, skipping anyone already kept. Capped at the room's capacity
/// so it's never overstaffed. With nothing kept this is just the reserve team (the old behaviour).
fn merge_kept(kept: &[String], off_team: &[String], capacity: usize) -> Vec<String> {
    let mut team: Vec<String> = kept.to_vec();
    for op in off_team {
        if team.len() >= capacity {
            break;
        }
        if !team.contains(op) {
            team.push(op.clone());
        }
    }
    team
}

/// Key a production room by what it produces, so Trading Posts, Gold factories, and EXP factories
/// each draw their reserve off-team from their own pool.
fn group_key(room_type: &str, formula: Option<&str>) -> String {
    match formula {
        Some(f) => format!("{room_type}:{f}"),
        None => room_type.to_string(),
    }
}

/// Build the recommended rotation for the player's base, pairing each shift's
/// recommended crews with their saved presets.
pub fn recommend_shift_rotation(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> ShiftRotation {
    // Best deployment, then the best reserve deployment from the operators it didn't
    // use - together these give enough teams to rotate. The reserve excludes EVERY operator
    // the primary stations (production AND the Control Center), so the reserve's Control-Center
    // crew is a genuinely different off-team that lets the primary CC operators actually rest -
    // not the same faces re-picked.
    let primary =
        compute_optimal_assignment(operators, building, building_data, registry, morale_drains);
    let used: std::collections::HashSet<String> = primary
        .rooms
        .iter()
        .flat_map(|r| r.operators.iter().cloned())
        .collect();
    let reserve_pool: Vec<OperatorBaseProfile> = operators
        .iter()
        .filter(|op| !used.contains(&op.char_id))
        .cloned()
        .collect();
    let reserve = compute_optimal_assignment(
        &reserve_pool,
        building,
        building_data,
        registry,
        morale_drains,
    );

    // Operators the player holds at full morale 24/7 with a manager (Fiammetta). They keep working
    // through the middle (rest) shift instead of rotating out - the one carve-out from "nobody is
    // forced 24/7", because the player explicitly sustains them and the frontend labels it.
    let sustained = preset_sustained_operators(building, operators, morale_drains, building_data);

    // Production rooms run a work / rest / work cycle: the main team works shifts 1 & 3 and rests the
    // MIDDLE shift, where a reserve OFF-team of the same kind covers it (or the room rests dark when
    // the roster has no reserve). No operator works two shifts back-to-back, and nobody is forced
    // 24/7 - this is the two-preset swap (shifts 1 & 3 = preset A, the middle shift = preset B).
    // Reserve teams are matched to the main rooms by what they produce, in order.
    let mut reserve_offs: HashMap<String, VecDeque<Vec<String>>> = HashMap::new();
    for room in reserve
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        reserve_offs
            .entry(group_key(&room.room_type, room.formula_type.as_deref()))
            .or_default()
            .push_back(room.operators.clone());
    }
    let prod_rooms: Vec<ProdRoom> = primary
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .map(|r| ProdRoom {
            slot_id: r.slot_id.clone(),
            room_type: r.room_type.clone(),
            formula_type: r.formula_type.clone(),
            off: reserve_offs
                .get_mut(&group_key(&r.room_type, r.formula_type.as_deref()))
                .and_then(VecDeque::pop_front),
            kept: r
                .operators
                .iter()
                .filter(|op| sustained.contains(*op))
                .cloned()
                .collect(),
            main: r.operators.clone(),
        })
        .collect();

    // Preset shifts per room slot, from the in-game preset queue.
    let presets: HashMap<&str, &Vec<Vec<String>>> = building
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), &r.preset_shifts))
        .collect();
    // The player saves two presets and swaps between them, matching the rotation's work / rest /
    // work shape: the main shifts (1 & 3) run preset A and the rest shift (2) runs preset B. So map
    // both main shifts to preset 0 and the rest shift to preset 1, rather than indexing presets by
    // the raw shift number (which would leave the third shift with no preset to compare against).
    let preset_for = |slot: &str, k: usize| -> Vec<String> {
        let preset_idx = usize::from(k == SHIFT_COUNT / 2);
        presets
            .get(slot)
            .and_then(|shifts| shifts.get(preset_idx))
            .cloned()
            .unwrap_or_default()
    };

    // Control Center: the best crew from the primary deployment, plus an OFF-SHIFT backup
    // crew from the reserve deployment so the Center keeps running while the main team rests
    // (mirrors how production rotates) instead of going dark a shift.
    let cc_main: Vec<(String, Vec<String>)> = primary
        .rooms
        .iter()
        .filter(|r| r.room_type == "CONTROL")
        .map(|r| (r.slot_id.clone(), r.operators.clone()))
        .collect();
    let cc_backup: Vec<Vec<String>> = reserve
        .rooms
        .iter()
        .filter(|r| r.room_type == "CONTROL")
        .map(|r| r.operators.clone())
        .collect();

    // Auxiliary facilities (HR Office, Reception Room): a rotating main + off-team, like the
    // production rooms and Control Center. The main crew comes from the best deployment and the
    // off-team from the reserve deployment (different operators, so the main actually rests),
    // matched by slot. With no reserve crew the room just rests its shift (the old behaviour).
    let aux_main: Vec<(String, String, Vec<String>)> = primary
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type.as_str(), "HIRE" | "MEETING"))
        .map(|r| (r.slot_id.clone(), r.room_type.clone(), r.operators.clone()))
        .collect();
    let aux_backup: HashMap<String, Vec<String>> = reserve
        .rooms
        .iter()
        .filter(|r| matches!(r.room_type.as_str(), "HIRE" | "MEETING"))
        .map(|r| (r.slot_id.clone(), r.operators.clone()))
        .collect();

    // Power plants: the best stable crew per plant.
    let facility_counts: HashMap<String, usize> =
        building.rooms.iter().fold(HashMap::new(), |mut m, r| {
            *m.entry(r.room_type.clone()).or_insert(0) += 1;
            m
        });
    let power_plan = build_power_plan(
        operators,
        building,
        building_data,
        registry,
        &facility_counts,
        &primary,
        &reserve,
    );

    let mut shifts = Vec::with_capacity(SHIFT_COUNT);
    for k in 0..SHIFT_COUNT {
        let mut rooms = Vec::new();
        // EVERY room rests its main on the MIDDLE shift (work / off / work). Synchronising the rest
        // across the whole base keeps the primary team together on shifts 1 & 3, so cross-room buff
        // synergies hold - e.g. Viviana's Control-Center buff only reaches full strength while her
        // Knight operators are working in the production rooms; resting them on different shifts
        // would break it. The middle shift runs the reserve (the player's "preset B").
        let off = k == SHIFT_COUNT / 2;

        // Production rooms: main team on shifts 1 & 3, reserve off-team on the middle shift (or dark
        // when there's no reserve), so no operator works two shifts back-to-back. The sole exception
        // is a manager-sustained operator (`kept`): it stays working the middle shift too, taking a
        // slot ahead of the reserve team, because the player explicitly holds it 24/7 with Fiammetta.
        for room in &prod_rooms {
            let (team, active) = match (off, &room.off) {
                (true, Some(off_team)) => (merge_kept(&room.kept, off_team, room.main.len()), true),
                // No reserve team: the room would rest dark, but a sustained operator keeps working
                // it alone (understaffed) rather than dropping its 24/7 output.
                (true, None) if !room.kept.is_empty() => (room.kept.clone(), true),
                (true, None) => (room.main.clone(), false),
                (false, _) => (room.main.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: room.slot_id.clone(),
                room_type: room.room_type.clone(),
                formula_type: room.formula_type.clone(),
                recommended: team,
                current: preset_for(&room.slot_id, k),
                active,
            });
        }

        // Power plants: the main specialist on shifts 1 & 3, its off-team on the middle shift (or
        // resting dark - a plant generates electricity regardless of who staffs it), like every
        // other room, so its operators recover instead of working two shifts in a row.
        for plant in &power_plan {
            let (crew, active) = match (off, plant.backup.is_empty()) {
                (true, false) => (plant.backup.clone(), true),
                (true, true) => (plant.main.clone(), false),
                (false, _) => (plant.main.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: plant.slot_id.clone(),
                room_type: "POWER".to_string(),
                formula_type: None,
                recommended: crew,
                current: preset_for(&plant.slot_id, k),
                active,
            });
        }

        // Auxiliary facilities (Office, Reception): main crew two shifts, then the off-team
        // covers the rest shift (rests on the MIDDLE shift, like the Control Center, so its
        // operators never sit two shifts back-to-back). With no reserve crew the room rests dark.
        for (slot, room_type, main) in &aux_main {
            let backup = aux_backup.get(slot).filter(|b| !b.is_empty());
            let (team, active) = match (off, backup) {
                (true, Some(b)) => (b.clone(), true),
                (true, None) => (main.clone(), false),
                (false, _) => (main.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: room_type.clone(),
                formula_type: None,
                recommended: team,
                current: preset_for(slot, k),
                active,
            });
        }

        // Control Center: main crew two shifts, then the backup off-team covers the rest shift
        // so the global buff never drops. With no reserve crew (small roster) the room genuinely
        // has no off-team, so it rests that shift (active = false), the old behaviour.
        for (i, (slot, main)) in cc_main.iter().enumerate() {
            let backup = cc_backup.get(i).filter(|b| !b.is_empty());
            let (team, active) = match (off, backup) {
                (true, Some(b)) => (b.clone(), true),
                (true, None) => (main.clone(), false),
                (false, _) => (main.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: "CONTROL".to_string(),
                formula_type: None,
                recommended: team,
                current: preset_for(slot, k),
                active,
            });
        }

        shifts.push(Shift {
            index: k + 1,
            rooms,
        });
    }

    // Label only the operators actually held 24/7 in a recommended room (a detected 24/7 operator
    // the optimizer doesn't deploy keeps no slot, so it isn't surfaced).
    let mut sustained_label: Vec<String> = prod_rooms
        .iter()
        .flat_map(|r| r.kept.iter().cloned())
        .collect();
    sustained_label.sort();
    sustained_label.dedup();

    ShiftRotation {
        shifts,
        sustained: sustained_label,
    }
}

/// A power plant's electricity output is fixed by its level - operators only matter
/// if they carry a Power Plant base skill (drone recovery, shared morale drain, etc.).
/// An operator with no power skill (e.g. a pure Dormitory/Training operator) is dead
/// weight there, so only genuine power specialists are recommended for the plants.
fn has_power_skill(op: &OperatorBaseProfile, building_data: &BuildingDataFile) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "POWER")
    })
}

/// Rough strength of an operator's POWER base skill, for ranking who staffs the plants:
/// the largest value any of its power buffs resolves to (drone recovery, power output,
/// etc.). Promoted skills resolve higher, so an E2 power specialist outranks an E0 one.
fn power_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    let ctx = EvalContext {
        facility_counts,
        total_dorm_levels: 0,
        room_teammates: Vec::new(),
        self_order_limit: 0,
    };
    op.available_buffs
        .iter()
        .filter(|b| {
            building_data
                .buffs
                .get(*b)
                .is_some_and(|buff| buff.room_type == "POWER")
        })
        .filter_map(|b| registry.get(b))
        .map(|s| match s {
            // A facility-count enabler (Greyy the Lightningbearer E2) earns no drone output, but
            // it must be STATIONED in a power plant for its "+1 Power Plant" to fire and power the
            // automation factories - so rank it highly here, above ordinary drone operators.
            BuffResolutionStrategy::FacilityCountModifier {
                target_room,
                amount,
            } if target_room == "POWER" => f64::from(*amount) * 30.0,
            other => evaluate_buff(other, &ctx),
        })
        .fold(0.0, f64::max)
}

/// A power plant's main crew plus an off-team that covers its rest shift.
struct PowerPlant {
    slot_id: String,
    /// Best specialist(s), stationed two of the three shifts.
    main: Vec<String>,
    /// Next-best specialist(s), covering the shift the main rests (empty if the roster has no
    /// spare power operator, in which case the main simply stays on).
    backup: Vec<String>,
}

/// Power plants generate electricity regardless of who staffs them, so the optimizer
/// doesn't pick their crews. Staff each plant with the BEST leftover power specialists
/// (operators carrying a power base skill that aren't already in a production room or the
/// Control Center), ranked by power-skill strength so the strongest drone/power operators
/// go in - not whoever happens to be first in the roster. Each plant also gets an OFF-TEAM
/// (the next-best specialist) that covers the shift its main rests, so the plant rotates a
/// stable two-operator pair rather than churning through nine random picks. Operators with no
/// power skill are left out entirely - parking them here would do nothing but drain morale.
fn build_power_plan(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
    primary: &BaseAssignment,
    reserve: &BaseAssignment,
) -> Vec<PowerPlant> {
    let power_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .collect();
    if power_rooms.is_empty() {
        return Vec::new();
    }

    // Exclude every operator already stationed anywhere in the assembled rotation - BOTH the
    // primary and the reserve deployment, across all room types (production, Control Center,
    // Office/Reception). Otherwise a power specialist who also earns a reserve production seat or
    // an aux seat would be double-booked into two rooms in the same shift.
    let used: HashSet<&str> = primary
        .rooms
        .iter()
        .chain(reserve.rooms.iter())
        .flat_map(|r| r.operators.iter().map(String::as_str))
        .collect();
    let mut ranked: Vec<(String, f64)> = operators
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .filter(|op| has_power_skill(op, building_data))
        .map(|op| {
            (
                op.char_id.clone(),
                power_value(op, building_data, registry, facility_counts),
            )
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = ranked.into_iter().map(|(id, _)| id);

    let slots: Vec<usize> = power_rooms
        .iter()
        .map(|r| max_stationed_at_level(building_data, "POWER", r.level).max(1) as usize)
        .collect();
    // Hand every plant its main crew FIRST (so the strongest operators are the ones actually
    // working most), then a second pass fills each plant's off-team from what remains.
    let mut mains: Vec<Vec<String>> = slots
        .iter()
        .map(|n| (0..*n).filter_map(|_| best.next()).collect())
        .collect();
    let backups: Vec<Vec<String>> = slots
        .iter()
        .map(|n| (0..*n).filter_map(|_| best.next()).collect())
        .collect();
    power_rooms
        .iter()
        .enumerate()
        .map(|(i, r)| PowerPlant {
            slot_id: r.slot_id.clone(),
            main: std::mem::take(&mut mains[i]),
            backup: backups[i].clone(),
        })
        .collect()
}
