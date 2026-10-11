//! The In-Game Music index against the real EN tree, with the other servers'
//! trees as the clip and level fallbacks production gives it, then the same for
//! CN when its tree is on disk. Prints the census the gallery's numbers come
//! from. Skips without the EN audio tree, which the CI `game-data` artifact does
//! not carry.
//!
//! Run with `cargo test --test music_real_data_test -- --nocapture`.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use backend::app::services::music::{
    MusicIndex, MusicKind, MusicPlaceKind, MusicRole, MusicSources, build_music_index,
};
use backend::core::gamedata::types::GameData;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output")
}

/// Build one server's index the way production does: the asked server's tree
/// first, then the others as clip and level fallbacks.
fn build(gd: &GameData, server: &str) -> MusicIndex {
    let own = root().join(server);
    let mut level_dirs = vec![own.join("gamedata/levels")];
    let mut clip_trees = vec![(None, own.clone())];
    for other in ["en", "jp", "kr", "cn"] {
        let dir = root().join(other);
        if other != server && dir.is_dir() {
            level_dirs.push(dir.join("gamedata/levels"));
            clip_trees.push((Some(other.to_owned()), dir));
        }
    }
    let started = std::time::Instant::now();
    let index = build_music_index(
        gd,
        &MusicSources {
            assets_dir: own,
            level_dirs,
            clip_trees,
        },
    );
    println!("[{server}] build {} ms", started.elapsed().as_millis());
    index
}

/// A value off a `[{key, value}]` excel map.
fn keyed<'a>(list: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    list.as_array()?
        .iter()
        .find(|e| e["key"] == key)
        .map(|e| &e["value"])
}

fn excel(server: &str, table: &str) -> serde_json::Value {
    let path = root().join(format!("{server}/gamedata/excel/{table}.json"));
    serde_json::from_slice(&std::fs::read(&path).expect("table on disk")).expect("table parses")
}

