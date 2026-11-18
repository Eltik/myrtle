use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct BuildRow {
    pub operator_id: String,
    pub elite: i16,
    pub level: i16,
    pub potential: i16,
    pub skill_level: i16,
    pub masteries: Vec<i16>,
    pub module_ids: Vec<String>,
    pub module_levels: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct SummaryExtrasRow {
    pub server_id: i16,
    pub skins: i64,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct NewSnapshot<'a> {
    pub period_start: i64,
    pub activity_id: Option<&'a str>,
    pub activity_name: Option<&'a str>,
    pub summary: &'a Value,
    pub build: &'a Value,
}

#[derive(Debug, Clone)]
pub struct SnapshotRow {
    pub period_start: DateTime<Utc>,
    pub taken_at: DateTime<Utc>,
    pub activity_id: Option<String>,
    pub activity_name: Option<String>,
    pub summary: Value,
}

#[derive(Debug, Clone)]
pub struct SnapshotBuildRow {
    pub taken_at: DateTime<Utc>,
    pub summary: Value,
    pub build: Value,
}

fn at(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap_or_default()
}

pub async fn period_taken(
    conn: &mut Conn,
    user: UserId,
    period_start: i64,
) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM user_lookback_snapshots
                          WHERE user_id = $1 AND period_start = $2) AS "taken!""#,
        user.0,
        at(period_start),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn build_rows(conn: &mut Conn, user: UserId) -> Result<Vec<BuildRow>, DbError> {
    Ok(sqlx::query_as!(
        BuildRow,
        r#"SELECT o.operator_id::text AS "operator_id!", o.elite, o.level, o.potential, o.skill_level,
             COALESCE((SELECT array_agg(s.specialize_level ORDER BY s.skill_index)
                       FROM user_operator_skills s
                       WHERE s.user_id = o.user_id AND s.operator_id = o.operator_id),
                      '{}') AS "masteries!",
             COALESCE(md.ids, '{}') AS "module_ids!", COALESCE(md.levels, '{}') AS "module_levels!"
         FROM user_operators o
         CROSS JOIN LATERAL (
             SELECT array_agg(m.module_id::text ORDER BY m.module_id) AS ids,
                    array_agg(m.module_level ORDER BY m.module_id) AS levels
             FROM user_operator_modules m
             WHERE m.user_id = o.user_id AND m.operator_id = o.operator_id AND NOT m.locked
         ) md
         WHERE o.user_id = $1 ORDER BY o.operator_id"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn summary_extras(
    conn: &mut Conn,
    user: UserId,
) -> Result<Option<SummaryExtrasRow>, DbError> {
    Ok(sqlx::query_as!(
        SummaryExtrasRow,
        r#"SELECT u.server_id,
             (SELECT COUNT(*) FROM user_skins sk
              WHERE sk.user_id = u.id AND sk.skin_id LIKE '%@%') AS "skins!",
             sc.total_score AS "total_score?", sc.grade::text AS grade
         FROM users u LEFT JOIN user_scores sc ON sc.user_id = u.id WHERE u.id = $1"#,
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn insert(
    conn: &mut Conn,
    user: UserId,
    snapshot: &NewSnapshot<'_>,
    keep_every_days: i32,
) -> Result<bool, DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    let written = sqlx::query!(
        "INSERT INTO user_lookback_snapshots
             (user_id, period_start, activity_id, activity_name, summary, build)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, period_start) DO NOTHING",
        user.0,
        at(snapshot.period_start),
        snapshot.activity_id,
        snapshot.activity_name,
        snapshot.summary,
        snapshot.build,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if written {
        prune(&mut tx, user, keep_every_days).await?;
    }
    tx.commit().await?;
    Ok(written)
}

async fn prune(conn: &mut Conn, user: UserId, keep_every_days: i32) -> Result<u64, DbError> {
    Ok(sqlx::query!(
        "DELETE FROM user_lookback_snapshots s
         WHERE s.user_id = $1 AND s.period_start < now() - make_interval(days => $2)
           AND EXISTS (SELECT 1 FROM user_lookback_snapshots e
                       WHERE e.user_id = s.user_id AND e.period_start < s.period_start
                         AND date_trunc('month', e.period_start AT TIME ZONE 'UTC')
                           = date_trunc('month', s.period_start AT TIME ZONE 'UTC'))",
        user.0,
        keep_every_days,
    )
    .execute(conn)
    .await?
    .rows_affected())
}

pub async fn list(conn: &mut Conn, user: UserId) -> Result<Vec<SnapshotRow>, DbError> {
    Ok(sqlx::query_as!(
        SnapshotRow,
        "SELECT period_start, taken_at, activity_id::text AS activity_id, activity_name, summary
         FROM user_lookback_snapshots WHERE user_id = $1 ORDER BY period_start",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get(
    conn: &mut Conn,
    user: UserId,
    period_start: i64,
) -> Result<Option<SnapshotBuildRow>, DbError> {
    Ok(sqlx::query_as!(
        SnapshotBuildRow,
        "SELECT taken_at, summary, build FROM user_lookback_snapshots
         WHERE user_id = $1 AND period_start = $2",
        user.0,
        at(period_start),
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn obtained_times(conn: &mut Conn, user: UserId) -> Result<Vec<i64>, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COALESCE(obtained_at, 0) AS "obtained_at!" FROM user_operators WHERE user_id = $1"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}
