//! A refused service account queues one Discord alert per outage, from the jobs that use it.

#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::alerts::service_account_refused_embed;
use myrtle_app::services::gacha::refresh_pool_details;
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_hypergryph::{
    AccountOptions, AuthSession, DeviceIds, Domain, DurableCredential, GameAccount, GameAccounts,
    NetworkConfig, SessionKey, SessionStore, Upstream, VersionInfo,
};
use myrtle_jobs::{Job, JobFilter, JobKind, JobState, PgQueue};
use myrtle_testkit::sessions::MemorySessions;
use myrtle_types::{Server, WorkerId};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const ACCEPT: u8 = 0;
const REFUSE: u8 = 1;
const SLOW_TOKEN: u8 = 2;
const LOGIN_DOWN: u8 = 3;
const GAME_DOWN: u8 = 4;

/// Yostar and the game server on one port; `mode` picks how `getToken` and game routes answer
async fn game_server(mode: Arc<AtomicU8>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let mode = Arc::clone(&mode);
            tokio::spawn(async move {
                let Some(path) = read_path(&mut sock).await else {
                    return;
                };
                let mode = mode.load(Ordering::SeqCst);
                let (status, body) = match path.as_str() {
                    "/u8/user/v1/getToken" if mode == REFUSE => (200, json!({ "result": 1 })),
                    "/u8/user/v1/getToken" => {
                        if mode == SLOW_TOKEN {
                            tokio::time::sleep(Duration::from_secs(2)).await;
                        }
                        (
                            200,
                            json!({ "result": 0, "uid": "u8-uid", "token": "u8-tok" }),
                        )
                    }
                    "/gs/account/login" if mode == LOGIN_DOWN => (503, json!({})),
                    "/gs/account/login" => (
                        200,
                        json!({ "result": 0, "uid": "u8-uid", "secret": "secret" }),
                    ),
                    _ if mode == GAME_DOWN => (503, json!({})),
                    _ => (200, json!("not a pool detail")), // nothing fetched, so no sidecar write or reload
                };
                let body = body.to_string();
                let reply = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(reply.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    base
}

async fn read_path(sock: &mut tokio::net::TcpStream) -> Option<String> {
    let mut buf = Vec::new();
    let mut chunk = [0_u8; 4096];
    let head_end = loop {
        let n = sock.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let len = head
        .lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
        .unwrap_or(0);
    while buf.len() < head_end + len {
        let n = sock.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    head.split(' ').nth(1).map(str::to_owned)
}

fn upstream(base: &str) -> Upstream {
    let mut config = NetworkConfig::new(DeviceIds::default());
    config.set_domain(Server::EN, Domain::GS, format!("{base}/gs"));
    config.set_domain(Server::EN, Domain::U8, format!("{base}/u8"));
    config.set_version(
        Server::EN,
        VersionInfo {
            res_version: "res-1".into(),
            client_version: "client-1".into(),
        },
    );
    Upstream::new(
        reqwest::Client::builder().no_proxy().build().unwrap(),
        config,
    )
    .with_request_timeout(Duration::from_millis(500))
}

/// One worker process: its own account, so its own outage flag, on the shared queue
fn worker(ctx: &AppContext) -> AppContext {
    let credential = DurableCredential {
        yostar_uid: "yostar-uid".into(),
        yostar_token: "tok".into(),
    };
    let store = Arc::new(MemorySessions::default());
    store.put_durable(
        &SessionKey::ServiceAccount { server: Server::EN },
        credential.clone(),
    );
    let store: Arc<dyn SessionStore> = store;
    let options = AccountOptions {
        secret_max_age: Duration::ZERO, // every request re-runs the login, so it sees the token's state
    };
    let mut accounts = GameAccounts::default();
    accounts.insert(GameAccount::new(
        AuthSession::from_durable("", &credential),
        Server::EN,
        store,
        options,
    ));
    AppContext {
        accounts: Arc::new(accounts),
        ..ctx.clone()
    }
}

async fn setup(dir: &std::path::Path) -> (AppContext, myrtle_testkit::db::TestDb, Arc<AtomicU8>) {
    let (mut ctx, db) = common::context().await;
    let mode = Arc::new(AtomicU8::new(ACCEPT));
    let base = game_server(Arc::clone(&mode)).await;
    let mut config = (*ctx.config).clone();
    config.assets.dir = dir.to_string_lossy().into_owned();
    config.jobs.gacha_detail.call_delay_ms = 0;
    ctx.snapshots = Arc::new(SnapshotRegistry::new(&config));
    ctx.config = Arc::new(config);
    ctx.upstream = Arc::new(upstream(&base));
    let mut model = GameModel::default();
    model.gacha.gacha_pool_client = vec![GachaPoolClient {
        gacha_pool_id: "POOL_1".into(),
        end_time: i64::MAX,
        ..GachaPoolClient::default()
    }];
    ctx.snapshots.install(
        Server::EN,
        Snapshot::from_model(model, SandboxUniverse::default()),
    );
    (worker(&ctx), db, mode)
}

async fn alerts(ctx: &AppContext) -> Vec<Job> {
    let filter = JobFilter {
        state: None,
        kind: Some(JobKind::ServiceAccountAlert),
        limit: 100,
    };
    PgQueue::new(ctx.db.clone()).recent(filter).await.unwrap()
}

/// Runs every queued alert, so the queue's own dedupe can no longer fold a later one in
async fn deliver(ctx: &AppContext) {
    let worker = WorkerId(uuid::Uuid::new_v4());
    for job in ctx
        .queue
        .claim(worker, &[JobKind::ServiceAccountAlert], 10)
        .await
        .unwrap()
    {
        ctx.queue
            .complete(worker, job.id, job.attempts, None)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn a_refusal_queues_one_alert_naming_the_server_and_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db, mode) = setup(dir.path()).await;
    mode.store(REFUSE, Ordering::SeqCst);

    assert_eq!(
        refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap(),
        0
    );

    let queued = alerts(&ctx).await;
    assert_eq!(queued.len(), 1, "{queued:?}");
    let alert = &queued[0];
    assert_eq!(alert.server, Some(Server::EN));
    assert_eq!(alert.state, JobState::Queued);
    let reason = alert.payload["error"].as_str().unwrap();
    assert!(!reason.is_empty(), "{:?}", alert.payload);
    db.close().await;
}

#[tokio::test]
async fn a_second_refusal_in_the_same_outage_queues_none() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db, mode) = setup(dir.path()).await;
    mode.store(REFUSE, Ordering::SeqCst);

    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    deliver(&ctx).await;
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();

    let sent = alerts(&ctx).await;
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].state, JobState::Succeeded);
    db.close().await;
}

#[tokio::test]
async fn refusals_from_two_workers_queue_one_alert() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db, mode) = setup(dir.path()).await;
    let other = worker(&ctx);
    mode.store(REFUSE, Ordering::SeqCst);

    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    refresh_pool_details(&other, Some(Server::EN))
        .await
        .unwrap();

    assert_eq!(alerts(&ctx).await.len(), 1);
    db.close().await;
}

