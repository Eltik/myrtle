use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::time::Instant;

use crate::app::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    status: &'static str,
    redis: RedisHealth,
    database: DatabaseHealth,
    timestamp: String,
    response_time_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RedisHealth {
    status: &'static str,
    state: &'static str,
    response_time_ms: u128,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseHealth {
    status: &'static str,
    response_time_ms: u128,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let start = Instant::now();

    // Ping Redis
    let redis_start = Instant::now();
    let mut redis_conn = state.redis.clone();
    let redis_ok = redis::cmd("PING")
        .query_async::<String>(&mut redis_conn)
        .await
        .is_ok();
    let redis_ms = redis_start.elapsed().as_millis();

    // Ping Database
    let db_start = Instant::now();
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let db_ms = db_start.elapsed().as_millis();

    let all_ok = redis_ok && db_ok;

    Json(HealthResponse {
        status: if all_ok { "ok" } else { "degraded" },
        redis: RedisHealth {
            status: if redis_ok {
                "connected"
            } else {
                "disconnected"
            },
            state: if redis_ok { "ready" } else { "error" },
            response_time_ms: redis_ms,
        },
        database: DatabaseHealth {
            status: if db_ok { "connected" } else { "disconnected" },
            response_time_ms: db_ms,
        },
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        response_time_ms: start.elapsed().as_millis(),
    })
}
