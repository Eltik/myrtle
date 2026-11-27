//! `/service-accounts` reports the health the worker process stored, not the API's own memory.

#![allow(clippy::unwrap_used)]

#[path = "../../app/tests/common/mod.rs"]
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::StatusCode;
use myrtle_app::AppContext;
use myrtle_hypergryph::{
    AccountOptions, AuthSession, DeviceIds, Domain, DurableCredential, GameAccount, GameAccounts,
    NetworkConfig, SessionKey, Upstream, VersionInfo,
};
use myrtle_types::Server;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

/// u8 and the game server on one port; `refuse` makes `getToken` reject the token
async fn game_server(refuse: Arc<AtomicBool>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let refuse = Arc::clone(&refuse);
            tokio::spawn(async move {
                let mut buf = vec![0_u8; 8192];
                let n = sock.read(&mut buf).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = head.split(' ').nth(1).unwrap_or_default().to_owned();
                let body = match path.as_str() {
                    "/u8/user/v1/getToken" if refuse.load(Ordering::SeqCst) => {
                        json!({ "result": 1 })
                    }
                    "/u8/user/v1/getToken" => {
                        json!({ "result": 0, "uid": "u8-uid", "token": "u8-tok" })
                    }
                    _ => json!({ "result": 0, "uid": "u8-uid", "secret": "secret" }),
                }
                .to_string();
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = sock.write_all(reply.as_bytes()).await;
                let _ = sock.shutdown().await;
            });
        }
    });
    base
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

fn credential() -> DurableCredential {
    DurableCredential {
        yostar_uid: "yostar-uid".into(),
        yostar_token: "tok".into(),
    }
}

/// A process's own account for `server`, on the shared session store
fn account(ctx: &AppContext, server: Server) -> Arc<GameAccount> {
    GameAccount::new(
        AuthSession::from_durable("", &credential()),
        server,
        Arc::clone(&ctx.sessions),
        AccountOptions {
            secret_max_age: Duration::ZERO, // every probe re-runs the login
        },
    )
}

fn with_accounts(ctx: &AppContext, accounts: &[Arc<GameAccount>]) -> AppContext {
    let mut loaded = GameAccounts::default();
    for account in accounts {
        loaded.insert(Arc::clone(account));
    }
    AppContext {
        accounts: Arc::new(loaded),
        ..ctx.clone()
    }
}

async fn reported(api: &AppContext) -> Value {
    let app = myrtle_api::router::build(api.clone());
    let request = Request::get("/api/service-accounts")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn never_used(server: &str) -> Value {
    json!({ "server": server, "state": "unknown", "refused_since": null, "last_ok_at": null })
}

#[tokio::test]
async fn the_api_reports_what_the_worker_observed() {
    let (base, db) = common::context().await;
    let refuse = Arc::new(AtomicBool::new(false));
    let ctx = AppContext {
        upstream: Arc::new(upstream(&game_server(Arc::clone(&refuse)).await)),
        ..base
    };
    ctx.sessions
        .save_durable(
            &SessionKey::ServiceAccount { server: Server::EN },
            &credential(),
        )
        .await
        .unwrap();
    let worker = account(&ctx, Server::EN);
    let api = with_accounts(
        &ctx,
        &[account(&ctx, Server::EN), account(&ctx, Server::JP)],
    );

    assert_eq!(
        reported(&api).await,
        json!({ "accounts": [never_used("en"), never_used("jp")] })
    );

    worker.probe(&ctx.upstream).await.unwrap();
    let healthy = reported(&api).await;
    let ok_at = worker.health().last_ok_at.unwrap();
    assert_eq!(
        healthy,
        json!({ "accounts": [
            { "server": "en", "state": "ok", "refused_since": null, "last_ok_at": ok_at },
            never_used("jp")
        ] })
    );

    refuse.store(true, Ordering::SeqCst);
    worker.probe(&ctx.upstream).await.unwrap_err();
    let since = worker.health().refused_since.unwrap();
    worker.probe(&ctx.upstream).await.unwrap_err();
    assert_eq!(
        reported(&api).await,
        json!({ "accounts": [
            { "server": "en", "state": "refused", "refused_since": since, "last_ok_at": ok_at },
            never_used("jp")
        ] }),
        "an outage keeps its start"
    );

    refuse.store(false, Ordering::SeqCst);
    worker.probe(&ctx.upstream).await.unwrap();
    let recovered = worker.health().last_ok_at.unwrap();
    assert!(recovered >= ok_at);
    assert_eq!(
        reported(&api).await,
        json!({ "accounts": [
            { "server": "en", "state": "ok", "refused_since": null, "last_ok_at": recovered },
            never_used("jp")
        ] })
    );
    assert_eq!(
        api.accounts.get(Server::EN).unwrap().health().state,
        myrtle_hypergryph::AccountState::Unknown,
        "the API's own copy never moved"
    );
    db.close().await;
}

#[tokio::test]
async fn a_worker_restart_keeps_the_outage_start() {
    let (base, db) = common::context().await;
    let refuse = Arc::new(AtomicBool::new(true));
    let ctx = AppContext {
        upstream: Arc::new(upstream(&game_server(Arc::clone(&refuse)).await)),
        ..base
    };
    ctx.sessions
        .save_durable(
            &SessionKey::ServiceAccount { server: Server::EN },
            &credential(),
        )
        .await
        .unwrap();
    let api = with_accounts(&ctx, &[account(&ctx, Server::EN)]);

    let before = account(&ctx, Server::EN);
    before.probe(&ctx.upstream).await.unwrap_err();
    let since = before.health().refused_since.unwrap();
    tokio::time::sleep(Duration::from_millis(1100)).await;
    let after = account(&ctx, Server::EN);
    after.probe(&ctx.upstream).await.unwrap_err();
    assert!(after.health().refused_since.unwrap() > since);

    assert_eq!(
        reported(&api).await["accounts"][0]["refused_since"],
        json!(since)
    );
    db.close().await;
}
