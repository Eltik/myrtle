use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct GachaRecordRow {
    pub id: i64,
    pub user_id: Uuid,
    pub char_id: String,
    pub pool_id: String,
    pub rarity: i16,
    pub pull_timestamp: i64,
    pub pool_name: Option<String>,
    pub gacha_type: Option<String>,
    pub batch_index: i16,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GachaStatsRow {
    pub user_id: Uuid,
    pub total_pulls: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub four_star_count: i64,
    pub first_pull: i64,
    pub last_pull: i64,
}

#[derive(Debug, Clone)]
pub struct GachaFlagsRow {
    pub store_gacha: bool,
    pub share_stats: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum BatchOutcome {
    Stored,
    StorageOff,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HistoryFilter<'a> {
    pub rarity: Option<i16>,
    pub gacha_types: Option<&'a [String]>,
    pub char_id: Option<&'a str>,
    pub from_ts: Option<i64>,
    pub to_ts: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SharedTotalsRow {
    pub total_pulls: i64,
    pub total_users: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub four_star_count: i64,
    pub three_star_count: i64,
    pub first_pull_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct OperatorPullsRow {
    pub char_id: String,
    pub rarity: i16,
    pub pull_count: i64,
}

#[derive(Debug, Clone)]
pub struct HourPullsRow {
    pub hour: i32,
    pub pull_count: i64,
}

#[derive(Debug, Clone)]
pub struct WeekdayPullsRow {
    pub day: i32,
    pub pull_count: i64,
}

#[derive(Debug, Clone)]
pub struct DatePullsRow {
    /// `YYYY-MM-DD`
    pub date: String,
    pub pull_count: i64,
}

#[derive(Debug, Clone)]
pub struct BannerPullsRow {
    pub pool_id: String,
    pub pull_count: i64,
    pub six_star_count: i64,
    pub five_star_count: i64,
    pub user_count: i64,
}

#[derive(Debug, Clone)]
pub struct CharRarityRow {
    pub char_id: String,
    pub rarity: i16,
}

pub async fn store_gacha_enabled(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    let stored = sqlx::query_scalar!(
        "SELECT store_gacha FROM user_settings WHERE user_id = $1 FOR SHARE",
        user.0,
    )
    .fetch_optional(conn)
    .await?;
    Ok(stored.unwrap_or(true))
}

pub async fn insert_batch(
    conn: &mut Conn,
    user: UserId,
    records: &Value,
) -> Result<BatchOutcome, DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    if !store_gacha_enabled(&mut tx, user).await? {
        return Ok(BatchOutcome::StorageOff);
    }
    sqlx::query!("CALL sp_insert_gacha_batch($1, $2)", user.0, records)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(BatchOutcome::Stored)
}

pub async fn history(
    conn: &mut Conn,
    user: UserId,
    rarity: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<GachaRecordRow>, DbError> {
    Ok(sqlx::query_as!(
        GachaRecordRow,
        "SELECT id, user_id, char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type,
                batch_index, created_at
         FROM gacha_records
         WHERE user_id = $1 AND ($2::smallint IS NULL OR rarity = $2)
         ORDER BY pull_timestamp DESC, batch_index ASC, id ASC
         LIMIT $3 OFFSET $4",
        user.0,
        rarity,
        limit,
        offset,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn history_filtered(
    conn: &mut Conn,
    user: UserId,
    filter: &HistoryFilter<'_>,
    newest_first: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<GachaRecordRow>, i64), DbError> {
    let rows = if newest_first {
        sqlx::query_as!(
            GachaRecordRow,
            "SELECT id, user_id, char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type,
                    batch_index, created_at
             FROM gacha_records
             WHERE user_id = $1
               AND ($2::smallint IS NULL OR rarity = $2)
               AND ($3::text[] IS NULL OR gacha_type = ANY($3))
               AND ($4::text IS NULL OR char_id = $4)
               AND ($5::bigint IS NULL OR pull_timestamp >= $5)
               AND ($6::bigint IS NULL OR pull_timestamp <= $6)
             ORDER BY pull_timestamp DESC, batch_index ASC, id ASC
             LIMIT $7 OFFSET $8",
            user.0,
            filter.rarity,
            filter.gacha_types,
            filter.char_id,
            filter.from_ts,
            filter.to_ts,
            limit,
            offset,
        )
        .fetch_all(&mut *conn)
        .await?
    } else {
        sqlx::query_as!(
            GachaRecordRow,
            "SELECT id, user_id, char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type,
                    batch_index, created_at
             FROM gacha_records
             WHERE user_id = $1
               AND ($2::smallint IS NULL OR rarity = $2)
               AND ($3::text[] IS NULL OR gacha_type = ANY($3))
               AND ($4::text IS NULL OR char_id = $4)
               AND ($5::bigint IS NULL OR pull_timestamp >= $5)
               AND ($6::bigint IS NULL OR pull_timestamp <= $6)
             ORDER BY pull_timestamp ASC, batch_index ASC, id ASC
             LIMIT $7 OFFSET $8",
            user.0,
            filter.rarity,
            filter.gacha_types,
            filter.char_id,
            filter.from_ts,
            filter.to_ts,
            limit,
            offset,
        )
        .fetch_all(&mut *conn)
        .await?
    };

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM gacha_records
           WHERE user_id = $1
             AND ($2::smallint IS NULL OR rarity = $2)
             AND ($3::text[] IS NULL OR gacha_type = ANY($3))
             AND ($4::text IS NULL OR char_id = $4)
             AND ($5::bigint IS NULL OR pull_timestamp >= $5)
             AND ($6::bigint IS NULL OR pull_timestamp <= $6)"#,
        user.0,
        filter.rarity,
        filter.gacha_types,
        filter.char_id,
        filter.from_ts,
        filter.to_ts,
    )
    .fetch_one(&mut *conn)
    .await?;

    Ok((rows, total))
}

pub async fn list_in_pools(
    conn: &mut Conn,
    user: UserId,
    pools: Option<&[String]>,
) -> Result<Vec<GachaRecordRow>, DbError> {
    Ok(sqlx::query_as!(
        GachaRecordRow,
        "SELECT id, user_id, char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type,
                batch_index, created_at
         FROM gacha_records
         WHERE user_id = $1 AND ($2::text[] IS NULL OR pool_id = ANY($2))
         ORDER BY pull_timestamp DESC, batch_index ASC, id ASC",
        user.0,
        pools,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_for_char(
    conn: &mut Conn,
    user: UserId,
    char_id: &str,
) -> Result<Vec<GachaRecordRow>, DbError> {
    Ok(sqlx::query_as!(
        GachaRecordRow,
        "SELECT id, user_id, char_id, pool_id, rarity, pull_timestamp, pool_name, gacha_type,
                batch_index, created_at
         FROM gacha_records
         WHERE user_id = $1 AND char_id = $2
         ORDER BY pull_timestamp DESC, batch_index ASC, id ASC",
        user.0,
        char_id,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn stats(conn: &mut Conn, user: UserId) -> Result<Option<GachaStatsRow>, DbError> {
    Ok(sqlx::query_as!(
        GachaStatsRow,
        r#"SELECT user_id AS "user_id!", total_pulls AS "total_pulls!",
                  six_star_count AS "six_star_count!", five_star_count AS "five_star_count!",
                  four_star_count AS "four_star_count!", first_pull AS "first_pull!",
                  last_pull AS "last_pull!"
           FROM v_gacha_stats WHERE user_id = $1"#,
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn public_history_enabled(conn: &mut Conn, user: UserId) -> Result<bool, DbError> {
    let public = sqlx::query_scalar!(
        "SELECT public_gacha FROM user_settings WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?;
    Ok(public.unwrap_or(false))
}

pub async fn flags(conn: &mut Conn, user: UserId) -> Result<GachaFlagsRow, DbError> {
    if let Some(row) = sqlx::query_as!(
        GachaFlagsRow,
        "SELECT store_gacha, share_stats FROM user_settings WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(&mut *conn)
    .await?
    {
        return Ok(row);
    }
    Ok(sqlx::query_as!(
        GachaFlagsRow,
        "INSERT INTO user_settings (user_id) VALUES ($1)
         ON CONFLICT (user_id) DO UPDATE SET user_id = EXCLUDED.user_id
         RETURNING store_gacha, share_stats",
        user.0,
    )
    .fetch_one(&mut *conn)
    .await?)
}

pub async fn update_flags(
    conn: &mut Conn,
    user: UserId,
    store_gacha: Option<bool>,
    share_stats: Option<bool>,
) -> Result<GachaFlagsRow, DbError> {
    Ok(sqlx::query_as!(
        GachaFlagsRow,
        "INSERT INTO user_settings AS s (user_id, store_gacha, share_stats)
         VALUES ($1, COALESCE($2, true), COALESCE($3, true))
         ON CONFLICT (user_id) DO UPDATE SET
             store_gacha = COALESCE($2, s.store_gacha),
             share_stats = COALESCE($3, s.share_stats)
         RETURNING store_gacha, share_stats",
        user.0,
        store_gacha,
        share_stats,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn shared_totals(conn: &mut Conn) -> Result<SharedTotalsRow, DbError> {
    Ok(sqlx::query_as!(
        SharedTotalsRow,
        r#"SELECT
               COUNT(*) AS "total_pulls!",
               COUNT(DISTINCT gr.user_id) AS "total_users!",
               COUNT(*) FILTER (WHERE gr.rarity = 6) AS "six_star_count!",
               COUNT(*) FILTER (WHERE gr.rarity = 5) AS "five_star_count!",
               COUNT(*) FILTER (WHERE gr.rarity = 4) AS "four_star_count!",
               COUNT(*) FILTER (WHERE gr.rarity = 3) AS "three_star_count!",
               (MIN(gr.pull_timestamp) / 1000) AS "first_pull_at?"
           FROM gacha_records gr
           JOIN user_settings us ON us.user_id = gr.user_id
           WHERE us.share_stats = true"#
    )
    .fetch_one(conn)
    .await?)
}

pub async fn shared_top_operators(
    conn: &mut Conn,
    per_rarity: i64,
) -> Result<Vec<OperatorPullsRow>, DbError> {
    Ok(sqlx::query_as!(
        OperatorPullsRow,
        r#"SELECT char_id AS "char_id!", rarity AS "rarity!", pull_count AS "pull_count!" FROM (
               SELECT gr.char_id, gr.rarity, COUNT(*) AS pull_count,
                      ROW_NUMBER() OVER (PARTITION BY gr.rarity ORDER BY COUNT(*) DESC, gr.char_id) AS rn
               FROM gacha_records gr
               JOIN user_settings us ON us.user_id = gr.user_id
               WHERE us.share_stats = true
               GROUP BY gr.char_id, gr.rarity
           ) t
           WHERE rn <= $1
           ORDER BY rarity DESC, pull_count DESC, char_id"#,
        per_rarity,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn shared_pulls_by_hour(conn: &mut Conn) -> Result<Vec<HourPullsRow>, DbError> {
    Ok(sqlx::query_as!(
        HourPullsRow,
        r#"SELECT EXTRACT(HOUR FROM to_timestamp(gr.pull_timestamp / 1000) AT TIME ZONE 'UTC')::int AS "hour!",
                  COUNT(*) AS "pull_count!"
           FROM gacha_records gr
           JOIN user_settings us ON us.user_id = gr.user_id
           WHERE us.share_stats = true
           GROUP BY 1
           ORDER BY 1"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn shared_pulls_by_weekday(conn: &mut Conn) -> Result<Vec<WeekdayPullsRow>, DbError> {
    Ok(sqlx::query_as!(
        WeekdayPullsRow,
        r#"SELECT EXTRACT(DOW FROM to_timestamp(gr.pull_timestamp / 1000) AT TIME ZONE 'UTC')::int AS "day!",
                  COUNT(*) AS "pull_count!"
           FROM gacha_records gr
           JOIN user_settings us ON us.user_id = gr.user_id
           WHERE us.share_stats = true
           GROUP BY 1
           ORDER BY 1"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn shared_pulls_by_date(conn: &mut Conn) -> Result<Vec<DatePullsRow>, DbError> {
    Ok(sqlx::query_as!(
        DatePullsRow,
        r#"SELECT to_char(to_timestamp(gr.pull_timestamp / 1000) AT TIME ZONE 'UTC', 'YYYY-MM-DD') AS "date!",
                  COUNT(*) AS "pull_count!"
           FROM gacha_records gr
           JOIN user_settings us ON us.user_id = gr.user_id
           WHERE us.share_stats = true
           GROUP BY 1
           ORDER BY 1"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn shared_banner_totals(conn: &mut Conn) -> Result<Vec<BannerPullsRow>, DbError> {
    Ok(sqlx::query_as!(
        BannerPullsRow,
        r#"SELECT
               gr.pool_id AS "pool_id!",
               COUNT(*) AS "pull_count!",
               COUNT(*) FILTER (WHERE gr.rarity = 6) AS "six_star_count!",
               COUNT(*) FILTER (WHERE gr.rarity = 5) AS "five_star_count!",
               COUNT(DISTINCT gr.user_id) AS "user_count!"
           FROM gacha_records gr
           JOIN user_settings us ON us.user_id = gr.user_id
           WHERE us.share_stats = true
           GROUP BY gr.pool_id
           ORDER BY COUNT(*) DESC, gr.pool_id"#
    )
    .fetch_all(conn)
    .await?)
}

pub async fn distinct_char_rarities(conn: &mut Conn) -> Result<Vec<CharRarityRow>, DbError> {
    Ok(sqlx::query_as!(
        CharRarityRow,
        "SELECT DISTINCT char_id, rarity FROM gacha_records"
    )
    .fetch_all(conn)
    .await?)
}

pub async fn correct_rarities(
    conn: &mut Conn,
    char_ids: &[String],
    rarities: &[i16],
) -> Result<u64, DbError> {
    let result = sqlx::query!(
        "UPDATE gacha_records gr
         SET rarity = u.rarity
         FROM UNNEST($1::text[], $2::smallint[]) AS u(char_id, rarity)
         WHERE gr.char_id = u.char_id AND gr.rarity <> u.rarity",
        char_ids,
        rarities,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected())
}
