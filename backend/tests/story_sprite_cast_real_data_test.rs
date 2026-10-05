//! The story cast on the real extracts: every character the scripts put on
//! stage has an entry, under the id it was saved with.
//!
//! 188 folders on EN and CN are a bare `avg_npc_NNN`, and cutting the
//! `_NNN` as a set number filed all of them under one id `avg_npc`: Maria
//! Nearl (`avg_npc_061`, 772 lines as "Maria") had no entry, nor did 186
//! other referenced cast members. Three more folders no file-name rule spells
//! (`char_242_mayer`, `char_253_greyy`, `char_2006_weiywfmzuki_1`) had none
//! either. The catalogue was 1,443 on EN before and 1,633 after (2026-10-05):
//! 183 bare folders under their own name, 4 under `_0` because a set-numbered
//! character already holds the name, and 3 hub-only folders. Skips where
//! the sprite tree is not on disk, as CI's artifact carries no textures.

mod common;

use std::path::{Path, PathBuf};

use backend::app::services::tier_entity::{FacetValue, catalogue, resolve, validate};
use backend::core::gamedata::assets::AssetIndex;
use backend::database::models::tier_list::EntityKind;

fn server_dir(server: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../assets/output/{server}"))
}

fn has_sprites(server: &str) -> bool {
    server_dir(server).join("textures/avg/characters").is_dir()
}

#[test]
fn en_story_cast_has_every_bare_avg_npc_character() {
    if !has_sprites("en") {
        eprintln!("no EN story sprite tree, skipping");
        return;
    }
    let gd = common::load_game_data();
    let assets = AssetIndex::build(&server_dir("en"));
    let cast = catalogue(gd, &assets, EntityKind::StorySprite);
    println!("EN story sprites {}", cast.len());
    assert!(cast.len() >= 1633, "{} story sprites", cast.len());

    let sprite = |id: &str| {
        resolve(gd, &assets, EntityKind::StorySprite, id)
            .unwrap_or_else(|| panic!("story sprite `{id}` does not resolve"))
    };
    let maria = sprite("avg_npc_061");
    assert_eq!(maria.name, "Maria");
    assert_eq!(
        maria.facets.get("source"),
        Some(&FacetValue::One("npc".to_owned()))
    );
    // Her other speaker name is an alias, so a search for it finds her: 93
    // of the 135 EN "Blemishine" lines are spoken over this sprite.
    match maria.facets.get("aliases") {
        Some(FacetValue::Many(aliases)) => assert!(
            aliases.iter().any(|a| a == "Blemishine"),
            "avg_npc_061 aliases {aliases:?}"
        ),
        other => panic!("avg_npc_061 aliases facet {other:?}"),
    }
    // No alias repeats the entry's own name or is a letterless `???`.
    for s in &cast {
        if let Some(FacetValue::Many(aliases)) = s.facets.get("aliases") {
            assert!(!aliases.is_empty() && aliases.len() <= 4, "{}", s.id);
            for a in aliases {
                assert!(a.chars().any(char::is_alphanumeric), "{}: {a}", s.id);
                assert!(!a.eq_ignore_ascii_case(&s.name), "{}: {a}", s.id);
            }
        }
    }
    assert!(validate([(gd, &assets)], EntityKind::StorySprite, "avg_npc_061").is_ok());
    assert!(cast.iter().any(|s| s.id == "avg_npc_061"));

    // The ids saved before the fix still resolve, to the same art.
    let legacy = sprite("avg_npc");
    assert_eq!(legacy.icon.as_deref(), Some("/story-sprite-thumb/avg_npc"));
    assert_eq!(
        assets
            .story_sprites()
            .get("avg_npc")
            .map(|s| s.folder.as_str()),
        Some("avg_npc_001")
    );
    for id in ["npc", "avg_doc"] {
        sprite(id);
    }
    // A bare folder sharing its number with a set-numbered character is
    // someone else, under `_0`; the sets keep the id and the name they had.
    assert_eq!(sprite("avg_npc_208").name, "Monique");
    assert_eq!(sprite("avg_npc_208_0").name, "Cannot");
    assert_eq!(sprite("avg_npc_213").name, "Toland");
    for id in ["avg_npc_102_0", "avg_npc_058_0", "avg_npc_213_0"] {
        sprite(id);
    }

    // Folders only the hub spells.
    for id in ["char_242_mayer", "char_253_greyy", "char_2006_weiywfmzuki"] {
        sprite(id);
    }

    // No other bare `avg_npc_NNN` folder is left without its own entry.
    let missing: Vec<String> = std::fs::read_dir(server_dir("en").join("textures/avg/characters"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_ascii_lowercase())
        .filter(|f| {
            f.strip_prefix("avg_npc_")
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .filter(|f| {
            f != "avg_npc_001"
                && assets.story_sprites().get(f).is_none()
                && assets.story_sprites().get(&format!("{f}_0")).is_none()
        })
        .collect();
    assert!(
        missing.is_empty(),
        "bare folders with no entry: {missing:?}"
    );
}

#[test]
fn cn_story_cast_has_maria() {
    if !has_sprites("cn") {
        eprintln!("no CN story sprite tree, skipping");
        return;
    }
    let assets = AssetIndex::build(&server_dir("cn"));
    let sprites = assets.story_sprites();
    println!("CN story sprites {}", sprites.len());
    let maria = sprites.get("avg_npc_061").expect("avg_npc_061 on CN");
    // No speaker check: the CN extract's story tree is mostly synopses (34
    // of 5,385 files carry a `Character` command), so its cast goes unnamed.
    assert_eq!(maria.folder, "avg_npc_061");
    assert_eq!(
        sprites.get("avg_npc").map(|s| s.folder.as_str()),
        Some("avg_npc_001")
    );
}
