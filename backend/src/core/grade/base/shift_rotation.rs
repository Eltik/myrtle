//! Recommended 3-shift base rotation, for comparing the player's in-game presets
//! against an optimized rotation.
//!
//! Production rooms of each kind (Trading Posts, Gold factories, EXP factories) are
//! staffed by THREE teams that rotate so each rests exactly one shift:
//!
//! ```text
//!   Shift 1: teams {1, 2}    (team 3 rests)
//!   Shift 2: teams {2, 3}    (team 1 rests)
//!   Shift 3: teams {3, 1}    (team 2 rests)
//! ```
//!
//! Power plants swap their operators every shift; the Control Center runs two
//! shifts on, one off. The teams come from the optimizer (best deployment, plus a
//! reserve team formed from the remaining operators).

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::{
    assignment::compute_optimal_assignment,
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

/// A group of same-kind production rooms and the three teams that rotate through them.
struct RoomGroup {
    /// The physical room slots in this group (in assignment order).
    rooms: Vec<RoomSlot>,
    /// Three operator teams (`char_id` lists) that rotate across the rooms.
    teams: Vec<Vec<String>>,
}

struct RoomSlot {
    slot_id: String,
    room_type: String,
    formula_type: Option<String>,
}

/// Key a production room by what it produces, so Trading Posts, Gold factories, and
/// EXP factories each rotate within their own pool.
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

    let groups = build_production_groups(&primary, &reserve);

    // Preset shifts per room slot, from the in-game preset queue.
    let presets: HashMap<&str, &Vec<Vec<String>>> = building
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), &r.preset_shifts))
        .collect();
    let preset_for = |slot: &str, shift: usize| -> Vec<String> {
        presets
            .get(slot)
            .and_then(|shifts| shifts.get(shift))
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
    );

    let mut shifts = Vec::with_capacity(SHIFT_COUNT);
    for k in 0..SHIFT_COUNT {
        let mut rooms = Vec::new();
        // The Control Center rests its main on the MIDDLE shift (work / off / work) so its
        // operators never sit through two shifts back-to-back - they were staying in too long.
        // Power plants rest their main on the last shift, staggering the weaker off-teams onto
        // different shifts from the Control Center's.
        let cc_off = k == SHIFT_COUNT / 2;
        let aux_off = k == SHIFT_COUNT / 2;
        let power_off = k == SHIFT_COUNT - 1;

        // Production rooms: rotate the three teams so each rests one shift.
        for group in &groups {
            for (j, room) in group.rooms.iter().enumerate() {
                let team = &group.teams[(j + k) % group.teams.len().max(1)];
                rooms.push(ShiftRoom {
                    slot_id: room.slot_id.clone(),
                    room_type: room.room_type.clone(),
                    formula_type: room.formula_type.clone(),
                    recommended: team.clone(),
                    current: preset_for(&room.slot_id, k),
                    active: true,
                });
            }
        }

        // Power plants: the strongest specialist two shifts, then its off-team covers the third.
        for plant in &power_plan {
            let crew = if power_off && !plant.backup.is_empty() { &plant.backup } else { &plant.main };
            rooms.push(ShiftRoom {
                slot_id: plant.slot_id.clone(),
                room_type: "POWER".to_string(),
                formula_type: None,
                recommended: crew.clone(),
                current: preset_for(&plant.slot_id, k),
                active: true,
            });
        }

        // Auxiliary facilities (Office, Reception): main crew two shifts, then the off-team
        // covers the rest shift (rests on the MIDDLE shift, like the Control Center, so its
        // operators never sit two shifts back-to-back). With no reserve crew the room rests dark.
        for (slot, room_type, main) in &aux_main {
            let backup = aux_backup.get(slot).filter(|b| !b.is_empty());
            let (team, active) = match (aux_off, backup) {
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
            let (team, active) = match (cc_off, backup) {
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

    ShiftRotation { shifts }
}

/// Group the production rooms by what they produce and attach three rotation teams:
/// the deployment's own crews plus a reserve crew from the second deployment.
fn build_production_groups(primary: &BaseAssignment, reserve: &BaseAssignment) -> Vec<RoomGroup> {
    let mut order: Vec<String> = Vec::new();
    let mut by_key: HashMap<String, RoomGroup> = HashMap::new();

    for room in primary
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let key = group_key(&room.room_type, room.formula_type.as_deref());
        let group = by_key.entry(key.clone()).or_insert_with(|| {
            order.push(key.clone());
            RoomGroup {
                rooms: Vec::new(),
                teams: Vec::new(),
            }
        });
        group.rooms.push(RoomSlot {
            slot_id: room.slot_id.clone(),
            room_type: room.room_type.clone(),
            formula_type: room.formula_type.clone(),
        });
        group.teams.push(room.operators.clone());
    }

    // Reserve teams from the second deployment, matched by what they produce.
    let mut reserve_teams: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for room in reserve
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let key = group_key(&room.room_type, room.formula_type.as_deref());
        reserve_teams
            .entry(key)
            .or_default()
            .push(room.operators.clone());
    }

    // Top up each group to SHIFT_COUNT teams using the reserves.
    for (key, group) in &mut by_key {
        if let Some(extra) = reserve_teams.get(key) {
            for team in extra {
                if group.teams.len() >= SHIFT_COUNT {
                    break;
                }
                group.teams.push(team.clone());
            }
        }
        // If still short (small roster), reuse existing teams so the rotation is
        // defined - the comparison still shows the gap.
        while group.teams.len() < SHIFT_COUNT && !group.teams.is_empty() {
            let dup = group.teams[group.teams.len() % group.teams.len().max(1)].clone();
            group.teams.push(dup);
        }
    }

    order
        .into_iter()
        .filter_map(|k| by_key.remove(&k))
        .collect()
}

/// A power plant's electricity output is fixed by its level - operators only matter
/// if they carry a Power Plant base skill (drone recovery, shared morale drain, etc.).
/// An operator with no power skill (e.g. a pure Dormitory/Training operator) is dead
/// weight there, so only genuine power specialists are recommended for the plants.
fn has_power_skill(op: &OperatorBaseProfile, building_data: &BuildingDataFile) -> bool {
    op.available_buffs
        .iter()
        .any(|b| building_data.buffs.get(b).is_some_and(|buff| buff.room_type == "POWER"))
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
        .filter(|b| building_data.buffs.get(*b).is_some_and(|buff| buff.room_type == "POWER"))
        .filter_map(|b| registry.get(b))
        .map(|s| match s {
            // A facility-count enabler (Greyy the Lightningbearer E2) earns no drone output, but
            // it must be STATIONED in a power plant for its "+1 Power Plant" to fire and power the
            // automation factories - so rank it highly here, above ordinary drone operators.
            BuffResolutionStrategy::FacilityCountModifier { target_room, amount }
                if target_room == "POWER" =>
            {
                f64::from(*amount) * 30.0
            }
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
) -> Vec<PowerPlant> {
    let power_rooms: Vec<&super::types::UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .collect();
    if power_rooms.is_empty() {
        return Vec::new();
    }

    let used: HashSet<&str> = primary
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type) || r.room_type == "CONTROL")
        .flat_map(|r| r.operators.iter().map(String::as_str))
        .collect();
    let mut ranked: Vec<(String, f64)> = operators
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .filter(|op| has_power_skill(op, building_data))
        .map(|op| (op.char_id.clone(), power_value(op, building_data, registry, facility_counts)))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut best = ranked.into_iter().map(|(id, _)| id);

    let slots: Vec<usize> = power_rooms
        .iter()
        .map(|r| max_stationed_at_level(building_data, "POWER", r.level).max(1) as usize)
        .collect();
    // Hand every plant its main crew FIRST (so the strongest operators are the ones actually
    // working most), then a second pass fills each plant's off-team from what remains.
    let mut mains: Vec<Vec<String>> = slots.iter().map(|n| (0..*n).filter_map(|_| best.next()).collect()).collect();
    let backups: Vec<Vec<String>> = slots.iter().map(|n| (0..*n).filter_map(|_| best.next()).collect()).collect();
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
