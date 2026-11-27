#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::services::grading;
use myrtle_app::services::sync::user_key;
use myrtle_app::snapshots::Snapshot;
use myrtle_db::repos::lookback;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_jobs::{JobKind, JobState};
use myrtle_testkit::db::TestDb;
use myrtle_types::{Server, UserId};

const UID: u32 = 4242;

/// `data` is the one server with game data; grading reads the first configured server's
async fn setup(data: Server, user_server: Server) -> (AppContext, TestDb, UserId) {
    let mut config = common::config();
    config.server.servers = vec![Server::EN, Server::CN];
    let (ctx, db) = common::context(config).await;
    ctx.snapshots.install(
        data,
        Snapshot::from_model(GameModel::default(), SandboxUniverse::default()),
    );
    let user = db.user(UID, user_server).await;
    let uid = UID.to_string();
    ctx.cache
        .set(&user_key(&uid), "stale profile", Duration::from_secs(600))
        .await;
    (ctx, db, user)
}

async fn snapshots(ctx: &AppContext, user: UserId) -> usize {
    lookback::list(&mut ctx.db.conn().await.unwrap(), user)
        .await
        .unwrap()
        .len()
}

async fn cached(ctx: &AppContext) -> Option<String> {
    ctx.cache.get(&user_key(&UID.to_string())).await
}

#[tokio::test]
async fn a_sync_regrade_snapshots_the_period_and_drops_the_cached_profile() {
    let (ctx, db, user) = setup(Server::EN, Server::EN).await;
    grading::enqueue_sync_regrade(&ctx, user).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, JobKind::RegradeUser).await;
    assert_eq!(jobs[0].state, JobState::Succeeded);
    assert_eq!(snapshots(&ctx, user).await, 1);
    assert_eq!(cached(&ctx).await, None);
    db.close().await;
}

#[tokio::test]
async fn a_sync_regrade_that_fails_still_snapshots_and_drops_the_cached_profile() {
    let (ctx, db, user) = setup(Server::CN, Server::CN).await;
    grading::enqueue_sync_regrade(&ctx, user).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, JobKind::RegradeUser).await;
    assert_eq!((jobs[0].state, jobs[0].attempts), (JobState::Queued, 1));
    assert_eq!(snapshots(&ctx, user).await, 1);
    assert_eq!(cached(&ctx).await, None);
    db.close().await;
}

#[tokio::test]
async fn a_bulk_regrade_takes_no_snapshot_and_leaves_the_cache() {
    let (ctx, db, user) = setup(Server::EN, Server::EN).await;
    grading::enqueue_regrade(&ctx, user).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, JobKind::RegradeUser).await;
    assert_eq!(jobs[0].state, JobState::Succeeded);
    assert_eq!(snapshots(&ctx, user).await, 0);
    assert_eq!(cached(&ctx).await.as_deref(), Some("stale profile"));
    db.close().await;
}

#[tokio::test]
async fn a_queued_bulk_regrade_does_not_absorb_the_syncs() {
    let (ctx, db, user) = setup(Server::EN, Server::EN).await;
    grading::enqueue_regrade(&ctx, user).await.unwrap();
    grading::enqueue_sync_regrade(&ctx, user).await.unwrap();

    let jobs = common::run_until_attempted(&ctx, JobKind::RegradeUser).await;
    assert_eq!(jobs.len(), 2);
    assert_eq!(snapshots(&ctx, user).await, 1);
    db.close().await;
}
