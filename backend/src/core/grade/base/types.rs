use std::collections::HashMap;

use crate::core::gamedata::types::building::BuildingDataFile;
use crate::{core::gamedata::types::building::BuildingChar, database::models::roster::RosterEntry};

#[derive(Clone)]
pub struct UserBuilding {
    /// One entry per built slot (`roomSlots` with state > 0 and level > 0).
    pub rooms: Vec<UserRoom>,
}

#[derive(Clone, Default)]
pub struct UserRoom {
    pub slot_id: String,
    pub room_type: String, // "MANUFACTURE", "TRADING", "POWER", "DORMITORY", etc.
    pub level: i32,        // 1-indexed (1, 2, 3)
    /// `char_id`s stationed now.
    pub current_operators: Vec<String>,
    /// Factory formula (`F_GOLD`, `F_EXP`, `F_DIAMOND`); `None` for non-factories.
    pub current_formula: Option<String>,
    /// In-game preset queue: each shift a list of `char_id`s. Empty if unset.
    pub preset_shifts: Vec<Vec<String>>,
    /// Dormitory ambience (0-5000) from the synced furniture, 0 elsewhere.
    pub comfort: i32,
    /// Out-of-scope room in a scoped planner run: the search leaves crew and recipe
    /// as drafted, scores it as drafted, and treats its operators as taken.
    pub frozen: bool,
}

/// One operator's synced morale: the bar as last written, when, and where.
#[derive(Debug, Clone)]
pub struct MoraleSnapshot {
    /// Morale points (0-24) at `at_unix`.
    pub morale: f64,
    /// Unix seconds of the game's last morale write for this operator.
    pub at_unix: i64,
    /// The room they occupied ("" = unstationed; morale is frozen there).
    pub room_slot: String,
}

/// Every operator's morale snapshot from the synced building data, keyed by
/// char id: `chars[].ap` counts 360,000 per morale point (24.0 = 8,640,000),
/// stamped by `lastApAddTime`. Project it forward with
/// [`super::sustain_sim::project_morale`] before showing it as "now".
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

fn formula_from_id(id: &str) -> Option<String> {
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

        // slot_id -> factory formula (`rooms.MANUFACTURE[slot].formulaId`).
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

        // slot_id -> dorm ambience (`rooms.DORMITORY[slot].comfort`).
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

        // slot_id -> preset shifts (`rooms.<type>[slot].presetQueue`, instIds -> char_ids).
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
        Self { rooms }
    }

    /// Sum of dorm levels, for &dorm&lv scaling.
    pub fn total_dorm_levels(&self) -> i32 {
        self.rooms
            .iter()
            .filter(|r| r.room_type == "DORMITORY")
            .map(|r| r.level)
            .sum()
    }

    pub const fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

#[derive(Clone)]
pub struct OperatorBaseProfile {
    pub char_id: String,
    /// `buff_ids` unlocked by the operator's elite/level (`BuildingChar.buff_char` Cond).
    pub available_buffs: Vec<String>,
    /// Lowercased group/nation/team ids, for match-count synergies like Dorothy's
    /// "+5% per Rhine Tech skill" or Morgan's "+20% per Glasgow Gang operator".
    pub faction_tags: Vec<String>,
    /// Faction tags plus the leading word of each skill name ("rhine",
    /// "standardization"). Precomputed to keep it out of the hot loop.
    pub match_tags: Vec<String>,
    /// 1-6. Reception Room ambience bonus scales with it (6★ > 5★ > 4★).
    pub rarity: i16,
    /// 0/1/2. Reception Room elite bonus (E2 > E1 > E0).
    pub elite: i16,
}

impl OperatorBaseProfile {
    /// `ignore_promotion` takes each slot's top tier regardless of elite/level ("at
    /// E2 max"). Anything planned this way is a target, not their current base.
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

/// Faction tags plus each skill name's leading word: what a `MatchCountScaling`
/// buff keys on.
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
    // Curated base tags ("all Knight Operators") arrive with the faction tags, from
    // the term glossary (`buff_registry::glossary_tags`).
    tags
}

