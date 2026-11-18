use chrono::{DateTime, Utc};
use myrtle_types::{ContentAuditAction, ContentTargetKind, UserId};
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct ContentAuditRow {
    pub id: i64,
    pub actor_id: Uuid,
    pub target_kind: String,
    pub target_id: Uuid,
    pub target_name: String,
    pub target_slug: Option<String>,
    pub owner_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub action: String,
    pub details: Value,
    pub event_count: i32,
    pub first_at: DateTime<Utc>,
    pub changed_at: DateTime<Utc>,
}

/// A row with the people it names, for the admin feed
#[derive(Debug, Clone)]
pub struct ContentAuditFeedRow {
    pub row: ContentAuditRow,
    pub actor_uid: Option<String>,
    pub actor_nickname: Option<String>,
    pub actor_secretary: Option<String>,
    pub actor_secretary_skin_id: Option<String>,
    pub owner_uid: Option<String>,
    pub owner_nickname: Option<String>,
    pub subject_uid: Option<String>,
    pub subject_nickname: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct NewContentAudit<'a> {
    pub actor_id: UserId,
    pub target_kind: ContentTargetKind,
    pub target_id: Uuid,
    pub target_name: &'a str,
    pub target_slug: Option<&'a str>,
    pub owner_id: Option<UserId>,
    pub subject_id: Option<UserId>,
    pub action: ContentAuditAction,
    pub details: &'a Value,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentAuditFilter {
    pub actor: Option<UserId>,
    pub target_kind: Option<ContentTargetKind>,
    pub target_id: Option<Uuid>,
}

pub async fn latest_locked(
    conn: &mut Conn,
    actor: UserId,
    target_kind: ContentTargetKind,
    target_id: Uuid,
    window_secs: f64,
) -> Result<Option<ContentAuditRow>, DbError> {
    Ok(sqlx::query_as!(
        ContentAuditRow,
        "SELECT id, actor_id, target_kind, target_id, target_name, target_slug, owner_id,
                subject_id, action, details, event_count, first_at, changed_at
         FROM content_audit_log
         WHERE actor_id = $1 AND target_kind = $2 AND target_id = $3
           AND changed_at > NOW() - make_interval(secs => $4)
         ORDER BY changed_at DESC, id DESC
         LIMIT 1
         FOR UPDATE",
        actor.0,
        target_kind.as_str(),
        target_id,
        window_secs,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn insert(conn: &mut Conn, entry: &NewContentAudit<'_>) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO content_audit_log
             (actor_id, target_kind, target_id, target_name, target_slug,
              owner_id, subject_id, action, details)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        entry.actor_id.0,
        entry.target_kind.as_str(),
        entry.target_id,
        entry.target_name,
        entry.target_slug,
        entry.owner_id.map(|u| u.0),
        entry.subject_id.map(|u| u.0),
        entry.action.as_str(),
        entry.details,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn update_coalesced(
    conn: &mut Conn,
    id: i64,
    target_name: &str,
    target_slug: Option<&str>,
    details: &Value,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE content_audit_log
         SET details = $2, target_name = $3, target_slug = COALESCE($4, target_slug),
             event_count = event_count + 1, changed_at = NOW()
         WHERE id = $1",
        id,
        details,
        target_name,
        target_slug,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list(
    conn: &mut Conn,
    filter: ContentAuditFilter,
    limit: i64,
    before: Option<DateTime<Utc>>,
) -> Result<Vec<ContentAuditFeedRow>, DbError> {
    let rows = sqlx::query!(
        r#"SELECT
               a.id, a.actor_id, a.target_kind, a.target_id, a.target_name, a.target_slug,
               a.owner_id, a.subject_id, a.action, a.details, a.event_count, a.first_at,
               a.changed_at,
               u.uid                AS "actor_uid?",
               u.nickname           AS "actor_nickname?",
               u.secretary          AS "actor_secretary?",
               u.secretary_skin_id  AS "actor_secretary_skin_id?",
               o.uid                AS "owner_uid?",
               o.nickname           AS "owner_nickname?",
               s.uid                AS "subject_uid?",
               s.nickname           AS "subject_nickname?"
           FROM content_audit_log a
           LEFT JOIN users u ON u.id = a.actor_id
           LEFT JOIN users o ON o.id = a.owner_id
           LEFT JOIN users s ON s.id = a.subject_id
           WHERE ($2::timestamptz IS NULL OR a.changed_at < $2)
             AND ($3::uuid IS NULL OR a.actor_id = $3)
             AND ($4::varchar IS NULL OR a.target_kind = $4)
             AND ($5::uuid IS NULL OR a.target_id = $5)
           ORDER BY a.changed_at DESC, a.id DESC
           LIMIT $1"#,
        limit,
        before,
        filter.actor.map(|u| u.0),
        filter.target_kind.map(ContentTargetKind::as_str),
        filter.target_id,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ContentAuditFeedRow {
            row: ContentAuditRow {
                id: r.id,
                actor_id: r.actor_id,
                target_kind: r.target_kind,
                target_id: r.target_id,
                target_name: r.target_name,
                target_slug: r.target_slug,
                owner_id: r.owner_id,
                subject_id: r.subject_id,
                action: r.action,
                details: r.details,
                event_count: r.event_count,
                first_at: r.first_at,
                changed_at: r.changed_at,
            },
            actor_uid: r.actor_uid,
            actor_nickname: r.actor_nickname,
            actor_secretary: r.actor_secretary,
            actor_secretary_skin_id: r.actor_secretary_skin_id,
            owner_uid: r.owner_uid,
            owner_nickname: r.owner_nickname,
            subject_uid: r.subject_uid,
            subject_nickname: r.subject_nickname,
        })
        .collect())
}

pub async fn count(conn: &mut Conn, filter: ContentAuditFilter) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM content_audit_log
           WHERE ($1::uuid IS NULL OR actor_id = $1)
             AND ($2::varchar IS NULL OR target_kind = $2)
             AND ($3::uuid IS NULL OR target_id = $3)"#,
        filter.actor.map(|u| u.0),
        filter.target_kind.map(ContentTargetKind::as_str),
        filter.target_id,
    )
    .fetch_one(conn)
    .await?)
}
