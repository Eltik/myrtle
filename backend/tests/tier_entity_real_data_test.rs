//! Every tier list entity kind's catalogue, read off the real EN extract.
//!
//! Floors each kind at the count it was specified against (EN tables of
//! 2026-09-05): game updates only add entities, so a count below its floor means
//! an inclusion rule dropped some. The floors are not exact because CI runs
//! against the newest `game-data` artifact, not the extract they were measured
//! on. Also proves every offered entity resolves and validates, and prints how
//! many have no art, which the client draws as a placeholder. CI's artifact
//! carries no textures, so story sprites are only counted where the sprite tree
//! is on disk.

mod common;

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use backend::app::error::ApiError;
use backend::app::services::tier_entity::{FacetValue, catalogue, known, resolve, validate};
use backend::core::gamedata::assets::AssetIndex;
use backend::database::models::tier_list::EntityKind;

fn en_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en")
}

fn en_assets() -> AssetIndex {
    AssetIndex::build(&en_dir())
}

fn story_sprites_present() -> bool {
    en_dir().join("textures/avg/characters").is_dir()
}

fn facet<'a>(facets: &'a BTreeMap<String, FacetValue>, name: &str) -> Option<&'a str> {
    match facets.get(name)? {
        FacetValue::One(s) => Some(s),
        FacetValue::Many(v) => v.first().map(String::as_str),
    }
}

#[test]
fn every_kind_catalogue_reaches_the_measured_counts() {
    let gd = common::load_game_data();
    let assets = en_assets();

    let floors: [(EntityKind, usize); 12] = [
        (EntityKind::Class, 8),
        (EntityKind::Subclass, 71),
        (EntityKind::Enemy, 1541),
        (EntityKind::Event, 138),
        (EntityKind::Faction, 45),
        (EntityKind::StrongholdBond, 23),
        (EntityKind::Skin, 474),
        (EntityKind::Module, 467),
        (EntityKind::Skill, 901),
        (EntityKind::IntegratedStrategies, 1681),
        (EntityKind::StorySprite, 1443),
        (EntityKind::MainStory, 17),
    ];
    let mut failures = Vec::new();
    for kind in EntityKind::ALL.iter().copied() {
        let cat = catalogue(gd, &assets, kind);
        let ids: HashSet<&str> = cat.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids.len(), cat.len(), "{} ids repeat", kind.as_str());
        let mut no_icon = Vec::new();
        for s in &cat {
            assert_eq!(s.kind, kind);
            let again = resolve(gd, &assets, kind, &s.id).unwrap_or_else(|| {
                panic!(
                    "{} `{}` is offered but does not resolve",
                    kind.as_str(),
                    s.id
                )
            });
            assert_eq!(&again, s, "catalogue and resolve disagree on {}", s.id);
            assert!(known(gd, &assets, kind, &s.id));
            assert!(
                !s.name.is_empty(),
                "{} `{}` has no name",
                kind.as_str(),
                s.id
            );
            if s.icon.is_none() {
                no_icon.push(s.id.clone());
            }
        }
        println!(
            "{:<16} {:>5} offered, {:>3} without icon {:?}",
            kind.as_str(),
            cat.len(),
            no_icon.len(),
            no_icon.iter().take(40).collect::<Vec<_>>()
        );
        if kind == EntityKind::StorySprite && !story_sprites_present() {
            continue;
        }
        if let Some((_, floor)) = floors.iter().find(|(k, _)| *k == kind)
            && cat.len() < *floor
        {
            failures.push(format!(
                "{}: {} offered, expected at least {floor}",
                kind.as_str(),
                cat.len()
            ));
        }
    }
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn the_event_pool_by_display_type() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let events = catalogue(gd, &assets, EntityKind::Event);
    let mut by_display: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for e in &events {
        let entry = by_display
            .entry(facet(&e.facets, "display_type").unwrap_or("?"))
            .or_default();
        entry.0 += 1;
        if e.icon.is_none() {
            entry.1 += 1;
        }
    }
    println!(
        "events {} of {} activities",
        events.len(),
        gd.activities.len()
    );
    for (display, (n, no_icon)) in &by_display {
        println!("  {display:<12} {n:>4} ({no_icon} without art)");
    }
    // Grani's event files its Archives cover under `act1d0`, not its activity id.
    let grani = events
        .iter()
        .find(|e| e.id == "1stact")
        .expect("1stact offered");
    assert!(
        grani
            .icon
            .as_deref()
            .is_some_and(|i| i.ends_with("storyEntryPic_act1d0.png")),
        "1stact icon: {:?}",
        grani.icon
    );
    for (display, floor) in [
        ("SIDESTORY", 64),
        ("MINISTORY", 20),
        ("BRANCHLINE", 6),
        ("NONE", 48),
    ] {
        let n = by_display.get(display).map_or(0, |c| c.0);
        assert!(
            n >= floor,
            "{display}: {n} events, expected at least {floor}"
        );
    }
}