#[derive(Clone, Default)]
pub struct RoomAssignment {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    pub operators: Vec<String>,
    /// Order-acquisition SPEED %, i.e. the productivity bonus the game shows.
    pub total_efficiency: f64,
    /// Order-VALUE % (LMD per order, e.g. Proviso). Multiplies LMD yield; separate
    /// from speed so it doesn't inflate the displayed efficiency.
    pub order_value: f64,
    /// Gold-throughput part of that value (Pure Gold/h over a bare post): Proviso's
    /// bonus bars come from stock, Tequila's LMD rider moves none. The yield model
    /// bounds the former by the factories' gold.
    pub order_gold: f64,
    /// Post's final order limit after every skill (`None` elsewhere); the buffer the
    /// yield model prices.
    pub order_limit: Option<i32>,
    /// Fixed synergy squad whose operators depend on each other (Shamare + Tequila +
    /// Bibeak, Texas + Lappland). False = interchangeable independent operators.
    pub locked: bool,
    /// Per-skill breakdown for the deep dive. Evaluate path only; display, not score.
    pub ledger: Vec<super::skill_ledger::LedgerLine>,
    /// Output-buffer size and fill time (deep dive). Evaluate path only.
    pub fill: Option<super::yield_model::RoomFill>,
}

#[derive(Clone)]
pub struct BaseAssignment {
    pub rooms: Vec<RoomAssignment>,
    pub total_production_efficiency: f64,
    /// Operators on SPARE seats (Control Center top-up after the value picks): chosen
    /// for low opportunity cost, not skills. The UI badges them so a gated skill on a
    /// benchwarmer doesn't read as the optimizer's reasoning.
    pub bench: Vec<String>,
}

/// STAGGERED rotation: the best operators staff the base (`main`) and only the
/// lowest-morale operator in a room swaps for its backup, so the base runs near peak
/// with few operators in the dorms at once. `shared_bench` covers ALL rooms: one
/// swap at a time means one versatile filler backs several rooms.
/// `sustained_efficiency` is 24/7 output: `main`'s peak minus backup cover time.
/// `sets` are overlapping staffings to cycle through instead of swapping wholesale.
pub struct RotationAssignment {
    pub main: BaseAssignment,
    pub rooms: Vec<RoomRotation>,
    pub shared_bench: Vec<String>,
    pub sets: Vec<RotationSet>,
    pub sustained_efficiency: f64,
}

/// One full staffing where each room has at most one main resting, covered by a
/// backup. Consecutive sets share all but one operator per room. E.g. a 2-seat post
/// over {Proviso, Gravel, Spot}: Proviso+Gravel -> Proviso+Spot -> Gravel+Spot.
pub struct RotationSet {
    pub rooms: Vec<RotationSetRoom>,
}

pub struct RotationSetRoom {
    pub slot_id: String,
    pub room_type: String,
    /// Resting main swapped out for the backup.
    pub working: Vec<String>,
    /// The main resting this set (whom `working` covers via the backup), if any.
    pub resting: Option<String>,
}

/// The rotation plan for one production room: who to swap first and when.
pub struct RoomRotation {
    pub slot_id: String,
    pub room_type: String,
    /// Ordered by who swaps FIRST (fastest drain hits low morale soonest).
    pub members: Vec<RotationMember>,
    pub backup: Option<String>,
}

/// A main operator in a room's rotation, with how long it works before a swap.
pub struct RotationMember {
    pub operator: String,
    /// Approximate hours before morale runs low and it swaps out.
    pub lasts_hours: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> serde_json::Value {
        serde_json::json!({
            "chars": {
                "1": { "charId": "char_002_amiya" },
                "2": { "charId": "char_017_huang" },
                "3": { "charId": "char_103_angel" },
                "x": { "charId": "char_bad_inst" },
                "4": { "noCharId": true },
            },
            "roomSlots": {
                "slot_1": { "roomId": "MANUFACTURE", "level": 3, "state": 2, "charInstIds": [1, 2, -1] },
                "slot_2": { "roomId": "TRADING", "level": 2, "state": 2, "charInstIds": [3] },
                "slot_3": { "roomId": "DORMITORY", "level": 4, "state": 2, "charInstIds": [] },
                "slot_4": { "roomId": "DORMITORY", "level": 1, "state": 1 },
                // Unbuilt (state 0) or level 0: not a room yet.
                "slot_5": { "roomId": "POWER", "level": 1, "state": 0 },
                "slot_6": { "roomId": "POWER", "level": 0, "state": 2 },
            },
            "rooms": {
                "MANUFACTURE": { "slot_1": { "formulaId": "4", "presetQueue": [[1, 2], [3, -1], [-1, -1]] } },
                "TRADING": { "slot_2": { "presetQueue": [] } },
                "DORMITORY": { "slot_3": { "comfort": 5000 }, "slot_4": { "comfort": 1200 } },
            },
        })
    }

