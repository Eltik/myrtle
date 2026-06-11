use sqlx::PgPool;
use uuid::Uuid;

/// Return the set of enemy IDs the user has encountered (the keys of the
/// stored `dexNav.enemy.enemies` map). Empty if the user has never synced or
/// the blob is absent.
pub async fn get_user_encountered_enemies(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT key
        FROM user_enemy_progress uep
        CROSS JOIN LATERAL jsonb_object_keys(uep.enemies) AS key
        WHERE uep.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

/// Community-wide average of distinct enemies encountered per user.
///
/// Averages the number of keys in each user's stored `enemies` map across every
/// user that has an enemy-progress row (i.e. has synced at least once). Returns
/// `(user_count, average)`; `average` is `0.0` when no users have synced yet.
pub async fn get_community_average_encountered(pool: &PgPool) -> Result<(i64, f64), sqlx::Error> {
    let row: (i64, f64) = sqlx::query_as(
        r"
        SELECT
            COUNT(*)::BIGINT AS user_count,
            COALESCE(AVG(seen_count), 0)::FLOAT8 AS average
        FROM (
            SELECT (
                SELECT COUNT(*) FROM jsonb_object_keys(uep.enemies)
            ) AS seen_count
            FROM user_enemy_progress uep
        ) AS per_user
        ",
    )
    .fetch_one(pool)
    .await?;
    Ok(row)
}
