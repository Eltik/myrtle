use chrono::{DateTime, Utc};
use myrtle_types::{TierListId, UserId};
use uuid::Uuid;

use super::{StoredList, TierListRow};
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct StatsRow {
    pub tier_list_id: Uuid,
    pub view_count: i64,
    pub unique_view_count: i64,
    pub favorite_count: i32,
    pub share_count: i32,
    pub is_trending: bool,
    pub trending_score: f64,
    pub views_last_24h: i32,
    pub views_last_7d: i32,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub stats_updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer<'a> {
    Account(UserId),
    Session(&'a str),
}

impl<'a> Viewer<'a> {
    #[must_use]
    pub const fn from_parts(user: Option<UserId>, session_hash: Option<&'a str>) -> Option<Self> {
        match (user, session_hash) {
            (Some(user), _) => Some(Self::Account(user)),
            (None, Some(hash)) => Some(Self::Session(hash)),
            (None, None) => None,
        }
    }
}

pub async fn get_stats(
    conn: &mut Conn,
    tier_list: TierListId,
) -> Result<Option<StatsRow>, DbError> {
    Ok(sqlx::query_as!(
        StatsRow,
        "SELECT tier_list_id, view_count, unique_view_count, favorite_count, share_count,
                is_trending, trending_score, views_last_24h, views_last_7d, last_viewed_at,
                stats_updated_at
         FROM tier_list_stats WHERE tier_list_id = $1",
        tier_list.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn get_stats_for_lists(
    conn: &mut Conn,
    tier_lists: &[TierListId],
) -> Result<Vec<StatsRow>, DbError> {
    if tier_lists.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = tier_lists.iter().map(|t| t.0).collect();
    Ok(sqlx::query_as!(
        StatsRow,
        "SELECT tier_list_id, view_count, unique_view_count, favorite_count, share_count,
                is_trending, trending_score, views_last_24h, views_last_7d, last_viewed_at,
                stats_updated_at
         FROM tier_list_stats WHERE tier_list_id = ANY($1)",
        &ids,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn ensure_stats_row(conn: &mut Conn, tier_list: TierListId) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO tier_list_stats (tier_list_id) VALUES ($1) ON CONFLICT DO NOTHING",
        tier_list.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn record_view(
    conn: &mut Conn,
    tier_list: TierListId,
    viewer: Viewer<'_>,
) -> Result<bool, DbError> {
    let (user_id, session_hash) = match viewer {
        Viewer::Account(id) => (Some(id.0), None),
        Viewer::Session(hash) => (None, Some(hash)),
    };
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    sqlx::query!(
        "SELECT pg_advisory_xact_lock(
             hashtextextended($1::uuid::text || COALESCE($2::uuid::text, $3), 0)
         )",
        tier_list.0,
        user_id,
        session_hash,
    )
    .execute(&mut *tx)
    .await?;
    let counted = sqlx::query_scalar!(
        "WITH prior AS (
             SELECT GREATEST(
                 (SELECT MAX(viewed_at) FROM tier_list_view_events
                   WHERE tier_list_id = $1 AND user_id = $2),
                 (SELECT MAX(viewed_at) FROM tier_list_view_events
                   WHERE tier_list_id = $1 AND session_hash = $3)
             ) AS last_at
         ),
         counted AS (
             INSERT INTO tier_list_view_events (tier_list_id, user_id, session_hash)
             SELECT $1, $2, $3 FROM prior
              WHERE last_at IS NULL OR last_at <= NOW() - INTERVAL '30 minutes'
             RETURNING tier_list_id
         )
         INSERT INTO tier_list_stats (tier_list_id, view_count, unique_view_count, last_viewed_at)
         SELECT c.tier_list_id, 1, CASE WHEN p.last_at IS NULL THEN 1 ELSE 0 END, NOW()
           FROM counted c, prior p
         ON CONFLICT (tier_list_id)
         DO UPDATE SET
             view_count = tier_list_stats.view_count + 1,
             unique_view_count = tier_list_stats.unique_view_count + EXCLUDED.unique_view_count,
             last_viewed_at = NOW(),
             stats_updated_at = NOW()
         RETURNING tier_list_id",
        tier_list.0,
        user_id,
        session_hash,
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some();
    tx.commit().await?;
    Ok(counted)
}

pub async fn bump_share(conn: &mut Conn, tier_list: TierListId) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO tier_list_stats (tier_list_id, share_count)
         SELECT id, 1 FROM tier_lists WHERE id = $1
         ON CONFLICT (tier_list_id) DO UPDATE SET share_count = tier_list_stats.share_count + 1",
        tier_list.0,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn add_favorite(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"WITH added AS (
               INSERT INTO tier_list_favorites (tier_list_id, user_id) VALUES ($1, $2)
               ON CONFLICT DO NOTHING
               RETURNING tier_list_id
           ),
           counted AS (
               INSERT INTO tier_list_stats (tier_list_id, favorite_count)
               SELECT tier_list_id,
                      1 + (SELECT COUNT(*) FROM tier_list_favorites WHERE tier_list_id = $1)::int
               FROM added
               ON CONFLICT (tier_list_id)
               DO UPDATE SET favorite_count =
                   GREATEST(EXCLUDED.favorite_count, tier_list_stats.favorite_count + 1)
           )
           SELECT EXISTS (SELECT 1 FROM added) AS "added!""#,
        tier_list.0,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn remove_favorite(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"WITH removed AS (
               DELETE FROM tier_list_favorites WHERE tier_list_id = $1 AND user_id = $2
               RETURNING tier_list_id
           ),
           counted AS (
               UPDATE tier_list_stats SET favorite_count = GREATEST(favorite_count - 1, 0)
               WHERE tier_list_id IN (SELECT tier_list_id FROM removed)
           )
           SELECT EXISTS (SELECT 1 FROM removed) AS "removed!""#,
        tier_list.0,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn is_favorited(
    conn: &mut Conn,
    tier_list: TierListId,
    user: UserId,
) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM tier_list_favorites WHERE tier_list_id = $1 AND user_id = $2
           ) AS "exists!""#,
        tier_list.0,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn find_favorited_by_user(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<TierListRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredList,
        "SELECT tl.id, tl.name, tl.slug, tl.description, tl.list_type, tl.created_by,
                tl.is_active, tl.is_listed, tl.flair_id, tl.entity_kinds, tl.created_at,
                tl.updated_at
         FROM tier_lists tl
         JOIN tier_list_favorites f ON f.tier_list_id = tl.id
         WHERE f.user_id = $1 AND tl.is_active = true
         ORDER BY f.favorited_at DESC",
        user.0,
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(Into::into)
    .collect())
}

pub async fn recompute_trending(conn: &mut Conn, top_n: i64) -> Result<(), DbError> {
    sqlx::query!(
        "WITH scored AS (
             SELECT s.tier_list_id,
                    CASE WHEN tl.is_active THEN COALESCE(w.v24, 0) ELSE 0 END AS v24,
                    CASE WHEN tl.is_active THEN COALESCE(w.v7, 0) ELSE 0 END AS v7,
                    CASE WHEN tl.is_active THEN
                        (COALESCE(w.v24, 0) * 1.0
                         + COALESCE(f.f24, 0) * 5.0
                         + COALESCE(w.v7, 0) * 0.15
                         - LEAST(EXTRACT(EPOCH FROM (NOW() - tl.created_at)) / 3600.0, 720)
                           * 0.1)::float8
                    ELSE 0 END AS score
             FROM tier_list_stats s
             JOIN tier_lists tl ON tl.id = s.tier_list_id
             LEFT JOIN (
                 SELECT tier_list_id,
                        COUNT(*) FILTER (WHERE viewed_at > NOW() - INTERVAL '24 hours') AS v24,
                        COUNT(*) AS v7
                 FROM tier_list_view_events
                 WHERE viewed_at > NOW() - INTERVAL '7 days'
                 GROUP BY tier_list_id
             ) w ON w.tier_list_id = tl.id
             LEFT JOIN (
                 SELECT tier_list_id, COUNT(*) AS f24
                 FROM tier_list_favorites
                 WHERE favorited_at > NOW() - INTERVAL '24 hours'
                 GROUP BY tier_list_id
             ) f ON f.tier_list_id = tl.id
         ),
         ranked AS (
             SELECT tier_list_id, v24, v7, score, RANK() OVER (ORDER BY score DESC) AS rnk
             FROM scored
         )
         UPDATE tier_list_stats s SET
             views_last_24h = r.v24,
             views_last_7d = r.v7,
             trending_score = r.score,
             is_trending = r.rnk <= $1 AND r.score > 0,
             stats_updated_at = NOW()
         FROM ranked r
         WHERE s.tier_list_id = r.tier_list_id",
        top_n,
    )
    .execute(conn)
    .await?;
    Ok(())
}
