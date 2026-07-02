use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};

use crate::app::routes::static_data::json_response;
use crate::app::services::operators::{
    OperatorIndexEntry, OperatorOwnershipResponse, get_index, get_operator_json,
    get_operator_skins, get_operator_voices, get_ownership, get_upcoming, resolve_operator_json,
};
use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::types::skin::SkinData;
use crate::core::gamedata::types::voice::Voices;
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

/// `GET /operators/ownership` - default-server operator ownership rates.
pub async fn ownership(
    State(state): State<AppState>,
) -> Result<Json<OperatorOwnershipResponse>, ApiError> {
    Ok(Json(get_ownership(&state, state.default_server).await?))
}

/// `GET /{server}/operators/ownership` - per-server operator ownership rates.
pub async fn ownership_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<OperatorOwnershipResponse>, ApiError> {
    Ok(Json(get_ownership(&state, server).await?))
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

/// `GET /operators/{id}` - one enriched operator. Resolves across loaded servers
/// (default first, then CN/others) and tags the response with the `server` it was
/// found on, so the client fetches once for both global and upcoming operators.
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let cached = resolve_operator_json(&state, state.default_server, &id).await?;
    Ok(json_response(cached, &headers))
}

/// `GET /{server}/operators/{id}` - one enriched operator from `{server}`.
pub async fn detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let cached = get_operator_json(&state, server, &id).await?;
    Ok(json_response(cached, &headers))
}

/// `GET /voices/{id}` - one operator's voice lines (default server).
pub async fn voices_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Voices>, ApiError> {
    Ok(Json(
        get_operator_voices(&state, state.default_server, &id).await?,
    ))
}

/// `GET /{server}/voices/{id}` - one operator's voice lines from `{server}`.
pub async fn voices_detail_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<Voices>, ApiError> {
    Ok(Json(get_operator_voices(&state, server, &id).await?))
}

/// `GET /skins/{id}` - one operator's skins (default server).
pub async fn skins_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SkinData>, ApiError> {
    Ok(Json(
        get_operator_skins(&state, state.default_server, &id).await?,
    ))
}

/// `GET /{server}/skins/{id}` - one operator's skins from `{server}`.
pub async fn skins_detail_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<SkinData>, ApiError> {
    Ok(Json(get_operator_skins(&state, server, &id).await?))
}
