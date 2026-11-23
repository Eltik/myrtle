#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::cache::{Cache, CachedJson};
use myrtle_app::services::catalog::release::jobs::{mirror_art, record_ledger};
use myrtle_app::services::catalog::static_data::{ENEMY_AVERAGE_KEY, community_enemy_average};
use myrtle_app::services::gacha::reconcile_rarities;
use myrtle_app::services::gacha::sidecar::{is_pending, mark_pending};
use myrtle_app::snapshots::{SnapshotRegistry, fingerprint, game_version};
use myrtle_db::repos::data_generations::{self, Generation};
use myrtle_gamedata::assets::AssetKind;
use myrtle_hypergryph::{DeviceIds, NetworkConfig, Upstream};
use myrtle_jobs::JobKind;
use myrtle_types::{Server, WorkerId};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body).unwrap();
}

fn extract(root: &Path, version: &str) {
    write(
        root,
        "gamedata/excel/data_version.txt.txt",
        &format!("Stream:\nChange:\nVersionControl:{version}\n"),
    );
    write(root, "gamedata/excel/item_table.json", "{}");
    write(
        root,
        "gamedata/excel/character_table.json",
        r#"{"Characters":[]}"#,
    );
    write(root, "textures/avatars/a.png", "png");
}

#[test]
fn no_gamedata_has_no_fingerprint() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(fingerprint(dir.path()), None);
    assert_eq!(game_version(dir.path()).res, "");
}

#[test]
fn the_fingerprint_follows_what_a_load_reads() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    extract(root, "52.0.0");
    let base = fingerprint(root).unwrap();
    assert_eq!(
        fingerprint(root).unwrap(),
        base,
        "stable when nothing moved"
    );

    write(root, "textures/avatars/b.png", "png");
    write(root, "derived/farm-stages.json", "[]");
    write(root, "derived/release-art/.absent/gacha-x/POOL", "");
    assert_eq!(
        fingerprint(root).unwrap(),
        base,
        "art, our own derived file and markers"
    );

    write(root, "derived/gacha_pool_details.json", "{}");
    let sidecar = fingerprint(root).unwrap();
    assert_ne!(sidecar, base);

    write(root, "derived/release-art/gacha/POOL.png", "png");
    let art = fingerprint(root).unwrap();
    assert_ne!(art, sidecar);

    write(root, "gamedata/excel/item_table.json", "{\"items\":{}}");
    assert_ne!(fingerprint(root).unwrap(), art);
}

#[test]
fn the_game_version_carries_the_data_version_and_fingerprint() {
    let dir = tempfile::tempdir().unwrap();
    extract(dir.path(), "52.0.0");
    let version = game_version(dir.path());
    assert_eq!(version.client, "52.0.0");
    assert_eq!(version.res, fingerprint(dir.path()).unwrap());
}

fn upstream(route: &str) -> Arc<Upstream> {
    let mut network = NetworkConfig::new(DeviceIds::default());
    network.hosts.network_routes = std::array::from_fn(|_| route.to_owned());
    Arc::new(Upstream::new(reqwest::Client::new(), network))
}

fn process(base: &AppContext, assets: &Path, retry: Duration) -> AppContext {
    serving(base, assets, retry, &[Server::EN])
}

fn serving(base: &AppContext, assets: &Path, retry: Duration, servers: &[Server]) -> AppContext {
    let mut config = (*base.config).clone();
    config.assets.dir = assets.to_str().unwrap().to_owned();
    config.server.servers = servers.to_vec();
    AppContext {
        snapshots: Arc::new(SnapshotRegistry::new(&config).with_retry(retry)),
        cache: Cache::memory(),
        upstream: upstream("http://127.0.0.1:9/"),
        config: Arc::new(config),
        ..base.clone()
    }
}

