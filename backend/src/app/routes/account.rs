//! Account-wide figures that are not a base or a single plan.
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::services::max_level::{LevelTarget, MaxLevelCostResponse, max_level_costs};
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct AccountParams {
    pub uid: Option<String>,
    /// Where each operator's walk stops; the level cap when omitted.
    #[serde(default)]
    pub target: LevelTarget,
}

/// `GET /user/max-level-cost`: the EXP and LMD still needed to bring every
/// owned operator to its target, the level cap or the level its modules
/// unlock at, against what the account holds. A roster walk over static
/// tables - milliseconds, no admission needed.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/user/max-level-cost",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token."),
        ("target" = Option<LevelTarget>, Query, description = "Where each operator's walk stops: `max` (the level cap, the default) or `module` (the level its modules unlock at; an operator without a module takes its rarity's module level, a rarity with no modules keeps its cap).")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Materials and LMD required, and what the player already holds.", body = MaxLevelCostResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_max_level_cost(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<AccountParams>,
) -> Result<Json<MaxLevelCostResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    Ok(Json(max_level_costs(&state, &uid, params.target).await?))
}
