//! Differential dump of the compiled game model: each top-level field of
//! `GameModel` that main's `GameData` also has, serialized to JSON, one file per
//! field, so main's `init_game_data` output and `gamedata::compile` output compare
//! field by field.
//!
//! `GAMEDATA_DUMP=<out dir> GAMEDATA_ROOT=<dir holding gamedata/excel> [GAMEDATA_SERVER=cn]
//! cargo test -p myrtle-parity --test gamedata_dump -- --nocapture`. Skips without
//! `GAMEDATA_DUMP`. Prints the loader's warnings (tables that failed to parse).

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata_raw::{DirSource, load};
use myrtle_types::Server;

fn put<T: serde::Serialize>(dir: &std::path::Path, name: &str, v: &T) {
    match serde_json::to_vec(v) {
        Ok(bytes) => std::fs::write(dir.join(format!("{name}.json")), bytes).unwrap(),
        Err(e) => println!("  {name}: not serializable: {e}"),
    }
}

#[test]
fn gamedata_dump() {
    let Ok(out) = std::env::var("GAMEDATA_DUMP") else {
        println!("GAMEDATA_DUMP unset, skipping");
        return;
    };
    let root = PathBuf::from(std::env::var("GAMEDATA_ROOT").unwrap());
    let server = match std::env::var("GAMEDATA_SERVER").as_deref() {
        Ok("cn") => Server::CN,
        _ => Server::EN,
    };
    let out = PathBuf::from(out);
    std::fs::create_dir_all(&out).unwrap();
    let raw = load(&DirSource { root }, server).unwrap();
    for w in &raw.warnings {
        println!("  load warning: {w}");
    }
    let compiled = compile(raw, &CompileOptions::new(server)).unwrap();
    for w in &compiled.report.warnings {
        println!("  compile warning: {w}");
    }
    let m = &compiled.model;
    put(&out, "operators", &m.operators);
    put(&out, "skills", &m.skills);
    put(&out, "materials", &m.materials);
    put(&out, "modules", &m.modules);
    put(&out, "skins", &m.skins);
    put(&out, "handbook", &m.handbook);
    put(&out, "ranges", &m.ranges);
    put(&out, "favor", &m.favor);
    put(&out, "voices", &m.voices);
    put(&out, "gacha", &m.gacha);
    put(&out, "zones", &m.zones);
    put(&out, "stages", &m.stages);
    put(&out, "activities", &m.activities);
    put(&out, "activity_op_stages", &m.activity_op_stages);
    put(&out, "activity_farm_stages", &m.activity_farm_stages);
    put(&out, "activity_mission_tokens", &m.activity_mission_tokens);
    put(&out, "retro_acts", &m.retro_acts);
    put(&out, "story_reviews", &m.story_reviews);
    put(&out, "chapters", &m.chapters);
    put(&out, "zone_chapters", &m.zone_chapters);
    put(&out, "zone_open_times", &m.zone_open_times);
    put(&out, "storylines_debug", &format!("{:?}", m.storylines));
    let mut themes: Vec<String> = m
        .roguelike
        .themes
        .iter()
        .map(|(k, t)| format!("{k}: {t:?}"))
        .collect();
    themes.sort();
    put(&out, "roguelike_themes_debug", &themes);
    put(&out, "stage_universe", &m.stage_universe);
    put(
        &out,
        "sandbox_universe_debug",
        &format!("{:?}", m.sandbox_universe),
    );
    println!("  dumped to {}", out.display());
}
