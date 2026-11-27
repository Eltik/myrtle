#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::progress;
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

async fn put_sandbox(conn: &mut Conn, user: UserId, progress: &Value) {
    sqlx::query("INSERT INTO user_sandbox_progress (user_id, progress) VALUES ($1, $2)")
        .bind(user.0)
        .bind(progress)
        .execute(conn)
        .await
        .unwrap();
}

async fn put_stages(conn: &mut Conn, user: UserId, stages: &Value) {
    sqlx::query("INSERT INTO user_stage_progress (user_id, stages) VALUES ($1, $2)")
        .bind(user.0)
        .bind(stages)
        .execute(conn)
        .await
        .unwrap();
}

async fn put_enemies(conn: &mut Conn, user: UserId, enemies: &Value) {
    sqlx::query("INSERT INTO user_enemy_progress (user_id, enemies) VALUES ($1, $2)")
        .bind(user.0)
        .bind(enemies)
        .execute(conn)
        .await
        .unwrap();
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn building_and_sandbox_read_the_stored_blob_or_none(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    sqlx::query("INSERT INTO user_building (user_id, data) VALUES ($1, $2)")
        .bind(user.0)
        .bind(json!({"rooms": 3}))
        .execute(&mut *conn)
        .await
        .unwrap();
    put_sandbox(&mut conn, user, &json!({"template": {}})).await;

    assert_eq!(
        progress::building(&mut conn, user).await.unwrap(),
        Some(json!({"rooms": 3}))
    );
    assert_eq!(
        progress::sandbox(&mut conn, user).await.unwrap(),
        Some(json!({"template": {}}))
    );
    assert_eq!(progress::building(&mut conn, other).await.unwrap(), None);
    assert_eq!(progress::sandbox(&mut conn, other).await.unwrap(), None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn medals_and_roguelike_themes_belong_to_their_account(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    sqlx::query(
        "INSERT INTO user_medals (user_id, medal_id, val, first_ts, reach_ts) VALUES
         ($1, 'medal_a', '[1, 2]', 10, 20), ($2, 'medal_b', NULL, NULL, NULL)",
    )
    .bind(user.0)
    .bind(other.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO user_roguelike_progress (user_id, theme_id, progress) VALUES
         ($1, 'rogue_1', '{\"bp\": 1}'), ($2, 'rogue_2', '{}')",
    )
    .bind(user.0)
    .bind(other.0)
    .execute(&mut *conn)
    .await
    .unwrap();

    let medals = progress::medals(&mut conn, user).await.unwrap();
    assert_eq!(medals.len(), 1);
    assert_eq!(medals[0].medal_id, "medal_a");
    assert_eq!(medals[0].val, Some(json!([1, 2])));
    assert_eq!(
        (medals[0].first_ts, medals[0].reach_ts),
        (Some(10), Some(20))
    );
    let other_medals = progress::medals(&mut conn, other).await.unwrap();
    assert_eq!(other_medals[0].val, None);

    let themes = progress::roguelike(&mut conn, user).await.unwrap();
    assert_eq!(themes.len(), 1);
    assert_eq!(themes[0].theme_id, "rogue_1");
    assert_eq!(themes[0].progress, json!({"bp": 1}));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_stored_sandbox_topic_creates_its_template_and_keeps_the_rest(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let flat = common::user(&mut conn, 2).await;
    put_sandbox(
        &mut conn,
        user,
        &json!({"template": {"sandbox_1": {"main": {"day": 3}}}}),
    )
    .await;
    put_sandbox(&mut conn, flat, &json!({"template": "unloaded"})).await;

    progress::store_sandbox_topic(&mut conn, user, "sandbox_1", "rift", &json!({"day": 1}))
        .await
        .unwrap();
    progress::store_sandbox_topic(&mut conn, user, "sandbox_2", "main", &json!({"day": 9}))
        .await
        .unwrap();
    progress::store_sandbox_topic(&mut conn, flat, "sandbox_1", "main", &json!({"day": 1}))
        .await
        .unwrap();

    assert_eq!(
        progress::sandbox(&mut conn, user).await.unwrap(),
        Some(json!({"template": {
            "sandbox_1": {"main": {"day": 3}, "rift": {"day": 1}},
            "sandbox_2": {"main": {"day": 9}}
        }}))
    );
    assert_eq!(
        progress::sandbox(&mut conn, flat).await.unwrap(),
        Some(json!({"template": "unloaded"}))
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn known_stage_ids_span_every_account_on_the_same_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let peer = common::user(&mut conn, 2).await;
    let elsewhere = common::user_on(&mut conn, 3, Server::CN).await;
    put_stages(&mut conn, user, &json!({"main_00-01": {"state": 3}})).await;
    put_stages(
        &mut conn,
        peer,
        &json!({"main_00-01": {"state": 2}, "act1_01": {"state": 0}}),
    )
    .await;
    put_stages(&mut conn, elsewhere, &json!({"cn_only": {"state": 3}})).await;

    let mut ids: Vec<String> = progress::known_stage_ids_for_server(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .collect();
    ids.sort();
    assert_eq!(ids, ["act1_01", "main_00-01"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn stage_clears_read_each_record_with_its_defaults(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let missing = common::user(&mut conn, 2).await;
    let odd = common::user(&mut conn, 3).await;
    put_stages(
        &mut conn,
        user,
        &json!({
            "main_00-01": {"state": 3, "completeTimes": 4, "practiceTimes": 1},
            "main_00-02": {"state": 1, "stateMax": 3, "inferred": true},
            "main_00-03": {"inferred": "mission"},
            "main_00-04": {"state": 2, "inferred": false}
        }),
    )
    .await;
    put_stages(&mut conn, odd, &json!([1, 2])).await;
    let synced: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT updated_at FROM users WHERE id = $1")
            .bind(user.0)
            .fetch_one(&mut *conn)
            .await
            .unwrap();

    let read = progress::stage_clears(&mut conn, user).await.unwrap();
    assert_eq!(read.last_synced_ts, Some(synced.timestamp()));
    assert_eq!(read.clears.len(), 4);
    let full = read.clears["main_00-01"];
    assert_eq!((full.state, full.state_max, full.inferred), (3, 3, false));
    assert_eq!((full.complete_times, full.practice_times), (4, 1));
    let capped = read.clears["main_00-02"];
    assert_eq!(
        (capped.state, capped.state_max, capped.inferred),
        (1, 3, true)
    );
    let tagged = read.clears["main_00-03"];
    assert_eq!(
        (tagged.state, tagged.complete_times, tagged.inferred),
        (0, 0, true)
    );
    assert!(!read.clears["main_00-04"].inferred);

    let none = progress::stage_clears(&mut conn, missing).await.unwrap();
    assert!(none.clears.is_empty());
    assert_eq!(none.last_synced_ts, None);

    let not_a_map = progress::stage_clears(&mut conn, odd).await.unwrap();
    assert!(not_a_map.clears.is_empty());
    assert!(not_a_map.last_synced_ts.is_some());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn stage_numbers_past_their_field_range_saturate(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    put_stages(
        &mut conn,
        user,
        &json!({
            "high": {"state": 70_000, "completeTimes": 3_000_000_000_i64, "practiceTimes": 1},
            "low": {"state": 1, "stateMax": -70_000, "practiceTimes": -3_000_000_000_i64}
        }),
    )
    .await;

    let read = progress::stage_clears(&mut conn, user).await.unwrap();
    let high = read.clears["high"];
    assert_eq!((high.state, high.state_max), (i16::MAX, i16::MAX));
    assert_eq!((high.complete_times, high.practice_times), (i32::MAX, 1));
    let low = read.clears["low"];
    assert_eq!((low.state, low.state_max), (1, i16::MIN));
    assert_eq!((low.complete_times, low.practice_times), (0, i32::MIN));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn encountered_enemies_and_their_community_average(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let empty = progress::community_average_encountered(&mut conn)
        .await
        .unwrap();
    assert_eq!(empty.users, 0);
    assert!(empty.average.abs() < 1e-9);

    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let never = common::user(&mut conn, 3).await;
    put_enemies(
        &mut conn,
        user,
        &json!({"enemy_1007_slime": 1, "enemy_1000_gopro": 1, "enemy_1001_bigbo": 0}),
    )
    .await;
    put_enemies(&mut conn, other, &json!({"enemy_1007_slime": 1})).await;

    let mut seen = progress::encountered_enemies(&mut conn, user)
        .await
        .unwrap();
    seen.sort();
    assert_eq!(
        seen,
        ["enemy_1000_gopro", "enemy_1001_bigbo", "enemy_1007_slime"]
    );
    assert!(
        progress::encountered_enemies(&mut conn, never)
            .await
            .unwrap()
            .is_empty()
    );

    let average = progress::community_average_encountered(&mut conn)
        .await
        .unwrap();
    assert_eq!(average.users, 2);
    assert!((average.average - 2.0).abs() < 1e-9);
}
