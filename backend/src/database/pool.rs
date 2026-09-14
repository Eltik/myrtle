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
const DEFAULT_MAX_CONNECTIONS: u32 = 40;

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

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max = max_connections();
    tracing::info!(
        max_connections = max,
        acquire_timeout_secs = ACQUIRE_TIMEOUT.as_secs(),
        "creating database pool"
    );

    PgPoolOptions::new()
        .max_connections(max)
        .min_connections(2)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .idle_timeout(Duration::from_mins(10))
        .max_lifetime(Duration::from_mins(30))
        // sqlx defaults this to true, which spends a round-trip on every
        // checkout to prove the connection is alive - a cost paid per query,
        // and one that dominates a batch job. `max_lifetime` already retires
        // connections, and one that dies between checkouts surfaces as a query
        // error the caller can retry.
        .test_before_acquire(false)
        .connect(database_url)
        .await
}
