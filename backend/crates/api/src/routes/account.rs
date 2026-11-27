use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::planner::max_level::{self, LevelTarget, MaxLevelCostResponse};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::MaybeCaller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/user/max-level-cost", get(max_level_cost))
}

#[derive(Deserialize)]
struct AccountParams {
    uid: Option<String>,
    #[serde(default)]
    target: LevelTarget,
}

async fn max_level_cost(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<AccountParams>,
) -> ApiResult<Json<MaxLevelCostResponse>> {
    let cost =
        max_level::max_level_cost(&ctx, &caller, params.uid.as_deref(), params.target).await?;
    Ok(Json(cost))
}
