use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id;
use crate::app::routes::static_data::json_response;
use crate::app::services::static_data::get_stage_detail;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::stages::get_user_stage_clears;

#[derive(Deserialize)]
pub struct StageClearsParams {
    pub uid: Option<String>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize)]
pub struct StageClearDto {
    pub state: i16,
    #[serde(rename = "stateMax")]
    pub state_max: i16,
    pub inferred: bool,
    #[serde(rename = "completeTimes")]
    pub complete_times: i32,
    #[serde(rename = "practiceTimes")]
    pub practice_times: i32,
}

/// `GET /stages/{stageId}/detail` - one stage plus its zone, level data, the
/// enemies it references and its drop materials (default server). Replaces the
/// stage-detail page's full stages/zones/enemies/materials table fetches.
/// One stage's full record.
#[utoipa::path(
    get,
    path = "/stages/{stage_id}/detail",
    tag = "gamedata",
    params(
        ("stage_id" = String, Path, description = "Stage id, e.g. `main_01-07`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The stage. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stage_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(stage_id): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_stage_detail(&state, state.default_server, &stage_id).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/stages/{stageId}/detail` - per-server variant.
/// One stage's full record.///
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/stages/{stage_id}/detail",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("stage_id" = String, Path, description = "Stage id, e.g. `main_01-07`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The stage. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stage_detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, stage_id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_stage_detail(&state, server, &stage_id).await?;
    Ok(json_response(body, &headers))
}

/// Which stages a player has cleared, and how.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/stage-clears",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Clear records keyed by stage id.", body = std::collections::HashMap<String, StageClearDto>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_stage_clears(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<StageClearsParams>,
) -> Result<Json<std::collections::HashMap<String, StageClearDto>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let data = get_user_stage_clears(&state.db, user_id).await?;
    let body = data
        .clears
        .into_iter()
        .map(|(id, c)| {
            (
                id,
                StageClearDto {
                    state: c.state,
                    state_max: c.state_max,
                    inferred: c.inferred,
                    complete_times: c.complete_times,
                    practice_times: c.practice_times,
                },
            )
        })
        .collect();
    Ok(Json(body))
}
