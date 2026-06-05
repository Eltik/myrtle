use sqlx::PgPool;
use uuid::Uuid;

use crate::database::models::roster::RosterEntry;

/// Get full roster for a user
pub async fn get_roster(pool: &PgPool, user_id: Uuid) -> Result<Vec<RosterEntry>, sqlx::Error> {
    sqlx::query_as::<_, RosterEntry>("SELECT * FROM v_user_roster WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(pool)
        .await
}

/// Get a single operator for a user
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

/// Sync full user data from game server
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
    checkin: &[i16],
    supports: &serde_json::Value,
    nick_number: Option<&str>,
    enemies: &serde_json::Value,
) -> Result<(), sqlx::Error> {
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
    .execute(pool)
    .await?;
    Ok(())
}

/// Get support units for a user (joined with operator state).
pub async fn get_supports(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<crate::database::models::roster::SupportUnit>, sqlx::Error> {
    sqlx::query_as::<_, crate::database::models::roster::SupportUnit>(
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
