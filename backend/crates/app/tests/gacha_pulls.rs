#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;
use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::policy::Subject;
use myrtle_app::services::accounts::sessions::portal_session_key;
use myrtle_app::services::gacha::stats::{
    self, GLOBAL_STATS_KEY, GlobalGachaStats, PER_BANNER_STATS_KEY, enhanced_stats_key,
    shape_enhanced, snap_top_n,
};
use myrtle_app::services::gacha::{
    self, GachaApiItem, GachaApiResponse, HistoryQuery, STORAGE_OFF_MESSAGE, classify_gacha_group,
    gacha_types_of_group, parse_pool_filter, pull_records, rarity,
};
use myrtle_db::repos::gacha::{self as repo, BatchOutcome, OperatorPullsRow, SharedTotalsRow};
use myrtle_db::repos::users::{self, SettingsUpdate};
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorRarity};
use myrtle_types::{Role, Server, UserId};
use serde_json::{Value, json};

fn item(char_id: &str, star: &str, pool_id: &str, at: i64) -> GachaApiItem {
    serde_json::from_value(json!({
        "charId": char_id, "charName": "x", "star": star, "poolId": pool_id,
        "poolName": "Banner", "typeName": "t", "at": at
    }))
    .unwrap()
}

fn operators(entries: &[(&str, OperatorRarity)]) -> HashMap<String, Operator> {
    entries
        .iter()
        .map(|(id, rarity)| {
            (
                (*id).to_owned(),
                Operator {
                    rarity: rarity.clone(),
                    ..Operator::default()
                },
            )
        })
        .collect()
}

async fn seed(ctx: &myrtle_app::AppContext, user: UserId, rows: &[(&str, i16, &str, i64)]) {
    let records: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(i, (char_id, rarity, pool, at))| {
            json!({
                "char_id": char_id, "pool_id": pool, "rarity": rarity, "pull_timestamp": at,
                "pool_name": null, "gacha_type": if pool.starts_with("LIMITED_") { "limited" } else { "normal" },
                "batch_index": i,
            })
        })
        .collect();
    let mut conn = ctx.db.conn().await.unwrap();
    let outcome = repo::insert_batch(&mut conn, user, &Value::Array(records))
        .await
        .unwrap();
    assert_eq!(outcome, BatchOutcome::Stored);
}

async fn set_flags(ctx: &myrtle_app::AppContext, user: UserId, update: SettingsUpdate<'_>) {
    let mut conn = ctx.db.conn().await.unwrap();
    repo::flags(&mut conn, user).await.unwrap();
    users::update_settings(&mut conn, user, &update)
        .await
        .unwrap();
}

#[test]
fn a_portal_page_parses() {
    let page: GachaApiResponse = serde_json::from_value(json!({"data": {"rows": [
        {"charId": "char_002_amiya", "charName": "Amiya", "star": "5", "poolId": "LIMITED_EN_40_0_4",
         "poolName": "Limited", "typeName": "Limited", "at": 1_700_000_000}
    ], "count": 1}}))
    .unwrap();
    let data = page.data.unwrap();
    assert_eq!(data.count, Some(1));
    assert_eq!(data.rows[0].char_id, "char_002_amiya");
    assert_eq!(data.rows[0].gacha_type(), "limited");

    let empty: GachaApiResponse = serde_json::from_value(json!({"data": null})).unwrap();
    assert!(empty.data.is_none());
}

#[test]
fn pool_prefixes_name_the_gacha_type() {
    for (pool, kind) in [
        ("LIMITED_1", "limited"),
        ("LINKAGE_1", "linkage"),
        ("CLASSIC_1", "classic"),
        ("SINGLE_1", "single"),
        ("BOOT_1", "boot"),
        ("NORM_1", "normal"),
        ("limited_1", "normal"),
    ] {
        assert_eq!(item("c", "3", pool, 0).gacha_type(), kind, "{pool}");
    }
}

