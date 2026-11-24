//! The library, galleries and reader over the local EN extract; skips when it is absent

#![allow(clippy::unwrap_used, clippy::expect_used)]

use myrtle_engine_story::assets::served_rel;
use myrtle_engine_story::dto::{StoryArchiveSection, StoryCategory};
use myrtle_engine_story::gallery::build_gallery;
use myrtle_engine_story::index::mainline_chapters;
use myrtle_engine_story::sprites::build_sprite_index;
use myrtle_engine_story::story_art::{MIN_BYTES_PER_PX, build_story_art};
use myrtle_engine_story::{StoryAssetIndex, build_index};
use myrtle_gamedata_raw::{DirSource, TableSource};
use myrtle_testkit::fixtures;
use myrtle_types::Server;

#[test]
fn the_en_library_galleries_and_reader() {
    let root = fixtures::root(Server::EN);
    if !root.join("textures/avg").is_dir() {
        eprintln!("skip: no EN extract at {}", root.display());
        return;
    }
    let Some(model) = fixtures::model(Server::EN) else {
        eprintln!("skip: no EN extract at {}", root.display());
        return;
    };
    let src = DirSource { root: root.clone() };
    #[allow(clippy::disallowed_methods)] // the build time is the measurement
    let started = std::time::Instant::now();
    let assets = StoryAssetIndex::build(&src, &src, &|tree| {
        DirSource {
            root: src.root.join(tree),
        }
        .list_assets()
        .unwrap_or_default()
    })
    .unwrap();
    eprintln!(
        "story asset index built in {} ms",
        started.elapsed().as_millis()
    );
    assert!(assets.resolve_background("bg_cher_1").is_some());
    assert!(assets.resolve_character("char_002_amiya_1").is_some());
    assert!(assets.resolve_avatar("$avatar_amiya").is_some());

    let lib = build_index(model, &model.assets, &assets, &src);
    let groups = &lib.index.groups;
    assert!(groups.len() > 400, "{} groups", groups.len());
    assert!(lib.index.records.len() > 300);
    assert!(
        lib.index.totals.with_script > 1500,
        "{:?}",
        lib.index.totals
    );
    let mains: Vec<_> = groups
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
        .collect();
    assert!(mains.len() >= 17);
    assert!(
        mains
            .iter()
            .all(|g| g.chapter_number.is_some() && g.cover_url.is_some())
    );
    assert!(groups.iter().filter(|g| g.music.is_some()).count() > 80);
    assert!(!lib.index.storylines.is_empty());
    assert!(lib.by_txt.len() > 1500);
    assert!(lib.gates.values().any(|g| !g.is_empty()));

    let chapters = mainline_chapters(model, &model.assets);
    assert_eq!(chapters.len(), mains.len());
    assert_eq!(chapters[0].number, Some(0));

    let main_14 = &lib.archives["main_14"];
    assert!(
        main_14
            .sections
            .iter()
            .any(|s| matches!(s, StoryArchiveSection::Recordings { .. }))
    );
    assert!(lib.archives.contains_key("act13side"));

    let story = groups[0].stories.iter().find(|s| s.has_script).unwrap();
    let parsed = lib.load_story(&src, &assets, &story.id).unwrap().unwrap();
    assert_eq!(parsed.word_count, story.word_count);
    assert!(!parsed.assets.characters.is_empty());
    assert!(
        lib.load_story(&src, &assets, "no_such_story")
            .unwrap()
            .is_none()
    );
    let ill = lib.illustrations(&assets, &groups[0].id).unwrap();
    assert!(ill.backgrounds.iter().any(|b| b.url.is_some()));

    let gallery = build_gallery(model, &assets);
    assert!(
        gallery
            .groups
            .iter()
            .map(|g| g.pictures.len())
            .sum::<usize>()
            > 200
    );

    let file_len = |url: &str| {
        std::fs::metadata(root.join(served_rel(url)))
            .ok()
            .map(|m| m.len())
    };
    let [(cg, cg_census), (scene, _)] = build_story_art(&lib, &assets, &file_len, MIN_BYTES_PER_PX);
    assert!(cg_census.passed > 1000, "{cg_census:?}");
    assert!(!cg.groups.is_empty() && !scene.groups.is_empty());

    let sprites = build_sprite_index(model, &lib, &assets, &src, &src, &file_len);
    assert!(sprites.index.sprites.len() > 1000);
    assert!(sprites.census.hidden.iter().any(|f| f == "char_empty"));
    let amiya = &sprites.details["char_002_amiya_1"];
    assert_eq!(amiya.sprite.char_id.as_deref(), Some("char_002_amiya"));
    assert!(amiya.sprite.lines > 100.0);
}
