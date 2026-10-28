use sqlx::types::Json;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::database::models::content_audit::{
    ContentAuditAction, ContentAuditDetails, ContentAuditRow, ContentAuditRowWithPeople,
    ContentTargetKind,
};

/// One action about to be written.
pub struct NewContentAudit<'a> {
    pub actor_id: Uuid,
    pub target_kind: ContentTargetKind,
    pub target_id: Uuid,
    pub target_name: &'a str,
    pub target_slug: Option<&'a str>,
    pub owner_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub action: ContentAuditAction,
    pub details: ContentAuditDetails,
}

/// Which rows the admin feed reads. Every `None` is "any".
#[derive(Debug, Clone, Copy, Default)]
pub struct ContentAuditFilter {
    pub actor: Option<Uuid>,
    pub target_kind: Option<ContentTargetKind>,
    pub target_id: Option<Uuid>,
}

/// The actor's latest row on one target, locked for the caller's
/// transaction, when it is younger than `window_secs`.
pub async fn latest_locked(
    conn: &mut PgConnection,
    actor_id: Uuid,
    target_kind: ContentTargetKind,
    target_id: Uuid,
    window_secs: f64,
) -> Result<Option<ContentAuditRow>, sqlx::Error> {
    sqlx::query_as::<_, ContentAuditRow>(
        r"
        SELECT * FROM content_audit_log
        WHERE actor_id = $1 AND target_kind = $2 AND target_id = $3
          AND changed_at > NOW() - make_interval(secs => $4)
        ORDER BY changed_at DESC, id DESC
        LIMIT 1
        FOR UPDATE
        ",
    )
    .bind(actor_id)
    .bind(target_kind.as_str())
    .bind(target_id)
    .bind(window_secs)
    .fetch_optional(conn)
    .await
}

pub async fn insert(
    conn: &mut PgConnection,
    entry: &NewContentAudit<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO content_audit_log
            (actor_id, target_kind, target_id, target_name, target_slug,
             owner_id, subject_id, action, details)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ",
    )
    .bind(entry.actor_id)
    .bind(entry.target_kind.as_str())
    .bind(entry.target_id)
    .bind(entry.target_name)
    .bind(entry.target_slug)
    .bind(entry.owner_id)
    .bind(entry.subject_id)
    .bind(entry.action.as_str())
    .bind(Json(&entry.details))
    .execute(conn)
    .await?;
    Ok(())
}

/// Fold one more request into row `id`: its merged details, the target's
/// current name, one more event, touched now.
pub async fn update_coalesced(
    conn: &mut PgConnection,
    id: i64,
    target_name: &str,
    target_slug: Option<&str>,
    details: &ContentAuditDetails,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        UPDATE content_audit_log
        SET details = $2, target_name = $3, target_slug = COALESCE($4, target_slug),
            event_count = event_count + 1, changed_at = NOW()
        WHERE id = $1
        ",
    )
    .bind(id)
    .bind(Json(details))
    .bind(target_name)
    .bind(target_slug)
    .execute(conn)
    .await?;
    Ok(())
}

/// The feed, newest first. `before` pages by `changed_at`.
pub async fn list(
    pool: &PgPool,
    filter: ContentAuditFilter,
    limit: i64,
    before: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Vec<ContentAuditRowWithPeople>, sqlx::Error> {
    sqlx::query_as::<_, ContentAuditRowWithPeople>(
        r"
        SELECT
            a.*,
            u.uid                AS actor_uid,
            u.nickname           AS actor_nickname,
            u.secretary          AS actor_secretary,
            u.secretary_skin_id  AS actor_secretary_skin_id,
            o.uid                AS owner_uid,
            o.nickname           AS owner_nickname,
            s.uid                AS subject_uid,
            s.nickname           AS subject_nickname
        FROM content_audit_log a
        LEFT JOIN users u ON u.id = a.actor_id
        LEFT JOIN users o ON o.id = a.owner_id
        LEFT JOIN users s ON s.id = a.subject_id
        WHERE ($2::timestamptz IS NULL OR a.changed_at < $2)
          AND ($3::uuid IS NULL OR a.actor_id = $3)
          AND ($4::varchar IS NULL OR a.target_kind = $4)
          AND ($5::uuid IS NULL OR a.target_id = $5)
        ORDER BY a.changed_at DESC, a.id DESC
        LIMIT $1
        ",
    )
    .bind(limit)
    .bind(before)
    .bind(filter.actor)
    .bind(filter.target_kind.map(ContentTargetKind::as_str))
    .bind(filter.target_id)
    .fetch_all(pool)
    .await
}

/// Rows the feed would page through under `filter`.
pub async fn count(pool: &PgPool, filter: ContentAuditFilter) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r"
        SELECT COUNT(*) FROM content_audit_log
        WHERE ($1::uuid IS NULL OR actor_id = $1)
          AND ($2::varchar IS NULL OR target_kind = $2)
          AND ($3::uuid IS NULL OR target_id = $3)
        ",
    )
    .bind(filter.actor)
    .bind(filter.target_kind.map(ContentTargetKind::as_str))
    .bind(filter.target_id)
    .fetch_one(pool)
    .await
}
