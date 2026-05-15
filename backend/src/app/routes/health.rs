use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::time::{Duration, Instant};

use crate::app::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    status: &'static str,
    cache: CacheHealth,
    database: DatabaseHealth,
    timestamp: String,
    response_time_ms: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheHealth {
    backend: &'static str,
    status: &'static str,
    response_time_ms: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseHealth {
    status: &'static str,
    response_time_ms: f64,
}

fn elapsed_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let start = Instant::now();

    // Ping cache
    let cache_start = Instant::now();
    let cache_ok = state.cache.ping().await;
    let cache_ms = elapsed_ms(cache_start.elapsed());

    let backend = if state.cache.is_redis() {
        "redis"
    } else {
        "memory"
    };

    // Ping Database
    let db_start = Instant::now();
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    let db_ms = elapsed_ms(db_start.elapsed());

    let all_ok = cache_ok && db_ok;

    Json(HealthResponse {
        status: if all_ok { "ok" } else { "degraded" },
        cache: CacheHealth {
            backend,
            status: if cache_ok {
                "connected"
            } else {
                "disconnected"
            },
            response_time_ms: cache_ms,
        },
        database: DatabaseHealth {
            status: if db_ok { "connected" } else { "disconnected" },
            response_time_ms: db_ms,
        },
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        response_time_ms: elapsed_ms(start.elapsed()),
    })
}
