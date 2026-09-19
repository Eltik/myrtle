use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services;
use crate::app::services::tier_list::check_permission;
use crate::app::services::tier_list::get_by_slug;
use crate::app::services::tier_list::invalidate_detail;
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

/// One tier list with its tiers and placements.
#[utoipa::path(
    get,
    path = "/tier-lists/{slug}",
    operation_id = "tier_list_get",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    responses(
        (status = 200, description = "The list.", body = crate::app::services::tier_list::TierListDetail),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<services::tier_list::TierListDetail>, ApiError> {
    let detail = get_by_slug(&state, &slug).await?;
    Ok(Json(detail))
}

/// Every listed tier list.
#[utoipa::path(
    get,
    path = "/tier-lists",
    operation_id = "tier_lists_list",
    tag = "tier-lists",
    responses(
        (status = 200, description = "Public, listed tier lists.", body = Vec<TierList>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<TierList>>, ApiError> {
    let lists = find_all_active(&state.db, None).await?;
    Ok(Json(lists))
}

#[derive(Deserialize)]
pub struct ListDetailsQuery {
    pub limit: Option<i64>,
}

/// Every listed tier list, with its tiers and placements resolved.
#[utoipa::path(
    get,
    path = "/tier-lists/details",
    tag = "tier-lists",
    params(
        ("limit" = Option<i64>, Query, description = "Maximum lists to return.")
    ),
    responses(
        (status = 200, description = "Tier lists with their contents.", body = Vec<crate::app::services::tier_list::TierListDetail>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_details(
    State(state): State<AppState>,
    Query(params): Query<ListDetailsQuery>,
) -> Result<Json<Vec<services::tier_list::TierListDetail>>, ApiError> {
    let limit = params.limit.unwrap_or(60).clamp(1, 200);
    let details = services::tier_list::list_details(&state, limit).await?;
    Ok(Json(details))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub list_type: String,
}

/// Create a tier list. The caller becomes its owner.
#[utoipa::path(
    post,
    path = "/tier-lists",
    operation_id = "tier_list_create",
    tag = "tier-lists",
    request_body = CreateRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The created list.", body = TierList),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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

/// Tier lists the caller owns or can edit.
#[utoipa::path(
    get,
    path = "/tier-lists/mine",
    tag = "tier-lists",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The caller's lists.", body = Vec<TierList>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TierList>>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let lists = find_by_user(&state.db, user_id).await?;
    Ok(Json(lists))
}

/// Tier lists the caller has favourited.
#[utoipa::path(
    get,
    path = "/tier-lists/favorites",
    tag = "tier-lists",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Favourited lists.", body = Vec<TierList>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn favorites(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<TierList>>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let lists = find_favorited_by_user(&state.db, user_id).await?;
    Ok(Json(lists))
}

/// Delete a tier list and everything in it.
///
/// Owner only.
#[utoipa::path(
    delete,
    path = "/tier-lists/{slug}",
    operation_id = "tier_list_delete",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The list is gone.", body = crate::app::routes::StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    // Owner check is inside check_permission; Admin level covers delete.
    check_permission(&state, &list, user_id, auth.role, Permission::Admin).await?;
    delete_list(&state.db, list.id).await?;
    invalidate_detail(&state, &slug).await;
    Ok(ok_status())
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Rename a tier list or change its description.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    put,
    path = "/tier-lists/{slug}",
    operation_id = "tier_list_update",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = UpdateRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The list as stored.", body = TierList),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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
    invalidate_detail(&state, &slug).await;
    Ok(Json(list))
}
