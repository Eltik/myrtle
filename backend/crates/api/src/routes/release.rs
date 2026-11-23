use axum::Json;
use axum::extract::{Path, State};
use axum::routing::{delete, get};
use myrtle_app::AppContext;
use myrtle_app::services::catalog::release as svc;
use myrtle_engine_release::{
    BannersResponse, EventsResponse, LagResponse, PutOverride, PutReleasePlan, ReleaseOverride,
    ReleasePlan, SkinsResponse,
};

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/release/events", get(events))
        .route("/release/banners", get(banners))
        .route("/release/skins", get(skins))
        .route("/release/lag", get(lag))
        .route("/release/plan", get(get_plan).put(put_plan))
        .route("/release/overrides", get(list_overrides).put(put_override))
        .route("/release/overrides/{kind}/{cn_id}", delete(delete_override))
}

async fn events(State(ctx): State<AppContext>) -> ApiResult<Json<EventsResponse>> {
    Ok(Json(svc::get_events(&ctx).await?))
}

async fn banners(State(ctx): State<AppContext>) -> ApiResult<Json<BannersResponse>> {
    Ok(Json(svc::get_banners(&ctx).await?))
}

async fn skins(State(ctx): State<AppContext>) -> ApiResult<Json<SkinsResponse>> {
    Ok(Json(svc::get_skins(&ctx).await?))
}

async fn lag(State(ctx): State<AppContext>) -> ApiResult<Json<LagResponse>> {
    Ok(Json(svc::get_lag(&ctx).await?))
}

async fn list_overrides(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<ReleaseOverride>>> {
    Ok(Json(svc::list_overrides(&ctx, &caller).await?))
}

async fn put_override(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<PutOverride>,
) -> ApiResult<Json<ReleaseOverride>> {
    Ok(Json(svc::put_override(&ctx, &caller, body).await?))
}

async fn delete_override(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path((kind, cn_id)): Path<(String, String)>,
) -> ApiResult<Json<StatusOk>> {
    svc::delete_override(&ctx, &caller, &kind, &cn_id).await?;
    Ok(ok())
}

async fn get_plan(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Option<ReleasePlan>>> {
    Ok(Json(svc::get_plan(&ctx, &caller).await?))
}

async fn put_plan(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<PutReleasePlan>,
) -> ApiResult<Json<ReleasePlan>> {
    Ok(Json(svc::put_plan(&ctx, &caller, body).await?))
}
