use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use uuid::Uuid;

use crate::core::grade::stages::types::StageClear;

pub struct UserStageData {
    pub clears: HashMap<String, StageClear>,
    /// Unix-seconds timestamp of the row's last update, used by the event-pool
    /// scorer to skip activities that started after the user's last sync.
    pub last_synced_ts: Option<i64>,
}

/// Returns the set of stage IDs that at least one user on the same server as
/// `user_id` has appearing in their dungeon JSON.
///
/// The gamedata bundled with the backend can be ahead of the user's actual
/// server (e.g. EN players grading against CN-era `stage_table`), which leaves
/// the universe full of stages no one on that server can possibly clear.
/// Filtering the universe by this set caps each server's universe at content
/// that's actually live there.
pub async fn get_known_stage_ids_for_server(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT DISTINCT key
        FROM users u
        JOIN user_stage_progress usp ON usp.user_id = u.id
        CROSS JOIN LATERAL jsonb_object_keys(usp.stages) AS key
        WHERE u.server_id = (SELECT server_id FROM users WHERE id = $1)
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect())
}

pub async fn get_user_stage_clears(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserStageData, sqlx::Error> {
    let row: Option<(serde_json::Value, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        r"
        SELECT usp.stages, u.updated_at
        FROM user_stage_progress usp
        JOIN users u ON u.id = usp.user_id
        WHERE usp.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some((json, updated_at)) = row else {
        return Ok(UserStageData {
            clears: HashMap::new(),
            last_synced_ts: None,
        });
    };

    let last_synced_ts = updated_at.map(|ts| ts.timestamp());

    let Some(obj) = json.as_object() else {
        return Ok(UserStageData {
            clears: HashMap::new(),
            last_synced_ts,
        });
    };

    let mut clears = HashMap::with_capacity(obj.len());
    for (stage_id, entry) in obj {
        let state = entry
            .get("state")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i16;
        let complete_times = entry
            .get("completeTimes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32;
        let practice_times = entry
            .get("practiceTimes")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0) as i32;

        clears.insert(
            stage_id.clone(),
            StageClear {
                state,
                complete_times,
                practice_times,
            },
        );
    }

    Ok(UserStageData {
        clears,
        last_synced_ts,
    })
}
