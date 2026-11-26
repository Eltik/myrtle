use std::collections::HashSet;

use chrono::{DateTime, Utc};
use myrtle_db::repos::i18n;
use myrtle_db::repos::i18n::{AuditRow, EntryRow};
use myrtle_types::{FieldError, UserId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{CACHE_PREFIX, require_locale, require_panel};
use crate::policy::{Action, Resource, Subject};
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslationEntry {
    pub key: String,
    pub namespace: String,
    pub source_text: String,
    pub source_hash: String,
    pub description: Option<String>,
    pub placeholders: serde_json::Value,
    pub value: Option<String>,
    pub translated_hash: Option<String>,
    pub translated_source_text: Option<String>,
    pub is_stale: bool,
    pub updated_at: Option<DateTime<Utc>>,
    pub updated_by: Option<Uuid>,
}

impl From<EntryRow> for TranslationEntry {
    fn from(r: EntryRow) -> Self {
        Self {
            key: r.key,
            namespace: r.namespace,
            source_text: r.source_text,
            source_hash: r.source_hash,
            description: r.description,
            placeholders: r.placeholders,
            value: r.value,
            translated_hash: r.translated_hash,
            translated_source_text: r.translated_source_text,
            is_stale: r.is_stale,
            updated_at: r.updated_at,
            updated_by: r.updated_by,
        }
    }
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct UiMessageAuditEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub actor: UiMessageAuditActor,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct UiMessageAuditActor {
    pub id: Uuid,
    pub nickname: Option<String>,
}

impl From<AuditRow> for UiMessageAuditEntry {
    fn from(r: AuditRow) -> Self {
        Self {
            id: r.id,
            message_key: r.message_key,
            locale: r.locale,
            old_value: r.old_value,
            new_value: r.new_value,
            changed_by: r.changed_by,
            changed_at: r.changed_at,
            actor: UiMessageAuditActor {
                id: r.changed_by,
                nickname: r.actor_nickname,
            },
        }
    }
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslationListResponse {
    pub entries: Vec<TranslationEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LocaleProgress {
    pub locale: String,
    #[ts(type = "number")]
    pub total: i64,
    #[ts(type = "number")]
    pub translated: i64,
    #[ts(type = "number")]
    pub stale: i64,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslationAuditActor {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslationAuditEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub actor: TranslationAuditActor,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TranslationAuditResponse {
    pub entries: Vec<TranslationAuditEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub locale: String,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub locale: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ClearMessageRequest {
    pub locale: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub actor: Option<UserId>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct EntryAuditQuery {
    pub locale: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct SourceEntry {
    pub key: String,
    pub namespace: String,
    pub source_text: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub placeholders: Vec<String>,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SyncResult {
    #[ts(type = "number")]
    pub upserted: u64,
    #[ts(type = "number")]
    pub deactivated: u64,
}

#[must_use]
pub fn source_hash(text: &str) -> String {
    hex::encode(&Sha256::digest(text.as_bytes())[..8])
}

pub fn referenced_placeholders(message: &str) -> Result<HashSet<String>, String> {
    enum Ctx {
        Message,
        Branches,
    }
    const SELECTORS: [&str; 3] = ["plural", "selectordinal", "select"];

    let chars: Vec<char> = message.chars().collect();
    let skip_ws = |mut j: usize| {
        while chars.get(j).is_some_and(|c| c.is_whitespace()) {
            j += 1;
        }
        j
    };
    let mut found = HashSet::new();
    let mut stack = vec![Ctx::Message];
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\'' => {
                if chars.get(i + 1) == Some(&'\'') {
                    i += 1;
                } else if matches!(chars.get(i + 1), Some('{' | '}' | '#' | '|')) {
                    i += 1;
                    while i < chars.len() && chars[i] != '\'' {
                        i += 1;
                    }
                }
            }
            '{' if matches!(stack.last(), Some(Ctx::Branches)) => stack.push(Ctx::Message),
            '{' => {
                let start = skip_ws(i + 1);
                let mut end = start;
                while chars
                    .get(end)
                    .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                {
                    end += 1;
                }
                if end > start {
                    found.insert(chars[start..end].iter().collect::<String>());
                }
                let after = skip_ws(end);
                if chars.get(after) == Some(&',') {
                    let kind_start = skip_ws(after + 1);
                    let mut kind_end = kind_start;
                    while chars.get(kind_end).is_some_and(|c| c.is_alphabetic()) {
                        kind_end += 1;
                    }
                    let kind: String = chars[kind_start..kind_end].iter().collect();
                    if SELECTORS.contains(&kind.as_str()) {
                        stack.push(Ctx::Branches);
                        i = kind_end;
                        continue;
                    }
                }
                stack.push(Ctx::Message);
                i = end;
                continue;
            }
            '}' => {
                stack.pop();
                if stack.is_empty() {
                    return Err("unbalanced '}' in message".to_owned());
                }
            }
            _ => {}
        }
        i += 1;
    }

    if stack.len() != 1 {
        return Err("unbalanced '{' in message".to_owned());
    }
    Ok(found)
}

pub fn validate_message(value: &str, declared: &serde_json::Value) -> Result<(), AppError> {
    let field_error = |message| FieldError {
        field: "value".to_owned(),
        message,
    };
    let braced = |mut names: Vec<&String>| {
        names.sort();
        let names: Vec<String> = names.into_iter().map(|s| format!("{{{s}}}")).collect();
        names.join(", ")
    };
    let referenced =
        referenced_placeholders(value).map_err(|e| AppError::Validation(vec![field_error(e)]))?;
    let declared: HashSet<String> = declared
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    let mut errors = Vec::new();
    let undeclared: Vec<&String> = referenced.difference(&declared).collect();
    if !undeclared.is_empty() {
        let declared = if declared.is_empty() {
            "none".to_owned()
        } else {
            braced(declared.iter().collect())
        };
        errors.push(field_error(format!(
            "unknown placeholder(s): {}. The source string declares: {declared}",
            braced(undeclared)
        )));
    }
    let missing: Vec<&String> = declared.difference(&referenced).collect();
    if !missing.is_empty() {
        errors.push(field_error(format!(
            "missing placeholder(s): {}",
            braced(missing)
        )));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}

pub async fn list_messages(
    ctx: &AppContext,
    subject: &Subject,
    q: &ListMessagesQuery,
) -> Result<TranslationListResponse, AppError> {
    require_panel(ctx, subject).await?;
    let namespace = q.namespace.as_deref().unwrap_or("");
    let search = q.search.as_deref().unwrap_or("");
    let filter = q.filter.as_deref().unwrap_or("all");
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let offset = q.offset.unwrap_or(0).max(0);
    let mut conn = ctx.db.conn().await?;
    let entries = i18n::list_entries(
        &mut conn, &q.locale, namespace, search, filter, limit, offset,
    )
    .await?;
    let total = i18n::count_entries(&mut conn, &q.locale, namespace, search, filter).await?;
    Ok(TranslationListResponse {
        entries: entries.into_iter().map(TranslationEntry::from).collect(),
        total,
    })
}

pub async fn list_namespaces(ctx: &AppContext, subject: &Subject) -> Result<Vec<String>, AppError> {
    require_panel(ctx, subject).await?;
    Ok(i18n::list_namespaces(&mut *ctx.db.conn().await?).await?)
}

pub async fn locale_progress(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<LocaleProgress>, AppError> {
    require_panel(ctx, subject).await?;
    let rows = i18n::locale_progress(&mut *ctx.db.conn().await?).await?;
    Ok(rows
        .into_iter()
        .map(|r| LocaleProgress {
            locale: r.code,
            total: r.total,
            translated: r.translated,
            stale: r.stale,
        })
        .collect())
}

pub async fn entry_history(
    ctx: &AppContext,
    subject: &Subject,
    q: &EntryAuditQuery,
) -> Result<Vec<UiMessageAuditEntry>, AppError> {
    require_panel(ctx, subject).await?;
    let rows = i18n::get_audit_log(&mut *ctx.db.conn().await?, &q.key, &q.locale).await?;
    Ok(rows.into_iter().map(UiMessageAuditEntry::from).collect())
}

pub async fn audit_feed(
    ctx: &AppContext,
    subject: &Subject,
    q: &AuditQuery,
) -> Result<TranslationAuditResponse, AppError> {
    require_panel(ctx, subject).await?;
    let locale = q.locale.as_deref();
    let limit = q.limit.unwrap_or(100).clamp(1, 500);
    let mut conn = ctx.db.conn().await?;
    let rows = i18n::get_audit_log_global(&mut conn, locale, q.actor, limit, q.before).await?;
    let total = i18n::count_audit_log(&mut conn, locale, q.actor).await?;
    let entries = rows
        .into_iter()
        .map(|r| TranslationAuditEntry {
            id: r.id,
            message_key: r.message_key,
            locale: r.locale,
            old_value: r.old_value,
            new_value: r.new_value,
            changed_at: r.changed_at,
            actor: TranslationAuditActor {
                user_id: r.actor_user_id,
                uid: r.actor_uid,
                nickname: r.actor_nickname,
                secretary: r.actor_secretary,
                secretary_skin_id: r.actor_secretary_skin_id,
            },
        })
        .collect();
    Ok(TranslationAuditResponse { entries, total })
}

pub async fn update_message(
    ctx: &AppContext,
    subject: &Subject,
    req: &UpdateMessageRequest,
) -> Result<TranslationEntry, AppError> {
    let user = require_locale(ctx, subject, Action::WriteTranslations, &req.locale).await?;
    let mut conn = ctx.db.conn().await?;
    let existing = i18n::get_entry(&mut conn, &req.locale, &req.key)
        .await?
        .ok_or(AppError::NotFound(None))?;
    validate_message(&req.value, &existing.placeholders)?;
    i18n::upsert_message(
        &mut conn,
        &req.key,
        &req.locale,
        &req.value,
        &existing.source_hash,
        &existing.source_text,
        user,
    )
    .await?;
    if existing.value.as_deref() != Some(req.value.as_str()) {
        i18n::insert_audit(
            &mut conn,
            &req.key,
            &req.locale,
            existing.value.as_deref(),
            Some(&req.value),
            user,
        )
        .await?;
    }
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;

    let entry = i18n::get_entry(&mut conn, &req.locale, &req.key)
        .await?
        .ok_or(AppError::NotFound(None))?;
    Ok(entry.into())
}

pub async fn clear_message(
    ctx: &AppContext,
    subject: &Subject,
    req: &ClearMessageRequest,
) -> Result<(), AppError> {
    let user = require_locale(ctx, subject, Action::WriteTranslations, &req.locale).await?;
    let mut conn = ctx.db.conn().await?;
    let existing = i18n::get_entry(&mut conn, &req.locale, &req.key)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let Some(old) = existing.value else {
        return Ok(());
    };
    i18n::delete_message(&mut conn, &req.key, &req.locale).await?;
    i18n::insert_audit(&mut conn, &req.key, &req.locale, Some(&old), None, user).await?;
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
    Ok(())
}

pub async fn sync_source_catalog(
    ctx: &AppContext,
    subject: &Subject,
    entries: &[SourceEntry],
) -> Result<SyncResult, AppError> {
    ctx.policy
        .require(subject, Action::ManageLocales, &Resource::AdminPanel)?;
    if entries.is_empty() {
        return Err(AppError::bad_request(
            "refusing to sync an empty catalog: that would deactivate every key",
        ));
    }
    let keys: Vec<String> = entries.iter().map(|e| e.key.clone()).collect();
    let namespaces: Vec<String> = entries.iter().map(|e| e.namespace.clone()).collect();
    let texts: Vec<String> = entries.iter().map(|e| e.source_text.clone()).collect();
    let hashes: Vec<String> = entries
        .iter()
        .map(|e| source_hash(&e.source_text))
        .collect();
    let descriptions: Vec<Option<String>> = entries.iter().map(|e| e.description.clone()).collect();
    let placeholders: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| serde_json::json!(e.placeholders))
        .collect();

    let mut conn = ctx.db.conn().await?;
    let upserted = i18n::sync_keys(
        &mut conn,
        &keys,
        &namespaces,
        &texts,
        &hashes,
        &descriptions,
        &placeholders,
    )
    .await?;
    let deactivated = i18n::deactivate_missing_keys(&mut conn, &keys).await?;
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
    Ok(SyncResult {
        upserted,
        deactivated,
    })
}
