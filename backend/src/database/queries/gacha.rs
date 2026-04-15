use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::gacha::{GachaRecord, GachaStats};
use crate::database::models::user::UserSettings;

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

/// Get full history with filters for user-facing `/gacha/history` endpoint.
#[allow(clippy::too_many_arguments)]
pub async fn get_history_filtered(
    pool: &PgPool,
    user_id: Uuid,
    rarity: Option<i16>,
    gacha_type: Option<String>,
    char_id: Option<String>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    order_desc: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<GachaRecord>, i64), sqlx::Error> {
    let order_sql = if order_desc { "DESC" } else { "ASC" };

    // Build WHERE clause. Map gacha_type param:
    //   "limited" -> ('limited','linkage')
    //   "regular" -> ('normal','classic')
    //   "special" -> ('single','boot')
    let gacha_types: Option<Vec<&'static str>> = gacha_type.as_deref().map(|gt| match gt {
        "limited" => vec!["limited", "linkage"],
        "regular" => vec!["normal", "classic"],
        "special" => vec!["single", "boot"],
        _ => vec![],
    });

    let rows_query = format!(
        "SELECT * FROM gacha_records \
         WHERE user_id = $1 \
           AND ($2::SMALLINT IS NULL OR rarity = $2) \
           AND ($3::TEXT[] IS NULL OR gacha_type = ANY($3)) \
           AND ($4::TEXT IS NULL OR char_id = $4) \
           AND ($5::BIGINT IS NULL OR pull_timestamp >= $5) \
           AND ($6::BIGINT IS NULL OR pull_timestamp <= $6) \
         ORDER BY pull_timestamp {order_sql} \
         LIMIT $7 OFFSET $8"
    );

    let count_query = "SELECT COUNT(*) FROM gacha_records \
         WHERE user_id = $1 \
           AND ($2::SMALLINT IS NULL OR rarity = $2) \
           AND ($3::TEXT[] IS NULL OR gacha_type = ANY($3)) \
           AND ($4::TEXT IS NULL OR char_id = $4) \
           AND ($5::BIGINT IS NULL OR pull_timestamp >= $5) \
           AND ($6::BIGINT IS NULL OR pull_timestamp <= $6)";

    let records = sqlx::query_as::<_, GachaRecord>(&rows_query)
        .bind(user_id)
        .bind(rarity)
        .bind(gacha_types.as_deref())
        .bind(char_id.as_deref())
        .bind(from_ts)
        .bind(to_ts)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let total: i64 = sqlx::query_scalar(count_query)
        .bind(user_id)
        .bind(rarity)
        .bind(gacha_types.as_deref())
        .bind(char_id.as_deref())
        .bind(from_ts)
        .bind(to_ts)
        .fetch_one(pool)
        .await?;

    Ok((records, total))
}

/// All records for a user (used for stored-records grouping).
pub async fn get_all_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    sqlx::query_as::<_, GachaRecord>(
        "SELECT * FROM gacha_records WHERE user_id = $1 ORDER BY pull_timestamp DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// All pulls of a given char for the user
pub async fn get_by_char_for_user(
    pool: &PgPool,
    user_id: Uuid,
    char_id: &str,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    sqlx::query_as::<_, GachaRecord>(
        "SELECT * FROM gacha_records WHERE user_id = $1 AND char_id = $2 ORDER BY pull_timestamp DESC",
    )
    .bind(user_id)
    .bind(char_id)
    .fetch_all(pool)
    .await
}

/// Fetch settings row, creating defaults if missing.
pub async fn get_or_create_settings(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserSettings, sqlx::Error> {
    if let Some(s) =
        sqlx::query_as::<_, UserSettings>("SELECT * FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?
    {
        return Ok(s);
    }

    sqlx::query_as::<_, UserSettings>(
        "INSERT INTO user_settings (user_id) VALUES ($1) \
         ON CONFLICT (user_id) DO UPDATE SET user_id = EXCLUDED.user_id \
         RETURNING *",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Patch settings flags.
pub async fn update_gacha_flags(
    pool: &PgPool,
    user_id: Uuid,
    store_gacha: Option<bool>,
    share_stats: Option<bool>,
) -> Result<UserSettings, sqlx::Error> {
    // Ensure row exists
    let _ = get_or_create_settings(pool, user_id).await?;
    sqlx::query_as::<_, UserSettings>(
        "UPDATE user_settings SET \
            store_gacha = COALESCE($2, store_gacha), \
            share_stats = COALESCE($3, share_stats), \
            updated_at = NOW() \
         WHERE user_id = $1 \
         RETURNING *",
    )
    .bind(user_id)
    .bind(store_gacha)
    .bind(share_stats)
    .fetch_one(pool)
    .await
}
