use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use myrtle_app::AppContext;
use myrtle_app::services::i18n::{
    self, AuditQuery, ClearMessageRequest, DeleteOverrideRequest, EntryAuditQuery,
    GamedataOverride, ListMessagesQuery, Locale, LocaleProgress, SourceEntry, SyncResult,
    TranslationAuditResponse, TranslationEntry, TranslationGrant, TranslationGrantRequest,
    TranslationListResponse, UiMessageAuditEntry, UpdateMessageRequest, UpsertLocaleRequest,
    UpsertOverrideRequest,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::respond::Cached;
use crate::respond::{StatusOk, ok};
use crate::router::ApiRouter;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const SHORT: &str = "public, max-age=30";

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/i18n/manifest", get(manifest))
        .route("/i18n/{locale}/{namespace}/{hash}", get(catalog))
        .route("/admin/i18n/locales", get(list_locales).put(upsert_locale))
        .route("/admin/i18n/writable-locales", get(writable_locales))
        .route("/admin/i18n/messages", get(list_messages))
        .route("/admin/i18n/message", put(update_message))
        .route("/admin/i18n/message/clear", post(clear_message))
        .route("/admin/i18n/namespaces", get(namespaces))
        .route("/admin/i18n/progress", get(progress))
        .route("/admin/i18n/audit", get(audit_log))
        .route("/admin/i18n/audit/entry", get(entry_audit_log))
        .route("/admin/i18n/sync", post(sync))
        .route(
            "/admin/i18n/overrides",
            get(list_overrides).put(put_override),
        )
        .route("/admin/i18n/overrides/delete", post(delete_override))
        .route("/admin/i18n/permissions", get(list_grants).post(grant))
        .route("/admin/i18n/permissions/me", get(my_grants))
        .route("/admin/i18n/permissions/revoke", post(revoke))
}

async fn manifest(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(Cached::new(i18n::manifest(&ctx).await?).cache_control(SHORT))
}

async fn catalog(
    State(ctx): State<AppContext>,
    Path((locale, namespace, hash)): Path<(String, String, String)>,
) -> ApiResult<Cached> {
    let response = i18n::catalog(&ctx, &locale, &namespace, &hash).await?;
    let cache_control = if response.is_current && hash != "latest" {
        IMMUTABLE
    } else {
        SHORT
    };
    Ok(Cached::new(response.json).cache_control(cache_control))
}

async fn list_locales(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<Locale>>> {
    Ok(Json(i18n::list_locales(&ctx, &caller).await?))
}

async fn upsert_locale(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<UpsertLocaleRequest>,
) -> ApiResult<Json<Locale>> {
    Ok(Json(i18n::upsert_locale(&ctx, &caller, &body).await?))
}

async fn writable_locales(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(i18n::writable_locales(&ctx, &caller).await?))
}

async fn list_messages(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<ListMessagesQuery>,
) -> ApiResult<Json<TranslationListResponse>> {
    Ok(Json(i18n::list_messages(&ctx, &caller, &query).await?))
}

async fn update_message(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<UpdateMessageRequest>,
) -> ApiResult<Json<TranslationEntry>> {
    Ok(Json(i18n::update_message(&ctx, &caller, &body).await?))
}

async fn clear_message(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<ClearMessageRequest>,
) -> ApiResult<Json<StatusOk>> {
    i18n::clear_message(&ctx, &caller, &body).await?;
    Ok(ok())
}

async fn namespaces(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(i18n::list_namespaces(&ctx, &caller).await?))
}

async fn progress(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<LocaleProgress>>> {
    Ok(Json(i18n::locale_progress(&ctx, &caller).await?))
}

async fn audit_log(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<AuditQuery>,
) -> ApiResult<Json<TranslationAuditResponse>> {
    Ok(Json(i18n::audit_feed(&ctx, &caller, &query).await?))
}

async fn entry_audit_log(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<EntryAuditQuery>,
) -> ApiResult<Json<Vec<UiMessageAuditEntry>>> {
    Ok(Json(i18n::entry_history(&ctx, &caller, &query).await?))
}

#[derive(Deserialize)]
struct SyncRequest {
    entries: Vec<SourceEntry>,
}

async fn sync(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<SyncRequest>,
) -> ApiResult<Json<SyncResult>> {
    Ok(Json(
        i18n::sync_source_catalog(&ctx, &caller, &body.entries).await?,
    ))
}

#[derive(Deserialize)]
struct OverridesQuery {
    locale: String,
}

async fn list_overrides(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<OverridesQuery>,
) -> ApiResult<Json<Vec<GamedataOverride>>> {
    Ok(Json(
        i18n::list_overrides(&ctx, &caller, &query.locale).await?,
    ))
}

async fn put_override(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<UpsertOverrideRequest>,
) -> ApiResult<Json<StatusOk>> {
    i18n::upsert_override(&ctx, &caller, &body).await?;
    Ok(ok())
}

async fn delete_override(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<DeleteOverrideRequest>,
) -> ApiResult<Json<StatusOk>> {
    i18n::delete_override(&ctx, &caller, &body).await?;
    Ok(ok())
}

#[derive(Deserialize)]
struct GrantsQuery {
    #[serde(default)]
    locale: Option<String>,
}

async fn list_grants(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<GrantsQuery>,
) -> ApiResult<Json<Vec<TranslationGrant>>> {
    let grants = i18n::list_grants(&ctx, &caller, query.locale.as_deref()).await?;
    Ok(Json(grants))
}

async fn my_grants(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<Vec<TranslationGrant>>> {
    Ok(Json(i18n::my_grants(&ctx, &caller).await?))
}

async fn grant(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<TranslationGrantRequest>,
) -> ApiResult<Json<StatusOk>> {
    i18n::grant(&ctx, &caller, &body).await?;
    Ok(ok())
}

async fn revoke(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<TranslationGrantRequest>,
) -> ApiResult<Json<StatusOk>> {
    i18n::revoke(&ctx, &caller, &body).await?;
    Ok(ok())
}
