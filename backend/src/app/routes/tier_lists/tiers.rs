use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services::tier_list::find_and_authorize;
use crate::app::services::tier_list::invalidate_detail;
use crate::app::state::AppState;
use crate::app::validation::{
    TIER_DESCRIPTION_MAX, TIER_NAME_MAX, validate_hex_color, validate_length, validate_opt_length,
};
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::Tier;
use crate::database::queries::tier_lists::create_tier;
use crate::database::queries::tier_lists::delete_tier;
use crate::database::queries::tier_lists::update_tier;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateTierRequest {
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

fn validate_tier_body(body: &CreateTierRequest) -> Result<(), ApiError> {
    validate_length("tier name", &body.name, TIER_NAME_MAX)?;
    validate_opt_length(
        "tier description",
        body.description.as_deref(),
        TIER_DESCRIPTION_MAX,
    )?;
    validate_hex_color(body.color.as_deref())?;
    Ok(())
}

/// Add a tier to a list.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/tiers",
    operation_id = "tier_create",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = CreateTierRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The created tier.", body = Tier),
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
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    validate_tier_body(&body)?;

    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let tier = create_tier(
        &state.db,
        list.id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(tier))
}

/// Rename, recolour or reorder a tier.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    put,
    path = "/tier-lists/{slug}/tiers/{tier_id}",
    operation_id = "tier_update",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("tier_id" = String, Path, description = "Tier id (UUID).")
    ),
    request_body = CreateTierRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The tier as stored.", body = Tier),
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
    Path((slug, tier_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    validate_tier_body(&body)?;

    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    // `tier_id` comes from the request and the permission was checked against
    // `slug`, so the write is scoped to `list.id`: a tier on another list is
    // not found.
    let tier = update_tier(
        &state.db,
        list.id,
        tier_id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(tier))
}

/// Delete a tier and the placements in it.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    delete,
    path = "/tier-lists/{slug}/tiers/{tier_id}",
    operation_id = "tier_delete",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("tier_id" = String, Path, description = "Tier id (UUID).")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The tier is gone.", body = crate::app::routes::StatusOk),
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
    Path((slug, tier_id)): Path<(String, Uuid)>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Admin).await?;

    if !delete_tier(&state.db, list.id, tier_id).await? {
        return Err(ApiError::NotFound);
    }
    invalidate_detail(&state, &slug).await;
    Ok(ok_status())
}
