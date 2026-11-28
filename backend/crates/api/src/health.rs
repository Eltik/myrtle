use std::collections::BTreeMap;
use std::time::Instant;

use axum::Json;
use axum::extract::State;
use myrtle_app::AppContext;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    status: &'static str,
    cache: CacheHealth,
    database: DatabaseHealth,
    game_data: GameDataHealth,
    expired_switches: Vec<&'static str>,
    timestamp: String,
    response_time_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CacheHealth {
    backend: &'static str,
    status: &'static str,
    response_time_ms: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatabaseHealth {
    status: &'static str,
    response_time_ms: f64,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameDataHealth {
    degraded: bool,
    generations: BTreeMap<&'static str, i64>,
    tables: BTreeMap<&'static str, Vec<String>>,
}

const fn connection(up: bool) -> &'static str {
    if up { "connected" } else { "disconnected" }
}

fn ms_since(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

pub async fn health(State(ctx): State<AppContext>) -> Json<HealthResponse> {
    let start = Instant::now();
    let cache_up = ctx.cache.ping().await;
    let cache_ms = ms_since(start);
    let db_start = Instant::now();
    let db_up = ctx.db.ping().await.is_ok();
    let db_ms = ms_since(db_start);

    let mut game_data = GameDataHealth::default();
    for server in ctx.snapshots.loaded() {
        let Some(snapshot) = ctx.snapshots.get(server) else {
            continue;
        };
        game_data
            .generations
            .insert(server.as_str(), snapshot.generation);
        let warnings = ctx.table_warnings(&snapshot);
        if !warnings.is_empty() {
            game_data.tables.insert(server.as_str(), warnings);
        }
    }
    game_data.degraded = !game_data.tables.is_empty();

    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    Json(HealthResponse {
        status: if cache_up && db_up { "ok" } else { "degraded" },
        cache: CacheHealth {
            backend: if ctx.cache.is_redis() {
                "redis"
            } else {
                "memory"
            },
            status: connection(cache_up),
            response_time_ms: cache_ms,
        },
        database: DatabaseHealth {
            status: connection(db_up),
            response_time_ms: db_ms,
        },
        game_data,
        expired_switches: ctx
            .config
            .expired_switches(&today)
            .into_iter()
            .map(|switch| switch.name)
            .collect(),
        timestamp: now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        response_time_ms: ms_since(start),
    })
}
