#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::time::Instant;

use myrtle_app::AppError;
use myrtle_app::services::catalog::archives::scene_index;
use myrtle_app::services::story;
use myrtle_app::snapshots::Snapshot;
use myrtle_engine_story::dto::{StoryArtKind, StoryCategory};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_types::Server;

fn empty_snapshot() -> Arc<Snapshot> {
    Arc::new(Snapshot::from_model(
        GameModel::default(),
        SandboxUniverse::default(),
    ))
}

#[tokio::test]
async fn a_server_with_no_data_is_not_found() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        story::get_story_index(&ctx, Server::EN).await,
        Err(AppError::NotFound(Some(_)))
    ));
    assert!(matches!(
        story::gallery_shared(&ctx, Server::EN).await,
        Err(AppError::NotFound(Some(_)))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_model_only_snapshot_answers_empty_and_refuses_unknown_ids() {
    let (ctx, db) = common::context().await;
    ctx.snapshots.install_shared(Server::EN, empty_snapshot());

    let index = story::get_story_index(&ctx, Server::EN).await.unwrap();
    assert!(index.groups.is_empty());
    assert!(
        story::get_story(&ctx, Server::EN, "no_such_story")
            .await
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        story::get_group_illustrations(&ctx, Server::EN, "main_0").await,
        Err(AppError::NotFound(Some(m))) if m.contains("main_0")
    ));
    assert!(matches!(
        story::get_group_archive(&ctx, Server::EN, "main_0").await,
        Err(AppError::NotFound(Some(_)))
    ));
    assert!(matches!(
        story::get_story_sprite(&ctx, Server::EN, "avg_npc_043_1").await,
        Err(AppError::NotFound(Some(m))) if m.contains("avg_npc_043_1")
    ));
    assert!(matches!(
        story::get_variant_thumb(&ctx, Server::EN, "avg_npc_043_1", "#1$1").await,
        Err(AppError::NotFound(None))
    ));
    assert!(
        story::get_gallery(&ctx, Server::EN)
            .await
            .unwrap()
            .groups
            .is_empty()
    );
    assert!(
        story::get_story_art(&ctx, Server::EN, StoryArtKind::Cg)
            .await
            .unwrap()
            .groups
            .is_empty()
    );
    assert!(!story::story_art_known(&ctx, StoryArtKind::Scene, "bg_black").await);
    assert!(matches!(
        story::ensure_picture(
            &ctx,
            Server::EN,
            "act13side_pic_0",
            myrtle_engine_story::gallery::GallerySize::Thumb
        )
        .await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_readers_share_one_build_and_a_reload_retires_it() {
    let (ctx, db) = common::context().await;
    let first = empty_snapshot();
    ctx.snapshots.install_shared(Server::EN, Arc::clone(&first));

    let mut tasks = Vec::new();
    for _ in 0..8 {
        let ctx = ctx.clone();
        tasks.push(tokio::spawn(async move {
            story::gallery_shared(&ctx, Server::EN).await.unwrap()
        }));
    }
    let mut built = Vec::new();
    for task in tasks {
        built.push(task.await.unwrap());
    }
    assert!(built.iter().all(|g| Arc::ptr_eq(g, &built[0])));
    assert!(first.story_built());

    let second = empty_snapshot();
    ctx.snapshots
        .install_shared(Server::EN, Arc::clone(&second));
    assert!(!second.story_built());
    let rebuilt = story::gallery_shared(&ctx, Server::EN).await.unwrap();
    assert!(!Arc::ptr_eq(&rebuilt, &built[0]));
    assert!(second.story_built());
    assert!(Arc::ptr_eq(
        &story::snapshot(&ctx, Server::EN).await.unwrap(),
        &second
    ));
    db.close().await;
}

#[tokio::test]
async fn variants_try_the_server_then_the_default_then_cn() {
    let (ctx, db) = common::context().await;
    let cn = ctx.config.switches.asset_cn_fallback.enabled;
    let with_cn = |mut v: Vec<Server>| {
        if cn {
            v.push(Server::CN);
        }
        v
    };
    assert_eq!(
        story::with_default_fallback(&ctx, Server::EN),
        with_cn(vec![Server::EN])
    );
    assert_eq!(
        story::with_default_fallback(&ctx, Server::JP),
        with_cn(vec![Server::JP, Server::EN])
    );
    assert_eq!(
        story::with_default_fallback(&ctx, Server::CN),
        vec![Server::CN, Server::EN]
    );
    assert_eq!(
        story::with_default_fallback(&ctx, Server::Bilibili),
        vec![Server::Bilibili, Server::EN]
    );
    db.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_en_library_galleries_and_sprites() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let started = Instant::now();
    let index = story::get_story_index(&ctx, Server::EN).await.unwrap();
    eprintln!("story library: {} ms", started.elapsed().as_millis());
    assert!(index.groups.len() > 400, "{} groups", index.groups.len());
    assert!(
        index
            .groups
            .iter()
            .any(|g| g.category == StoryCategory::Main)
    );

    let entry = index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .find(|s| s.has_script)
        .unwrap();
    let parsed = story::get_story(&ctx, Server::EN, &entry.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(parsed.word_count, entry.word_count);

    let illustrations = story::get_group_illustrations(&ctx, Server::EN, "main_0")
        .await
        .unwrap();
    assert!(!illustrations.backgrounds.is_empty());
    let archive = story::get_group_archive(&ctx, Server::EN, "main_14")
        .await
        .unwrap();
    assert!(!archive.sections.is_empty());

    let gallery = story::get_gallery(&ctx, Server::EN).await.unwrap();
    assert!(!gallery.groups.is_empty());

    let started = Instant::now();
    let sprites = story::sprites_shared(&ctx, Server::EN).await.unwrap();
    eprintln!("story sprites: {} ms", started.elapsed().as_millis());
    let has_avg = myrtle_testkit::fixtures::has_story_avg();
    if has_avg {
        let first = &sprites.index.sprites[0];
        let detail = story::get_story_sprite(&ctx, Server::EN, &first.base.to_ascii_uppercase())
            .await
            .unwrap();
        assert_eq!(detail.sprite.base, first.base);
        assert!(!scene_index(&sprites, &snapshot.story().unwrap().library.index).is_empty());
    } else {
        eprintln!("skipping the sprite and scene checks: no en/textures/avg");
    }

    let art = story::story_art_shared(&ctx, Server::EN).await.unwrap();
    assert!(!has_avg || art.census(StoryArtKind::Scene).distinct > 0);
    if let Some(picture) = art
        .gallery(StoryArtKind::Scene)
        .groups
        .first()
        .and_then(|g| g.pictures.first())
    {
        assert!(story::story_art_known(&ctx, StoryArtKind::Scene, &picture.id).await);
    }
    assert!(!story::story_art_known(&ctx, StoryArtKind::Cg, "no_such_plate").await);
    db.close().await;
}
