#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use myrtle_app::config::Config;
use myrtle_app::services::gacha::pools::{
    commit_pool_details, pool_detail_path, pools_to_fetch, read_pool_details,
};
use myrtle_app::services::gacha::shops::{
    commit_event_shops, event_shop_path, read_event_shops, shops_to_fetch, stored_data,
};
use myrtle_app::services::gacha::sidecar::{
    self, Fingerprint, is_pending, mark_pending, take_pending,
};
use myrtle_app::services::gacha::{refresh_event_shops, refresh_pool_details};
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::event_shop::{
    EVENT_SHOP_FILE_VERSION, EVENT_SHOP_REL_PATH, EventShopData, EventShopFile,
};
use myrtle_gamedata_raw::tables::gacha::GachaPoolClient;
use myrtle_gamedata_raw::tables::gacha_detail::{
    GachaPoolDetail, POOL_DETAIL_FILE_VERSION, POOL_DETAIL_REL_PATH, PoolDetailFile,
};
use myrtle_types::Server;
use serde_json::json;

fn detail(has_rate_up: bool) -> GachaPoolDetail {
    GachaPoolDetail {
        has_rate_up,
        ..GachaPoolDetail::default()
    }
}

fn shop(price: i32) -> EventShopData {
    serde_json::from_value(json!({
        "shopId": "s", "shopName": "n", "tokenId": "t",
        "startTime": 1, "endTime": 2, "maxPrice": price, "goods": []
    }))
    .unwrap()
}

fn pool(id: &str, end_time: i64) -> GachaPoolClient {
    GachaPoolClient {
        gacha_pool_id: id.into(),
        end_time,
        ..GachaPoolClient::default()
    }
}

fn activity(id: &str, start: i64, end: i64, shop: Option<&str>) -> ActivityBasicInfo {
    ActivityBasicInfo {
        id: id.into(),
        start_time: start,
        end_time: end,
        template_shop_id: shop.map(Into::into),
        ..ActivityBasicInfo::default()
    }
}

fn with_assets(
    ctx: &mut myrtle_app::AppContext,
    dir: &std::path::Path,
    edit: impl Fn(&mut Config),
) {
    let mut config = (*ctx.config).clone();
    config.assets.dir = dir.to_string_lossy().into_owned();
    edit(&mut config);
    ctx.snapshots = Arc::new(SnapshotRegistry::new(&config));
    ctx.config = Arc::new(config);
}

#[test]
fn pending_is_per_server_and_taken_once() {
    let server = Server::Bilibili;
    assert!(!take_pending(server));
    mark_pending(server);
    mark_pending(server);
    assert!(is_pending(server));
    assert!(!is_pending(Server::KR));
    assert!(take_pending(server));
    assert!(!is_pending(server));
    assert!(!take_pending(server));
}

#[test]
fn fingerprints_ignore_key_order_but_not_values() {
    let a: HashMap<String, i32> = (0..64).map(|i| (format!("pool_{i}"), i)).collect();
    let b: HashMap<String, i32> = (0..64).rev().map(|i| (format!("pool_{i}"), i)).collect();
    assert!(!Fingerprint::of(&a).differs(&Fingerprint::of(&b)));

    let x: HashMap<&str, i32> = [("x", 1), ("y", 2)].into();
    let changed: HashMap<&str, i32> = [("x", 1), ("y", 3)].into();
    let added: HashMap<&str, i32> = [("x", 1), ("y", 2), ("z", 0)].into();
    assert!(Fingerprint::of(&x).differs(&Fingerprint::of(&changed)));
    assert!(Fingerprint::of(&x).differs(&Fingerprint::of(&added)));

    let bad: HashMap<(i32, i32), i32> = [((1, 2), 3)].into();
    let bad = Fingerprint::of(&bad);
    assert!(bad.differs(&bad.clone()));
}

#[test]
fn pool_run_metadata_is_not_a_change() {
    let mut a = PoolDetailFile::default();
    a.pools.insert("LIMITED_1".into(), detail(true));
    let mut b = a.clone();
    b.fetched_at = 21_600;
    b.server = "en".into();
    b.version = POOL_DETAIL_FILE_VERSION + 1;
    assert!(!Fingerprint::of(&a.pools).differs(&Fingerprint::of(&b.pools)));

    b.pools.insert("LIMITED_1".into(), detail(false));
    assert!(Fingerprint::of(&a.pools).differs(&Fingerprint::of(&b.pools)));
}

#[test]
fn a_shop_refusal_is_stored_but_does_not_touch_shops() {
    let mut a = EventShopFile::default();
    a.shops.insert("act1side".into(), shop(10));
    let mut meta = a.clone();
    meta.fetched_at = 21_600;
    meta.server = "en".into();
    assert!(!stored_data(&a).differs(&stored_data(&meta)));

    let mut refused = a.clone();
    refused.absent.insert("act2side".into(), 5);
    assert!(stored_data(&a).differs(&stored_data(&refused)));
    assert!(!Fingerprint::of(&a.shops).differs(&Fingerprint::of(&refused.shops)));

    let mut repriced = a.clone();
    repriced.shops.insert("act1side".into(), shop(11));
    assert!(Fingerprint::of(&a.shops).differs(&Fingerprint::of(&repriced.shops)));
}

