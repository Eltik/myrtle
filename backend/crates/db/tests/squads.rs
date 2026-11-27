#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::repos::squads;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_saved_squad_is_replaced_in_its_slot_and_keeps_created_at(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;

    let first = squads::upsert_saved(
        &mut conn,
        user,
        2,
        "first",
        &json!([{"charId": "char_002_amiya"}, null]),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(first.support, None);

    let second = squads::upsert_saved(
        &mut conn,
        user,
        2,
        "second",
        &json!([null]),
        Some(&json!({"charId": "char_003_kalts"})),
        Some("main_01-07"),
    )
    .await
    .unwrap();
    assert_eq!(second.created_at, first.created_at);
    assert_eq!(second.name, "second");
    assert_eq!(second.members, json!([null]));
    assert_eq!(second.support, Some(json!({"charId": "char_003_kalts"})));
    assert_eq!(second.stage_id.as_deref(), Some("main_01-07"));

    squads::upsert_saved(&mut conn, user, 1, "one", &json!([]), None, None)
        .await
        .unwrap();
    let slots: Vec<i16> = squads::list_saved(&mut conn, user)
        .await
        .unwrap()
        .iter()
        .map(|s| s.slot)
        .collect();
    assert_eq!(slots, [1, 2]);
    assert!(
        squads::list_saved(&mut conn, other)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn delete_reports_whether_the_slot_held_a_squad(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    squads::upsert_saved(&mut conn, user, 1, "a", &json!([]), None, None)
        .await
        .unwrap();

    assert!(squads::delete_saved(&mut conn, user, 1).await.unwrap());
    assert!(!squads::delete_saved(&mut conn, user, 1).await.unwrap());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn game_squads_come_back_in_index_order_with_the_account_sync_time(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    sqlx::query(
        "INSERT INTO user_game_squads (user_id, squad_index, name, slots)
         VALUES ($1, 1, 'b', '[]'), ($1, 0, 'a', '[null]')",
    )
    .bind(user.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    let synced: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT updated_at FROM users WHERE id = $1")
            .bind(user.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();

    let rows = squads::list_game_squads(&mut conn, user).await.unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].squad_index, rows[0].name.as_str()), (0, "a"));
    assert_eq!(rows[0].slots, json!([null]));
    assert_eq!(rows[1].synced_at, synced);
}
