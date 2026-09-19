use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::routes::static_data::json_response;
use crate::app::services::static_data::get_chibi;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `GET /chibis/{operatorId}` - one operator's chibi catalog entry (default
/// server). Replaces fetching the whole `/static/chibis` catalog to find one.
/// An operator's chibi animation set, with the asset paths to load it.
#[utoipa::path(
    get,
    path = "/chibis/{operator_id}",
    tag = "gamedata",
    params(
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The chibi record. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn chibi_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(operator_id): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_chibi(&state, state.default_server, &operator_id).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/chibis/{operatorId}` - per-server variant.
/// An operator's chibi animation set, with the asset paths to load it.///
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/chibis/{operator_id}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The chibi record. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn chibi_detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, operator_id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_chibi(&state, server, &operator_id).await?;
    Ok(json_response(body, &headers))
}
