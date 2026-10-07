//! Trevor's tables (v035): versions and their panels, the job queue, answers
//! and feedback. Callers live in `app/services/trevor`.
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AnswerRow {
    pub id: Uuid,
    pub version: String,
    pub server: String,
    pub horizon: String,
    pub answer: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub kind: String,
    pub user_id: Option<Uuid>,
    pub question: String,
    pub qkey: String,
    pub horizon: String,
    pub server: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub answer_id: Option<Uuid>,
}

const JOB_COLUMNS: &str = "id, kind, user_id, question, qkey, horizon, server, status, worker_id, \
     error, created_at, started_at, finished_at, answer_id";

/// The active corpus version of a server, if one was published.
pub async fn active_version(pool: &PgPool, server: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT version FROM trevor_versions WHERE server = $1 AND active")
        .bind(server)
        .fetch_optional(pool)
        .await
}

/// A cached answer by its full key, counting the hit.
pub async fn hit_answer(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    horizon: &str,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(
        r"
        UPDATE trevor_answers SET hits = hits + 1
        WHERE version = $1 AND server = $2 AND qkey = $3 AND horizon = $4
        RETURNING id, version, server, horizon, answer, created_at
        ",
    )
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(horizon)
    .fetch_optional(pool)
    .await
}

pub async fn answer_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(
        "SELECT id, version, server, horizon, answer, created_at FROM trevor_answers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Open jobs (queued or running) of one user.
pub async fn open_jobs_of_user(pool: &PgPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs WHERE user_id = $1 AND status IN ('queued', 'running')",
    )
    .bind(user_id)
    .fetch_one(pool)
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
/// asker joins it instead of spending a second model run.
pub async fn open_job_for_key(
    pool: &PgPool,
    server: &str,
    qkey: &str,
    horizon: &str,
) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS} FROM trevor_jobs \
         WHERE server = $1 AND qkey = $2 AND horizon = $3 AND status IN ('queued', 'running') \
         ORDER BY created_at LIMIT 1"
    ))
    .bind(server)
    .bind(qkey)
    .bind(horizon)
    .fetch_optional(pool)
    .await
}

pub async fn insert_job(
    pool: &PgPool,
    user_id: Option<Uuid>,
    question: &str,
    qkey: &str,
    horizon: &str,
    server: &str,
) -> Result<JobRow, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        "INSERT INTO trevor_jobs (user_id, question, qkey, horizon, server) \
         VALUES ($1, $2, $3, $4, $5) RETURNING {JOB_COLUMNS}"
    ))
    .bind(user_id)
    .bind(question)
    .bind(qkey)
    .bind(horizon)
    .bind(server)
    .fetch_one(pool)
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

/// 1-based place of a queued job among its server's queued jobs.
pub async fn queue_position(pool: &PgPool, job: &JobRow) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs WHERE server = $1 AND status = 'queued' AND created_at <= $2",
    )
    .bind(&job.server)
    .bind(job.created_at)
    .fetch_one(pool)
    .await
}

/// Claim the oldest queued job of a server for a worker. `SKIP LOCKED` lets
/// several pollers claim concurrently without ever taking the same row.
pub async fn claim_next(
    pool: &PgPool,
    server: &str,
    worker_id: &str,
) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        r"
        UPDATE trevor_jobs SET status = 'running', worker_id = $2, started_at = now(),
               attempts = attempts + 1
        WHERE id = (
            SELECT id FROM trevor_jobs
            WHERE server = $1 AND status = 'queued'
            ORDER BY created_at
            FOR UPDATE SKIP LOCKED
            LIMIT 1
        )
        RETURNING {JOB_COLUMNS}
        "
    ))
    .bind(server)
    .bind(worker_id)
    .fetch_optional(pool)
    .await
}

/// Store a worker's answer under its key (the latest write wins) and close the
/// job, in one transaction. `None` when the job is not running under that worker.
pub async fn complete_job(
    pool: &PgPool,
    job_id: Uuid,
    worker_id: &str,
    version: &str,
    answer: &Value,
) -> Result<Option<Uuid>, sqlx::Error> {
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
    let answer_id: Uuid = sqlx::query_scalar(
        r"
        INSERT INTO trevor_answers (version, server, qkey, horizon, question, answer)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT ON CONSTRAINT trevor_answers_key
        DO UPDATE SET answer = EXCLUDED.answer, created_at = now()
        RETURNING id
        ",
    )
    .bind(version)
    .bind(&job.server)
    .bind(&job.qkey)
    .bind(&job.horizon)
    .bind(&job.question)
    .bind(answer)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE trevor_jobs SET status = 'done', finished_at = now(), answer_id = $2 WHERE id = $1",
    )
    .bind(job_id)
    .bind(answer_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some(answer_id))
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

