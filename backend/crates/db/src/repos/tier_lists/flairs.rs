use chrono::{DateTime, Utc};

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct FlairRow {
    pub id: i16,
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list_flairs(conn: &mut Conn, only_active: bool) -> Result<Vec<FlairRow>, DbError> {
    Ok(sqlx::query_as!(
        FlairRow,
        "SELECT id, code, label, color, display_order, is_active, created_at, updated_at
         FROM tier_list_flairs WHERE is_active OR NOT $1
         ORDER BY display_order, label",
        only_active,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn get_flair_by_id(conn: &mut Conn, id: i16) -> Result<Option<FlairRow>, DbError> {
    Ok(sqlx::query_as!(
        FlairRow,
        "SELECT id, code, label, color, display_order, is_active, created_at, updated_at
         FROM tier_list_flairs WHERE id = $1",
        id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn get_flairs_by_ids(conn: &mut Conn, ids: &[i16]) -> Result<Vec<FlairRow>, DbError> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as!(
        FlairRow,
        "SELECT id, code, label, color, display_order, is_active, created_at, updated_at
         FROM tier_list_flairs WHERE id = ANY($1)",
        ids,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn create_flair(
    conn: &mut Conn,
    code: &str,
    label: &str,
    color: Option<&str>,
    display_order: i16,
) -> Result<FlairRow, DbError> {
    Ok(sqlx::query_as!(
        FlairRow,
        "INSERT INTO tier_list_flairs (code, label, color, display_order) VALUES ($1, $2, $3, $4)
         RETURNING id, code, label, color, display_order, is_active, created_at, updated_at",
        code,
        label,
        color,
        display_order,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn update_flair(
    conn: &mut Conn,
    id: i16,
    label: &str,
    color: Option<&str>,
    display_order: i16,
    is_active: bool,
) -> Result<FlairRow, DbError> {
    Ok(sqlx::query_as!(
        FlairRow,
        "UPDATE tier_list_flairs
         SET label = $2, color = $3, display_order = $4, is_active = $5, updated_at = NOW()
         WHERE id = $1
         RETURNING id, code, label, color, display_order, is_active, created_at, updated_at",
        id,
        label,
        color,
        display_order,
        is_active,
    )
    .fetch_one(conn)
    .await?)
}