#[test]
fn records_number_each_batch_and_prefer_game_data_rarity() {
    let ops = operators(&[("char_a", OperatorRarity::SixStar)]);
    let items = [
        item("char_a", "3", "LIMITED_1", 100),
        item("char_a", "3", "LIMITED_1", 100),
        item("char_new", "5", "LIMITED_1", 100),
        item("char_new", "bad", "NORM_1", 100),
        item("char_a", "6", "LIMITED_1", 200),
    ];
    let (records, missing) = pull_records(&items, Some(&ops));
    let rows = records.as_array().unwrap();
    let field = |i: usize, k: &str| rows[i][k].clone();
    assert_eq!(
        (0..5).map(|i| field(i, "batch_index")).collect::<Vec<_>>(),
        [json!(0), json!(1), json!(2), json!(0), json!(0)]
    );
    assert_eq!(field(0, "rarity"), json!(6));
    assert_eq!(field(2, "rarity"), json!(5));
    assert_eq!(field(3, "rarity"), json!(3));
    assert_eq!(field(3, "gacha_type"), json!("normal"));
    assert_eq!(field(0, "pool_name"), json!("Banner"));
    assert_eq!(missing, ["char_new"]);

    let (_, all_missing) = pull_records(&items, None);
    assert_eq!(all_missing, ["char_a", "char_new"]);
}

#[test]
fn groups_pool_types_into_three_tabs() {
    assert_eq!(classify_gacha_group("limited"), "limited");
    assert_eq!(classify_gacha_group("linkage"), "limited");
    assert_eq!(classify_gacha_group("single"), "special");
    assert_eq!(classify_gacha_group("boot"), "special");
    for other in ["normal", "classic", "double", "LIMITED", ""] {
        assert_eq!(classify_gacha_group(other), "regular", "{other}");
    }
    assert_eq!(gacha_types_of_group("regular"), ["normal", "classic"]);
    assert!(gacha_types_of_group("bogus").is_empty());
}

#[test]
fn pool_filter_keeps_distinct_ids_in_order() {
    assert_eq!(parse_pool_filter(None).unwrap(), None);
    assert_eq!(parse_pool_filter(Some(" , ,")).unwrap(), None);
    assert_eq!(
        parse_pool_filter(Some(
            "LIMITED_EN_40_0_4, SINGLE_EN_40_0_1,LIMITED_EN_40_0_4"
        ))
        .unwrap(),
        Some(vec![
            "LIMITED_EN_40_0_4".to_owned(),
            "SINGLE_EN_40_0_1".to_owned()
        ])
    );
}

