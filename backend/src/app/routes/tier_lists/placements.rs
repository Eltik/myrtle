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
use crate::app::validation::{PLACEMENT_DESCRIPTION_MAX, validate_opt_length};
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierPlacement;
use crate::database::queries::tier_lists::add_placement;
use crate::database::queries::tier_lists::get_placements;
use crate::database::queries::tier_lists::get_tiers;
use crate::database::queries::tier_lists::move_placement;
use crate::database::queries::tier_lists::remove_placement;
use crate::database::queries::tier_lists::set_placement_description;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AddPlacementRequest {
    pub tier_id: Uuid,
    pub operator_id: String,
    pub sub_order: Option<i16>,
    pub description: Option<String>,
}

/// Place an operator into a tier.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/placements",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = AddPlacementRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The placement.", body = TierPlacement),
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
pub async fn add(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<AddPlacementRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    validate_opt_length(
        "placement description",
        body.description.as_deref(),
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    // `body.tier_id` comes from the request and the permission was checked
    // against `slug`, so the insert is scoped to `list.id`: a tier on another
    // list is not found.
    let placement = add_placement(
        &state.db,
        list.id,
        body.tier_id,
        &body.operator_id,
        body.sub_order.unwrap_or(0),
        body.description.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(placement))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateDescriptionRequest {
    pub description: Option<String>,
}

/// Change the note attached to one placement.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    patch,
    path = "/tier-lists/{slug}/placements/{operator_id}",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`.")
    ),
    request_body = UpdateDescriptionRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The placement as stored.", body = TierPlacement),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update_description(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
    Json(body): Json<UpdateDescriptionRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    validate_opt_length(
        "placement description",
        body.description.as_deref(),
        PLACEMENT_DESCRIPTION_MAX,
    )?;
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let placement = set_placement_description(
        &state.db,
        list.id,
        &operator_id,
        body.description.as_deref(),
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(placement))
}

/// Take an operator out of the list.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    delete,
    path = "/tier-lists/{slug}/placements/{operator_id}",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The placement is gone.", body = crate::app::routes::StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn remove(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    // Find which tier this operator is in, then remove. Idempotent: if the
    // placement is already gone (e.g. cascade-deleted by a prior tier delete in
    // the same save batch), report success rather than NotFound.
    let tiers = get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            remove_placement(&state.db, list.id, tier.id, &operator_id).await?;
            invalidate_detail(&state, &slug).await;
            break;
        }
    }
    Ok(ok_status())
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MovePlacementRequest {
    pub new_tier_id: Uuid,
    pub sub_order: Option<i16>,
}

/// Move an operator to a different tier, or reorder it within one.
/// Needs edit rights on the list: its owner, or a grant from
/// `/tier-lists/{slug}/permissions`.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/placements/{operator_id}/move",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("operator_id" = String, Path, description = "Operator id, e.g. `char_002_amiya`.")
    ),
    request_body = MovePlacementRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The placement in its new position.", body = TierPlacement),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn move_to(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, operator_id)): Path<(String, String)>,
    Json(body): Json<MovePlacementRequest>,
) -> Result<Json<TierPlacement>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let tiers = get_tiers(&state.db, list.id).await?;
    for tier in &tiers {
        let placements = get_placements(&state.db, tier.id).await?;
        if placements.iter().any(|p| p.operator_id == operator_id) {
            // `body.new_tier_id` comes from the request, so the move is
            // scoped to `list.id` and cannot place the operator on another
            // list. `None` is that case, with nothing deleted.
            let result = move_placement(
                &state.db,
                list.id,
                tier.id,
                body.new_tier_id,
                &operator_id,
                body.sub_order.unwrap_or(0),
            )
            .await?
            .ok_or_else(|| {
                ApiError::BadRequest("destination tier is not on this tier list".into())
            })?;
            invalidate_detail(&state, &slug).await;
            return Ok(Json(result));
        }
    }
    Err(ApiError::NotFound)
}
