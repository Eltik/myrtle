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
    pub deferred: bool,
}

const JOB_COLUMNS: &str = "id, kind, user_id, question, qkey, horizon_pos, server, corpus_version, rung, profile, \
     status, worker_id, error, attempts, created_at, started_at, finished_at, answer_id, deferred";

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

/// "Quick look": the same question's cached answer from the latest era
/// before `era_pos` (so spoiler-safe for this reader). No hit is counted.
pub async fn quick_look(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "SELECT {ANSWER_COLUMNS} FROM trevor_answers \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos < $4 AND cached \
         ORDER BY era_pos DESC LIMIT 1"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_optional(pool)
    .await
}

/// Whether a cached answer exists under the full key (no hit counted).
/// Whether a version of a server was published.
pub async fn version_known(
    pool: &PgPool,
    version: &str,
    server: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trevor_versions WHERE version = $1 AND server = $2)",
    )
    .bind(version)
    .bind(server)
    .fetch_one(pool)
    .await
}

pub async fn has_cached(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trevor_answers \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos = $4 AND cached)",
    )
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_one(pool)
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
                      carried_from = NULL, created_at = now()
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
    chunk_shas: &[(String, String)],
    carry_over: bool,
) -> Result<u64, sqlx::Error> {
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
    if !chunk_shas.is_empty() {
        let ids: Vec<&str> = chunk_shas.iter().map(|(id, _)| id.as_str()).collect();
        let shas: Vec<&str> = chunk_shas.iter().map(|(_, sha)| sha.as_str()).collect();
        sqlx::query(
            r"
            INSERT INTO trevor_chunk_shas (version, server, chunk_id, sha)
            SELECT $1, $2, t.chunk_id, t.sha FROM UNNEST($3::text[], $4::text[]) AS t(chunk_id, sha)
            ON CONFLICT (version, server, chunk_id) DO UPDATE SET sha = EXCLUDED.sha
            ",
        )
        .bind(version)
        .bind(server)
        .bind(ids)
        .bind(shas)
        .execute(&mut *tx)
        .await?;
    }
    let mut carried = 0;
    if activate && carry_over {
        carried = carry_answers(&mut tx, version, server).await?;
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
    tx.commit().await?;
    Ok(carried)
}

/// The cache carry-over (contract v1.1 section 3b), run while activating
/// `version`, before the active flag moves: copy each cached answer of the
/// server's active version whose every cited chunk is in `version`'s chunk
/// sha map with the same sha, and whose era position is 0, the internal
/// no-limit era, or a position of `version`'s eras. An answer with no
/// `cited_shas` (a citation without a sha, or none) never moves. The copy
/// keeps its hits and names its source in `carried_from`; an answer `version`
/// already holds under the key wins.
async fn carry_answers(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    version: &str,
    server: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source,
                                    question, answer, cached, cited_shas, carried_from, hits)
        SELECT $1, a.server, a.qkey, a.era_pos, a.gate_pos, a.rung, a.source,
               a.question, a.answer, true, a.cited_shas, a.version, a.hits
        FROM trevor_answers a
        JOIN trevor_versions pv ON pv.version = a.version AND pv.server = a.server AND pv.active
        WHERE a.server = $2 AND a.version <> $1 AND a.cached
          AND a.cited_shas IS NOT NULL AND a.cited_shas <> '{}'::jsonb
          AND (a.era_pos = 0 OR a.era_pos = 2147483647 OR a.era_pos IN (
                SELECT (e ->> 'position')::int
                FROM trevor_versions nv, jsonb_array_elements(nv.manifest -> 'eras') e
                WHERE nv.version = $1 AND nv.server = $2))
          AND NOT EXISTS (
                SELECT 1 FROM jsonb_each_text(a.cited_shas) c
                WHERE NOT EXISTS (
                    SELECT 1 FROM trevor_chunk_shas s
                    WHERE s.version = $1 AND s.server = $2
                      AND s.chunk_id = c.key AND s.sha = c.value))
        ON CONFLICT (version, server, qkey, era_pos) WHERE cached DO NOTHING
        ",
    )
    .bind(version)
    .bind(server)
    .execute(&mut **tx)
    .await?;
    Ok(done.rows_affected())
}

// --- asks: "My questions", popular, warm-up (contract v1.1 sections 3c, 5) --

/// One ask to record.
pub struct NewAsk<'a> {
    pub user_id: Uuid,
    pub server: &'a str,
    pub version: Option<&'a str>,
    pub question: &'a str,
    pub normq: &'a str,
    pub qkey: &'a str,
    pub era_pos: i32,
    pub job_id: Option<Uuid>,
    pub deferred_job_id: Option<Uuid>,
    pub answer_id: Option<Uuid>,
}

