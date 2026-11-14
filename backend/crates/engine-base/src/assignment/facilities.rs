use std::collections::{HashMap, HashSet};

use crate::{
    registry::BuffResolutionStrategy,
    types::{OperatorBaseProfile, UserBuilding},
    util::{buff_family, max_stationed_at_level},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

pub(super) fn count_facilities(building: &UserBuilding) -> HashMap<String, usize> {
    let mut facility_counts: HashMap<String, usize> = HashMap::new();
    for room in &building.rooms {
        *facility_counts.entry(room.room_type.clone()).or_insert(0) += 1;
    }
    facility_counts
}

/// The building's facility counts with the enabler bonuses the roster can field
#[must_use]
pub fn effective_facility_counts(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    seats: &HashMap<String, String>,
) -> HashMap<String, usize> {
    let parked = parked_tokens(seats, operators, registry);
    effective_facility_counts_with_inert(
        building,
        operators,
        registry,
        building_data,
        seats,
        &parked,
    )
}

pub fn parked_tokens(
    seats: &HashMap<String, String>,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> HashSet<String> {
    use crate::registry::FacilityGate;
    let is_robot = |id: &str| {
        operators
            .iter()
            .find(|o| o.char_id == id)
            .is_some_and(|o| o.match_tags.iter().any(|t| t == "robot"))
    };
    let modifiers = || {
        operators.iter().flat_map(|op| {
            op.available_buffs
                .iter()
                .filter_map(move |b| match registry.get(b) {
                    Some(BuffResolutionStrategy::FacilityCountModifier {
                        target_room,
                        owner_room,
                        gate,
                        amount,
                    }) if *amount > 0 && seats.get(&op.char_id) == Some(owner_room) => {
                        Some((target_room, gate))
                    }
                    _ => None,
                })
        })
    };
    let exclusion_rooms: HashSet<&String> = modifiers()
        .filter(|(_, gate)| matches!(gate, FacilityGate::NoRobotsInOtherRooms))
        .map(|(room, _)| room)
        .collect();
    modifiers()
        .filter_map(|(_, gate)| match gate {
            FacilityGate::NamedCharInRoom { char_id, room }
                if seats.get(char_id) == Some(room)
                    && exclusion_rooms.contains(room)
                    && is_robot(char_id) =>
            {
                Some(char_id.clone())
            }
            _ => None,
        })
        .collect()
}

#[must_use]
pub fn working_crew(crew: &[String], parked: &HashSet<String>) -> Vec<String> {
    crew.iter()
        .filter(|id| !parked.contains(*id))
        .cloned()
        .collect()
}

pub fn effective_facility_counts_with_inert(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    seats: &HashMap<String, String>,
    inert: &HashSet<String>,
) -> HashMap<String, usize> {
    use crate::registry::FacilityGate;
    let mut counts = count_facilities(building);
    let mut seen_families: HashSet<&str> = HashSet::new();
    let robot_in = |room: &str| {
        seats.iter().any(|(id, rt)| {
            rt == room
                && !inert.contains(id)
                && operators
                    .iter()
                    .find(|o| &o.char_id == id)
                    .is_some_and(|o| o.match_tags.iter().any(|t| t == "robot"))
        })
    };
    for op in operators {
        for buff_id in &op.available_buffs {
            if let Some(BuffResolutionStrategy::FacilityCountModifier {
                target_room,
                amount,
                owner_room,
                gate,
            }) = registry.get(buff_id)
            {
                if seats.get(&op.char_id) != Some(owner_room) {
                    continue;
                }
                let gate_ok = match gate {
                    FacilityGate::None => true,
                    FacilityGate::NamedCharInRoom { char_id, room } => {
                        seats.get(char_id) == Some(room)
                    }
                    FacilityGate::NoRobotsInOtherRooms => !robot_in(target_room),
                };
                let family = buff_family(buff_id);
                if gate_ok && *amount > 0 && seen_families.insert(family) {
                    *counts.entry(target_room.clone()).or_insert(0) += *amount as usize;
                }
            }
        }
    }
    let factory_count = counts.get("MANUFACTURE").copied().unwrap_or(0);
    let distinct_formulas: HashSet<&str> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .filter_map(|r| r.current_formula.as_deref())
        .collect();
    let recipe_types = if distinct_formulas.is_empty() {
        factory_count.min(2)
    } else {
        distinct_formulas.len()
    };
    counts.insert(MANUFACTURE_RECIPE_TYPES.to_string(), recipe_types);
    let gold_lines = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE" && r.current_formula.as_deref() == Some("F_GOLD"))
        .count();
    counts.insert(GOLD_LINES.to_string(), gold_lines);
    counts.insert(
        DRONE_CAPACITY.to_string(),
        drone_capacity(building, building_data),
    );
    if let Some(lv) = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "MEETING")
        .map(|r| r.level.max(0) as usize)
        .max()
    {
        counts.insert(MEETING_LEVEL.to_string(), lv);
    }
    let training_level: usize = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRAINING")
        .map(|r| r.level.max(0) as usize)
        .sum();
    if training_level > 0 {
        counts.insert(TRAINING_LEVEL.to_string(), training_level);
    }
    let trading_min_level = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "TRADING")
        .map(|r| r.level.max(1) as usize)
        .min();
    if let Some(lv) = trading_min_level {
        counts.insert(TRADING_MIN_LEVEL.to_string(), lv);
    }
    let functional_levels: usize = building
        .rooms
        .iter()
        .filter(|r| is_functional_building(building_data, &r.room_type))
        .map(|r| r.level.max(0) as usize)
        .sum();
    counts.insert(FUNCTIONAL_LEVEL_SUM.to_string(), functional_levels);
    counts
}

#[must_use]
pub fn drone_capacity(building: &UserBuilding, building_data: &BuildingDataFile) -> usize {
    if let Some(cap) = building.drone_capacity {
        return usize::try_from(cap).unwrap_or(0);
    }
    let Some(layout) = building_data.layouts.values().next() else {
        return 0;
    };
    let built: i32 = building
        .rooms
        .iter()
        .filter_map(|r| layout.slots.get(&r.slot_id))
        .map(|s| s.provide_labor)
        .sum();
    let total = if built > 0
        || building
            .rooms
            .iter()
            .any(|r| layout.slots.contains_key(&r.slot_id))
    {
        building_data.init_max_labor + built
    } else {
        building_data.init_max_labor + layout.slots.values().map(|s| s.provide_labor).sum::<i32>()
    };
    usize::try_from(total).unwrap_or(0)
}

// Synthetic `facility_counts` keys
pub(super) const DRONE_CAPACITY: &str = "DRONE_CAPACITY";

pub(super) const MANUFACTURE_RECIPE_TYPES: &str = "MANUFACTURE_RECIPE_TYPES";

pub const TRADING_MIN_LEVEL: &str = "TRADING_MIN_LEVEL";
pub const MEETING_LEVEL: &str = "MEETING_LEVEL";
pub const TRAINING_LEVEL: &str = "TRAINING_LEVEL";
pub const GOLD_LINES: &str = "GOLD_LINES";

pub const FUNCTIONAL_LEVEL_SUM: &str = "FUNCTIONAL_LEVEL_SUM";

pub fn is_functional_building(building_data: &BuildingDataFile, room_type: &str) -> bool {
    room_type == "CONTROL"
        || building_data
            .rooms
            .get(room_type)
            .is_some_and(|r| matches!(r.category.as_str(), "OUTPUT" | "CUSTOM" | "FUNCTION"))
}

/// Max stationed for the user's highest-level room of `room_type`
pub fn max_stationed_for_room(
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
