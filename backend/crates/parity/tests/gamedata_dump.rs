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
    // extended field set
    put(&out, "activity_skin_refs", &m.activity_skin_refs);
    put(
        &out,
        "autochess_bonds_debug",
        &format!("{:?}", m.autochess_bonds),
    );
    put(&out, "building_debug", &format!("{:?}", m.building));
    put(
        &out,
        "campaign_rotations_debug",
        &format!("{:?}", m.campaign_rotations),
    );
    put(&out, "consts", &m.consts);
    put(&out, "enemy_audio", &m.enemy_audio);
    put(&out, "enemy_stage_index", &m.enemy_stage_index);
    put(&out, "factions_debug", &format!("{:?}", m.factions));
    put(
        &out,
        "home_backgrounds_debug",
        &format!("{:?}", m.home_backgrounds),
    );
    put(
        &out,
        "mission_archives_debug",
        &format!("{:?}", m.mission_archives),
    );
    put(&out, "missions_debug", &format!("{:?}", m.missions));
    put(&out, "mode_levels", &m.mode_levels);
    put(&out, "music_debug", &format!("{:?}", m.music));
    put(&out, "profession_names", &m.profession_names);
    put(
        &out,
        "recommend_tags_debug",
        &format!("{:?}", m.recommend_tags),
    );
    put(&out, "skin_listings", &m.skin_listings);
    put(&out, "skin_windows", &m.skin_windows);
    put(&out, "squad_debug", &format!("{:?}", m.squad));
    put(
        &out,
        "stage_evidence_debug",
        &format!("{:?}", m.stage_evidence),
    );
    put(&out, "stage_index", &m.stage_index);
    put(
        &out,
        "storyline_story_sets_debug",
        &format!("{:?}", m.storyline_story_sets),
    );
    put(
        &out,
        "story_archives_debug",
        &format!("{:?}", m.story_archives),
    );
    put(&out, "enemies", &m.enemies);
    put(&out, "event_shops", &m.event_shops);
    put(
        &out,
        "sandbox_universe_debug",
        &format!("{:?}", m.sandbox_universe),
    );
    println!("  dumped to {}", out.display());
}
