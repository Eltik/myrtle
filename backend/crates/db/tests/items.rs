#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::items::{self, Currency, Holding};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};
use sqlx::PgPool;

/// A player with Inventory visible unless `layout` hides it, holding `lmd` and `hgg_shard`
async fn holder(
    conn: &mut Conn,
    n: u32,
    server: Server,
    public: bool,
    layout: Option<Value>,
    lmd: i32,
    hgg_shard: i32,
) -> UserId {
    let user = common::user_on(conn, n, server).await;
    sqlx::query(
        "INSERT INTO user_settings (user_id, public_profile, profile_layout) VALUES ($1, $2, $3)",
    )
    .bind(user.0)
    .bind(public)
    .bind(layout)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_status (user_id, lmd, hgg_shard) VALUES ($1, $2, $3)")
        .bind(user.0)
        .bind(lmd)
        .bind(hgg_shard)
        .execute(&mut *conn)
        .await
        .unwrap();
    user
}

async fn give(conn: &mut Conn, user: UserId, item_id: &str, quantity: i32) {
    sqlx::query("INSERT INTO user_items (user_id, item_id, quantity) VALUES ($1, $2, $3)")
        .bind(user.0)
        .bind(item_id)
        .bind(quantity)
        .execute(&mut *conn)
        .await
        .unwrap();
}

fn ranks(rows: &[items::ItemLeaderboardRow]) -> Vec<(i64, &str, i64)> {
    rows.iter()
        .map(|r| (r.rank, r.uid.as_str(), r.quantity))
        .collect()
}

#[test]
fn the_twelve_currencies_map_from_their_item_ids() {
    for id in [
        "4001",
        "4002",
        "4003",
        "4004",
        "4005",
        "7001",
        "7003",
        "7004",
        "6001",
        "SOCIAL_PT",
        "classic_gacha",
        "classic_gacha_10",
    ] {
        let currency = Currency::from_item_id(id).unwrap();
        assert_eq!(currency.item_id(), id);
        assert_eq!(Holding::of(id), Holding::Currency(currency));
    }
    assert_eq!(Holding::of("4006"), Holding::Item("4006"));
    assert_eq!(Holding::of("30011"), Holding::Item("30011"));
}

