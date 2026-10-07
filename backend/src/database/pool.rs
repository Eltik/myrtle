use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// Connections the pool may open.
///
/// Size this against the widest single unit of work rather than against request
/// count: grading one account holds eight connections at once, so a background
/// pass at concurrency four wants more than thirty before live traffic is
/// served at all. Past the limit `acquire` waits out `ACQUIRE_TIMEOUT` and
/// fails, which sheds as a 503 (see `ApiError::from<sqlx::Error>`).
///
/// It is an env var because the ceiling is really Postgres's: its own
/// `max_connections` must exceed this times the number of app instances, plus
/// whatever else connects.
///
/// 20, down from 40 (`DATABASE_MAX_CONNECTIONS=40` restores it). A TRADE for
/// the 3-vCPU VPS, not a derivation: Postgres runs one process per
/// connection, and three cores cannot run more than a handful of queries at
/// once, so connections past that only queue inside Postgres while each holds
/// its own backend's memory.
///
/// Measured 2026-10-07 on the local database (8 real accounts,
/// `tests/pool_footprint_test.rs`, read-only): one `calculate_user_grade`
/// holds up to 9 connections at once, and the regrade pass at its default
/// concurrency 4 peaked at 32 busy (33 opened). So a regrade pass now WAITS
/// for connections inside this pool instead of opening 12 more backends, and
/// requests arriving during a pass share that queue. Each grade query holds
/// its connection for milliseconds, far under `ACQUIRE_TIMEOUT`, but whether
/// request latency moves during a pass is not measured: a
/// `database pool exhausted; shedding request` warning in production would
/// say it does (then raise this, or lower `REGRADE_CONCURRENCY`).
const DEFAULT_MAX_CONNECTIONS: u32 = 20;

/// Connections kept open while idle. Unchanged at 2;
/// `DATABASE_MIN_CONNECTIONS` overrides.
const DEFAULT_MIN_CONNECTIONS: u32 = 2;

/// How long a connection above the minimum may sit idle before it is closed.
/// 60 s, down from 600 (`DATABASE_IDLE_TIMEOUT_SECS=600` restores it): a burst
/// (a regrade pass, a traffic spike) used to leave its extra backends resident
/// for ten minutes after it ended.
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 60;

/// How long a caller waits for a connection before being shed. Short on
/// purpose: a request that has been queued five seconds has a client that has
/// usually given up, and holding it only deepens the queue.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

fn max_connections() -> u32 {
    std::env::var("DATABASE_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(DEFAULT_MAX_CONNECTIONS)
}

fn min_connections(max: u32) -> u32 {
    std::env::var("DATABASE_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(DEFAULT_MIN_CONNECTIONS)
        .min(max)
}

fn idle_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("DATABASE_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .filter(|n| *n > 0)
            .unwrap_or(DEFAULT_IDLE_TIMEOUT_SECS),
    )
}

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max = max_connections();
    let min = min_connections(max);
    let idle = idle_timeout();
    tracing::info!(
        max_connections = max,
        min_connections = min,
        idle_timeout_secs = idle.as_secs(),
        acquire_timeout_secs = ACQUIRE_TIMEOUT.as_secs(),
        "creating database pool"
    );

    PgPoolOptions::new()
        .max_connections(max)
        .min_connections(min)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .idle_timeout(idle)
        .max_lifetime(Duration::from_mins(30))
        // sqlx defaults this to true: a round-trip per checkout, which dominates batch
        // jobs. `max_lifetime` already retires connections, and one that dies between
        // checkouts surfaces as a retryable query error.
        .test_before_acquire(false)
        .connect(database_url)
        .await
}
