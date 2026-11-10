#![allow(clippy::cast_sign_loss)]

use std::sync::OnceLock;

use myrtle_engine_base::BaseData;

/// The game data, loaded once per test binary and shared by every test in it.
pub fn load_game_data() -> &'static BaseData {
    static GAME_DATA: OnceLock<BaseData> = OnceLock::new();
    GAME_DATA.get_or_init(|| {
        let excel = myrtle_testkit::real_data::excel_dir();
        let (operators, building, consts) =
            myrtle_testkit::real_data::base_tables(&excel).expect("Failed to load game data");
        BaseData::new(operators, building, consts)
    })
}

pub fn max_stationed(game_data: &BaseData, room_type: &str, level: i32) -> i32 {
    game_data
        .building()
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1).max(0) as usize))
        .map_or(1, |phase| phase.max_stationed_num)
}
