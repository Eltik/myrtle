//! A job's end: its answer stored, the Neuron ledger booked, or a failure.

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::{JOB_COLUMNS, JobRow, UTC_DAY};

/// A worker's answer, ready to store.
pub struct CompletedAnswer<'a> {
    pub version: &'a str,
    pub rung: &'a str,
    pub source: &'a str,
    pub answer: &'a Value,
    /// Neurons the answer cost if it ran on Cloudflare.
    pub neurons: i64,
    /// The worker says the answer came from the `cf` rung.
    pub reported_cf: bool,
    /// Serve this answer from the cache. False for a passages-only answer
    /// (unless `TREVOR_CACHE_LIVE_ONLY=0`); the version check below can also
    /// clear it.
    pub cacheable: bool,
    /// Store as uncached when the job's corpus version differs from the
    /// answer's (`TREVOR_CACHE_VERSION_CHECK=0` turns the check off).
    pub version_check: bool,
    /// {chunkId: sha} of the cited chunks, when every citation carried one.
    pub cited_shas: Option<&'a Value>,
}

/// Add Neurons to today's (UTC) ledger row.
pub async fn book_neurons<'e, E>(exec: E, neurons: i64) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    if neurons <= 0 {
        return Ok(());
    }
    let add = i32::try_from(neurons).unwrap_or(i32::MAX);
    sqlx::query(&format!(
        "INSERT INTO trevor_neurons (day, neurons) VALUES ({UTC_DAY}, $1) \
         ON CONFLICT (day) DO UPDATE SET neurons = \
         LEAST(trevor_neurons.neurons::bigint + EXCLUDED.neurons, 2147483647)::int"
    ))
    .bind(add)
    .execute(exec)
    .await?;
    Ok(())
}

/// Store a worker's answer under its key (the latest write wins), book its
/// Neurons when the job or the answer is on the `cf` rung, and close the job,
/// in one transaction. `None` when the job is not running under that worker.
pub async fn complete_job(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: &str,
    done: &CompletedAnswer<'_>,
) -> Result<Option<(Uuid, JobRow)>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let job: Option<JobRow> = sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS} FROM trevor_jobs WHERE id = $1 AND status = 'running' AND worker_id = $2 FOR UPDATE"
    ))
    .bind(job_id)
    .bind(worker_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(job) = job else {
        return Ok(None);
    };
    let era_pos = job
        .horizon_pos
        .unwrap_or(crate::app::services::trevor::NO_LIMIT_ERA);
    let cached = done.cacheable
        && !(done.version_check
            && job
                .corpus_version
                .as_deref()
                .is_some_and(|v| v != done.version));
    // Only a served answer takes the cache key; an uncached one is a row of
    // its own, so it never shadows (or replaces) a cached answer.
    let sql = if cached {
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source, question, answer, cached, cited_shas)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10)
        ON CONFLICT (version, server, qkey, era_pos) WHERE cached
        DO UPDATE SET answer = EXCLUDED.answer, rung = EXCLUDED.rung, source = EXCLUDED.source,
                      gate_pos = EXCLUDED.gate_pos, cited_shas = EXCLUDED.cited_shas,
                      carried_from = NULL, origin = NULL, created_at = now()
        RETURNING id
        "
    } else {
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source, question, answer, cached, cited_shas)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, $10)
        RETURNING id
        "
    };
    let answer_id: Uuid = sqlx::query_scalar(sql)
        .bind(done.version)
        .bind(&job.server)
        .bind(&job.qkey)
        .bind(era_pos)
        .bind(job.horizon_pos)
        .bind(done.rung)
        .bind(done.source)
        .bind(&job.question)
        .bind(done.answer)
        .bind(done.cited_shas)
        .fetch_one(&mut *tx)
        .await?;
    if done.reported_cf || job.rung == "cf" {
        book_neurons(&mut *tx, done.neurons).await?;
    }
    let job: JobRow = sqlx::query_as::<_, JobRow>(&format!(
        "UPDATE trevor_jobs SET status = 'done', finished_at = now(), answer_id = $2 \
         WHERE id = $1 RETURNING {JOB_COLUMNS}"
    ))
    .bind(job_id)
    .bind(answer_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some((answer_id, job)))
}

/// Close a running job as failed. False when it is not running under that worker.
pub async fn fail_job(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: &str,
    error: &str,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE trevor_jobs SET status = 'failed', finished_at = now(), error = $3 \
         WHERE id = $1 AND status = 'running' AND worker_id = $2",
    )
    .bind(job_id)
    .bind(worker_id)
    .bind(error)
    .execute(pool)
    .await?;
    Ok(done.rows_affected() == 1)
}
