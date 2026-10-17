//! The job queue: insert, read, ETAs, the claim, and the stale sweep.

use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::{JOB_COLUMNS, JobRow};

/// A job to queue.
pub struct NewJob<'a> {
    pub user_id: Option<Uuid>,
    pub question: &'a str,
    pub qkey: &'a str,
    pub horizon_pos: Option<i32>,
    pub server: &'a str,
    pub corpus_version: Option<&'a str>,
    pub rung: &'a str,
    pub profile: &'a str,
    /// Claimed only when no normal job the worker can serve waits.
    pub deferred: bool,
    /// `ask`, or `warm` for the update script's internal re-answers.
    pub kind: &'a str,
}

/// Queue a job, or `None` when an identical open job (same server, qkey,
/// horizon and deferral) won the race: the unique `trevor_jobs_open_key`
/// decides.
pub async fn insert_job<'e, E>(exec: E, job: &NewJob<'_>) -> Result<Option<JobRow>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query_as::<_, JobRow>(&format!(
        "INSERT INTO trevor_jobs (user_id, question, qkey, horizon_pos, server, corpus_version, rung, profile, deferred, kind) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         ON CONFLICT (server, qkey, horizon_pos, deferred) WHERE status IN ('queued', 'running') DO NOTHING \
         RETURNING {JOB_COLUMNS}"
    ))
    .bind(job.user_id)
    .bind(job.question)
    .bind(job.qkey)
    .bind(job.horizon_pos)
    .bind(job.server)
    .bind(job.corpus_version)
    .bind(job.rung)
    .bind(job.profile)
    .bind(job.deferred)
    .bind(job.kind)
    .fetch_optional(exec)
    .await
}

pub async fn job_by_id(pool: &PgPool, id: Uuid) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS} FROM trevor_jobs WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// The queued jobs of a job's server up to and including it, per rung; a
/// normal job does not count the deferred ones (they wait behind it).
pub async fn queued_ahead_by_rung(
    pool: &PgPool,
    job: &JobRow,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT rung, count(*) FROM trevor_jobs \
         WHERE server = $1 AND status = 'queued' AND created_at <= $2 \
           AND (NOT deferred OR $3) GROUP BY rung",
    )
    .bind(&job.server)
    .bind(job.created_at)
    .bind(job.deferred)
    .fetch_all(pool)
    .await
}

/// Per model rung: the median `finished_at - started_at` (seconds) of its
/// last [`crate::app::services::trevor::ETA_WINDOW`] finished jobs, and how
/// many there were.
pub async fn rung_etas(pool: &PgPool) -> Result<Vec<(String, Option<f64>, i64)>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT r.rung, s.median, s.n
        FROM unnest(ARRAY['cf', 'cpu', 'passages']) AS r(rung)
        CROSS JOIN LATERAL (
            SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY secs)::float8 AS median,
                   count(*) AS n
            FROM (
                SELECT extract(epoch FROM finished_at - started_at)::float8 AS secs
                FROM trevor_jobs
                WHERE rung = r.rung AND status = 'done'
                  AND started_at IS NOT NULL AND finished_at IS NOT NULL
                ORDER BY finished_at DESC
                LIMIT $1
            ) recent
        ) s
        ",
    )
    .bind(crate::app::services::trevor::ETA_WINDOW)
    .fetch_all(pool)
    .await
}

/// How `claim_next` picks (each a kill switch).
#[derive(Debug, Clone, Copy)]
pub struct ClaimRules {
    /// Normal jobs before deferred ones, quick passages jobs first among the
    /// normal (`TREVOR_ADMISSION`).
    pub admission: bool,
    /// Fewest running per user, then the oldest `queued_at`
    /// (`TREVOR_FAIR_CLAIM`); off: the oldest `created_at`.
    pub fair: bool,
    /// Only jobs this worker can serve at their rung (or that no other
    /// online worker can) (`TREVOR_CLAIM_BY_RUNG`).
    pub by_rung: bool,
    /// The claiming worker's rungs from its last heartbeat.
    pub cf: bool,
    pub cpu: bool,
    pub online_secs: i64,
}

