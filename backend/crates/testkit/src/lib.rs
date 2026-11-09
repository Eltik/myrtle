//! Test fixtures and fakes. Allowed only as a dev-dependency (`xtask
//! check-layers`, rule 3).
//!
//! Moves in: `backend/tests/common/mod.rs`, `backend/tests/fixtures/` (the
//! expected DPS/HPS JSON went to `crates/dps-formulas/tests/fixtures/` with the
//! suites, lift-3), and the real-data loading that
//! the 15 `backend/tests/*_real_data_test.rs` files each repeat.

pub mod fixtures;
pub mod real_data;
pub mod sessions;
