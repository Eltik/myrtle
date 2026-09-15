use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::services::i18n as service;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::i18n::{
    GamedataOverride, Locale, TranslationEntry, UiMessageAuditEntry,
};

/// Reading the translation workspace. Open to any staff role rather than to
/// translators alone, so a tier-list admin can see progress without being
/// granted a locale. Writes are gated per locale in the service.
const fn assert_can_read(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.role.can_access_admin_panel() {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

// ---------------------------------------------------------------- locales

pub async fn list_locales(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Locale>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(service::list_locales(&state, false).await?))
}

/// The locales this caller may actually write, for the editor's locale picker.
pub async fn writable_locales(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<String>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(service::writable_locales(&state, &auth).await?))
}

#[derive(Deserialize)]
pub struct UpsertLocaleRequest {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub fallback_locale: Option<String>,
    pub gamedata_server: String,
    pub enabled: bool,
    pub sort_order: Option<i32>,
}

/// Adding or enabling a locale is a super-admin action: enabling one commits
/// the deployment to a game-data region and a font subset, which is an
/// infrastructure decision rather than a translation one.
pub async fn upsert_locale(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpsertLocaleRequest>,
) -> Result<Json<Locale>, ApiError> {
    if !auth.role.is_super_admin() {
        return Err(ApiError::Forbidden);
    }
    let locale = crate::database::queries::i18n::upsert_locale(
        &state.db,
        &body.code,
        &body.english_name,
        &body.native_name,
        body.fallback_locale.as_deref(),
        &body.gamedata_server,
        body.enabled,
        body.sort_order.unwrap_or(100),
    )
    .await?;
    state.cache.invalidate_by_prefix("i18n:").await;
    Ok(Json(locale))
}

// ---------------------------------------------------------------- messages

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    pub locale: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    /// `all` | `untranslated` | `stale` | `translated`
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

pub async fn list_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListMessagesQuery>,
) -> Result<Json<service::TranslationListResponse>, ApiError> {
    assert_can_read(&auth)?;
    let response = service::list_entries(
        &state,
        &service::ListParams {
            locale: &params.locale,
            namespace: params.namespace.as_deref().unwrap_or(""),
            search: params.search.as_deref().unwrap_or(""),
            filter: params.filter.as_deref().unwrap_or("all"),
            limit: params.limit.unwrap_or(100),
            offset: params.offset.unwrap_or(0),
        },
    )
    .await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct UpdateMessageRequest {
    pub locale: String,
    pub key: String,
    pub value: String,
}

/// `PUT /admin/i18n/message`
///
/// The key travels in the body rather than the path: message keys are dotted
/// and arbitrary, and path-encoding them buys nothing.
pub async fn update_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateMessageRequest>,
) -> Result<Json<TranslationEntry>, ApiError> {
    let user_id = service::assert_can_write(&state, &auth, &body.locale, Permission::Edit).await?;
    let entry =
        service::update_message(&state, &body.locale, &body.key, &body.value, user_id).await?;
    Ok(Json(entry))
}

#[derive(Deserialize)]
pub struct ClearMessageRequest {
    pub locale: String,
    pub key: String,
}

pub async fn clear_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ClearMessageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = service::assert_can_write(&state, &auth, &body.locale, Permission::Edit).await?;
    service::clear_message(&state, &body.locale, &body.key, user_id).await?;
    Ok(ok_status())
}

// ---------------------------------------------------------------- progress + audit

/// `GET /admin/i18n/namespaces` - the full namespace list, independent of
/// whatever page of messages the editor happens to be showing.
pub async fn namespaces(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<String>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(service::list_namespaces(&state).await?))
}

pub async fn progress(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<service::LocaleProgress>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(service::locale_progress(&state).await?))
}

#[derive(Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<AuditQuery>,
) -> Result<Json<service::TranslationAuditResponse>, ApiError> {
    assert_can_read(&auth)?;
    let response = service::get_global_audit_log(
        &state,
        params.locale.as_deref(),
        params.limit.unwrap_or(100),
        params.before,
    )
    .await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct EntryAuditQuery {
    pub locale: String,
    pub key: String,
}

/// Per-key history. This is what the editor's revert button reads: every entry
/// carries the value it replaced, so reverting is an ordinary write of
/// `old_value` rather than a special restore path.
pub async fn entry_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<EntryAuditQuery>,
) -> Result<Json<Vec<UiMessageAuditEntry>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(
        service::get_audit_log(&state, &params.key, &params.locale).await?,
    ))
}

// ---------------------------------------------------------------- sync

#[derive(Deserialize)]
pub struct SyncRequest {
    pub entries: Vec<service::SourceEntry>,
}

/// `POST /admin/i18n/sync` - the extractor's output becomes the truth about
/// which keys exist and what their English source is. Super-admin only: it can
/// deactivate every key in the catalog.
pub async fn sync(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<SyncRequest>,
) -> Result<Json<service::SyncResult>, ApiError> {
    if !auth.role.is_super_admin() {
        return Err(ApiError::Forbidden);
    }
    Ok(Json(
        service::sync_source_catalog(&state, &body.entries).await?,
    ))
}

// ---------------------------------------------------------------- overrides

#[derive(Deserialize)]
pub struct OverridesQuery {
    pub locale: String,
}

pub async fn list_overrides(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<OverridesQuery>,
) -> Result<Json<Vec<GamedataOverride>>, ApiError> {
    assert_can_read(&auth)?;
    Ok(Json(service::list_overrides(&state, &params.locale).await?))
}

#[derive(Deserialize)]
pub struct UpsertOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
}

pub async fn put_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpsertOverrideRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = service::assert_can_write(&state, &auth, &body.locale, Permission::Edit).await?;
    service::upsert_override(
        &state,
        &service::OverrideInput {
            locale: &body.locale,
            kind: &body.kind,
            entity_id: &body.entity_id,
            field: &body.field,
            value: &body.value,
        },
        user_id,
    )
    .await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct DeleteOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
}

pub async fn delete_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<DeleteOverrideRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    service::assert_can_write(&state, &auth, &body.locale, Permission::Edit).await?;
    service::delete_override(
        &state,
        &body.locale,
        &body.kind,
        &body.entity_id,
        &body.field,
    )
    .await?;
    Ok(ok_status())
}
