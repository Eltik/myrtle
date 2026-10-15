use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::operator_notes::{
    OperatorNote, OperatorNoteAuditEntry, OperatorNoteAuditEntryWithContext,
};

pub async fn get_all(pool: &PgPool) -> Result<Vec<OperatorNote>, sqlx::Error> {
    sqlx::query_as::<_, OperatorNote>("SELECT * FROM operator_notes ORDER BY operator_id")
        .fetch_all(pool)
        .await
}

pub async fn get_by_operator(
    pool: &PgPool,
    operator_id: &str,
) -> Result<Option<OperatorNote>, sqlx::Error> {
    sqlx::query_as::<_, OperatorNote>("SELECT * FROM operator_notes WHERE operator_id = $1")
        .bind(operator_id)
        .fetch_optional(pool)
        .await
}

#[allow(clippy::too_many_arguments)]
pub async fn upsert(
    pool: &PgPool,
    operator_id: &str,
    pros: Option<&str>,
    cons: Option<&str>,
    notes: Option<&str>,
    trivia: Option<&str>,
    summary: Option<&str>,
    tags: Option<&serde_json::Value>,
) -> Result<OperatorNote, sqlx::Error> {
    sqlx::query_as::<_, OperatorNote>(
        r"
        INSERT INTO operator_notes (operator_id, pros, cons, notes, trivia, summary, tags)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (operator_id) DO UPDATE SET
            pros = COALESCE(EXCLUDED.pros, operator_notes.pros),
            cons = COALESCE(EXCLUDED.cons, operator_notes.cons),
            notes = COALESCE(EXCLUDED.notes, operator_notes.notes),
            trivia = COALESCE(EXCLUDED.trivia, operator_notes.trivia),
            summary = COALESCE(EXCLUDED.summary, operator_notes.summary),
            tags = COALESCE(EXCLUDED.tags, operator_notes.tags),
            updated_at = NOW()
        RETURNING *
        ",
    )
    .bind(operator_id)
    .bind(pros)
    .bind(cons)
    .bind(notes)
    .bind(trivia)
    .bind(summary)
    .bind(tags)
    .fetch_one(pool)
    .await
}

pub async fn insert_audit(
    pool: &PgPool,
    note_id: Uuid,
    field_name: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    changed_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO operator_notes_audit_log (note_id, field_name, old_value, new_value, changed_by) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(note_id)
    .bind(field_name)
    .bind(old_value)
    .bind(new_value)
    .bind(changed_by)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_audit_log(
    pool: &PgPool,
    operator_id: &str,
) -> Result<Vec<OperatorNoteAuditEntry>, sqlx::Error> {
    sqlx::query_as::<_, OperatorNoteAuditEntry>(
        r"
        SELECT a.*, a.changed_by AS actor_id, u.nickname AS actor_nickname
        FROM operator_notes_audit_log a
        JOIN operator_notes n ON n.id = a.note_id
        LEFT JOIN users u ON u.id = a.changed_by
        WHERE n.operator_id = $1
        ORDER BY a.changed_at DESC
        ",
    )
    .bind(operator_id)
    .fetch_all(pool)
    .await
}

/// The audit log across every note, newest first. `actor` keeps only one
/// account's rows; `before` pages by `changed_at`.
pub async fn get_audit_log_global(
    pool: &PgPool,
    limit: i64,
    before: Option<chrono::DateTime<chrono::Utc>>,
    actor: Option<Uuid>,
) -> Result<Vec<OperatorNoteAuditEntryWithContext>, sqlx::Error> {
    // LEFT JOIN users so audit rows survive a hard-deleted user.
    sqlx::query_as::<_, OperatorNoteAuditEntryWithContext>(
        r"
        SELECT
            a.id,
            a.note_id,
            a.field_name,
            a.old_value,
            a.new_value,
            a.changed_at,
            n.operator_id,
            a.changed_by         AS actor_user_id,
            u.uid                AS actor_uid,
            u.nickname           AS actor_nickname,
            u.secretary          AS actor_secretary,
            u.secretary_skin_id  AS actor_secretary_skin_id
        FROM operator_notes_audit_log a
        JOIN operator_notes n ON n.id = a.note_id
        LEFT JOIN users u ON u.id = a.changed_by
        WHERE ($2::timestamptz IS NULL OR a.changed_at < $2)
          AND ($3::uuid IS NULL OR a.changed_by = $3)
        ORDER BY a.changed_at DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .bind(before)
    .bind(actor)
    .fetch_all(pool)
    .await
}

/// Rows in the audit log, or only `actor`'s.
pub async fn count_audit_log(pool: &PgPool, actor: Option<Uuid>) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM operator_notes_audit_log WHERE ($1::uuid IS NULL OR changed_by = $1)",
    )
    .bind(actor)
    .fetch_one(pool)
    .await?;
    Ok(count)
}
