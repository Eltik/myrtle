//! Small shared helpers for the base optimizer: room classification and
//! building-data lookups used across assignment and scoring.

use crate::core::gamedata::types::building::{BuildingDataFile, RoomPhase};

/// Trading posts and factories - the yield-producing rooms the optimizer staffs.
pub fn is_production_room(room_type: &str) -> bool {
    matches!(room_type, "MANUFACTURE" | "TRADING")
}

/// The building-data phase (per-level stats) for a room of `room_type` at `level`
/// (1-indexed), or `None` when the room type or level is unknown.
pub fn room_phase<'a>(
    building_data: &'a BuildingDataFile,
    room_type: &str,
    level: i32,
) -> Option<&'a RoomPhase> {
    building_data
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1).max(0) as usize))
}

/// Max operators stationable in a room of `room_type` at `level`, falling back to
/// 1 when the room type or level is unknown.
pub fn max_stationed_at_level(
    building_data: &BuildingDataFile,
    room_type: &str,
    level: i32,
) -> i32 {
    room_phase(building_data, room_type, level).map_or(1, |phase| phase.max_stationed_num)
}
