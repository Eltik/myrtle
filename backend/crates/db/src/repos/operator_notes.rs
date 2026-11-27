use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct NoteRow {
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

#[derive(Debug, Clone)]
pub struct AuditRow {
    pub id: i64,
    pub note_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
    pub actor_nickname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditFeedRow {
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

pub async fn get_all(conn: &mut Conn) -> Result<Vec<NoteRow>, DbError> {
    Ok(sqlx::query_as!(
        NoteRow,
        "SELECT id, operator_id, pros, cons, notes, trivia, summary, tags, created_at, updated_at
         FROM operator_notes ORDER BY operator_id"
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_by_operator(
    conn: &mut Conn,
    operator_id: &str,
) -> Result<Option<NoteRow>, DbError> {
    Ok(sqlx::query_as!(
        NoteRow,
        "SELECT id, operator_id, pros, cons, notes, trivia, summary, tags, created_at, updated_at
         FROM operator_notes WHERE operator_id = $1",
        operator_id,
    )
    .fetch_optional(conn)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert(
    conn: &mut Conn,
    operator_id: &str,
    pros: Option<&str>,
    cons: Option<&str>,
    notes: Option<&str>,
    trivia: Option<&str>,
    summary: Option<&str>,
    tags: Option<&Value>,
) -> Result<NoteRow, DbError> {
    Ok(sqlx::query_as!(
        NoteRow,
        "INSERT INTO operator_notes (operator_id, pros, cons, notes, trivia, summary, tags)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (operator_id) DO UPDATE SET
             pros = COALESCE(EXCLUDED.pros, operator_notes.pros),
             cons = COALESCE(EXCLUDED.cons, operator_notes.cons),
             notes = COALESCE(EXCLUDED.notes, operator_notes.notes),
             trivia = COALESCE(EXCLUDED.trivia, operator_notes.trivia),
             summary = COALESCE(EXCLUDED.summary, operator_notes.summary),
             tags = COALESCE(EXCLUDED.tags, operator_notes.tags),
             updated_at = NOW()
         RETURNING id, operator_id, pros, cons, notes, trivia, summary, tags, created_at, updated_at",
        operator_id,
        pros,
        cons,
        notes,
        trivia,
        summary,
        tags,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn insert_audit(
    conn: &mut Conn,
    note_id: Uuid,
    field_name: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: UserId,
) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO operator_notes_audit_log (note_id, field_name, old_value, new_value, changed_by)
         VALUES ($1, $2, $3, $4, $5)",
        note_id,
        field_name,
        old_value,
        new_value,
        changed_by.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn get_audit_log(conn: &mut Conn, operator_id: &str) -> Result<Vec<AuditRow>, DbError> {
    Ok(sqlx::query_as!(
        AuditRow,
        "SELECT a.id, a.note_id, a.field_name, a.old_value, a.new_value, a.changed_by, a.changed_at,
                u.nickname AS actor_nickname
         FROM operator_notes_audit_log a
         JOIN operator_notes n ON n.id = a.note_id
         LEFT JOIN users u ON u.id = a.changed_by
         WHERE n.operator_id = $1
         ORDER BY a.changed_at DESC",
        operator_id,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_audit_log_global(
    conn: &mut Conn,
    limit: i64,
    before: Option<DateTime<Utc>>,
    actor: Option<UserId>,
) -> Result<Vec<AuditFeedRow>, DbError> {
    Ok(sqlx::query_as!(
        AuditFeedRow,
        r#"SELECT a.id, a.note_id, n.operator_id, a.field_name, a.old_value, a.new_value,
                  a.changed_at,
                  a.changed_by AS actor_user_id,
                  u.uid AS "actor_uid?",
                  u.nickname AS actor_nickname,
                  u.secretary AS actor_secretary,
                  u.secretary_skin_id AS actor_secretary_skin_id
           FROM operator_notes_audit_log a
           JOIN operator_notes n ON n.id = a.note_id
           LEFT JOIN users u ON u.id = a.changed_by
           WHERE ($2::timestamptz IS NULL OR a.changed_at < $2)
             AND ($3::uuid IS NULL OR a.changed_by = $3)
           ORDER BY a.changed_at DESC
           LIMIT $1"#,
        limit,
        before,
        actor.map(|a| a.0),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_audit_log(conn: &mut Conn, actor: Option<UserId>) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM operator_notes_audit_log
           WHERE ($1::uuid IS NULL OR changed_by = $1)"#,
        actor.map(|a| a.0),
    )
    .fetch_one(conn)
    .await?)
}
