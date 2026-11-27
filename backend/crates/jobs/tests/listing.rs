#![allow(clippy::unwrap_used)]

mod common;

use common::{claim_one, enqueue, queue, worker};
use myrtle_jobs::{JobFilter, JobKind, JobQueue, JobState};
use sqlx::PgPool;

const ALL: JobFilter = JobFilter {
    state: None,
    kind: None,
    limit: 50,
};

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn recent_lists_newest_first_within_the_limit(pool: PgPool) {
    let queue = queue(&pool);
    let first = enqueue(&queue, JobKind::RegradeUser).await;
    let second = enqueue(&queue, JobKind::SirenSync).await;
    let third = enqueue(&queue, JobKind::RegradeUser).await;

    let ids: Vec<_> = queue
        .recent(ALL)
        .await
        .unwrap()
        .iter()
        .map(|j| j.id)
        .collect();
    assert_eq!(ids, vec![third, second, first]);

    let limited = queue.recent(JobFilter { limit: 2, ..ALL }).await.unwrap();
    assert_eq!(
        limited.iter().map(|j| j.id).collect::<Vec<_>>(),
        vec![third, second]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn recent_filters_by_state_and_kind(pool: PgPool) {
    let queue = queue(&pool);
    let queued = enqueue(&queue, JobKind::RegradeUser).await;
    let other_kind = enqueue(&queue, JobKind::SirenSync).await;
    let me = worker();
    enqueue(&queue, JobKind::TrendingRecompute).await;
    let failed = claim_one(&queue, me, JobKind::TrendingRecompute).await;
    queue.fail(me, failed, 1, "boom", None).await.unwrap();

    let only = |state, kind| JobFilter { state, kind, ..ALL };
    let dead = queue
        .recent(only(Some(JobState::Dead), None))
        .await
        .unwrap();
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].id, failed);
    assert_eq!(dead[0].attempts, 1);
    assert_eq!(dead[0].last_error.as_deref(), Some("boom"));

    let regrades = queue
        .recent(only(None, Some(JobKind::RegradeUser)))
        .await
        .unwrap();
    assert_eq!(
        regrades.iter().map(|j| j.id).collect::<Vec<_>>(),
        vec![queued]
    );

    let both = only(Some(JobState::Queued), Some(JobKind::SirenSync));
    let siren = queue.recent(both).await.unwrap();
    assert_eq!(
        siren.iter().map(|j| j.id).collect::<Vec<_>>(),
        vec![other_kind]
    );

    let none = only(Some(JobState::Running), None);
    assert!(queue.recent(none).await.unwrap().is_empty());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn counts_cover_every_state_and_follow_the_kind(pool: PgPool) {
    let queue = queue(&pool);
    assert_eq!(
        queue.counts(None).await.unwrap(),
        JobState::ALL.map(|s| (s, 0)).to_vec()
    );

    enqueue(&queue, JobKind::RegradeUser).await;
    enqueue(&queue, JobKind::SirenSync).await;
    let me = worker();
    claim_one(&queue, me, JobKind::SirenSync).await;
    enqueue(&queue, JobKind::SirenSync).await;
    enqueue(&queue, JobKind::RegradeAll).await;
    let done = claim_one(&queue, me, JobKind::RegradeAll).await;
    queue.complete(me, done, 1, None).await.unwrap();

    let counts = queue.counts(None).await.unwrap();
    assert_eq!(
        counts,
        vec![
            (JobState::Queued, 2),
            (JobState::Running, 1),
            (JobState::Succeeded, 1),
            (JobState::Dead, 0),
        ]
    );
    let siren = queue.counts(Some(JobKind::SirenSync)).await.unwrap();
    assert_eq!(siren[0], (JobState::Queued, 1));
    assert_eq!(siren[1], (JobState::Running, 1));
    assert_eq!(siren[2].1 + siren[3].1, 0);
}
