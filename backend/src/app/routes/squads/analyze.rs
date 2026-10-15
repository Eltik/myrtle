use axum::Json;
use axum::extract::{Query, State};

use crate::app::error::ApiError;
use crate::app::routes::tier_lists::ServerQuery;
use crate::app::services::squad::{self, SquadRules};
use crate::app::state::AppState;
use crate::core::squad::types::{SquadAnalysis, SquadAnalyzeRequest};

/// Analyse one squad: role coverage, DP economy, synergy, readiness, themes
/// and the DPS calculator hand-off. Public; operators the game data does not know
/// are skipped.
#[utoipa::path(
    post,
    path = "/squads/analyze",
    operation_id = "squad_analyze",
    tag = "squads",
    params(
        ("server" = Option<String>, Query, description = "Game server whose operators, numbers and names the analysis reads: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server. The text rules' readings come from EN by id; an operator EN does not have yet is listed in `unread_seats`.")
    ),
    request_body = SquadAnalyzeRequest,
    responses(
        (status = 200, description = "The squad's five panels and the per-seat hand-off.", body = SquadAnalysis),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, description = "The CPU admission queue is full. Retry shortly.")
    )
)]
pub async fn analyze(
    State(state): State<AppState>,
    Query(query): Query<ServerQuery>,
    Json(body): Json<SquadAnalyzeRequest>,
) -> Result<Json<SquadAnalysis>, ApiError> {
    let server = query.or_default(&state);
    let analysis = squad::analyze(&state, body, server).await?;
    Ok(Json(analysis))
}

/// The per-server facts the page reads before any squad exists: the enemy
/// grade bounds behind the stage threats, and each challenge stage's normal
/// stage. Public.
#[utoipa::path(
    get,
    path = "/squads/rules",
    operation_id = "squad_rules",
    tag = "squads",
    params(
        ("server" = Option<String>, Query, description = "Game server whose tables to read: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server.")
    ),
    responses(
        (status = 200, description = "Threat thresholds from `enemy_handbook_table.LevelInfoList` and the challenge-to-normal stage map.", body = SquadRules),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn rules(
    State(state): State<AppState>,
    Query(query): Query<ServerQuery>,
) -> Json<SquadRules> {
    Json(squad::rules(&state, query.or_default(&state)))
}
