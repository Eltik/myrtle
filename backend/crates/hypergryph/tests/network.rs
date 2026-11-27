#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use common::{Recorded, Reply, serve};
use myrtle_hypergryph::config::network::parse_network;
use myrtle_hypergryph::{Domain, NetworkConfig, Upstream, UpstreamError};
use myrtle_types::Server;
use serde_json::json;

const FAST: Duration = Duration::from_millis(10);

fn response(content: &str) -> String {
    json!({ "sign": "x", "content": content }).to_string()
}

fn routes(base: &str, code: &str) -> String {
    let content = json!({
        "funcVer": "V1",
        "configs": { "V1": { "network": {
            "gs": format!("{base}/gs"),
            "hv": format!("{base}/v/{code}/{{0}}/version"),
        } } },
    });
    response(&content.to_string())
}

fn unloaded(base: &str) -> NetworkConfig {
    let mut cfg = NetworkConfig::new(common::device_ids());
    for server in Server::ALL {
        cfg.hosts.network_routes[server.index() as usize] =
            format!("{base}/conf/{}", server.as_str());
    }
    cfg
}

fn fast(cfg: NetworkConfig) -> Arc<Upstream> {
    Arc::new(Upstream::new(common::client(), cfg).with_network_retry(FAST, FAST * 4))
}

fn conf_requests(server: &common::FakeServer, code: &str) -> usize {
    server.to(&format!("/conf/{code}")).len()
}

#[test]
fn the_retry_wait_doubles_from_fifteen_seconds_to_ten_minutes() {
    let up = Upstream::new(common::client(), NetworkConfig::new(common::device_ids()));
    let waits: Vec<u64> = [0, 1, 3, 5, 6, u32::MAX]
        .into_iter()
        .map(|n| up.network_retry_delay(n).as_secs())
        .collect();
    assert_eq!(waits, [15, 30, 120, 480, 600, 600]);
}

#[test]
fn a_server_has_its_network_once_any_domain_is_set() {
    let mut cfg = NetworkConfig::new(common::device_ids());
    assert!(Server::ALL.into_iter().all(|s| !cfg.has_network(s)));
    cfg.set_domain(Server::EN, Domain::U8, "https://as.example/u8");
    assert!(cfg.has_network(Server::EN));
    assert!(!cfg.has_network(Server::JP));
}

#[test]
fn a_bad_network_body_says_what_is_wrong() {
    let cases = [
        ("<html>blocked</html>".to_owned(), "unexpected response"),
        (response("not json"), "unexpected content"),
        (
            response(r#"{"funcVer":"V058","configs":{"V057":{"network":{}}}}"#),
            "no config for funcVer V058",
        ),
        (
            response(r#"{"funcVer":"V058","configs":{"V058":{"override":true}}}"#),
            "no network table under funcVer V058",
        ),
        (
            response(r#"{"funcVer":"V058","configs":{"V058":{"network":{"mystery":"x"}}}}"#),
            "names no known domain",
        ),
        (
            response(r#"{"funcVer":"V058","configs":{"V058":{"network":{}}}}"#),
            "names no known domain",
        ),
    ];
    for (body, expected) in cases {
        let error = parse_network(body.as_bytes()).expect_err(&body);
        assert!(
            matches!(&error, UpstreamError::Parse { message, .. } if message.contains(expected)),
            "{error:?} should mention {expected:?}"
        );
    }
}

#[tokio::test]
async fn a_network_with_no_known_domain_is_not_loaded() {
    let server = serve(|req: &Recorded| {
        Reply::text(
            200,
            &response(&json!({ "funcVer": "V1", "configs": { "V1": { "network": { "mystery": req.path } } } }).to_string()),
        )
    })
    .await;
    let up = Upstream::new(common::client(), unloaded(&server.base));
    assert_eq!(up.reload_network().await, 0);
    assert_eq!(up.missing_networks(&Server::ALL), Server::ALL);
}

#[tokio::test]
async fn an_error_status_is_a_failed_load() {
    let server = serve(|req: &Recorded| {
        let base = req.header("host").map(|h| format!("http://{h}")).unwrap();
        Reply::text(503, &routes(&base, "en"))
    })
    .await;
    let up = Upstream::new(common::client(), unloaded(&server.base));
    assert_eq!(up.reload_network().await, 0);
    assert!(!up.config().has_network(Server::EN));
}

#[tokio::test]
async fn the_retry_loads_each_missing_server_then_its_version_and_stops() {
    let en_tries = Arc::new(AtomicUsize::new(0));
    let tries = Arc::clone(&en_tries);
    let server = serve(move |req: &Recorded| {
        let base = req.header("host").map(|h| format!("http://{h}")).unwrap();
        match req.path.as_str() {
            "/conf/en" if tries.fetch_add(1, Ordering::SeqCst) < 2 => Reply::text(500, "down"),
            "/conf/en" => Reply::text(200, &routes(&base, "en")),
            "/conf/jp" => Reply::text(200, &routes(&base, "jp")),
            "/v/en/Android/version" => {
                Reply::json(&json!({ "resVersion": "r-en", "clientVersion": "c-en" }))
            }
            _ => Reply::text(500, "down"),
        }
    })
    .await;
    let up = fast(unloaded(&server.base));
    up.reload_network().await;
    assert_eq!(up.missing_networks(&[Server::EN, Server::JP]), [Server::EN]);

    let retry = up.spawn_network_retry(vec![Server::EN, Server::JP]);
    tokio::time::timeout(Duration::from_secs(5), retry)
        .await
        .expect("the retry ends once every configured server is loaded")
        .unwrap();

    let cfg = up.config();
    assert_eq!(
        cfg.domain(Server::EN, Domain::GS),
        Some(format!("{}/gs", server.base).as_str())
    );
    assert_eq!(cfg.version(Server::EN).client_version, "c-en");
    assert_eq!(
        conf_requests(&server, "en"),
        3,
        "one boot load and two retries"
    );
    assert_eq!(
        conf_requests(&server, "jp"),
        1,
        "a loaded server is not retried"
    );
    assert_eq!(
        conf_requests(&server, "kr"),
        1,
        "an unconfigured server is not retried"
    );
}

#[tokio::test]
async fn the_retry_keeps_going_while_a_server_is_missing() {
    let server = serve(|_: &Recorded| Reply::text(500, "down")).await;
    let up = fast(unloaded(&server.base));
    let retry = up.spawn_network_retry(vec![Server::EN]);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!retry.is_finished());
    assert!(conf_requests(&server, "en") >= 3);

    drop(up);
    tokio::time::timeout(Duration::from_secs(1), retry)
        .await
        .expect("the retry ends with its upstream")
        .unwrap();
}
