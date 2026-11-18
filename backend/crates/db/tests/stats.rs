#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::stats::{self, OperatorChoiceRow, OperatorLevelRow};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

/// `share` None leaves the account without a settings row
async fn account(conn: &mut Conn, n: u32, server: Server, share: Option<bool>) -> UserId {
    let user = common::user_on(conn, n, server).await;
    if let Some(share) = share {
        sqlx::query("INSERT INTO user_settings (user_id, share_stats) VALUES ($1, $2)")
            .bind(user.0)
            .bind(share)
            .execute(conn)
            .await
            .unwrap();
    }
    user
}

async fn operator(
    conn: &mut Conn,
    user: UserId,
    operator_id: &str,
    elite: i16,
    default_skill: i16,
    current_equip: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO user_operators (user_id, operator_id, elite, default_skill, current_equip)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.0)
    .bind(operator_id)
    .bind(elite)
    .bind(default_skill)
    .bind(current_equip)
    .execute(conn)
    .await
    .unwrap();
}

async fn skill(conn: &mut Conn, user: UserId, skill_index: i16, specialize_level: i16) {
    sqlx::query(
        "INSERT INTO user_operator_skills (user_id, operator_id, skill_index, specialize_level)
         VALUES ($1, 'op_a', $2, $3)",
    )
    .bind(user.0)
    .bind(skill_index)
    .bind(specialize_level)
    .execute(conn)
    .await
    .unwrap();
}

async fn module(conn: &mut Conn, user: UserId, module_level: i16, locked: bool) {
    sqlx::query(
        "INSERT INTO user_operator_modules (user_id, operator_id, module_id, module_level, locked)
         VALUES ($1, 'op_a', 'uniequip_002_a', $2, $3)",
    )
    .bind(user.0)
    .bind(module_level)
    .bind(locked)
    .execute(conn)
    .await
    .unwrap();
}

