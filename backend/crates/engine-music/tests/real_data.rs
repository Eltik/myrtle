#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use myrtle_engine_music::{
    CLIP_ROOTS, ClipTree, MusicIndex, MusicPlaceKind, MusicSources, TABLES, build_music_index,
};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_testkit::real_data::excel_dir;
use myrtle_types::Server;

fn root() -> PathBuf {
    excel_dir().join("../..")
}

fn en() -> Option<(GameModel, MusicSources)> {
    let root = root();
    if !root.join("gamedata/excel").is_dir() || !root.join(CLIP_ROOTS[0]).is_dir() {
        return None;
    }
    let source = DirSource { root };
    let raw = load(&source, Server::EN).unwrap();
    let gd = compile(raw, &CompileOptions::new(Server::EN))
        .unwrap()
        .model;
    let clips = vec![ClipTree {
        server: None,
        paths: common::clip_paths(&source),
    }];
    let (src, warnings) = common::music_sources(&gd, &source, &[&source], clips);
    assert!(warnings.is_empty(), "{warnings:?}");
    Some((gd, src))
}

fn census(index: &MusicIndex) {
    let t = &index.totals;
    println!(
        "[en] {} tracks, {} titled, {} with audio, {} unavailable",
        t.tracks, t.titled, t.with_audio, t.unavailable
    );
    let mut places: BTreeMap<String, usize> = BTreeMap::new();
    for track in &index.tracks {
        for p in &track.places {
            *places.entry(format!("{:?}", p.kind)).or_default() += 1;
        }
    }
    println!("[en] places by kind: {places:?}");
}

#[test]
fn every_table_the_build_reads_is_on_disk() {
    let root = root();
    if !root.join("gamedata/excel").is_dir() {
        return;
    }
    for name in TABLES {
        assert!(
            root.join(format!("gamedata/excel/{name}.json")).is_file(),
            "{name}"
        );
    }
}

#[test]
fn the_en_index_is_whole_and_stable() {
    let Some((gd, src)) = en() else {
        return;
    };
    let index = build_music_index(&gd, &src);
    census(&index);
    let t = &index.totals;
    assert_eq!(t.tracks as usize, index.tracks.len());
    assert_eq!(t.with_audio + t.unavailable, t.tracks);
    assert_eq!(t.from_other_server, 0);
    assert!(t.tracks > 500, "{} tracks", t.tracks);
    assert!(index.tracks.windows(2).all(|w| w[0].id < w[1].id));
    assert!(index.tracks.iter().all(|t| t.id != "empty"));
    for kind in [
        MusicPlaceKind::Stage,
        MusicPlaceKind::Story,
        MusicPlaceKind::Season,
        MusicPlaceKind::Event,
        MusicPlaceKind::Chapter,
        MusicPlaceKind::Lobby,
        MusicPlaceKind::Map,
    ] {
        assert!(
            index
                .tracks
                .iter()
                .any(|t| t.places.iter().any(|p| p.kind == kind)),
            "no {kind:?} place"
        );
    }
    let with_origin = index.tracks.iter().filter(|t| t.origin.is_some()).count();
    assert!(with_origin * 10 > index.tracks.len() * 9, "{with_origin}");
    assert_eq!(
        serde_json::to_vec(&index).unwrap(),
        serde_json::to_vec(&build_music_index(&gd, &src)).unwrap()
    );
}
