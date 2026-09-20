use sqlx::PgPool;
use ts_rs::TS;
use uuid::Uuid;

use crate::database::queries::item_leaderboard::currency_values_sql;

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct ItemEntry {
    pub item_id: String,
    pub quantity: i32,
}

/// Everything a user holds, by game item id: the `user_items` rows plus the
/// currencies that live on `user_status` (LMD, Originite Prime, Orundum,
/// certificates, permits), so a reader never has to know which table an item
/// is in. Zero balances are left out, like an item never held.
pub async fn get_inventory(pool: &PgPool, user_id: Uuid) -> Result<Vec<ItemEntry>, sqlx::Error> {
    let sql = format!(
        r"
        SELECT item_id, quantity FROM user_items WHERE user_id = $1
        UNION ALL
        SELECT c.item_id, c.q::int AS quantity
        FROM user_status st
        CROSS JOIN LATERAL (VALUES {}) AS c(item_id, q)
        WHERE st.user_id = $1 AND c.q > 0
        ORDER BY item_id
        ",
        currency_values_sql()
    );
    sqlx::query_as::<_, ItemEntry>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await
}
