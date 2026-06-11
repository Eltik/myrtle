use axum::{
    Json,
    extract::{Path, State},
};

use crate::app::services::operators::{
    OperatorIndexEntry, get_index, get_operator, get_operator_skins, get_operator_voices,
    get_upcoming, resolve_operator,
};
use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::types::operator::Operator;
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
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let (op, server) =
        resolve_operator(&state, state.default_server, &id).ok_or(ApiError::NotFound)?;
    with_server(&op, server)
}

/// `GET /{server}/operators/{id}` - one enriched operator from `{server}`.
pub async fn detail_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let op = get_operator(&state, server, &id).await?;
    with_server(&op, server)
}

/// Serialize an operator and tag it with the server it was resolved from, so the
/// client can build region-correct asset URLs without a separate probe.
fn with_server(op: &Operator, server: Server) -> Result<Json<serde_json::Value>, ApiError> {
    let mut value = serde_json::to_value(op).map_err(|e| ApiError::Internal(e.into()))?;
    if let serde_json::Value::Object(map) = &mut value {
        map.insert(
            "server".to_string(),
            serde_json::Value::String(server.as_str().to_string()),
        );
    }
    Ok(Json(value))
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
