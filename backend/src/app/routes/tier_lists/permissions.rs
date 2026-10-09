use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::state::AppState;
use crate::database::models::tier_list::{GrantedTierList, TierListGrant, TierListPermission};
use crate::database::queries::tier_lists::get_all_grants;
use crate::database::queries::tier_lists::get_granted_lists;
use crate::database::queries::tier_lists::get_permissions;
use crate::database::queries::tier_lists::grant_permission;
use crate::database::queries::tier_lists::revoke_permission;

/// Who may edit this list.
///
/// Owner only.
#[utoipa::path(
    get,
    path = "/tier-lists/{slug}/permissions",
    operation_id = "tier_list_permissions_list",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Current grants.", body = Vec<TierListPermission>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TierListPermission>>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = super::load_tier_list(&state, &slug).await?;

    let perms = get_permissions(&state.db, tier_list.id).await?;
    Ok(Json(perms))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct GrantRequest {
    pub user_id: Uuid,
    pub permission: String,
}

/// Give another account a permission on this list.
///
/// Owner only.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/permissions",
    operation_id = "tier_list_permission_grant",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = GrantRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The grant was recorded.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn grant(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<GrantRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let granter_id: Uuid = auth.user_uuid()?;
    let tier_list = super::load_tier_list(&state, &slug).await?;

    grant_permission(
        &state.db,
        tier_list.id,
        body.user_id,
        &body.permission,
        granter_id,
    )
    .await?;
    Ok(ok_status())
}

/// Withdraw one permission from one account.
///
/// Owner only. Idempotent.
#[utoipa::path(
    delete,
    path = "/tier-lists/{slug}/permissions/{user_id}/{permission}",
    operation_id = "tier_list_permission_revoke",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL."),
        ("user_id" = String, Path, description = "Account id (UUID) to revoke from."),
        ("permission" = String, Path, description = "Permission name to withdraw.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The grant is gone.", body = crate::app::routes::StatusOk),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, target_user_id, permission)): Path<(String, Uuid, String)>,
) -> Result<Json<StatusOk>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let tier_list = super::load_tier_list(&state, &slug).await?;

    revoke_permission(&state.db, tier_list.id, target_user_id, &permission).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct AllGrantsQuery {
    /// Only this grantee's grants.
    pub user_id: Option<Uuid>,
}

/// Every grant on every active list, with the list, grantee and granter named.
///
/// Tier list admin or super-admin.
#[utoipa::path(
    get,
    path = "/admin/tier-lists/permissions",
    tag = "admin",
    params(
        ("user_id" = Option<String>, Query, description = "Only this account's grants (UUID).")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Grants, newest first.", body = Vec<TierListGrant>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_all(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<AllGrantsQuery>,
) -> Result<Json<Vec<TierListGrant>>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let grants = get_all_grants(&state.db, params.user_id).await?;
    Ok(Json(grants))
}

/// The lists the caller holds a grant on, at their highest level on each.
///
/// Any signed-in account. Lists the caller created but holds no grant on are
/// not here; `/tier-lists/mine` has those.
#[utoipa::path(
    get,
    path = "/tier-lists/granted",
    tag = "tier-lists",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Granted lists, by title.", body = Vec<GrantedTierList>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn granted(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<GrantedTierList>>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let lists = get_granted_lists(&state.db, user_id).await?;
    Ok(Json(lists))
}
