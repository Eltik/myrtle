use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::gacha::{GachaRecord, GachaStats};

/// Batch insert gacha records via stored procedure
pub async fn insert_batch(
    pool: &PgPool,
    user_id: Uuid,
    records: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("CALL sp_insert_gacha_batch($1, $2)")
        .bind(user_id)
        .bind(records)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get gacha history
pub async fn get_history(
    pool: &PgPool,
    user_id: Uuid,
    rarity: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    if let Some(r) = rarity {
        sqlx::query_as::<_, GachaRecord>(
            "SELECT * FROM gacha_records WHERE user_id = $1 AND rarity = $2 ORDER BY pull_timestamp DESC LIMIT $3 OFFSET $4"
        )
        .bind(user_id)
        .bind(r)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, GachaRecord>(
            "SELECT * FROM gacha_records WHERE user_id = $1 ORDER BY pull_timestamp DESC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Get gacha stats for a user
pub async fn get_stats(pool: &PgPool, user_id: Uuid) -> Result<Option<GachaStats>, sqlx::Error> {
    sqlx::query_as::<_, GachaStats>("SELECT * FROM v_gacha_stats WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}
