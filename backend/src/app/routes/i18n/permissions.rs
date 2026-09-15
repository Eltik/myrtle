use std::str::FromStr;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
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

pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListQuery>,
) -> Result<Json<Vec<TranslationPermission>>, ApiError> {
    if !auth.role.can_access_admin_panel() {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(
        service::list_permissions(&state, params.locale.as_deref()).await?,
    ))
}

#[derive(Deserialize)]
pub struct GrantRequest {
    pub locale: String,
    pub user_id: Uuid,
    pub permission: String,
}

pub async fn grant(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let granted_by = assert_can_manage(&state, &auth, &body.locale).await?;
    let permission = Permission::from_str(&body.permission).map_err(ApiError::BadRequest)?;
    service::grant_permission(&state, &body.locale, body.user_id, permission, granted_by).await?;
    Ok(ok_status())
}

pub async fn revoke(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<GrantRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    assert_can_manage(&state, &auth, &body.locale).await?;
    let permission = Permission::from_str(&body.permission).map_err(ApiError::BadRequest)?;
    service::revoke_permission(&state, &body.locale, body.user_id, permission).await?;
    Ok(ok_status())
}
