use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SightingInput {
    pub kind: &'static str,
    pub id: String,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UpsertStats {
    pub inserted: u64,
    pub refreshed: u64,
}

pub async fn upsert_sightings(
    pool: &PgPool,
    server_id: i16,
    res_version: Option<&str>,
    rows: &[SightingInput],
) -> Result<UpsertStats, sqlx::Error> {
    if rows.is_empty() {
        return Ok(UpsertStats::default());
    }
    let kinds: Vec<&str> = rows.iter().map(|r| r.kind).collect();
    let ids: Vec<&str> = rows.iter().map(|r| r.id.as_str()).collect();
    let starts: Vec<Option<i64>> = rows.iter().map(|r| r.start_time).collect();
    let ends: Vec<Option<i64>> = rows.iter().map(|r| r.end_time).collect();

    let flags: Vec<bool> = sqlx::query_scalar(
        "INSERT INTO gamedata_sightings \
             (server_id, kind, id, res_version, start_time, end_time) \
         SELECT $1, k, i, $2, s, e \
         FROM UNNEST($3::text[], $4::text[], $5::bigint[], $6::bigint[]) AS t(k, i, s, e) \
         ON CONFLICT (server_id, kind, id) DO UPDATE SET \
             last_seen_at = now(), \
             start_time = EXCLUDED.start_time, \
             end_time = EXCLUDED.end_time \
         RETURNING (xmax = 0) AS inserted",
    )
    .bind(server_id)
    .bind(res_version)
    .bind(&kinds)
    .bind(&ids)
    .bind(&starts)
    .bind(&ends)
    .fetch_all(pool)
    .await?;

    let inserted = flags.iter().filter(|f| **f).count() as u64;
    Ok(UpsertStats {
        inserted,
        refreshed: flags.len() as u64 - inserted,
    })
}

pub async fn debut_pairs(
    pool: &PgPool,
    server_id: i16,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT p.id, c.id \
         FROM gamedata_sightings p \
         JOIN gamedata_sightings c \
           ON c.server_id = p.server_id AND c.kind = 'char' \
          AND c.res_version = p.res_version \
         WHERE p.server_id = $1 AND p.kind = 'pool' AND p.res_version IS NOT NULL \
         ORDER BY p.id, c.id",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
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

pub async fn list_overrides(pool: &PgPool) -> Result<Vec<OverrideRow>, sqlx::Error> {
    sqlx::query_as::<_, OverrideRow>(
        "SELECT kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_at \
         FROM release_overrides ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverrideInput {
    pub kind: String,
    pub cn_id: String,
    pub en_id: Option<String>,
    pub en_name: Option<String>,
    pub en_start: Option<i64>,
    pub en_end: Option<i64>,
    pub featured_chars: Option<Vec<String>>,
    pub source: String,
    pub note: String,
    pub updated_by: Option<Uuid>,
}

pub async fn put_override(pool: &PgPool, o: &OverrideInput) -> Result<OverrideRow, sqlx::Error> {
    sqlx::query_as::<_, OverrideRow>(
        "INSERT INTO release_overrides \
             (kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_by, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now()) \
         ON CONFLICT (kind, cn_id) DO UPDATE SET \
             en_id = EXCLUDED.en_id, en_name = EXCLUDED.en_name, \
             en_start = EXCLUDED.en_start, en_end = EXCLUDED.en_end, \
             featured_chars = EXCLUDED.featured_chars, \
             source = EXCLUDED.source, note = EXCLUDED.note, \
             updated_by = EXCLUDED.updated_by, updated_at = now() \
         RETURNING kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_at",
    )
    .bind(&o.kind)
    .bind(&o.cn_id)
    .bind(&o.en_id)
    .bind(&o.en_name)
    .bind(o.en_start)
    .bind(o.en_end)
    .bind(&o.featured_chars)
    .bind(&o.source)
    .bind(&o.note)
    .bind(o.updated_by)
    .fetch_one(pool)
    .await
}

pub async fn overrides_by_source(
    pool: &PgPool,
    source: &str,
) -> Result<Vec<OverrideRow>, sqlx::Error> {
    sqlx::query_as::<_, OverrideRow>(
        "SELECT kind, cn_id, en_id, en_name, en_start, en_end, featured_chars, source, note, updated_at \
         FROM release_overrides WHERE source = $1",
    )
    .bind(source)
    .fetch_all(pool)
    .await
}

pub async fn delete_override(pool: &PgPool, kind: &str, cn_id: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM release_overrides WHERE kind = $1 AND cn_id = $2")
        .bind(kind)
        .bind(cn_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlanRow {
    pub initial: i32,
    pub initial_manual: bool,
    pub picks: serde_json::Value,
    pub stages: serde_json::Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_plan(pool: &PgPool, user_id: Uuid) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT initial, initial_manual, picks, stages, updated_at FROM release_plans WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

pub async fn put_plan(
    pool: &PgPool,
    user_id: Uuid,
    initial: i32,
    initial_manual: bool,
    picks: &serde_json::Value,
    stages: &serde_json::Value,
) -> Result<PlanRow, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        r"
        INSERT INTO release_plans (user_id, initial, initial_manual, picks, stages, updated_at)
        VALUES ($1, $2, $3, $4, $5, now())
        ON CONFLICT (user_id) DO UPDATE SET
            initial = EXCLUDED.initial, initial_manual = EXCLUDED.initial_manual,
            picks = EXCLUDED.picks, stages = EXCLUDED.stages, updated_at = now()
        RETURNING initial, initial_manual, picks, stages, updated_at
        ",
    )
    .bind(user_id)
    .bind(initial)
    .bind(initial_manual)
    .bind(picks)
    .bind(stages)
    .fetch_one(pool)
    .await
}
