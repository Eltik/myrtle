use std::time::Duration;

use async_trait::async_trait;
use myrtle_db::{Conn, Db, DbError};
use myrtle_types::{JobId, Server, WorkerId};
use serde_json::Value;
use sqlx::postgres::PgQueryResult;
use thiserror::Error;

use crate::model::{Job, JobKind, JobSpec, JobState};

const ENQUEUE: &str = "
INSERT INTO jobs (kind, server_id, payload, dedupe_key, priority, run_at, max_attempts, lease_secs)
VALUES ($1, $2, $3, $4, $5, COALESCE($6, now()), $7, $8)
ON CONFLICT (kind, dedupe_key) WHERE state = 'queued' DO UPDATE
   SET priority = GREATEST(jobs.priority, EXCLUDED.priority),
       run_at = LEAST(jobs.run_at, EXCLUDED.run_at)
RETURNING id";

// Dropping the key lets a newer enqueue queue behind a run that already read its input.
const CLAIM: &str = "
WITH due AS MATERIALIZED (
    SELECT id
      FROM jobs
     WHERE state = 'queued' AND run_at <= now() AND kind = ANY($2)
     ORDER BY priority DESC, run_at, id
     LIMIT $3
       FOR UPDATE SKIP LOCKED)
UPDATE jobs
   SET state = 'running',
       attempts = attempts + 1,
       dedupe_key = NULL,
       leased_by = $1,
       lease_expires_at = now() + make_interval(secs => lease_secs)
  FROM due
 WHERE jobs.id = due.id
RETURNING jobs.id, kind, server_id, payload, state, priority, attempts, max_attempts, lease_secs,
          run_at, leased_by, lease_expires_at, last_error, created_at, finished_at";

const HEARTBEAT: &str = "
UPDATE jobs
   SET lease_expires_at = now() + make_interval(secs => lease_secs)
 WHERE id = $1 AND leased_by = $2 AND attempts = $3 AND state = 'running'";

const COMPLETE: &str = "
UPDATE jobs
   SET state = 'succeeded',
       finished_at = now(),
       leased_by = NULL,
       lease_expires_at = NULL,
       output = $4
 WHERE id = $1 AND leased_by = $2 AND attempts = $3 AND state = 'running'";

const FAIL: &str = "
UPDATE jobs
   SET state = CASE WHEN attempts >= max_attempts THEN 'dead' ELSE 'queued' END,
       finished_at = CASE WHEN attempts >= max_attempts THEN now() END,
       run_at = now() + make_interval(
           secs => COALESCE($5::float8, LEAST(3600, 60 * power(4::float8, attempts - 1)))),
       leased_by = NULL,
       lease_expires_at = NULL,
       last_error = $4
 WHERE id = $1 AND leased_by = $2 AND attempts = $3 AND state = 'running'";

const REAP: &str = "
UPDATE jobs
   SET state = CASE WHEN attempts >= max_attempts THEN 'dead' ELSE 'queued' END,
       finished_at = CASE WHEN attempts >= max_attempts THEN now() END,
       leased_by = NULL,
       lease_expires_at = NULL,
       last_error = 'lease expired'
 WHERE state = 'running' AND lease_expires_at < now()";

const PRUNE: &str = "DELETE FROM jobs WHERE finished_at < now() - make_interval(secs => $1)";

const RECENT: &str = "
SELECT id, kind, server_id, payload, state, priority, attempts, max_attempts, lease_secs,
       run_at, leased_by, lease_expires_at, last_error, created_at, finished_at
  FROM jobs
 WHERE ($1::text IS NULL OR state = $1) AND ($2::text IS NULL OR kind = $2)
 ORDER BY id DESC
 LIMIT $3";

const COUNTS: &str = "
SELECT state, count(*)
  FROM jobs
 WHERE $1::text IS NULL OR kind = $1
 GROUP BY state";

#[derive(Debug, Clone, Copy)]
pub struct JobFilter {
    pub state: Option<JobState>,
    pub kind: Option<JobKind>,
    pub limit: u32,
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error("job {} is not leased by this worker", .0.0)]
    LeaseLost(JobId),
}

impl From<sqlx::Error> for QueueError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e.into())
    }
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, spec: JobSpec) -> Result<JobId, QueueError>;

    async fn claim(
        &self,
        worker: WorkerId,
        kinds: &[JobKind],
        max: u16,
    ) -> Result<Vec<Job>, QueueError>;

    async fn heartbeat(&self, worker: WorkerId, job: JobId, attempt: u16)
    -> Result<(), QueueError>;

    async fn complete(
        &self,
        worker: WorkerId,
        job: JobId,
        attempt: u16,
        output: Option<Value>,
    ) -> Result<(), QueueError>;

    async fn fail(
        &self,
        worker: WorkerId,
        job: JobId,
        attempt: u16,
        error: &str,
        retry_in: Option<Duration>,
    ) -> Result<(), QueueError>;
}

