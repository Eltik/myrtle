//! Worker heartbeats and what they say: online counts, tripped guards,
//! alert signals, rungs, the Neuron ledger and the model queue depth.

use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::UTC_DAY;

/// Workers seen in the last `online_secs`, and whether any of them offers
/// each model rung.
pub async fn workers_online(
    pool: &PgPool,
    online_secs: i64,
) -> Result<(i64, bool, bool), sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*), \
                COALESCE(bool_or((rungs ->> 'cf')::boolean), false), \
                COALESCE(bool_or((rungs ->> 'cpu')::boolean), false) \
         FROM trevor_workers WHERE last_seen > now() - make_interval(secs => $1)",
    )
    .bind(online_secs as f64)
    .fetch_one(pool)
    .await
}

/// Online workers per model rung and how many have a tripped guard:
/// (cf, cpu, tripped).
pub async fn worker_counts(
    pool: &PgPool,
    online_secs: i64,
) -> Result<(i64, i64, i64), sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) FILTER (WHERE COALESCE((rungs ->> 'cf')::boolean, false)), \
                count(*) FILTER (WHERE COALESCE((rungs ->> 'cpu')::boolean, false)), \
                count(*) FILTER (WHERE guard ->> 'state' = 'tripped') \
         FROM trevor_workers WHERE last_seen > now() - make_interval(secs => $1)",
    )
    .bind(online_secs as f64)
    .fetch_one(pool)
    .await
}

/// Online workers whose guard is tripped, with the reason they gave.
pub async fn tripped_workers(
    pool: &PgPool,
    online_secs: i64,
) -> Result<Vec<(String, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT worker_id, guard ->> 'reason' FROM trevor_workers \
         WHERE last_seen > now() - make_interval(secs => $1) AND guard ->> 'state' = 'tripped' \
         ORDER BY worker_id",
    )
    .bind(online_secs as f64)
    .fetch_all(pool)
    .await
}

/// What the alert evaluator measures, in one round trip.
#[derive(Debug, Clone, FromRow)]
pub struct AlertSignalsRow {
    pub secs_since_seen: Option<f64>,
    pub online: i64,
    pub queued: i64,
    pub oldest_queued_normal_secs: Option<f64>,
    pub finished: i64,
    pub failed: i64,
    pub neurons_today: i64,
}

/// `failed` counts a failed job, and a model-rung job whose answer came back
/// as passages (the worker's model failed and it fell back).
pub async fn alert_signals(
    pool: &PgPool,
    online_secs: i64,
    window: i64,
) -> Result<AlertSignalsRow, sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT \
           (SELECT extract(epoch FROM now() - max(last_seen))::float8 FROM trevor_workers) AS secs_since_seen, \
           (SELECT count(*) FROM trevor_workers WHERE last_seen > now() - make_interval(secs => $1)) AS online, \
           (SELECT count(*) FROM trevor_jobs WHERE status = 'queued') AS queued, \
           (SELECT extract(epoch FROM now() - min(queued_at))::float8 FROM trevor_jobs \
             WHERE status = 'queued' AND NOT deferred) AS oldest_queued_normal_secs, \
           f.finished, f.failed, \
           COALESCE((SELECT neurons::bigint FROM trevor_neurons WHERE day = {UTC_DAY}), 0) AS neurons_today \
         FROM (SELECT count(*) AS finished, \
                      count(*) FILTER (WHERE r.status = 'failed' \
                          OR (r.rung IN ('cf', 'cpu') AND a.source = 'passages')) AS failed \
               FROM (SELECT status, rung, answer_id FROM trevor_jobs \
                     WHERE status IN ('done', 'failed') AND finished_at IS NOT NULL \
                     ORDER BY finished_at DESC LIMIT $2) r \
               LEFT JOIN trevor_answers a ON a.id = r.answer_id) f"
    ))
    .bind(online_secs as f64)
    .bind(window)
    .fetch_one(pool)
    .await
}

/// The rungs one worker last reported, and today's Neurons.
pub async fn worker_rungs_and_neurons(
    pool: &PgPool,
    worker_id: &str,
) -> Result<(Option<Value>, i64), sqlx::Error> {
    sqlx::query_as(&format!(
        "SELECT (SELECT rungs FROM trevor_workers WHERE worker_id = $1), \
                COALESCE((SELECT neurons::bigint FROM trevor_neurons WHERE day = {UTC_DAY}), 0)"
    ))
    .bind(worker_id)
    .fetch_one(pool)
    .await
}

pub async fn neurons_today(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(&format!(
        "SELECT COALESCE((SELECT neurons::bigint FROM trevor_neurons WHERE day = {UTC_DAY}), 0)"
    ))
    .fetch_one(pool)
    .await
}

/// Jobs queued or running on the two model rungs: (cf, cpu).
pub async fn model_queue_depth(pool: &PgPool) -> Result<(i64, i64), sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) FILTER (WHERE rung = 'cf'), count(*) FILTER (WHERE rung = 'cpu') \
         FROM trevor_jobs WHERE status IN ('queued', 'running')",
    )
    .fetch_one(pool)
    .await
}

/// Record a worker's heartbeat.
pub async fn upsert_worker(
    pool: &PgPool,
    worker_id: &str,
    rungs: &Value,
    corpus_version: Option<&str>,
    busy_job_id: Option<Uuid>,
    guard: Option<&Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO trevor_workers (worker_id, last_seen, rungs, corpus_version, busy_job_id, guard)
        VALUES ($1, now(), $2, $3, $4, $5)
        ON CONFLICT (worker_id) DO UPDATE SET
            last_seen = now(), rungs = EXCLUDED.rungs,
            corpus_version = EXCLUDED.corpus_version, busy_job_id = EXCLUDED.busy_job_id,
            guard = EXCLUDED.guard
        ",
    )
    .bind(worker_id)
    .bind(rungs)
    .bind(corpus_version)
    .bind(busy_job_id)
    .bind(guard)
    .execute(pool)
    .await?;
    Ok(())
}
