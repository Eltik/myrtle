use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MedalOwnershipRow {
    pub medal_id: String,
    pub owners: i64,
}

/// Per-server earned counts plus the server's eligible population (the
/// denominator), read from the precomputed aggregate. Medals nobody has earned
/// are absent; callers treat a missing id as zero.
pub async fn get_medal_ownership(
    pool: &PgPool,
    server_id: i16,
) -> Result<(i64, Vec<MedalOwnershipRow>), sqlx::Error> {
    // population is identical across every row for a server; pull it once.
    let population: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(population), 0)::BIGINT \
         FROM medal_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, MedalOwnershipRow>(
        "SELECT medal_id, owners::BIGINT AS owners \
         FROM medal_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok((population, rows))
}

/// Timestamp of the most recent aggregate refresh, used to pace the background
/// job. `None` means it has never been computed.
pub async fn latest_medal_ownership_refresh_at(
    pool: &PgPool,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar("SELECT MAX(computed_at) FROM medal_ownership_stats")
        .fetch_one(pool)
        .await
}

/// Recompute the per-server aggregate from scratch and atomically replace the
/// table. Background job only, never on a request.
///
/// Only users who opted into stat sharing count, for both the tallies and the
/// denominator (same policy as `refresh_operator_ownership`). The denominator is
/// eligible users on each server who have synced medal data.
///
/// `user_medals` has rows for in-progress medals too, so the WHERE mirrors
/// `is_medal_earned` in `app/services/roster.rs` (the source of truth): earned is
/// `reach_ts > 0`, or `reach_ts = -1` with `first_ts > 0` and every
/// `[achieved, required]` pair in `val` met (empty counts as met).
pub async fn refresh_medal_ownership(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM medal_ownership_stats")
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r"
        INSERT INTO medal_ownership_stats (server_id, medal_id, owners, population)
        SELECT
            u.server_id,
            um.medal_id,
            COUNT(*)::INT AS owners,
            pop.population
        FROM user_medals um
        JOIN users u         ON u.id = um.user_id
        JOIN user_settings s ON s.user_id = u.id
        JOIN (
            SELECT u2.server_id, COUNT(DISTINCT um2.user_id)::INT AS population
            FROM user_medals um2
            JOIN users u2         ON u2.id = um2.user_id
            JOIN user_settings s2 ON s2.user_id = u2.id
            WHERE s2.share_stats = true
            GROUP BY u2.server_id
        ) pop ON pop.server_id = u.server_id
        WHERE s.share_stats = true
          AND (
              COALESCE(um.reach_ts, 0) > 0
              OR (
                  COALESCE(um.reach_ts, 0) = -1
                  AND COALESCE(um.first_ts, 0) > 0
                  AND jsonb_typeof(um.val) = 'array'
                  AND NOT EXISTS (
                      SELECT 1
                      FROM jsonb_array_elements(um.val) AS cond
                      WHERE NOT (
                          jsonb_typeof(cond) = 'array'
                          AND jsonb_array_length(cond) >= 2
                          AND jsonb_typeof(cond -> 0) = 'number'
                          AND jsonb_typeof(cond -> 1) = 'number'
                          AND (cond ->> 0)::numeric >= (cond ->> 1)::numeric
                      )
                  )
              )
          )
        GROUP BY u.server_id, um.medal_id, pop.population
        ",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
