#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path;
use std::sync::Arc;

use myrtle_app::cache::Cache;
use myrtle_app::context::Announced;
use myrtle_app::services::catalog::release::jobs::{
    self, enqueue_boot_pass, mirror_art, record_ledger,
};
use myrtle_app::services::catalog::reload::announce_and_record;
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_app::{AppContext, AppError};
use myrtle_db::repos::data_generations::{self, Generation};
use myrtle_db::repos::release;
use myrtle_hypergryph::{DeviceIds, NetworkConfig, Upstream};
use myrtle_jobs::{Job, JobKind};
use myrtle_types::{Server, WorkerId};

const BANNER: &str = "textures/ui_home_act_banner_gacha_x/pic_pool1.png";

fn over(base: &AppContext, assets: &Path, ledger: bool, art_fallback: bool) -> AppContext {
    let mut config = (*base.config).clone();
    config.assets.dir = assets.to_str().unwrap().to_owned();
    config.server.servers = vec![Server::EN, Server::JP, Server::CN];
    config.switches.release_ledger.enabled = ledger;
    config.switches.art_fallback.enabled = art_fallback;
    let mut network = NetworkConfig::new(DeviceIds::default());
    network.hosts.network_routes = std::array::from_fn(|_| "http://127.0.0.1:9/".to_owned());
    AppContext {
        snapshots: Arc::new(SnapshotRegistry::new(&config)),
        cache: Cache::memory(),
        upstream: Arc::new(Upstream::new(reqwest::Client::new(), network)),
        config: Arc::new(config),
        ..base.clone()
    }
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn extract(root: &Path) {
    write(
        root,
        "gamedata/excel/data_version.txt.txt",
        "VersionControl:52.0.0\n",
    );
    write(
        root,
        "gamedata/excel/character_table.json",
        r#"{"Characters":[]}"#,
    );
    write(
        root,
        "gamedata/excel/gacha_table.json",
        r#"{"GachaPoolClient":[{"GachaPoolId":"POOL1","GachaIndex":0,"OpenTime":100,
            "EndTime":200,"GachaPoolName":"P","GachaPoolSummary":"","Guarantee5Avail":0,
            "Guarantee5Count":0,"GachaRuleType":"NORMAL"}],
            "NewbeeGachaPoolClient":[],"GachaTags":[],"RecruitPool":{"RecruitTimeTable":[]}}"#,
    );
    write(root, BANNER, "png");
}

fn extract_with(root: &Path, pools: &[&str], chars: &[&str]) {
    extract(root);
    let chars: Vec<String> = chars
        .iter()
        .map(|id| format!(r#"{{"Key":"{id}","Value":{{"Name":"{id}","Appellation":"{id}"}}}}"#))
        .collect();
    write(
        root,
        "gamedata/excel/character_table.json",
        &format!(r#"{{"Characters":[{}]}}"#, chars.join(",")),
    );
    let pools: Vec<String> = pools
        .iter()
        .map(|id| {
            format!(
                r#"{{"GachaPoolId":"{id}","GachaIndex":0,"OpenTime":100,"EndTime":200,
                "GachaPoolName":"P","GachaPoolSummary":"","Guarantee5Avail":0,
                "Guarantee5Count":0,"GachaRuleType":"NORMAL"}}"#
            )
        })
        .collect();
    write(
        root,
        "gamedata/excel/gacha_table.json",
        &format!(
            r#"{{"GachaPoolClient":[{}],"NewbeeGachaPoolClient":[],"GachaTags":[],
            "RecruitPool":{{"RecruitTimeTable":[]}}}}"#,
            pools.join(",")
        ),
    );
}

async fn claim(ctx: &AppContext, kind: JobKind) -> Vec<Job> {
    ctx.queue
        .claim(WorkerId(uuid::Uuid::new_v4()), &[kind], 10)
        .await
        .unwrap()
}

fn announced(job: &Job) -> Announced {
    serde_json::from_value(job.payload.clone()).unwrap()
}

async fn generations(ctx: &AppContext) -> Vec<(Server, i64)> {
    data_generations::all(&mut ctx.db.conn().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .map(|(server, stored)| (server, stored.generation))
        .collect()
}

#[tokio::test]
async fn only_servers_with_their_own_art_archive_it() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(assets.path().join("cn/portraits")).unwrap();
    let ctx = over(&base, assets.path(), true, true);
    assert!(ctx.snapshots.has_own_art(Server::EN), "the default server");
    assert!(ctx.snapshots.has_own_art(Server::CN));
    assert!(
        ctx.snapshots.has_own_art(Server::Bilibili),
        "shares CN's tree"
    );
    assert!(!ctx.snapshots.has_own_art(Server::JP), "text-only");
    assert_eq!(
        mirror_art(&ctx, Server::JP, 0).await.unwrap(),
        "jp: text-only, no art to archive"
    );
    let own = over(&base, assets.path(), true, false);
    assert!(
        own.snapshots.has_own_art(Server::JP),
        "fallback off: every tree is its own"
    );
    db.close().await;
}

