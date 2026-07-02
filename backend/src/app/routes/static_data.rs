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
pub async fn get_static(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(resource): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, state.default_server, &resource).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/static/{resource}` - per-server game-data table.
pub async fn get_static_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, resource)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, server, &resource).await?;
    Ok(json_response(body, &headers))
}