async fn reconciles(ctx: &AppContext) -> usize {
    ctx.queue
        .claim(
            WorkerId(uuid::Uuid::new_v4()),
            &[JobKind::RarityReconcile],
            10,
        )
        .await
        .unwrap()
        .len()
}

#[tokio::test]
async fn an_announcement_reloads_every_process_once() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root, "52.0.0");
    let api = process(&base, assets.path(), Duration::from_mins(10));
    let worker = process(&base, assets.path(), Duration::from_mins(10));
    for ctx in [&api, &worker] {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
        assert_eq!(ctx.reload_if_changed().await.unwrap(), []);
    }
    let before = api.snapshots.require(Server::EN).unwrap();
    assert_eq!(before.model.meta.game_version.client, "52.0.0");
    assert_eq!(
        before.model.meta.game_version.res,
        format!("0-{}", fingerprint(&root).unwrap())
    );

    write(&root, "gamedata/excel/item_table.json", "{\"items\":{}}");
    write(&root, "derived/story-thumbs/x.webp", "traffic");
    for ctx in [&api, &worker] {
        assert_eq!(
            ctx.reload_if_changed().await.unwrap(),
            [],
            "files alone are not news"
        );
    }

    assert_eq!(
        worker
            .announce_update(Server::EN, Some("v1"))
            .await
            .unwrap(),
        1
    );
    let after = format!("1-{}", fingerprint(&root).unwrap());
    for ctx in [&api, &worker] {
        assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
        let live = ctx.snapshots.require(Server::EN).unwrap();
        assert_eq!(live.model.meta.game_version.res, after);
        assert_eq!(live.generation, 1);
        assert!(!Arc::ptr_eq(&live, &before));
        assert_eq!(ctx.reload_if_changed().await.unwrap(), []);
    }
    assert_eq!(
        reconciles(&base).await,
        1,
        "queued by the announcement alone"
    );
    db.close().await;
}

#[tokio::test]
async fn an_asset_only_update_reloads_the_asset_index() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root, "52.0.0");
    let ctx = process(&base, assets.path(), Duration::from_mins(10));
    ctx.snapshots
        .activate_at(Server::EN, &Generation::default())
        .await
        .unwrap();
    let banner = |ctx: &AppContext| {
        ctx.snapshots
            .require(Server::EN)
            .unwrap()
            .model
            .assets
            .gacha_banner_path("POOL1")
            .map(str::to_owned)
    };
    assert_eq!(banner(&ctx), None);

    write(
        &root,
        "textures/ui_home_act_banner_gacha_x/pic_pool1.png",
        "png",
    );
    ctx.reload_game_data(Server::EN).await.unwrap();
    assert_eq!(
        banner(&ctx).as_deref(),
        Some("/textures/ui_home_act_banner_gacha_x/pic_pool1.png")
    );
    let mut conn = ctx.db.conn().await.unwrap();
    assert_eq!(
        data_generations::all(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|(server, stored)| (server, stored.generation))
            .collect::<Vec<_>>(),
        [(Server::EN, 1)]
    );
    drop(conn);
    db.close().await;
}

#[tokio::test]
async fn a_process_on_old_data_never_writes_or_serves_the_reloaded_key() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root, "52.0.0");
    let api = process(&base, assets.path(), Duration::from_mins(10));
    let worker = AppContext {
        snapshots: Arc::new(SnapshotRegistry::new(&api.config)),
        ..api.clone()
    };
    for ctx in [&api, &worker] {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
    }
    worker.announce_update(Server::EN, None).await.unwrap();
    assert_eq!(worker.reload_if_changed().await.unwrap(), [Server::EN]);

    let old = api.snapshots.require(Server::EN).unwrap();
    let new = worker.snapshots.require(Server::EN).unwrap();
    let (old_key, new_key) = (old.key(ENEMY_AVERAGE_KEY), new.key(ENEMY_AVERAGE_KEY));
    assert_ne!(old_key, new_key, "same files, another generation");

    community_enemy_average(&api).await.unwrap();
    assert!(api.cache.get(&old_key).await.is_some());
    assert_eq!(
        worker.cache.get(&new_key).await,
        None,
        "never written by the old process"
    );

    let stale = CachedJson::new("\"built from the old data\"".to_owned());
    api.cache
        .set_json(&old_key, &stale, Duration::from_secs(60))
        .await;
    assert_eq!(community_enemy_average(&api).await.unwrap(), stale);
    let fresh = community_enemy_average(&worker).await.unwrap();
    assert_ne!(fresh, stale, "never served to the reloaded process");
    assert_eq!(worker.cache.get_json(&new_key).await, Some(fresh));
    db.close().await;
}