#[test]
fn hidden_enemies_resolve_but_are_not_offered() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let hidden: Vec<&str> = gd
        .enemies
        .enemy_data
        .iter()
        .filter(|(_, e)| e.hide_in_handbook)
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(hidden.len(), 49);
    let offered: HashSet<String> = catalogue(gd, &assets, EntityKind::Enemy)
        .into_iter()
        .map(|s| s.id)
        .collect();
    for id in hidden {
        assert!(!offered.contains(id), "{id} is hidden but offered");
        assert!(resolve(gd, &assets, EntityKind::Enemy, id).is_some());
        assert!(
            validate([(gd, &assets)], EntityKind::Enemy, id).is_ok(),
            "{id} must validate"
        );
    }
}

#[test]
fn an_unknown_id_is_a_bad_request_for_every_kind() {
    let gd = common::load_game_data();
    let assets = en_assets();
    for kind in EntityKind::ALL.iter().copied() {
        match validate([(gd, &assets)], kind, "no_such_entity_9999") {
            Err(ApiError::BadRequest(msg)) => {
                assert!(
                    msg.contains(kind.as_str()) && msg.contains("no_such_entity_9999"),
                    "{msg}"
                );
            }
            other => panic!("{}: expected BadRequest, got {other:?}", kind.as_str()),
        }
    }
    // Ids that exist in the table but are not entities of the kind.
    assert!(validate([(gd, &assets)], EntityKind::Faction, "none").is_err());
    assert!(validate([(gd, &assets)], EntityKind::Subclass, "notchar1").is_err());
    assert!(validate([(gd, &assets)], EntityKind::Class, "TOKEN").is_err());
    // Kinds do not share an id space: an operator id is not an enemy.
    assert!(validate([(gd, &assets)], EntityKind::Enemy, "char_002_amiya").is_err());
}

#[test]
fn the_integrated_strategies_pool_by_theme_and_type() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let entries = catalogue(gd, &assets, EntityKind::IntegratedStrategies);
    let mut by: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    for e in &entries {
        let key = (
            facet(&e.facets, "theme").unwrap_or("?").to_owned(),
            facet(&e.facets, "item_type").unwrap_or("?").to_owned(),
        );
        let entry = by.entry(key).or_default();
        entry.0 += 1;
        if e.icon.is_none() {
            entry.1 += 1;
        }
    }
    for ((theme, item_type), (n, no_icon)) in &by {
        println!("  {theme} {item_type:<13} {n:>5} ({no_icon} without icon)");
    }
    let count = |theme: &str, item_type: &str| {
        by.get(&(theme.to_owned(), item_type.to_owned()))
            .map_or(0, |c| c.0)
    };
    // rogue_N is IS N+1: five themes, IS2 to IS6, each listed once.
    for n in 1..=5 {
        assert_eq!(count(&format!("rogue_{n}"), "theme"), 1);
    }
    assert_eq!(count("rogue_1", "capsule"), 26, "IS2 Plays");
    assert_eq!(
        count("rogue_1", "active_tool"),
        6,
        "IS2 tools sit in the Trap archive"
    );
    assert_eq!(count("rogue_3", "totem"), 43, "IS4 Foldartals");
    assert_eq!(count("rogue_4", "fragment"), 54, "IS5 Thoughts");
    assert_eq!(count("rogue_5", "copper"), 103, "IS6 Tongbao, one per name");
    assert_eq!(count("rogue_5", "wrath"), 12);
    assert_eq!(
        count("rogue_4", "band"),
        17,
        "IS5 squads without the Mimic variants"
    );
    // Relic upgrade variants (`rogue_4_relic_legacy_1_a`) and tickets are never offered.
    assert!(
        entries
            .iter()
            .all(|e| facet(&e.facets, "item_type") != Some("relic")
                || !(e.id.ends_with("_a") || e.id.ends_with("_b") || e.id.ends_with("_c")))
    );
    assert!(
        entries
            .iter()
            .all(|e| facet(&e.facets, "item_type") != Some("recruit_ticket"))
    );
    let total: usize = by.values().map(|c| c.0).sum();
    assert!(total >= 1681, "{total} entries, expected at least 1681");
}

