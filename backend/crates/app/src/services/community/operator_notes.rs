//! Staff-written operator notes and their per-field edit history.

use chrono::{DateTime, Utc};
use myrtle_db::repos::operator_notes::{self as notes, AuditFeedRow, AuditRow, NoteRow};
use myrtle_types::UserId;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::policy::{Action, Resource, Subject};
use crate::{AppContext, AppError};

#[derive(Debug, Clone, Serialize)]
pub struct OperatorNote {
    pub id: Uuid,
    pub operator_id: String,
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorNoteAuditEntry {
    pub id: i64,
    pub note_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub actor: OperatorNoteAuditActor,
}

#[derive(Debug, Clone, Serialize)]
pub struct OperatorNoteAuditActor {
    pub id: Uuid,
    pub nickname: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogActor {
    pub user_id: Uuid,
    pub uid: Option<String>,
    pub nickname: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub note_id: Uuid,
    pub operator_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub actor: AuditLogActor,
}

#[derive(Debug, Serialize)]
pub struct GlobalAuditLogResponse {
    pub entries: Vec<AuditLogEntry>,
    pub total: i64,
}

#[derive(Debug, Default)]
pub struct UpdateFields {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<Value>,
}

pub const AUDIT_PAGE_DEFAULT: i64 = 100;

pub async fn list(ctx: &AppContext) -> Result<Vec<OperatorNote>, AppError> {
    let rows = notes::get_all(&mut *ctx.db.conn().await?).await?;
    Ok(rows.into_iter().map(note).collect())
}

pub async fn get(ctx: &AppContext, operator_id: &str) -> Result<OperatorNote, AppError> {
    notes::get_by_operator(&mut *ctx.db.conn().await?, operator_id)
        .await?
        .map(note)
        .ok_or(AppError::NotFound(None))
}

pub async fn audit_log(
    ctx: &AppContext,
    subject: &Subject,
    operator_id: &str,
) -> Result<Vec<OperatorNoteAuditEntry>, AppError> {
    ctx.policy
        .require(subject, Action::EditOperatorNotes, &Resource::OperatorNotes)?;
    let rows = notes::get_audit_log(&mut *ctx.db.conn().await?, operator_id).await?;
    Ok(rows.into_iter().map(audit_entry).collect())
}

/// Newest edits first; `limit` defaults to 100 and is clamped to 1..=500
pub async fn global_audit_log(
    ctx: &AppContext,
    subject: &Subject,
    limit: Option<i64>,
    before: Option<DateTime<Utc>>,
    actor: Option<&str>,
) -> Result<GlobalAuditLogResponse, AppError> {
    let actor = audit_actor_scope(ctx, subject, actor)?;
    let limit = limit.unwrap_or(AUDIT_PAGE_DEFAULT).clamp(1, 500);
    let mut conn = ctx.db.conn().await?;
    let rows = notes::get_audit_log_global(&mut conn, limit, before, actor).await?;
    let total = notes::count_audit_log(&mut conn, actor).await?;
    Ok(GlobalAuditLogResponse {
        entries: rows.into_iter().map(feed_entry).collect(),
        total,
    })
}

/// Whose edits the caller may read (`None` is everyone); always their own without `ReadOthersNoteAudit`
pub fn audit_actor_scope(
    ctx: &AppContext,
    subject: &Subject,
    actor: Option<&str>,
) -> Result<Option<UserId>, AppError> {
    ctx.policy
        .require(subject, Action::EditOperatorNotes, &Resource::OperatorNotes)?;
    let me = || subject.user_id().ok_or(AppError::Unauthorized);
    let named = match actor.map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some("me") => Some(me()?),
        Some(id) => Some(UserId(id.parse::<Uuid>().map_err(|_| {
            AppError::BadRequest(format!("actor must be `me` or an account id, got `{id}`"))
        })?)),
    };
    let reads_others = ctx
        .policy
        .can(
            subject,
            Action::ReadOthersNoteAudit,
            &Resource::OperatorNotes,
        )
        .is_allowed();
    if reads_others {
        return Ok(named);
    }
    let me = me()?;
    match named {
        Some(id) if id != me => Err(AppError::Forbidden),
        _ => Ok(Some(me)),
    }
}

/// Writes the note and one audit row per text field the edit changed
pub async fn update(
    ctx: &AppContext,
    subject: &Subject,
    operator_id: &str,
    fields: UpdateFields,
) -> Result<OperatorNote, AppError> {
    ctx.policy
        .require(subject, Action::EditOperatorNotes, &Resource::OperatorNotes)?;
    let changed_by = subject.user_id().ok_or(AppError::Unauthorized)?;
    let mut conn = ctx.db.conn().await?;
    let existing = notes::get_by_operator(&mut conn, operator_id).await?;
    let row = notes::upsert(
        &mut conn,
        operator_id,
        fields.pros.as_deref(),
        fields.cons.as_deref(),
        fields.notes.as_deref(),
        fields.trivia.as_deref(),
        fields.summary.as_deref(),
        fields.tags.as_ref(),
    )
    .await?;

    let old = existing.as_ref();
    let diffs = [
        ("pros", old.and_then(|e| e.pros.as_deref()), &fields.pros),
        ("cons", old.and_then(|e| e.cons.as_deref()), &fields.cons),
        ("notes", old.and_then(|e| e.notes.as_deref()), &fields.notes),
        (
            "trivia",
            old.and_then(|e| e.trivia.as_deref()),
            &fields.trivia,
        ),
        (
            "summary",
            old.and_then(|e| e.summary.as_deref()),
            &fields.summary,
        ),
    ];
    for (field, old, new) in diffs {
        if let Some(new) = new.as_deref()
            && old != Some(new)
        {
            notes::insert_audit(&mut conn, row.id, field, old, Some(new), changed_by).await?;
        }
    }
    Ok(note(row))
}

fn note(row: NoteRow) -> OperatorNote {
    OperatorNote {
        id: row.id,
        operator_id: row.operator_id,
        pros: row.pros,
        cons: row.cons,
        notes: row.notes,
        trivia: row.trivia,
        summary: row.summary,
        tags: row.tags,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn audit_entry(row: AuditRow) -> OperatorNoteAuditEntry {
    OperatorNoteAuditEntry {
        id: row.id,
        note_id: row.note_id,
        field_name: row.field_name,
        old_value: row.old_value,
        new_value: row.new_value,
        changed_by: row.changed_by,
        changed_at: row.changed_at,
        actor: OperatorNoteAuditActor {
            id: row.changed_by,
            nickname: row.actor_nickname,
        },
    }
}

fn feed_entry(row: AuditFeedRow) -> AuditLogEntry {
    AuditLogEntry {
        id: row.id,
        note_id: row.note_id,
        operator_id: row.operator_id,
        field_name: row.field_name,
        old_value: row.old_value,
        new_value: row.new_value,
        changed_at: row.changed_at,
        actor: AuditLogActor {
            user_id: row.actor_user_id,
            uid: row.actor_uid,
            nickname: row.actor_nickname,
            secretary: row.actor_secretary,
            secretary_skin_id: row.actor_secretary_skin_id,
        },
    }
}
