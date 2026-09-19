use std::str::FromStr;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services::i18n as service;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::i18n::TranslationPermission;

/// Who may hand out locales. `Permission::Admin` on a locale makes a locale
/// lead able to recruit within their own language without being a site
/// super-admin - which is the whole point of keeping grants in a table.
async fn assert_can_manage(
    state: &AppState,
    auth: &AuthUser,
    locale: &str,
) -> Result<Uuid, ApiError> {
    service::assert_can_write(state, auth, locale, Permission::Admin).await
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub locale: Option<String>,
}

/// Who holds which translation grant.
#[utoipa::path(
    get,
    path = "/admin/i18n/permissions",
    operation_id = "i18n_permissions_list",
    tag = "i18n-admin",
    params(
        ("locale" = Option<String>, Query, description = "Restrict to one locale.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Current grants.", body = Vec<TranslationPermission>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<TranslationPermission>>, ApiError> {
    if !service::can_access_admin_panel(&state, &auth).await? {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(
        service::list_permissions(&state, params.locale.as_deref()).await?,
    ))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct GrantRequest {
    pub locale: String,
    pub user_id: Uuid,
    pub permission: String,
}

/// Give an account a translation grant on one locale.
///
/// Holding any grant is on its own enough to open the admin panel, since a
/// grant would be unusable otherwise.
#[utoipa::path(
    post,
    path = "/admin/i18n/permissions",
    operation_id = "i18n_permission_grant",
    tag = "i18n-admin",
    request_body = GrantRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The grant was recorded.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn grant(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GrantRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    let granted_by = assert_can_manage(&state, &auth, &body.locale).await?;
    let permission = Permission::from_str(&body.permission).map_err(ApiError::BadRequest)?;
    service::grant_permission(&state, &body.locale, body.user_id, permission, granted_by).await?;
    Ok(ok_status())
}

/// Withdraw one translation grant. Idempotent.
#[utoipa::path(
    post,
    path = "/admin/i18n/permissions/revoke",
    operation_id = "i18n_permission_revoke",
    tag = "i18n-admin",
    request_body = GrantRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The grant is gone.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GrantRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    assert_can_manage(&state, &auth, &body.locale).await?;
    let permission = Permission::from_str(&body.permission).map_err(ApiError::BadRequest)?;
    service::revoke_permission(&state, &body.locale, body.user_id, permission).await?;
    Ok(ok_status())
}
