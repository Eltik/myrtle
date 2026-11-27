//! Call sites under `Upstream::with_request_timeout` honour it; `syncData` and the account portal keep their own.

mod common;

use std::time::{Duration, Instant};

use common::{Recorded, Reply, serve, upstream};
use myrtle_hypergryph::fetch::auth_request;
use myrtle_hypergryph::login::{bilibili, passport, yostar};
use myrtle_hypergryph::session::refresh_secret;
use myrtle_hypergryph::{
    AccountOptions, AuthSession, GameAccount, SessionKey, SessionStore, Upstream, UpstreamError,
};
use myrtle_types::Server;
use serde_json::json;
use std::sync::Arc;

const SHORT: Duration = Duration::from_millis(200);
const PROMPT: Duration = Duration::from_secs(2);

async fn stalled() -> common::FakeServer {
    serve(|_: &Recorded| Reply::json(&json!({})).after(Duration::from_secs(4))).await
}

fn short(server: &common::FakeServer) -> Upstream {
    upstream(&server.base).with_request_timeout(SHORT)
}

fn assert_timeout<T>(started: Instant, result: Result<T, UpstreamError>) {
    assert!(started.elapsed() < PROMPT, "took {:?}", started.elapsed());
    assert!(
        matches!(result, Err(UpstreamError::Timeout(_))),
        "not a timeout"
    );
}

#[tokio::test]
async fn auth_request_uses_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let mut session = AuthSession::new(Some("u"), Some("s"), Some(1), None);
    let started = Instant::now();
    let result = auth_request(&up, "x", None, &mut session, Server::EN).await;
    assert_timeout(started, result.map(drop));
}

#[tokio::test]
async fn bilibili_posts_use_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let started = Instant::now();
    let result = bilibili::login(&up, "doc", "pw").await;
    assert_timeout(started, result.map(drop));
}

#[tokio::test]
async fn passport_posts_use_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let started = Instant::now();
    let result = passport::send_phone_code(&up, "1").await;
    assert_timeout(started, result);
}

#[tokio::test]
async fn reload_network_uses_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let started = Instant::now();
    assert_eq!(up.reload_network().await, 0);
    assert!(started.elapsed() < PROMPT, "took {:?}", started.elapsed());
}

#[tokio::test]
async fn reload_versions_uses_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let started = Instant::now();
    assert_eq!(up.reload_versions().await, 0);
    assert!(started.elapsed() < PROMPT, "took {:?}", started.elapsed());
    assert_eq!(up.config().version(Server::EN), &common::version());
}

#[tokio::test]
async fn yostar_sdk_posts_use_the_handle_timeout() {
    let server = stalled().await;
    let up = short(&server);
    let started = Instant::now();
    let result = yostar::send_code(&up, "a@b", Server::EN).await;
    assert_timeout(started, result);
}

#[tokio::test]
async fn account_login_uses_the_handle_timeout() {
    let server = serve(|req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" })),
        _ => Reply::json(&json!({})).after(Duration::from_secs(4)),
    })
    .await;
    let up = short(&server);
    let mut session = common::session_with(&common::durable("y", "tok"));
    let started = Instant::now();
    let result = refresh_secret(&up, &mut session, Server::EN).await;
    assert_timeout(started, result);
    assert!(!server.to("/gs/account/login").is_empty());
}

#[tokio::test]
async fn sync_data_and_the_portal_outlast_the_handle_timeout() {
    let slow = Duration::from_millis(400);
    let server = serve(move |req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" })),
        "/gs/account/login" => Reply::json(&json!({ "result": 0, "secret": "s" })),
        "/portal/api/user/login" => Reply::json(&json!({}))
            .with_header("Set-Cookie", "YSSID=abc; Path=/")
            .with_header("Set-Cookie", "YSSID.sig=def; Path=/")
            .after(slow),
        _ => Reply::json(&json!({})).body_after(slow),
    })
    .await;
    let up = short(&server);

    let store = Arc::new(myrtle_testkit::sessions::MemorySessions::default());
    let key = SessionKey::ServiceAccount { server: Server::EN };
    store.put_durable(&key, common::durable("y", "tok"));
    let dyn_store: Arc<dyn SessionStore> = store;
    let acct = GameAccount::load(Server::EN, dyn_store, AccountOptions::default())
        .await
        .unwrap()
        .unwrap();
    acct.sync_player(&up).await.unwrap();

    let portal = yostar::account_portal_login(&up, "a@b", "t").await.unwrap();
    assert_eq!(portal.yssid, "abc");
}
