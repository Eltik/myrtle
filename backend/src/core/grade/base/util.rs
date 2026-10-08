use crate::core::gamedata::types::building::{BuildingDataFile, RoomPhase};

/// Trading posts and factories: the rooms that produce yield.
pub fn is_production_room(room_type: &str) -> bool {
    matches!(room_type, "MANUFACTURE" | "TRADING")
}

/// Per-level stats for `room_type` at `level` (1-indexed).
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

/// Falls back to 1 when the room type or level is unknown.
pub fn max_stationed_at_level(
    building_data: &BuildingDataFile,
    room_type: &str,
    level: i32,
) -> i32 {
    room_phase(building_data, room_type, level).map_or(1, |phase| phase.max_stationed_num)
}

/// Buff id without the tier suffix (`manu_prod_spd_variable[000]` ->
/// `manu_prod_spd_variable`). Non-stacking and priority clauses name families.
pub fn buff_family(buff_id: &str) -> &str {
    buff_id.split('[').next().unwrap_or(buff_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::building::RoomDef;

    fn room_def(id: &str, stationed: &[i32]) -> RoomDef {
        let phases: Vec<serde_json::Value> = stationed
            .iter()
            .map(|n| {
                serde_json::json!({
                    "UnlockCondId": "",
                    "MaxStationedNum": n,
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

    fn building() -> BuildingDataFile {
        let mut data = BuildingDataFile::default();
        data.rooms
            .insert("MANUFACTURE".into(), room_def("MANUFACTURE", &[1, 2, 3]));
        data
    }

    #[test]
    fn only_factories_and_trading_posts_produce() {
        assert!(is_production_room("MANUFACTURE"));
        assert!(is_production_room("TRADING"));
        for other in ["POWER", "DORMITORY", "CONTROL", "manufacture", ""] {
            assert!(!is_production_room(other), "{other}");
        }
    }

    #[test]
    fn room_phase_is_one_indexed() {
        let data = building();
        assert_eq!(
            room_phase(&data, "MANUFACTURE", 1).map(|p| p.max_stationed_num),
            Some(1)
        );
        assert_eq!(
            room_phase(&data, "MANUFACTURE", 3).map(|p| p.max_stationed_num),
            Some(3)
        );
        assert!(room_phase(&data, "MANUFACTURE", 4).is_none());
        assert!(room_phase(&data, "TRADING", 1).is_none());
    }

    #[test]
    fn level_zero_and_below_read_the_first_phase() {
        let data = building();
        assert_eq!(
            room_phase(&data, "MANUFACTURE", 0).map(|p| p.max_stationed_num),
            Some(1)
        );
        assert_eq!(
            room_phase(&data, "MANUFACTURE", -3).map(|p| p.max_stationed_num),
            Some(1)
        );
    }

    #[test]
    fn max_stationed_falls_back_to_one() {
        let data = building();
        assert_eq!(max_stationed_at_level(&data, "MANUFACTURE", 2), 2);
        assert_eq!(max_stationed_at_level(&data, "MANUFACTURE", 9), 1);
        assert_eq!(max_stationed_at_level(&data, "WORKSHOP", 1), 1);
    }

    #[test]
    fn buff_family_drops_the_tier_suffix() {
        assert_eq!(
            buff_family("manu_prod_spd_variable[000]"),
            "manu_prod_spd_variable"
        );
        assert_eq!(buff_family("trade_ord_spd[010]"), "trade_ord_spd");
        assert_eq!(buff_family("no_suffix"), "no_suffix");
        assert_eq!(buff_family("[000]"), "");
        assert_eq!(buff_family(""), "");
    }
}