#[tokio::test]
async fn a_failed_load_waits_out_the_retry_window() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root, "52.0.0");
    let patient = process(&base, assets.path(), Duration::from_mins(10));
    let eager = process(&base, assets.path(), Duration::ZERO);
    for ctx in [&patient, &eager] {
        ctx.snapshots
            .activate_at(Server::EN, &Generation::default())
            .await
            .unwrap();
    }
    std::fs::remove_file(root.join("gamedata/excel/character_table.json")).unwrap();
    base.announce_update(Server::EN, None).await.unwrap();
    for ctx in [&patient, &eager] {
        assert_eq!(ctx.reload_if_changed().await.unwrap(), []);
        assert_eq!(
            ctx.snapshots.require(Server::EN).unwrap().generation,
            0,
            "the old data stays"
        );
    }

    extract(&root, "52.0.1");
    assert_eq!(
        patient.reload_if_changed().await.unwrap(),
        [],
        "inside the window"
    );
    assert_eq!(eager.reload_if_changed().await.unwrap(), [Server::EN]);
    assert_eq!(eager.snapshots.require(Server::EN).unwrap().generation, 1);

    base.announce_update(Server::EN, None).await.unwrap();
    assert_eq!(
        patient.reload_if_changed().await.unwrap(),
        [Server::EN],
        "a new generation skips the wait"
    );
    db.close().await;
}

#[tokio::test]
async fn a_job_for_a_generation_this_process_cannot_load_fails_to_retry() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let root = assets.path().join("en");
    extract(&root, "52.0.0");
    let mut ctx = process(&base, assets.path(), Duration::ZERO);
    let mut config = (*ctx.config).clone();
    config.switches.release_ledger.enabled = true;
    ctx.config = Arc::new(config);
    ctx.snapshots
        .activate_at(Server::EN, &Generation::default())
        .await
        .unwrap();
    std::fs::remove_file(root.join("gamedata/excel/character_table.json")).unwrap();
    let generation = ctx.announce_update(Server::EN, Some("v1")).await.unwrap();

    assert!(record_ledger(&ctx, Server::EN, generation).await.is_err());
    assert!(mirror_art(&ctx, Server::EN, generation).await.is_err());
    assert!(reconcile_rarities(&ctx, generation).await.is_err());
    assert_eq!(ctx.snapshots.require(Server::EN).unwrap().generation, 0);

    extract(&root, "52.0.0");
    assert_eq!(
        record_ledger(&ctx, Server::EN, generation).await.unwrap(),
        "en: 0 inserted, 0 refreshed as v1"
    );
    assert_eq!(
        ctx.snapshots.require(Server::EN).unwrap().generation,
        1,
        "the job reloaded first"
    );
    assert!(
        record_ledger(&ctx, Server::EN, 0).await.is_ok(),
        "newer data serves an older job"
    );
    db.close().await;
}