#[tokio::test]
async fn an_announcement_queues_its_jobs_once_however_many_processes_reload() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let (api, worker) = (
        over(&base, assets.path(), true, true),
        over(&base, assets.path(), true, true),
    );
    for ctx in [&api, &worker] {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
    }
    let generation = api.announce_update(Server::EN, Some("v1")).await.unwrap();
    assert_eq!(worker.reload_if_changed().await.unwrap(), [Server::EN]);
    let want = Announced { generation };
    for kind in [
        JobKind::ReleaseLedgerRecord,
        JobKind::ReleaseArtMirror,
        JobKind::RarityReconcile,
    ] {
        let jobs = claim(&worker, kind).await;
        assert_eq!(jobs.len(), 1, "{kind:?}");
        assert_eq!(announced(&jobs[0]), want, "{kind:?}");
    }

    assert_eq!(api.reload_if_changed().await.unwrap(), [Server::EN]);
    for kind in [
        JobKind::ReleaseLedgerRecord,
        JobKind::ReleaseArtMirror,
        JobKind::RarityReconcile,
    ] {
        assert!(
            claim(&worker, kind).await.is_empty(),
            "{kind:?}: the second reload queues nothing"
        );
    }

    api.announce_update(Server::JP, Some("v2")).await.unwrap();
    assert_eq!(claim(&api, JobKind::ReleaseLedgerRecord).await.len(), 1);
    assert!(
        claim(&api, JobKind::ReleaseArtMirror).await.is_empty(),
        "JP has no art of its own"
    );
    assert!(
        claim(&api, JobKind::RarityReconcile).await.is_empty(),
        "only the default server's data holds rarities"
    );

    let off = over(&base, assets.path(), false, true);
    off.announce_update(Server::EN, None).await.unwrap();
    assert!(claim(&off, JobKind::ReleaseLedgerRecord).await.is_empty());
    assert!(claim(&off, JobKind::ReleaseArtMirror).await.is_empty());
    db.close().await;
}

#[tokio::test]
async fn one_update_heard_by_every_process_at_once_is_published_once() {
    const KINDS: [JobKind; 3] = [
        JobKind::ReleaseLedgerRecord,
        JobKind::ReleaseArtMirror,
        JobKind::RarityReconcile,
    ];
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let processes: Vec<AppContext> = (0..4)
        .map(|_| over(&base, assets.path(), true, true))
        .collect();
    for ctx in &processes {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
    }
    let announces: Vec<_> = processes
        .iter()
        .cloned()
        .map(|ctx| tokio::spawn(async move { ctx.announce_update(Server::EN, Some("v1")).await }))
        .collect();
    for announce in announces {
        assert_eq!(announce.await.unwrap().unwrap(), 1);
    }
    let ctx = &processes[0];
    assert_eq!(generations(ctx).await, [(Server::EN, 1)]);
    for kind in KINDS {
        let jobs = claim(ctx, kind).await;
        assert_eq!(jobs.len(), 1, "{kind:?}");
        assert_eq!(announced(&jobs[0]), Announced { generation: 1 }, "{kind:?}");
    }

    assert_eq!(
        ctx.announce_update(Server::EN, Some("v1")).await.unwrap(),
        1
    );
    for kind in KINDS {
        assert!(claim(ctx, kind).await.is_empty(), "{kind:?}: a late repeat");
    }
    assert_eq!(ctx.announce_update(Server::EN, None).await.unwrap(), 2);
    assert_eq!(
        ctx.announce_update(Server::EN, Some("v2")).await.unwrap(),
        3
    );
    for kind in KINDS {
        assert_eq!(
            claim(ctx, kind).await.len(),
            2,
            "{kind:?}: one per real bump"
        );
    }
    db.close().await;
}

#[tokio::test]
async fn a_sidecar_reload_queues_the_ledger_record_at_its_generation() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, &Generation::default())
        .await
        .unwrap();
    ctx.reload_game_data(Server::EN).await.unwrap();
    let jobs = claim(&ctx, JobKind::ReleaseLedgerRecord).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        announced(&jobs[0]),
        Announced { generation: 1 },
        "the announced generation, not the one this process had loaded"
    );
    db.close().await;
}