/// Print the census and answer the tracks with no traced place.
fn census(gd: &GameData, server: &str, index: &MusicIndex) -> Vec<String> {
    let t = &index.totals;
    println!(
        "[{server}] music: {} tracks, {} titled, {} with audio ({} from another server), {} unavailable",
        t.tracks, t.titled, t.with_audio, t.from_other_server, t.unavailable,
    );
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut places: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut no_place = Vec::new();
    let mut no_origin = Vec::new();
    let mut nameless_chapters = 0;
    let mut with_intro = 0;
    for track in &index.tracks {
        for k in &track.kinds {
            *kinds.entry(format!("{k:?}")).or_default() += 1;
        }
        if track.places.is_empty() {
            no_place.push(track.id.clone());
        }
        if track.origin.is_none() {
            no_origin.push(track.id.as_str());
        }
        if track.intro_url.is_some() {
            with_intro += 1;
        }
        let mut seen = std::collections::HashSet::new();
        for p in &track.places {
            let e = places.entry(format!("{:?}", p.kind)).or_default();
            e.1 += p.items.len();
            if seen.insert(p.kind) {
                e.0 += 1;
            }
            if p.kind == MusicPlaceKind::Chapter && p.name.is_none() {
                nameless_chapters += 1;
            }
        }
    }
    // The Reclamation Algorithm run `sandbox_1` against the separate 2023
    // event `act1sandbox`, both names read off the tables.
    let topic = excel(server, "sandbox_perm_table");
    let topic = keyed(&topic["BasicInfo"], "sandbox_1")
        .and_then(|v| v["TopicName"].as_str())
        .map(str::to_owned);
    let act1sandbox = gd.activities.get("act1sandbox").map(|a| a.name.clone());
    let sandbox_1: Vec<_> = index
        .tracks
        .iter()
        .filter(|t| {
            t.loop_url
                .as_deref()
                .is_some_and(|u| u.contains("/sandbox_1/"))
                || t.banks
                    .iter()
                    .any(|b| b.starts_with("sys.") && b.ends_with(".sandbox_1"))
        })
        .collect();
    let as_event = sandbox_1
        .iter()
        .filter(|t| t.origin.is_some() && t.origin == act1sandbox)
        .count();
    let as_topic = sandbox_1
        .iter()
        .filter(|t| t.origin.is_some() && t.origin == topic)
        .count();
    println!("[{server}] kinds (tracks): {kinds:?}");
    println!("[{server}] places (tracks, items): {places:?}");
    println!(
        "[{server}] {with_intro} tracks with an intro clip; {} with an origin, {} with none: {no_origin:?}",
        index.tracks.len() - no_origin.len(),
        no_origin.len(),
    );
    println!("[{server}] {nameless_chapters} nameless Chapter places");
    let other: Vec<_> = sandbox_1
        .iter()
        .filter(|t| t.origin != topic)
        .map(|t| (t.id.as_str(), t.origin.as_deref()))
        .collect();
    println!(
        "[{server}] sandbox_1 tracks: {}, labelled {act1sandbox:?}: {as_event}, labelled {topic:?}: {as_topic}, other: {other:?}",
        sandbox_1.len()
    );
    println!(
        "[{server}] {} with no traced place: {no_place:?}",
        no_place.len()
    );
    let unavailable: Vec<&str> = index
        .tracks
        .iter()
        .filter(|t| t.loop_url.is_none())
        .map(|t| t.id.as_str())
        .collect();
    println!("[{server}] unavailable: {unavailable:?}");

    assert_eq!(t.tracks as usize, index.tracks.len());
    assert_eq!(t.with_audio + t.unavailable, t.tracks);
    assert!(index.tracks.iter().all(|t| !t.kinds.is_empty()));
    assert!(index.tracks.iter().all(|t| t.id != "empty"));
    assert_eq!(
        nameless_chapters, 0,
        "[{server}] every chapter place is named"
    );
    assert!(topic.is_some(), "[{server}] sandbox_1 has a topic name");
    assert_eq!(
        as_topic,
        sandbox_1.len(),
        "[{server}] every sandbox_1 track carries the run's topic name"
    );
    assert!(
        topic == act1sandbox || as_event == 0,
        "[{server}] no sandbox_1 track carries the act1sandbox event's name"
    );

    // Each main story theme is a named chapter, keyed through the zone's own
    // `GameMusicId`, so the side-story zones after Episode 14 are named too.
    let zones = excel(server, "zone_table");
    let zone_names = &zones["Zones"];
    for info in zones["MainlineAdditionInfo"]
        .as_array()
        .expect("addition info")
    {
        let music = info["value"]["GameMusicId"].as_str().unwrap_or_default();
        let zone = info["value"]["ZoneId"].as_str().unwrap_or_default();
        let Some(bank) = gd.music.musics().find(|m| m.id == music).map(|m| &m.bank) else {
            continue;
        };
        let names = keyed(zone_names, zone).expect("zone row");
        let want: Vec<&str> = ["ZoneNameSecond", "ZoneNameFirst"]
            .iter()
            .filter_map(|f| names[*f].as_str())
            .collect();
        let track = index
            .tracks
            .iter()
            .find(|t| t.banks.contains(bank))
            .unwrap_or_else(|| panic!("[{server}] {music} has a track"));
        assert!(
            track
                .places
                .iter()
                .any(|p| p.kind == MusicPlaceKind::Chapter
                    && p.name.as_deref().is_some_and(|n| want.contains(&n))),
            "[{server}] {music} plays at chapter {zone} ({want:?}): {:?}",
            track.places
        );
    }

    // Every Integrated Strategies floor signal the topic table names plays as
    // a map, the boss floor's included, with the floor's name as an item.
    let rogue = excel(server, "roguelike_topic_table");
    let mut floors = 0;
    for detail in rogue["Details"].as_array().expect("details") {
        let theme = detail["key"].as_str().unwrap_or_default();
        for zone in detail["value"]["Zones"].as_array().into_iter().flatten() {
            let (Some(sig), Some(name)) = (
                zone["value"]["BgmSignal"].as_str(),
                zone["value"]["Name"].as_str(),
            ) else {
                continue;
            };
            let bank = format!("sys.{sig}.{theme}");
            let Some(track) = index.tracks.iter().find(|t| t.banks.contains(&bank)) else {
                continue;
            };
            floors += 1;
            assert!(
                track.kinds.contains(&MusicKind::Map),
                "[{server}] {bank} is map music"
            );
            assert!(
                track.places.iter().any(|p| p.kind == MusicPlaceKind::Map
                    && p.role == Some(MusicRole::Map)
                    && p.items.iter().any(|i| i.name.as_deref() == Some(name))),
                "[{server}] {bank} names floor {name}: {:?}",
                track.places
            );
        }
    }
    assert!(
        floors > 0,
        "[{server}] the topic table's floors reach tracks"
    );

    // Furniture that plays music carries the furniture's own name.
    let building = excel(server, "building_data");
    let mut furniture = 0;
    for f in building["CustomData"]["Furnitures"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let (Some(music), Some(name)) =
            (f["value"]["MusicId"].as_str(), f["value"]["Name"].as_str())
        else {
            continue;
        };
        let Some(bank) = gd.music.musics().find(|m| m.id == music).map(|m| &m.bank) else {
            continue;
        };
        let track = index
            .tracks
            .iter()
            .find(|t| t.banks.contains(bank))
            .unwrap_or_else(|| panic!("[{server}] {music} has a track"));
        furniture += 1;
        assert!(
            track
                .places
                .iter()
                .any(|p| p.role == Some(MusicRole::Furniture)
                    && p.name.as_deref() == Some(name.trim())),
            "[{server}] {music} plays at furniture {name}: {:?}",
            track.places
        );
    }
    println!("[{server}] {floors} floor signals and {furniture} furniture pieces checked");
    no_place
}

