use std::collections::{HashMap, HashSet};

use super::morale::op_lasts_hours;
use crate::{
    registry::BuffResolutionStrategy,
    rotation::sustain_sim::sustains_24h_block,
    types::{OperatorBaseProfile, RoomAssignment, UserBuilding},
    util::{is_production_room, max_stationed_at_level},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

pub(super) fn aux_room_value(
    op: &OperatorBaseProfile,
    room_type: &str,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> f64 {
    op.available_buffs
        .iter()
        .filter(|b| {
            building_data
                .buffs
                .get(*b)
                .is_some_and(|buff| buff.room_type == room_type)
        })
        .filter_map(|b| match registry.get(b) {
            Some(BuffResolutionStrategy::NonProduction { value }) => Some(*value),
            _ => None,
        })
        .fold(0.0, f64::max)
}

pub(super) const RECEPTION_INNATE_PCT: f64 = 5.0;

/// Rarity ambience bonus
pub(super) const fn rarity_bonus(stars: i16) -> f64 {
    match stars {
        6 => 5.0,
        5 => 4.0,
        4 => 2.0,
        _ => 0.0,
    }
}
/// Elite ambience bonus
pub(super) const fn elite_bonus(elite: i16) -> f64 {
    match elite {
        2 => 16.0,
        1 => 8.0,
        _ => 0.0,
    }
}
/// Reception Room level bonus
pub(super) const fn rr_level_bonus(level: i32) -> f64 {
    if level >= 3 {
        11.0
    } else if level == 2 {
        9.0
    } else {
        7.0
    }
}

pub(super) fn reception_skill(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    alone: bool,
) -> f64 {
    op.available_buffs
        .iter()
        .filter_map(|b| {
            let buff = building_data.buffs.get(b)?;
            if buff.room_type != "MEETING" {
                return None;
            }
            let value = match registry.get(b) {
                Some(BuffResolutionStrategy::NonProduction { value }) => *value,
                _ => return None,
            };
            let d = buff.description.to_lowercase();
            if (d.contains("no other operator") || d.contains("if no other")) && !alone {
                return None;
            }
            if d.contains("together with") {
                return None;
            }
            Some(value)
        })
        .sum()
}

/// One shift
pub(super) const SHIFT_HOURS: f64 = 12.0;

/// Fraction of a 12h shift a Reception Room operator can work before morale runs out
pub(super) fn reception_sustain(
    op: &OperatorBaseProfile,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    (op_lasts_hours(op, morale_drains, 0.0) / SHIFT_HOURS).min(1.0)
}

pub(super) fn reception_op_value(
    op: &OperatorBaseProfile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
    alone: bool,
) -> f64 {
    let raw = reception_skill(op, registry, building_data, alone)
        + rarity_bonus(op.rarity)
        + elite_bonus(op.elite);
    raw * reception_sustain(op, morale_drains)
}

pub(super) fn best_reception_crew(
    candidates: &[&OperatorBaseProfile],
    cap: usize,
    level: i32,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
) -> Option<(Vec<String>, f64)> {
    if candidates.is_empty() || cap == 0 {
        return None;
    }
    let mut paired: Vec<(String, f64)> = candidates
        .iter()
        .map(|op| {
            (
                op.char_id.clone(),
                reception_op_value(op, registry, building_data, morale_drains, false),
            )
        })
        .collect();
    paired.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let take = cap.min(paired.len());
    let pair_crew: Vec<String> = paired.iter().take(take).map(|(id, _)| id.clone()).collect();
    let pair_val: f64 = paired.iter().take(take).map(|(_, v)| v).sum();

    let best_solo = candidates
        .iter()
        .map(|op| {
            (
                op.char_id.clone(),
                reception_op_value(op, registry, building_data, morale_drains, true),
            )
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let level_bonus = rr_level_bonus(level);
    match best_solo {
        Some((id, val)) if cap >= 2 && val > pair_val => Some((vec![id], val + level_bonus)),
        _ if !pair_crew.is_empty() => Some((pair_crew, pair_val + level_bonus)),
        Some((id, val)) => Some((vec![id], val + level_bonus)),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn assign_auxiliary_rooms(
    rooms: &mut Vec<RoomAssignment>,
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    assigned: &mut HashSet<String>,
    only_24h_sustainable: bool,
) {
    let bar_ok = |op: &OperatorBaseProfile| -> bool {
        !only_24h_sustainable || sustains_24h_block(op, morale_drains)
    };
    for room_type in ["HIRE", "MEETING"] {
        for room in building.rooms.iter().filter(|r| r.room_type == room_type) {
            if rooms.iter().any(|r| r.slot_id == room.slot_id) {
                continue; // already staffed
            }
            let cap = max_stationed_at_level(building_data, room_type, room.level).max(0) as usize;
            if cap == 0 {
                continue;
            }

            let (crew, eff): (Vec<String>, f64) = if room_type == "MEETING" {
                let candidates: Vec<&OperatorBaseProfile> = operators
                    .iter()
                    .filter(|op| !assigned.contains(&op.char_id))
                    .filter(|op| bar_ok(op))
                    .filter(|op| {
                        op.available_buffs.iter().any(|b| {
                            building_data
                                .buffs
                                .get(b)
                                .is_some_and(|bf| bf.room_type == "MEETING")
                        })
                    })
                    .collect();
                match best_reception_crew(
                    &candidates,
                    cap,
                    room.level,
                    registry,
                    building_data,
                    morale_drains,
                ) {
                    Some(x) => x,
                    None => continue,
                }
            } else {
                let mut ranked: Vec<(&str, f64)> = operators
                    .iter()
                    .filter(|op| !assigned.contains(&op.char_id))
                    .filter(|op| bar_ok(op))
                    .map(|op| {
                        (
                            op.char_id.as_str(),
                            aux_room_value(op, room_type, registry, building_data),
                        )
                    })
                    .filter(|(_, v)| *v > 0.0)
                    .collect();
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let crew: Vec<String> = ranked
                    .iter()
                    .take(cap)
                    .map(|(id, _)| (*id).to_string())
                    .collect();
                let eff: f64 = ranked.iter().take(cap).map(|(_, v)| *v).sum();
                (crew, eff)
            };

            if crew.is_empty() {
                continue;
            }
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
                order_gold: 0.0,
                order_limit: None,
                locked: false,
                ledger: Vec::new(),
                fill: None,
            });
        }
    }
}

pub fn support_room_figure(
    room_type: &str,
    seated: &[String],
    working: &[String],
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    from_cc: f64,
) -> f64 {
    let own_values = |op: &OperatorBaseProfile| -> Vec<f64> {
        op.available_buffs
            .iter()
            .filter(|b| {
                building_data
                    .buffs
                    .get(*b)
                    .is_some_and(|buff| buff.room_type == room_type)
            })
            .filter_map(|b| match registry.get(b) {
                Some(
                    BuffResolutionStrategy::NonProduction { value }
                    | BuffResolutionStrategy::DirectEfficiency { value },
                ) => Some(*value),
                _ => None,
            })
            .collect()
    };
    let figure = match room_type {
        "MEETING" => {
            let alone = working.len() == 1;
            let skills: f64 = working
                .iter()
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| reception_skill(op, registry, building_data, alone))
                .sum();
            let ambience: f64 = seated
                .iter()
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| rarity_bonus(op.rarity) + elite_bonus(op.elite))
                .sum();
            skills + ambience + RECEPTION_INNATE_PCT + from_cc
        }
        "HIRE" => {
            working
                .iter()
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| own_values(op).into_iter().fold(0.0, f64::max))
                .fold(0.0, f64::max)
                + from_cc
        }
        _ => {
            working
                .iter()
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| own_values(op).into_iter().sum::<f64>())
                .sum::<f64>()
                + from_cc
        }
    };
    if figure == 0.0 { 0.0 } else { figure }
}

