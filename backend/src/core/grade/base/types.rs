use std::collections::HashMap;

use crate::{core::gamedata::types::building::BuildingChar, database::models::roster::RosterEntry};

pub struct UserBuilding {
    /// How many factories, trading posts, power plants, and their levels.
    /// Key = slot_id (e.g. "slot_1"), Value = room info.
    pub rooms: Vec<UserRoom>,
}

pub struct UserRoom {
    pub slot_id: String,
    pub room_type: String, // "MANUFACTURE", "TRADING", "POWER", "DORMITORY", etc.
    pub level: i32,        // 1-indexed (1, 2, 3)
}

impl UserBuilding {
    pub fn from_json(data: &serde_json::Value) -> Self {
        let mut rooms = Vec::new();
        if let Some(slots) = data.get("roomSlots").and_then(|v| v.as_object()) {
            for (slot_id, slot) in slots {
                let room_type = slot
                    .get("roomId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let level = slot.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let state = slot.get("state").and_then(|v| v.as_i64()).unwrap_or(0);
                if state > 0 && level > 0 {
                    rooms.push(UserRoom {
                        slot_id: slot_id.clone(),
                        room_type: room_type.to_string(),
                        level,
                    });
                }
            }
        }
        Self { rooms }
    }

    /// Total dormitory levels (sum of all dorm levels, for &dorm&lv scaling)
    pub fn total_dorm_levels(&self) -> i32 {
        self.rooms
            .iter()
            .filter(|r| r.room_type == "DORMITORY")
            .map(|r| r.level)
            .sum()
    }

    /// Is the building data present/non-empty?
    pub fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

pub struct OperatorBaseProfile {
    pub char_id: String,
    /// Which buff_ids this operator has unlocked (based on their elite/level
    /// meeting the Cond requirements from BuildingChar.buff_char)
    pub available_buffs: Vec<String>,
}

impl OperatorBaseProfile {
    pub fn build(roster: &RosterEntry, building_char: &BuildingChar) -> Self {
        let mut available_buffs = Vec::new();

        for slot in &building_char.buff_char {
            let mut best: Option<&str> = None;
            for entry in &slot.buff_data {
                if roster.elite as i32 >= entry.cond.elite()
                    && roster.level as i32 >= entry.cond.level
                {
                    best = Some(&entry.buff_id);
                }
            }
            if let Some(buff_id) = best {
                available_buffs.push(buff_id.to_string());
            }
        }

        Self {
            char_id: building_char.char_id.clone(),
            available_buffs,
        }
    }
}

pub struct EvalContext<'a> {
    /// How many of each room type exist in the base
    pub facility_counts: &'a HashMap<String, usize>,
    /// Total dormitory levels (for &dorm&lv scaling)
    pub total_dorm_levels: i32,
    /// Other operators in the same room and their resolved direct-efficiency values
    pub room_teammates: Vec<TeammateInfo>,
}

#[derive(Clone)]
pub struct TeammateInfo {
    pub buff_ids: Vec<String>,
    /// Sum of DirectEfficiency values from this teammate's buffs
    pub direct_efficiency: f64,
    pub order_limit_contribution: i32,
}

pub struct RoomAssignment {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    pub operators: Vec<String>, // char_ids assigned to this room
    pub total_efficiency: f64,  // computed total bonus for this room
}

pub struct BaseAssignment {
    pub rooms: Vec<RoomAssignment>,
    pub total_production_efficiency: f64, // sum across all production rooms
}

pub struct ShiftAssignment {
    pub shift_a: BaseAssignment,
    pub shift_b: BaseAssignment,
    pub sustained_efficiency: f64, // average of both shifts
}
