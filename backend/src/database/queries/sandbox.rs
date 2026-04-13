use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_user_sandbox_progress(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<serde_json::Value>, sqlx::Error> {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT progress FROM user_sandbox_progress WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(v,)| v))
}