#[test]
fn pool_filter_refuses_what_no_pool_id_looks_like() {
    for bad in ["LIMITED'; --", "a b", "pool%", "x".repeat(65).as_str()] {
        assert!(
            matches!(parse_pool_filter(Some(bad)), Err(AppError::BadRequest(_))),
            "{bad}"
        );
    }
    let at_cap = (0..64)
        .map(|i| format!("P_{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        parse_pool_filter(Some(&at_cap)).unwrap().map(|p| p.len()),
        Some(64)
    );
    assert!(parse_pool_filter(Some(&format!("{at_cap},P_X"))).is_err());
}

#[test]
fn top_n_rounds_up_to_a_served_size() {
    for (asked, served) in [
        (None, 20),
        (Some(0), 10),
        (Some(10), 10),
        (Some(11), 20),
        (Some(21), 50),
        (Some(u32::MAX), 50),
    ] {
        assert_eq!(snap_top_n(asked), served, "{asked:?}");
    }
    let mut sizes: Vec<u32> = (0..=200).map(|n| snap_top_n(Some(n))).collect();
    sizes.sort_unstable();
    sizes.dedup();
    assert_eq!(sizes, [10, 20, 50]);
}

#[test]
fn cache_keys_keep_their_text() {
    assert_eq!(GLOBAL_STATS_KEY, "gacha:global_stats");
    assert_eq!(PER_BANNER_STATS_KEY, "gacha:per_banner_stats");
    assert_eq!(enhanced_stats_key(20, true), "gacha:enhanced_stats:20:true");
    assert_eq!(stats::GLOBAL_STATS_TTL, Duration::from_mins(5));
    assert_eq!(stats::ENHANCED_STATS_TTL, Duration::from_mins(10));
    assert_eq!(stats::PER_BANNER_STATS_TTL, Duration::from_mins(10));
}

#[test]
fn enhanced_stats_shape_rates_and_averages() {
    let totals = SharedTotalsRow {
        total_pulls: 200,
        total_users: 3,
        six_star_count: 4,
        five_star_count: 0,
        four_star_count: 100,
        three_star_count: 96,
        first_pull_at: None,
    };
    let ops = vec![OperatorPullsRow {
        char_id: "char_a".into(),
        rarity: 6,
        pull_count: 4,
    }];
    let shaped = shape_enhanced(&totals, ops, None);
    assert!((shaped.pull_rates.six_star_rate - 0.02).abs() < 1e-12);
    assert!((shaped.average_pulls_to_six_star - 50.0).abs() < 1e-12);
    assert!(shaped.average_pulls_to_five_star.abs() < f64::EPSILON);
    assert!((shaped.most_common_operators[0].percentage - 0.02).abs() < 1e-12);
    let json = serde_json::to_value(&shaped).unwrap();
    assert!(json["collectiveStats"].get("firstPullAt").is_none());
    assert!(json.get("pullTiming").is_none());
    assert_eq!(json["cached"], json!(false));

    let empty = shape_enhanced(
        &SharedTotalsRow {
            total_pulls: 0,
            six_star_count: 0,
            ..totals
        },
        vec![],
        None,
    );
    assert!(empty.pull_rates.six_star_rate.abs() < f64::EPSILON);
}

#[tokio::test]
async fn a_fetch_with_storage_off_stores_nothing_and_never_reads_the_portal_session() {
    let (ctx, db) = common::context().await;
    let user = db.user(1, Server::EN).await;
    let subject = common::user(&ctx, user, Role::User).await;
    set_flags(
        &ctx,
        user,
        SettingsUpdate {
            store_gacha: Some(false),
            ..SettingsUpdate::default()
        },
    )
    .await;
    ctx.cache
        .set(&portal_session_key("1"), "not json", Duration::from_mins(1))
        .await;

    let err = gacha::fetch_and_store(&ctx, &subject).await.unwrap_err();
    assert!(matches!(&err, AppError::Conflict(m) if m == STORAGE_OFF_MESSAGE));

    let mut conn = ctx.db.conn().await.unwrap();
    let outcome = repo::insert_batch(&mut conn, user, &json!([]))
        .await
        .unwrap();
    assert_eq!(outcome, BatchOutcome::StorageOff);
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn a_fetch_needs_a_signed_in_user_and_a_portal_session() {
    let (ctx, db) = common::context().await;
    let user = db.user(2, Server::EN).await;
    let subject = common::user(&ctx, user, Role::User).await;
    assert!(matches!(
        gacha::fetch_and_store(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    assert!(
        matches!(gacha::fetch_and_store(&ctx, &subject).await, Err(AppError::BadRequest(m)) if m.contains("no portal session"))
    );
    ctx.cache
        .set(&portal_session_key("2"), "{", Duration::from_mins(1))
        .await;
    assert!(
        matches!(gacha::fetch_and_store(&ctx, &subject).await, Err(AppError::BadRequest(m)) if m == "invalid portal session")
    );
    db.close().await;
}

#[tokio::test]
async fn history_filters_pages_and_groups() {
    let (ctx, db) = common::context().await;
    let user = db.user(3, Server::EN).await;
    let subject = common::user(&ctx, user, Role::User).await;
    seed(
        &ctx,
        user,
        &[
            ("char_a", 6, "LIMITED_1", 300),
            ("char_b", 3, "NORM_1", 200),
            ("char_a", 6, "NORM_1", 100),
        ],
    )
    .await;

    let page = gacha::history(
        &ctx,
        &subject,
        HistoryQuery {
            newest_first: true,
            limit: 2,
            ..HistoryQuery::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(page.pagination.total, 3);
    assert!(page.pagination.has_more);
    assert_eq!(page.records[0].pull_timestamp, 300);
    assert!(page.filters_applied.date_range.is_none());

    let limited = gacha::history(
        &ctx,
        &subject,
        HistoryQuery {
            gacha_type: Some("limited".into()),
            from: Some(50),
            limit: 10,
            ..HistoryQuery::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(limited.pagination.total, 1);
    assert!(!limited.pagination.has_more);
    let json = serde_json::to_value(&limited).unwrap();
    assert_eq!(
        json["filtersApplied"]["dateRange"],
        json!({"from": 50, "to": null})
    );
    assert_eq!(json["records"][0]["gachaType"], json!("limited"));

    let none = gacha::history(
        &ctx,
        &subject,
        HistoryQuery {
            gacha_type: Some("bogus".into()),
            limit: 10,
            ..HistoryQuery::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(none.pagination.total, 0);

    assert_eq!(
        gacha::history_for_char(&ctx, &subject, "char_a")
            .await
            .unwrap()
            .len(),
        2
    );
    let grouped = gacha::stored_records(&ctx, &subject).await.unwrap();
    assert_eq!(
        (
            grouped.limited.total,
            grouped.regular.total,
            grouped.special.total
        ),
        (1, 2, 0)
    );
    assert_eq!(grouped.limited.records[0].star, "6");

    let own = gacha::user_stats(&ctx, &subject).await.unwrap();
    assert_eq!((own.total_pulls, own.six_star_count), (3, 2));
    db.close().await;
}

#[tokio::test]
async fn public_records_need_a_public_profile_and_the_opt_in() {
    let (ctx, db) = common::context().await;
    let owner = db.user(4, Server::EN).await;
    let visitor = common::user(&ctx, db.user(5, Server::EN).await, Role::User).await;
    seed(
        &ctx,
        owner,
        &[
            ("char_a", 6, "LIMITED_1", 100),
            ("char_b", 3, "NORM_1", 100),
        ],
    )
    .await;
    set_flags(
        &ctx,
        owner,
        SettingsUpdate {
            public_profile: Some(true),
            ..SettingsUpdate::default()
        },
    )
    .await;

    assert!(matches!(
        gacha::public_records(&ctx, &visitor, "4", None).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        gacha::public_records(&ctx, &visitor, "404", None).await,
        Err(AppError::NotFound(None))
    ));
    assert!(
        matches!(
            gacha::public_records(&ctx, &visitor, "4", Some("bad id")).await,
            Err(AppError::Forbidden)
        ),
        "the gate answers before the filter"
    );
    assert!(matches!(
        gacha::public_records(&ctx, &visitor, "404", Some("bad id")).await,
        Err(AppError::NotFound(None))
    ));
    let mine = gacha::public_records(
        &ctx,
        &common::user(&ctx, owner, Role::User).await,
        "4",
        None,
    )
    .await
    .unwrap();
    assert_eq!(mine.limited.total + mine.regular.total, 2);

    set_flags(
        &ctx,
        owner,
        SettingsUpdate {
            public_gacha: Some(true),
            ..SettingsUpdate::default()
        },
    )
    .await;
    let shared = gacha::public_records(&ctx, &Subject::Anonymous, "4", Some("LIMITED_1"))
        .await
        .unwrap();
    assert_eq!((shared.limited.total, shared.regular.total), (1, 0));
    assert!(matches!(
        gacha::public_records(&ctx, &Subject::Anonymous, "4", Some("bad id")).await,
        Err(AppError::BadRequest(_))
    ));

    set_flags(
        &ctx,
        owner,
        SettingsUpdate {
            public_profile: Some(false),
            ..SettingsUpdate::default()
        },
    )
    .await;
    assert!(matches!(
        gacha::public_records(&ctx, &Subject::Anonymous, "4", None).await,
        Err(AppError::Forbidden)
    ));
    db.close().await;
}

#[tokio::test]
async fn community_stats_count_only_sharers_and_a_share_flip_drops_the_cache() {
    let (ctx, db) = common::context().await;
    let sharer = db.user(6, Server::EN).await;
    let private = db.user(7, Server::EN).await;
    seed(
        &ctx,
        sharer,
        &[
            ("char_a", 6, "LIMITED_1", 100),
            ("char_b", 3, "LIMITED_1", 100),
        ],
    )
    .await;
    seed(&ctx, private, &[("char_c", 5, "NORM_1", 100)]).await;
    set_flags(&ctx, sharer, SettingsUpdate::default()).await;
    let private_subject = common::user(&ctx, private, Role::User).await;
    gacha::update_settings(&ctx, &private_subject, None, Some(false))
        .await
        .unwrap();

    let global = gacha::global_stats(&ctx).await.unwrap();
    assert_eq!((global.total_pulls, global.total_users), (2, 1));
    assert!((global.six_star_rate - 0.5).abs() < 1e-12);
    let banners = gacha::per_banner_stats(&ctx).await.unwrap();
    assert_eq!(banners.len(), 1);
    assert_eq!(banners[0].pool_id, "LIMITED_1");
    let enhanced = gacha::enhanced_stats(&ctx, Some(3), true).await.unwrap();
    assert!(!enhanced.cached);
    assert_eq!(
        enhanced
            .pull_timing
            .as_ref()
            .unwrap()
            .by_date
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert!(
        gacha::enhanced_stats(&ctx, Some(10), true)
            .await
            .unwrap()
            .cached
    );
    assert!(ctx.cache.get(&enhanced_stats_key(10, true)).await.is_some());

    let unchanged = gacha::update_settings(&ctx, &private_subject, Some(true), Some(false))
        .await
        .unwrap();
    assert!(!unchanged.share_anonymous_stats);
    assert!(ctx.cache.get(GLOBAL_STATS_KEY).await.is_some());

    let flipped = gacha::update_settings(&ctx, &private_subject, None, Some(true))
        .await
        .unwrap();
    assert!(flipped.share_anonymous_stats && flipped.store_records);
    assert_eq!(flipped.total_pulls, 1);
    for key in [
        GLOBAL_STATS_KEY,
        PER_BANNER_STATS_KEY,
        &enhanced_stats_key(10, true),
    ] {
        assert!(ctx.cache.get(key).await.is_none(), "{key}");
    }
    let regrown: GlobalGachaStats = gacha::global_stats(&ctx).await.unwrap();
    assert_eq!(regrown.total_users, 2);
    db.close().await;
}

#[tokio::test]
async fn settings_start_from_the_defaults() {
    let (ctx, db) = common::context().await;
    let subject = common::user(&ctx, db.user(8, Server::EN).await, Role::User).await;
    let settings = gacha::settings(&ctx, &subject).await.unwrap();
    assert!(settings.store_records && settings.share_anonymous_stats);
    assert_eq!(settings.total_pulls, 0);
    assert!(matches!(
        gacha::user_stats(&ctx, &subject).await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        gacha::settings(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn reconcile_rewrites_disagreeing_rarities_and_flushes_stats() {
    let (ctx, db) = common::context().await;
    let user = db.user(9, Server::EN).await;
    seed(
        &ctx,
        user,
        &[
            ("char_a", 3, "LIMITED_1", 100),
            ("char_a", 3, "LIMITED_1", 200),
            ("char_b", 5, "NORM_1", 100),
            ("char_gone", 4, "NORM_1", 300),
        ],
    )
    .await;
    gacha::global_stats(&ctx).await.unwrap();
    let ops = operators(&[
        ("char_a", OperatorRarity::SixStar),
        ("char_b", OperatorRarity::FiveStar),
    ]);

    let sync = rarity::reconcile_against(&ctx, &ops).await.unwrap();
    assert_eq!(
        sync,
        rarity::RaritySync {
            distinct_pairs: 3,
            fixed_char_ids: 1,
            rows_updated: 2
        }
    );
    assert!(ctx.cache.get(GLOBAL_STATS_KEY).await.is_none());
    let stats = gacha::user_stats(&ctx, &common::user(&ctx, user, Role::User).await)
        .await
        .unwrap();
    assert_eq!(stats.six_star_count, 2);

    gacha::global_stats(&ctx).await.unwrap();
    let again = rarity::reconcile_against(&ctx, &ops).await.unwrap();
    assert_eq!(again.rows_updated, 0);
    assert!(ctx.cache.get(GLOBAL_STATS_KEY).await.is_some());

    assert!(matches!(
        gacha::reconcile_rarities(&ctx, 0).await,
        Err(AppError::Unavailable)
    ));
    db.close().await;
}