#[tokio::test]
async fn an_extract_with_new_art_costs_one_more_bump_and_no_second_round_of_jobs() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root);
    let (listener, worker) = (
        over(&base, assets.path(), true, true),
        over(&base, assets.path(), true, true),
    );
    for ctx in [&listener, &worker] {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
    }
    listener
        .announce_update(Server::EN, Some("v1"))
        .await
        .unwrap();
    assert_eq!(listener.reload_if_changed().await.unwrap(), [Server::EN]);

    let mirror = claim(&worker, JobKind::ReleaseArtMirror).await;
    assert_eq!(mirror.len(), 1);
    let generation = announced(&mirror[0]).generation;
    assert_eq!(
        mirror_art(&worker, Server::EN, generation).await.unwrap(),
        "en: 1 copied, 0 present, 0 mirrored, 0 failed",
        "the worker reloaded to the announced data before archiving"
    );
    assert_eq!(
        std::fs::read(root.join("derived/release-art/gacha/POOL1.png")).unwrap(),
        b"png"
    );
    assert_eq!(generations(&worker).await, [(Server::EN, 2)]);
    assert_eq!(worker.snapshots.require(Server::EN).unwrap().generation, 2);
    for kind in [
        JobKind::ReleaseArtMirror,
        JobKind::ReleaseLedgerRecord,
        JobKind::RarityReconcile,
    ] {
        let queued = claim(&worker, kind).await;
        assert!(
            queued.iter().all(|job| announced(job).generation == 1),
            "{kind:?}: the art bump owes no job"
        );
    }

    assert_eq!(
        mirror_art(&worker, Server::EN, 2).await.unwrap(),
        "en: 0 copied, 1 present, 0 mirrored, 0 failed"
    );
    assert_eq!(
        generations(&worker).await,
        [(Server::EN, 2)],
        "a rerun bumps nothing"
    );

    std::fs::remove_file(root.join(BANNER)).unwrap();
    assert_eq!(listener.reload_if_changed().await.unwrap(), [Server::EN]);
    let live = listener.snapshots.require(Server::EN).unwrap();
    assert_eq!(live.generation, 2);
    assert_eq!(
        live.model.assets.gacha_banner_path("POOL1"),
        Some("/derived/release-art/gacha/POOL1.png"),
        "the archive outlives its source"
    );
    assert_eq!(listener.reload_if_changed().await.unwrap(), []);

    assert_eq!(
        record_ledger(&worker, Server::EN, 1).await.unwrap(),
        "en: 1 inserted, 0 refreshed as v1",
        "the art bump moved no pipeline version"
    );
    db.close().await;
}

#[tokio::test]
async fn the_boot_pass_queues_each_loaded_server_once() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, &Generation::default())
        .await
        .unwrap();
    assert_eq!(enqueue_boot_pass(&ctx).await.unwrap(), 2);
    assert_eq!(
        enqueue_boot_pass(&ctx).await.unwrap(),
        2,
        "a second worker boots"
    );
    for kind in [JobKind::ReleaseLedgerRecord, JobKind::ReleaseArtMirror] {
        let jobs = claim(&ctx, kind).await;
        assert_eq!(jobs.len(), 1, "{kind:?}: deduplicated while queued");
        assert_eq!(jobs[0].server, Some(Server::EN));
        assert_eq!(announced(&jobs[0]), Announced::default());
    }
    assert!(claim(&ctx, JobKind::RarityReconcile).await.is_empty());
    db.close().await;
}

async fn record(ctx: &AppContext) -> Option<String> {
    record_with(ctx).await.unwrap()
}

async fn record_with(ctx: &AppContext) -> Result<Option<String>, AppError> {
    jobs::record(ctx, Server::EN).await
}

async fn pairs(ctx: &AppContext) -> Vec<(String, String)> {
    release::debut_pairs(&mut ctx.db.conn().await.unwrap(), Server::EN)
        .await
        .unwrap()
}

fn pair(pool: &str, char: &str) -> (String, String) {
    (pool.to_owned(), char.to_owned())
}

async fn cursor(ctx: &AppContext) -> i64 {
    let mut tx = ctx.db.begin().await.unwrap();
    data_generations::lock_recorded(&mut tx, Server::EN)
        .await
        .unwrap()
}

