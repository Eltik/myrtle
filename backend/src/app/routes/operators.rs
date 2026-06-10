use axum::{
    Json,
    extract::{Path, State},
};

use crate::app::services::operators::{OperatorIndexEntry, get_index, get_upcoming};
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `GET /operators/index` - default (EN) operator index.
pub async fn index(
    State(state): State<AppState>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_index(&state, state.default_server).await?))
}

/// `GET /{server}/operators/index` - per-server operator index.
pub async fn index_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_index(&state, server).await?))
}

/// `GET /upcoming` - operators on CN not yet on the default (EN) server.
pub async fn upcoming(
    State(state): State<AppState>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_upcoming(&state, Server::CN).await?))
}

/// `GET /{server}/upcoming` - operators on `{server}` not yet on the default server.
pub async fn upcoming_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_upcoming(&state, server).await?))
}
