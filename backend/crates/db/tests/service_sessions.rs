#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::service_sessions;
use myrtle_types::Server;
use sqlx::PgPool;

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn one_sealed_session_per_server_replaced_in_place(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert!(
        service_sessions::get(&mut conn, Server::CN)
            .await
            .unwrap()
            .is_none()
    );

    service_sessions::upsert(&mut conn, Server::CN, &[1, 2], &[3])
        .await
        .unwrap();
    service_sessions::upsert(&mut conn, Server::CN, &[9; 24], b"sealed")
        .await
        .unwrap();
    service_sessions::upsert(&mut conn, Server::EN, &[4], &[5])
        .await
        .unwrap();

    let row = service_sessions::get(&mut conn, Server::CN)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.nonce, [9; 24]);
    assert_eq!(row.ciphertext, b"sealed");
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM service_account_sessions")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(rows, 2);
    assert!(
        service_sessions::get(&mut conn, Server::JP)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_upsert_moves_updated_at(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    service_sessions::upsert(&mut conn, Server::EN, &[1], &[2])
        .await
        .unwrap();
    sqlx::query("UPDATE service_account_sessions SET updated_at = now() - interval '1 day'")
        .execute(&mut *conn)
        .await
        .unwrap();
    service_sessions::upsert(&mut conn, Server::EN, &[1], &[3])
        .await
        .unwrap();
    let fresh: bool = sqlx::query_scalar(
        "SELECT updated_at > now() - interval '1 minute' FROM service_account_sessions",
    )
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert!(fresh);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn lock_reads_the_row_inside_a_transaction(pool: PgPool) {
    let db = common::db(pool).await;
    service_sessions::upsert(&mut db.conn().await.unwrap(), Server::KR, &[7], &[8])
        .await
        .unwrap();
    let mut tx = db.begin().await.unwrap();
    let row = service_sessions::lock(&mut tx, Server::KR)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.ciphertext, [8]);
    assert!(
        service_sessions::lock(&mut tx, Server::JP)
            .await
            .unwrap()
            .is_none()
    );
    tx.commit().await.unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn delete_reports_whether_a_session_was_stored(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    service_sessions::upsert(&mut conn, Server::Bilibili, &[1], &[2])
        .await
        .unwrap();

    assert!(
        service_sessions::delete(&mut conn, Server::Bilibili)
            .await
            .unwrap()
    );
    assert!(
        !service_sessions::delete(&mut conn, Server::Bilibili)
            .await
            .unwrap()
    );
    assert!(
        service_sessions::get(&mut conn, Server::Bilibili)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn insert_new_never_replaces_a_stored_session(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    assert!(
        service_sessions::insert_new(&mut conn, Server::JP, &[1], b"first")
            .await
            .unwrap()
    );
    assert!(
        !service_sessions::insert_new(&mut conn, Server::JP, &[2], b"second")
            .await
            .unwrap()
    );
    let row = service_sessions::get(&mut conn, Server::JP)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(row.ciphertext, b"first");
}
