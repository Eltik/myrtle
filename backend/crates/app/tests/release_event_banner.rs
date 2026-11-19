#![allow(clippy::unwrap_used)]

mod common;

use std::path::Path;
use std::sync::Arc;

use base64::Engine;
use myrtle_app::config::CpuConfig;
use myrtle_app::limits::CpuLimiter;
use myrtle_app::services::catalog::release::event_banner;
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_app::{AppContext, AppError};
use myrtle_engine_release::event_banner::{HEIGHT, WIDTH, cache_rel_path};
use myrtle_gamedata::assets::AssetIndex;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_gamedata_raw::tables::stage::{StorySetEventData, StorylineStorySet};
use myrtle_testkit::db::TestDb;
use myrtle_types::Server;

/// An 8x4 opaque RGBA PNG
const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAECAYAAACzzX7wAAAAEklEQVR4nGM4kWL0Hx9moL0CAHmxS6FrGgA4AAAAAElFTkSuQmCC";
const ACT: &str = "act9test";
const BACKGROUND: &str = "textures/spritepack/mixstory_retro_bkg_sprites_0/storybg_test.png";
const TITLE: &str = "textures/spritepack/mixstory_title_sprites_0/title_test.png";

fn png() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(PNG)
        .unwrap()
}

/// A context whose extracts live under `dir`, with `servers` configured and none loaded
async fn context(dir: &Path, servers: &[Server]) -> (AppContext, TestDb) {
    let (mut ctx, db) = common::context().await;
    let mut config = (*ctx.config).clone();
    config.assets.dir = dir.to_str().unwrap().to_owned();
    config.server.servers = servers.to_vec();
    config.switches.asset_cn_fallback.enabled = true;
    ctx.snapshots = Arc::new(SnapshotRegistry::new(&config));
    ctx.config = Arc::new(config);
    (ctx, db)
}

/// Installs a `server` snapshot whose story set covers [`ACT`], its art written with `art` into its art tree
fn install(ctx: &AppContext, server: Server, art: &[u8]) {
    let root = ctx.snapshots.art_dir(server);
    for rel in [BACKGROUND, TITLE] {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, art).unwrap();
    }
    let mut model = GameModel {
        assets: AssetIndex::from_paths(&[BACKGROUND.to_owned(), TITLE.to_owned()]),
        ..Default::default()
    };
    model.storyline_story_sets.insert(
        "set".to_owned(),
        StorylineStorySet {
            story_set_id: "set".to_owned(),
            relevant_activity_id: Some(ACT.to_owned()),
            title_image_id: Some("title_test".to_owned()),
            ss_data: Some(StorySetEventData {
                background_id: Some("storybg_test".to_owned()),
                reopen_activity_id: None,
            }),
            ..Default::default()
        },
    );
    ctx.snapshots.install(
        server,
        Snapshot::from_model(model, SandboxUniverse::default()),
    );
}

fn assert_banner(path: &Path) {
    let jpeg = std::fs::read(path).unwrap();
    assert!(jpeg.starts_with(&[0xFF, 0xD8, 0xFF]), "not a JPEG");
    let sof = jpeg.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
    let dim = |at: usize| u32::from(u16::from_be_bytes([jpeg[at], jpeg[at + 1]]));
    assert_eq!((dim(sof + 7), dim(sof + 5)), (WIDTH, HEIGHT));
}

#[tokio::test]
async fn an_unsafe_or_unknown_activity_id_is_not_found() {
    let (ctx, db) = common::context().await;
    for id in ["", "../etc", ".hidden", "a/b", "act17side"] {
        assert!(
            matches!(
                event_banner(&ctx, Server::EN, id).await,
                Err(AppError::NotFound(None))
            ),
            "{id:?}"
        );
    }
    db.close().await;
}

#[tokio::test]
async fn an_evicted_stub_answers_not_found_without_rendering() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db) = context(dir.path(), &[Server::EN]).await;
    install(&ctx, Server::EN, b"");
    let res = event_banner(&ctx, Server::EN, ACT).await;
    assert!(matches!(res, Err(AppError::NotFound(None))), "{res:?}");
    let target = ctx
        .snapshots
        .server_dir(Server::EN)
        .join(cache_rel_path(ACT));
    assert!(!target.exists());
    db.close().await;
}

#[tokio::test]
async fn a_banner_renders_once_into_the_servers_derived_tree() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db) = context(dir.path(), &[Server::EN]).await;
    install(&ctx, Server::EN, &png());
    let path = event_banner(&ctx, Server::EN, ACT).await.unwrap();
    assert_eq!(
        path,
        ctx.snapshots
            .server_dir(Server::EN)
            .join(cache_rel_path(ACT))
    );
    assert_banner(&path);
    let made = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(event_banner(&ctx, Server::EN, ACT).await.unwrap(), path);
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        made,
        "a fresh banner is served, not redrawn"
    );
    db.close().await;
}

#[tokio::test]
async fn a_set_only_cn_knows_renders_from_the_cn_tree() {
    let dir = tempfile::tempdir().unwrap();
    let (ctx, db) = context(dir.path(), &[Server::EN, Server::CN]).await;
    std::fs::create_dir_all(ctx.snapshots.server_dir(Server::CN).join("portraits")).unwrap(); // its own art tree
    install(&ctx, Server::CN, &png());
    let path = event_banner(&ctx, Server::EN, ACT).await.unwrap();
    assert!(path.starts_with(ctx.snapshots.server_dir(Server::CN)));
    assert_banner(&path);
    db.close().await;
}

#[tokio::test]
async fn a_banner_waits_for_the_cpu_instead_of_shedding() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ctx, db) = context(dir.path(), &[Server::EN]).await;
    ctx.cpu = CpuLimiter::new(&CpuConfig {
        task_permits: 1,
        light_permits: 1,
        task_wait_ms: 0,
        task_queue: 0,
    });
    let _busy = ctx.cpu.admit("busy").await.unwrap();
    assert!(matches!(
        ctx.cpu.admit("probe").await,
        Err(AppError::Unavailable)
    ));
    install(&ctx, Server::EN, &png());
    assert_banner(&event_banner(&ctx, Server::EN, ACT).await.unwrap());
    db.close().await;
}

#[tokio::test]
async fn a_text_only_server_renders_from_the_shared_art_tree() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ctx, db) = context(dir.path(), &[Server::EN, Server::KR]).await;
    let mut config = (*ctx.config).clone();
    config.switches.art_fallback.enabled = true;
    ctx.snapshots = Arc::new(SnapshotRegistry::new(&config));
    ctx.config = Arc::new(config);
    assert!(!ctx.snapshots.has_own_art(Server::KR));
    install(&ctx, Server::KR, &png());
    let path = event_banner(&ctx, Server::KR, ACT).await.unwrap();
    assert_eq!(
        path,
        ctx.snapshots
            .server_dir(Server::EN)
            .join(cache_rel_path(ACT)),
        "one banner file serves every server reading the EN art"
    );
    assert_banner(&path);
    assert!(!ctx.snapshots.server_dir(Server::KR).exists());
    db.close().await;
}
