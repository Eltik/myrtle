use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::{
    app::{
        error::ApiError, extractors::auth::AuthUser, routes::ok_status, services, state::AppState,
    },
    database::models::planner::{OperatorPlanResponse, PlanGroup, PlannerResponse},
};

#[derive(Deserialize)]
pub struct ListPlansQuery {
    pub active: Option<String>,
}

#[derive(Deserialize)]
pub struct UpsertPlanRequest {
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: serde_json::Value,
    pub target_modules: serde_json::Value,
    pub display_on_profile: bool,
    pub groups: Option<Vec<String>>,
}

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

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = auth.user_uuid()?;
    services::planner::delete_plan(&state, user_id, &operator_id).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct GroupRequest {
    pub name: String,
}

pub async fn create_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GroupRequest>,
) -> Result<Json<PlanGroup>, ApiError> {
    let user_id = auth.user_uuid()?;
    let group = services::planner::upsert_group(&state, user_id, None, &body.name).await?;
    Ok(Json(group))
}

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

pub async fn delete_group(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(group_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = auth.user_uuid()?;
    services::planner::delete_group(&state, user_id, &group_name).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct PublicPlansQuery {
    pub uid: String,
}

pub async fn list_public(
    State(state): State<AppState>,
    Query(query): Query<PublicPlansQuery>,
) -> Result<Json<Vec<OperatorPlanResponse>>, ApiError> {
    let profile = if let Some(p) = crate::database::queries::users::find_by_uid(&state.db, &query.uid).await? {
        p
    } else if let Ok(uuid) = uuid::Uuid::parse_str(&query.uid) {
        crate::database::queries::users::find_by_id(&state.db, uuid)
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
