#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use myrtle_app::AppContext;
use myrtle_app::config::Config;
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_db::repos::data_generations::{self, Generation};
use myrtle_jobs::{JobKind, PgQueue};
use myrtle_types::Server;
use myrtle_worker::Runtime;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

const VERSION: &str = "24-11-27-10-00-00";

#[derive(Default)]
struct Pipeline {
    connected: AtomicUsize,
    closed: AtomicUsize,
}

/// Each connection gets one `update_complete`, then is read until the client hangs up
async fn pipeline() -> (String, Arc<Pipeline>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Pipeline::default());
    let counts = Arc::clone(&seen);
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let counts = Arc::clone(&counts);
            tokio::spawn(async move {
                let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
                counts.connected.fetch_add(1, Ordering::SeqCst);
                let frame = format!(r#"{{"type":"update_complete","version":"{VERSION}"}}"#);
                ws.send(Message::text(frame)).await.unwrap();
                while let Some(Ok(_)) = ws.next().await {}
                counts.closed.fetch_add(1, Ordering::SeqCst);
            });
        }
    });
    (url, seen)
}

fn config(url: &str, background_jobs: bool) -> Config {
    let mut config = common::config();
    config.assets.ws_urls = [("en".to_owned(), url.to_owned())].into();
    config.switches.background_jobs.enabled = background_jobs;
    config
}

async fn generation(ctx: &AppContext, server: Server) -> Option<Generation> {
    data_generations::all(&mut ctx.db.conn().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .find_map(|(s, g)| (s == server).then_some(g))
}

async fn eventually(what: &str, check: impl Fn() -> bool) {
    let waited = async {
        while !check() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    tokio::time::timeout(Duration::from_secs(10), waited)
        .await
        .unwrap_or_else(|_| panic!("{what}"));
}

#[tokio::test]
async fn the_worker_listens_and_an_update_bumps_the_generation_with_its_version() {
    let (url, seen) = pipeline().await;
    let (ctx, db) = common::context(config(&url, true)).await;
    let queue = PgQueue::new(ctx.db.clone());

    let bumped = async {
        while generation(&ctx, Server::EN).await.is_none() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    let run = Runtime::new(ctx.clone(), queue).run(bumped);
    tokio::time::timeout(Duration::from_secs(20), run)
        .await
        .expect("the update is announced")
        .unwrap();

    let stored = generation(&ctx, Server::EN).await.unwrap();
    assert_eq!(stored.generation, 1);
    assert_eq!(stored.res_version.as_deref(), Some(VERSION));
    assert_eq!(seen.connected.load(Ordering::SeqCst), 1);
    eventually("the listener hangs up on shutdown", || {
        seen.closed.load(Ordering::SeqCst) == 1
    })
    .await;
    db.close().await;
}

#[tokio::test]
async fn two_workers_hearing_one_update_publish_it_once() {
    let (url, seen) = pipeline().await;
    let (first, db) = common::context(config(&url, true)).await;
    let second = AppContext {
        snapshots: Arc::new(SnapshotRegistry::new(&first.config)),
        ..first.clone()
    };
    let (seen_by, ctx) = (&seen, &first);
    let settled = move || async move {
        while seen_by.connected.load(Ordering::SeqCst) < 2
            || generation(ctx, Server::EN).await.is_none()
        {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        tokio::time::sleep(Duration::from_millis(1500)).await; // the slower worker's announce lands
    };
    let runs = futures_util::future::join(
        Runtime::new(first.clone(), PgQueue::new(first.db.clone())).run(settled()),
        Runtime::new(second.clone(), PgQueue::new(second.db.clone())).run(settled()),
    );
    let (a, b) = tokio::time::timeout(Duration::from_secs(30), runs)
        .await
        .expect("both workers hear the update");
    a.unwrap();
    b.unwrap();

    let stored = generation(&first, Server::EN).await.unwrap();
    assert_eq!(stored.generation, 1);
    assert_eq!(stored.pipeline_seq, 1);
    assert_eq!(stored.res_version.as_deref(), Some(VERSION));
    let queue = PgQueue::new(first.db.clone());
    for kind in [JobKind::ReleaseLedgerRecord, JobKind::RarityReconcile] {
        assert_eq!(common::jobs(&queue, kind).await.len(), 1, "{kind:?}");
    }
    db.close().await;
}

#[tokio::test]
async fn the_worker_does_not_listen_with_background_jobs_off() {
    let (url, seen) = pipeline().await;
    let (ctx, db) = common::context(config(&url, false)).await;
    let queue = PgQueue::new(ctx.db.clone());

    Runtime::new(ctx.clone(), queue)
        .run(tokio::time::sleep(Duration::from_millis(1500)))
        .await
        .unwrap();

    assert_eq!(seen.connected.load(Ordering::SeqCst), 0);
    assert!(generation(&ctx, Server::EN).await.is_none());
    db.close().await;
}
