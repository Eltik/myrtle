#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::services::accounts::sessions::Sessions;
use myrtle_db::repos::service_sessions;
use myrtle_hypergryph::{AuthSession, DurableCredential, SessionKey, SessionStoreError};
use myrtle_jobs::JobKind;
use myrtle_jobs::schedule::{self, Schedule};
use myrtle_types::{GameUid, Server};
use serde_json::json;

fn player(uid: &str) -> SessionKey {
    SessionKey::Player {
        server: Server::EN,
        uid: GameUid(uid.into()),
    }
}

fn session(uid: &str) -> AuthSession {
    AuthSession {
        yostar_uid: "y-1".into(),
        yostar_token: "y-token".into(),
        ..AuthSession::new(Some(uid), Some("secret"), Some(3), Some("token"))
    }
}

#[tokio::test]
async fn a_player_session_lives_in_the_cache_and_its_credential_in_the_table() {
    let (ctx, db) = common::context().await;
    db.user(77, Server::EN).await;
    let key = player("77");
    let store = &ctx.sessions;

    assert_eq!(store.load(&key).await.unwrap(), None);
    assert_eq!(store.load_durable(&key).await.unwrap(), None);

    let held = session("77");
    store.save(&key, &held).await.unwrap();
    assert_eq!(store.load(&key).await.unwrap(), Some(held.clone()));
    assert_eq!(
        store.load_durable(&key).await.unwrap(),
        None,
        "save is the cache half only"
    );

    let credential = DurableCredential::of(&held).unwrap();
    store.save_durable(&key, &credential).await.unwrap();
    assert_eq!(store.load_durable(&key).await.unwrap(), Some(credential));

    assert!(store.forget(&key).await.unwrap());
    assert_eq!(store.load(&key).await.unwrap(), None);
    assert_eq!(store.load_durable(&key).await.unwrap(), None);
    assert!(!store.forget(&key).await.unwrap());
    db.close().await;
}

#[tokio::test]
async fn a_credential_for_an_unknown_uid_is_refused_not_dropped() {
    let (ctx, db) = common::context().await;
    let credential = DurableCredential {
        yostar_uid: "y".into(),
        yostar_token: "t".into(),
    };
    assert!(
        ctx.sessions
            .save_durable(&player("404"), &credential)
            .await
            .is_err()
    );
    assert_eq!(
        ctx.sessions.load_durable(&player("404")).await.unwrap(),
        None
    );
    db.close().await;
}

#[tokio::test]
async fn a_service_account_session_survives_in_its_own_row() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::CN };
    let other = SessionKey::ServiceAccount { server: Server::EN };
    let store = &ctx.sessions;

    assert_eq!(store.load(&key).await.unwrap(), None);
    let held = session("9001");
    store.save(&key, &held).await.unwrap();
    assert_eq!(store.load(&key).await.unwrap(), Some(held.clone()));
    assert_eq!(
        store.load(&other).await.unwrap(),
        None,
        "one row per server"
    );
    assert_eq!(
        store.load_durable(&key).await.unwrap(),
        DurableCredential::of(&held)
    );

    let renewed = DurableCredential {
        yostar_uid: "y-2".into(),
        yostar_token: "y-token-2".into(),
    };
    store.save_durable(&key, &renewed).await.unwrap();
    let after = store.load(&key).await.unwrap().unwrap();
    assert_eq!(after.yostar_token.as_ref(), "y-token-2");
    assert_eq!(after.secret, held.secret, "the rest of the session is kept");

    assert!(store.forget(&key).await.unwrap());
    assert_eq!(store.load(&key).await.unwrap(), None);
    assert!(!store.forget(&key).await.unwrap());
    db.close().await;
}

#[tokio::test]
async fn the_stored_service_session_is_sealed() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::EN };
    ctx.sessions.save(&key, &session("9001")).await.unwrap();
    let row = service_sessions::get(&mut ctx.db.conn().await.unwrap(), Server::EN)
        .await
        .unwrap()
        .unwrap();
    let stored = String::from_utf8_lossy(&row.ciphertext);
    assert!(
        !stored.contains("y-token") && !stored.contains("secret") && !stored.contains("9001"),
        "{stored}"
    );
    db.close().await;
}

