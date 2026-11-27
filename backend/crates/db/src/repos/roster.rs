use myrtle_types::roster::{RosterEntry, SupportUnit};
use myrtle_types::{Server, UserId};
use serde_json::Value;

use crate::{Conn, DbError};

#[derive(Debug, Clone, Copy)]
pub struct SyncUserData<'a> {
    pub uid: &'a str,
    pub server: Server,
    pub nickname: &'a str,
    pub level: i16,
    pub avatar_id: Option<&'a str>,
    pub secretary: Option<&'a str>,
    pub secretary_skin_id: Option<&'a str>,
    pub resume_id: Option<&'a str>,
    pub operators: &'a Value,
    pub skills: &'a Value,
    pub modules: &'a Value,
    pub items: &'a Value,
    pub skins: &'a Value,
    pub status: &'a Value,
    pub stages: &'a Value,
    pub roguelike: &'a Value,
    pub sandbox: &'a Value,
    pub medals: &'a Value,
    pub building: &'a Value,
    pub checkin: &'a Value,
    pub supports: &'a Value,
    pub nick_number: Option<&'a str>,
    pub enemies: &'a Value,
    pub game_squads: &'a Value,
}

#[derive(Debug, Clone)]
pub struct OwnedSkinRow {
    pub skin_id: String,
    pub obtained_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SkinPopularityRow {
    pub skin_id: String,
    pub owners: i64,
}

pub async fn roster(conn: &mut Conn, user: UserId) -> Result<Vec<RosterEntry>, DbError> {
    Ok(sqlx::query_as!(
        RosterEntry,
        r#"SELECT user_id AS "user_id!", operator_id AS "operator_id!", elite AS "elite!",
             level AS "level!", exp AS "exp!", potential AS "potential!",
             skill_level AS "skill_level!", favor_point AS "favor_point!", skin_id,
             default_skill, voice_lan, current_equip, current_tmpl, obtained_at,
             masteries AS "masteries!", modules AS "modules!"
         FROM v_user_roster WHERE user_id = $1"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn operator(
    conn: &mut Conn,
    user: UserId,
    operator_id: &str,
) -> Result<Option<RosterEntry>, DbError> {
    Ok(sqlx::query_as!(
        RosterEntry,
        r#"SELECT user_id AS "user_id!", operator_id AS "operator_id!", elite AS "elite!",
             level AS "level!", exp AS "exp!", potential AS "potential!",
             skill_level AS "skill_level!", favor_point AS "favor_point!", skin_id,
             default_skill, voice_lan, current_equip, current_tmpl, obtained_at,
             masteries AS "masteries!", modules AS "modules!"
         FROM v_user_roster WHERE user_id = $1 AND operator_id = $2"#,
        user.0,
        operator_id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn sync_user_data(conn: &mut Conn, data: &SyncUserData<'_>) -> Result<(), DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    sqlx::query!(
        "CALL sp_sync_user_data($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
                                $16, $17, $18, $19, $20, $21, $22, $23)",
        data.uid,
        data.server.index(),
        data.nickname,
        data.level,
        data.avatar_id,
        data.secretary,
        data.secretary_skin_id,
        data.resume_id,
        data.operators,
        data.skills,
        data.modules,
        data.items,
        data.skins,
        data.status,
        data.stages,
        data.roguelike,
        data.sandbox,
        data.medals,
        data.building,
        data.checkin,
        data.supports,
        data.nick_number,
        data.enemies,
    )
    .execute(&mut *tx)
    .await?;
    let originite = data
        .status
        .get("originite")
        .and_then(Value::as_i64)
        .map(|v| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX }));
    sqlx::query!(
        "UPDATE user_status SET originite = $3
         WHERE user_id = (SELECT id FROM users WHERE uid = $1 AND server_id = $2)",
        data.uid,
        data.server.index(),
        originite,
    )
    .execute(&mut *tx)
    .await?;
    sync_game_squads(&mut tx, data.uid, data.server, data.game_squads).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn sync_game_squads(
    conn: &mut Conn,
    uid: &str,
    server: Server,
    game_squads: &Value,
) -> Result<u64, DbError> {
    let empty = Value::Array(Vec::new());
    let squads = if game_squads.is_array() {
        game_squads
    } else {
        &empty
    };
    let written = sqlx::query_scalar!(
        r#"WITH account AS (SELECT id FROM users WHERE uid = $1 AND server_id = $2),
           incoming AS (
               SELECT (e->>'index')::smallint AS squad_index, COALESCE(e->>'name', '') AS name,
                      COALESCE(e->'slots', '[]'::jsonb) AS slots
               FROM jsonb_array_elements($3) AS e
               WHERE e ? 'index'
           ),
           upserted AS (
               INSERT INTO user_game_squads (user_id, squad_index, name, slots, synced_at)
               SELECT a.id, i.squad_index, i.name, i.slots, now() FROM account a, incoming i
               ON CONFLICT (user_id, squad_index) DO UPDATE
                  SET name = EXCLUDED.name, slots = EXCLUDED.slots, synced_at = EXCLUDED.synced_at
                  WHERE (user_game_squads.name, user_game_squads.slots)
                        IS DISTINCT FROM (EXCLUDED.name, EXCLUDED.slots)
               RETURNING 1
           ),
           deleted AS (
               DELETE FROM user_game_squads g USING account a
               WHERE g.user_id = a.id AND g.squad_index NOT IN (SELECT squad_index FROM incoming)
               RETURNING 1
           )
           SELECT (SELECT count(*) FROM upserted) + (SELECT count(*) FROM deleted) AS "written!""#,
        uid,
        server.index(),
        squads,
    )
    .fetch_one(conn)
    .await?;
    Ok(u64::try_from(written).unwrap_or_default())
}

pub async fn supports(conn: &mut Conn, user: UserId) -> Result<Vec<SupportUnit>, DbError> {
    Ok(sqlx::query_as!(
        SupportUnit,
        r#"SELECT su.slot, su.operator_id, COALESCE(su.skin_id, uo.skin_id) AS skin_id,
             su.skill_index, COALESCE(su.current_equip, uo.current_equip) AS current_equip,
             uo.elite AS "elite?", uo.level AS "level?", uo.potential AS "potential?",
             uo.skill_level AS "skill_level?", uo.favor_point AS "favor_point?",
             COALESCE(s.specialize_level, 0::smallint) AS "specialize_level!"
         FROM user_support_units su
         LEFT JOIN user_operators uo
           ON uo.user_id = su.user_id AND uo.operator_id = su.operator_id
         LEFT JOIN user_operator_skills s
           ON s.user_id = su.user_id
          AND s.operator_id = su.operator_id
          AND s.skill_index = su.skill_index
         WHERE su.user_id = $1
         ORDER BY su.slot"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn owned_skins(conn: &mut Conn, user: UserId) -> Result<Vec<OwnedSkinRow>, DbError> {
    Ok(sqlx::query_as!(
        OwnedSkinRow,
        "SELECT skin_id, obtained_at FROM user_skins
         WHERE user_id = $1 AND skin_id LIKE '%@%'
         ORDER BY skin_id",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn skin_popularity(conn: &mut Conn) -> Result<(i64, Vec<SkinPopularityRow>), DbError> {
    let total =
        sqlx::query_scalar!(r#"SELECT COUNT(DISTINCT user_id) AS "count!" FROM user_skins"#)
            .fetch_one(&mut *conn)
            .await?;
    let rows = sqlx::query_as!(
        SkinPopularityRow,
        r#"SELECT skin_id, COUNT(*)::bigint AS "owners!"
         FROM user_skins
         WHERE skin_id LIKE '%@%'
         GROUP BY skin_id"#,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok((total, rows))
}

pub async fn users_with_roster(conn: &mut Conn) -> Result<i64, DbError> {
    Ok(
        sqlx::query_scalar!(r#"SELECT COUNT(DISTINCT user_id) AS "count!" FROM user_operators"#)
            .fetch_one(conn)
            .await?,
    )
}
