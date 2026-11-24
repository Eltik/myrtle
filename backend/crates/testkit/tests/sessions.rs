#![allow(clippy::unwrap_used)]

use std::sync::atomic::Ordering;

use myrtle_hypergryph::{AuthSession, DurableCredential, SessionKey, SessionStore};
use myrtle_testkit::sessions::MemorySessions;
use myrtle_types::Server;

const EN: SessionKey = SessionKey::ServiceAccount { server: Server::EN };
const CN: SessionKey = SessionKey::ServiceAccount { server: Server::CN };

fn durable(token: &str) -> DurableCredential {
    DurableCredential {
        yostar_uid: "uid".into(),
        yostar_token: token.into(),
    }
}

#[tokio::test]
async fn a_saved_session_loads_under_its_own_key_only() {
    let store = MemorySessions::default();
    let session = AuthSession::from_durable("1", &durable("tok"));
    store.save(&EN, &session).await.unwrap();
    assert_eq!(store.load(&EN).await.unwrap(), Some(session.clone()));
    assert_eq!(store.load(&CN).await.unwrap(), None);
    assert_eq!(store.saves(), vec![session]);
}

#[tokio::test]
async fn forget_drops_the_session_and_the_credential() {
    let store = MemorySessions::default();
    store.save_durable(&EN, &durable("tok")).await.unwrap();
    assert_eq!(store.load_durable(&EN).await.unwrap(), Some(durable("tok")));
    assert!(store.forget(&EN).await.unwrap());
    assert_eq!(store.load_durable(&EN).await.unwrap(), None);
    assert!(!store.forget(&EN).await.unwrap());
}

#[tokio::test]
async fn an_injected_failure_stores_nothing() {
    let store = MemorySessions::default();
    store.fail_save.store(true, Ordering::SeqCst);
    let failed = store.save(&EN, &AuthSession::default()).await.unwrap_err();
    assert_eq!(failed.kind(), "unavailable");
    assert!(store.saves().is_empty());
    assert_eq!(store.session(&EN), None);

    store.fail_load.store(true, Ordering::SeqCst);
    store.fail_load_durable.store(true, Ordering::SeqCst);
    assert!(store.load(&EN).await.is_err());
    assert!(store.load_durable(&EN).await.is_err());
}