#[test]
fn default_skins_and_original_modules_are_not_offered() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let skins = catalogue(gd, &assets, EntityKind::Skin);
    assert!(skins.iter().all(|s| s.id.contains('@')));
    // The default art still resolves when a list already holds it.
    assert!(resolve(gd, &assets, EntityKind::Skin, "char_002_amiya#1").is_some());
    let no_brand = skins
        .iter()
        .filter(|s| !s.facets.contains_key("brand"))
        .count();
    println!(
        "skins {} offered of {} in the table, {no_brand} without a brand",
        skins.len(),
        gd.skins.char_skins.len()
    );

    let modules = catalogue(gd, &assets, EntityKind::Module);
    let originals = gd
        .modules
        .equip_dict
        .values()
        .filter(|m| m.type_icon == "original")
        .count();
    println!(
        "modules {} offered of {} in the table, {originals} ORIGINAL",
        modules.len(),
        gd.modules.equip_dict.len()
    );
    assert!(validate([(gd, &assets)], EntityKind::Module, "uniequip_001_amiya").is_err());
    let mut by_type: BTreeMap<&str, usize> = BTreeMap::new();
    for m in &modules {
        *by_type
            .entry(facet(&m.facets, "module_type").unwrap_or("?"))
            .or_default() += 1;
    }
    println!("  by type {by_type:?}");
}

#[test]
fn story_sprites_are_one_per_character_and_mostly_named() {
    if !story_sprites_present() {
        eprintln!(
            "no EN story sprite tree at {}, skipping",
            en_dir().display()
        );
        return;
    }
    let gd = common::load_game_data();
    let started = std::time::Instant::now();
    let assets = en_assets();
    println!("asset index built in {} ms", started.elapsed().as_millis());
    let sprites = catalogue(gd, &assets, EntityKind::StorySprite);
    let operators = sprites
        .iter()
        .filter(|s| facet(&s.facets, "source") == Some("operator"))
        .count();
    let named = sprites.iter().filter(|s| s.name != s.id).count();
    let with_face = sprites
        .iter()
        .filter(|s| s.facets.contains_key("face"))
        .count();
    let mut sprites_by_name: BTreeMap<&str, usize> = BTreeMap::new();
    for s in &sprites {
        *sprites_by_name.entry(s.name.as_str()).or_default() += 1;
    }
    let shared = sprites_by_name.values().filter(|&&n| n > 1).count();
    println!(
        "story sprites {}: {operators} operators, {named} named, {with_face} with a face position, {shared} names shared by more than one",
        sprites.len()
    );
    let kalts = resolve(gd, &assets, EntityKind::StorySprite, "char_003_kalts").expect("Kal'tsit");
    assert_eq!(kalts.name, "Kal'tsit");
    assert!(validate([(gd, &assets)], EntityKind::StorySprite, "char_003_kalts").is_ok());
    // Sprites are checked against the extract: a folder-shaped id it lacks is refused.
    assert!(validate([(gd, &assets)], EntityKind::StorySprite, "avg_npc_999999").is_err());
    assert_eq!(facet(&kalts.facets, "source"), Some("operator"));
    // The tile art is the head-and-shoulders thumbnail, not the whole plate.
    assert_eq!(
        kalts.icon.as_deref(),
        Some("/story-sprite-thumb/char_003_kalts")
    );
    // Face centres are fractions of the BODY plate. Dividing by the hub's
    // face-patch size put all of them on the bottom-right corner (1.0, 1.0).
    let corner = sprites
        .iter()
        .filter_map(|s| facet(&s.facets, "face"))
        .filter(|f| {
            f.split(',')
                .filter_map(|v| v.parse::<f32>().ok())
                .any(|v| v >= 0.999)
        })
        .count();
    println!("{corner} of {with_face} face positions on the plate's edge");
    assert!(with_face >= 700, "{with_face} with a face position");
    assert!(
        corner * 50 <= with_face,
        "{corner} of {with_face} on the edge"
    );
    assert!(
        named * 10 >= sprites.len() * 8,
        "{named} of {} named",
        sprites.len()
    );
}

