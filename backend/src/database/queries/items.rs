use sqlx::PgPool;
use uuid::Uuid;

/// Simple key-value pair for inventory
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ItemEntry {
    pub item_id: String,
    pub quantity: i32,
}

pub async fn get_inventory(pool: &PgPool, user_id: Uuid) -> Result<Vec<ItemEntry>, sqlx::Error> {
    sqlx::query_as::<_, ItemEntry>(
        "SELECT item_id, quantity FROM user_items WHERE user_id = $1 ORDER BY item_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
