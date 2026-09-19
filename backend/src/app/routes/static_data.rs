use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::app::cache::CachedJson;
use crate::app::services::static_data::get_resource;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// Wrap a cached JSON body (with its precomputed `ETag`) in the shared game-data
/// caching headers (`ETag` + `Cache-Control: public, max-age=300`) and honor
/// `If-None-Match`. Reused by the slim per-record game-data endpoints
/// (stage/enemy/chibi/skins/operator). The 304 branch never touches the body.
pub(crate) fn json_response(cached: CachedJson, headers: &HeaderMap) -> Response {
    let CachedJson { body, etag } = cached;
    if let Some(inm) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        && inm == etag
    {
        return (
            StatusCode::NOT_MODIFIED,
            [
                (header::ETAG, etag),
                (header::CACHE_CONTROL, "public, max-age=300".to_owned()),
            ],
        )
            .into_response();
    }

    (
        [
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=300".to_owned()),
        ],
        body,
    )
        .into_response()
}

/// `GET /static/{resource}` - default (EN) game-data table.
/// One whole game-data table, verbatim.
///
/// The resource names are fixed; `tests/api_shape_test.rs` pins the JSON shape
/// of every one of them.
#[utoipa::path(
    get,
    path = "/static/{resource}",
    tag = "gamedata",
    params(
        ("resource" = String, Path, description = "Table name, e.g. `item_table` or `stage-index`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The table. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_static(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(resource): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, state.default_server, &resource).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/static/{resource}` - per-server game-data table.
/// One whole game-data table, verbatim.///
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/static/{resource}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("resource" = String, Path, description = "Table name, e.g. `item_table` or `stage-index`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The table. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_static_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, resource)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, server, &resource).await?;
    Ok(json_response(body, &headers))
}