#[test]
fn skills_are_one_per_operator_slot() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let skills = catalogue(gd, &assets, EntityKind::Skill);
    let mut by_slot: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_sp: BTreeMap<&str, usize> = BTreeMap::new();
    for s in &skills {
        *by_slot
            .entry(facet(&s.facets, "slot").unwrap_or("?"))
            .or_default() += 1;
        *by_sp
            .entry(facet(&s.facets, "sp_type").unwrap_or("passive"))
            .or_default() += 1;
    }
    println!(
        "skills {}: by slot {by_slot:?}, by sp {by_sp:?}",
        skills.len()
    );
    // A generic skill is one entry per operator that has it.
    let shared = skills.iter().filter(|s| s.id.contains(":skcom_")).count();
    assert!(shared > 0);
    assert_eq!(
        resolve(
            gd,
            &assets,
            EntityKind::Skill,
            "char_002_amiya:skchr_amiya_2"
        )
        .map(|s| s.facets.get("slot").cloned()),
        Some(Some(FacetValue::One("2".to_owned())))
    );
    // A real skill id under the wrong operator is not an entry.
    assert!(
        validate(
            [(gd, &assets)],
            EntityKind::Skill,
            "char_003_kalts:skchr_amiya_2"
        )
        .is_err()
    );
    assert!(validate([(gd, &assets)], EntityKind::Skill, "skchr_amiya_2").is_err());
}

#[test]
fn main_story_is_one_entry_per_episode_in_order() {
    let gd = common::load_game_data();
    let assets = en_assets();
    let episodes = catalogue(gd, &assets, EntityKind::MainStory);
    let numbers: Vec<Option<&str>> = episodes
        .iter()
        .map(|e| facet(&e.facets, "episode"))
        .collect();
    let no_icon = episodes.iter().filter(|e| e.icon.is_none()).count();
    let mut by_act: BTreeMap<&str, usize> = BTreeMap::new();
    for e in &episodes {
        *by_act
            .entry(facet(&e.facets, "act_name").unwrap_or("?"))
            .or_default() += 1;
    }
    println!(
        "main story {}: {no_icon} without icon, by act {by_act:?}",
        episodes.len()
    );
    // EN 2026-10-05: Episodes 00 to 16, `main_15` and `main_16` numbered off
    // their `act*mainss` zones.
    assert_eq!(episodes.len(), 17);
    let expected: Vec<String> = (0..17).map(|n| n.to_string()).collect();
    assert_eq!(
        numbers,
        expected
            .iter()
            .map(|n| Some(n.as_str()))
            .collect::<Vec<_>>(),
        "episode order"
    );
    assert_eq!(no_icon, 0);
    // Four acts, the `chapter_table` ones, every episode in one.
    assert_eq!(by_act.len(), 4, "{by_act:?}");
    assert!(!by_act.contains_key("?"));
    let ep15 = resolve(gd, &assets, EntityKind::MainStory, "main_15").expect("main_15");
    assert_eq!(facet(&ep15.facets, "act"), Some("3"));
    // A side story is not a main story episode.
    assert!(validate([(gd, &assets)], EntityKind::MainStory, "act17side").is_err());
}

/// Every kind's catalogue, one pretty-printed JSON file per kind.
/// Run it before and after a change to `tier_entity` and diff the two
/// directories: a refactor must leave every file byte-identical.
/// `CATALOGUE_DUMP_DIR=/path cargo test --test tier_entity_real_data_test dump_every_catalogue -- --ignored`
#[test]
#[ignore = "writes every kind's catalogue as JSON to CATALOGUE_DUMP_DIR"]
fn dump_every_catalogue() {
    let dir =
        std::path::PathBuf::from(std::env::var("CATALOGUE_DUMP_DIR").expect("CATALOGUE_DUMP_DIR"));
    std::fs::create_dir_all(&dir).expect("dir");
    let gd = common::load_game_data();
    let assets = en_assets();
    for kind in EntityKind::ALL.iter().copied() {
        let cat = catalogue(gd, &assets, kind);
        let json = serde_json::to_string_pretty(&cat).expect("json");
        std::fs::write(dir.join(format!("{}.json", kind.as_str())), json).expect("write");
    }
}