#[test]
fn every_named_track_is_indexed_and_counted() {
    let en = root().join("en");
    if !en.join("audio/audio/sound_beta_2/music").is_dir() {
        eprintln!(
            "music_real_data_test: no EN music tree at {}, skipping",
            en.display()
        );
        return;
    }
    let gd = common::load_game_data();
    let index = build(gd, "en");
    let json = serde_json::to_vec(&index).expect("serializes");
    println!("[en] payload {} bytes", json.len());
    // The route answers the brotli form stored once per build; it must decode
    // to exactly the JSON and stay a fraction of it.
    let started = std::time::Instant::now();
    let shared = backend::app::cache::SharedJson::new(json.clone());
    println!(
        "[en] brotli {} bytes, encoded once in {} ms",
        shared.br.len(),
        started.elapsed().as_millis()
    );
    let mut plain = Vec::new();
    std::io::Read::read_to_end(
        &mut brotli::Decompressor::new(&shared.br[..], 4096),
        &mut plain,
    )
    .expect("brotli decodes");
    assert_eq!(plain, json, "the brotli form decodes to the JSON");
    assert!(
        shared.br.len() * 5 < json.len(),
        "brotli {} of {} bytes",
        shared.br.len(),
        json.len()
    );
    assert_ne!(shared.etag, shared.br_etag);
    let no_place = census(gd, "en", &index);
    let other: Vec<String> = index
        .tracks
        .iter()
        .filter_map(|t| t.server.as_ref().map(|s| format!("{}@{s}", t.id)))
        .collect();
    println!("[en] from another server: {other:?}");

    // The anchors the derivation was checked against by hand.
    let by_id = |id: &str| index.tracks.iter().find(|t| t.id == id);
    let indust = by_id("m_bat_indust").expect("m_bat_indust");
    assert!(indust.kinds.contains(&MusicKind::Battle));
    assert!(
        indust
            .places
            .iter()
            .any(|p| p.kind == MusicPlaceKind::Stage)
    );
    let title = by_id("m_sys_title").expect("m_sys_title");
    assert!(
        title
            .loop_url
            .as_deref()
            .is_some_and(|u| u.contains("/anon/"))
    );
    let void = by_id("m_sys_void").expect("m_sys_void");
    assert_eq!(void.title.as_deref(), Some("Lifeflow"));
    // 35 before the topic-name fix: the `act1sandbox` map, mapboss and
    // factory banks now resolve to their event.
    assert_eq!(no_place.len(), 31, "[en] tracks with no traced place");
    let main15 = index
        .tracks
        .iter()
        .find(|t| {
            t.places.iter().any(|p| {
                p.kind == MusicPlaceKind::Chapter
                    && p.name.as_deref() == Some("Dissociative Recombination")
            })
        })
        .expect("music_3in1bg_main15 plays at Dissociative Recombination");
    assert!(main15.banks.iter().any(|b| {
        gd.music
            .musics()
            .any(|m| m.id == "music_3in1bg_main15" && &m.bank == b)
    }));

    // One origin per step of the rule, each name read off the tables.
    let origin = |id: &str, want: Option<String>| {
        assert!(
            want.is_some(),
            "[en] {id}: the expected name is in the tables"
        );
        assert_eq!(
            by_id(id).and_then(|t| t.origin.clone()),
            want,
            "[en] {id} origin"
        );
    };
    let act = |id: &str| gd.activities.get(id).map(|a| a.name.trim().to_owned());
    let zone = |id: &str| {
        gd.zones
            .get(id)
            .and_then(|z| z.zone_name_second.as_deref())
            .map(|n| n.trim().to_owned())
    };
    // CHAPTER: the Episode 07 theme, though its clip sits in `act9d0d0`.
    origin("m_avg_batmeeting", zone("main_7"));
    // CLAIM: a Story Collection set and an archive name a borrowed clip's
    // maker over its folder.
    origin("m_avg_drift", act("act12mini"));
    origin(
        "m_sys_act17mini",
        gd.roguelike
            .themes
            .get("rogue_4")
            .map(|t| t.theme_name.trim().to_owned()),
    );
    // MENU, a rerun read as its original: `m_sys_alive` plays on the menus of
    // `act3d0` and its rerun `act11d7`.
    origin("m_sys_alive", act("act3d0"));
    // STAGE, the same: its earliest stages are the Grani rerun's.
    origin("m_bat_game02", act("1stact"));
    // STORY, the group with the most `[PlayMusic]` calls: 8 in "The Masses'
    // Travels", 1 in the earlier "Return To Mist".
    origin("m_avg_calm", act("act42side"));
}

#[test]
fn the_cn_index_names_every_chapter_and_run() {
    if !root().join("en/audio/audio/sound_beta_2/music").is_dir() {
        return;
    }
    let Some(gd) = common::load_server_with_en_art("cn") else {
        eprintln!("music_real_data_test: no CN tree, skipping");
        return;
    };
    let index = build(&gd, "cn");
    let no_place = census(&gd, "cn", &index);
    // 40 before the topic-name fix.
    assert_eq!(no_place.len(), 33, "[cn] tracks with no traced place");
}
