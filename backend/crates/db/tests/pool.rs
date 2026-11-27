//! `Db::connect` against the database `#[sqlx::test]` hands each test.

#![allow(clippy::unwrap_used)]

use std::time::Duration;

use myrtle_db::{Db, DbError, PoolOptions};
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

fn options(connect: &PgConnectOptions) -> PoolOptions {
    let mut opts = PoolOptions::new(connect.to_url_lossy());
    opts.min_connections = 0;
    opts.max_connections = 1;
    opts
}

async fn show(db: &Db, setting: &str) -> String {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SHOW {setting}")))
        .fetch_one(&mut *db.conn().await.unwrap())
        .await
        .unwrap()
}

#[sqlx::test(migrations = false)]
async fn unset_timeouts_leave_the_server_defaults(_: PgPoolOptions, connect: PgConnectOptions) {
    let db = Db::connect(&options(&connect)).await.unwrap();
    db.ping().await.unwrap();
    assert_eq!(show(&db, "statement_timeout").await, "0");
    assert_eq!(show(&db, "lock_timeout").await, "0");
    assert_eq!(show(&db, "idle_in_transaction_session_timeout").await, "0");
    db.close().await;
}

#[sqlx::test(migrations = false)]
async fn session_timeouts_reach_every_connection(_: PgPoolOptions, connect: PgConnectOptions) {
    let mut opts = options(&connect);
    opts.statement_timeout = Some(Duration::from_millis(1500));
    opts.lock_timeout = Some(Duration::from_secs(2));
    opts.idle_in_transaction_timeout = Some(Duration::from_micros(500));
    let db = Db::connect(&opts).await.unwrap();
    assert_eq!(show(&db, "statement_timeout").await, "1500ms");
    assert_eq!(show(&db, "lock_timeout").await, "2s");
    assert_eq!(
        show(&db, "idle_in_transaction_session_timeout").await,
        "1ms"
    );
    db.close().await;
}

#[sqlx::test(migrations = false)]
async fn an_exhausted_pool_times_out(_: PgPoolOptions, connect: PgConnectOptions) {
    let mut opts = options(&connect);
    opts.acquire_timeout = Duration::from_millis(100);
    let db = Db::connect(&opts).await.unwrap();
    let _held = db.conn().await.unwrap();
    assert!(matches!(db.conn().await, Err(DbError::PoolTimedOut)));
}

#[sqlx::test(migrations = false)]
async fn a_dropped_transaction_rolls_back(_: PgPoolOptions, connect: PgConnectOptions) {
    let db = Db::connect(&options(&connect)).await.unwrap();
    let mut tx = db.begin().await.unwrap();
    sqlx::query("CREATE TABLE pool_test_dropped (id int)")
        .execute(&mut *tx)
        .await
        .unwrap();
    drop(tx);
    let left: bool = sqlx::query_scalar("SELECT to_regclass('pool_test_dropped') IS NOT NULL")
        .fetch_one(&mut *db.conn().await.unwrap())
        .await
        .unwrap();
    assert!(!left);
}
