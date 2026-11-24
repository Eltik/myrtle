//! `GameAccount` against a scripted game server: refresh, seqnum, 401s, refusals, and the store.

mod common;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use common::{
    Reply, ValidTokens, durable, extension_token, game_server, serve, session_with, upstream,
};
use myrtle_hypergryph::{
    AccountOptions, AccountState, Domain, GameAccount, GameAccounts, SessionKey, SessionStore,
    UpstreamError, envelope_code,
};
use myrtle_testkit::sessions::MemorySessions;
use myrtle_types::Server;
use serde_json::json;

const EN: SessionKey = SessionKey::ServiceAccount { server: Server::EN };

fn store_with(token: &str) -> Arc<MemorySessions> {
    let store = Arc::new(MemorySessions::default());
    store.put_session(&EN, session_with(&durable("yostar-uid", token)));
    store
}

async fn account(store: &Arc<MemorySessions>, options: AccountOptions) -> Arc<GameAccount> {
    let dyn_store: Arc<dyn SessionStore> = store.clone();
    GameAccount::load(Server::EN, dyn_store, options)
        .await
        .expect("memory store never fails")
        .expect("store holds a credential")
}

const fn first_of_outage(err: &UpstreamError) -> Option<bool> {
    match err {
        UpstreamError::ServiceAccountRefused {
            server: Server::EN,
            first_of_outage,
        } => Some(*first_of_outage),
        _ => None,
    }
}

fn ok_gs(_: &common::Recorded, _: usize) -> Reply {
    Reply::json(&json!({ "ok": 1 }))
}

#[tokio::test]
async fn first_request_refreshes_then_bumps_seqnum_and_saves() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    let acct = account(&store, AccountOptions::default()).await;

    let answer = acct
        .request(&up, "test/ping", &json!({ "x": 1 }))
        .await
        .unwrap();
    assert_eq!(answer, json!({ "ok": 1 }));
    assert_eq!(
        server.paths(),
        ["/u8/user/v1/getToken", "/gs/account/login", "/gs/test/ping"]
    );

    let reqs = server.requests();
    let get_token = &reqs[0];
    common::assert_signed(get_token, "", "");
    let body = get_token.json();
    assert_eq!(body["channelId"], "3");
    assert_eq!(body["subChannel"], "3");
    assert_eq!(body["worldId"], "3");
    assert_eq!(body["appId"], "1");
    assert_eq!(body["platform"], "1");
    assert_eq!(body["deviceId2"], common::DEVICE_IDS[1]);
    assert_eq!(
        body["extension"],
        r#"{"token":"tok-a","type":1,"uid":"yostar-uid"}"#
    );
    let extension = body["extension"].as_str().unwrap();
    let ids = common::DEVICE_IDS;
    let expected_sign = myrtle_hypergryph::crypto::generate_u8_sign(&[
        ("appId", "1"),
        ("platform", "1"),
        ("channelId", "3"),
        ("subChannel", "3"),
        ("extension", extension),
        ("worldId", "3"),
        ("deviceId", ids[0]),
        ("deviceId2", ids[1]),
        ("deviceId3", ids[2]),
    ]);
    assert_eq!(body["sign"], expected_sign);
    assert_eq!(get_token.header("uid"), None);

    let login = &reqs[1];
    common::assert_signed(login, "u8-uid", "");
    assert_eq!(login.header("uid"), Some("u8-uid"));
    assert_eq!(login.header("secret"), Some(""));
    assert_eq!(login.seqnum(), 1);
    assert_eq!(
        login.json(),
        json!({
            "platform": 1, "networkVersion": "1", "assetsVersion": "res-1", "clientVersion": "client-1",
            "token": "u8-tok-a", "uid": "u8-uid",
            "deviceId": ids[0], "deviceId2": ids[1], "deviceId3": ids[2],
        })
    );

    let ping = &reqs[2];
    assert_eq!(ping.method, "POST");
    assert_eq!(ping.header("uid"), Some("u8-uid"));
    assert_eq!(ping.header("secret"), Some("secret-1"));
    assert_eq!(ping.seqnum(), 2);
    assert_eq!(ping.header("authorization"), None);
    assert_eq!(ping.header("x-unity-version"), Some("2017.4.39f1"));
    assert_eq!(ping.json(), json!({ "x": 1 }));

    let saved = store.session(&EN).unwrap();
    assert_eq!(&*saved.secret, "secret-1");
    assert_eq!(saved.seqnum, 2);
    assert_eq!(&*saved.token, "u8-tok-a");
    assert_eq!(&*saved.yostar_token, "tok-a");
    assert_eq!(
        store.saves().len(),
        2,
        "after the refresh and after the request"
    );
    assert_eq!(acct.health().state, AccountState::Ok);
    assert_eq!(acct.uid().await, "u8-uid");
}

