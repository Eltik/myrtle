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
    types::{BaseAssignment, OperatorBaseProfile, UserBuilding},
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

/// Operators staffing the production rooms of `assignment` (used to exclude a
/// deployment's crews when forming the reserve teams).
fn production_operators(assignment: &BaseAssignment) -> Vec<String> {
    assignment
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .flat_map(|r| r.operators.iter().cloned())
        .collect()
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
    // use - together these give enough teams to rotate.
    let primary =
        compute_optimal_assignment(operators, building, building_data, registry, morale_drains);
    let used: std::collections::HashSet<String> =
        production_operators(&primary).into_iter().collect();
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

    // Control Center and Power plants, taken from the best deployment.
    let cc_ops: Vec<(String, Vec<String>)> = primary
        .rooms
        .iter()
        .filter(|r| r.room_type == "CONTROL")
        .map(|r| (r.slot_id.clone(), r.operators.clone()))
        .collect();
    // slot_id -> per-shift fresh power crews.
    let power_plan = build_power_plan(operators, building, building_data, &primary);

    let mut shifts = Vec::with_capacity(SHIFT_COUNT);
    for k in 0..SHIFT_COUNT {
        let mut rooms = Vec::new();

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

        // Power plants: a fresh crew every shift so the operators actually rest.
        for (slot, shift_crews) in &power_plan {
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: "POWER".to_string(),
                formula_type: None,
                recommended: shift_crews.get(k).cloned().unwrap_or_default(),
                current: preset_for(slot, k),
                active: true,
            });
        }

        // Control Center: two shifts on, one off (the last shift rests it).
        let cc_active = k != SHIFT_COUNT - 1;
        for (slot, ops) in &cc_ops {
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: "CONTROL".to_string(),
                formula_type: None,
                recommended: ops.clone(),
                current: preset_for(slot, k),
                active: cc_active,
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
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "POWER")
    })
}

/// Power plants generate power regardless of who staffs them, so the optimizer doesn't
/// pick their crews. Fill them with the leftover POWER SPECIALISTS (operators with a
/// power base skill that aren't already staffing a production room or the Control
/// Center), giving each plant a fresh crew every shift so they share the load and rest.
/// Operators with no power skill are left out entirely - parking them here would do
/// nothing but drain their morale. Returns each plant's per-shift crews in shift order.
fn build_power_plan(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    primary: &BaseAssignment,
) -> Vec<(String, Vec<Vec<String>>)> {
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
    let mut leftovers = operators
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .filter(|op| has_power_skill(op, building_data))
        .map(|op| op.char_id.clone());

    // Hand out a fresh crew per plant per shift, sequentially from the leftover pool.
    let mut plan: Vec<(String, Vec<Vec<String>>)> = power_rooms
        .iter()
        .map(|r| (r.slot_id.clone(), vec![Vec::new(); SHIFT_COUNT]))
        .collect();
    for shift in 0..SHIFT_COUNT {
        for (i, room) in power_rooms.iter().enumerate() {
            let slots = max_stationed_at_level(building_data, "POWER", room.level).max(1) as usize;
            for _ in 0..slots {
                if let Some(op) = leftovers.next() {
                    plan[i].1[shift].push(op);
                }
            }
        }
    }
    plan
}