async fn medal(
    conn: &mut Conn,
    user: UserId,
    medal_id: &str,
    reach_ts: i64,
    first_ts: i64,
    val: Value,
) {
    sqlx::query(
        "INSERT INTO user_medals (user_id, medal_id, val, first_ts, reach_ts)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user.0)
    .bind(medal_id)
    .bind(val)
    .bind(first_ts)
    .bind(reach_ts)
    .execute(conn)
    .await
    .unwrap();
}

/// Four sharing EN accounts (one only at E1), one EN account not sharing, one without
/// settings, one sharing JP account; returns the fourth sharing EN account
async fn roster_fixture(conn: &mut Conn) -> UserId {
    let u1 = account(conn, 1, Server::EN, Some(true)).await;
    let u2 = account(conn, 2, Server::EN, Some(true)).await;
    let u3 = account(conn, 3, Server::EN, Some(true)).await;
    let u4 = account(conn, 4, Server::EN, Some(false)).await;
    let u5 = account(conn, 5, Server::EN, None).await;
    let u6 = account(conn, 6, Server::JP, Some(true)).await;
    let u7 = account(conn, 7, Server::EN, Some(true)).await;

    operator(conn, u1, "op_a", 2, 1, Some("uniequip_002_a")).await;
    operator(conn, u1, "op_b", 0, 0, None).await;
    operator(conn, u2, "op_a", 2, 0, Some("uniequip_001_a")).await;
    operator(conn, u2, "op_c", 2, -1, Some("uniequip_003_c")).await;
    operator(conn, u3, "op_a", 1, 2, Some("uniequip_002_a")).await;
    operator(conn, u4, "op_a", 2, 0, Some("uniequip_002_a")).await;
    operator(conn, u5, "op_a", 2, 0, Some("uniequip_002_a")).await;
    operator(conn, u6, "op_a", 2, 2, None).await;
    operator(conn, u7, "op_a", 2, 1, Some("uniequip_003_a")).await;

    skill(conn, u1, 0, 3).await;
    skill(conn, u1, 1, 0).await;
    skill(conn, u2, 0, 1).await;
    skill(conn, u3, 0, 0).await;
    skill(conn, u4, 0, 3).await;
    skill(conn, u7, 0, 3).await;

    module(conn, u1, 2, false).await;
    module(conn, u2, 1, true).await;
    module(conn, u3, 3, false).await;
    module(conn, u7, 1, true).await;
    u7
}

fn choices(rows: &[OperatorChoiceRow]) -> Vec<(&str, i64)> {
    rows.iter().map(|r| (r.choice.as_str(), r.users)).collect()
}

fn levels(rows: &[OperatorLevelRow]) -> Vec<(&str, i16, i64)> {
    rows.iter()
        .map(|r| (r.key.as_str(), r.level, r.users))
        .collect()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn ownership_counts_sharing_rosters_per_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    roster_fixture(&mut conn).await;
    stats::refresh_operator_stats(&mut conn).await.unwrap();

    let (population, mut rows) = stats::operator_ownership(&mut conn, Server::EN)
        .await
        .unwrap();
    rows.sort_by(|a, b| a.operator_id.cmp(&b.operator_id));
    let got: Vec<(&str, i64, i64)> = rows
        .iter()
        .map(|r| (r.operator_id.as_str(), r.owners, r.e2_owners))
        .collect();
    assert_eq!(population, 4);
    assert_eq!(got, [("op_a", 4, 3), ("op_b", 1, 0), ("op_c", 1, 1)]);

    let (population, rows) = stats::operator_ownership(&mut conn, Server::JP)
        .await
        .unwrap();
    assert_eq!(population, 1);
    assert_eq!(rows.len(), 1);
    let (population, rows) = stats::operator_ownership(&mut conn, Server::KR)
        .await
        .unwrap();
    assert_eq!((population, rows.len()), (0, 0));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn choices_count_e2_skills_and_advanced_modules_most_picked_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    roster_fixture(&mut conn).await;
    stats::refresh_operator_stats(&mut conn).await.unwrap();

    let (skills, modules) = stats::operator_choices(&mut conn, Server::EN, "op_a")
        .await
        .unwrap();
    assert_eq!(choices(&skills), [("1", 2), ("0", 1)]);
    assert_eq!(
        choices(&modules),
        [("uniequip_002_a", 2), ("uniequip_003_a", 1)]
    );

    let (skills, modules) = stats::operator_choices(&mut conn, Server::EN, "op_c")
        .await
        .unwrap();
    assert!(skills.is_empty());
    assert_eq!(choices(&modules), [("uniequip_003_c", 1)]);

    let (skills, _) = stats::operator_choices(&mut conn, Server::JP, "op_a")
        .await
        .unwrap();
    assert_eq!(choices(&skills), [("2", 1)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn tied_choices_break_on_the_choice_itself(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    for (n, skill_index, equip) in [(1, 2, "uniequip_003_a"), (2, 0, "uniequip_002_a")] {
        let user = account(&mut conn, n, Server::EN, Some(true)).await;
        operator(&mut conn, user, "op_a", 2, skill_index, Some(equip)).await;
    }
    stats::refresh_operator_stats(&mut conn).await.unwrap();

    let (skills, modules) = stats::operator_choices(&mut conn, Server::EN, "op_a")
        .await
        .unwrap();
    assert_eq!(choices(&skills), [("0", 1), ("2", 1)]);
    assert_eq!(
        choices(&modules),
        [("uniequip_002_a", 1), ("uniequip_003_a", 1)]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn levels_count_e2_owners_with_a_locked_module_at_zero(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    roster_fixture(&mut conn).await;
    stats::refresh_operator_stats(&mut conn).await.unwrap();

    let (masteries, modules) = stats::operator_levels(&mut conn, Server::EN, "op_a")
        .await
        .unwrap();
    assert_eq!(levels(&masteries), [("0", 1, 1), ("0", 3, 2), ("1", 0, 1)]);
    assert_eq!(
        levels(&modules),
        [("uniequip_002_a", 0, 2), ("uniequip_002_a", 2, 1)]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_refresh_replaces_the_previous_aggregate(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let u7 = roster_fixture(&mut conn).await;

    assert!(
        stats::operator_ownership_refreshed_at(&mut conn)
            .await
            .unwrap()
            .is_none()
    );
    stats::refresh_operator_stats(&mut conn).await.unwrap();
    assert!(
        stats::operator_ownership_refreshed_at(&mut conn)
            .await
            .unwrap()
            .is_some()
    );
    let counts = stats::operator_stats_counts(&mut conn).await.unwrap();
    assert_eq!(
        (
            counts.ownership_rows,
            counts.e2_owners,
            counts.skill_choice_rows,
            counts.module_choice_rows
        ),
        (4, 5, 3, 3)
    );

    sqlx::query("UPDATE user_settings SET share_stats = false WHERE user_id = $1")
        .bind(u7.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    stats::refresh_operator_stats(&mut conn).await.unwrap();

    let (population, _) = stats::operator_ownership(&mut conn, Server::EN)
        .await
        .unwrap();
    assert_eq!(population, 3);
    let (skills, modules) = stats::operator_choices(&mut conn, Server::EN, "op_a")
        .await
        .unwrap();
    assert_eq!(choices(&skills), [("0", 1), ("1", 1)]);
    assert_eq!(choices(&modules), [("uniequip_002_a", 2)]);
    let counts = stats::operator_stats_counts(&mut conn).await.unwrap();
    assert_eq!((counts.ownership_rows, counts.e2_owners), (4, 4));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_condition_counts_as_met_only_between_i64_integers(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = account(&mut conn, 1, Server::EN, Some(true)).await;
    medal(&mut conn, user, "int", -1, 5, json!([[5, 3], [-2, -9]])).await;
    medal(&mut conn, user, "float", -1, 5, json!([[3.0, 1]])).await;
    medal(&mut conn, user, "fraction", -1, 5, json!([[3, 1.5]])).await;
    medal(
        &mut conn,
        user,
        "u64",
        -1,
        5,
        json!([[9_223_372_036_854_775_808_u64, 1]]),
    )
    .await;
    medal(&mut conn, user, "text", -1, 5, json!([["5", 3]])).await;
    medal(&mut conn, user, "short", -1, 5, json!([[5]])).await;
    medal(&mut conn, user, "long", -1, 5, json!([[5, 3, "x"]])).await;

    stats::refresh_medal_ownership(&mut conn).await.unwrap();
    let (_, mut rows) = stats::medal_ownership(&mut conn, Server::EN).await.unwrap();
    rows.sort_by(|a, b| a.medal_id.cmp(&b.medal_id));
    let got: Vec<&str> = rows.iter().map(|r| r.medal_id.as_str()).collect();
    assert_eq!(got, ["int", "long"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn medal_ownership_counts_earned_medals_of_sharing_accounts(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let u1 = account(&mut conn, 1, Server::EN, Some(true)).await;
    let u2 = account(&mut conn, 2, Server::EN, Some(true)).await;
    let u3 = account(&mut conn, 3, Server::EN, Some(true)).await;
    let private = account(&mut conn, 4, Server::EN, Some(false)).await;
    let jp = account(&mut conn, 5, Server::JP, Some(true)).await;
    medal(&mut conn, u1, "m1", 100, 0, json!(null)).await;
    medal(&mut conn, u1, "m2", -1, 5, json!([[3, 3], [5, 2]])).await;
    medal(&mut conn, u2, "m2", -1, 5, json!([[1, 3]])).await;
    medal(&mut conn, u2, "m3", -1, 5, json!([])).await;
    medal(&mut conn, u3, "m3", -1, 0, json!([])).await;
    medal(&mut conn, u3, "m4", 0, 0, json!(null)).await;
    medal(&mut conn, private, "m1", 100, 0, json!(null)).await;
    medal(&mut conn, jp, "m1", 1, 0, json!(null)).await;

    assert!(
        stats::medal_ownership_refreshed_at(&mut conn)
            .await
            .unwrap()
            .is_none()
    );
    stats::refresh_medal_ownership(&mut conn).await.unwrap();
    assert!(
        stats::medal_ownership_refreshed_at(&mut conn)
            .await
            .unwrap()
            .is_some()
    );

    let (population, mut rows) = stats::medal_ownership(&mut conn, Server::EN).await.unwrap();
    rows.sort_by(|a, b| a.medal_id.cmp(&b.medal_id));
    let got: Vec<(&str, i64)> = rows
        .iter()
        .map(|r| (r.medal_id.as_str(), r.owners))
        .collect();
    assert_eq!(population, 3);
    assert_eq!(got, [("m1", 1), ("m2", 1), ("m3", 1)]);

    let (population, rows) = stats::medal_ownership(&mut conn, Server::JP).await.unwrap();
    assert_eq!((population, rows.len()), (1, 1));
    assert_eq!(stats::medal_ownership_rows(&mut conn).await.unwrap(), 4);

    sqlx::query("UPDATE user_settings SET share_stats = false WHERE user_id = $1")
        .bind(jp.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    stats::refresh_medal_ownership(&mut conn).await.unwrap();
    assert_eq!(stats::medal_ownership_rows(&mut conn).await.unwrap(), 3);
}
