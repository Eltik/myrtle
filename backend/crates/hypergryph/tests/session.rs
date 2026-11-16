//! Secret refresh, the stale-version retry, `ensure_fresh`, and the raw fetch paths.

mod common;

use std::time::Duration;

use common::{MemoryStore, Recorded, Reply, ValidTokens, durable, game_server, serve, upstream};
use myrtle_hypergryph::fetch::{
    FetchRequest, auth_request, auth_request_with_timeout, fetch_domain, read_body,
    read_body_tolerating,
};
use myrtle_hypergryph::session::{ensure_fresh, refresh_secret, refresh_secret_by_step};
use myrtle_hypergryph::{
    AuthSession, Domain, RefreshStep, SessionKey, UpstreamError, VersionInfo, envelope_code,
};
use myrtle_types::{GameUid, Server};
use serde_json::json;

fn durable_session(token: &str) -> AuthSession {
    common::session_with(&durable("yostar-uid", token))
}

#[tokio::test]
async fn refresh_mints_a_fresh_session_in_place() {
    let server = serve(game_server(ValidTokens::new(&["tok"]), |_, _| {
        Reply::json(&json!({}))
    }))
    .await;
    let up = upstream(&server.base);
    let mut session = durable_session("tok");
    session.seqnum = 40;

    refresh_secret(&up, &mut session, Server::EN).await.unwrap();
    assert_eq!(
        session,
        AuthSession {
            uid: "u8-uid".into(),
            secret: "secret-1".into(),
            seqnum: 1,
            token: "u8-tok".into(),
            yostar_uid: "yostar-uid".into(),
            yostar_token: "tok".into(),
        }
    );
    assert_eq!(
        server.paths(),
        ["/u8/user/v1/getToken", "/gs/account/login"]
    );
}

#[tokio::test]
async fn refresh_names_the_failing_step() {
    let server = serve(|req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" if common::extension_token(req) == "good" => {
            Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" }))
        }
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 2 })),
        _ => Reply::json(&json!({ "result": 4 })),
    })
    .await;
    let up = upstream(&server.base);

    let (step, err) = refresh_secret_by_step(&up, &mut durable_session("bad"), Server::EN)
        .await
        .unwrap_err();
    assert_eq!(step, RefreshStep::Token);
    assert!(
        matches!(&err, UpstreamError::LoginRejected { context: "session::user/v1/getToken", code, .. } if code == "2"),
        "{err:?}"
    );

    let (step, err) = refresh_secret_by_step(&up, &mut durable_session("good"), Server::EN)
        .await
        .unwrap_err();
    assert_eq!(step, RefreshStep::Login);
    assert!(
        matches!(&err, UpstreamError::LoginRejected { context: "session::account/login", code, .. } if code == "4"),
        "{err:?}"
    );

    let (step, err) = refresh_secret_by_step(&up, &mut AuthSession::default(), Server::EN)
        .await
        .unwrap_err();
    assert_eq!(step, RefreshStep::Token);
    assert!(matches!(err, UpstreamError::NotLoggedIn));
}

fn stale_version_server(fresh_res: &'static str) -> impl Fn(&Recorded) -> Reply + Send + Sync {
    move |req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" })),
        "/hv/en/Android/version" => {
            Reply::json(&json!({ "resVersion": fresh_res, "clientVersion": "client-2" }))
        }
        "/gs/account/login" if req.json()["assetsVersion"] == "res-2" => {
            Reply::json(&json!({ "result": 0, "secret": "s" }))
        }
        _ if req.path.starts_with("/hv/") => Reply::status(404, &json!({})),
        _ => Reply::json(&json!({ "result": 1 })),
    }
}

#[tokio::test]
async fn a_stale_version_is_reloaded_and_the_login_retried_once() {
    let server = serve(stale_version_server("res-2")).await;
    let up = upstream(&server.base);
    let mut session = durable_session("tok");

    refresh_secret(&up, &mut session, Server::EN).await.unwrap();
    assert_eq!(&*session.secret, "s");
    let logins = server.to("/gs/account/login");
    assert_eq!(logins.len(), 2);
    assert_eq!(logins[0].json()["assetsVersion"], "res-1");
    assert_eq!(logins[1].json()["clientVersion"], "client-2");
    assert_eq!(server.to("/hv/en/Android/version").len(), 1);
    assert_eq!(up.config().version(Server::EN).res_version, "res-2");
}

#[tokio::test]
async fn a_failed_retry_after_a_version_change_is_the_last_attempt() {
    let server = serve(|req: &Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" })),
        "/hv/en/Android/version" => {
            Reply::json(&json!({ "resVersion": "res-2", "clientVersion": "client-2" }))
        }
        _ if req.path.starts_with("/hv/") => Reply::status(404, &json!({})),
        _ => Reply::json(&json!({ "result": 1 })),
    })
    .await;
    let up = upstream(&server.base);

    let (step, err) = refresh_secret_by_step(&up, &mut durable_session("tok"), Server::EN)
        .await
        .unwrap_err();
    assert_eq!(step, RefreshStep::Login);
    assert!(
        matches!(err, UpstreamError::LoginRejected { .. }),
        "{err:?}"
    );
    let logins = server.to("/gs/account/login");
    assert_eq!(logins.len(), 2);
    assert_eq!(logins[1].json()["assetsVersion"], "res-2");
    assert_eq!(server.to("/hv/en/Android/version").len(), 1);
}

