use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::core::grade::stages::types::StageClear;

pub async fn get_user_stage_clears(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<HashMap<String, StageClear>, sqlx::Error> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        r#"
        SELECT stages
        FROM user_stage_progress
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some((json,)) = row else {
        return Ok(HashMap::new());
    };

    let Some(obj) = json.as_object() else {
        return Ok(HashMap::new());
    };

    let mut out = HashMap::with_capacity(obj.len());
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

        out.insert(
            stage_id.clone(),
            StageClear {
                state,
                complete_times,
                practice_times,
            },
        );
    }

    Ok(out)
}
