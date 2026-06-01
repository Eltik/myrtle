use std::collections::HashMap;

use crate::{core::gamedata::types::building::BuildingChar, database::models::roster::RosterEntry};

pub struct UserBuilding {
    /// How many factories, trading posts, power plants, and their levels.
    /// Key = `slot_id` (e.g. "`slot_1`"), Value = room info.
    pub rooms: Vec<UserRoom>,
}

#[derive(Default)]
pub struct UserRoom {
    pub slot_id: String,
    pub room_type: String, // "MANUFACTURE", "TRADING", "POWER", "DORMITORY", etc.
    pub level: i32,        // 1-indexed (1, 2, 3)
    /// `char_id`s currently stationed in this room (the player's live setup).
    pub current_operators: Vec<String>,
    /// Current production formula for factories ("`F_GOLD"/"F_EXP"/"F_DIAMOND`"),
    /// parsed from the live building data. None for non-factory rooms.
    pub current_formula: Option<String>,
    /// The player's planned rotation shifts for this room (each a list of
    /// `char_id`s), from the in-game preset queue. Empty if none set.
    pub preset_shifts: Vec<Vec<String>>,
}

/// Map a manufacture `formulaId` to its production formula type.
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
        // instId -> char_id (which operator each stationed instance is).
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

        // slot_id -> current factory formula, from `rooms.MANUFACTURE[slot].formulaId`.
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

        // slot_id -> planned rotation shifts, from `rooms.<type>[slot].presetQueue`
        // (a list of shifts, each a list of stationed instIds → char_ids).
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
    pub const fn is_empty(&self) -> bool {
        self.rooms.is_empty()
    }
}

pub struct OperatorBaseProfile {
    pub char_id: String,
    /// Which `buff_ids` this operator has unlocked (based on their elite/level
    /// meeting the Cond requirements from `BuildingChar.buff_char`)
    pub available_buffs: Vec<String>,
    /// Lowercased faction identifiers (group/nation/team id), used by
    /// match-count synergies like Dorothy's "+5% per Rhine Tech skill" or
    /// Morgan's "+20% per Glasgow Gang operator".
    pub faction_tags: Vec<String>,
    /// Precomputed match tags for count-scaling synergies: faction tags plus the
    /// leading word of each of this operator's skill names (e.g. "rhine",
    /// "standardization"). Computed once here rather than in the hot inner loop.
    pub match_tags: Vec<String>,
}

impl OperatorBaseProfile {
    pub fn build(
        roster: &RosterEntry,
        building_char: &BuildingChar,
        faction_tags: Vec<String>,
        building_data: &crate::core::gamedata::types::building::BuildingDataFile,
    ) -> Self {
        let mut available_buffs = Vec::new();

        for slot in &building_char.buff_char {
            let mut best: Option<&str> = None;
            for entry in &slot.buff_data {
                if i32::from(roster.elite) >= entry.cond.elite()
                    && i32::from(roster.level) >= entry.cond.level
                {
                    best = Some(&entry.buff_id);
                }
            }
            if let Some(buff_id) = best {
                available_buffs.push(buff_id.to_string());
            }
        }

        let match_tags = compute_match_tags(&faction_tags, &available_buffs, building_data);
        Self {
            char_id: building_char.char_id.clone(),
            available_buffs,
            faction_tags,
            match_tags,
        }
    }
}

/// Faction tags + the leading word of each of the operator's skill names - the
/// set a `MatchCountScaling` buff keys on. Computed once per operator.
pub fn compute_match_tags(
    faction_tags: &[String],
    available_buffs: &[String],
    building_data: &crate::core::gamedata::types::building::BuildingDataFile,
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
    // Base-skill tag aliases: some Control Center buffs target a curated base tag
    // ("all Knight Operators") rather than a single faction id. Pinus Sylvestris
    // is literally the "Knightclub", so its members count as Knights (e.g. Wild
    // Mane benefits from Viviana's "+7% Knights in Factories").
    for (faction, alias) in FACTION_BASE_TAG_ALIASES {
        if tags.iter().any(|t| t == faction) && !tags.iter().any(|t| t == alias) {
            tags.push((*alias).to_string());
        }
    }
    tags
}

/// Maps a faction id to the curated base tag its members carry, for Control
/// Center buffs that target base tags rather than factions.
const FACTION_BASE_TAG_ALIASES: &[(&str, &str)] = &[("pinus", "knight")];

