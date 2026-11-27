use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, post, put};
use myrtle_app::AppContext;
use myrtle_app::services::planner::{
    self, DeletePlansResponse, OperatorPlanResponse, PlanGroup, PlanPreset, PlannerResponse,
    UpsertPlanRequest,
};
use myrtle_engine_planner::PresetTarget;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{self, Caller, MaybeCaller};
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/plans", get(list))
        .route("/plans/public", get(list_public))
        .route("/plans/delete", post(delete_many))
        .route("/plan/{operator_id}", post(upsert).delete(delete_plan))
        .route("/plan/group", post(create_group))
        .route(
            "/plan/group/{group_name}",
            put(update_group).delete(delete_group),
        )
        .route("/plan/presets", get(list_presets))
        .route("/plan/preset", post(upsert_preset))
        .route("/plan/preset/{preset_name}", delete(delete_preset))
}

#[derive(Deserialize)]
struct ListPlansQuery {
    active: Option<String>,
    max_tier: Option<i16>,
    by_operator: Option<bool>,
}

async fn list(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<ListPlansQuery>,
) -> ApiResult<Json<PlannerResponse>> {
    let response = planner::list_plans(
        &ctx,
        &caller,
        query.active.as_deref(),
        query.max_tier,
        query.by_operator.unwrap_or(false),
    )
    .await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
struct PublicPlansQuery {
    uid: String,
}

async fn list_public(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<PublicPlansQuery>,
) -> ApiResult<Json<Vec<OperatorPlanResponse>>> {
    Ok(Json(
        planner::list_public_plans(&ctx, &caller, &query.uid).await?,
    ))
}

async fn upsert(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(operator_id): Path<String>,
    extract::Json(body): extract::Json<UpsertPlanRequest>,
) -> ApiResult<Json<OperatorPlanResponse>> {
    Ok(Json(
        planner::upsert_plan(&ctx, &caller, &operator_id, body).await?,
    ))
}

async fn delete_plan(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(operator_id): Path<String>,
) -> ApiResult<Json<StatusOk>> {
    planner::delete_plan(&ctx, &caller, &operator_id).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct DeletePlansRequest {
    operator_ids: Vec<String>,
}

async fn delete_many(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<DeletePlansRequest>,
) -> ApiResult<Json<DeletePlansResponse>> {
    Ok(Json(
        planner::delete_plans(&ctx, &caller, &body.operator_ids).await?,
    ))
}

#[derive(Deserialize)]
struct GroupRequest {
    name: String,
}

async fn create_group(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<GroupRequest>,
) -> ApiResult<Json<PlanGroup>> {
    Ok(Json(
        planner::create_group(&ctx, &caller, &body.name).await?,
    ))
}

#[derive(Deserialize)]
struct UpdateGroupRequest {
    name: Option<String>,
    pinned: Option<bool>,
}

async fn update_group(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(group_name): Path<String>,
    extract::Json(body): extract::Json<UpdateGroupRequest>,
) -> ApiResult<Json<PlanGroup>> {
    let group = planner::update_group(
        &ctx,
        &caller,
        &group_name,
        body.name.as_deref(),
        body.pinned,
    )
    .await?;
    Ok(Json(group))
}

async fn delete_group(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(group_name): Path<String>,
) -> ApiResult<Json<StatusOk>> {
    planner::delete_group(&ctx, &caller, &group_name).await?;
    Ok(ok())
}

async fn list_presets(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<PlanPreset>>> {
    Ok(Json(planner::list_presets(&ctx, &caller).await?))
}

#[derive(Deserialize)]
struct PresetRequest {
    name: String,
    target: PresetTarget,
}

async fn upsert_preset(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<PresetRequest>,
) -> ApiResult<Json<PlanPreset>> {
    Ok(Json(
        planner::upsert_preset(&ctx, &caller, &body.name, &body.target).await?,
    ))
}

async fn delete_preset(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(preset_name): Path<String>,
) -> ApiResult<Json<StatusOk>> {
    planner::delete_preset(&ctx, &caller, &preset_name).await?;
    Ok(ok())
}
