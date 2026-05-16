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