fn tree(assets: &Path, updates: &[&str]) {
    let pools: Vec<String> = std::iter::once("POOL0".to_owned())
        .chain(updates.iter().map(|u| format!("POOL_{u}")))
        .collect();
    let chars: Vec<String> = std::iter::once("char_0".to_owned())
        .chain(updates.iter().map(|u| format!("char_{u}")))
        .collect();
    let (pools, chars): (Vec<&str>, Vec<&str>) = (
        pools.iter().map(String::as_str).collect(),
        chars.iter().map(String::as_str).collect(),
    );
    extract_with(&assets.join("en"), &pools, &chars);
}

async fn process(base: &AppContext, assets: &Path) -> AppContext {
    let ctx = over(base, assets, true, true);
    let stored = data_generations::all(&mut ctx.db.conn().await.unwrap())
        .await
        .unwrap()
        .into_iter()
        .find(|(server, _)| *server == Server::EN)
        .map(|(_, stored)| stored)
        .unwrap_or_default();
    ctx.snapshots
        .activate_at(Server::EN, &stored)
        .await
        .unwrap();
    ctx
}

async fn booted(base: &AppContext, assets: &Path) -> AppContext {
    tree(assets, &[]);
    let ctx = process(base, assets).await;
    assert_eq!(record(&ctx).await.unwrap(), "en: 2 inserted, 0 refreshed");
    ctx
}

async fn pipeline_update(ctx: &AppContext, version: &str) {
    ctx.announce_update(Server::EN, Some(version))
        .await
        .unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
}

#[tokio::test]
async fn the_listener_records_each_update_on_its_own_data() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let ctx = booted(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    assert_eq!(record(&ctx).await.unwrap(), "en: 0 inserted, 2 refreshed");
    pipeline_update(&ctx, "v1").await;
    assert_eq!(
        record(&ctx).await.unwrap(),
        "en: 2 inserted, 2 refreshed as v1"
    );
    tree(assets.path(), &["v1", "v2"]);
    record(&ctx).await;
    pipeline_update(&ctx, "v2").await;
    assert_eq!(
        record(&ctx).await.unwrap(),
        "en: 2 inserted, 4 refreshed as v2"
    );
    assert_eq!(
        pairs(&ctx).await,
        [pair("POOL_v1", "char_v1"), pair("POOL_v2", "char_v2")],
    );
    assert_eq!(cursor(&ctx).await, 2);
    db.close().await;
}

async fn a_stale_process_records(b_first: bool) {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let a = booted(&base, assets.path()).await;
    let b = process(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    pipeline_update(&a, "v1").await;
    assert!(record(&a).await.unwrap().ends_with("as v1"));
    assert_eq!(b.reload_if_changed().await.unwrap(), [Server::EN]);

    tree(assets.path(), &["v1", "v2"]);
    a.announce_update(Server::EN, Some("v2")).await.unwrap();
    let order = if b_first { [&b, &a] } else { [&a, &b] };
    for ctx in order {
        assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
        record(ctx).await;
    }
    assert_eq!(
        pairs(&a).await,
        [pair("POOL_v1", "char_v1"), pair("POOL_v2", "char_v2")],
        "b first: {b_first}"
    );

    tree(assets.path(), &["v1", "v2", "v3", "v4"]);
    a.announce_update(Server::EN, Some("v3")).await.unwrap();
    pipeline_update(&b, "v4").await;
    assert!(
        !record(&b).await.unwrap().contains(" as "),
        "v3 was never recorded alone"
    );
    assert_eq!(pairs(&a).await.len(), 2, "b first: {b_first}");
    assert_eq!(cursor(&a).await, 4);
    db.close().await;
}

#[tokio::test]
async fn a_stale_process_recording_first_stamps_like_one_process() {
    a_stale_process_records(true).await;
}

#[tokio::test]
async fn a_stale_process_recording_second_stamps_like_one_process() {
    a_stale_process_records(false).await;
}

#[tokio::test]
async fn two_updates_before_a_record_link_no_debut() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let ctx = booted(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    ctx.announce_update(Server::EN, Some("v1")).await.unwrap();
    let behind = process(&base, assets.path()).await;
    tree(assets.path(), &["v1", "v2"]);
    pipeline_update(&ctx, "v2").await;
    assert_eq!(record(&ctx).await.unwrap(), "en: 4 inserted, 2 refreshed");
    assert_eq!(pairs(&ctx).await, []);
    assert_eq!(cursor(&ctx).await, 2);
    assert_eq!(
        record(&behind).await,
        None,
        "a process still on v1 writes nothing"
    );
    db.close().await;
}

#[tokio::test]
async fn a_sidecar_load_stamps_no_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let ctx = booted(&base, assets.path()).await;
    tree(assets.path(), &["side"]);
    ctx.reload_game_data(Server::EN).await.unwrap();
    assert_eq!(record(&ctx).await.unwrap(), "en: 2 inserted, 2 refreshed");
    assert_eq!(pairs(&ctx).await, []);
    db.close().await;
}