#[tokio::test]
async fn a_service_session_leaves_the_alert_schedule_memory_alone() {
    let (ctx, db) = common::context().await;
    let mut conn = ctx.db.conn().await.unwrap();
    let alert = Schedule {
        server: Some(Server::EN),
        ..Schedule::new(JobKind::ServiceAccountAlert, Duration::from_hours(24))
    };
    schedule::sync(&mut conn, &[alert], Duration::ZERO)
        .await
        .unwrap();
    let memory = json!({ "alerted": true });
    schedule::remember(
        &mut conn,
        JobKind::ServiceAccountAlert,
        Some(Server::EN),
        &memory,
    )
    .await
    .unwrap();

    let key = SessionKey::ServiceAccount { server: Server::EN };
    ctx.sessions.save(&key, &session("9001")).await.unwrap();
    assert!(ctx.sessions.forget(&key).await.unwrap());
    let kept = schedule::memory(&mut conn, JobKind::ServiceAccountAlert, Some(Server::EN))
        .await
        .unwrap();
    assert_eq!(kept, memory);
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn a_renewed_service_credential_with_no_session_starts_one() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::JP };
    let credential = DurableCredential {
        yostar_uid: "y-3".into(),
        yostar_token: "y-token-3".into(),
    };
    ctx.sessions.save_durable(&key, &credential).await.unwrap();
    assert_eq!(
        ctx.sessions.load_durable(&key).await.unwrap(),
        Some(credential)
    );
    db.close().await;
}

#[tokio::test]
async fn a_service_session_that_will_not_open_is_corrupt_and_still_forgettable() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::KR };
    service_sessions::upsert(
        &mut ctx.db.conn().await.unwrap(),
        Server::KR,
        &[0; 24],
        &[1],
    )
    .await
    .unwrap();
    assert!(matches!(
        ctx.sessions.load(&key).await,
        Err(SessionStoreError::Corrupt(_))
    ));
    assert!(ctx.sessions.forget(&key).await.unwrap());
    assert_eq!(ctx.sessions.load(&key).await.unwrap(), None);
    db.close().await;
}

#[tokio::test]
async fn a_sealed_session_that_does_not_parse_names_no_field_value() {
    let (ctx, db) = common::context().await;
    let owner = uuid::Uuid::from_u128(Server::KR.index() as u128 + 1);
    let (nonce, ciphertext) = ctx
        .secrets
        .credentials
        .seal(owner, br#"{"uid": "1", "seqnum": "leaked-token"}"#)
        .unwrap();
    service_sessions::upsert(
        &mut ctx.db.conn().await.unwrap(),
        Server::KR,
        &nonce,
        &ciphertext,
    )
    .await
    .unwrap();
    let key = SessionKey::ServiceAccount { server: Server::KR };
    let Err(SessionStoreError::Corrupt(detail)) = ctx.sessions.load(&key).await else {
        panic!("a sealed non-session opened");
    };
    assert!(detail.contains("line 1"), "{detail}");
    assert!(!detail.contains("leaked-token"), "{detail}");
    db.close().await;
}

#[tokio::test]
async fn a_credential_that_will_not_open_reads_as_absent() {
    let (ctx, db) = common::context().await;
    let user = db.user(78, Server::EN).await;
    myrtle_db::repos::credentials::store(
        &mut ctx.db.conn().await.unwrap(),
        user,
        &[0; 24],
        &[1, 2, 3],
    )
    .await
    .unwrap();
    assert_eq!(
        ctx.sessions.load_durable(&player("78")).await.unwrap(),
        None
    );
    db.close().await;
}

fn seeded(token: &str) -> AuthSession {
    AuthSession {
        yostar_uid: "y-seed".into(),
        yostar_token: token.into(),
        ..AuthSession::new(Some("9100"), Some("seed-secret"), Some(1), Some("token"))
    }
}

fn sessions(ctx: &myrtle_app::AppContext) -> Sessions {
    Sessions::new(
        ctx.db.clone(),
        ctx.cache.clone(),
        std::sync::Arc::clone(&ctx.secrets.credentials),
    )
}

#[tokio::test]
async fn a_seed_replaces_the_whole_session_and_a_stale_persist_keeps_it() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::EN };
    let mut running = session("9001");
    ctx.sessions.save(&key, &running).await.unwrap();

    let seed = seeded("y-new");
    assert!(sessions(&ctx).seed(Server::EN, &seed, true).await.unwrap());
    running.seqnum = 7;
    running.secret = "refreshed-before-the-seed".into();
    ctx.sessions.save(&key, &running).await.unwrap();
    assert_eq!(
        ctx.sessions.load(&key).await.unwrap(),
        Some(seed.clone()),
        "the other login's persist pairs nothing with the seed"
    );

    let mut adopted = seed;
    adopted.seqnum = 2;
    adopted.secret = "after-the-seed".into();
    ctx.sessions.save(&key, &adopted).await.unwrap();
    assert_eq!(ctx.sessions.load(&key).await.unwrap(), Some(adopted));
    db.close().await;
}