#[test]
fn only_unseen_or_live_pools_are_fetched() {
    let mut file = PoolDetailFile::default();
    file.pools.insert("ENDED_SEEN".into(), detail(true));
    file.pools.insert("LIVE_SEEN".into(), detail(true));
    let pools = [
        pool("ENDED_SEEN", 50),
        pool("LIVE_SEEN", 500),
        pool("ENDED_NEW", 50),
        pool("", 500),
    ];
    assert_eq!(
        pools_to_fetch(&pools, &file, 100),
        ["LIVE_SEEN", "ENDED_NEW"]
    );
}

#[test]
fn only_started_shops_not_held_or_still_open_are_fetched_latest_first() {
    let mut file = EventShopFile::default();
    file.shops.insert("held_ended".into(), shop(1));
    file.shops.insert("held_open".into(), shop(1));
    file.absent.insert("refused".into(), 1);
    let activities: HashMap<String, ActivityBasicInfo> = [
        activity("held_ended", 10, 50, Some("s1")),
        activity("held_open", 10, 300, Some("s2")),
        activity("new_ended", 10, 60, Some("s3")),
        activity("refused", 10, 900, Some("s4")),
        activity("future", 200, 900, Some("s5")),
        activity("unscheduled", 0, 900, Some("s6")),
        activity("no_shop", 10, 900, None),
    ]
    .into_iter()
    .map(|a| (a.id.clone(), a))
    .collect();
    assert_eq!(
        shops_to_fetch(&activities, &file, 100),
        [
            ("held_open".to_owned(), "s2".to_owned()),
            ("new_ended".to_owned(), "s3".to_owned())
        ]
    );
}

#[tokio::test]
async fn sidecars_land_where_the_loader_reads_them_and_round_trip() {
    let (mut ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    with_assets(&mut ctx, dir.path(), |_| {});

    let pool_path = pool_detail_path(&ctx, Server::EN);
    assert_eq!(pool_path, dir.path().join("en").join(POOL_DETAIL_REL_PATH));
    assert_eq!(
        event_shop_path(&ctx, Server::Bilibili),
        dir.path().join("cn").join(EVENT_SHOP_REL_PATH)
    );
    assert_eq!(read_pool_details(&pool_path).pools.len(), 0);

    let mut pools = PoolDetailFile::default();
    pools.pools.insert("LIMITED_1".into(), detail(true));
    commit_pool_details(&pool_path, &mut pools, Server::EN, 1234).unwrap();
    let back = read_pool_details(&pool_path);
    assert_eq!(
        (back.version, back.fetched_at, back.server.as_str()),
        (POOL_DETAIL_FILE_VERSION, 1234, "en")
    );
    assert!(back.pools["LIMITED_1"].has_rate_up);
    assert!(!pool_path.with_extension("json.tmp").exists());

    let shop_path = event_shop_path(&ctx, Server::EN);
    let mut shops = EventShopFile::default();
    shops.shops.insert("act1side".into(), shop(7));
    shops.absent.insert("act2side".into(), 9);
    commit_event_shops(&shop_path, &mut shops, Server::EN, 99).unwrap();
    let back = read_event_shops(&shop_path);
    assert_eq!(back.version, EVENT_SHOP_FILE_VERSION);
    assert_eq!(back.shops["act1side"].max_price, 7);
    assert_eq!(back.absent["act2side"], 9);
    db.close().await;
}

#[tokio::test]
async fn a_stale_version_or_a_corrupt_file_reads_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("derived").join("x.json");
    sidecar::write(&path, br#"{"version": 99, "pools": {"P": {}}}"#).unwrap();
    assert!(read_pool_details(&path).pools.is_empty());
    sidecar::write(&path, br#"{"version": 1, "pools": {"P": {}}}"#).unwrap();
    assert_eq!(read_pool_details(&path).pools.len(), 1);
    sidecar::write(&path, b"{ truncated").unwrap();
    assert!(read_pool_details(&path).pools.is_empty());
    assert!(read_event_shops(&path).shops.is_empty());
}

#[tokio::test]
async fn the_dir_override_names_the_file_after_the_data_server() {
    let (mut ctx, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let out_dir = out.path().to_string_lossy().into_owned();
    with_assets(&mut ctx, assets.path(), |c| {
        c.jobs.gacha_detail.dir.clone_from(&out_dir);
        c.jobs.event_shop.dir.clone_from(&out_dir);
    });
    assert_eq!(
        pool_detail_path(&ctx, Server::Bilibili),
        out.path().join("cn.json")
    );
    assert_eq!(
        event_shop_path(&ctx, Server::EN),
        out.path().join("en.json")
    );
    db.close().await;
}

#[tokio::test]
async fn jobs_without_service_accounts_do_nothing() {
    let (ctx, db) = common::context().await;
    assert_eq!(refresh_pool_details(&ctx, None).await.unwrap(), 0);
    assert_eq!(
        refresh_event_shops(&ctx, Some(Server::EN)).await.unwrap(),
        0
    );
    assert!(sidecar::account_servers(&ctx).is_empty());
    db.close().await;
}