#[tokio::test]
async fn recovery_then_a_new_refusal_queues_another() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db, mode) = setup(dir.path()).await;
    mode.store(REFUSE, Ordering::SeqCst);
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    deliver(&ctx).await;

    mode.store(ACCEPT, Ordering::SeqCst);
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    assert_eq!(
        alerts(&ctx).await.len(),
        1,
        "a working login alerts nothing"
    );

    mode.store(REFUSE, Ordering::SeqCst);
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    let all = alerts(&ctx).await;
    assert_eq!(all.len(), 2, "{all:?}");
    assert_eq!(
        all.iter().filter(|j| j.state == JobState::Queued).count(),
        1
    );
    db.close().await;
}

#[tokio::test]
async fn an_ordinary_upstream_failure_queues_none() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db, mode) = setup(dir.path()).await;

    for failure in [SLOW_TOKEN, LOGIN_DOWN, GAME_DOWN] {
        mode.store(failure, Ordering::SeqCst);
        refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    }
    assert!(alerts(&ctx).await.is_empty());

    mode.store(REFUSE, Ordering::SeqCst);
    refresh_pool_details(&ctx, Some(Server::EN)).await.unwrap();
    assert_eq!(
        alerts(&ctx).await.len(),
        1,
        "the outage starts at the refusal"
    );
    db.close().await;
}

#[test]
fn the_embed_says_how_to_log_in_again_per_server() {
    let embed = service_account_refused_embed("en", "code `1`");
    assert_eq!(embed["title"], "Service account login refused (en)");
    let description = embed["description"].as_str().unwrap();
    assert!(
        description.contains("myrtle session send-code --server en --email <addr>"),
        "{description}"
    );
    assert!(
        description.ends_with("Upstream said: `code '1'`"),
        "{description}"
    );

    let bili: Value = service_account_refused_embed("bili", "x");
    let description = bili["description"].as_str().unwrap();
    assert!(description.contains("myrtle session sms --phone <phone> --otp <code>"));
    assert!(!description.contains("--email"));
}
