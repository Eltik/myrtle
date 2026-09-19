use axum::Json;
use axum::extract::State;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services::stats::get_admin_stats;
use crate::app::services::stats::get_stats;
use crate::app::services::stats::{AdminStatsResponse, StatsResponse};
use crate::app::state::AppState;

/// Public site-wide totals: tracked players, operators, and the like.
#[utoipa::path(
    get,
    path = "/stats",
    operation_id = "site_stats",
    tag = "meta",
    responses(
        (status = 200, description = "Aggregate counts.", body = StatsResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let stats = get_stats(&state).await?;
    Ok(Json(stats))
}

/// Operational counters behind the admin panel.
#[utoipa::path(
    get,
    path = "/admin/stats",
    tag = "admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Administrative counters.", body = AdminStatsResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn admin_stats(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<AdminStatsResponse>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let stats = get_admin_stats(&state).await?;
    Ok(Json(stats))
}
