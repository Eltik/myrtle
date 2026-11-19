#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashSet;
use std::time::Duration;

use chrono::{DateTime, Utc};
use common::{queue, worker};
use myrtle_db::{Db, DbError};
use myrtle_jobs::schedule::{self, DEFAULTS, Schedule, next_run, offset_for};
use myrtle_jobs::{JobKind, JobQueue, QueueError};
use myrtle_types::Server;
use sqlx::PgPool;

const STAGGER: Duration = Duration::from_mins(3);

const fn at(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap()
}

#[test]
fn the_next_run_is_the_first_grid_point_after_now() {
    let every = Duration::from_mins(15);
    let offset = Duration::from_secs(40);
    assert_eq!(next_run(every, offset, at(1_000)), at(1_840));
    assert_eq!(next_run(every, offset, at(1_839)), at(1_840));
    assert_eq!(next_run(every, offset, at(1_840)), at(2_740)); // strictly after
    assert_eq!(next_run(every, Duration::ZERO, at(0)), at(900));
    assert_eq!(next_run(every, offset, at(10)), at(40));
}

#[test]
fn an_offset_longer_than_the_period_wraps_and_a_zero_period_still_advances() {
    let every = Duration::from_secs(60);
    assert_eq!(next_run(every, Duration::from_secs(130), at(0)), at(10));
    assert_eq!(next_run(Duration::ZERO, Duration::ZERO, at(500)), at(501));
}

#[test]
fn offsets_are_stable_and_inside_the_window() {
    for kind in JobKind::ALL {
        let offset = offset_for(kind, STAGGER);
        assert!(offset < STAGGER, "{kind:?} at {offset:?}");
        assert_eq!(offset, offset_for(kind, STAGGER));
        assert_eq!(offset_for(kind, Duration::ZERO), Duration::ZERO);
    }
}

#[test]
fn the_scheduled_kinds_do_not_land_together() {
    let offsets: HashSet<Duration> = DEFAULTS
        .iter()
        .map(|s| offset_for(s.kind, STAGGER))
        .collect();
    assert!(offsets.len() >= DEFAULTS.len() - 1, "{offsets:?}");
}

#[test]
fn the_defaults_name_each_kind_once() {
    let kinds: HashSet<JobKind> = DEFAULTS.iter().map(|s| s.kind).collect();
    assert_eq!(kinds.len(), DEFAULTS.len());
    let off: Vec<JobKind> = DEFAULTS
        .iter()
        .filter(|s| !s.enabled)
        .map(|s| s.kind)
        .collect();
    assert_eq!(off, [JobKind::DpsUpstreamWatch]);
}

async fn make_due(pool: &PgPool) {
    sqlx::query("UPDATE job_schedules SET next_run_at = now() - interval '1 second'")
        .execute(pool)
        .await
        .unwrap();
}

