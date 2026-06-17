use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OperatorOwnershipRow {
    pub operator_id: String,
    pub owners: i64,
}

/// Per-server ownership counts plus the server's eligible population (the
/// denominator), read from the precomputed aggregate. Operators with no owners
/// are absent; callers treat a missing id as zero.
pub async fn get_operator_ownership(
    pool: &PgPool,
    server_id: i16,
) -> Result<(i64, Vec<OperatorOwnershipRow>), sqlx::Error> {
    // population is identical across every row for a server; pull it once.
    let population: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(population), 0)::BIGINT \
         FROM operator_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, OperatorOwnershipRow>(
        "SELECT operator_id, owners::BIGINT AS owners \
         FROM operator_ownership_stats WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok((population, rows))
}

/// Timestamp of the most recent aggregate refresh, used to pace the background
/// job. `None` means it has never been computed.
pub async fn latest_ownership_refresh_at(
    pool: &PgPool,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar("SELECT MAX(computed_at) FROM operator_ownership_stats")
        .fetch_one(pool)
        .await
}

/// Recompute the per-server aggregate from scratch and atomically replace the
/// table. Heavy enough to run only from the background job, never on a request.
///
/// Only users who opted into stat sharing are counted, for both the owner
/// tallies and the population denominator. The denominator is the eligible
/// users on each server who have imported a roster, so percentages reflect the
/// sharing population rather than every registered account.
pub async fn refresh_operator_ownership(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM operator_ownership_stats")
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        r"
        INSERT INTO operator_ownership_stats (server_id, operator_id, owners, population)
        SELECT
            u.server_id,
            uo.operator_id,
            COUNT(*)::INT AS owners,
            pop.population
        FROM user_operators uo
        JOIN users u         ON u.id = uo.user_id
        JOIN user_settings s ON s.user_id = u.id
        JOIN (
            SELECT u2.server_id, COUNT(DISTINCT uo2.user_id)::INT AS population
            FROM user_operators uo2
            JOIN users u2         ON u2.id = uo2.user_id
            JOIN user_settings s2 ON s2.user_id = u2.id
            WHERE s2.share_stats = true
            GROUP BY u2.server_id
        ) pop ON pop.server_id = u.server_id
        WHERE s.share_stats = true
        GROUP BY u.server_id, uo.operator_id, pop.population
        ",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
