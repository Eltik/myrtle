use serde::Serialize;
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OperatorNote {
    pub id: Uuid,
    pub operator_id: String,
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OperatorNoteAuditEntry {
    pub id: i64,
    pub note_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// Audit entry enriched with the actor's display info, so clients don't have to
/// fan out N user lookups per page. `actor_uid` is the game UID (used in
/// `/user/{uid}` routes); `actor_user_id` is the internal DB UUID.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct OperatorNoteAuditEntryWithContext {
    pub id: i64,
    pub note_id: Uuid,
    pub operator_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub actor_user_id: Uuid,
    pub actor_uid: Option<String>,
    pub actor_nickname: Option<String>,
    pub actor_secretary: Option<String>,
    pub actor_secretary_skin_id: Option<String>,
}
