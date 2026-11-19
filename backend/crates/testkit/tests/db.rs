#![allow(clippy::unwrap_used)]

use myrtle_testkit::db::{database, unmigrated};
use sqlx::{Connection, PgConnection};

const LOCAL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

async fn exists(name: &str) -> bool {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| LOCAL.to_owned());
    let mut admin = PgConnection::connect(&url).await.unwrap();
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
        .bind(name)
        .fetch_one(&mut admin)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_test_that_panics_before_close_leaves_no_database() {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let test = tokio::spawn(async move {
        let db = database().await;
        tx.send(db.name().to_owned()).unwrap();
        db.user(1, myrtle_types::Server::EN).await;
        panic!("the test fails while its database is open");
    });
    assert!(test.await.unwrap_err().is_panic());
    let name = rx.await.unwrap();
    assert!(name.starts_with("myrtle_test_"));
    assert!(!exists(&name).await, "{name} was left behind");
}

#[tokio::test]
async fn close_drops_the_database() {
    let db = database().await;
    let name = db.name().to_owned();
    assert!(exists(&name).await);
    db.close().await;
    assert!(!exists(&name).await);
}

#[tokio::test]
async fn a_panic_before_migrating_leaves_no_database() {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let test = tokio::spawn(async move {
        let db = unmigrated().await;
        tx.send(db.name().to_owned()).unwrap();
        panic!("migration fails");
    });
    assert!(test.await.unwrap_err().is_panic());
    let name = rx.await.unwrap();
    assert!(!exists(&name).await, "{name} was left behind");
}
