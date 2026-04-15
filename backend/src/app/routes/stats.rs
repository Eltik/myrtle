use axum::Json;
use axum::extract::State;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::services::stats::{AdminStatsResponse, StatsResponse};
use crate::app::state::AppState;

pub async fn stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let stats = services::stats::get_stats(&state).await?;
    Ok(Json(stats))
}

pub async fn admin_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<AdminStatsResponse>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let stats = services::stats::get_admin_stats(&state).await?;
    Ok(Json(stats))
}
