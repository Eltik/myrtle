#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashSet;
use std::time::Duration;

use chrono::{SubsecRound, Utc};
use common::{claim_one, enqueue, expire_lease, probe, queue, worker};
use myrtle_db::Db;
use myrtle_jobs::{JobKind, JobQueue, JobSpec, JobState, QueueError, enqueue_in};
use myrtle_types::Server;
use serde_json::json;
use sqlx::PgPool;

const KIND: JobKind = JobKind::RegradeUser;
/// One attempt, so a single failure is final
const ONE_SHOT: JobKind = JobKind::TrendingRecompute;

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_claimed_job_carries_its_row(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = queue
        .enqueue(JobSpec {
            server: Some(Server::CN),
            payload: json!({ "uid": "42" }),
            priority: 3,
            ..JobSpec::new(KIND)
        })
        .await
        .unwrap();

    let jobs = queue.claim(me, &[KIND], 5).await.unwrap();
    assert_eq!(jobs.len(), 1);
    let job = &jobs[0];
    assert_eq!(job.id, id);
    assert_eq!(job.kind, KIND);
    assert_eq!(job.server, Some(Server::CN));
    assert_eq!(job.payload, json!({ "uid": "42" }));
    assert_eq!(job.state, JobState::Running);
    assert_eq!(job.priority, 3);
    assert_eq!(job.attempts, 1);
    assert_eq!(job.max_attempts, KIND.max_attempts());
    assert_eq!(job.lease_secs, KIND.lease_secs());
    assert_eq!(job.leased_by, Some(me));
    let lease = job.lease_expires_at.unwrap() - Utc::now();
    assert!((100..=120).contains(&lease.num_seconds()), "{lease}");
    assert!(job.finished_at.is_none() && job.last_error.is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn claims_go_by_priority_then_age(pool: PgPool) {
    let queue = queue(&pool);
    let old = enqueue(&queue, KIND).await;
    let new = enqueue(&queue, KIND).await;
    let urgent = queue
        .enqueue(JobSpec {
            priority: 10,
            ..JobSpec::new(KIND)
        })
        .await
        .unwrap();

    let me = worker();
    assert_eq!(claim_one(&queue, me, KIND).await, urgent);
    assert_eq!(claim_one(&queue, me, KIND).await, old);
    assert_eq!(claim_one(&queue, me, KIND).await, new);
    let none = queue.claim(me, &[KIND], 1).await.unwrap();
    assert!(none.is_empty());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_claim_skips_later_jobs_and_other_kinds(pool: PgPool) {
    let queue = queue(&pool);
    queue
        .enqueue(JobSpec {
            run_at: Some(Utc::now() + Duration::from_mins(10)),
            ..JobSpec::new(KIND)
        })
        .await
        .unwrap();
    enqueue(&queue, JobKind::SirenSync).await;

    let me = worker();
    let mine = queue.claim(me, &[KIND], 5).await.unwrap();
    assert!(mine.is_empty());
    let nothing_asked = queue.claim(me, &[], 5).await.unwrap();
    assert!(nothing_asked.is_empty());
    let none_wanted = queue.claim(me, &[JobKind::SirenSync], 0).await.unwrap();
    assert!(none_wanted.is_empty());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_duplicate_key_joins_the_queued_job(pool: PgPool) {
    let queue = queue(&pool);
    let keyed = |priority, delay: Duration| JobSpec {
        dedupe_key: Some("user:42".to_owned()),
        priority,
        run_at: Some(Utc::now() + delay),
        ..JobSpec::new(KIND)
    };
    let first = queue
        .enqueue(keyed(0, Duration::from_mins(10)))
        .await
        .unwrap();
    let again = queue.enqueue(keyed(5, Duration::ZERO)).await.unwrap();
    assert_eq!(first, again);

    let row = probe(&pool, first).await;
    assert_eq!(row.priority, 5);
    assert!(row.run_at <= Utc::now());

    let other_kind = queue
        .enqueue(JobSpec {
            dedupe_key: Some("user:42".to_owned()),
            ..JobSpec::new(JobKind::SandboxBackfill)
        })
        .await
        .unwrap();
    assert_ne!(other_kind, first);
    assert_ne!(enqueue(&queue, KIND).await, first); // no key, no joining
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_batch_enqueues_each_spec_as_if_alone(pool: PgPool) {
    let queue = queue(&pool);
    assert!(queue.enqueue_many(&[]).await.unwrap().is_empty());
    let keyed = |key: &str, priority, delay: Duration| JobSpec {
        dedupe_key: Some(key.to_owned()),
        payload: json!({ "key": key, "priority": priority }),
        priority,
        run_at: Some(Utc::now() + delay),
        ..JobSpec::new(KIND)
    };
    let held = queue
        .enqueue(keyed("user:1", 0, Duration::from_mins(10)))
        .await
        .unwrap();
    let later = Utc::now() + Duration::from_mins(5);
    let specs = [
        keyed("user:1", 5, Duration::ZERO),
        keyed("user:2", 1, Duration::from_mins(10)),
        JobSpec {
            server: Some(Server::JP),
            run_at: Some(later),
            ..JobSpec::new(ONE_SHOT)
        },
        keyed("user:2", 3, Duration::from_mins(20)),
        JobSpec {
            dedupe_key: Some("user:2".to_owned()),
            ..JobSpec::new(JobKind::SandboxBackfill)
        },
    ];
    let ids = queue.enqueue_many(&specs).await.unwrap();
    assert_eq!(ids.len(), specs.len());
    assert_eq!(ids[0], held, "joins the queued key");
    assert_eq!(ids[3], ids[1], "a key twice in one batch joins itself");
    assert_eq!(ids.iter().collect::<HashSet<_>>().len(), 4);

    let row = probe(&pool, held).await;
    assert_eq!((row.priority, row.state.as_str()), (5, "queued"));
    assert!(row.run_at <= Utc::now());
    let row = probe(&pool, ids[1]).await;
    assert_eq!(row.priority, 3);
    assert!(row.run_at < Utc::now() + Duration::from_mins(11));

    let (server, payload, run_at, max_attempts, lease_secs): (
        Option<i16>,
        serde_json::Value,
        _,
        i16,
        i32,
    ) = sqlx::query_as(
        "SELECT server_id, payload, run_at, max_attempts, lease_secs FROM jobs WHERE id = $1",
    )
    .bind(ids[2].0)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(server, Some(Server::JP.index()));
    assert_eq!(payload, json!({}));
    assert_eq!(
        (run_at, max_attempts, lease_secs),
        (
            later.trunc_subsecs(6),
            ONE_SHOT.max_attempts() as i16,
            ONE_SHOT.lease_secs() as i32
        )
    );
    let payload: serde_json::Value = sqlx::query_scalar("SELECT payload FROM jobs WHERE id = $1")
        .bind(ids[1].0)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        payload,
        json!({ "key": "user:2", "priority": 1 }),
        "the first payload stays"
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_running_job_no_longer_holds_its_key(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let keyed = || JobSpec {
        dedupe_key: Some("user:42".to_owned()),
        ..JobSpec::new(KIND)
    };
    let first = queue.enqueue(keyed()).await.unwrap();
    assert_eq!(claim_one(&queue, me, KIND).await, first);

    let second = queue.enqueue(keyed()).await.unwrap();
    assert_ne!(second, first);

    // The retry comes back keyless, so it cannot collide with the queued twin.
    queue
        .fail(me, first, 1, "boom", Some(Duration::ZERO))
        .await
        .unwrap();
    assert_eq!(probe(&pool, first).await.state, "queued");
    assert_eq!(queue.enqueue(keyed()).await.unwrap(), second);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_heartbeat_extends_only_the_holders_lease(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;
    expire_lease(&pool, id).await;

    let stranger = queue.heartbeat(worker(), id, 1).await;
    assert!(matches!(stranger, Err(QueueError::LeaseLost(job)) if job == id));
    assert!(probe(&pool, id).await.lease_expires_at.unwrap() < Utc::now());

    queue.heartbeat(me, id, 1).await.unwrap();
    assert!(probe(&pool, id).await.lease_expires_at.unwrap() > Utc::now());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_completed_job_keeps_its_output_and_cannot_finish_twice(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;

    queue
        .complete(me, id, 1, Some(json!({ "graded": 7 })))
        .await
        .unwrap();
    let row = probe(&pool, id).await;
    assert_eq!(row.state, "succeeded");
    assert!(row.finished && row.lease_expires_at.is_none());
    let output: serde_json::Value = sqlx::query_scalar("SELECT output FROM jobs WHERE id = $1")
        .bind(id.0)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(output, json!({ "graded": 7 }));

    assert!(matches!(
        queue.complete(me, id, 1, None).await,
        Err(QueueError::LeaseLost(_))
    ));
    assert!(matches!(
        queue.fail(me, id, 1, "late", None).await,
        Err(QueueError::LeaseLost(_))
    ));
    assert!(matches!(
        queue.heartbeat(me, id, 1).await,
        Err(QueueError::LeaseLost(_))
    ));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_failure_backs_off_a_minute_then_four(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, KIND).await;

    for (attempt, expected_secs) in [(1, 60), (2, 240)] {
        claim_one(&queue, me, KIND).await;
        queue
            .fail(me, id, attempt, "upstream 502", None)
            .await
            .unwrap();
        let row = probe(&pool, id).await;
        assert_eq!(row.state, "queued");
        assert_eq!(row.last_error.as_deref(), Some("upstream 502"));
        assert!(!row.finished && row.lease_expires_at.is_none());
        let wait = (row.run_at - Utc::now()).num_seconds();
        assert!(
            (expected_secs - 10..=expected_secs).contains(&wait),
            "{wait}"
        );
        sqlx::query("UPDATE jobs SET run_at = now() WHERE id = $1")
            .bind(id.0)
            .execute(&pool)
            .await
            .unwrap();
    }
    assert_eq!(probe(&pool, id).await.attempts, 2);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_backoff_stops_at_an_hour(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;
    sqlx::query("UPDATE jobs SET attempts = 4 WHERE id = $1") // 60 * 4^3 = 3840
        .bind(id.0)
        .execute(&pool)
        .await
        .unwrap();

    queue.fail(me, id, 4, "boom", None).await.unwrap();
    let wait = (probe(&pool, id).await.run_at - Utc::now()).num_seconds();
    assert!((3590..=3600).contains(&wait), "{wait}");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_job_dies_on_its_last_attempt(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, ONE_SHOT).await;
    claim_one(&queue, me, ONE_SHOT).await;

    queue
        .fail(me, id, 1, "boom", Some(Duration::ZERO))
        .await
        .unwrap();
    let row = probe(&pool, id).await;
    assert_eq!(row.state, "dead");
    assert!(row.finished);
    let none = queue.claim(me, &[ONE_SHOT], 1).await.unwrap();
    assert!(none.is_empty());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn reaping_requeues_an_expired_lease_and_kills_a_spent_one(pool: PgPool) {
    let queue = queue(&pool);
    let crashed = worker();
    let retryable = enqueue(&queue, KIND).await;
    let spent = enqueue(&queue, ONE_SHOT).await;
    let alive = enqueue(&queue, JobKind::SirenSync).await;
    claim_one(&queue, crashed, KIND).await;
    claim_one(&queue, crashed, ONE_SHOT).await;
    claim_one(&queue, crashed, JobKind::SirenSync).await;
    expire_lease(&pool, retryable).await;
    expire_lease(&pool, spent).await;

    assert_eq!(queue.reap_expired().await.unwrap(), 2);
    assert_eq!(queue.reap_expired().await.unwrap(), 0);

    let row = probe(&pool, retryable).await;
    assert_eq!(row.state, "queued");
    assert_eq!(row.last_error.as_deref(), Some("lease expired"));
    let row = probe(&pool, spent).await;
    assert_eq!(row.state, "dead");
    assert!(row.finished);
    assert_eq!(probe(&pool, alive).await.state, "running");

    // The crashed worker's late report loses to the next holder.
    let next = worker();
    assert_eq!(claim_one(&queue, next, KIND).await, retryable);
    assert!(matches!(
        queue.complete(crashed, retryable, 1, None).await,
        Err(QueueError::LeaseLost(_))
    ));
    queue.complete(next, retryable, 2, None).await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_stale_run_cannot_report_for_the_same_workers_next_attempt(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let id = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;
    expire_lease(&pool, id).await;
    queue.reap_expired().await.unwrap();
    claim_one(&queue, me, KIND).await;

    for stale in [
        queue.heartbeat(me, id, 1).await,
        queue.complete(me, id, 1, None).await,
        queue.fail(me, id, 1, "late", None).await,
    ] {
        assert!(matches!(stale, Err(QueueError::LeaseLost(_))));
    }
    assert_eq!(probe(&pool, id).await.state, "running");
    queue.complete(me, id, 2, None).await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn pruning_drops_only_old_finished_jobs(pool: PgPool) {
    let queue = queue(&pool);
    let me = worker();
    let old = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;
    queue.complete(me, old, 1, None).await.unwrap();
    let fresh = enqueue(&queue, KIND).await;
    claim_one(&queue, me, KIND).await;
    queue.complete(me, fresh, 1, None).await.unwrap();
    let queued = enqueue(&queue, KIND).await;
    sqlx::query("UPDATE jobs SET finished_at = now() - interval '8 days' WHERE id = $1")
        .bind(old.0)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(queue.prune(Duration::from_hours(24 * 7)).await.unwrap(), 1);
    let left: Vec<i64> = sqlx::query_scalar("SELECT id FROM jobs ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(left, [fresh.0, queued.0]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn racing_workers_never_share_a_job(pool: PgPool) {
    let queue = queue(&pool);
    for _ in 0..40 {
        enqueue(&queue, KIND).await;
    }

    let claims: Vec<_> = (0..8)
        .map(|_| {
            let queue = queue.clone();
            tokio::spawn(async move { queue.claim(worker(), &[KIND], 5).await.unwrap() })
        })
        .collect();
    let mut seen = HashSet::new();
    let mut total = 0;
    for claim in claims {
        for job in claim.await.unwrap() {
            total += 1;
            seen.insert(job.id);
        }
    }
    assert_eq!(seen.len(), total, "a job was handed out twice");

    // SKIP LOCKED may hand a racing worker fewer than it asked for; the rest stay claimable.
    let rest = queue.claim(worker(), &[KIND], 40).await.unwrap();
    assert_eq!(total + rest.len(), 40);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_job_enqueued_in_a_transaction_rolls_back_with_it(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM jobs")
            .fetch_one(&pool)
            .await
            .unwrap()
    };

    let mut tx = db.begin().await.unwrap();
    enqueue_in(&mut tx, &JobSpec::new(KIND)).await.unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(count().await, 0);

    let mut tx = db.begin().await.unwrap();
    enqueue_in(&mut tx, &JobSpec::new(KIND)).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(count().await, 1);
}