pub struct EvalContext<'a> {
    /// How many of each room type exist in the base
    pub facility_counts: &'a HashMap<String, usize>,
    /// Total dormitory levels (for &dorm&lv scaling)
    pub total_dorm_levels: i32,
    /// Other operators in the same room (borrowed to avoid per-evaluation clones).
    pub room_teammates: Vec<&'a TeammateInfo>,
}

#[derive(Clone)]
pub struct TeammateInfo {
    /// `char_id` of the teammate - used to resolve named-teammate conditional
    /// buffs (e.g. "+65% only when Lappland is in the same room").
    pub char_id: String,
    /// Match tags for count-scaling synergies: faction ids + leading words of
    /// this operator's skill names (e.g. "rhine", "standardization"). May be
    /// augmented at evaluation time by a skill-type converter in the room.
    pub match_tags: Vec<String>,
    pub buff_ids: Vec<String>,
    /// Sum of `DirectEfficiency` values from this teammate's buffs
    pub direct_efficiency: f64,
    pub order_limit_contribution: i32,
}

#[derive(Clone, Default)]
pub struct RoomAssignment {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub formula_type: Option<String>,
    pub operators: Vec<String>, // char_ids assigned to this room
    /// Order-acquisition SPEED %, i.e. the productivity bonus the game shows.
    pub total_efficiency: f64,
    /// Order-VALUE % (LMD per order, e.g. Proviso) - multiplies LMD yield, kept
    /// separate from speed so it doesn't inflate the displayed efficiency.
    pub order_value: f64,
    /// True when this is a FIXED synergy squad - its operators depend on each
    /// other (e.g. Shamare + Tequila + Bibeak, or Texas + Lappland) and can't be
    /// swapped without breaking the combo. False = a flexible team of independent
    /// operators that are interchangeable with similar ones.
    pub locked: bool,
}

#[derive(Clone)]
pub struct BaseAssignment {
    pub rooms: Vec<RoomAssignment>,
    pub total_production_efficiency: f64, // sum across all production rooms
}

/// A STAGGERED rotation: your best operators staff the base (the `main`), and you
/// swap only the lowest-morale operator in a room for its backup, so the base runs
/// at near-peak almost always and few operators sit in the dorms at once.
/// `rooms` gives the per-room rotation plan (swap order + timing + backup).
/// `shared_bench` is the small pool of fillers that covers ALL rooms: because only
/// one operator is swapped at a time, a versatile filler backs up several rooms.
/// `sustained_efficiency` is the 24/7 output: close to `main`'s peak, reduced only
/// by the time a backup covers a resting operator (low-drain teams rest less).
/// `sets` expresses the rotation as a handful of overlapping staffings to cycle
/// through (rather than swapping the whole base at once).
pub struct RotationAssignment {
    pub main: BaseAssignment,
    pub rooms: Vec<RoomRotation>,
    pub shared_bench: Vec<String>,
    pub sets: Vec<RotationSet>,
    pub sustained_efficiency: f64,
}

/// One snapshot of the staggered rotation: a complete base staffing in which each
/// room has (at most) one main resting, covered by a backup. Cycling through the
/// sets every swap interval rests every operator in turn, and CONSECUTIVE sets share
/// all-but-one operator per room - so you never break the whole base at once. E.g.
/// a 2-seat post over {Proviso, Gravel, Spot} cycles Proviso+Gravel -> Proviso+Spot
/// -> Gravel+Spot.
pub struct RotationSet {
    pub rooms: Vec<RotationSetRoom>,
}

/// One room's staffing within a rotation set.
pub struct RotationSetRoom {
    pub slot_id: String,
    pub room_type: String,
    /// The operators working this room in this set (the resting main swapped out for
    /// the backup).
    pub working: Vec<String>,
    /// The main resting this set (whom `working` covers via the backup), if any.
    pub resting: Option<String>,
}

/// The rotation plan for one production room: who to swap first and when.
pub struct RoomRotation {
    pub slot_id: String,
    pub room_type: String,
    /// The room's main operators, ordered by who needs swapping FIRST (the
    /// fastest-draining operator hits low morale soonest).
    pub members: Vec<RotationMember>,
    /// The backup operator to rotate in (`char_id`), if one is available.
    pub backup: Option<String>,
}

/// A main operator in a room's rotation, with how long it works before a swap.
pub struct RotationMember {
    pub operator: String,
    /// Approximate hours this operator works before its morale runs low and you
    /// rotate it out (fast-draining skills last fewer hours).
    pub lasts_hours: f64,
}
