//! Trevor's tables (v039): versions and their panels, the job queue, answers,
//! worker heartbeats, the Neuron ledger and feedback. Callers live in
//! `app/services/trevor`.
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AnswerRow {
    pub id: Uuid,
    pub version: String,
    pub server: String,
    pub era_pos: i32,
    pub rung: String,
    pub source: String,
    pub answer: Value,
    pub created_at: DateTime<Utc>,
}

const ANSWER_COLUMNS: &str = "id, version, server, era_pos, rung, source, answer, created_at";

#[derive(Debug, Clone, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub kind: String,
    pub user_id: Option<Uuid>,
    pub question: String,
    pub qkey: String,
    pub horizon_pos: Option<i32>,
    pub server: String,
    pub corpus_version: Option<String>,
    pub rung: String,
    pub profile: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub error: Option<String>,
    pub attempts: i16,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub answer_id: Option<Uuid>,
}

const JOB_COLUMNS: &str = "id, kind, user_id, question, qkey, horizon_pos, server, corpus_version, rung, profile, \
     status, worker_id, error, attempts, created_at, started_at, finished_at, answer_id";

/// The UTC day the Neuron ledger books against.
const UTC_DAY: &str = "(now() AT TIME ZONE 'utc')::date";

/// The active corpus version of a server, if one was published.
pub async fn active_version(pool: &PgPool, server: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT version FROM trevor_versions WHERE server = $1 AND active")
        .bind(server)
        .fetch_optional(pool)
        .await
}

/// The active version of a server and its manifest's `eras` (not the whole
/// manifest: `storyOrder` alone is ~40 KB).
pub async fn active_version_eras(
    pool: &PgPool,
    server: &str,
) -> Result<Option<(String, Option<Value>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT version, manifest -> 'eras' FROM trevor_versions WHERE server = $1 AND active",
    )
    .bind(server)
    .fetch_optional(pool)
    .await
}

/// The highest gatePos among `story_ids` in one version's panels; `None`
/// when none of them has one.
pub async fn max_gate_pos(
    pool: &PgPool,
    version: &str,
    server: &str,
    story_ids: &[String],
) -> Result<Option<i32>, sqlx::Error> {
    if story_ids.is_empty() {
        return Ok(None);
    }
    sqlx::query_scalar(
        "SELECT max(gate_pos) FROM trevor_panels \
         WHERE version = $1 AND server = $2 AND story_id = ANY($3)",
    )
    .bind(version)
    .bind(server)
    .bind(story_ids)
    .fetch_one(pool)
    .await
}

/// One story's gatePos in a version: `None` when the story has no panel,
/// `Some(None)` when its panel carries no position.
pub async fn story_gate_pos(
    pool: &PgPool,
    version: &str,
    server: &str,
    story_id: &str,
) -> Result<Option<Option<i32>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT gate_pos FROM trevor_panels WHERE version = $1 AND server = $2 AND story_id = $3",
    )
    .bind(version)
    .bind(server)
    .bind(story_id)
    .fetch_optional(pool)
    .await
}

/// A cached answer by its full key, counting the hit.
pub async fn hit_answer(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "UPDATE trevor_answers SET hits = hits + 1 \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos = $4 AND cached \
         RETURNING {ANSWER_COLUMNS}"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_optional(pool)
    .await
}

pub async fn answer_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "SELECT {ANSWER_COLUMNS} FROM trevor_answers WHERE id = $1"
    ))
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
    horizon_pos: i32,
) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        "SELECT {JOB_COLUMNS} FROM trevor_jobs \
         WHERE server = $1 AND qkey = $2 AND horizon_pos = $3 AND status IN ('queued', 'running') \
         ORDER BY created_at LIMIT 1"
    ))
    .bind(server)
    .bind(qkey)
    .bind(horizon_pos)
    .fetch_optional(pool)
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
}

/// Queue a job, or `None` when an identical open job (same server, qkey and
/// horizon) won the race: the unique `trevor_jobs_open_key` decides.
pub async fn insert_job(pool: &PgPool, job: &NewJob<'_>) -> Result<Option<JobRow>, sqlx::Error> {
    sqlx::query_as::<_, JobRow>(&format!(
        "INSERT INTO trevor_jobs (user_id, question, qkey, horizon_pos, server, corpus_version, rung, profile) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (server, qkey, horizon_pos) WHERE status IN ('queued', 'running') DO NOTHING \
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
    .fetch_optional(pool)
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

/// The queued jobs of a job's server up to and including it, per rung.
pub async fn queued_ahead_by_rung(
    pool: &PgPool,
    job: &JobRow,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT rung, count(*) FROM trevor_jobs \
         WHERE server = $1 AND status = 'queued' AND created_at <= $2 GROUP BY rung",
    )
    .bind(&job.server)
    .bind(job.created_at)
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
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO trevor_workers (worker_id, last_seen, rungs, corpus_version, busy_job_id)
        VALUES ($1, now(), $2, $3, $4)
        ON CONFLICT (worker_id) DO UPDATE SET
            last_seen = now(), rungs = EXCLUDED.rungs,
            corpus_version = EXCLUDED.corpus_version, busy_job_id = EXCLUDED.busy_job_id
        ",
    )
    .bind(worker_id)
    .bind(rungs)
    .bind(corpus_version)
    .bind(busy_job_id)
    .execute(pool)
    .await?;
    Ok(())
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
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source, question, answer, cached)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true)
        ON CONFLICT (version, server, qkey, era_pos) WHERE cached
        DO UPDATE SET answer = EXCLUDED.answer, rung = EXCLUDED.rung, source = EXCLUDED.source,
                      gate_pos = EXCLUDED.gate_pos, created_at = now()
        RETURNING id
        "
    } else {
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source, question, answer, cached)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false)
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

