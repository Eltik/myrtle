use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::{
    app::{
        error::ApiError, extractors::auth::AuthUser, routes::ok_status, services, state::AppState,
    },
    database::models::planner::OperatorPlanResponse,
};

#[derive(Deserialize)]
pub struct UpsertPlanRequest {
    pub target_elite: i16,
    pub target_level: i16,
    pub target_skill_level: i16,
    pub target_skills: serde_json::Value,
    pub target_modules: serde_json::Value,
    pub display_on_profile: bool,
}

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<OperatorPlanResponse>>, ApiError> {
    let user_id = auth.user_uuid()?;
    let plans = services::planner::list_plans(&state, user_id).await?;
    Ok(Json(plans))
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
