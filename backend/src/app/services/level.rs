//! On-demand stage **level** loading for the Stage Viewer.
//!
//! Resolves a stage id to its `level_id`, reads the matching `level_*.json` off
//! disk, and normalizes it into a [`StageMap`] (tiles + routes + spawn timeline
//! + roster). Results are cached so the file is parsed at most once per stage.

use serde_json::Value;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::level::parse_stage_map;
use crate::core::hypergryph::constants::Server;

pub async fn get_level(
    state: &AppState,
    server: Server,
    stage_id: &str,
) -> Result<Value, ApiError> {
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;

    let resource = format!("level:{stage_id}");
    let key = CacheKey::StaticData {
        resource: &resource,
        server: server.as_str(),
        fields_hash: 0,
        page: 0,
    };
    if let Some(cached) = state.cache.get::<Value>(&key).await {
        return Ok(cached);
    }

    let gd = server_data.game_data.load_full();
    let stage = gd.stages.get(stage_id).ok_or(ApiError::NotFound)?;
    let level_id = stage.level_id.as_deref().ok_or(ApiError::NotFound)?;

    // level_id (e.g. "Obt/Main/level_main_01-07") lowercases to the on-disk path
    // under `gamedata/levels/`.
    let rel = level_id.to_lowercase().replace('\\', "/");
    let path = format!("{}/gamedata/levels/{rel}.json", server_data.assets_dir);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let map = parse_stage_map(stage, level_id, &bytes, &gd.enemies)
        .map_err(|e| ApiError::Internal(e.into()))?;
    let value = serde_json::to_value(&map).map_err(|e| ApiError::Internal(e.into()))?;

    state.cache.set(&key, &value).await;
    Ok(value)
}
