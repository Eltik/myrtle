use axum::Json;
use axum::extract::State;
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::stats::{self, AdminStatsResponse, StatsResponse};

use crate::error::ApiResult;
use crate::extract::Caller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/stats", get(site_stats))
        .route("/admin/stats", get(admin_stats))
}

async fn site_stats(State(ctx): State<AppContext>) -> ApiResult<Json<StatsResponse>> {
    Ok(Json(stats::site(&ctx).await?))
}

async fn admin_stats(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<AdminStatsResponse>> {
    Ok(Json(stats::admin(&ctx, &caller).await?))
}
