use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::core::grade::stages::types::StageClear;

pub struct UserStageData {
    pub clears: HashMap<String, StageClear>,
    /// Unix-seconds timestamp of the row's last update, used by the event-pool
    /// scorer to skip activities that started after the user's last sync.
    pub last_synced_ts: Option<i64>,
}

pub async fn get_user_stage_clears(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<UserStageData, sqlx::Error> {
    let row: Option<(serde_json::Value, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        r#"
        SELECT usp.stages, u.updated_at
        FROM user_stage_progress usp
        JOIN users u ON u.id = usp.user_id
        WHERE usp.user_id = $1
        "#,
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
        let state = entry.get("state").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        let complete_times = entry
            .get("completeTimes")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let practice_times = entry
            .get("practiceTimes")
            .and_then(|v| v.as_i64())
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
