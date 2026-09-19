use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

/// `locales` table. `gamedata_server` names the Arknights client whose text
/// serves this locale's operator/skill/stage strings.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Locale {
    pub code: String,
    pub english_name: String,
    pub native_name: String,
    pub fallback_locale: Option<String>,
    pub gamedata_server: String,
    pub enabled: bool,
    pub sort_order: i32,
    /// The locale the source text is written in. Complete by definition: its
    /// text is `ui_message_keys.source_text`, not a `ui_messages` row.
    pub is_source: bool,
}

/// `ui_message_keys` table - one row per translatable string in the frontend
/// source, written by the extractor's sync and never by a translator.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UiMessageKey {
    pub key: String,
    pub namespace: String,
    pub source_text: String,
    pub source_hash: String,
    pub description: Option<String>,
    pub placeholders: serde_json::Value,
    pub is_active: bool,
}

/// `ui_messages` table - the rendered text for one key in one locale.
///
/// `source_text` is the English this value was written against, snapshotted at
/// save time; it is `None` on rows written before that column existed.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UiMessage {
    pub key: String,
    pub locale: String,
    pub value: String,
    pub source_hash: String,
    pub source_text: Option<String>,
    pub updated_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
}

/// One row of the translation editor: the key and its English source joined to
/// this locale's translation, if any.
///
/// `is_stale` is computed in SQL rather than stored - it is true when a
/// translation exists but was written against an older English source, which
/// is the only staleness this system has.
///
/// `translated_source_text` is the English that was on screen when the value
/// was saved. It is what turns `is_stale` from a badge into a diff, and it is
/// `None` for rows saved before the snapshot column existed.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
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

/// Per-key/locale history. `old_value` is what makes revert a write rather
/// than a restore.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UiMessageAuditEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// Audit entry enriched with the actor's display info, so the global feed
/// doesn't fan out N user lookups per page. Mirrors
/// `OperatorNoteAuditEntryWithContext`.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UiMessageAuditEntryWithContext {
    #[ts(type = "number")]
    pub id: i64,
    pub message_key: String,
    pub locale: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub actor_user_id: Uuid,
    pub actor_uid: Option<String>,
    pub actor_nickname: Option<String>,
    pub actor_secretary: Option<String>,
    pub actor_secretary_skin_id: Option<String>,
}

/// `translation_permissions` row. Shape-identical to a tier-list permission.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TranslationPermission {
    pub locale: String,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

/// `gamedata_overrides` row - a hand-entered replacement for one field of one
/// game-data entity in one locale.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GamedataOverride {
    pub locale: String,
    pub kind: String,
    pub entity_id: String,
    pub field: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

/// `ui_documents` row - long-form prose translated as a whole document.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct UiDocument {
    pub slug: String,
    pub locale: String,
    pub title: String,
    pub body: String,
    pub updated_at: DateTime<Utc>,
}
