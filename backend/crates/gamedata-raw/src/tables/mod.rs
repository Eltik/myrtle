//! One module per table family. Each holds the serde structs exactly as the
//! excel JSON is shaped; no derived fields.
//!
//! Not moved here: `backend/src/core/gamedata/types/chibi.rs` (chibi data comes
//! from the asset listing, see `gamedata::enrich::chibi`) and the derived
//! `stage_index.rs`, `stage_universe.rs`, `stage_evidence.rs` (see
//! `gamedata::index::stage`).

pub mod activity;
pub mod audio;
pub mod autochess;
pub mod building;
pub mod campaign;
pub mod chapter;
pub mod character;
pub mod consts;
pub mod enemy;
pub mod gacha;
pub mod handbook;
pub mod handbook_network;
pub mod handbook_team;
pub mod item;
pub mod material;
pub mod medal;
pub mod mission;
pub mod module;
pub mod obtain;
pub mod operator;
pub mod roguelike;
pub mod roguelike_entries;
pub mod sandbox;
pub mod sandbox_universe;
pub mod skill;
pub mod stage;
pub mod story;
pub mod trust;
pub mod voice;
pub mod zone;
