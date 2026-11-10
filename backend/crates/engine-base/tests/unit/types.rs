use crate::types::*;

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