#[tokio::test]
async fn a_current_version_is_not_retried() {
    let server = serve(stale_version_server("res-1")).await;
    let up = upstream(&server.base);
    let mut cfg = common::network_config(&server.base);
    cfg.set_version(
        Server::EN,
        VersionInfo {
            res_version: "res-1".into(),
            client_version: "client-2".into(),
        },
    );
    up.update(|c| *c = cfg.clone());

    let err = refresh_secret(&up, &mut durable_session("tok"), Server::EN)
        .await
        .unwrap_err();
    assert!(
        matches!(err, UpstreamError::LoginRejected { .. }),
        "{err:?}"
    );
    assert_eq!(server.to("/gs/account/login").len(), 1);
}

#[tokio::test]
async fn missing_versions_load_before_the_first_login() {
    let server = serve(stale_version_server("res-2")).await;
    let up = upstream(&server.base);
    let mut cfg = common::network_config(&server.base);
    cfg.set_version(Server::EN, VersionInfo::default());
    up.update(|c| *c = cfg.clone());

    refresh_secret(&up, &mut durable_session("tok"), Server::EN)
        .await
        .unwrap();
    assert_eq!(server.to("/gs/account/login").len(), 1);
    let paths = server.paths();
    let at = |p: &str| paths.iter().position(|x| x == p).unwrap();
    assert!(at("/hv/en/Android/version") < at("/gs/account/login"));
}

#[tokio::test]
async fn cn_uses_channel_one_network_five_and_the_override_wins() {
    let server = serve(game_server(ValidTokens::new(&["code"]), |_, _| {
        Reply::json(&json!({}))
    }))
    .await;
    let up = upstream(&server.base);
    refresh_secret(
        &up,
        &mut common::session_with(&durable("cn-uid", "code")),
        Server::CN,
    )
    .await
    .unwrap();
    let body = server.to("/u8/user/v1/getToken")[0].json();
    assert_eq!(body["channelId"], "1");
    assert_eq!(
        body["extension"],
        r#"{"access_token":"code","uid":"cn-uid"}"#
    );
    assert_eq!(
        server.to("/gs/account/login")[0].json()["networkVersion"],
        "5"
    );

    let mut cfg = common::network_config(&server.base);
    cfg.network_version_override = Some("9".into());
    up.update(|c| *c = cfg.clone());
    refresh_secret(
        &up,
        &mut common::session_with(&durable("b", "code")),
        Server::Bilibili,
    )
    .await
    .unwrap();
    assert_eq!(
        server.to("/u8/user/v1/getToken")[1].json()["channelId"],
        "2"
    );
    assert_eq!(
        server.to("/gs/account/login")[1].json()["networkVersion"],
        "9"
    );
}

