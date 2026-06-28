use axum::Json;
use axum::extract::{Path, State};

use crate::app::services::level::get_level;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `GET /level/{stage_id}` - raw Arknights level (camelCased keys, 2D map grid)
/// for the default (EN) server. Powers the Stage Viewer map + pathing renderer.
pub async fn get_level_map(
    State(state): State<AppState>,
    Path(stage_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = get_level(&state, state.default_server, &stage_id).await?;
    Ok(Json(value))
}

/// `GET /{server}/level/{stage_id}` - per-server variant of the raw level.
pub async fn get_level_map_srv(
    State(state): State<AppState>,
    Path((server, stage_id)): Path<(Server, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = get_level(&state, server, &stage_id).await?;
    Ok(Json(value))
}
