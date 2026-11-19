#![allow(clippy::unwrap_used)]

mod common;

use std::fs::FileTimes;
use std::path::Path;
use std::time::{Duration, SystemTime};

use base64::Engine;
use myrtle_app::AppError;
use myrtle_app::services::story;
use myrtle_engine_story::assets::CharacterSprite;
use myrtle_engine_story::dto::StorySpriteVariant;
use myrtle_engine_story::gallery::{GallerySize, cache_rel_path};
use myrtle_engine_story::thumbs::{FORMAT, ThumbFormat, head_rel_path, variant_rel_path};

/// An 8x4 opaque RGBA PNG
const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAgAAAAECAYAAACzzX7wAAAAEklEQVR4nGM4kWL0Hx9moL0CAHmxS6FrGgA4AAAAAElFTkSuQmCC";

fn png() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(PNG)
        .unwrap()
}

fn put(root: &Path, rel: &str, bytes: &[u8]) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn age(path: &Path, ago: Duration) {
    let file = std::fs::OpenOptions::new().write(true).open(path).unwrap();
    file.set_times(FileTimes::new().set_modified(SystemTime::now() - ago))
        .unwrap();
}

fn leftovers(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .count()
}

#[test]
fn an_atomic_write_creates_its_folder_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("derived/a/b.jpg");
    story::write_atomically(&target, b"one").unwrap();
    story::write_atomically(&target, b"two").unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"two");
    assert_eq!(leftovers(target.parent().unwrap()), 0);
}

#[test]
fn only_an_old_hit_has_its_mtime_refreshed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("x.jpg");
    std::fs::write(&path, b"x").unwrap();
    age(&path, Duration::from_hours(48));
    story::touch_if_old(&path);
    let refreshed = story::modified(&path).unwrap();
    assert!(SystemTime::now().duration_since(refreshed).unwrap() < Duration::from_mins(1));

    age(&path, Duration::from_hours(2));
    let recent = story::modified(&path).unwrap();
    story::touch_if_old(&path);
    assert_eq!(story::modified(&path).unwrap(), recent);
    assert!(story::modified(&dir.path().join("missing")).is_none());
}

#[tokio::test]
async fn a_variant_renders_once_and_again_after_its_source_changes() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    put(&root, "textures/avg/imgs/pic#1.png", &png());
    let rel = cache_rel_path("pic_1", GallerySize::Thumb);

    let (served_root, served) = story::ensure_variant(
        &ctx,
        root.clone(),
        "/textures/avg/imgs/pic%231.png",
        rel.clone(),
        GallerySize::Thumb,
    )
    .await
    .unwrap();
    assert_eq!(
        (served_root.as_path(), served.as_str()),
        (root.as_path(), rel.as_str())
    );
    let target = root.join(&rel);
    assert_eq!(&std::fs::read(&target).unwrap()[..2], [0xFF, 0xD8]);

    age(
        &root.join("textures/avg/imgs/pic#1.png"),
        Duration::from_hours(2),
    );
    age(&target, Duration::from_hours(1));
    let kept = story::modified(&target).unwrap();
    story::ensure_variant(
        &ctx,
        root.clone(),
        "/textures/avg/imgs/pic%231.png",
        rel.clone(),
        GallerySize::Thumb,
    )
    .await
    .unwrap();
    assert_eq!(
        story::modified(&target).unwrap(),
        kept,
        "a fresh hit is not re-rendered"
    );

    put(&root, "textures/avg/imgs/pic#1.png", &png());
    story::ensure_variant(
        &ctx,
        root.clone(),
        "/textures/avg/imgs/pic%231.png",
        rel.clone(),
        GallerySize::Thumb,
    )
    .await
    .unwrap();
    assert!(
        story::modified(&target).unwrap() > kept,
        "a newer source re-renders"
    );
    db.close().await;
}

#[tokio::test]
async fn a_failed_render_serves_the_original() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    put(dir.path(), "textures/avg/imgs/broken.png", b"not a png");
    let (_, served) = story::ensure_variant(
        &ctx,
        dir.path().to_path_buf(),
        "/textures/avg/imgs/broken.png",
        cache_rel_path("broken", GallerySize::Header),
        GallerySize::Header,
    )
    .await
    .unwrap();
    assert_eq!(served, "textures/avg/imgs/broken.png");
    db.close().await;
}

