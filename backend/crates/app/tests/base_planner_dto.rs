#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use myrtle_app::base_planner::{ShiftRotationDto, shift_rotation_to_dto};
use myrtle_engine_base::BaseData;
use myrtle_engine_base::rotation::shift::{Shift, ShiftRoom, ShiftRotation};
use myrtle_engine_base::types::UserBuilding;
use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_gamedata_raw::tables::consts::GameDataConst;
use serde_json::json;

fn room(slot: &str, rec: &[&str], cur: &[&str]) -> ShiftRoom {
    ShiftRoom {
        slot_id: slot.into(),
        room_type: "TRADING".into(),
        formula_type: None,
        recommended: rec.iter().map(|s| (*s).to_owned()).collect(),
        current: cur.iter().map(|s| (*s).to_owned()).collect(),
        active: true,
        efficiency: None,
        team_id: None,
        team_label: None,
    }
}

fn to_dto(rooms: Vec<ShiftRoom>) -> ShiftRotationDto {
    let rotation = ShiftRotation {
        shifts: vec![Shift { index: 1, rooms }],
        sustained: Vec::new(),
        bench: Vec::new(),
        parked: Vec::new(),
    };
    let data = BaseData::new(
        HashMap::new(),
        BuildingDataFile::default(),
        GameDataConst::default(),
    );
    shift_rotation_to_dto(
        &rotation,
        &data,
        &[],
        &UserBuilding::from_json(&json!({})),
        &HashMap::new(),
        &HashMap::new(),
    )
}

fn current(dto: &ShiftRotationDto, slot: &str) -> Vec<String> {
    let room = dto.shifts[0]
        .rooms
        .iter()
        .find(|r| r.slot_id == slot)
        .unwrap();
    let mut ids: Vec<String> = room.current.iter().map(|o| o.operator_id.clone()).collect();
    ids.sort();
    ids
}

#[test]
fn matching_is_order_independent_across_same_type_rooms() {
    let dto = to_dto(vec![
        room("a", &["A", "B", "C"], &["D", "E", "F"]),
        room("b", &["D", "E", "F"], &["A", "B", "C"]),
    ]);
    assert_eq!(
        current(&dto, "a"),
        ["A", "B", "C"],
        "wherever the player keeps it"
    );
    assert_eq!(current(&dto, "b"), ["D", "E", "F"]);
}

#[test]
fn matching_picks_the_closest_team_to_minimise_swaps() {
    let dto = to_dto(vec![
        room("a", &["X", "Y", "Z"], &["P", "Q", "R"]),
        room("b", &["M", "N", "O"], &["X", "Y", "Q"]),
    ]);
    assert_eq!(
        current(&dto, "a"),
        ["Q", "X", "Y"],
        "one swap from the near team, not three from the one in the same slot"
    );
    assert_eq!(current(&dto, "b"), ["P", "Q", "R"]);
}
