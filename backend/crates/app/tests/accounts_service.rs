#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::services::accounts::service_account::{
    ImportOutcome, ServiceLogin, import_service_sessions, send_service_code,
    service_session_status, set_service_session,
};
use std::sync::Arc;

use myrtle_hypergryph::{
    AuthSession, DeviceIds, Domain, NetworkConfig, SessionKey, Upstream, VersionInfo,
};
use myrtle_types::Server;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Notify;

fn file(token: &str) -> String {
    json!({
        "uid": "9100",
        "secret": "s",
        "seqnum": 4,
        "token": "t",
        "yostar_uid": "y-uid",
        "yostar_token": token,
    })
    .to_string()
}

async fn stored(ctx: &myrtle_app::AppContext, server: Server) -> Option<AuthSession> {
    ctx.sessions
        .load(&SessionKey::ServiceAccount { server })
        .await
        .unwrap()
}

#[tokio::test]
async fn session_files_import_once_unless_forced() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("en.json"), file("token-en")).unwrap();
    std::fs::write(dir.path().join("bili.json"), file("token-bili")).unwrap();
    std::fs::write(dir.path().join("bili.otp.json"), "{}").unwrap();

    assert_eq!(
        import_service_sessions(&ctx, dir.path(), false).await,
        [
            (Server::EN, ImportOutcome::Imported),
            (Server::Bilibili, ImportOutcome::Imported)
        ]
    );
    let en = stored(&ctx, Server::EN).await.unwrap();
    assert_eq!(
        (en.uid.as_ref(), en.seqnum, en.yostar_token.as_ref()),
        ("9100", 4, "token-en")
    );
    assert_eq!(stored(&ctx, Server::JP).await, None);

    std::fs::write(dir.path().join("en.json"), file("token-en-2")).unwrap();
    assert_eq!(
        import_service_sessions(&ctx, dir.path(), false).await,
        [
            (Server::EN, ImportOutcome::Existing),
            (Server::Bilibili, ImportOutcome::Existing)
        ]
    );
    let token = |s: Option<AuthSession>| s.unwrap().yostar_token.to_string();
    assert_eq!(token(stored(&ctx, Server::EN).await), "token-en");
    assert_eq!(
        import_service_sessions(&ctx, dir.path(), true).await,
        [
            (Server::EN, ImportOutcome::Imported),
            (Server::Bilibili, ImportOutcome::Imported)
        ]
    );
    assert_eq!(token(stored(&ctx, Server::EN).await), "token-en-2");
    db.close().await;
}

#[tokio::test]
async fn a_bad_session_file_costs_only_its_server_and_never_echoes_a_token() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("en.json"), file("token-en")).unwrap();
    std::fs::write(dir.path().join("jp.json"), file("")).unwrap();
    std::fs::write(
        dir.path().join("kr.json"),
        r#"{"uid": "1", "secret": "leaked-secret", "seqnum": "leaked-token"}"#,
    )
    .unwrap();
    std::fs::write(dir.path().join("cn.json"), " ".repeat(70 * 1024)).unwrap();
    let outcomes = import_service_sessions(&ctx, dir.path(), false).await;
    let reason = |server: Server| match outcomes.iter().find(|(s, _)| *s == server) {
        Some((_, ImportOutcome::Failed(reason))) => reason.clone(),
        other => panic!("{server:?}: {other:?}"),
    };
    assert_eq!(outcomes[0], (Server::EN, ImportOutcome::Imported));
    assert_eq!(reason(Server::JP), "no durable token");
    let kr = reason(Server::KR);
    assert!(kr.starts_with("not a session (Data at line 1"), "{kr}");
    assert!(!kr.contains("leaked"), "{kr}");
    assert_eq!(reason(Server::CN), "larger than 65536 bytes");
    assert_eq!(outcomes.len(), 4);
    assert_eq!(
        stored(&ctx, Server::EN)
            .await
            .unwrap()
            .yostar_token
            .as_ref(),
        "token-en"
    );
    db.close().await;
}

#[cfg(unix)]
#[tokio::test]
async fn a_symlinked_session_file_is_not_followed() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let target = elsewhere.path().join("session.json");
    std::fs::write(&target, file("token-en")).unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join("en.json")).unwrap();
    assert_eq!(
        import_service_sessions(&ctx, dir.path(), false).await,
        [(
            Server::EN,
            ImportOutcome::Failed("not a regular file".to_owned())
        )]
    );
    assert_eq!(stored(&ctx, Server::EN).await, None);
    db.close().await;
}

#[tokio::test]
async fn an_empty_directory_imports_nothing() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    assert!(
        import_service_sessions(&ctx, dir.path(), true)
            .await
            .is_empty()
    );
    db.close().await;
}

