use axum::Json;
use axum::extract::{Path, State};

use crate::app::services::static_data::get_resource;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `GET /static/{resource}` - default (EN) game-data table.
pub async fn get_static(
    State(state): State<AppState>,
    Path(resource): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = get_resource(&state, state.default_server, &resource).await?;
    Ok(Json(value))
}

/// `GET /{server}/static/{resource}` - per-server game-data table.
pub async fn get_static_srv(
    State(state): State<AppState>,
    Path((server, resource)): Path<(Server, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = get_resource(&state, server, &resource).await?;
    Ok(Json(value))
}
