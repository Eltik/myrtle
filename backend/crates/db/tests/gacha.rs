#![allow(clippy::unwrap_used)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::gacha::{self, BatchOutcome, GachaRecordRow, HistoryFilter};
use myrtle_types::UserId;
use serde_json::{Value, json};
use sqlx::PgPool;

const T1: i64 = 1_700_000_000_000;
const T2: i64 = 1_700_003_600_000;
const T3: i64 = 1_700_090_000_000;

type FilterCase<'a> = (HistoryFilter<'a>, &'a [(i64, &'a str, i16)]);

fn pull(char_id: &str, pool_id: &str, rarity: i16, at: i64, gacha_type: &str, index: i16) -> Value {
    json!({
        "char_id": char_id, "pool_id": pool_id, "rarity": rarity, "pull_timestamp": at,
        "pool_name": pool_id, "gacha_type": gacha_type, "batch_index": index,
    })
}

async fn settings(conn: &mut Conn, user: UserId, store_gacha: bool, share_stats: bool) {
    sqlx::query(
        "INSERT INTO user_settings (user_id, store_gacha, share_stats) VALUES ($1, $2, $3)",
    )
    .bind(user.0)
    .bind(store_gacha)
    .bind(share_stats)
    .execute(conn)
    .await
    .unwrap();
}

async fn store(conn: &mut Conn, user: UserId, pulls: Value) {
    let outcome = gacha::insert_batch(conn, user, &pulls).await.unwrap();
    assert_eq!(outcome, BatchOutcome::Stored);
}

async fn stored_pulls(conn: &mut Conn, user: UserId) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM gacha_records WHERE user_id = $1")
        .bind(user.0)
        .fetch_one(conn)
        .await
        .unwrap()
}

fn keys(rows: &[GachaRecordRow]) -> Vec<(i64, &str, i16)> {
    rows.iter()
        .map(|r| (r.pull_timestamp, r.char_id.as_str(), r.batch_index))
        .collect()
}