#[tokio::test]
async fn a_fresh_secret_is_reused() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let acct = account(
        &store_with("tok-a"),
        AccountOptions {
            secret_max_age: Duration::from_secs(3600),
        },
    )
    .await;

    acct.request(&up, "a", &json!({})).await.unwrap();
    acct.request(&up, "b", &json!({})).await.unwrap();
    assert_eq!(server.to("/gs/account/login").len(), 1);
    assert_eq!(
        (
            server.to("/gs/a")[0].seqnum(),
            server.to("/gs/b")[0].seqnum()
        ),
        (2, 3)
    );
}

#[tokio::test]
async fn a_stale_secret_is_refreshed_before_the_request() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let acct = account(
        &store_with("tok-a"),
        AccountOptions {
            secret_max_age: Duration::ZERO,
        },
    )
    .await;

    acct.request(&up, "a", &json!({})).await.unwrap();
    acct.request(&up, "b", &json!({})).await.unwrap();
    assert_eq!(server.to("/gs/account/login").len(), 2);
    let b = &server.to("/gs/b")[0];
    assert_eq!((b.seqnum(), b.header("secret")), (2, Some("secret-2")));
}

#[tokio::test]
async fn a_401_refreshes_once_and_retries() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), |_, nth| {
        if nth == 0 {
            Reply::status(401, &json!({}))
        } else {
            Reply::json(&json!({ "ok": nth }))
        }
    }))
    .await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    let acct = account(&store, AccountOptions::default()).await;

    let answer = acct.request(&up, "shop", &json!({})).await.unwrap();
    assert_eq!(answer, json!({ "ok": 1 }));
    assert_eq!(
        server.paths(),
        [
            "/u8/user/v1/getToken",
            "/gs/account/login",
            "/gs/shop",
            "/u8/user/v1/getToken",
            "/gs/account/login",
            "/gs/shop",
        ]
    );
    let shops = server.to("/gs/shop");
    assert_eq!(
        (shops[0].seqnum(), shops[0].header("secret")),
        (2, Some("secret-1"))
    );
    assert_eq!(
        (shops[1].seqnum(), shops[1].header("secret")),
        (2, Some("secret-2"))
    );
    assert_eq!(store.session(&EN).unwrap().seqnum, 2);
}

#[tokio::test]
async fn a_second_401_is_not_logged_in() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), |_, _| {
        Reply::status(401, &json!({}))
    }))
    .await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    let acct = account(&store, AccountOptions::default()).await;

    let err = acct.request(&up, "shop", &json!({})).await.unwrap_err();
    assert!(matches!(err, UpstreamError::NotLoggedIn), "{err:?}");
    assert_eq!(server.to("/gs/shop").len(), 2);
    assert_eq!(server.to("/gs/account/login").len(), 2);
}

