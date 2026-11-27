#![allow(clippy::unwrap_used)]

mod common;

use myrtle_jobs::{JobKind, JobSpec, JobState};
use serde_json::json;

#[tokio::test]
async fn a_claimed_job_runs_and_records_its_summary() {
    let (ctx, db) = common::context(common::config()).await;
    let kind = JobKind::MusicStatDedupeSweep;
    ctx.queue.enqueue(JobSpec::new(kind)).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, kind).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!((jobs[0].state, jobs[0].attempts), (JobState::Succeeded, 1));
    assert!(jobs[0].leased_by.is_none());
    let output: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT output FROM jobs WHERE id = $1")
            .bind(jobs[0].id.0)
            .fetch_one(&mut *ctx.db.conn().await.unwrap())
            .await
            .unwrap();
    assert_eq!(output, Some(json!("0 music stat dedupe rows swept")));
    db.close().await;
}

#[tokio::test]
async fn a_service_account_alert_with_no_webhook_succeeds() {
    let (ctx, db) = common::context(common::config()).await;
    let kind = JobKind::ServiceAccountAlert;
    let alert = JobSpec {
        server: Some(myrtle_types::Server::EN),
        payload: json!({ "error": "u8 getToken rejected" }),
        ..JobSpec::new(kind)
    };
    ctx.queue.enqueue(alert).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, kind).await;
    assert_eq!((jobs[0].state, jobs[0].attempts), (JobState::Succeeded, 1));
    db.close().await;
}

#[tokio::test]
async fn a_failed_job_requeues_with_its_error() {
    let (ctx, db) = common::context(common::config()).await;
    let kind = JobKind::RegradeUser;
    let bad = JobSpec {
        payload: json!({ "user_id": "not a uuid" }),
        ..JobSpec::new(kind)
    };
    ctx.queue.enqueue(bad).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, kind).await;
    assert_eq!((jobs[0].state, jobs[0].attempts), (JobState::Queued, 1));
    let error = jobs[0].last_error.as_deref().unwrap();
    assert!(error.starts_with("bad payload: "), "{error}");
    assert!(jobs[0].run_at > chrono::Utc::now(), "the retry backs off");
    db.close().await;
}

#[tokio::test]
async fn a_fan_out_runs_every_job_past_its_lane_width() {
    let mut config = common::config();
    config.jobs.regrade.concurrency = 2;
    let (ctx, db) = common::context(config).await;
    let kind = JobKind::RegradeUser;
    for n in 1..=7 {
        let user = db.user(n, myrtle_types::Server::EN).await;
        let spec = JobSpec {
            payload: json!({ "user_id": user.0 }),
            ..JobSpec::new(kind)
        };
        ctx.queue.enqueue(spec).await.unwrap();
    }

    let jobs = common::run_until_attempted(&ctx, kind).await;
    assert_eq!(jobs.len(), 7);
    assert!(jobs.iter().all(|j| j.attempts == 1)); // no game data in this context, so each fails once
    db.close().await;
}
