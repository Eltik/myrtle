use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::gacha::{GachaRecord, GachaStats};
use crate::database::models::user::UserSettings;

/// What [`insert_batch`] did with a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum BatchOutcome {
    /// The rows were written.
    Stored,
    /// The account's `user_settings.store_gacha` is off: nothing was written.
    StorageOff,
}

/// Whether `user_settings.store_gacha` lets pulls be stored for `user_id`.
///
/// A missing settings row reads as ON, explicitly: the column defaults to
/// `true` and `get_or_create_settings` creates the row with that default, so
/// an account that never opened Settings stores pulls exactly as before.
pub async fn store_gacha_enabled<'e, E>(executor: E, user_id: Uuid) -> Result<bool, sqlx::Error>
where
    E: sqlx::PgExecutor<'e>,
{
    let stored: Option<bool> =
        sqlx::query_scalar("SELECT store_gacha FROM user_settings WHERE user_id = $1 FOR SHARE")
            .bind(user_id)
            .fetch_optional(executor)
            .await?;
    // None is the missing row, not a false switch.
    Ok(stored.unwrap_or(true))
}

/// The one write path into `gacha_records`, so the opt-out is enforced HERE
/// rather than by each caller: before this, `store_gacha` ("Store gacha
/// history", Settings > Privacy) was never read on the way in, and an
/// opted-out account had every synced pull stored.
///
/// The switch is read in the same transaction as the write, under `FOR SHARE`,
/// so a toggle that lands while a fetch is paging through Yostar either commits
/// first (and this stores nothing) or waits for this batch to commit.
pub async fn insert_batch(
    pool: &PgPool,
    user_id: Uuid,
    records: &serde_json::Value,
) -> Result<BatchOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    if !store_gacha_enabled(&mut *tx, user_id).await? {
        return Ok(BatchOutcome::StorageOff);
    }
    sqlx::query("CALL sp_insert_gacha_batch($1, $2)")
        .bind(user_id)
        .bind(records)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(BatchOutcome::Stored)
}

/// All history queries order ties on `batch_index ASC, id ASC`: a multi-pull
/// shares one `pull_timestamp`, and the in-game Headhunting History lists the
/// batch `batch_index` 0->9 top-to-bottom in its newest-first view. Without the
/// tiebreaker Postgres returns tied rows in undefined order, which mis-numbers
/// pulls and corrupts pity distances downstream.
pub async fn get_history(
    pool: &PgPool,
    user_id: Uuid,
    rarity: Option<i16>,
    limit: i64,
    offset: i64,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    if let Some(r) = rarity {
        sqlx::query_as::<_, GachaRecord>(
            "SELECT * FROM gacha_records WHERE user_id = $1 AND rarity = $2 ORDER BY pull_timestamp DESC, batch_index ASC, id ASC LIMIT $3 OFFSET $4"
        )
        .bind(user_id)
        .bind(r)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, GachaRecord>(
            "SELECT * FROM gacha_records WHERE user_id = $1 ORDER BY pull_timestamp DESC, batch_index ASC, id ASC LIMIT $2 OFFSET $3"
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

pub async fn get_stats(pool: &PgPool, user_id: Uuid) -> Result<Option<GachaStats>, sqlx::Error> {
    sqlx::query_as::<_, GachaStats>("SELECT * FROM v_gacha_stats WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

/// Filtered history for `/gacha/history`.
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
         ORDER BY pull_timestamp {order_sql}, batch_index ASC, id ASC \
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
    get_for_user_in_pools(pool, user_id, None).await
}

/// A user's records from the banners in `pools`, or from every banner when
/// `None`, in [`get_all_for_user`]'s order.
pub async fn get_for_user_in_pools(
    pool: &PgPool,
    user_id: Uuid,
    pools: Option<&[String]>,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    sqlx::query_as::<_, GachaRecord>(
        "SELECT * FROM gacha_records \
         WHERE user_id = $1 AND ($2::TEXT[] IS NULL OR pool_id = ANY($2)) \
         ORDER BY pull_timestamp DESC, batch_index ASC, id ASC",
    )
    .bind(user_id)
    .bind(pools)
    .fetch_all(pool)
    .await
}

/// Whether `user_settings.public_gacha` lets visitors read `user_id`'s pull
/// history. A missing settings row reads as OFF, the column's default: the
/// switch is opt-in, so an account that never opened Settings shares nothing.
pub async fn public_history_enabled(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let public: Option<bool> =
        sqlx::query_scalar("SELECT public_gacha FROM user_settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(public.unwrap_or(false))
}

pub async fn get_by_char_for_user(
    pool: &PgPool,
    user_id: Uuid,
    char_id: &str,
) -> Result<Vec<GachaRecord>, sqlx::Error> {
    sqlx::query_as::<_, GachaRecord>(
        "SELECT * FROM gacha_records WHERE user_id = $1 AND char_id = $2 ORDER BY pull_timestamp DESC, batch_index ASC, id ASC",
    )
    .bind(user_id)
    .bind(char_id)
    .fetch_all(pool)
    .await
}

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

pub async fn update_gacha_flags(
    pool: &PgPool,
    user_id: Uuid,
    store_gacha: Option<bool>,
    share_stats: Option<bool>,
) -> Result<UserSettings, sqlx::Error> {
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
