use crate::score::*;
use crate::types::UserBuilding;
use crate::types::UserRoom;
use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_gamedata_raw::tables::building::RoomDef;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

fn room_def(id: &str, phase_count: usize) -> RoomDef {
    let phases: Vec<serde_json::Value> = (0..phase_count)
        .map(|_| {
            serde_json::json!({
                "UnlockCondId": "",
                "MaxStationedNum": 1,
                "Electricity": 0,
                "ManpowerCost": 0,
                "BuildCost": { "Labor": 0, "Time": 0 },
            })
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "Id": id,
        "Name": id,
        "Category": "FUNCTION",
        "MaxCount": 1,
        "CanLevelDown": true,
        "DefaultPrefabId": "",
        "Size": { "Col": 1, "Row": 1 },
        "Phases": phases,
    }))
    .expect("room def")
}

fn room(slot: &str, room_type: &str, level: i32) -> UserRoom {
    UserRoom {
        slot_id: slot.into(),
        room_type: room_type.into(),
        level,
        current_operators: vec!["char_002_amiya".into()],
        ..UserRoom::default()
    }
}

#[test]
fn log_curve_maps_the_unit_interval_onto_itself() {
    assert!(close(log_curve_ratio(0.0), 0.0));
    assert!(close(log_curve_ratio(1.0), 1.0));
    assert!(close(log_curve_ratio(0.5), 0.584_962_500_721_156_2));
    assert!(log_curve_ratio(0.25) > 0.25);
}

#[test]
fn blend_weights_utilization_three_to_one() {
    let only_utilization = BaseGrade::blend(1.0, 0.0);
    assert!(close(only_utilization.score, 0.75));
    let only_infrastructure = BaseGrade::blend(0.0, 1.0);
    assert!(close(only_infrastructure.score, 0.25));
    let mixed = BaseGrade::blend(0.5, 1.0);
    assert!(close(mixed.score, 0.625));
    assert!(close(mixed.utilization, 0.5));
    assert!(close(mixed.infrastructure, 1.0));
}

#[test]
fn default_grade_is_all_zero() {
    let g = BaseGrade::default();
    assert!(close(g.score, 0.0) && close(g.utilization, 0.0) && close(g.infrastructure, 0.0));
}

#[test]
fn max_level_building_raises_known_rooms_to_their_phase_count() {
    let mut data = BuildingDataFile::default();
    data.rooms
        .insert("MANUFACTURE".into(), room_def("MANUFACTURE", 3));
    data.rooms
        .insert("DORMITORY".into(), room_def("DORMITORY", 5));
    let building = UserBuilding {
        drone_capacity: None,
        rooms: vec![
            room("slot_a", "MANUFACTURE", 1),
            room("slot_b", "DORMITORY", 2),
            room("slot_c", "WORKSHOP", 2),
            room("slot_d", "MANUFACTURE", 7),
        ],
    };
    let maxed = max_level_building(&building, &data);
    let levels: Vec<(&str, i32)> = maxed
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r.level))
        .collect();
    assert_eq!(
        levels,
        vec![("slot_a", 3), ("slot_b", 5), ("slot_c", 2), ("slot_d", 7)]
    );
    assert_eq!(maxed.rooms[0].current_operators, vec!["char_002_amiya"]);
    assert_eq!(maxed.rooms[0].room_type, "MANUFACTURE");
    assert_eq!(building.rooms[0].level, 1);
}
