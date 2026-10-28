use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::RoleCheckedUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services::content_audit;
use crate::app::services::tier_list::find_and_authorize;
use crate::app::services::tier_list::invalidate_detail;
use crate::app::state::AppState;
use crate::app::validation::{
    TIER_DESCRIPTION_MAX, TIER_NAME_MAX, validate_hex_color, validate_length, validate_opt_length,
};
use crate::core::auth::permissions::Permission;
use crate::database::models::content_audit::{
    AuditTierRef, AuditTierRename, ContentAuditAction, ContentAuditDetails,
};
use crate::database::models::tier_list::Tier;
use crate::database::queries::tier_lists::create_tier;
use crate::database::queries::tier_lists::delete_tier;
use crate::database::queries::tier_lists::get_placements;
use crate::database::queries::tier_lists::get_tiers;
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
    security(("bearer_auth" = [])),
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
    auth: RoleCheckedUser,
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
    let details = ContentAuditDetails {
        tiers_added: Some(vec![tier_ref(&tier)]),
        ..Default::default()
    };
    content_audit::record(
        &state,
        content_audit::tier_list_entry(&list, user_id, ContentAuditAction::Edit, details),
    )
    .await;
    Ok(Json(tier))
}

fn tier_ref(tier: &Tier) -> AuditTierRef {
    AuditTierRef {
        id: tier.id,
        name: tier.name.clone(),
    }
}

/// What an update changed, as an `edit` row's details. The editor reorders
/// by parking a tier at `10000 + i` and settling it, two requests the run
/// folds into one reorder.
fn tier_update_details(before: &Tier, after: &Tier) -> ContentAuditDetails {
    let changed = |yes: bool| yes.then(|| vec![tier_ref(after)]);
    ContentAuditDetails {
        tiers_renamed: (before.name != after.name).then(|| {
            vec![AuditTierRename {
                id: after.id,
                from: before.name.clone(),
                to: after.name.clone(),
            }]
        }),
        tiers_reordered: changed(before.display_order != after.display_order),
        tiers_restyled: changed(
            before.color != after.color || before.description != after.description,
        ),
        ..Default::default()
    }
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
    security(("bearer_auth" = [])),
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
    auth: RoleCheckedUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    validate_tier_body(&body)?;

    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;
    let before = get_tiers(&state.db, list.id)
        .await?
        .into_iter()
        .find(|t| t.id == tier_id);

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
    if let Some(before) = before {
        content_audit::record(
            &state,
            content_audit::tier_list_entry(
                &list,
                user_id,
                ContentAuditAction::Edit,
                tier_update_details(&before, &tier),
            ),
        )
        .await;
    }
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
    security(("bearer_auth" = [])),
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
    auth: RoleCheckedUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Admin).await?;
    let before = get_tiers(&state.db, list.id)
        .await?
        .into_iter()
        .find(|t| t.id == tier_id);
    let dropped = match &before {
        Some(tier) => get_placements(&state.db, tier.id).await?.len(),
        None => 0,
    };

    if !delete_tier(&state.db, list.id, tier_id).await? {
        return Err(ApiError::NotFound);
    }
    invalidate_detail(&state, &slug).await;
    if let Some(tier) = before {
        let details = ContentAuditDetails {
            tiers_deleted: Some(vec![tier_ref(&tier)]),
            entries_dropped: (dropped > 0).then(|| content_audit::count_of(dropped)),
            ..Default::default()
        };
        content_audit::record(
            &state,
            content_audit::tier_list_entry(&list, user_id, ContentAuditAction::Edit, details),
        )
        .await;
    }
    Ok(ok_status())
}