    fn by_slot(b: &UserBuilding) -> Vec<&UserRoom> {
        let mut rooms: Vec<&UserRoom> = b.rooms.iter().collect();
        rooms.sort_by(|a, b| a.slot_id.cmp(&b.slot_id));
        rooms
    }

    #[test]
    fn keeps_only_built_slots() {
        let b = UserBuilding::from_json(&sample());
        let slots: Vec<&str> = by_slot(&b).iter().map(|r| r.slot_id.as_str()).collect();
        assert_eq!(slots, vec!["slot_1", "slot_2", "slot_3", "slot_4"]);
        assert!(!b.is_empty());
    }

    #[test]
    fn maps_instance_ids_to_char_ids_and_drops_unknown_seats() {
        let b = UserBuilding::from_json(&sample());
        let rooms = by_slot(&b);
        assert_eq!(rooms[0].room_type, "MANUFACTURE");
        assert_eq!(rooms[0].level, 3);
        assert_eq!(
            rooms[0].current_operators,
            vec!["char_002_amiya", "char_017_huang"]
        );
        assert_eq!(rooms[1].current_operators, vec!["char_103_angel"]);
        assert!(rooms[3].current_operators.is_empty());
    }

    #[test]
    fn reads_formula_comfort_and_presets_per_slot() {
        let b = UserBuilding::from_json(&sample());
        let rooms = by_slot(&b);
        assert_eq!(rooms[0].current_formula.as_deref(), Some("F_GOLD"));
        assert_eq!(rooms[1].current_formula, None);
        // An unset seat (`-1`) maps to nothing, so a dark third shift is empty.
        assert_eq!(
            rooms[0].preset_shifts,
            vec![
                vec!["char_002_amiya".to_string(), "char_017_huang".to_string()],
                vec!["char_103_angel".to_string()],
                Vec::<String>::new(),
            ]
        );
        assert!(rooms[1].preset_shifts.is_empty());
        assert_eq!(rooms[2].comfort, 5000);
        assert_eq!(rooms[3].comfort, 1200);
        assert_eq!(rooms[0].comfort, 0);
        assert!(rooms.iter().all(|r| !r.frozen));
    }

    #[test]
    fn total_dorm_levels_sums_dormitories_only() {
        let b = UserBuilding::from_json(&sample());
        assert_eq!(b.total_dorm_levels(), 5);
    }

    #[test]
    fn empty_or_malformed_json_is_an_empty_building() {
        assert!(UserBuilding::from_json(&serde_json::json!({})).is_empty());
        assert!(UserBuilding::from_json(&serde_json::json!(null)).is_empty());
        assert!(UserBuilding::from_json(&serde_json::json!({ "roomSlots": [] })).is_empty());
        assert_eq!(
            UserBuilding::from_json(&serde_json::json!({})).total_dorm_levels(),
            0
        );
    }

    #[test]
    fn formula_ids_map_to_recipes() {
        assert_eq!(formula_from_id("4").as_deref(), Some("F_GOLD"));
        for exp in ["1", "2", "3"] {
            assert_eq!(formula_from_id(exp).as_deref(), Some("F_EXP"), "{exp}");
        }
        assert_eq!(formula_from_id("13").as_deref(), Some("F_DIAMOND"));
        assert_eq!(formula_from_id("14").as_deref(), Some("F_DIAMOND"));
        assert_eq!(formula_from_id("5"), None);
        assert_eq!(formula_from_id(""), None);
    }
}