#[tokio::test]
async fn an_art_bump_before_the_record_keeps_the_updates_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let ctx = booted(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    pipeline_update(&ctx, "v1").await;
    ctx.announce_art(Server::EN).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
    assert!(record(&ctx).await.unwrap().ends_with("as v1"));
    assert_eq!(pairs(&ctx).await, [pair("POOL_v1", "char_v1")]);
    db.close().await;
}

#[tokio::test]
async fn a_boot_on_a_database_five_updates_in_stamps_no_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    tree(assets.path(), &["v5"]);
    let mut conn = db.db.conn().await.unwrap();
    for n in 1..=5 {
        data_generations::bump(&mut conn, Server::EN, Some(&format!("v{n}")))
            .await
            .unwrap();
    }
    drop(conn);
    let ctx = process(&base, assets.path()).await;
    let live = ctx.snapshots.require(Server::EN).unwrap();
    assert_eq!(
        (live.pipeline_seq, live.res_version.as_deref()),
        (5, Some("v5"))
    );
    assert_eq!(record(&ctx).await.unwrap(), "en: 4 inserted, 0 refreshed");
    assert_eq!(pairs(&ctx).await, []);
    assert_eq!(cursor(&ctx).await, 5);
    db.close().await;
}

#[tokio::test]
async fn two_concurrent_records_stamp_once() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let a = booted(&base, assets.path()).await;
    let b = process(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    pipeline_update(&a, "v1").await;
    assert_eq!(b.reload_if_changed().await.unwrap(), [Server::EN]);
    let (one, two) = tokio::join!(
        tokio::spawn({
            let a = a.clone();
            async move { record(&a).await.unwrap() }
        }),
        tokio::spawn({
            let b = b.clone();
            async move { record(&b).await.unwrap() }
        }),
    );
    let mut results = [one.unwrap(), two.unwrap()];
    results.sort();
    assert_eq!(
        results,
        [
            "en: 0 inserted, 4 refreshed".to_owned(),
            "en: 2 inserted, 2 refreshed as v1".to_owned(),
        ]
    );
    assert_eq!(pairs(&a).await, [pair("POOL_v1", "char_v1")]);
    assert_eq!(cursor(&a).await, 1);
    db.close().await;
}

#[tokio::test]
async fn a_failed_record_rolls_its_cursor_back() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let ctx = booted(&base, assets.path()).await;
    extract_with(
        &assets.path().join("en"),
        &["POOL0", "POOL_v1"],
        &["char_0", "char_v1", r"char_\u0000"],
    );
    pipeline_update(&ctx, "v1").await;
    assert!(record_with(&ctx).await.is_err(), "Postgres refuses a NUL");
    assert_eq!(cursor(&ctx).await, 0);
    assert_eq!(pairs(&ctx).await, [], "the pool's upsert rolled back too");

    tree(assets.path(), &["v1"]);
    ctx.reload_game_data(Server::EN).await.unwrap();
    assert!(record(&ctx).await.unwrap().ends_with("as v1"));
    assert_eq!(pairs(&ctx).await, [pair("POOL_v1", "char_v1")]);
    db.close().await;
}

#[tokio::test]
async fn a_versionless_bump_keeps_the_last_pipeline_version_and_seq() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, &Generation::default())
        .await
        .unwrap();
    ctx.announce_update(Server::EN, Some("v1")).await.unwrap();
    ctx.reload_game_data(Server::EN).await.unwrap();
    let live = ctx.snapshots.require(Server::EN).unwrap();
    assert_eq!(
        (
            live.generation,
            live.res_version.as_deref(),
            live.pipeline_seq
        ),
        (2, Some("v1"), 1)
    );
    db.close().await;
}

#[tokio::test]
async fn the_listener_sequence_pairs_each_update_with_its_own_data() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    tree(assets.path(), &[]);
    let ctx = process(&base, assets.path()).await;
    tree(assets.path(), &["v1"]);
    announce_and_record(&ctx, Server::EN, Some("v1"))
        .await
        .unwrap();
    tree(assets.path(), &["v1", "v2"]);
    announce_and_record(&ctx, Server::EN, Some("v2"))
        .await
        .unwrap();
    assert_eq!(
        pairs(&ctx).await,
        [pair("POOL_v1", "char_v1"), pair("POOL_v2", "char_v2")],
        "the boot load's unrecorded data is recorded before v1 moves the cursor"
    );
    assert_eq!(cursor(&ctx).await, 2);
    db.close().await;
}
