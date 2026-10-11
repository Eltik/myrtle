//! The Archives: personnel files, the game's own network chart, and the roster
//! counted up. Pure derivation from game data; the cache and routes live in
//! `app::services::archives` and `app::routes::archives`.

pub mod build;
pub mod dto;
pub mod network;
pub mod profile;

pub use build::{Archive, SceneIndex, build_archive};
pub use dto::*;
