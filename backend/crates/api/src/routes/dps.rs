use axum::Json;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use myrtle_app::AppContext;
use myrtle_app::services::catalog::dps::{
    self, BatchRequest, CalculateRequest, RankRequest, RankResponse, RetentionResponse,
    SupportersQuery,
};

use crate::error::ApiResult;
use crate::extract;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/dps/operators", get(operators))
        .route("/dps/calculate", post(calculate))
        .route("/hps/operators", get(healers))
        .route("/hps/calculate", post(calculate_hps))
        .route("/dps/batch", post(batch))
        .route("/hps/batch", post(batch_hps))
        .route("/dps/rank", post(rank))
        .route("/dps/retention", get(retention))
        .route("/dps/supporters", get(supporters))
}

async fn operators(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(Cached::gamedata(dps::list_operators(&ctx).await?))
}

async fn healers(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(Cached::gamedata(dps::list_healers(&ctx).await?))
}

async fn calculate(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<CalculateRequest>,
) -> ApiResult<Cached> {
    Ok(dps::calculate(&ctx, body).await?.into())
}

async fn calculate_hps(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<CalculateRequest>,
) -> ApiResult<Cached> {
    Ok(dps::calculate_hps(&ctx, body).await?.into())
}

async fn batch(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<BatchRequest>,
) -> ApiResult<Cached> {
    Ok(dps::batch(&ctx, body).await?.into())
}

async fn batch_hps(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<BatchRequest>,
) -> ApiResult<Cached> {
    Ok(dps::batch_hps(&ctx, body).await?.into())
}

async fn rank(
    State(ctx): State<AppContext>,
    extract::Json(body): extract::Json<RankRequest>,
) -> ApiResult<Json<RankResponse>> {
    Ok(Json(dps::rank(&ctx, body).await?))
}

async fn retention(State(ctx): State<AppContext>) -> ApiResult<Json<RetentionResponse>> {
    Ok(Json(dps::retention(&ctx).await?))
}

async fn supporters(
    State(ctx): State<AppContext>,
    Query(query): Query<SupportersQuery>,
) -> ApiResult<Cached> {
    Ok(dps::supporters(&ctx, query).await?.into())
}
