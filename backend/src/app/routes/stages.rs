use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct StageClearDto {
    pub state: i16,
    #[serde(rename = "completeTimes")]
    pub complete_times: i32,
    #[serde(rename = "practiceTimes")]
    pub practice_times: i32,
}

/// `GET /stages/{stageId}/detail` - one stage plus its zone, level data, the
/// enemies it references and its drop materials (default server). Replaces the
/// stage-detail page's full stages/zones/enemies/materials table fetches.
pub async fn stage_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(stage_id): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_stage_detail(&state, state.default_server, &stage_id).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/stages/{stageId}/detail` - per-server variant.
pub async fn stage_detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, stage_id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_stage_detail(&state, server, &stage_id).await?;
    Ok(json_response(body, &headers))
}

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
                    complete_times: c.complete_times,
                    practice_times: c.practice_times,
                },
            )
        })
        .collect();
    Ok(Json(body))
}
