use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services::i18n as service;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::i18n::{
    GamedataOverride, Locale, TranslationEntry, UiMessageAuditEntry,
};

/// Reading the translation workspace. Open to any staff role rather than to
/// translators alone, so a tier-list admin can see progress without being
/// granted a locale - and to any grant holder whatever their global role, so a
/// locale grant is usable by itself. Writes are gated per locale in the
/// service.
async fn assert_can_read(state: &AppState, auth: &AuthUser) -> Result<(), ApiError> {
    if service::can_access_admin_panel(state, auth).await? {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

// ---------------------------------------------------------------- locales

/// Every locale, enabled or not.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    get,
    path = "/admin/i18n/locales",
    tag = "i18n-admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "All locales.", body = Vec<Locale>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_locales(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Locale>>, ApiError> {
    assert_can_read(&state, &auth).await?;
    Ok(Json(service::list_locales(&state, false).await?))
}

/// The locales this caller may actually write, for the editor's locale picker.
///
/// The workspace uses this to decide what to render read-only.
#[utoipa::path(
    get,
    path = "/admin/i18n/writable-locales",
    tag = "i18n-admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Locale codes the caller can edit.", body = Vec<String>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn writable_locales(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<String>>, ApiError> {
    assert_can_read(&state, &auth).await?;
    Ok(Json(service::writable_locales(&state, &auth).await?))
}

#[derive(Deserialize, utoipa::ToSchema)]
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
#[utoipa::path(
    put,
    path = "/admin/i18n/locales",
    tag = "i18n-admin",
    request_body = UpsertLocaleRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The locale as stored.", body = Locale),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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

/// UI messages for one locale, filtered and paged.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    get,
    path = "/admin/i18n/messages",
    tag = "i18n-admin",
    params(
        ("locale" = String, Query, description = "Locale to read."),
        ("namespace" = Option<String>, Query, description = "Restrict to one namespace."),
        ("search" = Option<String>, Query, description = "Free-text filter over key and value."),
        ("filter" = Option<String>, Query, description = "`all`, `untranslated`, `stale` or `translated`."),
        ("limit" = Option<u32>, Query, description = "Page size."),
        ("offset" = Option<u32>, Query, description = "Rows to skip.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "One page of messages with their translation state.", body = crate::app::services::i18n::TranslationListResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_messages(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<ListMessagesQuery>,
) -> Result<Json<service::TranslationListResponse>, ApiError> {
    assert_can_read(&state, &auth).await?;
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateMessageRequest {
    pub locale: String,
    pub key: String,
    pub value: String,
}

/// `PUT /admin/i18n/message`
///
/// The key travels in the body rather than the path: message keys are dotted
/// and arbitrary, and path-encoding them buys nothing.
///
/// Direct edit with no review step; every write lands in the audit log.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    put,
    path = "/admin/i18n/message",
    tag = "i18n-admin",
    request_body = UpdateMessageRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The entry as stored.", body = TranslationEntry),
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ClearMessageRequest {
    pub locale: String,
    pub key: String,
}

/// Remove one translation, returning that key to its fallback locale.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    post,
    path = "/admin/i18n/message/clear",
    tag = "i18n-admin",
    request_body = ClearMessageRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The translation is gone.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn clear_message(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<ClearMessageRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id = service::assert_can_write(&state, &auth, &body.locale, Permission::Edit).await?;
    service::clear_message(&state, &body.locale, &body.key, user_id).await?;
    Ok(ok_status())
}

// ---------------------------------------------------------------- progress + audit

/// `GET /admin/i18n/namespaces` - the full namespace list, independent of
/// whatever page of messages the editor happens to be showing.
#[utoipa::path(
    get,
    path = "/admin/i18n/namespaces",
    tag = "i18n-admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Namespace names.", body = Vec<String>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn namespaces(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<String>>, ApiError> {
    assert_can_read(&state, &auth).await?;
    Ok(Json(service::list_namespaces(&state).await?))
}

/// How far each locale has got, as translated and stale counts.
#[utoipa::path(
    get,
    path = "/admin/i18n/progress",
    tag = "i18n-admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Per-locale progress.", body = Vec<crate::app::services::i18n::LocaleProgress>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn progress(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<service::LocaleProgress>>, ApiError> {
    assert_can_read(&state, &auth).await?;
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

/// Translation edits across the workspace, newest first.
#[utoipa::path(
    get,
    path = "/admin/i18n/audit",
    operation_id = "i18n_audit_log",
    tag = "i18n-admin",
    params(
        ("locale" = Option<String>, Query, description = "Restrict to one locale."),
        ("limit" = Option<i64>, Query, description = "Maximum entries to return."),
        ("before" = Option<String>, Query, description = "RFC 3339 timestamp; keep entries strictly older than this. Page by passing the oldest timestamp you have seen.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Audit entries.", body = crate::app::services::i18n::TranslationAuditResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<AuditQuery>,
) -> Result<Json<service::TranslationAuditResponse>, ApiError> {
    assert_can_read(&state, &auth).await?;
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
#[utoipa::path(
    get,
    path = "/admin/i18n/audit/entry",
    tag = "i18n-admin",
    params(
        ("locale" = String, Query, description = "Locale to read."),
        ("key" = String, Query, description = "Message key.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Audit entries for that key.", body = Vec<UiMessageAuditEntry>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn entry_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<EntryAuditQuery>,
) -> Result<Json<Vec<UiMessageAuditEntry>>, ApiError> {
    assert_can_read(&state, &auth).await?;
    Ok(Json(
        service::get_audit_log(&state, &params.key, &params.locale).await?,
    ))
}

// ---------------------------------------------------------------- sync

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SyncRequest {
    pub entries: Vec<service::SourceEntry>,
}

/// `POST /admin/i18n/sync` - the extractor's output becomes the truth about
/// which keys exist and what their English source is. Super-admin only: it can
/// deactivate every key in the catalog.
///
/// New keys are added, changed source marks its translations stale, and keys
/// the frontend dropped are retired.
#[utoipa::path(
    post,
    path = "/admin/i18n/sync",
    tag = "i18n-admin",
    request_body = SyncRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "What the sync added, changed and retired.", body = crate::app::services::i18n::SyncResult),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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

/// Game-data text overridden for one locale.
///
/// This is the override layer, not a full game-data translation: only the
/// fields someone has explicitly replaced appear here.
#[utoipa::path(
    get,
    path = "/admin/i18n/overrides",
    operation_id = "i18n_list_overrides",
    tag = "i18n-admin",
    params(
        ("locale" = String, Query, description = "Locale to read.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Stored overrides for that locale.", body = Vec<GamedataOverride>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_overrides(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<OverridesQuery>,
) -> Result<Json<Vec<GamedataOverride>>, ApiError> {
    assert_can_read(&state, &auth).await?;
    Ok(Json(service::list_overrides(&state, &params.locale).await?))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpsertOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
}

/// Override one game-data field for one locale.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    put,
    path = "/admin/i18n/overrides",
    operation_id = "i18n_put_override",
    tag = "i18n-admin",
    request_body = UpsertOverrideRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The override was stored.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn put_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpsertOverrideRequest>,
) -> Result<Json<StatusOk>, ApiError> {
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeleteOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
}

/// Drop one game-data override, returning that field to the game's own text.
/// Needs a translation grant on the locale, or a role that can open the admin
/// panel.
#[utoipa::path(
    post,
    path = "/admin/i18n/overrides/delete",
    operation_id = "i18n_delete_override",
    tag = "i18n-admin",
    request_body = DeleteOverrideRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The override is gone.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete_override(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<DeleteOverrideRequest>,
) -> Result<Json<StatusOk>, ApiError> {
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