pub async fn enqueue_in(conn: &mut Conn, spec: &JobSpec) -> Result<JobId, QueueError> {
    let id = sqlx::query_scalar(ENQUEUE)
        .bind(spec.kind.as_str())
        .bind(spec.server.map(Server::index))
        .bind(&spec.payload)
        .bind(spec.dedupe_key.as_deref())
        .bind(spec.priority)
        .bind(spec.run_at)
        .bind(spec.kind.max_attempts() as i16)
        .bind(spec.kind.lease_secs() as i32)
        .fetch_one(conn)
        .await?;
    Ok(JobId(id))
}

#[derive(Debug, Clone)]
pub struct PgQueue {
    db: Db,
}

impl PgQueue {
    #[must_use]
    pub const fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn reap_expired(&self) -> Result<u64, QueueError> {
        let mut conn = self.db.conn().await?;
        let reaped = sqlx::query(REAP).execute(&mut *conn).await?;
        Ok(reaped.rows_affected())
    }

    pub async fn prune(&self, older_than: Duration) -> Result<u64, QueueError> {
        let mut conn = self.db.conn().await?;
        let pruned = sqlx::query(PRUNE)
            .bind(older_than.as_secs_f64())
            .execute(&mut *conn)
            .await?;
        Ok(pruned.rows_affected())
    }

    pub async fn recent(&self, filter: JobFilter) -> Result<Vec<Job>, QueueError> {
        let mut conn = self.db.conn().await?;
        Ok(sqlx::query_as(RECENT)
            .bind(filter.state.map(JobState::as_str))
            .bind(filter.kind.map(JobKind::as_str))
            .bind(i64::from(filter.limit))
            .fetch_all(&mut *conn)
            .await?)
    }

    pub async fn counts(&self, kind: Option<JobKind>) -> Result<Vec<(JobState, i64)>, QueueError> {
        let mut conn = self.db.conn().await?;
        let rows: Vec<(String, i64)> = sqlx::query_as(COUNTS)
            .bind(kind.map(JobKind::as_str))
            .fetch_all(&mut *conn)
            .await?;
        Ok(JobState::ALL
            .into_iter()
            .map(|state| {
                let n = rows.iter().find(|(s, _)| s == state.as_str());
                (state, n.map_or(0, |(_, n)| *n))
            })
            .collect())
    }
}

#[async_trait]
impl JobQueue for PgQueue {
    async fn enqueue(&self, spec: JobSpec) -> Result<JobId, QueueError> {
        enqueue_in(&mut *self.db.conn().await?, &spec).await
    }

    async fn claim(
        &self,
        worker: WorkerId,
        kinds: &[JobKind],
        max: u16,
    ) -> Result<Vec<Job>, QueueError> {
        let kinds: Vec<&str> = kinds.iter().map(|k| k.as_str()).collect();
        if kinds.is_empty() || max == 0 {
            return Ok(Vec::new());
        }
        let mut conn = self.db.conn().await?;
        Ok(sqlx::query_as(CLAIM)
            .bind(worker.0)
            .bind(kinds)
            .bind(i64::from(max))
            .fetch_all(&mut *conn)
            .await?)
    }

    async fn heartbeat(
        &self,
        worker: WorkerId,
        job: JobId,
        attempt: u16,
    ) -> Result<(), QueueError> {
        let mut conn = self.db.conn().await?;
        let beat = sqlx::query(HEARTBEAT)
            .bind(job.0)
            .bind(worker.0)
            .bind(attempt as i16)
            .execute(&mut *conn)
            .await?;
        held(&beat, job)
    }

    async fn complete(
        &self,
        worker: WorkerId,
        job: JobId,
        attempt: u16,
        output: Option<Value>,
    ) -> Result<(), QueueError> {
        let mut conn = self.db.conn().await?;
        let done = sqlx::query(COMPLETE)
            .bind(job.0)
            .bind(worker.0)
            .bind(attempt as i16)
            .bind(output)
            .execute(&mut *conn)
            .await?;
        held(&done, job)
    }

    async fn fail(
        &self,
        worker: WorkerId,
        job: JobId,
        attempt: u16,
        error: &str,
        retry_in: Option<Duration>,
    ) -> Result<(), QueueError> {
        let mut conn = self.db.conn().await?;
        let failed = sqlx::query(FAIL)
            .bind(job.0)
            .bind(worker.0)
            .bind(attempt as i16)
            .bind(error)
            .bind(retry_in.map(|d| d.as_secs_f64()))
            .execute(&mut *conn)
            .await?;
        held(&failed, job)
    }
}

fn held(result: &PgQueryResult, job: JobId) -> Result<(), QueueError> {
    if result.rows_affected() == 0 {
        return Err(QueueError::LeaseLost(job));
    }
    Ok(())
}
