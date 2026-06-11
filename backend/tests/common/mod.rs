//! Shared helpers for the base-optimizer integration tests.
//!
//! Different test binaries use different subsets of these helpers, so unused
//! ones are expected per-crate.
#![allow(dead_code, clippy::cast_sign_loss)]

use backend::core::gamedata::types::GameData;
use backend::core::gamedata::{self};
use std::path::Path;

pub fn load_game_data() -> GameData {
    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());
    gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
        .expect("Failed to load game data")
}

/// Max operators stationable in a room of `room_type` at `level` (1-indexed).
pub fn max_stationed(game_data: &GameData, room_type: &str, level: i32) -> i32 {
    game_data
        .building
        .rooms
        .get(room_type)
        .and_then(|def| def.phases.get((level - 1).max(0) as usize))
        .map_or(1, |phase| phase.max_stationed_num)
}