#[tokio::test]
async fn a_reload_drops_only_its_own_servers_bodies() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"), "52.0.0");
    extract(&assets.path().join("jp"), "9.9.9");
    let ctx = serving(
        &base,
        assets.path(),
        Duration::from_mins(10),
        &[Server::EN, Server::JP],
    );
    for server in [Server::EN, Server::JP] {
        ctx.snapshots
            .activate_at(server, &Generation::default())
            .await
            .unwrap();
    }
    let builds = Arc::new(AtomicU32::new(0));
    let body = |key: &'static str| {
        let (ctx, builds) = (ctx.clone(), Arc::clone(&builds));
        async move {
            ctx.cache
                .shared(key, Duration::from_mins(5), move || async move {
                    builds.fetch_add(1, Ordering::SeqCst);
                    Ok("{}".to_owned())
                })
                .await
                .unwrap()
        }
    };
    body("static:en:operators:0:old").await;
    body("static:jp:operators:0:kept").await;
    ctx.cache
        .set("dps:calc:x", "memo", Duration::from_mins(5))
        .await;
    assert_eq!(builds.load(Ordering::SeqCst), 2);

    ctx.announce_update(Server::EN, None).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
    body("static:jp:operators:0:kept").await;
    assert_eq!(
        builds.load(Ordering::SeqCst),
        2,
        "another server's body stays"
    );
    body("static:en:operators:0:old").await;
    assert_eq!(
        builds.load(Ordering::SeqCst),
        3,
        "the old generation's body went"
    );
    assert_eq!(
        ctx.cache.get("dps:calc:x").await.as_deref(),
        Some("memo"),
        "content-keyed entries are never flushed"
    );
    db.close().await;
}

async fn counting_route() -> (String, Arc<AtomicU32>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let hits = Arc::new(AtomicU32::new(0));
    let counted = Arc::clone(&hits);
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            counted.fetch_add(1, Ordering::SeqCst);
            let _ = socket
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                .await;
        }
    });
    (url, hits)
}

#[tokio::test]
async fn a_default_server_reload_refreshes_the_network_config() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"), "52.0.0");
    extract(&assets.path().join("jp"), "9.9.9");
    let (route, hits) = counting_route().await;
    let ctx = AppContext {
        upstream: upstream(&route),
        ..serving(
            &base,
            assets.path(),
            Duration::from_mins(10),
            &[Server::EN, Server::JP],
        )
    };
    for server in [Server::EN, Server::JP] {
        ctx.snapshots
            .activate_at(server, &Generation::default())
            .await
            .unwrap();
    }
    ctx.announce_update(Server::JP, None).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::JP]);
    assert_eq!(hits.load(Ordering::SeqCst), 0, "not for another server");

    ctx.announce_update(Server::EN, None).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::EN]);
    assert!(hits.load(Ordering::SeqCst) > 0);
    db.close().await;
}

#[test]
fn each_data_server_is_loaded_once_the_default_first() {
    let mut config = common::config();
    config.server.servers = vec![Server::JP, Server::Bilibili, Server::CN, Server::EN];
    assert_eq!(
        SnapshotRegistry::new(&config).servers(),
        [Server::JP, Server::CN, Server::EN]
    );
}

#[tokio::test]
async fn a_text_only_server_reads_the_default_servers_art() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let (en, kr) = (assets.path().join("en"), assets.path().join("kr"));
    extract(&en, "52.0.0");
    write(&en, "portraits/char_002_amiya_1.png", "png");
    write(
        &en,
        "textures/ui_home_act_banner_gacha_x/pic_pool1.png",
        "png",
    );
    extract(&kr, "9.9.9");
    std::fs::remove_dir_all(kr.join("textures")).unwrap();

    for (fallback, art) in [(true, &en), (false, &kr)] {
        let mut config = (*base.config).clone();
        config.assets.dir = assets.path().to_str().unwrap().to_owned();
        config.server.servers = vec![Server::EN, Server::KR];
        config.switches.art_fallback.enabled = fallback;
        let snapshots = SnapshotRegistry::new(&config);
        assert_eq!(&snapshots.art_dir(Server::KR), art);
        snapshots
            .activate_at(Server::KR, &Generation::default())
            .await
            .unwrap();
        let live = snapshots.require(Server::KR).unwrap();
        assert_eq!(live.model.meta.game_version.client, "9.9.9", "KR's tables");
        assert_eq!(
            live.model.assets.gacha_banner_path("POOL1").is_some(),
            fallback
        );
        assert_eq!(
            live.model
                .assets
                .path(AssetKind::Portrait, "char_002_amiya_1")
                .is_some(),
            fallback
        );
        assert_eq!(live.story_dirs(), Some((art.as_path(), kr.as_path())));
    }
    db.close().await;
}

