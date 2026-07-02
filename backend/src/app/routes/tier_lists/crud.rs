use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::services;
use crate::app::services::tier_list::check_permission;
use crate::app::services::tier_list::get_by_slug;
use crate::app::services::tier_list::update_list;
use crate::app::state::AppState;
use crate::app::validation::{
    LIST_DESCRIPTION_MAX, LIST_NAME_MAX, validate_length, validate_opt_length,
};
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierList;
use crate::database::queries::tier_lists::delete_list;
use crate::database::queries::tier_lists::find_all_active;
use crate::database::queries::tier_lists::find_by_user;
use crate::database::queries::tier_lists::find_favorited_by_user;

fn validate_list_body(name: &str, description: Option<&str>) -> Result<(), ApiError> {
    validate_length("list name", name, LIST_NAME_MAX)?;
    validate_opt_length("list description", description, LIST_DESCRIPTION_MAX)?;
    Ok(())
}

pub async fn get(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<services::tier_list::TierListDetail>, ApiError> {
    let detail = get_by_slug(&state, &slug).await?;
    Ok(Json(detail))
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TierList>>, ApiError> {
    let lists = find_all_active(&state.db, None).await?;
    Ok(Json(lists))
}

#[derive(Deserialize)]
pub struct ListDetailsQuery {
    pub limit: Option<i64>,
}

pub async fn list_details(
    State(state): State<AppState>,
    Query(params): Query<ListDetailsQuery>,
) -> Result<Json<Vec<services::tier_list::TierListDetail>>, ApiError> {
    let limit = params.limit.unwrap_or(60).clamp(1, 200);
    let details = services::tier_list::list_details(&state, limit).await?;
    Ok(Json(details))
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
    validate_list_body(&body.name, body.description.as_deref())?;
    let user_id: Uuid = auth.user_uuid()?;
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
    let user_id: Uuid = auth.user_uuid()?;
    let lists = find_by_user(&state.db, user_id).await?;
    Ok(Json(lists))
}

pub async fn favorites(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TierList>>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let lists = find_favorited_by_user(&state.db, user_id).await?;
    Ok(Json(lists))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    // Owner check is inside check_permission; Admin level covers delete.
    check_permission(&state, &list, user_id, auth.role, Permission::Admin).await?;
    delete_list(&state.db, list.id).await?;
    Ok(ok_status())
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
    validate_list_body(&body.name, body.description.as_deref())?;
    let user_id: Uuid = auth.user_uuid()?;
    let list = update_list(
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
