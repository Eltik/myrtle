#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path;
use std::sync::Arc;

use myrtle_app::AppContext;
use myrtle_app::cache::Cache;
use myrtle_app::context::Announced;
use myrtle_app::services::catalog::release::jobs::{
    enqueue_boot_pass, has_own_art, mirror_art, record_ledger,
};
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_db::repos::data_generations;
use myrtle_db::repos::release::{self, Sighting};
use myrtle_hypergryph::{DeviceIds, NetworkConfig, Upstream};
use myrtle_jobs::{Job, JobKind};
use myrtle_types::{Server, WorkerId};

const BANNER: &str = "textures/ui_home_act_banner_gacha_x/pic_pool1.png";

/// One process over `assets`, its own registry and memory cache, network config from a closed local port
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

/// A tree whose gacha table names `POOL1` and whose art carries its banner
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

/// [`extract`] naming `pools` and the operators `chars` instead
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
    assert!(has_own_art(&ctx, Server::EN), "the default server");
    assert!(has_own_art(&ctx, Server::CN));
    assert!(has_own_art(&ctx, Server::Bilibili), "shares CN's tree");
    assert!(!has_own_art(&ctx, Server::JP), "text-only");
    assert_eq!(
        mirror_art(&ctx, Server::JP, 0).await.unwrap(),
        "jp: text-only, no art to archive"
    );
    let own = over(&base, assets.path(), true, false);
    assert!(
        has_own_art(&own, Server::JP),
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
            .activate_at(Server::EN, 0, None)
            .await
            .unwrap();
    }
    let generation = api.announce_update(Server::EN, Some("v1")).await.unwrap();
    assert_eq!(worker.reload_if_changed().await.unwrap(), [Server::EN]);
    let want = Announced {
        generation,
        res_version: Some("v1".into()),
    };
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
async fn a_sidecar_reload_carries_no_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, 0, None)
        .await
        .unwrap();
    ctx.reload_game_data(Server::EN).await.unwrap();
    let jobs = claim(&ctx, JobKind::ReleaseLedgerRecord).await;
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        announced(&jobs[0]),
        Announced {
            generation: 1,
            res_version: None
        },
        "never the loaded snapshot's generation and fingerprint"
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
            .activate_at(Server::EN, 0, None)
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
        record_ledger(&worker, Server::EN, Some("v1"), 1)
            .await
            .unwrap(),
        "en: 1 inserted, 0 refreshed"
    );
    db.close().await;
}

#[tokio::test]
async fn the_boot_pass_queues_each_loaded_server_once_with_no_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, 0, None)
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

#[tokio::test]
async fn a_job_run_on_newer_data_records_it_under_that_datas_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract_with(&root, &["POOL1"], &["char_a"]);
    let (pipeline, worker) = (
        over(&base, assets.path(), true, true),
        over(&base, assets.path(), true, true),
    );
    let v1 = pipeline
        .announce_update(Server::EN, Some("v1"))
        .await
        .unwrap();
    let mut conn = db.db.conn().await.unwrap();
    let pool1 = Sighting {
        kind: "pool",
        id: "POOL1",
        start_time: None,
        end_time: None,
    };
    release::upsert_sightings(&mut conn, Server::EN, Some("v1"), &[pool1])
        .await
        .unwrap();
    extract_with(&root, &["POOL1", "POOL2"], &["char_a", "char_b"]);
    pipeline
        .announce_update(Server::EN, Some("v2"))
        .await
        .unwrap();

    record_ledger(&worker, Server::EN, Some("v1"), v1)
        .await
        .unwrap();
    let live = worker.snapshots.require(Server::EN).unwrap();
    assert_eq!(live.generation, 2, "the v1 job ran on the v2 data");
    assert_eq!(live.res_version.as_deref(), Some("v2"));
    assert_eq!(
        release::debut_pairs(&mut conn, Server::EN).await.unwrap(),
        [
            ("POOL2".to_owned(), "char_a".to_owned()),
            ("POOL2".to_owned(), "char_b".to_owned()),
        ],
        "v2's sightings carry v2, and no v2 operator debuts on the v1 pool"
    );
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn a_bump_without_a_version_keeps_the_snapshots_pipeline_version() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"));
    let ctx = over(&base, assets.path(), true, true);
    ctx.snapshots
        .activate_at(Server::EN, 0, None)
        .await
        .unwrap();
    ctx.announce_update(Server::EN, Some("v1")).await.unwrap();
    ctx.reload_game_data(Server::EN).await.unwrap();
    let live = ctx.snapshots.require(Server::EN).unwrap();
    assert_eq!(
        (live.generation, live.res_version.as_deref()),
        (2, Some("v1"))
    );
    db.close().await;
}