/// Four pulls over three banners, the multi-pull sent with its second pull first
async fn history_fixture(conn: &mut Conn) -> UserId {
    let user = common::user(conn, 1).await;
    store(
        conn,
        user,
        json!([
            pull("char_b", "P1", 3, 1000, "limited", 1),
            pull("char_a", "P1", 6, 1000, "limited", 0),
            pull("char_c", "P2", 5, 2000, "normal", 0),
            pull("char_a", "P3", 6, 500, "single", 0),
        ]),
    )
    .await;
    let other = common::user(conn, 2).await;
    store(
        conn,
        other,
        json!([pull("char_a", "P1", 6, 3000, "limited", 0)]),
    )
    .await;
    user
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_opted_out_account_stores_no_pulls(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let off = common::user(&mut conn, 1).await;
    let on = common::user(&mut conn, 2).await;
    let no_row = common::user(&mut conn, 3).await;
    settings(&mut conn, off, false, true).await;
    settings(&mut conn, on, true, true).await;
    let pulls = || {
        json!([
            pull("char_002_amiya", "NORM_0_0_1", 5, T1, "normal", 0),
            pull("char_285_medic2", "NORM_0_0_1", 1, T1, "normal", 1),
        ])
    };

    assert!(!gacha::store_gacha_enabled(&mut conn, off).await.unwrap());
    assert!(gacha::store_gacha_enabled(&mut conn, no_row).await.unwrap());
    let outcome = gacha::insert_batch(&mut conn, off, &pulls()).await.unwrap();
    assert_eq!(outcome, BatchOutcome::StorageOff);
    assert_eq!(stored_pulls(&mut conn, off).await, 0);

    store(&mut conn, on, pulls()).await;
    assert_eq!(stored_pulls(&mut conn, on).await, 2);
    store(&mut conn, no_row, pulls()).await;
    assert_eq!(stored_pulls(&mut conn, no_row).await, 2);

    sqlx::query("UPDATE user_settings SET store_gacha = false WHERE user_id = $1")
        .bind(on.0)
        .execute(&mut *conn)
        .await
        .unwrap();
    let later = json!([pull("char_002_amiya", "NORM_0_0_1", 5, T2, "normal", 0)]);
    let outcome = gacha::insert_batch(&mut conn, on, &later).await.unwrap();
    assert_eq!(outcome, BatchOutcome::StorageOff);
    assert_eq!(stored_pulls(&mut conn, on).await, 2);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_resent_batch_replaces_the_pulls_of_its_timestamp_and_banner(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    store(
        &mut conn,
        user,
        json!([
            pull("char_a", "P1", 6, 1000, "limited", 0),
            pull("char_b", "P1", 3, 1000, "limited", 1),
            pull("char_c", "P2", 5, 1000, "normal", 0),
        ]),
    )
    .await;

    store(
        &mut conn,
        user,
        json!([pull("char_d", "P1", 4, 1000, "limited", 0)]),
    )
    .await;

    let rows = gacha::list_in_pools(&mut conn, user, None).await.unwrap();
    let mut chars: Vec<&str> = rows.iter().map(|r| r.char_id.as_str()).collect();
    chars.sort_unstable();
    assert_eq!(chars, ["char_c", "char_d"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn history_is_newest_first_with_a_multi_pull_in_batch_order(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = history_fixture(&mut conn).await;

    let rows = gacha::history(&mut conn, user, None, 50, 0).await.unwrap();
    assert_eq!(
        keys(&rows),
        [
            (2000, "char_c", 0),
            (1000, "char_a", 0),
            (1000, "char_b", 1),
            (500, "char_a", 0)
        ]
    );
    assert!(rows[1].id > rows[2].id);
    assert_eq!(rows[0].gacha_type.as_deref(), Some("normal"));
    assert_eq!(rows[0].pool_name.as_deref(), Some("P2"));

    let six = gacha::history(&mut conn, user, Some(6), 50, 0)
        .await
        .unwrap();
    assert_eq!(keys(&six), [(1000, "char_a", 0), (500, "char_a", 0)]);
    let page = gacha::history(&mut conn, user, None, 2, 1).await.unwrap();
    assert_eq!(keys(&page), [(1000, "char_a", 0), (1000, "char_b", 1)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn filtered_history_applies_each_filter_and_counts_past_the_page(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = history_fixture(&mut conn).await;
    let all = HistoryFilter::default();

    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, false, 50, 0)
        .await
        .unwrap();
    assert_eq!(
        keys(&rows),
        [
            (500, "char_a", 0),
            (1000, "char_a", 0),
            (1000, "char_b", 1),
            (2000, "char_c", 0)
        ]
    );
    assert_eq!(total, 4);

    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, true, 1, 0)
        .await
        .unwrap();
    assert_eq!(keys(&rows), [(2000, "char_c", 0)]);
    assert_eq!(total, 4);

    let limited = ["limited".to_owned(), "linkage".to_owned()];
    let cases: [FilterCase<'_>; 6] = [
        (
            HistoryFilter {
                gacha_types: Some(&limited[..]),
                ..all
            },
            &[(1000, "char_a", 0), (1000, "char_b", 1)],
        ),
        (
            HistoryFilter {
                gacha_types: Some(&[][..]),
                ..all
            },
            &[],
        ),
        (
            HistoryFilter {
                rarity: Some(5),
                ..all
            },
            &[(2000, "char_c", 0)],
        ),
        (
            HistoryFilter {
                char_id: Some("char_a"),
                ..all
            },
            &[(1000, "char_a", 0), (500, "char_a", 0)],
        ),
        (
            HistoryFilter {
                from_ts: Some(1000),
                to_ts: Some(1000),
                ..all
            },
            &[(1000, "char_a", 0), (1000, "char_b", 1)],
        ),
        (
            HistoryFilter {
                from_ts: Some(1001),
                ..all
            },
            &[(2000, "char_c", 0)],
        ),
    ];
    for (filter, expected) in cases {
        let (rows, total) = gacha::history_filtered(&mut conn, user, &filter, true, 50, 0)
            .await
            .unwrap();
        assert_eq!(keys(&rows), expected, "{filter:?}");
        assert_eq!(total, expected.len() as i64, "{filter:?}");
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn filtered_pages_keep_order_and_size_and_count_every_match(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = history_fixture(&mut conn).await;
    let all = HistoryFilter::default();

    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, true, 2, 1)
        .await
        .unwrap();
    assert_eq!(keys(&rows), [(1000, "char_a", 0), (1000, "char_b", 1)]);
    assert_eq!(total, 4);
    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, false, 2, 1)
        .await
        .unwrap();
    assert_eq!(keys(&rows), [(1000, "char_a", 0), (1000, "char_b", 1)]);
    assert_eq!(total, 4);
    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, false, 3, 0)
        .await
        .unwrap();
    assert_eq!(
        keys(&rows),
        [(500, "char_a", 0), (1000, "char_a", 0), (1000, "char_b", 1)]
    );
    assert_eq!(total, 4);

    let (rows, total) = gacha::history_filtered(&mut conn, user, &all, true, 2, 10)
        .await
        .unwrap();
    assert!(rows.is_empty());
    assert_eq!(total, 4);
    let none = HistoryFilter {
        char_id: Some("nobody"),
        ..all
    };
    let (rows, total) = gacha::history_filtered(&mut conn, user, &none, true, 2, 0)
        .await
        .unwrap();
    assert_eq!((rows.len(), total), (0, 0));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn records_narrow_to_the_named_banners_and_to_one_operator(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = history_fixture(&mut conn).await;

    let every = gacha::list_in_pools(&mut conn, user, None).await.unwrap();
    assert_eq!(every.len(), 4);
    let pools = ["P1".to_owned(), "P3".to_owned()];
    let some = gacha::list_in_pools(&mut conn, user, Some(&pools[..]))
        .await
        .unwrap();
    assert_eq!(
        keys(&some),
        [(1000, "char_a", 0), (1000, "char_b", 1), (500, "char_a", 0)]
    );
    assert!(
        gacha::list_in_pools(&mut conn, user, Some(&[][..]))
            .await
            .unwrap()
            .is_empty()
    );

    let char_a = gacha::list_for_char(&mut conn, user, "char_a")
        .await
        .unwrap();
    assert_eq!(keys(&char_a), [(1000, "char_a", 0), (500, "char_a", 0)]);
    assert!(char_a.iter().all(|r| r.user_id == user.0));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn stats_summarise_one_account_and_are_absent_without_pulls(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = history_fixture(&mut conn).await;
    let empty = common::user(&mut conn, 3).await;

    let stats = gacha::stats(&mut conn, user).await.unwrap().unwrap();
    assert_eq!(stats.user_id, user.0);
    assert_eq!(
        (
            stats.total_pulls,
            stats.six_star_count,
            stats.five_star_count,
            stats.four_star_count
        ),
        (4, 2, 1, 0)
    );
    assert_eq!((stats.first_pull, stats.last_pull), (500, 2000));
    assert!(gacha::stats(&mut conn, empty).await.unwrap().is_none());
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn public_history_is_off_unless_the_owner_opted_in(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let no_row = common::user(&mut conn, 1).await;
    let default = common::user(&mut conn, 2).await;
    let opted_in = common::user(&mut conn, 3).await;
    settings(&mut conn, default, true, true).await;
    settings(&mut conn, opted_in, true, true).await;
    sqlx::query("UPDATE user_settings SET public_gacha = true WHERE user_id = $1")
        .bind(opted_in.0)
        .execute(&mut *conn)
        .await
        .unwrap();

    assert!(
        !gacha::public_history_enabled(&mut conn, no_row)
            .await
            .unwrap()
    );
    assert!(
        !gacha::public_history_enabled(&mut conn, default)
            .await
            .unwrap()
    );
    assert!(
        gacha::public_history_enabled(&mut conn, opted_in)
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn flags_create_the_settings_row_and_an_update_keeps_the_unnamed_switch(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let stored = common::user(&mut conn, 2).await;
    let fresh = common::user(&mut conn, 3).await;
    settings(&mut conn, stored, false, true).await;

    let flags = gacha::flags(&mut conn, user).await.unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (true, true));
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_settings WHERE user_id = $1")
        .bind(user.0)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(rows, 1);
    let flags = gacha::flags(&mut conn, stored).await.unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (false, true));

    let flags = gacha::update_flags(&mut conn, user, Some(false), None)
        .await
        .unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (false, true));
    let flags = gacha::update_flags(&mut conn, user, None, Some(false))
        .await
        .unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (false, false));
    assert!(!gacha::store_gacha_enabled(&mut conn, user).await.unwrap());

    let flags = gacha::update_flags(&mut conn, fresh, None, Some(false))
        .await
        .unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (true, false));
}

async fn xact_table_stat(conn: &mut Conn, table: &str, column: &str) -> i64 {
    let sql = format!(
        "SELECT ({column})::bigint FROM pg_stat_xact_user_tables WHERE relname = '{table}'"
    );
    sqlx::query_scalar(sqlx::AssertSqlSafe(sql))
        .fetch_optional(conn)
        .await
        .unwrap()
        .unwrap_or(0)
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_first_switch_change_writes_the_settings_row_once(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;

    let mut tx = sqlx::Connection::begin(&mut *conn).await.unwrap();
    let flags = gacha::update_flags(&mut tx, user, Some(false), None)
        .await
        .unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (false, true));
    assert_eq!(
        xact_table_stat(&mut tx, "user_settings", "n_tup_ins").await,
        1
    );
    assert_eq!(
        xact_table_stat(&mut tx, "user_settings", "n_tup_upd").await,
        0
    );
    tx.commit().await.unwrap();

    let flags = gacha::update_flags(&mut conn, user, None, None)
        .await
        .unwrap();
    assert_eq!((flags.store_gacha, flags.share_stats), (false, true));
}

/// Two sharing accounts, one not sharing, one with no settings row
async fn community_fixture(conn: &mut Conn) {
    let a = common::user(conn, 1).await;
    let d = common::user(conn, 2).await;
    let private = common::user(conn, 3).await;
    let no_row = common::user(conn, 4).await;
    settings(conn, a, true, true).await;
    settings(conn, d, true, true).await;
    settings(conn, private, true, false).await;
    store(
        conn,
        a,
        json!([
            pull("c6a", "L1", 6, T1, "limited", 0),
            pull("c5a", "L1", 5, T1, "limited", 1),
            pull("c4a", "L1", 4, T1, "limited", 2),
            pull("c3a", "L1", 3, T1, "limited", 3),
            pull("c6a", "N1", 6, T2, "normal", 0),
            pull("c6b", "N1", 6, T2, "normal", 1),
        ]),
    )
    .await;
    store(
        conn,
        d,
        json!([
            pull("c6b", "N1", 6, T3, "normal", 0),
            pull("c6b", "N1", 6, T3, "normal", 1),
            pull("c6b", "N1", 6, T3, "normal", 2),
        ]),
    )
    .await;
    for user in [private, no_row] {
        store(conn, user, json!([pull("c6c", "X", 6, T1, "single", 0)])).await;
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn community_totals_count_only_sharing_accounts(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();

    let empty = gacha::shared_totals(&mut conn).await.unwrap();
    assert_eq!((empty.total_pulls, empty.first_pull_at), (0, None));

    community_fixture(&mut conn).await;
    let t = gacha::shared_totals(&mut conn).await.unwrap();
    assert_eq!((t.total_pulls, t.total_users), (9, 2));
    assert_eq!(
        (
            t.six_star_count,
            t.five_star_count,
            t.four_star_count,
            t.three_star_count
        ),
        (6, 1, 1, 1)
    );
    assert_eq!(t.first_pull_at, Some(T1 / 1000));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn top_operators_are_capped_per_rarity_and_rarest_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    community_fixture(&mut conn).await;

    let top = gacha::shared_top_operators(&mut conn, 1).await.unwrap();
    let got: Vec<(&str, i16, i64)> = top
        .iter()
        .map(|r| (r.char_id.as_str(), r.rarity, r.pull_count))
        .collect();
    assert_eq!(
        got,
        [("c6b", 6, 4), ("c5a", 5, 1), ("c4a", 4, 1), ("c3a", 3, 1)]
    );

    let two = gacha::shared_top_operators(&mut conn, 2).await.unwrap();
    assert_eq!(two.len(), 5);
    assert_eq!((two[1].char_id.as_str(), two[1].pull_count), ("c6a", 2));
}

async fn assert_utc_buckets(conn: &mut Conn) {
    let hours = gacha::shared_pulls_by_hour(&mut *conn).await.unwrap();
    let hours: Vec<(i32, i64)> = hours.iter().map(|r| (r.hour, r.pull_count)).collect();
    assert_eq!(hours, [(22, 4), (23, 5)]);

    let days = gacha::shared_pulls_by_weekday(&mut *conn).await.unwrap();
    let days: Vec<(i32, i64)> = days.iter().map(|r| (r.day, r.pull_count)).collect();
    assert_eq!(days, [(2, 6), (3, 3)]);

    let dates = gacha::shared_pulls_by_date(conn).await.unwrap();
    let dates: Vec<(&str, i64)> = dates
        .iter()
        .map(|r| (r.date.as_str(), r.pull_count))
        .collect();
    assert_eq!(dates, [("2023-11-14", 6), ("2023-11-15", 3)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn community_buckets_ignore_the_session_time_zone(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    community_fixture(&mut conn).await;
    sqlx::query("SET TIME ZONE 'Pacific/Kiritimati'")
        .execute(&mut *conn)
        .await
        .unwrap();
    assert_utc_buckets(&mut conn).await;
}

/// Five sharing pulls of rarity 5, each its own operator and banner, sent out of name order
async fn tied_fixture(conn: &mut Conn) {
    let user = common::user(conn, 1).await;
    settings(conn, user, true, true).await;
    let pulls: Vec<Value> = ["d", "b", "e", "a", "c"]
        .iter()
        .zip(0..)
        .map(|(n, i)| pull(&format!("c5{n}"), &format!("P{n}"), 5, T1 + i, "normal", 0))
        .collect();
    store(conn, user, Value::Array(pulls)).await;
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn tied_pull_counts_rank_operators_and_banners_by_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    tied_fixture(&mut conn).await;

    let top = gacha::shared_top_operators(&mut conn, 1).await.unwrap();
    let top: Vec<&str> = top.iter().map(|r| r.char_id.as_str()).collect();
    assert_eq!(top, ["c5a"]);
    let all = gacha::shared_top_operators(&mut conn, 5).await.unwrap();
    let all: Vec<&str> = all.iter().map(|r| r.char_id.as_str()).collect();
    assert_eq!(all, ["c5a", "c5b", "c5c", "c5d", "c5e"]);

    let banners = gacha::shared_banner_totals(&mut conn).await.unwrap();
    let banners: Vec<&str> = banners.iter().map(|b| b.pool_id.as_str()).collect();
    assert_eq!(banners, ["Pa", "Pb", "Pc", "Pd", "Pe"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn banner_totals_put_the_most_pulled_banner_first(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    community_fixture(&mut conn).await;

    let banners = gacha::shared_banner_totals(&mut conn).await.unwrap();
    let got: Vec<(&str, i64, i64, i64, i64)> = banners
        .iter()
        .map(|b| {
            (
                b.pool_id.as_str(),
                b.pull_count,
                b.six_star_count,
                b.five_star_count,
                b.user_count,
            )
        })
        .collect();
    assert_eq!(got, [("N1", 5, 5, 0, 2), ("L1", 4, 1, 1, 1)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn rarity_correction_rewrites_only_the_rows_that_disagree(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    store(
        &mut conn,
        user,
        json!([
            pull("char_x", "P1", 3, 1000, "normal", 0),
            pull("char_x", "P2", 3, 2000, "normal", 0),
            pull("char_y", "P2", 4, 2000, "normal", 1),
        ]),
    )
    .await;

    let mut pairs: Vec<(String, i16)> = gacha::distinct_char_rarities(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.char_id, r.rarity))
        .collect();
    pairs.sort();
    assert_eq!(pairs, [("char_x".to_owned(), 3), ("char_y".to_owned(), 4)]);

    let changed = gacha::correct_rarities(
        &mut conn,
        &["char_x".to_owned(), "char_y".to_owned()],
        &[5, 4],
    )
    .await
    .unwrap();
    assert_eq!(changed, 2);

    let mut pairs: Vec<(String, i16)> = gacha::distinct_char_rarities(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.char_id, r.rarity))
        .collect();
    pairs.sort();
    assert_eq!(pairs, [("char_x".to_owned(), 5), ("char_y".to_owned(), 4)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_switch_read_holds_off_a_concurrent_settings_write(pool: PgPool) {
    let db = common::db(pool).await;
    let user = common::user(&mut db.conn().await.unwrap(), 1).await;
    settings(&mut db.conn().await.unwrap(), user, true, false).await;
    let write = "UPDATE user_settings SET store_gacha = false WHERE user_id = $1";

    let mut reader = db.begin().await.unwrap();
    assert!(gacha::store_gacha_enabled(&mut reader, user).await.unwrap());

    let mut writer = db.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout = '200ms'")
        .execute(&mut *writer)
        .await
        .unwrap();
    assert!(
        sqlx::query(write)
            .bind(user.0)
            .execute(&mut *writer)
            .await
            .is_err()
    );
    writer.rollback().await.unwrap();

    reader.commit().await.unwrap();
    let mut writer = db.begin().await.unwrap();
    sqlx::query("SET LOCAL lock_timeout = '200ms'")
        .execute(&mut *writer)
        .await
        .unwrap();
    sqlx::query(write)
        .bind(user.0)
        .execute(&mut *writer)
        .await
        .unwrap();
    writer.commit().await.unwrap();
    assert!(
        !gacha::store_gacha_enabled(&mut db.conn().await.unwrap(), user)
            .await
            .unwrap()
    );
}
