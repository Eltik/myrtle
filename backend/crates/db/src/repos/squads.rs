use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct SavedSquadRow {
    pub slot: i16,
    pub name: String,
    pub members: Value,
    pub support: Option<Value>,
    pub stage_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GameSquadRow {
    pub squad_index: i16,
    pub name: String,
    pub slots: Value,
    pub synced_at: DateTime<Utc>,
}

pub async fn list_saved(conn: &mut Conn, user: UserId) -> Result<Vec<SavedSquadRow>, DbError> {
    Ok(sqlx::query_as!(
        SavedSquadRow,
        "SELECT slot, name, members, support, stage_id, created_at, updated_at
         FROM user_squads WHERE user_id = $1 ORDER BY slot",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn upsert_saved(
    conn: &mut Conn,
    user: UserId,
    slot: i16,
    name: &str,
    members: &Value,
    support: Option<&Value>,
    stage_id: Option<&str>,
) -> Result<SavedSquadRow, DbError> {
    Ok(sqlx::query_as!(
        SavedSquadRow,
        "INSERT INTO user_squads (user_id, slot, name, members, support, stage_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, slot) DO UPDATE
         SET name = EXCLUDED.name, members = EXCLUDED.members,
             support = EXCLUDED.support, stage_id = EXCLUDED.stage_id
         RETURNING slot, name, members, support, stage_id, created_at, updated_at",
        user.0,
        slot,
        name,
        members,
        support,
        stage_id,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn delete_saved(conn: &mut Conn, user: UserId, slot: i16) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM user_squads WHERE user_id = $1 AND slot = $2",
        user.0,
        slot,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_game_squads(conn: &mut Conn, user: UserId) -> Result<Vec<GameSquadRow>, DbError> {
    Ok(sqlx::query_as!(
        GameSquadRow,
        "SELECT g.squad_index, g.name, g.slots, u.updated_at AS synced_at
         FROM user_game_squads g JOIN users u ON u.id = g.user_id
         WHERE g.user_id = $1 ORDER BY g.squad_index",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}
