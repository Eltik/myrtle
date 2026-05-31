//! Shared helpers for the base-optimizer integration tests.
//!
//! Different test binaries use different subsets of these helpers, so unused
//! ones are expected per-crate.
#![allow(dead_code)]

use backend::core::gamedata::types::GameData;
use backend::core::gamedata::{self};
use backend::core::grade::base::types::OperatorBaseProfile;
use backend::database::models::roster::RosterEntry;
use std::path::Path;

pub fn load_game_data() -> GameData {
    let data_dir_str =
        std::env::var("GAME_DATA_DIR").unwrap_or_else(|_| "../assets/output/gamedata/excel".into());
    let assets_dir_str = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output".into());
    gamedata::init_game_data(Path::new(&data_dir_str), Path::new(&assets_dir_str))
        .expect("Failed to load game data")
}

/// A real user's base + roster, captured from the database into the committed
/// fixture `tests/fixtures/user_<uid>.json`.
pub struct UserFixture {
    pub uid: String,
    pub building: serde_json::Value,
    pub roster: Vec<RosterEntry>,
}

/// Load `tests/fixtures/user_<uid>.json`.
pub fn load_user_fixture(uid: &str) -> UserFixture {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("user_{uid}.json"));
    let raw: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display())),
    )
    .expect("parse user fixture");

    let building = raw.get("building").cloned().expect("fixture has building");
    let roster: Vec<RosterEntry> =
        serde_json::from_value(raw.get("roster").cloned().expect("fixture has roster"))
            .expect("deserialize roster");

    UserFixture {
        uid: uid.to_string(),
        building,
        roster,
    }
}

/// Roster → base-skill profiles (mirrors the production `build_base_improvements`
/// path: drops operators with no entry in building data).
pub fn build_profiles(roster: &[RosterEntry], game_data: &GameData) -> Vec<OperatorBaseProfile> {
    roster
        .iter()
        .filter_map(|entry| {
            let bc = game_data.building.chars.get(&entry.operator_id)?;
            let faction_tags = game_data
                .operators
                .get(&entry.operator_id)
                .map(backend::core::grade::base::buff_registry::faction_tags_of)
                .unwrap_or_default();
            Some(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                &game_data.building,
            ))
        })
        .collect()
}

pub fn operator_name<'a>(char_id: &'a str, game_data: &'a GameData) -> &'a str {
    game_data
        .operators
        .get(char_id)
        .map_or(char_id, |op| op.name.as_str())
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
