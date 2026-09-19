//! Account-wide figures that are not a base or a single plan.
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::services::max_level::{MaxLevelCostResponse, max_level_costs};
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct AccountParams {
    pub uid: Option<String>,
}

/// `GET /user/max-level-cost`: the EXP and LMD still needed to bring every
/// owned operator to its final promotion and level cap, against what the
/// account holds. A roster walk over static tables - milliseconds, no
/// admission needed.
/// What it would cost this player to take every owned operator to max level.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/user/max-level-cost",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
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
    Ok(Json(max_level_costs(&state, &uid).await?))
}