#[tokio::test]
async fn a_refused_token_adopts_a_newer_stored_credential() {
    let server = serve(game_server(ValidTokens::new(&["tok-new"]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-old");
    store.put_durable(&EN, durable("yostar-uid", "tok-new"));
    let acct = account(&store, AccountOptions::default()).await;

    acct.request(&up, "ping", &json!({})).await.unwrap();

    let tokens: Vec<String> = server
        .to("/u8/user/v1/getToken")
        .iter()
        .map(extension_token)
        .collect();
    assert_eq!(tokens, ["tok-old", "tok-new"]);
    assert_eq!(&*store.session(&EN).unwrap().yostar_token, "tok-new");
    assert_eq!(acct.health().state, AccountState::Ok);
}

#[tokio::test]
async fn a_refused_token_with_nothing_newer_is_refused_and_alerts_once_per_outage() {
    let valid = ValidTokens::new(&[]);
    let server = serve(game_server(valid.clone(), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    store.put_durable(&EN, durable("yostar-uid", "tok-a"));
    let acct = account(
        &store,
        AccountOptions {
            secret_max_age: Duration::ZERO,
        },
    )
    .await;

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true), "{err:?}");
    assert_eq!(
        server.to("/u8/user/v1/getToken").len(),
        1,
        "same credential is not retried"
    );
    assert!(server.to("/gs/ping").is_empty());
    let health = acct.health();
    assert_eq!(health.state, AccountState::Refused);
    let since = health.refused_since.unwrap();

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(false), "same outage");
    let err = acct.probe(&up).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(false), "same outage");
    assert_eq!(acct.health().refused_since, Some(since));

    valid.allow("tok-a");
    let before = chrono::Utc::now().timestamp();
    acct.request(&up, "ping", &json!({})).await.unwrap();
    let after = chrono::Utc::now().timestamp();
    let health = acct.health();
    assert_eq!(health.state, AccountState::Ok);
    assert_eq!(health.refused_since, None);
    let ok_at = health.last_ok_at.unwrap();
    assert!((before..=after).contains(&ok_at), "{ok_at}");

    valid.revoke("tok-a");
    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(
        first_of_outage(&err),
        Some(true),
        "a new outage alerts again"
    );
    let health = acct.health();
    assert_eq!(health.state, AccountState::Refused);
    assert!(
        health.refused_since.is_some_and(|t| t >= ok_at),
        "a new outage starts fresh: {health:?}"
    );
}

#[tokio::test]
async fn concurrent_requests_send_strictly_sequential_seqnums() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), |_, _| {
        Reply::json(&json!({})).after(Duration::from_millis(15))
    }))
    .await;
    let up = Arc::new(upstream(&server.base));
    let store = store_with("tok-a");
    let acct = account(&store, AccountOptions::default()).await;

    let mut tasks = Vec::new();
    for i in 0..8 {
        let (acct, up) = (Arc::clone(&acct), Arc::clone(&up));
        tasks.push(tokio::spawn(async move {
            acct.request(&up, "pool", &json!({ "i": i })).await
        }));
    }
    for t in tasks {
        t.await.unwrap().unwrap();
    }

    let seqnums: Vec<u32> = server
        .to("/gs/pool")
        .iter()
        .map(common::Recorded::seqnum)
        .collect();
    assert_eq!(seqnums, (2..=9).collect::<Vec<_>>());
    assert_eq!(server.to("/gs/account/login").len(), 1);
    assert_eq!(store.session(&EN).unwrap().seqnum, 9);
}

#[tokio::test]
async fn a_tolerated_envelope_code_is_still_an_error_and_still_saves() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), |_, _| {
        Reply::status(
            400,
            &json!({ "statusCode": 400, "code": envelope_code::INVALID_SHOP_ID, "msg": "invalid shop id" }),
        )
    }))
    .await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    let acct = account(&store, AccountOptions::default()).await;

    let err = acct
        .request_tolerating(
            &up,
            "templateShop/getGoodList",
            &json!({ "shopId": "x" }),
            &[envelope_code::INVALID_SHOP_ID],
        )
        .await
        .unwrap_err();
    assert!(
        err.is_envelope_code(envelope_code::INVALID_SHOP_ID),
        "{err:?}"
    );
    assert_eq!(store.session(&EN).unwrap().seqnum, 2);
    assert_eq!(acct.health().state, AccountState::Ok);
}

#[tokio::test]
async fn a_maintenance_envelope_is_maintenance() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), |_, _| {
        Reply::status(
            503,
            &json!({ "statusCode": 503, "msg": "Server under Maintenance" }),
        )
    }))
    .await;
    let up = upstream(&server.base);
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert!(matches!(err, UpstreamError::Maintenance(_)), "{err:?}");
}

#[tokio::test]
async fn a_login_step_failure_is_not_a_credential_refusal() {
    let server = serve(|req: &common::Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" })),
        _ => Reply::status(503, &json!({ "msg": "服务器维护中" })),
    })
    .await;
    let up = upstream(&server.base);
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let err = acct.probe(&up).await.unwrap_err();
    assert!(matches!(err, UpstreamError::Maintenance(_)), "{err:?}");
    assert_eq!(acct.health().state, AccountState::Unknown);
}

#[tokio::test]
async fn a_missing_u8_route_is_not_a_credential_refusal() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let mut cfg = myrtle_hypergryph::NetworkConfig::new(common::device_ids());
    cfg.set_domain(Server::EN, Domain::GS, format!("{}/gs", server.base));
    up.update(|c| *c = cfg.clone());
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let err = acct.probe(&up).await.unwrap_err();
    assert!(
        matches!(
            err,
            UpstreamError::DomainNotFound {
                server: Server::EN,
                domain: Domain::U8
            }
        ),
        "{err:?}"
    );
    assert_eq!(acct.health().state, AccountState::Unknown);
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn sync_player_posts_platform_one() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    acct.sync_player(&up).await.unwrap();
    assert_eq!(
        server.to("/gs/account/syncData")[0].json(),
        json!({ "platform": 1 })
    );
}

