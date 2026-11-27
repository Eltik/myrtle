#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::services::accounts::jwt::verify_token;
use myrtle_app::services::accounts::login::{
    bilibili_sms_key, complete_login, login_bilibili_sms, login_cn, parse_server,
};
use myrtle_app::services::accounts::sessions::{portal_session_key, session_key};
use myrtle_db::repos::users;
use myrtle_hypergryph::login::yostar::AccountPortalSession;
use myrtle_hypergryph::{AuthSession, DurableCredential, SessionKey};
use myrtle_types::{GameUid, Server, UserId};
use serde_json::json;

fn session(uid: &str, durable: bool) -> AuthSession {
    let mut session = AuthSession::new(Some(uid), Some("secret"), Some(1), Some("token"));
    if durable {
        session.yostar_uid = "y-uid".into();
        session.yostar_token = "y-token".into();
    }
    session
}

fn key(server: Server, uid: &str) -> SessionKey {
    SessionKey::Player {
        server,
        uid: GameUid(uid.into()),
    }
}

#[test]
fn parses_every_server_code() {
    let cases = [
        ("en", Server::EN),
        ("jp", Server::JP),
        ("kr", Server::KR),
        ("cn", Server::CN),
        ("bili", Server::Bilibili),
        ("tw", Server::TW),
    ];
    for (code, server) in cases {
        assert_eq!(parse_server(code).ok(), Some(server), "{code}");
    }
}

#[test]
fn codes_are_case_sensitive_and_unknown_ones_are_bad_requests() {
    for code in ["EN", "Bilibili", "bilibili", "us", ""] {
        match parse_server(code) {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, format!("unknown server: {code}")),
            other => panic!("{code}: {other:?}"),
        }
    }
}

#[tokio::test]
async fn a_login_creates_the_account_caches_the_session_and_issues_a_token() {
    let (ctx, db) = common::context().await;
    let held = session("5001", true);
    let portal = AccountPortalSession {
        yssid: "s".into(),
        yssid_sig: "sig".into(),
    };

    let response = complete_login(&ctx, Server::JP, &held, Some(&portal), true)
        .await
        .unwrap();
    assert_eq!(
        (response.uid.as_str(), response.server.as_str()),
        ("5001", "jp")
    );

    let row = users::find(&mut ctx.db.conn().await.unwrap(), "5001", Server::JP)
        .await
        .unwrap()
        .unwrap();
    let claims = verify_token(common::JWT_SECRET, &response.token).unwrap();
    assert_eq!(claims.user_id().unwrap(), UserId(row.id));
    assert_eq!(
        (
            claims.uid.as_str(),
            claims.server.as_str(),
            claims.role.as_str()
        ),
        ("5001", "jp", "user")
    );

    let cached: AuthSession = ctx.cache.get_json(&session_key("5001")).await.unwrap();
    assert_eq!(cached, held);
    let cached: AccountPortalSession = ctx
        .cache
        .get_json(&portal_session_key("5001"))
        .await
        .unwrap();
    assert_eq!(cached, portal);
    assert_eq!(
        ctx.sessions
            .load_durable(&key(Server::JP, "5001"))
            .await
            .unwrap(),
        DurableCredential::of(&held)
    );
    db.close().await;
}

#[tokio::test]
async fn repeat_logins_reuse_one_account_and_carry_its_stored_role() {
    let (ctx, db) = common::context().await;
    let first = complete_login(&ctx, Server::EN, &session("6001", false), None, true)
        .await
        .unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    let id = users::find(&mut conn, "6001", Server::EN)
        .await
        .unwrap()
        .unwrap()
        .id;
    users::set_role(&mut conn, UserId(id), "tier_list_admin")
        .await
        .unwrap();

    let second = complete_login(&ctx, Server::EN, &session("6001", false), None, true)
        .await
        .unwrap();
    let a = verify_token(common::JWT_SECRET, &first.token).unwrap();
    let b = verify_token(common::JWT_SECRET, &second.token).unwrap();
    assert_eq!(a.sub, b.sub);
    assert_eq!(b.role, "tier_list_admin");
    assert_eq!(
        users::count_by_uid(&mut conn, "6001", None).await.unwrap(),
        1
    );

    complete_login(&ctx, Server::KR, &session("6001", false), None, true)
        .await
        .unwrap();
    assert_eq!(
        users::count_by_uid(&mut conn, "6001", None).await.unwrap(),
        2,
        "one account per server"
    );
    assert_eq!(ctx.cache.get(&portal_session_key("6001")).await, None);
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn opting_out_stores_nothing_and_withdraws_an_earlier_credential() {
    let (ctx, db) = common::context().await;
    let player = key(Server::EN, "7001");
    complete_login(&ctx, Server::EN, &session("7001", true), None, true)
        .await
        .unwrap();
    assert!(ctx.sessions.load_durable(&player).await.unwrap().is_some());

    complete_login(&ctx, Server::EN, &session("7001", true), None, false)
        .await
        .unwrap();
    assert_eq!(ctx.sessions.load_durable(&player).await.unwrap(), None);
    assert!(
        ctx.cache.get(&session_key("7001")).await.is_some(),
        "the cached session still works for its hour"
    );

    complete_login(&ctx, Server::EN, &session("7001", false), None, true)
        .await
        .unwrap();
    assert_eq!(
        ctx.sessions.load_durable(&player).await.unwrap(),
        None,
        "a session with no durable half stores nothing"
    );
    db.close().await;
}

#[tokio::test]
async fn a_bilibili_sms_login_needs_a_live_challenge() {
    let (ctx, db) = common::context().await;
    let refused = |r: Result<_, AppError>| matches!(r, Err(AppError::BadRequest(m)) if m == "no live code for this number; send a new one");
    assert!(refused(
        login_bilibili_sms(&ctx, "13800000000", "1234", true).await
    ));

    let expired = json!({
        "phone": "13800000000", "country_code": "86", "captcha_key": "k",
        "device_id": "d", "bd_id": "b", "issued_at": 1,
    });
    ctx.cache
        .set_json(
            &bilibili_sms_key("13800000000"),
            &expired,
            std::time::Duration::from_mins(5),
        )
        .await;
    assert!(refused(
        login_bilibili_sms(&ctx, "13800000000", "1234", true).await
    ));
    assert_eq!(bilibili_sms_key("138"), "bilibili_sms:138");
    db.close().await;
}

#[tokio::test]
async fn a_cn_login_needs_a_password_or_a_code() {
    let (ctx, db) = common::context().await;
    let result = login_cn(&ctx, "13800000000", None, None, true).await;
    assert!(
        matches!(result, Err(AppError::BadRequest(m)) if m == "provide either password or code")
    );
    db.close().await;
}