/// A job the sweep changed: requeued (`status = 'queued'`) or failed.
#[derive(Debug, Clone, FromRow)]
pub struct SweptJob {
    pub id: Uuid,
    pub status: String,
    pub error: Option<String>,
}

/// Requeue running jobs whose worker went quiet (failing them once they had
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
         error = CASE WHEN attempts >= $2 THEN 'the worker stopped answering' ELSE error END, \
         finished_at = CASE WHEN attempts >= $2 THEN now() ELSE NULL END \
         WHERE status = 'running' AND started_at < now() - make_interval(secs => $1) \
           AND NOT ($3 AND EXISTS (SELECT 1 FROM trevor_workers w WHERE w.busy_job_id = j.id \
                    AND w.last_seen > now() - make_interval(secs => $4))) \
         RETURNING id, status, error",
    )
    .bind(running_secs as f64)
    .bind(max_attempts)
    .bind(rules.busy_protects)
    .bind(rules.online_secs as f64)
    .fetch_all(pool)
    .await?;
    let expired: Vec<SweptJob> = sqlx::query_as(
        "UPDATE trevor_jobs SET status = 'failed', finished_at = now(), error = 'expired in the queue' \
         WHERE status = 'queued' \
           AND CASE WHEN $2 THEN queued_at ELSE created_at END < now() - make_interval(secs => $1) \
         RETURNING id, status, error",
    )
    .bind(queued_secs as f64)
    .bind(rules.ttl_from_queued)
    .fetch_all(pool)
    .await?;
    swept.extend(expired);
    Ok(swept)
}

/// One panel as published: the story, its gatePos (the panel's `position`)
/// and the JSON.
pub struct PanelRow {
    pub story_id: String,
    pub gate_pos: Option<i32>,
    pub panels: Value,
}

/// Insert (or replace) a version's row and its panels; optionally make it the
/// active one of its server. A call without a manifest, gate report or build
/// time keeps the stored one, so `activate: false` chunks need not resend
/// them. Activation fills any panel without a position from the manifest's
/// `storyOrder`. One transaction, so readers never see a version active
/// without its panels.
#[allow(clippy::too_many_arguments)]
pub async fn publish_version(
    pool: &PgPool,
    version: &str,
    server: &str,
    built_at: Option<DateTime<Utc>>,
    gate_report: Option<&Value>,
    manifest: Option<&Value>,
    panels: &[PanelRow],
    activate: bool,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r"
        INSERT INTO trevor_versions (version, server, built_at, gate_report, manifest)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (version, server) DO UPDATE SET
            built_at = COALESCE(EXCLUDED.built_at, trevor_versions.built_at),
            gate_report = COALESCE(EXCLUDED.gate_report, trevor_versions.gate_report),
            manifest = COALESCE(EXCLUDED.manifest, trevor_versions.manifest),
            published_at = now()
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
        let ids: Vec<&str> = panels.iter().map(|p| p.story_id.as_str()).collect();
        let positions: Vec<Option<i32>> = panels.iter().map(|p| p.gate_pos).collect();
        let bodies: Vec<&Value> = panels.iter().map(|p| &p.panels).collect();
        sqlx::query(
            r"
            INSERT INTO trevor_panels (version, server, story_id, gate_pos, panels)
            SELECT $1, $2, t.story_id, t.gate_pos, t.panels
            FROM UNNEST($3::text[], $4::int[], $5::jsonb[]) AS t(story_id, gate_pos, panels)
            ON CONFLICT (version, server, story_id) DO UPDATE SET
                gate_pos = EXCLUDED.gate_pos, panels = EXCLUDED.panels
            ",
        )
        .bind(version)
        .bind(server)
        .bind(ids)
        .bind(positions)
        .bind(bodies)
        .execute(&mut *tx)
        .await?;
    }
    if activate {
        sqlx::query(
            r"
            UPDATE trevor_panels p SET gate_pos = o.pos::int
            FROM trevor_versions v,
                 jsonb_array_elements_text(v.manifest -> 'storyOrder') WITH ORDINALITY AS o(story_id, pos)
            WHERE v.version = $1 AND v.server = $2
              AND p.version = v.version AND p.server = v.server
              AND p.story_id = o.story_id AND p.gate_pos IS NULL
            ",
        )
        .bind(version)
        .bind(server)
        .execute(&mut *tx)
        .await?;
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
