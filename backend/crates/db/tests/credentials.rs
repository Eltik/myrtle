#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::credentials;
use sqlx::PgPool;

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_stored_credential_is_replaced_and_read_back_byte_for_byte(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    assert!(credentials::load(&mut conn, user).await.unwrap().is_none());

    credentials::store(&mut conn, user, &[1, 2, 3], &[0, 255, 7])
        .await
        .unwrap();
    let created: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT created_at FROM user_game_credentials WHERE user_id = $1")
            .bind(user.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
    credentials::store(&mut conn, user, &[9; 12], b"sealed")
        .await
        .unwrap();

    let row = credentials::load(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(row.nonce, [9; 12]);
    assert_eq!(row.ciphertext, b"sealed");
    let (kept, updated): (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as(
            "SELECT created_at, updated_at FROM user_game_credentials WHERE user_id = $1",
        )
        .bind(user.0)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(kept, created);
    assert!(updated >= created);
    assert!(credentials::load(&mut conn, other).await.unwrap().is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn delete_reports_whether_a_credential_was_stored(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    credentials::store(&mut conn, user, &[1], &[2])
        .await
        .unwrap();

    assert!(credentials::delete(&mut conn, user).await.unwrap());
    assert!(!credentials::delete(&mut conn, user).await.unwrap());
    assert!(credentials::load(&mut conn, user).await.unwrap().is_none());
}
