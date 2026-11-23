mod catalog;
mod grants;
mod messages;

pub use catalog::{
    ALL_NAMESPACES, CatalogResponse, I18nManifest, I18nManifestLocale, catalog, manifest,
};
pub use grants::{
    TranslationGrant, TranslationGrantRequest, grant, list_grants, my_grants, revoke,
};
pub use messages::{
    AuditQuery, ClearMessageRequest, EntryAuditQuery, ListMessagesQuery, LocaleProgress,
    SourceEntry, SyncResult, TranslationAuditActor, TranslationAuditEntry,
    TranslationAuditResponse, TranslationEntry, TranslationListResponse, UiMessageAuditActor,
    UiMessageAuditEntry, UpdateMessageRequest, audit_feed, clear_message, entry_history,
    list_messages, list_namespaces, locale_progress, referenced_placeholders, source_hash,
    sync_source_catalog, update_message, validate_message,
};

use chrono::{DateTime, Utc};
use myrtle_db::repos::i18n;
use myrtle_db::repos::i18n::{LocaleRow, OverrideRow};
use myrtle_types::{Role, UserId};
use serde::{Deserialize, Serialize};

use crate::policy::{Action, Resource, Subject};
use crate::services::accounts::with_grants;
use crate::{AppContext, AppError};

const CACHE_PREFIX: &str = "i18n:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locale {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub fallback_locale: Option<String>,
    pub gamedata_server: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub is_source: bool,
}

impl From<LocaleRow> for Locale {
    fn from(r: LocaleRow) -> Self {
        Self {
            code: r.code,
            english_name: r.english_name,
            native_name: r.native_name,
            fallback_locale: r.fallback_locale,
            gamedata_server: r.gamedata_server,
            enabled: r.enabled,
            sort_order: r.sort_order,
            is_source: r.is_source,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GamedataOverride {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

impl From<OverrideRow> for GamedataOverride {
    fn from(r: OverrideRow) -> Self {
        Self {
            locale: r.locale,
            kind: r.kind,
            entity_id: r.entity_id,
            field: r.field,
            value: r.value,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpsertLocaleRequest {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub fallback_locale: Option<String>,
    pub gamedata_server: String,
    pub enabled: bool,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpsertOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteOverrideRequest {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
}

async fn require_panel(ctx: &AppContext, subject: &Subject) -> Result<Subject, AppError> {
    let subject = with_grants(ctx, subject.clone()).await?;
    ctx.policy
        .require(&subject, Action::AccessAdminPanel, &Resource::AdminPanel)?;
    Ok(subject)
}

async fn require_locale(
    ctx: &AppContext,
    subject: &Subject,
    action: Action,
    locale: &str,
) -> Result<UserId, AppError> {
    let subject = with_grants(ctx, subject.clone()).await?;
    ctx.policy.require(
        &subject,
        action,
        &Resource::Locale {
            code: locale.to_owned(),
        },
    )?;
    subject.user_id().ok_or(AppError::Unauthorized)
}

pub async fn list_locales(ctx: &AppContext, subject: &Subject) -> Result<Vec<Locale>, AppError> {
    require_panel(ctx, subject).await?;
    let rows = i18n::list_locales(&mut *ctx.db.conn().await?, false).await?;
    Ok(rows.into_iter().map(Locale::from).collect())
}

pub async fn writable_locales(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<String>, AppError> {
    let subject = require_panel(ctx, subject).await?;
    let rows = i18n::list_locales(&mut *ctx.db.conn().await?, false).await?;
    let mut codes: Vec<String> = rows
        .into_iter()
        .map(|l| l.code)
        .filter(|code| {
            ctx.policy
                .can(
                    &subject,
                    Action::WriteTranslations,
                    &Resource::Locale { code: code.clone() },
                )
                .is_allowed()
        })
        .collect();
    if subject.role() != Some(Role::SuperAdmin) {
        codes.sort();
        codes.dedup();
    }
    Ok(codes)
}

pub async fn upsert_locale(
    ctx: &AppContext,
    subject: &Subject,
    req: &UpsertLocaleRequest,
) -> Result<Locale, AppError> {
    ctx.policy.require(
        subject,
        Action::ManageLocales,
        &Resource::Locale {
            code: req.code.clone(),
        },
    )?;
    let row = i18n::upsert_locale(
        &mut *ctx.db.conn().await?,
        &req.code,
        &req.english_name,
        &req.native_name,
        req.fallback_locale.as_deref(),
        &req.gamedata_server,
        req.enabled,
        req.sort_order.unwrap_or(100),
    )
    .await?;
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
    Ok(row.into())
}

pub async fn list_overrides(
    ctx: &AppContext,
    subject: &Subject,
    locale: &str,
) -> Result<Vec<GamedataOverride>, AppError> {
    require_panel(ctx, subject).await?;
    let rows = i18n::list_overrides(&mut *ctx.db.conn().await?, locale).await?;
    Ok(rows.into_iter().map(GamedataOverride::from).collect())
}

pub async fn upsert_override(
    ctx: &AppContext,
    subject: &Subject,
    req: &UpsertOverrideRequest,
) -> Result<(), AppError> {
    let user = require_locale(ctx, subject, Action::WriteTranslations, &req.locale).await?;
    i18n::upsert_override(
        &mut *ctx.db.conn().await?,
        &req.locale,
        &req.kind,
        &req.entity_id,
        &req.field,
        &req.value,
        user,
    )
    .await?;
    flush_with_static(ctx).await;
    Ok(())
}

pub async fn delete_override(
    ctx: &AppContext,
    subject: &Subject,
    req: &DeleteOverrideRequest,
) -> Result<(), AppError> {
    require_locale(ctx, subject, Action::WriteTranslations, &req.locale).await?;
    i18n::delete_override(
        &mut *ctx.db.conn().await?,
        &req.locale,
        &req.kind,
        &req.entity_id,
        &req.field,
    )
    .await?;
    flush_with_static(ctx).await;
    Ok(())
}

async fn flush_with_static(ctx: &AppContext) {
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
    ctx.cache.invalidate_prefix("static:").await;
}