/// Requeue running jobs whose worker went quiet, and expire queued jobs that
/// waited too long. Returns (requeued, expired).
pub async fn sweep_stale(
    pool: &PgPool,
    running_secs: i64,
    queued_secs: i64,
    max_attempts: i16,
) -> Result<(u64, u64), sqlx::Error> {
    let requeued = sqlx::query(
        "UPDATE trevor_jobs SET status = CASE WHEN attempts >= $2 THEN 'failed' ELSE 'queued' END, \
         worker_id = NULL, started_at = NULL, \
         error = CASE WHEN attempts >= $2 THEN 'the worker stopped answering' ELSE error END, \
         finished_at = CASE WHEN attempts >= $2 THEN now() ELSE NULL END \
         WHERE status = 'running' AND started_at < now() - make_interval(secs => $1)",
    )
    .bind(running_secs as f64)
    .bind(max_attempts)
    .execute(pool)
    .await?
    .rows_affected();
    let expired = sqlx::query(
        "UPDATE trevor_jobs SET status = 'failed', finished_at = now(), error = 'expired in the queue' \
         WHERE status = 'queued' AND created_at < now() - make_interval(secs => $1)",
    )
    .bind(queued_secs as f64)
    .execute(pool)
    .await?
    .rows_affected();
    Ok((requeued, expired))
}

/// Insert (or replace) a version's row and its panels; optionally make it the
/// active one of its server. One transaction, so readers never see a version
/// active without its panels.
#[allow(clippy::too_many_arguments)]
pub async fn publish_version(
    pool: &PgPool,
    version: &str,
    server: &str,
    built_at: Option<DateTime<Utc>>,
    gate_report: Option<&Value>,
    manifest: Option<&Value>,
    panels: &[(String, Value)],
    activate: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r"
        INSERT INTO trevor_versions (version, server, built_at, gate_report, manifest)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (version, server) DO UPDATE SET
            built_at = EXCLUDED.built_at, gate_report = EXCLUDED.gate_report,
            manifest = EXCLUDED.manifest, published_at = now()
        ",
    )
    .bind(version)
    .bind(server)
    .bind(built_at)
    .bind(gate_report)
    .bind(manifest)
    .execute(&mut *tx)
    .await?;
    if !panels.is_empty() {
        let (ids, bodies): (Vec<&str>, Vec<&Value>) =
            panels.iter().map(|(id, body)| (id.as_str(), body)).unzip();
        sqlx::query(
            r"
            INSERT INTO trevor_panels (version, server, story_id, panels)
            SELECT $1, $2, t.story_id, t.panels FROM UNNEST($3::text[], $4::jsonb[]) AS t(story_id, panels)
            ON CONFLICT (version, server, story_id) DO UPDATE SET panels = EXCLUDED.panels
            ",
        )
        .bind(version)
        .bind(server)
        .bind(ids)
        .bind(bodies)
        .execute(&mut *tx)
        .await?;
    }
    if activate {
        sqlx::query("UPDATE trevor_versions SET active = false WHERE server = $1 AND active")
            .bind(server)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE trevor_versions SET active = true WHERE version = $1 AND server = $2")
            .bind(version)
            .bind(server)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await
}

/// One story's panels under the active version of a server.
pub async fn active_panels(
    pool: &PgPool,
    server: &str,
    story_id: &str,
) -> Result<Option<(String, Value)>, sqlx::Error> {
    sqlx::query_as::<_, (String, Value)>(
        r"
        SELECT p.version, p.panels FROM trevor_panels p
        JOIN trevor_versions v ON v.version = p.version AND v.server = p.server AND v.active
        WHERE p.server = $1 AND p.story_id = $2
        ",
    )
    .bind(server)
    .bind(story_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_feedback(
    pool: &PgPool,
    answer_id: Uuid,
    user_id: Uuid,
    vote: i16,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO trevor_feedback (answer_id, user_id, vote, note)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (answer_id, user_id) DO UPDATE SET
            vote = EXCLUDED.vote, note = EXCLUDED.note, updated_at = now()
        ",
    )
    .bind(answer_id)
    .bind(user_id)
    .bind(vote)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}
