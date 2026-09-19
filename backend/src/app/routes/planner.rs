use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    app::{
        error::ApiError,
        extractors::auth::AuthUser,
        routes::{StatusOk, ok_status},
        services,
        state::AppState,
    },
    database::{
        models::planner::{OperatorPlanResponse, PlanGroup, PlannerResponse},
        queries::users::{find_by_id, find_by_uid},
    },
};

#[derive(Deserialize)]
pub struct ListPlansQuery {
    pub active: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpsertPlanRequest {
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: serde_json::Value,
    pub target_modules: serde_json::Value,
    pub display_on_profile: bool,
    pub groups: Option<Vec<String>>,
}

/// The caller's own operator plans.
#[utoipa::path(
    get,
    path = "/plans",
    operation_id = "plans_list",
    tag = "planner",
    params(
        ("active" = Option<String>, Query, description = "Restrict to one group name, or `all` for every plan.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Plans and their groups.", body = PlannerResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListPlansQuery>,
    auth: AuthUser,
) -> Result<Json<PlannerResponse>, ApiError> {
    let user_id = auth.user_uuid()?;
    let active_ids: Vec<String> = query
        .active
        .as_ref()
        .map(|s| {
            s.split(',')
                .map(|id| id.trim().to_owned())
                .filter(|id| !id.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let response = services::planner::list_plans(&state, user_id, active_ids).await?;
    Ok(Json(response))
}

/// Create or replace the caller's plan for one operator.
#[utoipa::path(
    post,
    path = "/plan/{operator_id}",
    tag = "planner",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    request_body = UpsertPlanRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The stored plan.", body = OperatorPlanResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn upsert(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
    Json(body): Json<UpsertPlanRequest>,
) -> Result<Json<OperatorPlanResponse>, ApiError> {
    let user_id = auth.user_uuid()?;
    let plan = services::planner::upsert_plan(
        &state,
        user_id,
        &operator_id,
        body.target_elite,
        body.target_level,
        body.target_skill_level,
        body.target_skills,
        body.target_modules,
        body.display_on_profile,
        body.groups,
    )
    .await?;
    Ok(Json(plan))
}

/// Remove the caller's plan for one operator.
#[utoipa::path(
    delete,
    path = "/plan/{operator_id}",
    operation_id = "plan_delete",
    tag = "planner",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The plan is gone. Deleting twice is not an error.", body = StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id = auth.user_uuid()?;
    services::planner::delete_plan(&state, user_id, &operator_id).await?;
    Ok(ok_status())
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct GroupRequest {
    pub name: String,
}

/// Create a named group to file plans under.
#[utoipa::path(
    post,
    path = "/plan/group",
    tag = "planner",
    request_body = GroupRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The created group.", body = PlanGroup),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn create_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GroupRequest>,
) -> Result<Json<PlanGroup>, ApiError> {
    let user_id = auth.user_uuid()?;
    let group = services::planner::upsert_group(&state, user_id, None, &body.name).await?;
    Ok(Json(group))
}

/// Rename a plan group.
#[utoipa::path(
    put,
    path = "/plan/group/{group_name}",
    tag = "planner",
    params(
        ("group_name" = String, Path, description = "Current group name.")
    ),
    request_body = GroupRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The group as stored.", body = PlanGroup),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_name): Path<String>,
    Json(body): Json<GroupRequest>,
) -> Result<Json<PlanGroup>, ApiError> {
    let user_id = auth.user_uuid()?;
    let group =
        services::planner::upsert_group(&state, user_id, Some(&group_name), &body.name).await?;
    Ok(Json(group))
}

/// Delete a plan group. The plans filed under it are kept.
#[utoipa::path(
    delete,
    path = "/plan/group/{group_name}",
    tag = "planner",
    params(
        ("group_name" = String, Path, description = "Group name.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The group is gone.", body = StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_name): Path<String>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id = auth.user_uuid()?;
    services::planner::delete_group(&state, user_id, &group_name).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct PublicPlansQuery {
    pub uid: String,
}

/// Another player's plans, where they chose to show them on their profile.
#[utoipa::path(
    get,
    path = "/plans/public",
    tag = "planner",
    params(
        ("uid" = String, Query, description = "Player whose public plans to read.")
    ),
    responses(
        (status = 200, description = "Plans the player displays publicly.", body = Vec<OperatorPlanResponse>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_public(
    State(state): State<AppState>,
    Query(query): Query<PublicPlansQuery>,
) -> Result<Json<Vec<OperatorPlanResponse>>, ApiError> {
    let profile = if let Some(p) = find_by_uid(&state.db, &query.uid).await? {
        p
    } else if let Ok(uuid) = Uuid::parse_str(&query.uid) {
        find_by_id(&state.db, uuid)
            .await?
            .ok_or(ApiError::NotFound)?
    } else {
        return Err(ApiError::NotFound);
    };

    if profile.public_profile != Some(true) {
        return Err(ApiError::Forbidden);
    }

    let response = services::planner::list_public_plans(&state, profile.id).await?;
    Ok(Json(response))
}