#[tokio::test]
async fn accounts_load_what_the_store_holds_and_cn_brings_bilibili() {
    let store = Arc::new(MemorySessions::default());
    store.put_durable(
        &SessionKey::ServiceAccount { server: Server::CN },
        durable("c", "t"),
    );
    store.put_durable(
        &SessionKey::ServiceAccount {
            server: Server::Bilibili,
        },
        durable("b", "t"),
    );
    store.put_session(&EN, session_with(&durable("e", "t")));
    store.put_durable(
        &SessionKey::ServiceAccount { server: Server::JP },
        durable("", ""),
    );
    let dyn_store: Arc<dyn SessionStore> = store;

    let accounts = GameAccounts::load(
        &[Server::EN, Server::JP, Server::CN],
        &dyn_store,
        AccountOptions::default(),
    )
    .await;
    let mut servers = accounts.servers();
    servers.sort();
    assert_eq!(servers, [Server::EN, Server::CN, Server::Bilibili]);
    let codes: Vec<&str> = accounts
        .health()
        .iter()
        .map(|h| h.server.as_str())
        .collect();
    assert_eq!(codes, ["bili", "cn", "en"]);
    assert!(
        accounts
            .health()
            .iter()
            .all(|h| h.state == AccountState::Unknown)
    );
    assert!(accounts.get(Server::JP).is_none());
    assert_eq!(accounts.len(), 3);
}

fn get_token_answers(reply: Reply) -> impl Fn(&common::Recorded) -> Reply + Send + Sync {
    move |req: &common::Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => reply.clone(),
        _ => Reply::json(&json!({ "result": 0, "uid": "u", "secret": "s" })),
    }
}

#[tokio::test]
async fn a_maintenance_or_error_envelope_at_get_token_refuses_the_credential() {
    for reply in [
        Reply::status(
            503,
            &json!({ "statusCode": 503, "msg": "Server under Maintenance" }),
        ),
        Reply::status(
            400,
            &json!({ "statusCode": 400, "code": 1, "msg": "invalid token" }),
        ),
        Reply::text(502, "<html>bad gateway</html>"),
    ] {
        let server = serve(get_token_answers(reply.clone())).await;
        let up = upstream(&server.base);
        let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

        let err = acct.probe(&up).await.unwrap_err();
        assert_eq!(first_of_outage(&err), Some(true), "{}: {err:?}", reply.body);
        assert_eq!(acct.health().state, AccountState::Refused);
        assert!(server.to("/gs/account/login").is_empty());
    }
}

#[tokio::test]
async fn a_body_read_timeout_at_get_token_is_a_timeout_not_a_refusal() {
    let server = serve(get_token_answers(
        Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" }))
            .body_after(Duration::from_secs(2)),
    ))
    .await;
    let up = upstream(&server.base).with_request_timeout(Duration::from_millis(200));
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let err = acct.probe(&up).await.unwrap_err();
    assert!(matches!(err, UpstreamError::Timeout(_)), "{err:?}");
    assert_eq!(acct.health().state, AccountState::Unknown);
}

