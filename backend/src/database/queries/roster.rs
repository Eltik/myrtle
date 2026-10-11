use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::roster::RosterEntry;
use crate::database::models::roster::SupportUnit;

pub async fn get_roster(pool: &PgPool, user_id: Uuid) -> Result<Vec<RosterEntry>, sqlx::Error> {
    sqlx::query_as::<_, RosterEntry>("SELECT * FROM v_user_roster WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

pub async fn get_operator(
    pool: &PgPool,
    user_id: Uuid,
    operator_id: &str,
) -> Result<Option<RosterEntry>, sqlx::Error> {
    sqlx::query_as::<_, RosterEntry>(
        "SELECT * FROM v_user_roster WHERE user_id = $1 AND operator_id = $2",
    )
    .bind(user_id)
    .bind(operator_id)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn sync_user_data(
    pool: &PgPool,
    uid: &str,
    server_id: i16,
    nickname: &str,
    level: i16,
    avatar_id: Option<&str>,
    secretary: Option<&str>,
    secretary_skin_id: Option<&str>,
    resume_id: Option<&str>,
    operators: &serde_json::Value,
    skills: &serde_json::Value,
    modules: &serde_json::Value,
    items: &serde_json::Value,
    skins: &serde_json::Value,
    status: &serde_json::Value,
    stages: &serde_json::Value,
    roguelike: &serde_json::Value,
    sandbox: &serde_json::Value,
    medals: &serde_json::Value,
    building: &serde_json::Value,
    checkin: &serde_json::Value,
    supports: &serde_json::Value,
    nick_number: Option<&str>,
    enemies: &serde_json::Value,
    game_squads: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    // One sync, one transaction: the procedure and the originite update land
    // together or not at all (an early `?` drops `tx`, which rolls back).
    let mut tx = pool.begin().await?;
    sqlx::query(
        "CALL sp_sync_user_data($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)"
    )
    .bind(uid)
    .bind(server_id)
    .bind(nickname)
    .bind(level)
    .bind(avatar_id)
    .bind(secretary)
    .bind(secretary_skin_id)
    .bind(resume_id)
    .bind(operators)
    .bind(skills)
    .bind(modules)
    .bind(items)
    .bind(skins)
    .bind(status)
    .bind(stages)
    .bind(roguelike)
    .bind(sandbox)
    .bind(medals)
    .bind(building)
    .bind(checkin)
    .bind(supports)
    .bind(nick_number)
    .bind(enemies)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE user_status SET originite = $3 WHERE user_id = (SELECT id FROM users WHERE uid = $1 AND server_id = $2)",
    )
    .bind(uid)
    .bind(server_id)
    .bind(
        status
            .get("originite")
            .and_then(serde_json::Value::as_i64)
            .map(|v| v as i32),
    )
    .execute(&mut *tx)
    .await?;
    sync_game_squads(&mut tx, uid, server_id, game_squads).await?;
    tx.commit().await?;
    Ok(())
}

/// Store the in-game squads one sync carries (`extract_game_squads`): one
/// upsert that writes a row only when its name or slots changed, and one
/// delete of the squads the sync no longer carries. An unchanged re-sync
/// writes 0 rows; the delete-and-reinsert this replaced wrote 2 per squad
/// (a delete and an insert, 8 for the usual 4). Returns the rows written.
pub async fn sync_game_squads(
    conn: &mut sqlx::PgConnection,
    uid: &str,
    server_id: i16,
    game_squads: &serde_json::Value,
) -> Result<u64, sqlx::Error> {
    let Some(user_id) = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM users WHERE uid = $1 AND server_id = $2",
    )
    .bind(uid)
    .bind(server_id)
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(0);
    };
    let squads = if game_squads.is_array() {
        game_squads.clone()
    } else {
        serde_json::json!([])
    };
    let upserted = sqlx::query(
        "INSERT INTO user_game_squads (user_id, squad_index, name, slots, synced_at)
         SELECT $1, (e->>'index')::smallint, COALESCE(e->>'name', ''), COALESCE(e->'slots', '[]'::jsonb), now()
         FROM jsonb_array_elements($2) AS e
         WHERE e ? 'index'
         ON CONFLICT (user_id, squad_index) DO UPDATE
            SET name = EXCLUDED.name, slots = EXCLUDED.slots, synced_at = EXCLUDED.synced_at
            WHERE (user_game_squads.name, user_game_squads.slots)
                  IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.slots)",
    )
    .bind(user_id)
    .bind(&squads)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    let deleted = sqlx::query(
        "DELETE FROM user_game_squads
         WHERE user_id = $1
           AND squad_index NOT IN (
               SELECT (e->>'index')::smallint FROM jsonb_array_elements($2) AS e WHERE e ? 'index'
           )",
    )
    .bind(user_id)
    .bind(&squads)
    .execute(&mut *conn)
    .await?
    .rows_affected();
    Ok(upserted + deleted)
}

/// Get support units for a user (joined with operator state).
pub async fn get_supports(pool: &PgPool, user_id: Uuid) -> Result<Vec<SupportUnit>, sqlx::Error> {
    sqlx::query_as::<_, SupportUnit>(
        r"
        SELECT
            su.slot,
            su.operator_id,
            COALESCE(su.skin_id, uo.skin_id) AS skin_id,
            su.skill_index,
            COALESCE(su.current_equip, uo.current_equip) AS current_equip,
            uo.elite,
            uo.level,
            uo.potential,
            uo.skill_level,
            uo.favor_point,
            COALESCE(s.specialize_level, 0::SMALLINT) AS specialize_level
        FROM user_support_units su
        LEFT JOIN user_operators uo
          ON uo.user_id = su.user_id AND uo.operator_id = su.operator_id
        LEFT JOIN user_operator_skills s
          ON s.user_id = su.user_id
         AND s.operator_id = su.operator_id
         AND s.skill_index = su.skill_index
        WHERE su.user_id = $1
        ORDER BY su.slot
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
