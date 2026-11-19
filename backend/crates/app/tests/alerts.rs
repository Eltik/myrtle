#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::alerts::{degraded_tables_embed, report_degraded_tables};
use myrtle_app::context::Secrets;
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_types::Server;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[test]
fn nothing_degraded_is_no_alert() {
    assert_eq!(degraded_tables_embed("en", &[]), None);
}

#[test]
fn the_embed_lists_fifteen_tables_and_counts_the_rest() {
    let mut warnings: Vec<String> = (0..16).map(|n| format!("table_{n}: bad")).collect();
    warnings[0] = "item_table: unknown `variant`".to_owned();
    let embed = degraded_tables_embed("kr", &warnings).unwrap();
    assert_eq!(embed["title"], "Degraded game-data tables (kr)");
    let description = embed["description"].as_str().unwrap();
    let lines: Vec<&str> = description.lines().collect();
    assert_eq!(lines.len(), 16);
    assert_eq!(lines[0], "• `item_table: unknown 'variant'`");
    assert_eq!(lines[14], "• `table_14: bad`");
    assert_eq!(lines[15], "…and 1 more");
    assert_eq!(embed["color"], 0x00E0_A81A_u32);
}

async fn posted(bodies: &mut mpsc::UnboundedReceiver<String>) -> String {
    tokio::time::timeout(Duration::from_secs(10), bodies.recv())
        .await
        .expect("an alert reached the webhook")
        .unwrap()
}

fn client() -> reqwest::Client {
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::new()
}

/// A webhook on a local port that answers 204 and forwards each request body
async fn webhook() -> (String, mpsc::UnboundedReceiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/hook", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut request = Vec::new();
            let mut buf = [0_u8; 4096];
            while let Ok(n) = socket.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&request);
                if let Some((head, body)) = text.split_once("\r\n\r\n") {
                    let length = head
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    if body.len() >= length {
                        let _ = tx.send(body.to_owned());
                        break;
                    }
                }
            }
            let _ = socket
                .write_all(b"HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n")
                .await;
        }
    });
    (url, rx)
}

#[tokio::test]
async fn a_degraded_load_posts_one_embed_to_the_webhook() {
    let (url, mut bodies) = webhook().await;
    let warnings = vec!["item_table: unknown variant".to_owned()];
    report_degraded_tables(&client(), Some(&url), "en", &warnings).await;
    let body: Value = serde_json::from_str(&posted(&mut bodies).await).unwrap();
    assert_eq!(
        body["embeds"][0],
        degraded_tables_embed("en", &warnings).unwrap()
    );
}

#[tokio::test]
async fn no_webhook_or_nothing_degraded_posts_nothing() {
    let (url, mut bodies) = webhook().await;
    let client = client();
    let warnings = vec!["item_table: unknown variant".to_owned()];
    report_degraded_tables(&client, None, "en", &warnings).await;
    report_degraded_tables(&client, Some(" "), "en", &warnings).await;
    report_degraded_tables(&client, Some(&url), "en", &[]).await;
    let waited = tokio::time::timeout(Duration::from_millis(200), bodies.recv()).await;
    assert!(waited.is_err(), "no request reached the webhook");
}

#[tokio::test]
async fn an_unreachable_webhook_is_swallowed() {
    let warnings = vec!["item_table: unknown variant".to_owned()];
    report_degraded_tables(&client(), Some("http://127.0.0.1:9/hook"), "en", &warnings).await;
}

fn extract(root: &Path) {
    let excel = root.join("gamedata/excel");
    std::fs::create_dir_all(&excel).unwrap();
    std::fs::write(excel.join("data_version.txt.txt"), "VersionControl:1.0.0\n").unwrap();
    std::fs::write(excel.join("character_table.json"), r#"{"Characters":[]}"#).unwrap();
}

#[tokio::test]
async fn a_reload_with_degraded_tables_posts_its_alert() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let (url, mut bodies) = webhook().await;
    let mut config = (*base.config).clone();
    config.assets.dir = assets.path().to_str().unwrap().to_owned();
    config.server.servers = vec![Server::EN];
    let ctx = AppContext {
        snapshots: Arc::new(SnapshotRegistry::new(&config)),
        config: Arc::new(config),
        secrets: Arc::new(Secrets {
            jwt: base.secrets.jwt.clone(),
            credentials: Arc::clone(&base.secrets.credentials),
            service_key: None,
            discord_alert_webhook: Some(url),
        }),
        ..base.clone()
    };
    ctx.snapshots
        .activate_at(Server::EN, 0, None)
        .await
        .unwrap();
    ctx.announce_update(Server::EN, None).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
    let body: Value = serde_json::from_str(&posted(&mut bodies).await).unwrap();
    let live = ctx.snapshots.require(Server::EN).unwrap();
    assert!(
        !live.model.meta.table_warnings.is_empty(),
        "a bare tree is degraded"
    );
    assert_eq!(
        body["embeds"][0],
        degraded_tables_embed("en", &live.model.meta.table_warnings).unwrap()
    );
    db.close().await;
}
