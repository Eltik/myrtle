//! Archives routes: the network chart, the dossiers, one personnel file, the
//! roster counted up, and the hidden factions. Each has a bare form (the
//! default server) and a `/{server}` form. Built once per game-data load; see
//! `app::services::archives`.

use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};

use crate::app::routes::static_data::shared_json_response;
use crate::app::services::archives::{
    get_demographics, get_dossiers, get_hidden_factions, get_network, get_operator,
};
use crate::app::{error::ApiError, state::AppState};
use crate::core::archives::{
    ArchiveDemographics, ArchiveDossiers, ArchiveHiddenFactions, ArchiveNetwork, ArchiveOperator,
};
use crate::core::hypergryph::constants::Server;

/// The game's own Archives network: one hex cluster per faction on the territories map, and one chart per group with every member placed on it.
#[utoipa::path(
    get,
    path = "/archives/network",
    operation_id = "archives_network",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The network. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveNetwork),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn network(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_network(&state, state.default_server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// `GET /{server}/archives/network`: the same, from `{server}`'s game data.
#[utoipa::path(
    get,
    path = "/{server}/archives/network",
    operation_id = "archives_network_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The network. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveNetwork),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn network_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_network(&state, server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// Every operator file, one row each, with the profile fields the Dossiers table sorts and filters on.
#[utoipa::path(
    get,
    path = "/archives/operators",
    operation_id = "archives_dossiers",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "Every dossier. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveDossiers),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn dossiers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_dossiers(&state, state.default_server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// `GET /{server}/archives/operators`: the same, from `{server}`'s game data.
#[utoipa::path(
    get,
    path = "/{server}/archives/operators",
    operation_id = "archives_dossiers_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "Every dossier. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveDossiers),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn dossiers_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_dossiers(&state, server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// By the Numbers: the operator files counted up by field, straight from each file.
#[utoipa::path(
    get,
    path = "/archives/demographics",
    operation_id = "archives_demographics",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The roster counted up. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveDemographics),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn demographics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_demographics(&state, state.default_server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// `GET /{server}/archives/demographics`: the same, from `{server}`'s game data.
#[utoipa::path(
    get,
    path = "/{server}/archives/demographics",
    operation_id = "archives_demographics_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The roster counted up. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveDemographics),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn demographics_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_demographics(&state, server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// Operators counted in a faction their card never names, and the nations behind a card that shows a narrower group. Story spoilers: the page shows this only with spoilers enabled.
#[utoipa::path(
    get,
    path = "/archives/hidden-factions",
    operation_id = "archives_hidden_factions",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The hidden factions. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveHiddenFactions),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn hidden_factions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_hidden_factions(&state, state.default_server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// `GET /{server}/archives/hidden-factions`: the same, from `{server}`'s game data.
#[utoipa::path(
    get,
    path = "/{server}/archives/hidden-factions",
    operation_id = "archives_hidden_factions_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The hidden factions. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", body = ArchiveHiddenFactions),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn hidden_factions_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let json = get_hidden_factions(&state, server).await?;
    Ok(shared_json_response(&json, &headers))
}

/// One personnel file: the profile as the handbook states it, the personnel
/// number, records, the stories the operator speaks in, who they share the
/// most scenes with, their links and the charts they sit on. 404 for an id
/// with no file.
#[utoipa::path(
    get,
    path = "/archives/operator/{char_id}",
    operation_id = "archives_operator",
    tag = "gamedata",
    params(
        ("char_id" = String, Path, description = "Operator id (`char_002_amiya`).")
    ),
    responses(
        (status = 200, description = "The personnel file.", body = ArchiveOperator),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn operator(
    State(state): State<AppState>,
    Path(char_id): Path<String>,
) -> Result<Json<ArchiveOperator>, ApiError> {
    Ok(Json(
        get_operator(&state, state.default_server, &char_id).await?,
    ))
}

/// `GET /{server}/archives/operator/{char_id}`: the same, from `{server}`.
#[utoipa::path(
    get,
    path = "/{server}/archives/operator/{char_id}",
    operation_id = "archives_operator_srv",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("char_id" = String, Path, description = "Operator id (`char_002_amiya`).")
    ),
    responses(
        (status = 200, description = "The personnel file.", body = ArchiveOperator),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn operator_srv(
    State(state): State<AppState>,
    Path((server, char_id)): Path<(Server, String)>,
) -> Result<Json<ArchiveOperator>, ApiError> {
    Ok(Json(get_operator(&state, server, &char_id).await?))
}