#[tokio::test]
async fn a_failing_session_read_falls_back_to_the_credential() {
    let server = serve(game_server(ValidTokens::new(&["tok-d"]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-cached");
    store.put_durable(&EN, durable("yostar-uid", "tok-d"));
    store.fail_load.store(true, Ordering::SeqCst);
    let acct = account(&store, AccountOptions::default()).await;

    acct.request(&up, "ping", &json!({})).await.unwrap();
    assert_eq!(extension_token(&server.requests()[0]), "tok-d");

    store.fail_load_durable.store(true, Ordering::SeqCst);
    let dyn_store: Arc<dyn SessionStore> = store.clone();
    let err = GameAccount::load(Server::EN, dyn_store, AccountOptions::default())
        .await
        .err()
        .unwrap();
    assert_eq!(err.kind(), "unavailable");
}

#[tokio::test]
async fn a_failing_save_does_not_fail_the_request() {
    let server = serve(game_server(ValidTokens::new(&["tok-a"]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    store.fail_save.store(true, Ordering::SeqCst);
    let acct = account(&store, AccountOptions::default()).await;

    let answer = acct.request(&up, "ping", &json!({})).await.unwrap();
    assert_eq!(answer, json!({ "ok": 1 }));
    assert!(store.saves().is_empty());
    assert_eq!(
        store.session(&EN).unwrap().seqnum,
        0,
        "the stored copy is untouched"
    );
}

#[tokio::test]
async fn an_unreadable_credential_store_on_refusal_is_a_refusal() {
    let server = serve(game_server(ValidTokens::new(&["tok-new"]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-old");
    store.put_durable(&EN, durable("yostar-uid", "tok-new"));
    let acct = account(&store, AccountOptions::default()).await;
    store.fail_load_durable.store(true, Ordering::SeqCst);

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true), "{err:?}");
    assert_eq!(server.to("/u8/user/v1/getToken").len(), 1);
}

#[tokio::test]
async fn an_adopted_credential_that_is_also_refused_is_a_refusal() {
    let server = serve(game_server(ValidTokens::new(&[]), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-old");
    store.put_durable(&EN, durable("yostar-uid", "tok-new"));
    let acct = account(&store, AccountOptions::default()).await;

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true), "{err:?}");
    let tokens: Vec<String> = server
        .to("/u8/user/v1/getToken")
        .iter()
        .map(extension_token)
        .collect();
    assert_eq!(tokens, ["tok-old", "tok-new"]);
    assert_eq!(acct.health().state, AccountState::Refused);
}

#[tokio::test]
async fn concurrent_refusals_flag_exactly_one_first_of_outage() {
    let server = serve(game_server(ValidTokens::new(&[]), ok_gs)).await;
    let up = Arc::new(upstream(&server.base));
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let mut tasks = Vec::new();
    for _ in 0..3 {
        let (acct, up) = (Arc::clone(&acct), Arc::clone(&up));
        tasks.push(tokio::spawn(async move {
            acct.request(&up, "ping", &json!({})).await
        }));
    }
    let mut firsts = Vec::new();
    for t in tasks {
        let err = t.await.unwrap().unwrap_err();
        firsts.push(first_of_outage(&err).expect("refused"));
    }
    assert_eq!(firsts.iter().filter(|f| **f).count(), 1, "{firsts:?}");
    assert_eq!(server.to("/u8/user/v1/getToken").len(), 3);
}

#[tokio::test]
async fn a_401_whose_refresh_is_refused_is_refused() {
    let valid = ValidTokens::new(&["tok-a"]);
    let revoke = valid.clone();
    let server = serve(game_server(valid, move |_, _| {
        revoke.revoke("tok-a");
        Reply::status(401, &json!({}))
    }))
    .await;
    let up = upstream(&server.base);
    let acct = account(&store_with("tok-a"), AccountOptions::default()).await;

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true), "{err:?}");
    assert_eq!(server.to("/gs/ping").len(), 1);
    assert_eq!(server.to("/u8/user/v1/getToken").len(), 2);
    assert_eq!(acct.health().state, AccountState::Refused);
}

#[tokio::test]
async fn a_stored_credential_differing_only_in_uid_is_adopted() {
    let server = serve(|req: &common::Recorded| match req.path.as_str() {
        "/u8/user/v1/getToken" => {
            let extension: serde_json::Value =
                serde_json::from_str(req.json()["extension"].as_str().unwrap()).unwrap();
            if extension["uid"] == "new-uid" {
                Reply::json(&json!({ "result": 0, "uid": "u", "token": "t" }))
            } else {
                Reply::json(&json!({ "result": 1 }))
            }
        }
        "/gs/account/login" => Reply::json(&json!({ "result": 0, "secret": "s" })),
        _ => Reply::json(&json!({ "ok": 1 })),
    })
    .await;
    let up = upstream(&server.base);
    let store = Arc::new(MemorySessions::default());
    store.put_session(&EN, session_with(&durable("old-uid", "tok")));
    store.put_durable(&EN, durable("new-uid", "tok"));
    let acct = account(&store, AccountOptions::default()).await;

    acct.request(&up, "ping", &json!({})).await.unwrap();
    assert_eq!(server.to("/u8/user/v1/getToken").len(), 2);
    assert_eq!(&*store.session(&EN).unwrap().yostar_uid, "new-uid");
}

#[tokio::test]
async fn an_outage_after_a_successful_adoption_alerts_again() {
    let valid = ValidTokens::new(&[]);
    let server = serve(game_server(valid.clone(), ok_gs)).await;
    let up = upstream(&server.base);
    let store = store_with("tok-a");
    store.put_durable(&EN, durable("yostar-uid", "tok-a"));
    let acct = account(
        &store,
        AccountOptions {
            secret_max_age: Duration::ZERO,
        },
    )
    .await;

    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true));

    store.put_durable(&EN, durable("yostar-uid", "tok-b"));
    valid.allow("tok-b");
    acct.request(&up, "ping", &json!({})).await.unwrap();

    valid.revoke("tok-b");
    let err = acct.request(&up, "ping", &json!({})).await.unwrap_err();
    assert_eq!(first_of_outage(&err), Some(true), "{err:?}");
}
