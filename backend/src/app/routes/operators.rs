use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
    response::Response,
};

use crate::app::routes::static_data::json_response;
use crate::app::services::operators::{
    OperatorBuildStatsResponse, OperatorIndexEntry, OperatorOwnershipResponse, get_build_stats,
    get_index, get_operator_json, get_operator_skins, get_operator_voices, get_ownership,
    get_upcoming, resolve_operator_json,
};
use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::types::skin::SkinData;
use crate::core::gamedata::types::voice::Voices;
use crate::core::hypergryph::constants::Server;

/// Every operator on the default (EN) server, in the compact form the roster
/// and list screens use.
#[utoipa::path(
    get,
    path = "/operators/index",
    tag = "gamedata",
    responses(
        (status = 200, description = "The operator index.", body = Vec<OperatorIndexEntry>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn index(
    State(state): State<AppState>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_index(&state, state.default_server).await?))
}

/// `GET /{server}/operators/index` - per-server operator index.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/operators/index",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`.")
    ),
    responses(
        (status = 200, description = "The operator index.", body = Vec<OperatorIndexEntry>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn index_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_index(&state, server).await?))
}

/// `GET /operators/ownership` - default-server operator ownership rates.
#[utoipa::path(
    get,
    path = "/operators/ownership",
    tag = "gamedata",
    responses(
        (status = 200, description = "Ownership counts by operator id.", body = OperatorOwnershipResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn ownership(
    State(state): State<AppState>,
) -> Result<Json<OperatorOwnershipResponse>, ApiError> {
    Ok(Json(get_ownership(&state, state.default_server).await?))
}

/// `GET /{server}/operators/ownership` - per-server operator ownership rates.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/operators/ownership",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`.")
    ),
    responses(
        (status = 200, description = "Ownership counts by operator id.", body = OperatorOwnershipResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn ownership_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<OperatorOwnershipResponse>, ApiError> {
    Ok(Json(get_ownership(&state, server).await?))
}

/// `GET /operators/{id}/build-stats` - default-server community defaults.
/// How tracked players actually build one operator: skill, mastery and module
/// choices, as proportions.
#[utoipa::path(
    get,
    path = "/operators/{id}/build-stats",
    tag = "gamedata",
    params(
        ("id" = String, Path, description = "Operator id, e.g. `char_002_amiya`.")
    ),
    responses(
        (status = 200, description = "Aggregate build choices.", body = OperatorBuildStatsResponse),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn build_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<OperatorBuildStatsResponse>, ApiError> {
    Ok(Json(
        get_build_stats(&state, state.default_server, &id).await?,
    ))
}

/// `GET /{server}/operators/{id}/build-stats` - per-server community defaults.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/operators/{id}/build-stats",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id, e.g. `char_002_amiya`.")
    ),
    responses(
        (status = 200, description = "Aggregate build choices.", body = OperatorBuildStatsResponse),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn build_stats_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<OperatorBuildStatsResponse>, ApiError> {
    Ok(Json(get_build_stats(&state, server, &id).await?))
}

/// `GET /upcoming` - operators on CN not yet on the default (EN) server.
#[utoipa::path(
    get,
    path = "/upcoming",
    tag = "gamedata",
    responses(
        (status = 200, description = "Operators still to arrive, soonest first.", body = Vec<OperatorIndexEntry>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn upcoming(
    State(state): State<AppState>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_upcoming(&state, Server::CN).await?))
}

/// `GET /{server}/upcoming` - operators on `{server}` not yet on the default server.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/upcoming",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`.")
    ),
    responses(
        (status = 200, description = "Operators still to arrive, soonest first.", body = Vec<OperatorIndexEntry>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn upcoming_srv(
    State(state): State<AppState>,
    Path(server): Path<Server>,
) -> Result<Json<Vec<OperatorIndexEntry>>, ApiError> {
    Ok(Json(get_upcoming(&state, server).await?))
}

/// One operator's full record: stats, talents, skills, modules and handbook.
///
/// Resolves across loaded servers (default first, then CN/others) and tags the
/// response with the `server` it was found on, so the client fetches once for
/// both global and upcoming operators.
#[utoipa::path(
    get,
    path = "/operators/{id}",
    tag = "gamedata",
    params(
        ("id" = String, Path, description = "Operator id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The operator. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let cached = resolve_operator_json(&state, state.default_server, &id).await?;
    Ok(json_response(cached, &headers))
}

/// `GET /{server}/operators/{id}` - one enriched operator from `{server}`.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/operators/{id}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The operator. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let cached = get_operator_json(&state, server, &id).await?;
    Ok(json_response(cached, &headers))
}

/// An operator's voice lines on the default server, with the audio URLs for
/// each language.
#[utoipa::path(
    get,
    path = "/voices/{id}",
    tag = "gamedata",
    params(
        ("id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "Voice lines by language.", body = Voices),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn voices_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Voices>, ApiError> {
    Ok(Json(
        get_operator_voices(&state, state.default_server, &id).await?,
    ))
}

/// `GET /{server}/voices/{id}` - one operator's voice lines from `{server}`.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/voices/{id}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "Voice lines by language.", body = Voices),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn voices_detail_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<Voices>, ApiError> {
    Ok(Json(get_operator_voices(&state, server, &id).await?))
}

/// `GET /skins/{id}` - one operator's skins (default server).
#[utoipa::path(
    get,
    path = "/skins/{id}",
    tag = "gamedata",
    params(
        ("id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "The operator's skins.", body = SkinData),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn skins_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SkinData>, ApiError> {
    Ok(Json(
        get_operator_skins(&state, state.default_server, &id).await?,
    ))
}

/// `GET /{server}/skins/{id}` - one operator's skins from `{server}`.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/skins/{id}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "The operator's skins.", body = SkinData),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn skins_detail_srv(
    State(state): State<AppState>,
    Path((server, id)): Path<(Server, String)>,
) -> Result<Json<SkinData>, ApiError> {
    Ok(Json(get_operator_skins(&state, server, &id).await?))
}
