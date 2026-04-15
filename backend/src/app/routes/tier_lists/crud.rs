use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;
use crate::database::models::tier_list::TierList;
use crate::database::queries::tier_lists as queries;

pub async fn get(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<services::tier_list::TierListDetail>, ApiError> {
    let detail = services::tier_list::get_by_slug(&state, &slug).await?;
    Ok(Json(detail))
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TierList>>, ApiError> {
    let lists = queries::find_all_active(&state.db, None).await?;
    Ok(Json(lists))
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub list_type: String,
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateRequest>,
) -> Result<Json<TierList>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = services::tier_list::create(
        &state,
        user_id,
        auth.role,
        &body.name,
        body.description.as_deref(),
        &body.list_type,
    )
    .await?;
    Ok(Json(list))
}

pub async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TierList>>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let lists = queries::find_by_user(&state.db, user_id).await?;
    Ok(Json(lists))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    // Owner check is inside check_permission; Admin level covers delete.
    services::tier_list::check_permission(
        &state,
        &list,
        user_id,
        auth.role,
        crate::core::auth::permissions::Permission::Admin,
    )
    .await?;
    queries::delete_list(&state.db, list.id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<TierList>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = services::tier_list::update_list(
        &state,
        &slug,
        user_id,
        auth.role,
        &body.name,
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(list))
}