#[tokio::test]
async fn a_persist_waits_on_the_row_lock_and_sees_the_seed_that_held_it() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::EN };
    let store = sessions(&ctx);
    let seed = seeded("y-new");
    store.seed(Server::EN, &seed, true).await.unwrap();
    let sealed_seed = service_sessions::get(&mut ctx.db.conn().await.unwrap(), Server::EN)
        .await
        .unwrap()
        .unwrap();
    let running = session("9001");
    store.seed(Server::EN, &running, true).await.unwrap();

    let mut tx = ctx.db.begin().await.unwrap();
    service_sessions::lock(&mut tx, Server::EN).await.unwrap();
    let persist = {
        let (store, key, mut live) = (std::sync::Arc::clone(&ctx.sessions), key.clone(), running);
        tokio::spawn(async move {
            live.secret = "refreshed".into();
            store.save(&key, &live).await.unwrap();
        })
    };
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        !persist.is_finished(),
        "the persist went around the row lock"
    );
    service_sessions::upsert(
        &mut tx,
        Server::EN,
        &sealed_seed.nonce,
        &sealed_seed.ciphertext,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    persist.await.unwrap();
    assert_eq!(ctx.sessions.load(&key).await.unwrap(), Some(seed));
    db.close().await;
}

#[tokio::test]
async fn concurrent_persists_and_renewals_lose_no_credential() {
    let (ctx, db) = common::context().await;
    let key = SessionKey::ServiceAccount { server: Server::EN };
    ctx.sessions.save(&key, &session("9001")).await.unwrap();
    let persists = {
        let (store, key) = (std::sync::Arc::clone(&ctx.sessions), key.clone());
        tokio::spawn(async move {
            for n in 0..25 {
                let mut live = session("9001");
                live.seqnum = n;
                store.save(&key, &live).await.unwrap();
            }
        })
    };
    let renewals = {
        let (store, key) = (std::sync::Arc::clone(&ctx.sessions), key.clone());
        tokio::spawn(async move {
            for n in 0..25 {
                let credential = DurableCredential {
                    yostar_uid: "y-2".into(),
                    yostar_token: format!("y-token-{n}"),
                };
                store.save_durable(&key, &credential).await.unwrap();
            }
        })
    };
    persists.await.unwrap();
    renewals.await.unwrap();
    let stored = ctx.sessions.load(&key).await.unwrap().unwrap();
    assert_eq!(
        (stored.yostar_uid.as_ref(), stored.yostar_token.as_ref()),
        ("y-2", "y-token-24")
    );
    db.close().await;
}

#[tokio::test]
async fn a_seed_without_replace_fills_only_an_empty_row() {
    let (ctx, db) = common::context().await;
    let store = sessions(&ctx);
    assert!(
        store
            .seed(Server::JP, &seeded("first"), false)
            .await
            .unwrap()
    );
    assert!(
        !store
            .seed(Server::JP, &seeded("second"), false)
            .await
            .unwrap()
    );
    let key = SessionKey::ServiceAccount { server: Server::JP };
    let stored = ctx.sessions.load(&key).await.unwrap().unwrap();
    assert_eq!(stored.yostar_token.as_ref(), "first");
    db.close().await;
}