#[tokio::test]
async fn a_text_only_servers_key_names_the_servers_it_was_aligned_on() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"), "52.0.0");
    extract(&assets.path().join("jp"), "9.9.9");
    let jp_key = async |aligned_on_en: bool| {
        let ctx = serving(
            &base,
            assets.path(),
            Duration::from_mins(10),
            &[Server::EN, Server::JP],
        );
        if aligned_on_en {
            ctx.snapshots
                .activate_at(Server::EN, &Generation::default())
                .await
                .unwrap();
        }
        ctx.snapshots
            .activate_at(Server::JP, &Generation::default())
            .await
            .unwrap();
        let en = ctx.snapshots.get(Server::EN).map(|en| en.key("k"));
        (ctx.snapshots.require(Server::JP).unwrap().key("k"), en)
    };
    let (with_en, en) = jp_key(true).await;
    let (again, _) = jp_key(true).await;
    let (alone, _) = jp_key(false).await;
    assert_eq!(with_en, again, "one reference set, one key");
    assert_ne!(with_en, alone, "same tree and generation, other references");
    assert_eq!(
        en.unwrap(),
        format!("k:0-{}", fingerprint(&assets.path().join("en")).unwrap()),
        "the default server's key is its own"
    );
    db.close().await;
}

#[tokio::test]
async fn any_reload_takes_the_sidecar_flag_and_a_failed_one_sets_it() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    let kr = assets.path().join("kr");
    extract(&assets.path().join("en"), "52.0.0");
    extract(&kr, "9.9.9");
    let ctx = serving(
        &base,
        assets.path(),
        Duration::from_mins(10),
        &[Server::EN, Server::KR],
    );
    for server in [Server::EN, Server::KR] {
        ctx.snapshots
            .activate_at(server, &Generation::default())
            .await
            .unwrap();
    }
    mark_pending(Server::KR);
    ctx.announce_update(Server::KR, Some("v1")).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::KR]);
    assert!(
        !is_pending(Server::KR),
        "the pipeline reload read the sidecars"
    );

    std::fs::remove_dir_all(kr.join("gamedata")).unwrap();
    ctx.announce_update(Server::KR, Some("v2")).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), []);
    assert!(is_pending(Server::KR), "the files on disk are ahead again");
    db.close().await;
}

#[tokio::test]
async fn a_reload_warms_the_story_library_without_waiting_for_it() {
    let (base, db) = common::context().await;
    let assets = tempfile::tempdir().unwrap();
    extract(&assets.path().join("en"), "52.0.0");
    extract(&assets.path().join("jp"), "9.9.9");
    let ctx = serving(
        &base,
        assets.path(),
        Duration::from_mins(10),
        &[Server::EN, Server::JP],
    );
    for server in [Server::EN, Server::JP] {
        ctx.snapshots
            .activate_at(server, &Generation::default())
            .await
            .unwrap();
    }
    ctx.announce_update(Server::JP, None).await.unwrap();
    assert_eq!(ctx.reload_if_changed().await.unwrap(), [Server::JP]);
    let live = ctx.snapshots.require(Server::JP).unwrap();
    assert!(
        !live.story_built(),
        "the reload returned before the warm ran"
    );
    tokio::time::timeout(Duration::from_secs(30), async {
        while !live.story_built() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    db.close().await;
}
