//! Shared helpers for the base-optimizer integration tests.
//!
//! Different test binaries use different subsets of these helpers, so unused
//! ones are expected per-crate.
#![allow(dead_code, clippy::cast_sign_loss)]

use backend::core::gamedata::types::GameData;
use backend::core::gamedata::{self};
use std::path::Path;
use std::sync::OnceLock;

/// The game data, loaded ONCE per test binary and shared by every test in it.
/// Each load is hundreds of MB; the old per-test loading held several copies in
/// flight under parallel test threads and dominated both memory and runtime.
pub fn load_game_data() -> &'static GameData {
    shared_game_data_ref()
}

/// The same single load, as the `Arc` an `AppState`'s `ArcSwap` wants.
///
/// Held as an `Arc` rather than a plain value so a test that needs a real
/// `AppState` shares this load instead of taking a second one: two copies of
/// the EN tables in one binary is where the integration suite used to run out
/// of memory.
pub fn shared_game_data() -> std::sync::Arc<GameData> {
    std::sync::Arc::clone(shared_game_data_arc())
}

fn shared_game_data_ref() -> &'static GameData {
    shared_game_data_arc()
}

fn shared_game_data_arc() -> &'static std::sync::Arc<GameData> {
    static GAME_DATA: OnceLock<std::sync::Arc<GameData>> = OnceLock::new();
    GAME_DATA.get_or_init(|| {
        std::sync::Arc::new({
            // Per-server layout: `{ASSETS_DIR}/{server}` holds each server's assets, with
            // the gamedata excel tables under `gamedata/excel`. Tests run against EN data.
            let data_dir_str = std::env::var("GAME_DATA_DIR")
                .unwrap_or_else(|_| "../assets/output/en/gamedata/excel".into());
            let assets_dir_str =
                std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
            gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
                .expect("Failed to load game data")
                .0
        })
    })
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