#[test]
fn only_the_five_header_currencies_name_a_profile_header_field() {
    for currency in Currency::ALL {
        let expected = match currency {
            Currency::Lmd => Some("lmd"),
            Currency::Originite => Some("originite"),
            Currency::Orundum => Some("orundum"),
            Currency::GachaTickets => Some("headhunting_permits"),
            Currency::TenPullTickets => Some("ten_pull_permits"),
            Currency::HggShard
            | Currency::LggShard
            | Currency::RecruitPermits
            | Currency::PracticeTickets
            | Currency::SocialPoint
            | Currency::ClassicGachaTickets
            | Currency::ClassicTenPullTickets => None,
        };
        assert_eq!(currency.header_field(), expected, "{currency:?}");
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_inventory_is_item_rows_plus_every_nonzero_currency(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    sqlx::query(
        "INSERT INTO user_status (user_id, lmd, originite, orundum, hgg_shard, lgg_shard,
             recruit_permits, gacha_tickets, ten_pull_tickets, practice_tickets, social_point,
             classic_gacha_tickets, classic_ten_pull_tickets, orundum_shard)
         VALUES ($1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13), ($2, 0, NULL, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0)",
    )
    .bind(user.0)
    .bind(other.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    give(&mut conn, user, "30011", 40).await;
    give(&mut conn, user, "mod_unlock_token", 2).await;
    give(&mut conn, other, "30011", 1).await;

    let held = items::inventory(&mut conn, user).await.unwrap();
    let mut ids: Vec<&str> = held.iter().map(|i| i.item_id.as_str()).collect();
    ids.sort_unstable();
    let mut expected: Vec<&str> = Currency::ALL.iter().map(|c| c.item_id()).collect();
    expected.extend(["30011", "mod_unlock_token"]);
    expected.sort_unstable();
    assert_eq!(ids, expected);
    let quantity = |id: &str| held.iter().find(|i| i.item_id == id).unwrap().quantity;
    assert_eq!(quantity("4001"), 1);
    assert_eq!(quantity("classic_gacha_10"), 12);
    assert_eq!(quantity("30011"), 40);

    let other_held = items::inventory(&mut conn, other).await.unwrap();
    let other_held: Vec<(&str, i32)> = other_held
        .iter()
        .map(|i| (i.item_id.as_str(), i.quantity))
        .collect();
    assert_eq!(other_held, [("30011", 1), ("4003", 5)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_currency_board_ranks_visible_holders_with_shared_ranks_for_ties(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    holder(&mut conn, 3, Server::EN, true, None, 500, 0).await;
    holder(&mut conn, 2, Server::JP, true, None, 900, 5).await;
    holder(&mut conn, 1, Server::EN, true, None, 500, 5).await;
    holder(&mut conn, 4, Server::EN, true, None, 0, 1).await;
    holder(&mut conn, 5, Server::EN, false, None, 10_000, 1).await;
    let no_tab = json!({"tabs": [{"id": "inventory", "visible": false}]});
    holder(&mut conn, 6, Server::EN, true, Some(no_tab), 10_000, 1).await;
    let no_lmd = json!({"header": {"hidden": ["lmd", "level"]}});
    holder(&mut conn, 7, Server::EN, true, Some(no_lmd), 10_000, 100).await;

    let lmd = Holding::of("4001");
    let page = items::leaderboard(&mut conn, lmd, None, None, 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&page), [(1, "2", 900), (2, "1", 500), (2, "3", 500)]);
    assert_eq!(page[0].server, "jp");
    let totals = items::holding_totals(&mut conn, lmd, None, None)
        .await
        .unwrap();
    assert_eq!((totals.holders, totals.quantity), (3, 1900));

    let shards = Holding::of("4004");
    let page = items::leaderboard(&mut conn, shards, None, None, 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&page)[0], (1, "7", 100));
    assert_eq!(page.len(), 4);
    assert_eq!(page[0].level, None);
    assert_eq!(page[1].level, Some(0));

    let en = items::leaderboard(&mut conn, lmd, Some(Server::EN), None, 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&en), [(2, "1", 500), (2, "3", 500)]);
    let found = items::leaderboard(&mut conn, lmd, None, Some("3"), 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&found), [(2, "3", 500)]);
    let totals = items::holding_totals(&mut conn, lmd, Some(Server::EN), Some("user"))
        .await
        .unwrap();
    assert_eq!((totals.holders, totals.quantity), (2, 1000));
    let second = items::leaderboard(&mut conn, lmd, None, None, 1, 1)
        .await
        .unwrap();
    assert_eq!(ranks(&second), [(2, "1", 500)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_item_board_ranks_user_items_rows_of_that_item_only(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = holder(&mut conn, 1, Server::EN, true, None, 0, 0).await;
    let b = holder(&mut conn, 2, Server::JP, true, None, 0, 0).await;
    let c = holder(&mut conn, 3, Server::EN, true, None, 0, 0).await;
    let private = holder(&mut conn, 4, Server::EN, false, None, 0, 0).await;
    give(&mut conn, a, "30011", 5).await;
    give(&mut conn, b, "30011", 9).await;
    give(&mut conn, c, "30011", 0).await;
    give(&mut conn, c, "30012", 50).await;
    give(&mut conn, private, "30011", 99).await;

    let item = Holding::of("30011");
    let page = items::leaderboard(&mut conn, item, None, None, 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&page), [(1, "2", 9), (2, "1", 5)]);
    let page = items::leaderboard(&mut conn, item, Some(Server::EN), Some("user"), 50, 0)
        .await
        .unwrap();
    assert_eq!(ranks(&page), [(2, "1", 5)]);
    let totals = items::holding_totals(&mut conn, item, None, None)
        .await
        .unwrap();
    assert_eq!((totals.holders, totals.quantity), (2, 14));

    let none = Holding::of("no_such_item");
    let totals = items::holding_totals(&mut conn, none, None, None)
        .await
        .unwrap();
    assert_eq!((totals.holders, totals.quantity), (0, 0));
    assert!(
        items::leaderboard(&mut conn, none, None, None, 20, 0)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_standing_ranks_the_player_globally_and_on_their_server(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = holder(&mut conn, 1, Server::EN, true, None, 300, 0).await;
    let b = holder(&mut conn, 2, Server::JP, true, None, 900, 0).await;
    holder(&mut conn, 3, Server::EN, true, None, 100, 0).await;
    holder(&mut conn, 4, Server::EN, false, None, 5000, 0).await;
    give(&mut conn, a, "30011", 1).await;
    give(&mut conn, b, "30011", 2).await;

    for (holding, id) in [
        (Holding::of("4001"), "4001"),
        (Holding::of("30011"), "30011"),
    ] {
        let standing = items::standing(&mut conn, holding, "1", Server::EN)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(standing.item_id, id);
        assert_eq!((standing.rank_global, standing.rank_server), (2, 1), "{id}");
    }
    let lmd = items::standing(&mut conn, Holding::of("4001"), "1", Server::EN)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(lmd.quantity, 300);
    assert_eq!((lmd.holders_global, lmd.holders_server), (3, 2));

    for (uid, server) in [("4", Server::EN), ("3", Server::JP), ("9", Server::EN)] {
        assert!(
            items::standing(&mut conn, Holding::of("4001"), uid, server)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert!(
        items::standing(&mut conn, Holding::of("30011"), "3", Server::EN)
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_catalog_counts_visible_holders_and_skips_hidden_header_currencies(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let a = holder(&mut conn, 1, Server::EN, true, None, 300, 2).await;
    let b = holder(&mut conn, 2, Server::JP, true, None, 900, 0).await;
    let no_lmd = json!({"header": {"hidden": ["lmd"]}});
    let c = holder(&mut conn, 3, Server::EN, true, Some(no_lmd), 7000, 3).await;
    let private = holder(&mut conn, 4, Server::EN, false, None, 5000, 9).await;
    give(&mut conn, a, "30011", 1).await;
    give(&mut conn, b, "30011", 4).await;
    give(&mut conn, c, "30011", 0).await;
    give(&mut conn, c, "mod_unlock_token", 6).await;
    give(&mut conn, private, "mod_unlock_token", 60).await;

    let catalog = items::catalog(&mut conn, None).await.unwrap();
    let rows: Vec<(&str, i64, i64, i64)> = catalog
        .iter()
        .map(|r| (r.item_id.as_str(), r.holders, r.top, r.total_quantity))
        .collect();
    assert_eq!(
        rows,
        [
            ("30011", 2, 4, 5),
            ("4001", 2, 900, 1200),
            ("4004", 2, 3, 5),
            ("mod_unlock_token", 1, 6, 6),
        ]
    );
    let totals = items::holding_totals(&mut conn, Holding::of("4001"), None, None)
        .await
        .unwrap();
    assert_eq!((totals.holders, totals.quantity), (2, 1200));

    let en: Vec<String> = items::catalog(&mut conn, Some(Server::EN))
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.item_id)
        .collect();
    assert_eq!(en, ["4004", "30011", "4001", "mod_unlock_token"]);

    assert_eq!(
        items::count_visible_players(&mut conn, None).await.unwrap(),
        3
    );
    assert_eq!(
        items::count_visible_players(&mut conn, Some(Server::EN))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        items::count_visible_players(&mut conn, Some(Server::KR))
            .await
            .unwrap(),
        0
    );
}