#[tokio::test]
async fn a_login_must_match_its_server_before_any_upstream_call() {
    let (ctx, db) = common::context().await;
    let yostar = ServiceLogin::Yostar {
        email: "a@example.com",
        code: "123456",
    };
    assert!(matches!(
        set_service_session(&ctx, Server::Bilibili, yostar).await,
        Err(AppError::BadRequest(m)) if m.contains("bili server logs in with Bilibili")
    ));
    let bilibili = ServiceLogin::Bilibili {
        username: "u",
        password: "p",
    };
    assert!(matches!(
        set_service_session(&ctx, Server::EN, bilibili).await,
        Err(AppError::BadRequest(m)) if m == "a Bilibili login needs the bili server"
    ));
    assert!(matches!(
        send_service_code(&ctx, Server::Bilibili, "a@example.com").await,
        Err(AppError::BadRequest(_))
    ));
    let no_credential = AuthSession::new(Some("1"), Some("s"), Some(1), Some("t"));
    assert!(matches!(
        set_service_session(&ctx, Server::EN, ServiceLogin::Import(no_credential)).await,
        Err(AppError::BadRequest(m)) if m.contains("no durable token")
    ));
    assert_eq!(stored(&ctx, Server::EN).await, None);
    db.close().await;
}

#[tokio::test]
async fn the_status_of_a_server_with_no_session_is_not_found() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        service_session_status(&ctx, Server::KR).await,
        Err(AppError::NotFound(Some(m))) if m == "no session for kr"
    ));
    db.close().await;
}

/// Answers the two refresh calls; `getToken` holds until `release` and signals `reached` first
async fn refresh_server(reached: Arc<Notify>, release: Arc<Notify>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        loop {
            let Ok((mut sock, _)) = listener.accept().await else {
                return;
            };
            let (reached, release) = (Arc::clone(&reached), Arc::clone(&release));
            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 4096];
                let path = loop {
                    let n = sock.read(&mut chunk).await.unwrap();
                    buf.extend_from_slice(&chunk[..n]);
                    let text = String::from_utf8_lossy(&buf);
                    let Some(head_end) = text.find("\r\n\r\n") else {
                        continue;
                    };
                    let len: usize = text[..head_end]
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse().unwrap())
                        })
                        .unwrap_or(0);
                    if buf.len() >= head_end + 4 + len {
                        break text.split(' ').nth(1).unwrap().to_owned();
                    }
                };
                let body = if path.ends_with("/user/v1/getToken") {
                    reached.notify_one();
                    release.notified().await;
                    json!({ "result": 0, "uid": "9100", "token": "u8-token" })
                } else {
                    json!({ "result": 0, "uid": "9100", "secret": "refreshed-secret" })
                }
                .to_string();
                let reply = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                sock.write_all(reply.as_bytes()).await.unwrap();
                let _ = sock.shutdown().await;
            });
        }
    });
    base
}

fn upstream(base: &str) -> Upstream {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let mut config = NetworkConfig::new(DeviceIds {
        device_id: "dev-one".into(),
        device_id2: "86000000000001".into(),
        device_id3: "dev-three".into(),
    });
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
}

#[tokio::test]
async fn a_status_check_keeps_its_secret_and_never_overwrites_a_seed_that_landed_mid_refresh() {
    let (mut ctx, db) = common::context().await;
    let (reached, release) = (Arc::new(Notify::new()), Arc::new(Notify::new()));
    ctx.upstream = Arc::new(upstream(
        &refresh_server(Arc::clone(&reached), Arc::clone(&release)).await,
    ));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("en.json"), file("token-en")).unwrap();
    import_service_sessions(&ctx, dir.path(), false).await;

    let check = {
        let ctx = ctx.clone();
        tokio::spawn(async move { service_session_status(&ctx, Server::EN).await })
    };
    reached.notified().await;
    release.notify_one();
    let status = check.await.unwrap().unwrap();
    assert_eq!(status.uid, "9100");
    assert_eq!(
        stored(&ctx, Server::EN).await.unwrap().secret.as_ref(),
        "refreshed-secret",
        "an uncontested check keeps the fresh secret"
    );

    let check = {
        let ctx = ctx.clone();
        tokio::spawn(async move { service_session_status(&ctx, Server::EN).await })
    };
    reached.notified().await;
    let mut other_login: AuthSession = serde_json::from_str(&file("token-other")).unwrap();
    other_login.yostar_uid = "y-other".into();
    other_login.secret = "other-secret".into();
    std::fs::write(
        dir.path().join("en.json"),
        serde_json::to_string(&other_login).unwrap(),
    )
    .unwrap();
    assert_eq!(
        import_service_sessions(&ctx, dir.path(), true).await,
        [(Server::EN, ImportOutcome::Imported)]
    );
    release.notify_one();
    check.await.unwrap().unwrap();
    assert_eq!(stored(&ctx, Server::EN).await, Some(other_login));
    db.close().await;
}
