use std::collections::{HashMap, HashSet};

use myrtle_types::UserId;
use myrtle_types::stage_clear::StageClear;
use serde_json::Value;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct MedalRow {
    pub medal_id: String,
    pub val: Option<Value>,
    pub first_ts: Option<i64>,
    pub reach_ts: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct RoguelikeRow {
    pub theme_id: String,
    pub progress: Value,
}

#[derive(Debug, Clone, Default)]
pub struct StageClears {
    pub clears: HashMap<String, StageClear>,
    pub last_synced_ts: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub struct EncounterAverage {
    pub users: i64,
    pub average: f64,
}

pub async fn building(conn: &mut Conn, user: UserId) -> Result<Option<Value>, DbError> {
    Ok(
        sqlx::query_scalar!("SELECT data FROM user_building WHERE user_id = $1", user.0)
            .fetch_optional(conn)
            .await?,
    )
}

pub async fn medals(conn: &mut Conn, user: UserId) -> Result<Vec<MedalRow>, DbError> {
    Ok(sqlx::query_as!(
        MedalRow,
        "SELECT medal_id, val, first_ts, reach_ts FROM user_medals WHERE user_id = $1",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn roguelike(conn: &mut Conn, user: UserId) -> Result<Vec<RoguelikeRow>, DbError> {
    Ok(sqlx::query_as!(
        RoguelikeRow,
        "SELECT theme_id, progress FROM user_roguelike_progress WHERE user_id = $1",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn sandbox(conn: &mut Conn, user: UserId) -> Result<Option<Value>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT progress FROM user_sandbox_progress WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn store_sandbox_topic(
    conn: &mut Conn,
    user: UserId,
    template: &str,
    topic: &str,
    blob: &Value,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE user_sandbox_progress SET progress = jsonb_set(
             jsonb_set(progress, ARRAY['template', $2],
                 CASE WHEN jsonb_typeof(progress #> ARRAY['template', $2]) = 'object'
                      THEN progress #> ARRAY['template', $2] ELSE '{}'::jsonb END, true),
             ARRAY['template', $2, $3], $4, true)
         WHERE user_id = $1 AND jsonb_typeof(progress -> 'template') = 'object'",
        user.0,
        template,
        topic,
        blob,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn known_stage_ids_for_server(
    conn: &mut Conn,
    user: UserId,
) -> Result<HashSet<String>, DbError> {
    let ids = sqlx::query_scalar!(
        r#"SELECT DISTINCT key AS "key!"
         FROM users u
         JOIN user_stage_progress usp ON usp.user_id = u.id
         CROSS JOIN LATERAL jsonb_object_keys(usp.stages) AS key
         WHERE u.server_id = (SELECT server_id FROM users WHERE id = $1)"#,
        user.0,
    )
    .fetch_all(conn)
    .await?;
    Ok(ids.into_iter().collect())
}

pub async fn stage_clears(conn: &mut Conn, user: UserId) -> Result<StageClears, DbError> {
    let row = sqlx::query!(
        "SELECT usp.stages, u.updated_at
         FROM user_stage_progress usp
         JOIN users u ON u.id = usp.user_id
         WHERE usp.user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?;
    let Some(row) = row else {
        return Ok(StageClears::default());
    };
    let last_synced_ts = Some(row.updated_at.timestamp());
    let Some(stages) = row.stages.as_object() else {
        return Ok(StageClears {
            clears: HashMap::new(),
            last_synced_ts,
        });
    };
    let int = |entry: &Value, key: &str| entry.get(key).and_then(Value::as_i64);
    let small = |v: i64| i16::try_from(v).unwrap_or(if v < 0 { i16::MIN } else { i16::MAX });
    let count = |v: i64| i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX });
    let clears = stages
        .iter()
        .map(|(stage_id, entry)| {
            let state = int(entry, "state").map_or(0, small);
            let clear = StageClear {
                state,
                state_max: int(entry, "stateMax").map_or(state, small),
                inferred: entry
                    .get("inferred")
                    .is_some_and(|v| v.as_bool() == Some(true) || v.is_string()), // older syncs tag it with a string
                complete_times: int(entry, "completeTimes").map_or(0, count),
                practice_times: int(entry, "practiceTimes").map_or(0, count),
            };
            (stage_id.clone(), clear)
        })
        .collect();
    Ok(StageClears {
        clears,
        last_synced_ts,
    })
}

pub async fn encountered_enemies(conn: &mut Conn, user: UserId) -> Result<Vec<String>, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT key AS "key!"
         FROM user_enemy_progress uep
         CROSS JOIN LATERAL jsonb_object_keys(uep.enemies) AS key
         WHERE uep.user_id = $1"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn community_average_encountered(conn: &mut Conn) -> Result<EncounterAverage, DbError> {
    Ok(sqlx::query_as!(
        EncounterAverage,
        r#"SELECT COUNT(*)::bigint AS "users!", COALESCE(AVG(seen_count), 0)::float8 AS "average!"
         FROM (
             SELECT (SELECT COUNT(*) FROM jsonb_object_keys(uep.enemies)) AS seen_count
             FROM user_enemy_progress uep
         ) AS per_user"#,
    )
    .fetch_one(conn)
    .await?)
}
