use axum::Json;
use axum::extract::{Query, State};
use axum::routing::{get, post, put};
use myrtle_app::AppContext;
use myrtle_app::base_planner::{
    self, AccountFactsReq, CatalogResponse, EvaluateRequest, EvaluateResponse, LayoutResponse,
    OptimizeRequest, RotationRequest,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{self, Caller, MaybeCaller};
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/base/catalog", get(get_catalog))
        .route("/base/layout", get(get_layout))
        .route("/base/evaluate", post(evaluate_layout))
        .route("/base/optimize", post(optimize_layout))
        .route("/base/rotation", post(rotation_plan))
        .route("/base/facts", put(put_facts))
}

#[derive(Deserialize)]
struct PlannerParams {
    uid: Option<String>,
}

async fn get_catalog(State(ctx): State<AppContext>) -> ApiResult<Json<CatalogResponse>> {
    Ok(Json(base_planner::catalog(&ctx)?))
}

async fn get_layout(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<PlannerParams>,
) -> ApiResult<Json<LayoutResponse>> {
    Ok(Json(
        base_planner::layout(&ctx, &caller, params.uid.as_deref()).await?,
    ))
}

async fn evaluate_layout(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<PlannerParams>,
    extract::Json(body): extract::Json<EvaluateRequest>,
) -> ApiResult<Json<EvaluateResponse>> {
    Ok(Json(
        base_planner::evaluate(&ctx, &caller, params.uid.as_deref(), body).await?,
    ))
}

async fn optimize_layout(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<PlannerParams>,
    extract::Json(body): extract::Json<OptimizeRequest>,
) -> ApiResult<Cached> {
    let json = base_planner::optimize(&ctx, &caller, params.uid.as_deref(), body).await?;
    Ok(Cached::gamedata(json))
}

async fn rotation_plan(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<PlannerParams>,
    extract::Json(body): extract::Json<RotationRequest>,
) -> ApiResult<Cached> {
    let json = base_planner::rotation(&ctx, &caller, params.uid.as_deref(), body).await?;
    Ok(Cached::gamedata(json))
}

async fn put_facts(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<AccountFactsReq>,
) -> ApiResult<Json<AccountFactsReq>> {
    Ok(Json(base_planner::save_facts(&ctx, &caller, body).await?))
}