#[tokio::test]
async fn a_head_crop_refuses_a_path_id_and_renders_otherwise() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    let body = "/textures/avg/characters/npc/npc.png";
    put(dir.path(), body.trim_start_matches('/'), &png());
    for id in ["", "../x", "a/b", "a\\b", ".hidden"] {
        assert!(matches!(
            story::ensure_head(&ctx, dir.path(), id, body, None, ThumbFormat::Webp).await,
            Err(AppError::NotFound(None))
        ));
    }
    let rel = story::ensure_head(&ctx, dir.path(), "npc", body, None, ThumbFormat::Png)
        .await
        .unwrap();
    assert_eq!(rel, head_rel_path("npc", ThumbFormat::Png));
    assert!(dir.path().join(&rel).is_file());
    assert!(matches!(
        story::ensure_head(&ctx, dir.path(), "gone", "/missing.png", None, FORMAT).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn concurrent_head_crops_of_one_id_all_succeed() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    let body = "/textures/avg/characters/race/race.png";
    put(dir.path(), body.trim_start_matches('/'), &png());
    let target = dir.path().join(head_rel_path("race", FORMAT));
    for _ in 0..20 {
        let _ = std::fs::remove_file(&target);
        let (a, b) = tokio::join!(
            story::ensure_head(&ctx, dir.path(), "race", body, None, FORMAT),
            story::ensure_head(&ctx, dir.path(), "race", body, None, FORMAT),
        );
        assert!(a.is_ok() && b.is_ok());
        assert!(target.is_file());
    }
    assert_eq!(leftovers(target.parent().unwrap()), 0);
    db.close().await;
}

#[tokio::test]
async fn an_expression_thumb_composes_body_and_face() {
    let (ctx, db) = common::context().await;
    let dir = tempfile::tempdir().unwrap();
    put(
        dir.path(),
        "textures/avg/characters/npc/npc#1$1.png",
        &png(),
    );
    put(dir.path(), "textures/avg/characters/npc/face.png", &png());
    let variant = StorySpriteVariant {
        key: "#1$1".to_owned(),
        sprite: CharacterSprite {
            body_url: "/textures/avg/characters/npc/npc%231$1.png".to_owned(),
            face_url: Some("/textures/avg/characters/npc/face.png".to_owned()),
            face_pos: None,
            body_size: None,
            plate: None,
        },
        whole_body: false,
        uses: 1,
    };
    let rel = story::ensure_variant_thumb(&ctx, dir.path(), "NPC", &variant)
        .await
        .unwrap();
    assert_eq!(rel, variant_rel_path("NPC", "#1$1", FORMAT));
    assert!(dir.path().join(&rel).is_file());

    let missing = StorySpriteVariant {
        key: "#2$1".to_owned(),
        sprite: CharacterSprite {
            body_url: "/textures/avg/characters/npc/none.png".to_owned(),
            ..variant.sprite.clone()
        },
        ..variant
    };
    assert!(matches!(
        story::ensure_variant_thumb(&ctx, dir.path(), "npc", &missing).await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

fn write(root: &Path, rel: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, png()).unwrap();
}

#[test]
fn a_story_tree_lists_each_directory_in_name_order_and_the_first_of_a_stem_wins() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    for rel in [
        "textures/avg/bg/bg_x.png",
        "textures/avg/bg/bg_x/bg_x.png",
        "textures/avg/bg/.hidden.png",
    ] {
        write(root, rel);
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        root.join("textures/avg/bg/bg_x.png"),
        root.join("textures/avg/bg/link.png"),
    )
    .unwrap();
    let listed = story::list_story_tree(root, "textures/avg/bg");
    let mut want = vec![".hidden.png", "bg_x/bg_x.png", "bg_x.png"];
    if cfg!(unix) {
        want.push("link.png");
    }
    assert_eq!(listed, want);
    assert!(story::list_story_tree(root, "video").is_empty());

    let src = myrtle_gamedata_raw::DirSource {
        root: root.to_path_buf(),
    };
    let idx = myrtle_engine_story::StoryAssetIndex::build(&src, &src, &|rel| {
        story::list_story_tree(root, rel)
    })
    .unwrap();
    assert_eq!(
        idx.resolve_background("bg_x").unwrap().0,
        "/textures/avg/bg/bg_x/bg_x.png"
    );
    assert!(idx.resolve_background(".hidden").is_some());
}

#[cfg(unix)]
#[test]
fn an_unreadable_directory_costs_only_its_own_files() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    write(root, "textures/spritepack/act_locked/x.png");
    write(root, "textures/spritepack/ui_char_avatar_0/char_1.png");
    write(root, "textures/spritepack/zz_pack/z.png");
    let locked = root.join("textures/spritepack/act_locked");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();
    let readable = std::fs::read_dir(&locked).is_ok();
    let listed = story::list_story_tree(root, "textures/spritepack");
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    if readable {
        return; // running as root: permissions lock nothing
    }
    assert_eq!(listed, ["ui_char_avatar_0/char_1.png", "zz_pack/z.png"]);
}
