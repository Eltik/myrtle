pub mod config;
pub mod custom;
pub mod data_cache;
pub mod engine;
pub mod engine_version;
pub mod formulas;
pub mod operator_data;
pub mod operator_unit;
pub mod supporters;
pub mod zones;

/// The engine's source hash from `build.rs` (`engine_version`), in every DPS
/// cache key so a deploy that changes the engine never reads results the old
/// one computed.
pub const ENGINE_VERSION: &str = env!("DPS_ENGINE_VERSION");