async fn queued(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM jobs WHERE state = 'queued'")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_new_schedule_waits_out_its_stagger_offset(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let kind = DEFAULTS
        .iter()
        .map(|s| s.kind)
        .max_by_key(|k| offset_for(*k, STAGGER))
        .unwrap();
    let offset = offset_for(kind, STAGGER).as_secs() as i64;
    assert!(offset > 5, "no scheduled kind has a visible offset");
    schedule::sync(
        &mut db.conn().await.unwrap(),
        &[Schedule::new(kind, Duration::from_hours(24))],
        STAGGER,
    )
    .await
    .unwrap();

    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), 0);
    let (every, stored, wait): (i32, i32, f64) = sqlx::query_as(
        "SELECT every_secs, offset_secs, extract(epoch FROM next_run_at - now())::float8
           FROM job_schedules",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(every, 86_400);
    assert_eq!(i64::from(stored), offset);
    assert!((wait - offset as f64).abs() < 5.0, "{wait}");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_due_schedule_enqueues_once_and_moves_to_its_grid(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    schedule::sync(&mut db.conn().await.unwrap(), &DEFAULTS, STAGGER)
        .await
        .unwrap();
    make_due(&pool).await;

    let enabled = DEFAULTS.iter().filter(|s| s.enabled).count() as u64;
    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), enabled);
    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), 0);
    assert_eq!(queued(&pool).await as u64, enabled);

    let rows: Vec<(String, i32, i32, DateTime<Utc>, bool)> = sqlx::query_as(
        "SELECT kind, every_secs, offset_secs, next_run_at, last_enqueued_at IS NOT NULL
           FROM job_schedules WHERE enabled",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    for (kind, every, offset, next, stamped) in rows {
        assert!(stamped, "{kind}");
        assert!(next > Utc::now(), "{kind}");
        assert_eq!(
            (next.timestamp() - i64::from(offset)) % i64::from(every),
            0,
            "{kind} is off its grid"
        );
    }

    let unfired: bool = sqlx::query_scalar(
        "SELECT last_enqueued_at IS NULL FROM job_schedules WHERE kind = 'dps_upstream_watch'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(unfired, "a disabled schedule fired");
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn racing_ticks_fire_each_schedule_once(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    schedule::sync(&mut db.conn().await.unwrap(), &DEFAULTS, STAGGER)
        .await
        .unwrap();
    make_due(&pool).await;

    let ticks: Vec<_> = (0..6)
        .map(|_| {
            let db = db.clone();
            tokio::spawn(async move { schedule::enqueue_due(&db).await.unwrap() })
        })
        .collect();
    let mut fired = 0;
    for tick in ticks {
        fired += tick.await.unwrap();
    }
    let enabled = DEFAULTS.iter().filter(|s| s.enabled).count() as u64;
    assert_eq!(fired, enabled);
    assert_eq!(queued(&pool).await as u64, enabled);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_run_still_queued_absorbs_the_next_tick(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let queue = queue(&pool);
    let kind = JobKind::TrendingRecompute;
    schedule::sync(
        &mut db.conn().await.unwrap(),
        &[Schedule::new(kind, Duration::from_mins(15))],
        Duration::ZERO,
    )
    .await
    .unwrap();

    schedule::enqueue_due(&db).await.unwrap();
    make_due(&pool).await;
    schedule::enqueue_due(&db).await.unwrap();
    assert_eq!(queued(&pool).await, 1);

    // Once the run is claimed, the next tick queues behind it.
    let jobs = queue.claim(worker(), &[kind], 1).await.unwrap();
    assert_eq!(jobs.len(), 1);
    make_due(&pool).await;
    schedule::enqueue_due(&db).await.unwrap();
    assert_eq!(queued(&pool).await, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_server_schedule_enqueues_for_its_server(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let queue = queue(&pool);
    let kind = JobKind::StoryCommunityAggregate;
    let per_server = [Server::EN, Server::CN].map(|server| Schedule {
        server: Some(server),
        ..Schedule::new(kind, Duration::from_hours(6))
    });
    schedule::sync(&mut db.conn().await.unwrap(), &per_server, Duration::ZERO)
        .await
        .unwrap();

    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), 2);
    let jobs = queue.claim(worker(), &[kind], 5).await.unwrap();
    let servers: HashSet<Option<Server>> = jobs.iter().map(|j| j.server).collect();
    assert_eq!(servers, HashSet::from([Some(Server::EN), Some(Server::CN)]));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn syncing_again_changes_the_cadence_and_keeps_the_clock(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let kind = JobKind::SirenSync;
    let mut conn = db.conn().await.unwrap();
    schedule::sync(
        &mut conn,
        &[Schedule::new(kind, Duration::from_hours(6))],
        STAGGER,
    )
    .await
    .unwrap();
    let before: DateTime<Utc> = sqlx::query_scalar("SELECT next_run_at FROM job_schedules")
        .fetch_one(&pool)
        .await
        .unwrap();

    let off = Schedule {
        enabled: false,
        ..Schedule::new(kind, Duration::from_hours(1))
    };
    schedule::sync(&mut conn, &[off], STAGGER).await.unwrap();
    let (every, enabled, after): (i32, bool, DateTime<Utc>) =
        sqlx::query_as("SELECT every_secs, enabled, next_run_at FROM job_schedules")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((every, enabled, after), (3_600, false, before));

    make_due(&pool).await;
    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_schedule_of_an_unknown_kind_is_left_alone(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    sqlx::query(
        "INSERT INTO job_schedules (kind, every_secs, next_run_at)
         VALUES ('from_a_newer_build', 60, now() - interval '1 minute')",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(schedule::enqueue_due(&db).await.unwrap(), 0);
    assert_eq!(queued(&pool).await, 0);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_schedule_remembers_what_its_last_run_left(pool: PgPool) {
    let db = Db::from_pool(pool.clone());
    let mut conn = db.conn().await.unwrap();
    let kind = JobKind::DpsUpstreamWatch;
    schedule::sync(&mut conn, &DEFAULTS, STAGGER).await.unwrap();
    assert_eq!(
        schedule::memory(&mut conn, kind, None).await.unwrap(),
        serde_json::json!({})
    );

    let seen = serde_json::json!({ "commit": "abc123" });
    schedule::remember(&mut conn, kind, None, &seen)
        .await
        .unwrap();
    schedule::sync(&mut conn, &DEFAULTS, STAGGER).await.unwrap();
    assert_eq!(schedule::memory(&mut conn, kind, None).await.unwrap(), seen);
    assert_eq!(
        schedule::memory(&mut conn, JobKind::RegradeAll, None)
            .await
            .unwrap(),
        serde_json::json!({})
    );

    let unscheduled = schedule::remember(&mut conn, kind, Some(Server::EN), &seen).await;
    assert!(matches!(
        unscheduled,
        Err(QueueError::Db(DbError::NotFound))
    ));
}
