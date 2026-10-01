use sqlx::PgPool;
use uuid::Uuid;

/// One `(theme_id, progress JSONB)` row per roguelike theme for a user.
pub async fn get_roguelike_progress(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<(String, serde_json::Value)>, sqlx::Error> {
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT theme_id, progress FROM user_roguelike_progress WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

    Ok(rows)
}
