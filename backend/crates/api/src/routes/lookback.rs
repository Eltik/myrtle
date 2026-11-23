use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::lookback::{self, LookbackBuild, LookbackOverview};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::MaybeCaller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/lookback", get(overview))
        .route("/lookback/current", get(current))
        .route("/lookback/snapshots/{id}", get(snapshot))
}

#[derive(Deserialize)]
struct LookbackQuery {
    uid: Option<String>,
}

async fn overview(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<LookbackQuery>,
) -> ApiResult<Json<LookbackOverview>> {
    Ok(Json(
        lookback::overview(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}

async fn snapshot(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Path(id): Path<i64>,
    Query(query): Query<LookbackQuery>,
) -> ApiResult<Json<LookbackBuild>> {
    Ok(Json(
        lookback::snapshot(&ctx, &caller, query.uid.as_deref(), id).await?,
    ))
}

async fn current(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<LookbackQuery>,
) -> ApiResult<Json<LookbackBuild>> {
    Ok(Json(
        lookback::current(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}