pub(super) fn append_support_rooms(
    rooms: &mut Vec<RoomAssignment>,
    pins: &[(String, String)],
    pinned_ids: &HashSet<String>,
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
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
            let figure = support_room_figure(
                room_type,
                &members,
                &members,
                op_index,
                registry,
                building_data,
                0.0,
            );
            rooms.push(RoomAssignment {
                slot_id: room.slot_id.clone(),
                room_type: room_type.to_string(),
                level: room.level,
                formula_type: None,
                operators: members,
                total_efficiency: figure,
                order_value: 0.0,
                order_gold: 0.0,
                order_limit: None,
                locked: false,
                ledger: Vec::new(),
                fill: None,
            });
        }
    }
}

// Morale / longevity model for staggered rotation.
// Each working operator burns morale and must rest now and then. A STAGGERED rotation swaps only the lowest-morale operator at a time.
//
// Constants are calibrated estimates (drain modifiers come in ~±0.25/hr units):
/// Neutral per-hour morale burn for a working operator before morale skills
pub(super) const BASE_MORALE_DRAIN: f64 = 0.5;
/// Per-hour morale recovery resting in a maxed dormitory
pub(super) const MORALE_RECOVERY: f64 = 1.0;
pub(super) const MAX_DORM_LEVEL: f64 = 5.0;
/// Floor on the recovery factor so a barely-built base still rotates (slowly).
pub(super) const MIN_RECOVERY_FACTOR: f64 = 0.3;
pub(super) const BACKUP_COVERAGE: f64 = 0.8;
