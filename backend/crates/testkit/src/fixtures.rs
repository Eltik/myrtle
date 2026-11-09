//! Load a small committed extract (or `MYRTLE_TEST_ASSETS` pointing at a real
//! one) into `RawGameData` and compile a `GameModel`, once per test binary.

use myrtle_gamedata::model::GameModel;

/// A compiled model for `server`, or `None` when no extract is available
/// (tests that need real data skip, as today's real-data tests do).
#[must_use]
pub fn model(server: myrtle_types::Server) -> Option<&'static GameModel> {
    let _ = server;
    todo!()
}
