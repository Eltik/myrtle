#![allow(clippy::unwrap_used)]

mod common;

use std::sync::atomic::AtomicBool;

use myrtle_app::AppContext;
use myrtle_app::services::grading::{RegradeMode, RegradePass, RegradeTally, regrade_pass};
use myrtle_db::repos::scores;
use myrtle_types::{Server, UserId};

fn pass(ctx: &AppContext) -> RegradePass {
    RegradePass {
        concurrency: 2,
        page_size: 2,
        ..RegradePass::from_config(ctx)
    }
}

async fn run(ctx: &AppContext, pass: &RegradePass) -> RegradeTally {
    regrade_pass(ctx, pass, &AtomicBool::new(false))
        .await
        .unwrap()
}

async fn score_rows(ctx: &AppContext, users: &[UserId]) -> usize {
    let mut conn = ctx.db.conn().await.unwrap();
    let mut n = 0;
    for user in users {
        n += usize::from(scores::get(&mut conn, *user).await.unwrap().is_some());
    }
    n
}

#[tokio::test]
async fn the_pass_pages_every_user_and_counts_each_failure() {
    let (ctx, db) = common::context().await;
    for n in 1..=5 {
        db.user(n, Server::EN).await;
    }
    db.user(6, Server::JP).await;

    let all = run(&ctx, &pass(&ctx)).await;
    assert_eq!(
        all,
        RegradeTally {
            targets: 6,
            regraded: 0,
            failed: 6,
            stopped_early: false,
        }
    );
    let jp = run(
        &ctx,
        &RegradePass {
            server: Some(Server::JP),
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!((jp.targets, jp.failed), (1, 1));
    let one = run(
        &ctx,
        &RegradePass {
            uid: Some("3".into()),
            mode: RegradeMode::Sandbox,
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!((one.targets, one.failed), (1, 1));
    let nobody = run(
        &ctx,
        &RegradePass {
            uid: Some("3".into()),
            server: Some(Server::JP),
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!(nobody, RegradeTally::default());
    db.close().await;
}

#[tokio::test]
async fn fail_fast_and_stop_dispatch_nothing_more() {
    let (ctx, db) = common::context().await;
    for n in 1..=4 {
        db.user(n, Server::EN).await;
    }
    let fast = run(
        &ctx,
        &RegradePass {
            concurrency: 1,
            fail_fast: true,
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!((fast.failed, fast.stopped_early), (1, true));

    let stopped = regrade_pass(&ctx, &pass(&ctx), &AtomicBool::new(true))
        .await
        .unwrap();
    assert_eq!(
        (stopped.targets, stopped.failed, stopped.stopped_early),
        (4, 0, true)
    );
    db.close().await;
}

#[tokio::test]
async fn a_dry_run_grades_without_writing() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let mut users = Vec::new();
    for n in 1..=3 {
        users.push(db.user(n, Server::EN).await);
    }
    let dry = run(
        &ctx,
        &RegradePass {
            dry_run: true,
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!((dry.regraded, dry.failed), (3, 0));
    assert_eq!(score_rows(&ctx, &users).await, 0);

    let sandbox_dry = run(
        &ctx,
        &RegradePass {
            dry_run: true,
            mode: RegradeMode::Sandbox,
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!(sandbox_dry.regraded, 3);
    assert_eq!(score_rows(&ctx, &users).await, 0);

    let full = run(&ctx, &pass(&ctx)).await;
    assert_eq!((full.regraded, full.failed), (3, 0));
    assert_eq!(score_rows(&ctx, &users).await, 3);
    let sandbox = run(
        &ctx,
        &RegradePass {
            mode: RegradeMode::Sandbox,
            ..pass(&ctx)
        },
    )
    .await;
    assert_eq!(sandbox.regraded, 3);
    db.close().await;
}
