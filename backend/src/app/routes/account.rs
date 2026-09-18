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
pub async fn get_max_level_cost(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<AccountParams>,
) -> Result<Json<MaxLevelCostResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    Ok(Json(max_level_costs(&state, &uid).await?))
}
