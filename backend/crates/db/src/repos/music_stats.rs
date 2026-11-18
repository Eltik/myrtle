use chrono::{DateTime, Utc};
use myrtle_types::UserId;

use crate::{Conn, DbError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatKind {
    Listen,
    Download,
}

impl StatKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Download => "download",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerSort {
    Listens,
    Time,
}

#[derive(Debug, Clone)]
pub struct TrackCountsRow {
    pub track_id: String,
    pub listens: i64,
    pub downloads: i64,
}

#[derive(Debug, Clone)]
pub struct TopTrackRow {
    pub rank: i64,
    pub track_id: String,
    pub listens: i64,
}

#[derive(Debug, Clone)]
pub struct ListenerRow {
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub listens: i64,
    pub listened_seconds: f64,
}

pub async fn claim_window(
    conn: &mut Conn,
    kind: StatKind,
    client: &str,
    track_id: &str,
    at: DateTime<Utc>,
    window_secs: f64,
) -> Result<bool, DbError> {
    let claimed = sqlx::query_scalar!(
        r#"INSERT INTO music_stat_dedupe (kind, client, track_id, counted_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (kind, client, track_id) DO UPDATE SET counted_at = EXCLUDED.counted_at
           WHERE music_stat_dedupe.counted_at <= EXCLUDED.counted_at - make_interval(secs => $5)
           RETURNING true AS "claimed!""#,
        kind.as_str(),
        client,
        track_id,
        at,
        window_secs,
    )
    .fetch_optional(conn)
    .await?;
    Ok(claimed.is_some())
}

pub async fn insert_event(
    conn: &mut Conn,
    kind: StatKind,
    track_id: &str,
    user: Option<UserId>,
    at: DateTime<Utc>,
) -> Result<(), DbError> {
    let (listens, downloads) = match kind {
        StatKind::Listen => (1_i64, 0_i64),
        StatKind::Download => (0, 1),
    };
    sqlx::query!(
        "WITH ev AS (
             INSERT INTO music_stat_events (kind, track_id, user_id, created_at)
             VALUES ($1, $2, $3, $4)
         )
         INSERT INTO music_track_stats (track_id, listens, downloads, updated_at)
         VALUES ($2, $5, $6, $4)
         ON CONFLICT (track_id) DO UPDATE SET
             listens = music_track_stats.listens + EXCLUDED.listens,
             downloads = music_track_stats.downloads + EXCLUDED.downloads,
             updated_at = EXCLUDED.updated_at",
        kind.as_str(),
        track_id,
        user.map(|u| u.0),
        at,
        listens,
        downloads,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn sweep_dedupe(conn: &mut Conn, before: DateTime<Utc>) -> Result<u64, DbError> {
    let result = sqlx::query!(
        "DELETE FROM music_stat_dedupe WHERE counted_at < $1",
        before,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}

pub async fn siren_song_exists(conn: &mut Conn, cid: &str) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM siren_songs WHERE cid = $1) AS "exists!""#,
        cid,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn all_counts(conn: &mut Conn) -> Result<Vec<TrackCountsRow>, DbError> {
    Ok(sqlx::query_as!(
        TrackCountsRow,
        "SELECT track_id, listens, downloads FROM music_track_stats
         WHERE listens > 0 OR downloads > 0",
    )
    .fetch_all(conn)
    .await?)
}

pub async fn top_tracks(
    conn: &mut Conn,
    since: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<TopTrackRow>, DbError> {
    Ok(match since {
        Some(since) => {
            sqlx::query_as!(
                TopTrackRow,
                r#"SELECT rank() OVER (ORDER BY count(*) DESC) AS "rank!", track_id,
                      count(*) AS "listens!"
               FROM music_stat_events
               WHERE kind = 'listen' AND created_at >= $1
               GROUP BY track_id
               ORDER BY count(*) DESC, track_id
               LIMIT $2"#,
                since,
                limit,
            )
            .fetch_all(conn)
            .await?
        }
        None => {
            sqlx::query_as!(
                TopTrackRow,
                r#"SELECT rank() OVER (ORDER BY listens DESC) AS "rank!", track_id, listens
               FROM music_track_stats
               WHERE listens > 0
               ORDER BY listens DESC, track_id
               LIMIT $1"#,
                limit,
            )
            .fetch_all(conn)
            .await?
        }
    })
}

pub async fn listeners(
    conn: &mut Conn,
    since: DateTime<Utc>,
    track_id: Option<&str>,
    sort: ListenerSort,
    limit: i64,
    offset: i64,
) -> Result<Vec<ListenerRow>, DbError> {
    Ok(sqlx::query_as!(
        ListenerRow,
        r#"WITH ev AS (
               SELECT user_id, count(*) AS listens
               FROM music_stat_events
               WHERE kind = 'listen' AND user_id IS NOT NULL AND created_at >= $1
                 AND ($2::text IS NULL OR track_id = $2)
               GROUP BY user_id
           ),
           hist AS (
               SELECT h.user_id, sum(least(h.listened_seconds, 1200))::float8 AS seconds
               FROM music_history h
               WHERE h.played_at >= $1
                 AND ($2::text IS NULL OR h.track_id = $2)
                 AND EXISTS (SELECT 1 FROM music_track_stats ts WHERE ts.track_id = h.track_id)
               GROUP BY h.user_id
           ),
           listeners AS (
               SELECT coalesce(ev.user_id, hist.user_id) AS user_id,
                      coalesce(ev.listens, 0) AS listens,
                      coalesce(hist.seconds, 0)::float8 AS seconds
               FROM ev FULL JOIN hist ON hist.user_id = ev.user_id
           )
           SELECT rank() OVER (
                      ORDER BY CASE WHEN $3::bool THEN l.seconds END DESC, l.listens DESC, l.seconds DESC
                  ) AS "rank!",
                  u.uid, u.nickname, u.nick_number,
                  CASE WHEN NOT EXISTS (
                      SELECT 1 FROM user_settings hs
                      WHERE hs.user_id = u.id AND hs.profile_layout -> 'header' -> 'hidden' @> '["level"]'::jsonb
                  ) THEN u.level END AS level,
                  u.avatar_id, s.code AS server,
                  l.listens AS "listens!", l.seconds AS "listened_seconds!"
           FROM listeners l
           JOIN users u ON u.id = l.user_id
           JOIN servers s ON s.id = u.server_id
           WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)
           ORDER BY CASE WHEN $3::bool THEN l.seconds END DESC, l.listens DESC, l.seconds DESC, u.uid
           LIMIT $4 OFFSET $5"#,
        since,
        track_id,
        matches!(sort, ListenerSort::Time),
        limit,
        offset,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_listeners(
    conn: &mut Conn,
    since: DateTime<Utc>,
    track_id: Option<&str>,
) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"WITH ev AS (
               SELECT user_id, count(*) AS listens
               FROM music_stat_events
               WHERE kind = 'listen' AND user_id IS NOT NULL AND created_at >= $1
                 AND ($2::text IS NULL OR track_id = $2)
               GROUP BY user_id
           ),
           hist AS (
               SELECT h.user_id, sum(least(h.listened_seconds, 1200))::float8 AS seconds
               FROM music_history h
               WHERE h.played_at >= $1
                 AND ($2::text IS NULL OR h.track_id = $2)
                 AND EXISTS (SELECT 1 FROM music_track_stats ts WHERE ts.track_id = h.track_id)
               GROUP BY h.user_id
           ),
           listeners AS (
               SELECT coalesce(ev.user_id, hist.user_id) AS user_id,
                      coalesce(ev.listens, 0) AS listens,
                      coalesce(hist.seconds, 0)::float8 AS seconds
               FROM ev FULL JOIN hist ON hist.user_id = ev.user_id
           )
           SELECT count(*) AS "count!" FROM listeners l
           JOIN users u ON u.id = l.user_id
           WHERE EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)"#,
        since,
        track_id,
    )
    .fetch_one(conn)
    .await?)
}
