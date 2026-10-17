//! What admission control reads: open and queued jobs, the open twin of a
//! question, the admission lock and its inputs.

use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::{JOB_COLUMNS, JobRow, UTC_DAY};

/// Open jobs (queued or running) of one user; `normal_only` leaves out the
/// deferred ones (they have their own cap, [`deferred_jobs_of_user`]).
pub async fn open_jobs_of_user(
    pool: &PgPool,
    user_id: Uuid,
    normal_only: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs WHERE user_id = $1 AND status IN ('queued', 'running') \
         AND NOT ($2 AND deferred)",
    )
    .bind(user_id)
    .bind(normal_only)
    .fetch_one(pool)
    .await
}

/// Open deferred jobs of one user.
pub async fn deferred_jobs_of_user<'e, E>(exec: E, user_id: Uuid) -> Result<i64, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs WHERE user_id = $1 AND status IN ('queued', 'running') \
         AND deferred",
    )
    .bind(user_id)
    .fetch_one(exec)
    .await
}

/// Queued jobs of one server.
pub async fn queued_jobs(pool: &PgPool, server: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM trevor_jobs WHERE server = $1 AND status = 'queued'")
        .bind(server)
        .fetch_one(pool)
        .await
}

/// An open job asking the same question under the same horizon, so a second
/// asker joins it instead of spending a second model run. `deferred` picks
/// the normal or the deferred one.
pub async fn open_job_for_key<'e, E>(
    exec: E,
    server: &str,
    qkey: &str,
    horizon_pos: i32,
    deferred: bool,
) -> Result<Option<JobRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS} FROM trevor_jobs \
         WHERE server = $1 AND qkey = $2 AND horizon_pos = $3 AND deferred = $4 \
           AND status IN ('queued', 'running') \
         ORDER BY created_at LIMIT 1"
    ))
    .bind(server)
    .bind(qkey)
    .bind(horizon_pos)
    .bind(deferred)
    .fetch_optional(exec)
    .await
}

/// What admission control reads, in one round trip: the rungs any online
/// worker offers (several workers, contract v1.1 section 1d; the most recent
/// heartbeat's while none is online, as in v1), today's Neurons, normal jobs open on `cpu` and on `cf`,
/// online workers offering `cpu`, and every open job.
#[derive(Debug, Clone, FromRow)]
pub struct AdmissionRow {
    pub rungs: Option<Value>,
    pub neurons_today: i64,
    pub cpu_depth: i64,
    pub cf_depth: i64,
    pub cpu_workers: i64,
    pub open_total: i64,
}

/// Serialize admission (contract v1.1 section 1b): held to the end of the
/// caller's transaction, so concurrent asks read the rung depths one at a
/// time and the caps hold under a burst. Measured 2026-10-17 without it:
/// 2,000 concurrent asks over 50 keys put all 50 jobs on a cpu rung capped
/// at 24 (`tests/trevor_load_test.rs`).
pub async fn admission_lock<'e, E>(exec: E) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    // "trevor admission" as a fixed key.
    sqlx::query("SELECT pg_advisory_xact_lock(7365312011)")
        .execute(exec)
        .await?;
    Ok(())
}

pub async fn admission_inputs<'e, E>(exec: E, online_secs: i64) -> Result<AdmissionRow, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as(&format!(
        "SELECT \
           COALESCE( \
             (SELECT jsonb_build_object( \
                       'cf', bool_or(COALESCE((rungs ->> 'cf')::boolean, false)), \
                       'cpu', bool_or(COALESCE((rungs ->> 'cpu')::boolean, false))) \
              FROM trevor_workers WHERE last_seen > now() - make_interval(secs => $1) \
              HAVING count(*) > 0), \
             (SELECT rungs FROM trevor_workers ORDER BY last_seen DESC LIMIT 1)) AS rungs, \
           COALESCE((SELECT neurons::bigint FROM trevor_neurons WHERE day = {UTC_DAY}), 0) AS neurons_today, \
           o.cpu_depth, o.cf_depth, \
           (SELECT count(*) FROM trevor_workers \
             WHERE last_seen > now() - make_interval(secs => $1) \
               AND COALESCE((rungs ->> 'cpu')::boolean, false)) AS cpu_workers, \
           o.open_total \
         FROM (SELECT count(*) FILTER (WHERE rung = 'cpu' AND NOT deferred) AS cpu_depth, \
                      count(*) FILTER (WHERE rung = 'cf' AND NOT deferred) AS cf_depth, \
                      count(*) AS open_total \
               FROM trevor_jobs WHERE status IN ('queued', 'running')) o"
    ))
    .bind(online_secs as f64)
    .fetch_one(exec)
    .await
}

/// What the rung choice reads, in one round trip: the rungs of the most
/// recent heartbeat (online or not), today's Neurons, and the CPU rung's
/// live jobs.
pub async fn rung_inputs(pool: &PgPool) -> Result<(Option<Value>, i64, i64), sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT \
           (SELECT rungs FROM trevor_workers ORDER BY last_seen DESC LIMIT 1), \
           COALESCE((SELECT neurons::bigint FROM trevor_neurons WHERE day = {UTC_DAY}), 0), \
           (SELECT count(*) FROM trevor_jobs WHERE rung = 'cpu' AND status IN ('queued', 'running'))"
    ))
    .fetch_one(pool)
    .await
}
