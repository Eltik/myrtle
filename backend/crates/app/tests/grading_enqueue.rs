#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;

use myrtle_app::AppContext;
use myrtle_app::services::grading::{enqueue_regrade, enqueue_regrade_all};
use myrtle_jobs::{JobKind, JobSpec};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};

type Row = (
    String,
    Option<i16>,
    Value,
    Option<String>,
    i16,
    i16,
    i32,
    String,
);

async fn queued(ctx: &AppContext) -> Vec<Row> {
    sqlx::query_as(
        "SELECT kind, server_id, payload, dedupe_key, priority, max_attempts, lease_secs, state
           FROM jobs ORDER BY payload ->> 'user_id', id",
    )
    .fetch_all(&mut *ctx.db.conn().await.unwrap())
    .await
    .unwrap()
}

async fn reset(ctx: &AppContext, urgent: UserId) {
    sqlx::query("DELETE FROM jobs")
        .execute(&mut *ctx.db.conn().await.unwrap())
        .await
        .unwrap();
    ctx.queue
        .enqueue(JobSpec {
            payload: json!({ "user_id": urgent.0 }),
            dedupe_key: Some(urgent.0.to_string()),
            priority: 5,
            ..JobSpec::new(JobKind::RegradeUser)
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn regrade_all_queues_the_rows_one_enqueue_per_user_would() {
    let (base, db) = common::context().await;
    let mut config = (*base.config).clone();
    config.jobs.regrade.page_size = 3;
    let ctx = AppContext {
        config: Arc::new(config),
        ..base.clone()
    };
    let mut users = Vec::new();
    for n in 1..=7 {
        users.push(db.user(n, Server::EN).await);
    }

    reset(&ctx, users[2]).await;
    for user in &users {
        enqueue_regrade(&ctx, *user).await.unwrap();
    }
    let looped = queued(&ctx).await;

    reset(&ctx, users[2]).await;
    assert_eq!(enqueue_regrade_all(&ctx).await.unwrap(), 7);
    let batched = queued(&ctx).await;
    assert_eq!(batched, looped);

    assert_eq!(batched.len(), 7, "the queued key joined, not doubled");
    let mut expected: Vec<String> = users.iter().map(|u| u.0.to_string()).collect();
    expected.sort_unstable();
    for (row, user) in batched.iter().zip(&expected) {
        assert_eq!(row.0, "regrade_user");
        assert_eq!(row.1, None);
        assert_eq!(row.2, json!({ "user_id": user }));
        assert_eq!(row.3.as_deref(), Some(user.as_str()));
        assert_eq!(
            row.4,
            if *user == users[2].0.to_string() {
                5
            } else {
                0
            }
        );
        assert_eq!(row.7, "queued");
    }

    assert_eq!(enqueue_regrade_all(&ctx).await.unwrap(), 7);
    assert_eq!(queued(&ctx).await, batched, "a second pass joins every key");
    db.close().await;
}
