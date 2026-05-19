use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct OwnedSkin {
    pub skin_id: String,
    pub obtained_at: Option<i64>,
}

/// Return every non-default skin the user owns. Skin IDs containing `@` are
/// non-default (the default-skin IDs use `#`).
pub async fn get_owned_skins(pool: &PgPool, user_id: Uuid) -> Result<Vec<OwnedSkin>, sqlx::Error> {
    sqlx::query_as::<_, OwnedSkin>(
        "SELECT skin_id, obtained_at \
         FROM user_skins \
         WHERE user_id = $1 AND skin_id LIKE '%@%' \
         ORDER BY skin_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SkinPopularityRow {
    pub skin_id: String,
    pub owners: i64,
}

/// Aggregate ownership counts for every non-default skin and the population of
/// users that have at least one skin record (i.e. have imported their data).
/// Used to compute "what % of users own this skin".
pub async fn get_skin_popularity(
    pool: &PgPool,
) -> Result<(i64, Vec<SkinPopularityRow>), sqlx::Error> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT user_id) FROM user_skins")
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query_as::<_, SkinPopularityRow>(
        "SELECT skin_id, COUNT(*)::BIGINT AS owners \
         FROM user_skins \
         WHERE skin_id LIKE '%@%' \
         GROUP BY skin_id",
    )
    .fetch_all(pool)
    .await?;

    Ok((total_users, rows))
}
