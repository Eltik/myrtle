use myrtle_gamedata_raw::tables::building::{BuildingDataFile, RoomPhase};

#[must_use]
pub fn is_production_room(room_type: &str) -> bool {
    matches!(room_type, "MANUFACTURE" | "TRADING")
}

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

pub fn max_stationed_at_level(
    building_data: &BuildingDataFile,
    room_type: &str,
    level: i32,
) -> i32 {
    room_phase(building_data, room_type, level).map_or(1, |phase| phase.max_stationed_num)
}

/// Buff id without the tier suffix
#[must_use]
pub fn buff_family(buff_id: &str) -> &str {
    buff_id.split('[').next().unwrap_or(buff_id)
}
