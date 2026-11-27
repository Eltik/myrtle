use std::collections::HashMap;

use myrtle_gamedata_raw::tables::building::BuildingChar;
use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_types::roster::RosterEntry;

#[derive(Clone)]
pub struct UserBuilding {
    /// One entry per built slot
    pub rooms: Vec<UserRoom>,
    /// Synced max Drone capacity, `None` on a draft or fixture
    pub drone_capacity: Option<i32>,
}

#[derive(Clone, Default)]
pub struct UserRoom {
    pub slot_id: String,
    pub room_type: String, // "MANUFACTURE", "TRADING", "POWER", "DORMITORY", etc.
    pub level: i32,        // 1-indexed
    pub current_operators: Vec<String>,
    pub current_formula: Option<String>,
    /// In-game preset queue: each shift a list of `char_id`s
    pub preset_shifts: Vec<Vec<String>>,
    /// Dormitory ambience (0-5000)
    pub comfort: i32,
    /// Out-of-scope room in a scoped planner run
    pub frozen: bool,
}

/// One operator's synced morale
#[derive(Debug, Clone)]
pub struct MoraleSnapshot {
    pub morale: f64,
    pub at_unix: i64,
    /// "" = unstationed
    pub room_slot: String,
}

/// Every operator's morale snapshot from the synced building data, keyed by char id
#[must_use]
pub fn live_morale_snapshot(
    data: &serde_json::Value,
) -> std::collections::HashMap<String, MoraleSnapshot> {
    const AP_PER_POINT: f64 = 360_000.0;
    const MORALE_MAX: f64 = 24.0;
    data.get("chars")
        .and_then(|v| v.as_object())
        .map(|chars| {
            chars
                .values()
                .filter_map(|c| {
                    let char_id = c.get("charId").and_then(|v| v.as_str())?;
                    let ap = c.get("ap").and_then(serde_json::Value::as_f64)?;
                    Some((
                        char_id.to_string(),
                        MoraleSnapshot {
                            morale: (ap / AP_PER_POINT).clamp(0.0, MORALE_MAX),
                            at_unix: c
                                .get("lastApAddTime")
                                .and_then(serde_json::Value::as_i64)
                                .unwrap_or(0),
                            room_slot: c
                                .get("roomSlotId")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default()
                                .to_string(),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn formula_from_id(id: &str) -> Option<String> {
    match id {
        "4" => Some("F_GOLD".to_string()),
        "1" | "2" | "3" => Some("F_EXP".to_string()),
        "13" | "14" => Some("F_DIAMOND".to_string()),
        _ => None,
    }
}

impl UserBuilding {
    pub fn from_json(data: &serde_json::Value) -> Self {
        let inst_to_char: HashMap<i64, String> = data
            .get("chars")
            .and_then(|v| v.as_object())
            .map(|chars| {
                chars
                    .iter()
                    .filter_map(|(inst, c)| {
                        let id = inst.parse::<i64>().ok()?;
                        let char_id = c.get("charId").and_then(|v| v.as_str())?;
                        Some((id, char_id.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let formula_by_slot: HashMap<String, String> = data
            .get("rooms")
            .and_then(|v| v.get("MANUFACTURE"))
            .and_then(|v| v.as_object())
            .map(|slots| {
                slots
                    .iter()
                    .filter_map(|(slot_id, room)| {
                        let id = room.get("formulaId").and_then(|v| v.as_str())?;
                        Some((slot_id.clone(), formula_from_id(id)?))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let comfort_by_slot: HashMap<String, i32> = data
            .get("rooms")
            .and_then(|v| v.get("DORMITORY"))
            .and_then(|v| v.as_object())
            .map(|slots| {
                slots
                    .iter()
                    .filter_map(|(slot_id, room)| {
                        let c = room.get("comfort").and_then(serde_json::Value::as_i64)?;
                        #[allow(clippy::cast_possible_truncation)]
                        Some((slot_id.clone(), c as i32))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut presets_by_slot: HashMap<String, Vec<Vec<String>>> = HashMap::new();
        if let Some(rooms_obj) = data.get("rooms").and_then(|v| v.as_object()) {
            for room_type_slots in rooms_obj.values() {
                let Some(slots) = room_type_slots.as_object() else {
                    continue;
                };
                for (slot_id, room) in slots {
                    let Some(queue) = room.get("presetQueue").and_then(|v| v.as_array()) else {
                        continue;
                    };
                    let shifts: Vec<Vec<String>> = queue
                        .iter()
                        .map(|shift| {
                            shift
                                .as_array()
                                .map(|ids| {
                                    ids.iter()
                                        .filter_map(serde_json::Value::as_i64)
                                        .filter_map(|id| inst_to_char.get(&id).cloned())
                                        .collect()
                                })
                                .unwrap_or_default()
                        })
                        .collect();
                    if !shifts.is_empty() {
                        presets_by_slot.insert(slot_id.clone(), shifts);
                    }
                }
            }
        }

        let mut rooms = Vec::new();
        if let Some(slots) = data.get("roomSlots").and_then(|v| v.as_object()) {
            for (slot_id, slot) in slots {
                let room_type = slot
                    .get("roomId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let level = slot
                    .get("level")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0) as i32;
                let state = slot
                    .get("state")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0);
                if state > 0 && level > 0 {
                    let current_operators = slot
                        .get("charInstIds")
                        .and_then(|v| v.as_array())
                        .map(|ids| {
                            ids.iter()
                                .filter_map(serde_json::Value::as_i64)
                                .filter_map(|id| inst_to_char.get(&id).cloned())
                                .collect()
                        })
                        .unwrap_or_default();
                    rooms.push(UserRoom {
                        slot_id: slot_id.clone(),
                        room_type: room_type.to_string(),
                        level,
                        current_operators,
                        current_formula: formula_by_slot.get(slot_id).cloned(),
                        preset_shifts: presets_by_slot.get(slot_id).cloned().unwrap_or_default(),
                        comfort: comfort_by_slot.get(slot_id).copied().unwrap_or(0),
                        frozen: false,
                    });
                }
            }
        }
        let drone_capacity = data
            .get("status")
            .and_then(|s| s.get("labor"))
            .and_then(|l| l.get("maxValue"))
            .and_then(serde_json::Value::as_i64)
            .and_then(|v| i32::try_from(v).ok());
        Self {
            rooms,
            drone_capacity,
        }
    }

    #[must_use]
    pub fn total_dorm_levels(&self) -> i32 {
        self.rooms
            .iter()
            .filter(|r| r.room_type == "DORMITORY")
            .map(|r| r.level)
            .sum()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

#[derive(Clone)]
pub struct OperatorBaseProfile {
    pub char_id: String,
    /// `buff_ids` unlocked by the operator's elite/level
    pub available_buffs: Vec<String>,
    /// Lowercased group/nation/team ids
    pub faction_tags: Vec<String>,
    /// Faction tags plus the leading word of each skill name
    pub match_tags: Vec<String>,
    pub rarity: i16,
    pub elite: i16,
}

impl OperatorBaseProfile {
    /// `ignore_promotion` takes each slot's top tier regardless of elite/level
    #[must_use]
    pub fn build(
        roster: &RosterEntry,
        building_char: &BuildingChar,
        faction_tags: Vec<String>,
        rarity: i16,
        building_data: &BuildingDataFile,
        ignore_promotion: bool,
    ) -> Self {
        let mut available_buffs = Vec::new();

        for slot in &building_char.buff_char {
            let best = slot.buff_data.iter().rfind(|entry| {
                ignore_promotion
                    || (i32::from(roster.elite) >= entry.cond.elite()
                        && i32::from(roster.level) >= entry.cond.level)
            });
            if let Some(entry) = best {
                available_buffs.push(entry.buff_id.clone());
            }
        }

        let match_tags = compute_match_tags(&faction_tags, &available_buffs, building_data);
        Self {
            char_id: building_char.char_id.clone(),
            available_buffs,
            faction_tags,
            match_tags,
            rarity,
            elite: roster.elite,
        }
    }
}

#[must_use]
pub fn compute_match_tags(
    faction_tags: &[String],
    available_buffs: &[String],
    building_data: &BuildingDataFile,
) -> Vec<String> {
    let mut tags = faction_tags.to_vec();
    for buff_id in available_buffs {
        if let Some(buff) = building_data.buffs.get(buff_id)
            && let Some(word) = buff.buff_name.split([' ', '-']).next()
        {
            let token = word.to_lowercase();
            if !token.is_empty() && !tags.contains(&token) {
                tags.push(token);
            }
        }
    }
    tags
}

#[derive(Clone, Default)]
pub struct RoomAssignment {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    pub operators: Vec<String>,
    /// Order-acquisition SPEED %
    pub total_efficiency: f64,
    /// Order-VALUE % (LMD per order)
    pub order_value: f64,
    /// Gold-throughput part of that value
    pub order_gold: f64,
    pub order_limit: Option<i32>,
    /// Fixed synergy squad whose operators depend on each other
    pub locked: bool,
    pub ledger: Vec<crate::scoring::skill_ledger::LedgerLine>,
    pub fill: Option<crate::scoring::yield_model::RoomFill>,
}

#[derive(Clone)]
pub struct BaseAssignment {
    pub rooms: Vec<RoomAssignment>,
    pub total_production_efficiency: f64,
    /// Operators on SPARE seats
    pub bench: Vec<String>,
}

/// STAGGERED rotation: only the lowest-morale operator in a room swaps for its backup
pub struct RotationAssignment {
    pub main: BaseAssignment,
    pub rooms: Vec<RoomRotation>,
    pub shared_bench: Vec<String>,
    pub sets: Vec<RotationSet>,
    pub sustained_efficiency: f64,
}

/// One full staffing where each room has at most one main resting
pub struct RotationSet {
    pub rooms: Vec<RotationSetRoom>,
}

pub struct RotationSetRoom {
    pub slot_id: String,
    pub room_type: String,
    pub working: Vec<String>,
    pub resting: Option<String>,
}

/// The rotation plan for one production room
pub struct RoomRotation {
    pub slot_id: String,
    pub room_type: String,
    /// Ordered by who swaps FIRST
    pub members: Vec<RotationMember>,
    pub backup: Option<String>,
}

pub struct RotationMember {
    pub operator: String,
    pub lasts_hours: f64,
}