pub async fn insert_ask(pool: &PgPool, a: &NewAsk<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO trevor_asks (user_id, server, version, question, normq, qkey, era_pos, job_id, \
                                  deferred_job_id, answer_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(a.user_id)
    .bind(a.server)
    .bind(a.version)
    .bind(a.question)
    .bind(a.normq)
    .bind(a.qkey)
    .bind(a.era_pos)
    .bind(a.job_id)
    .bind(a.deferred_job_id)
    .bind(a.answer_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// One row of "My questions": the ask, its primary job (the deferred one
/// when there is one) and that job's (or the cache hit's) answer.
#[derive(Debug, Clone, FromRow)]
pub struct MyAskRow {
    pub question: String,
    pub created_at: DateTime<Utc>,
    pub job_id: Option<Uuid>,
    pub status: Option<String>,
    pub rung: Option<String>,
    pub finished_at: Option<DateTime<Utc>>,
    pub hit: bool,
    pub answer_id: Option<Uuid>,
    pub a_version: Option<String>,
    pub a_server: Option<String>,
    pub a_era_pos: Option<i32>,
    pub a_rung: Option<String>,
    pub a_source: Option<String>,
    pub a_answer: Option<Value>,
}

pub async fn my_asks(
    pool: &PgPool,
    user_id: Uuid,
    before: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<MyAskRow>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT k.question, k.created_at, j.id AS job_id, j.status, j.rung, j.finished_at,
               (j.id IS NULL AND k.answer_id IS NOT NULL) AS hit,
               a.id AS answer_id, a.version AS a_version, a.server AS a_server,
               a.era_pos AS a_era_pos, a.rung AS a_rung, a.source AS a_source, a.answer AS a_answer
        FROM trevor_asks k
        LEFT JOIN trevor_jobs j ON j.id = COALESCE(k.deferred_job_id, k.job_id)
        LEFT JOIN trevor_answers a ON a.id = CASE
            WHEN j.id IS NULL THEN k.answer_id
            WHEN j.status = 'done' THEN j.answer_id
        END
        WHERE k.user_id = $1 AND ($2::timestamptz IS NULL OR k.created_at < $2)
        ORDER BY k.created_at DESC
        LIMIT $3
        ",
    )
    .bind(user_id)
    .bind(before)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// The most-asked cached questions of a version that a reader at `era_pos`
/// may see: per question, the cached answer of the latest era at or before
/// `era_pos`, listed when at least `min_askers` distinct accounts asked it
/// on the server (in any version, so a publish does not reset the list),
/// most askers (then hits) first.
pub async fn popular(
    pool: &PgPool,
    version: &str,
    server: &str,
    era_pos: i32,
    min_askers: i64,
    limit: i64,
) -> Result<Vec<(i64, AnswerRow, String)>, sqlx::Error> {
    #[derive(FromRow)]
    struct Row {
        askers: i64,
        question: String,
        #[sqlx(flatten)]
        answer: AnswerRow,
    }
    let rows: Vec<Row> = sqlx::query_as(
        r"
        WITH askers AS (
            SELECT qkey, count(DISTINCT user_id) AS askers
            FROM trevor_asks WHERE server = $2
            GROUP BY qkey HAVING count(DISTINCT user_id) >= $4
        ), best AS (
            SELECT DISTINCT ON (a.qkey) a.*
            FROM trevor_answers a JOIN askers k ON k.qkey = a.qkey
            WHERE a.version = $1 AND a.server = $2 AND a.cached AND a.era_pos <= $3
            ORDER BY a.qkey, a.era_pos DESC
        )
        SELECT k.askers, b.question, b.id, b.version, b.server, b.era_pos, b.rung, b.source,
               b.answer, b.created_at
        FROM best b JOIN askers k ON k.qkey = b.qkey
        ORDER BY k.askers DESC, b.hits DESC, b.qkey
        LIMIT $5
        ",
    )
    .bind(version)
    .bind(server)
    .bind(era_pos)
    .bind(min_askers)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.askers, r.answer, r.question))
        .collect())
}

/// The warm-up list (contract v1.1 section 3c): the questions most asked in
/// the last `days` days, per (normQ, era), that `version` holds no cached
/// answer for (so neither carried over nor answered since): (latest asked
/// text, normQ, eraPos, asks).
pub async fn warm_candidates(
    pool: &PgPool,
    version: &str,
    server: &str,
    days: i32,
    limit: i64,
) -> Result<Vec<(String, String, i32, i64)>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT (array_agg(k.question ORDER BY k.created_at DESC))[1] AS question,
               (array_agg(k.normq ORDER BY k.created_at DESC))[1] AS normq,
               k.era_pos, count(*) AS asks
        FROM trevor_asks k
        WHERE k.server = $2 AND k.created_at > now() - make_interval(days => $3)
          AND NOT EXISTS (SELECT 1 FROM trevor_answers a
                          WHERE a.version = $1 AND a.server = $2 AND a.qkey = k.qkey
                            AND a.era_pos = k.era_pos AND a.cached)
        GROUP BY k.qkey, k.era_pos
        ORDER BY count(*) DESC, k.qkey, k.era_pos
        LIMIT $4
        ",
    )
    .bind(version)
    .bind(server)
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await
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
