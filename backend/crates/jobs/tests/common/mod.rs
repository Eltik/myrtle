//! A `PgQueue` over the database `#[sqlx::test]` hands each test, and row probes.

#![allow(dead_code, clippy::unwrap_used)]

use chrono::{DateTime, Utc};
use myrtle_db::Db;
use myrtle_jobs::{JobKind, JobQueue, JobSpec, PgQueue};
use myrtle_types::{JobId, WorkerId};
use sqlx::PgPool;
use uuid::Uuid;

pub fn queue(pool: &PgPool) -> PgQueue {
    PgQueue::new(Db::from_pool(pool.clone()))
}

pub fn worker() -> WorkerId {
    WorkerId(Uuid::new_v4())
}

pub async fn claim_one(queue: &PgQueue, worker: WorkerId, kind: JobKind) -> JobId {
    let jobs = queue.claim(worker, &[kind], 1).await.unwrap();
    assert_eq!(jobs.len(), 1, "expected one claimable {kind:?}");
    jobs[0].id
}

pub async fn enqueue(queue: &PgQueue, kind: JobKind) -> JobId {
    queue.enqueue(JobSpec::new(kind)).await.unwrap()
}

pub struct Probe {
    pub state: String,
    pub attempts: i16,
    pub priority: i16,
    pub run_at: DateTime<Utc>,
    pub lease_expires_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub finished: bool,
}

pub async fn probe(pool: &PgPool, job: JobId) -> Probe {
    let (state, attempts, priority, run_at, lease_expires_at, last_error, finished) =
        sqlx::query_as(
            "SELECT state, attempts, priority, run_at, lease_expires_at, last_error,
                    finished_at IS NOT NULL
               FROM jobs WHERE id = $1",
        )
        .bind(job.0)
        .fetch_one(pool)
        .await
        .unwrap();
    Probe {
        state,
        attempts,
        priority,
        run_at,
        lease_expires_at,
        last_error,
        finished,
    }
}

pub async fn expire_lease(pool: &PgPool, job: JobId) {
    sqlx::query("UPDATE jobs SET lease_expires_at = now() - interval '1 second' WHERE id = $1")
        .bind(job.0)
        .execute(pool)
        .await
        .unwrap();
}
