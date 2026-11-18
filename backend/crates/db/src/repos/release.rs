use chrono::{DateTime, Utc};
use myrtle_types::{Server, UserId};
use serde_json::Value;

use crate::{Conn, DbError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sighting<'a> {
    pub kind: &'a str,
    pub id: &'a str,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpsertStats {
    pub inserted: u64,
    pub refreshed: u64,
}

#[derive(Debug, Clone)]
pub struct OverrideRow {
    pub kind: String,
    pub cn_id: String,
    pub en_id: Option<String>,
    pub en_name: Option<String>,
    pub en_start: Option<i64>,
    pub en_end: Option<i64>,
    pub featured_chars: Option<Vec<String>>,
    pub source: String,
    pub note: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct OverrideInput<'a> {
    pub kind: &'a str,
    pub cn_id: &'a str,
    pub en_id: Option<&'a str>,
    pub en_name: Option<&'a str>,
    pub en_start: Option<i64>,
    pub en_end: Option<i64>,
    pub featured_chars: Option<&'a [String]>,
    pub source: &'a str,
    pub note: &'a str,
    pub updated_by: Option<UserId>,
}

#[derive(Debug, Clone)]
pub struct PlanRow {
    pub initial: i32,
    pub initial_manual: bool,
    pub picks: Value,
    pub stages: Value,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_sightings(
    conn: &mut Conn,
    server: Server,
    res_version: Option<&str>,
    rows: &[Sighting<'_>],
) -> Result<UpsertStats, DbError> {
    if rows.is_empty() {
        return Ok(UpsertStats::default());
    }
    let kinds: Vec<&str> = rows.iter().map(|r| r.kind).collect();
    let ids: Vec<&str> = rows.iter().map(|r| r.id).collect();
    let starts: Vec<Option<i64>> = rows.iter().map(|r| r.start_time).collect();
    let ends: Vec<Option<i64>> = rows.iter().map(|r| r.end_time).collect();

    let flags = sqlx::query_scalar!(
        r#"INSERT INTO gamedata_sightings (server_id, kind, id, res_version, start_time, end_time)
           SELECT $1, k, i, $2, s, e
           FROM UNNEST($3::text[], $4::text[], $5::bigint[], $6::bigint[]) AS t(k, i, s, e)
           ON CONFLICT (server_id, kind, id) DO UPDATE SET
               last_seen_at = now(),
               start_time = EXCLUDED.start_time,
               end_time = EXCLUDED.end_time
           RETURNING (xmax = 0) AS "inserted!""#,
        server.index(),
        res_version,
        &kinds as _,
        &ids as _,
        &starts as _,
        &ends as _,
    )
    .fetch_all(conn)
    .await?;

    let inserted = flags.iter().filter(|f| **f).count() as u64;
    Ok(UpsertStats {
        inserted,
        refreshed: flags.len() as u64 - inserted,
    })
}

pub async fn debut_pairs(
    conn: &mut Conn,
    server: Server,
) -> Result<Vec<(String, String)>, DbError> {
    let rows = sqlx::query!(
        "SELECT p.id AS pool_id, c.id AS char_id
         FROM gamedata_sightings p
         JOIN gamedata_sightings c
           ON c.server_id = p.server_id AND c.kind = 'char'
          AND c.res_version = p.res_version
         WHERE p.server_id = $1 AND p.kind = 'pool' AND p.res_version IS NOT NULL
         ORDER BY p.id, c.id",
        server.index(),
    )
    .fetch_all(conn)
    .await?;
    Ok(rows.into_iter().map(|r| (r.pool_id, r.char_id)).collect())
}

pub async fn list_overrides(conn: &mut Conn) -> Result<Vec<OverrideRow>, DbError> {
    Ok(sqlx::query_as!(
        OverrideRow,
        "SELECT kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_at
         FROM release_overrides ORDER BY updated_at DESC"
    )
    .fetch_all(conn)
    .await?)
}

pub async fn put_override(conn: &mut Conn, o: &OverrideInput<'_>) -> Result<OverrideRow, DbError> {
    Ok(sqlx::query_as!(
        OverrideRow,
        "INSERT INTO release_overrides
             (kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note,
              updated_by, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
         ON CONFLICT (kind, cn_id) DO UPDATE SET
             en_id = EXCLUDED.en_id, en_name = EXCLUDED.en_name,
             en_start = EXCLUDED.en_start, en_end = EXCLUDED.en_end,
             featured_chars = EXCLUDED.featured_chars,
             source = EXCLUDED.source, note = EXCLUDED.note,
             updated_by = EXCLUDED.updated_by, updated_at = now()
         RETURNING kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note,
                   updated_at",
        o.kind,
        o.cn_id,
        o.en_id,
        o.en_name,
        o.en_start,
        o.en_end,
        o.featured_chars,
        o.source,
        o.note,
        o.updated_by.map(|u| u.0),
    )
    .fetch_one(conn)
    .await?)
}

pub async fn overrides_by_source(
    conn: &mut Conn,
    source: &str,
) -> Result<Vec<OverrideRow>, DbError> {
    Ok(sqlx::query_as!(
        OverrideRow,
        "SELECT kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_at
         FROM release_overrides WHERE source = $1",
        source,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn delete_override(conn: &mut Conn, kind: &str, cn_id: &str) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM release_overrides WHERE kind = $1 AND cn_id = $2",
        kind,
        cn_id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_plan(conn: &mut Conn, user: UserId) -> Result<Option<PlanRow>, DbError> {
    Ok(sqlx::query_as!(
        PlanRow,
        "SELECT initial, initial_manual, picks, stages, updated_at FROM release_plans WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn put_plan(
    conn: &mut Conn,
    user: UserId,
    initial: i32,
    initial_manual: bool,
    picks: &Value,
    stages: &Value,
) -> Result<PlanRow, DbError> {
    Ok(sqlx::query_as!(
        PlanRow,
        "INSERT INTO release_plans (user_id, initial, initial_manual, picks, stages, updated_at)
         VALUES ($1, $2, $3, $4, $5, now())
         ON CONFLICT (user_id) DO UPDATE SET
             initial = EXCLUDED.initial, initial_manual = EXCLUDED.initial_manual,
             picks = EXCLUDED.picks, stages = EXCLUDED.stages, updated_at = now()
         RETURNING initial, initial_manual, picks, stages, updated_at",
        user.0,
        initial,
        initial_manual,
        picks,
        stages,
    )
    .fetch_one(conn)
    .await?)
}
