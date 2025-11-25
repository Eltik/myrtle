use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub uid: String,
    pub server: String,
    pub data: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub async fn find_by_uid(pool: &sqlx::PgPool, uid: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(Self, "SELECT * FROM users WHERE uid = $1", uid)
            .fetch_optional(pool)
            .await
    }
    // ... other methods
}
