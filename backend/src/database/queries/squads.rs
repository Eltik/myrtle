//! Saved squads and synced in-game squads. Every function takes a connection
//! rather than the pool so a caller (or a test) can run it inside its own
//! transaction.

use sqlx::PgConnection;
use sqlx::types::Json;
use uuid::Uuid;

use crate::database::models::squad::{GameSquadRow, SavedSquadRow, SquadMember};

const SAVED_COLUMNS: &str = "slot, name, members, support, stage_id, created_at, updated_at";

pub async fn list_saved(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<SavedSquadRow>, sqlx::Error> {
    sqlx::query_as::<_, SavedSquadRow>(&format!(
        "SELECT {SAVED_COLUMNS} FROM user_squads WHERE user_id = $1 ORDER BY slot"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await
}

/// Write the squad into `slot`, replacing whatever was there. `created_at`
/// survives a replace; the timestamp trigger moves `updated_at`.
pub async fn upsert_saved(
    conn: &mut PgConnection,
    user_id: Uuid,
    slot: i16,
    name: &str,
    members: &[Option<SquadMember>],
    support: Option<&SquadMember>,
    stage_id: Option<&str>,
) -> Result<SavedSquadRow, sqlx::Error> {
    sqlx::query_as::<_, SavedSquadRow>(&format!(
        "INSERT INTO user_squads (user_id, slot, name, members, support, stage_id)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, slot) DO UPDATE
         SET name = EXCLUDED.name, members = EXCLUDED.members,
             support = EXCLUDED.support, stage_id = EXCLUDED.stage_id
         RETURNING {SAVED_COLUMNS}"
    ))
    .bind(user_id)
    .bind(slot)
    .bind(name)
    .bind(Json(members))
    .bind(support.map(Json))
    .bind(stage_id)
    .fetch_one(conn)
    .await
}

/// `true` when a squad was in `slot`.
pub async fn delete_saved(
    conn: &mut PgConnection,
    user_id: Uuid,
    slot: i16,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_squads WHERE user_id = $1 AND slot = $2")
        .bind(user_id)
        .bind(slot)
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_game_squads(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<GameSquadRow>, sqlx::Error> {
    sqlx::query_as::<_, GameSquadRow>(
        // `users.updated_at` moves on every sync (`trg_users_timestamp`); a
        // squad row is written only when it changes, so its own `synced_at`
        // is when it last changed, not when it was last seen.
        "SELECT g.squad_index, g.name, g.slots, u.updated_at AS synced_at
         FROM user_game_squads g JOIN users u ON u.id = g.user_id
         WHERE g.user_id = $1 ORDER BY g.squad_index",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
}