#[tokio::test]
async fn tw_is_unsupported_without_a_request() {
    let server = serve(|_: &Recorded| Reply::json(&json!({}))).await;
    let up = upstream(&server.base);
    let err = refresh_secret(&up, &mut durable_session("tok"), Server::TW)
        .await
        .unwrap_err();
    assert!(
        matches!(
            err,
            UpstreamError::Unsupported {
                server: Server::TW,
                ..
            }
        ),
        "{err:?}"
    );
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn ensure_fresh_prefers_the_cache_then_the_credential() {
    let server = serve(game_server(
        ValidTokens::new(&["cached", "durable"]),
        |_, _| Reply::json(&json!({})),
    ))
    .await;
    let up = upstream(&server.base);
    let store = MemoryStore::default();
    let key = SessionKey::Player {
        server: Server::EN,
        uid: GameUid("12345".into()),
    };

    let err = ensure_fresh(&up, &store, &key).await.unwrap_err();
    assert!(matches!(err, UpstreamError::NotLoggedIn), "{err:?}");
    assert!(server.requests().is_empty());

    store.put_durable(&key, durable("y", "durable"));
    let session = ensure_fresh(&up, &store, &key).await.unwrap();
    assert_eq!(&*session.secret, "secret-1");
    assert_eq!(common::extension_token(&server.requests()[0]), "durable");
    assert_eq!(store.session(&key).unwrap(), session);

    store.put_session(&key, durable_session("cached"));
    ensure_fresh(&up, &store, &key).await.unwrap();
    assert_eq!(common::extension_token(&server.requests()[2]), "cached");
    assert_eq!(&*store.session(&key).unwrap().secret, "secret-2");
}

fn logged_in() -> AuthSession {
    AuthSession::new(Some("u"), Some("s"), Some(7), None)
}

#[tokio::test]
async fn auth_request_without_a_uid_sends_nothing() {
    let server = serve(|_: &Recorded| Reply::json(&json!({}))).await;
    let up = upstream(&server.base);
    let mut session = AuthSession::default();
    let err = auth_request(&up, "x", None, &mut session, Server::EN)
        .await
        .unwrap_err();
    assert!(matches!(err, UpstreamError::NotLoggedIn));
    assert_eq!(session.seqnum, 0);
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn a_body_less_request_is_a_get_with_default_headers() {
    let server = serve(|_: &Recorded| Reply::json(&json!({ "a": 1 }))).await;
    let up = upstream(&server.base);
    let mut session = logged_in();
    let response = auth_request(&up, "user/info", None, &mut session, Server::JP)
        .await
        .unwrap();
    assert_eq!(read_body(response, "t").await.unwrap(), r#"{"a":1}"#);
    let req = &server.requests()[0];
    assert_eq!(
        (req.method.as_str(), req.path.as_str()),
        ("GET", "/gs/user/info")
    );
    assert_eq!(req.seqnum(), 8);
    assert_eq!(
        req.header("user-agent"),
        Some("Dalvik/2.1.0 (Linux; U; Android 11; KB2000 Build/RP1A.201005.001)")
    );
    assert_eq!(req.header("content-type"), Some("application/json"));
    assert_eq!(
        req.header("connection")
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("keep-alive")
    );
}

#[tokio::test]
async fn domains_substitute_the_platform_and_signing_uses_the_session() {
    let server = serve(|_: &Recorded| Reply::json(&json!({}))).await;
    let up = upstream(&server.base);
    let mut cfg = common::network_config(&server.base);
    cfg.set_domain(Server::KR, Domain::AS, format!("{}/as/{{0}}", server.base));
    up.update(|c| *c = cfg.clone());
    let session = logged_in();
    let body = json!({ "b": [1, 2] });
    fetch_domain(
        &up,
        Domain::AS,
        FetchRequest {
            endpoint: Some("x/y"),
            body: Some(&body),
            session: Some(&session),
            server: Server::KR,
            sign: true,
            timeout: Duration::from_secs(5),
        },
    )
    .await
    .unwrap();
    let req = &server.requests()[0];
    assert_eq!(req.path, "/as/Android/x/y");
    assert_eq!(req.body, r#"{"b":[1,2]}"#);
    common::assert_signed(req, "u", "s");
    let auth: serde_json::Value =
        serde_json::from_str(req.header("authorization").unwrap()).unwrap();
    assert_eq!(auth["Head"]["Lang"], "ko");
    assert_eq!(auth["Head"]["PID"], "KR-ARKNIGHTS");

    let err = fetch_domain(
        &up,
        Domain::RC,
        FetchRequest {
            endpoint: None,
            body: None,
            session: None,
            server: Server::KR,
            sign: false,
            timeout: Duration::from_secs(5),
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(
        err,
        UpstreamError::DomainNotFound {
            server: Server::KR,
            domain: Domain::RC
        }
    ));
}

#[tokio::test]
async fn envelopes_classify_and_tolerated_codes_stay_errors() {
    let server = serve(|req: &Recorded| match req.path.as_str() {
        "/gs/replay" => Reply::status(
            400,
            &json!({ "code": envelope_code::STAGE_NO_REPLAY, "msg": "no" }),
        ),
        "/gs/down" => Reply::status(500, &json!({ "error": "メンテナンス" })),
        _ => Reply::text(502, "<html>bad gateway</html>"),
    })
    .await;
    let up = upstream(&server.base);
    let mut session = logged_in();

    let response = auth_request(&up, "replay", Some(&json!({})), &mut session, Server::EN)
        .await
        .unwrap();
    let err = read_body_tolerating(response, "replay", &[envelope_code::STAGE_NO_REPLAY])
        .await
        .unwrap_err();
    assert!(
        err.is_envelope_code(envelope_code::STAGE_NO_REPLAY),
        "{err:?}"
    );

    let response = auth_request(&up, "down", None, &mut session, Server::EN)
        .await
        .unwrap();
    let err = read_body(response, "down").await.unwrap_err();
    assert!(matches!(err, UpstreamError::Maintenance(_)), "{err:?}");

    let response = auth_request(&up, "html", None, &mut session, Server::EN)
        .await
        .unwrap();
    let err = read_body(response, "html").await.unwrap_err();
    assert!(
        matches!(&err, UpstreamError::Parse { context, message } if context == "html" && message.contains("status=502")),
        "{err:?}"
    );
}

#[tokio::test]
async fn a_slow_answer_is_a_timeout() {
    let server = serve(|_: &Recorded| Reply::json(&json!({})).after(Duration::from_secs(3))).await;
    let up = upstream(&server.base);
    let mut session = logged_in();
    let err = auth_request_with_timeout(
        &up,
        "slow",
        None,
        &mut session,
        Server::EN,
        Duration::from_millis(200),
    )
    .await
    .unwrap_err();
    assert!(matches!(err, UpstreamError::Timeout(_)), "{err:?}");
}
