use axum::Json;
use axum::extract::{Path, State};
use axum::http::header;

use crate::app::services::level::get_level;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// A stage's raw level data (camelCased keys, 2D map grid) on the default (EN)
/// server: the tile map, enemy routes and wave schedule that drive the stage
/// viewer's pathing simulation.
#[utoipa::path(
    get,
    path = "/level/{stage_id}",
    tag = "gamedata",
    params(
        ("stage_id" = String, Path, description = "Stage id, e.g. `main_01-07`.")
    ),
    responses(
        (status = 200, description = "The level data, with `Cache-Control` set for long reuse.", content_type = "application/json"),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_level_map(
    State(state): State<AppState>,
    Path(stage_id): Path<String>,
) -> Result<
    (
        [(header::HeaderName, &'static str); 1],
        Json<serde_json::Value>,
    ),
    ApiError,
> {
    let value = get_level(&state, state.default_server, &stage_id).await?;
    Ok((
        [(header::CACHE_CONTROL, "public, max-age=300")],
        Json(value),
    ))
}

/// `GET /{server}/level/{stage_id}` - per-server variant of the raw level.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/level/{stage_id}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("stage_id" = String, Path, description = "Stage id, e.g. `main_01-07`.")
    ),
    responses(
        (status = 200, description = "The level data, with `Cache-Control` set for long reuse.", content_type = "application/json"),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_level_map_srv(
    State(state): State<AppState>,
    Path((server, stage_id)): Path<(Server, String)>,
) -> Result<
    (
        [(header::HeaderName, &'static str); 1],
        Json<serde_json::Value>,
    ),
    ApiError,
> {
    let value = get_level(&state, server, &stage_id).await?;
    Ok((
        [(header::CACHE_CONTROL, "public, max-age=300")],
        Json(value),
    ))
}