/// Claim a queued job of a server for a worker. `SKIP LOCKED` lets several
/// pollers claim concurrently without ever taking the same row.
///
/// Order under all rules (contract v1.1 section 1c): normal before deferred,
/// the quick passages jobs first (they cost no model and were promised at
/// once), then the job whose user has the fewest jobs running, then the
/// oldest `queued_at`. With every rule off: the oldest `created_at`, as in v1.
pub async fn claim_next(
    pool: &PgPool,
    server: &str,
    worker_id: &str,
    rules: ClaimRules,
) -> Result<Option<JobRow>, sqlx::Error> {
    let mut order: Vec<&str> = Vec::new();
    if rules.admission {
        order.push("j.deferred");
        order.push("(j.rung <> 'passages')");
    }
    if rules.fair {
        order.push(
            "(SELECT count(*) FROM trevor_jobs r WHERE r.user_id = j.user_id AND r.status = 'running')",
        );
        order.push("j.queued_at");
    } else {
        order.push("j.created_at");
    }
    // A job this worker cannot serve at its rung is left to one that can,
    // unless no other online worker can either (then it steps down here).
    let rung_filter = if rules.by_rung {
        r"
          AND (j.rung = 'passages'
               OR (j.rung = 'cf' AND $3)
               OR (j.rung = 'cpu' AND $4)
               OR NOT EXISTS (
                   SELECT 1 FROM trevor_workers w
                   WHERE w.worker_id <> $2
                     AND w.last_seen > now() - make_interval(secs => $5)
                     AND COALESCE((w.rungs ->> j.rung)::boolean, false)))"
    } else {
        ""
    };
    sqlx::query_as::<_, JobRow>(&format!(
        r"
        UPDATE trevor_jobs SET status = 'running', worker_id = $2, started_at = now(),
               attempts = attempts + 1
        WHERE id = (
            SELECT j.id FROM trevor_jobs j
            WHERE j.server = $1 AND j.status = 'queued' {rung_filter}
            ORDER BY {order}
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING {JOB_COLUMNS}
        ",
        order = order.join(", ")
    ))
    .bind(server)
    .bind(worker_id)
    .bind(rules.cf)
    .bind(rules.cpu)
    .bind(rules.online_secs as f64)
    .fetch_optional(pool)
    .await
}

/// Step a claimed job's rung (and profile) down.
pub async fn set_job_rung(
    pool: &PgPool,
    job_id: Uuid,
    rung: &str,
    profile: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE trevor_jobs SET rung = $2, profile = $3 WHERE id = $1")
        .bind(job_id)
        .bind(rung)
        .bind(profile)
        .execute(pool)
        .await?;
    Ok(())
}

/// A job the sweep changed: requeued (`status = 'queued'`) or failed.
#[derive(Debug, Clone, FromRow)]
pub struct SweptJob {
    pub id: Uuid,
    pub status: String,
    pub error: Option<String>,
}

/// Requeue running jobs whose worker went quiet, or that ran past
/// `max_run_secs` whatever the heartbeat says (failing them once they had
/// `max_attempts` claims), and expire queued jobs that waited too long.
/// How the sweep treats its edge cases (each a kill switch).
#[derive(Debug, Clone, Copy)]
pub struct SweepRules {
    /// Expire on `queued_at` (reset by a requeue) instead of `created_at`.
    pub ttl_from_queued: bool,
    /// Never requeue a job an online worker (`busy_job_id`, seen within
    /// `online_secs`) says it is answering.
    pub busy_protects: bool,
    pub online_secs: i64,
    /// The busy protection ends once a job has run this long: it is then
    /// requeued or failed like a silent one. `0` means no ceiling.
    pub max_run_secs: i64,
    /// A queued deferred job expires after this many seconds instead.
    pub deferred_ttl_secs: i64,
}

pub async fn sweep_stale(
    pool: &PgPool,
    running_secs: i64,
    queued_secs: i64,
    max_attempts: i16,
    rules: SweepRules,
) -> Result<Vec<SweptJob>, sqlx::Error> {
    let mut swept: Vec<SweptJob> = sqlx::query_as(
        "UPDATE trevor_jobs j SET status = CASE WHEN attempts >= $2 THEN 'failed' ELSE 'queued' END, \
         worker_id = NULL, started_at = NULL, queued_at = now(), \
         error = CASE WHEN attempts < $2 THEN error \
                      WHEN $3 AND EXISTS (SELECT 1 FROM trevor_workers w WHERE w.busy_job_id = j.id \
                           AND w.last_seen > now() - make_interval(secs => $4)) \
                      THEN 'the job ran past its time limit' \
                      ELSE 'the worker stopped answering' END, \
         finished_at = CASE WHEN attempts >= $2 THEN now() ELSE NULL END \
         WHERE status = 'running' AND started_at < now() - make_interval(secs => $1) \
           AND NOT ($3 AND ($5 <= 0 OR started_at >= now() - make_interval(secs => $5)) \
                    AND EXISTS (SELECT 1 FROM trevor_workers w WHERE w.busy_job_id = j.id \
                    AND w.last_seen > now() - make_interval(secs => $4))) \
         RETURNING id, status, error",
    )
    .bind(running_secs as f64)
    .bind(max_attempts)
    .bind(rules.busy_protects)
    .bind(rules.online_secs as f64)
    .bind(rules.max_run_secs as f64)
    .fetch_all(pool)
    .await?;
    let expired: Vec<SweptJob> = sqlx::query_as(
        "UPDATE trevor_jobs SET status = 'failed', finished_at = now(), error = 'expired in the queue' \
         WHERE status = 'queued' \
           AND CASE WHEN $2 THEN queued_at ELSE created_at END \
               < now() - make_interval(secs => CASE WHEN deferred THEN $3 ELSE $1 END) \
         RETURNING id, status, error",
    )
    .bind(queued_secs as f64)
    .bind(rules.ttl_from_queued)
    .bind(rules.deferred_ttl_secs as f64)
    .fetch_all(pool)
    .await?;
    swept.extend(expired);
    Ok(swept)
}
