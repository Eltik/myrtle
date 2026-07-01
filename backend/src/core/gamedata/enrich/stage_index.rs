//! Build the flat Stage List catalogue ([`StageIndex`]).
//!
//! Combines two sources, both classified through the single-source-of-truth
//! [`StageClassifier`]:
//!  1. Every `stage_table` stage (story, events, SSS, Annihilation, supplies).
//!  2. The procedural-mode level files whose stages are *not* in `stage_table`
//!     (Integrated Strategies, Reclamation Algorithm, Contingency Contract,
//!     Paradox Simulation), walked off disk under `gamedata/levels/`.
//!
//! Map-preview and banner art paths are resolved against disk once at load
//! (indexed by filename), so the frontend gets a single verified URL per stage
//! instead of guessing folder variants and 404-ing on the client.
//!
//! Deduping of code / difficulty variants is left to the frontend, which needs
//! the difficulty to pick the canonical card.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

use super::enemy_stages::{collect_level_files, relative_level_id};
use super::stage_class::{StageClassifier, StageInfo, read_json};
use crate::core::gamedata::types::activity::ActivityBasicInfo;
use crate::core::gamedata::types::stage::{Stage, StageDifficulty};
use crate::core::gamedata::types::stage_index::{StageIndex, StageIndexEntry};
use crate::core::gamedata::types::zone::Zone;

const fn difficulty_str(d: &StageDifficulty) -> &'static str {
    match d {
        StageDifficulty::Normal => "NORMAL",
        StageDifficulty::FourStar => "FOUR_STAR",
        StageDifficulty::SixStar => "SIX_STAR",
        StageDifficulty::Unknown => "NORMAL",
    }
}

/// Sort key within a group. Story zones carry their real episode number in
/// `zoneNameTitleCurrent` ("00".."16"), which orders MAINLINE and
/// `MAINLINE_ACTIVITY` together; everything else falls back to the zone index.
fn zone_order(
    group: &str,
    zone_id: &str,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> i64 {
    let zone = zones.get(zone_id);
    if group == "story"
        && let Some(z) = zone
        && let Some(n) = z
            .zone_name_title_current
            .as_deref()
            .and_then(|s| s.trim().parse::<i64>().ok())
    {
        return n;
    }
    // Events (zone_id == activity id) sort chronologically by release date.
    if let Some(a) = activities.get(zone_id)
        && a.start_time > 0
    {
        return a.start_time;
    }
    zone.map_or(0, |z| i64::from(z.zone_index))
}

/// TODO: Need to fix asset unpacker, may
/// be able to remove after.
///
/// Whether the level file backing a stage actually exists on disk. Some
/// `stage_table` stages (retired "Story mode" `easy_*` chapters, brand-new
/// events whose levels aren't extracted yet) have a `level_id` but no file, so
/// the map viewer 404s. The viewer resolves through `stage_table`, so those are
/// not viewable — the frontend uses this to prefer a playable duplicate and to
/// avoid linking to a dead detail page.
fn level_exists(levels_dir: &Path, level_id: Option<&str>) -> bool {
    let Some(level_id) = level_id else {
        return false;
    };
    let rel = level_id.to_lowercase().replace('\\', "/");
    levels_dir.join(format!("{rel}.json")).is_file()
}

/// Filename-keyed indexes of the on-disk art, built once at load. A stage's
/// preview file is named after the stage (`main_09-01.png`, `lt_07_01.png`) or,
/// for procedural modes, its node (`mem_gdglow_1.png`, `crisis_v2_01-02.png`),
/// so a single stem lookup resolves the full path regardless of the folder's
/// family/variant suffix.
struct ArtIndex {
    /// preview filename stem -> asset-relative path.
    preview: HashMap<String, String>,
    /// banner filename stem -> asset-relative path.
    banner: HashMap<String, String>,
    /// IS: lowercased level id -> node id (`obt/roguelike/ro1/level_rogue1_4-5`
    /// -> `ro1_n_4_5`), so a node resolves its `stage_mappreview_h2_ro*` preview.
    is_node_ids: HashMap<String, String>,
    /// IS: season id (`rogue_1`) -> active KV banner stem (`pic_rogue_1_KV2`).
    is_kv: HashMap<String, String>,
}

/// Value of a `[{key,value}]` FlatBuffer-map entry's field, as a &str.
fn fb_str<'a>(entry: &'a Value, field: &str) -> Option<&'a str> {
    entry.get(field).and_then(Value::as_str)
}

/// Parse `roguelike_topic_table` for the IS level->node-id map and each season's
/// active KV banner stem (`AutoSetKV`).
fn build_is_topic_maps(data_dir: &Path) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut node_ids = HashMap::new();
    let mut kv = HashMap::new();
    let Some(table) = read_json(data_dir, "roguelike_topic_table") else {
        return (node_ids, kv);
    };
    let Some(details) = table.get("Details").and_then(Value::as_array) else {
        return (node_ids, kv);
    };
    for entry in details {
        let Some(season) = entry.get("key").and_then(Value::as_str) else {
            continue;
        };
        let value = entry.get("value").unwrap_or(entry);
        if let Some(auto) = fb_str(value, "AutoSetKV").or_else(|| {
            value
                .get("DetailConst")
                .and_then(|c| fb_str(c, "AutoSetKV"))
        }) {
            kv.insert(season.to_owned(), auto.to_owned());
        }
        if let Some(stages) = value.get("Stages").and_then(Value::as_array) {
            for st in stages {
                let sv = st.get("value").unwrap_or(st);
                if let (Some(id), Some(level_id)) = (fb_str(sv, "Id"), fb_str(sv, "LevelId")) {
                    node_ids.insert(level_id.to_lowercase(), id.to_owned());
                }
            }
        }
    }
    (node_ids, kv)
}

/// Index every `*.png` in each immediate subfolder of `parent` whose folder
/// name passes `folder_ok`, keyed by file stem. First writer wins.
fn index_pngs(
    parent: &Path,
    rel_prefix: &str,
    folder_ok: impl Fn(&str) -> bool,
    out: &mut HashMap<String, String>,
) {
    let Ok(dirs) = std::fs::read_dir(parent) else {
        return;
    };
    for dir in dirs.flatten() {
        let dpath = dir.path();
        if !dpath.is_dir() {
            continue;
        }
        let folder = dpath.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if !folder_ok(folder) {
            continue;
        }
        let Ok(files) = std::fs::read_dir(&dpath) else {
            continue;
        };
        for file in files.flatten() {
            let fpath = file.path();
            if fpath.extension().and_then(|s| s.to_str()) != Some("png") {
                continue;
            }
            let Some(stem) = fpath.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            out.entry(stem.to_owned())
                .or_insert_with(|| format!("{rel_prefix}/{folder}/{stem}.png"));
        }
    }
}

fn build_art_index(assets_dir: &Path, data_dir: &Path) -> ArtIndex {
    let mut preview = HashMap::new();
    // Stage map previews: textures/arts/ui/stage_mappreview_h2_*  (includes the
    // Integrated Strategies per-node folders `..._ro<N>_<type>_0`, keyed by node id).
    index_pngs(
        &assets_dir.join("textures/arts/ui"),
        "textures/arts/ui",
        |f| f.contains("mappreview"),
        &mut preview,
    );
    // Reclamation Algorithm previews live in the sprite pack instead.
    index_pngs(
        &assets_dir.join("textures/spritepack"),
        "textures/spritepack",
        |f| f.contains("mappreview"),
        &mut preview,
    );

    let mut banner = HashMap::new();
    // Zone/event key-art (first writer wins), keyed by zone/activity id:
    //  - ui_home_act_banner_zone_*  : main-story landscape (main_9, act2mainss_zone1)
    //  - ui_zone_home_theme_act*    : clean event key-art with logo (`<activityId>.png`)
    index_pngs(
        &assets_dir.join("textures/spritepack"),
        "textures/spritepack",
        |f| f.contains("banner_zone") || f.starts_with("ui_zone_home_theme_act"),
        &mut banner,
    );
    // Integrated Strategies season KV banners: textures/avg/imgs/*/pic_rogue_N_KVk.png
    // (folders vary per season); index only the KV files by stem.
    index_named_pngs(
        &assets_dir.join("textures/avg/imgs"),
        "textures/avg/imgs",
        |stem| stem.starts_with("pic_rogue_") && stem.contains("_KV"),
        &mut banner,
    );

    let (is_node_ids, is_kv) = build_is_topic_maps(data_dir);
    ArtIndex {
        preview,
        banner,
        is_node_ids,
        is_kv,
    }
}

/// Like [`index_pngs`] but scans every subfolder and filters by file stem
/// (folder names are inconsistent), indexing only stems that pass `stem_ok`.
fn index_named_pngs(
    parent: &Path,
    rel_prefix: &str,
    stem_ok: impl Fn(&str) -> bool,
    out: &mut HashMap<String, String>,
) {
    let Ok(dirs) = std::fs::read_dir(parent) else {
        return;
    };
    for dir in dirs.flatten() {
        let dpath = dir.path();
        if !dpath.is_dir() {
            continue;
        }
        let folder = dpath.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let Ok(files) = std::fs::read_dir(&dpath) else {
            continue;
        };
        for file in files.flatten() {
            let fpath = file.path();
            if fpath.extension().and_then(|s| s.to_str()) != Some("png") {
                continue;
            }
            let Some(stem) = fpath.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if stem_ok(stem) {
                out.entry(stem.to_owned())
                    .or_insert_with(|| format!("{rel_prefix}/{folder}/{stem}.png"));
            }
        }
    }
}

/// Candidate preview filename stems for a stage, most-specific first.
fn preview_keys(stage_id: &str, level_id: Option<&str>, code: &str) -> Vec<String> {
    let mut keys = Vec::new();
    // Challenge-Mode ids (`main_11-01#f#`) reuse their base stage's preview.
    let base = stage_id.split('#').next().unwrap_or(stage_id);
    keys.push(stage_id.to_owned());
    if base != stage_id {
        keys.push(base.to_owned());
    }
    // Challenge / EX variants (SSS `lt_08_02_ex`, event `_ex`) reuse the base
    // stage's map preview.
    if let Some(b) = base.strip_suffix("_ex") {
        keys.push(b.to_owned());
    }
    // Procedural-mode nodes are named after their level file, not stage id.
    if let Some(lid) = level_id {
        let file = lid.rsplit(['/', '\\']).next().unwrap_or(lid).to_lowercase();
        let node = file.strip_prefix("level_").unwrap_or(&file);
        if let Some(rest) = node.strip_prefix("memory_") {
            keys.push(format!("mem_{rest}")); // level_memory_gdglow_1 -> mem_gdglow_1
        }
        if let Some(rest) = node.strip_prefix("sandbox") {
            keys.push(format!("sandbox_{}", rest.trim_start_matches('_'))); // sandbox1_04 -> sandbox_1_04
        }
        keys.push(node.to_owned()); // crisis_v2_01-02, etc.
    }
    if !code.is_empty() {
        keys.push(format!("crisis_v2_{code}"));
    }
    keys
}

fn resolve_preview(
    art: &ArtIndex,
    stage_id: &str,
    level_id: Option<&str>,
    code: &str,
) -> Option<String> {
    preview_keys(stage_id, level_id, code)
        .into_iter()
        .find_map(|k| art.preview.get(&k).cloned())
}

/// Resolve a zone/event banner. Main-story episodes and mainline-activity zones
/// are keyed directly (`main_9`, `act2mainss_zone1`); other events have no
/// reliably-keyed banner, so the UI falls back to a stage preview.
fn resolve_banner(
    art: &ArtIndex,
    zone_id: &str,
    group: &str,
    episode: Option<i64>,
) -> Option<String> {
    let mut keys = vec![zone_id.to_owned()];
    if group == "story"
        && let Some(n) = episode
    {
        keys.push(format!("main_{n}"));
    }
    // Integrated Strategies season KV banner: the topic table's AutoSetKV, then
    // fall back to other KVs when that exact one isn't extracted.
    if group == "is" {
        if let Some(kv) = art.is_kv.get(zone_id) {
            keys.push(kv.clone());
        }
        keys.push(format!("pic_{zone_id}_KV2"));
        keys.push(format!("pic_{zone_id}_KV1"));
    }
    // Events (zone_id == activity id) match ui_zone_home_theme via `zone_id`.
    keys.into_iter().find_map(|k| art.banner.get(&k).cloned())
}

fn entry_from_stage(
    stage: &Stage,
    info: &StageInfo,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
    can_view: bool,
    art: &ArtIndex,
) -> StageIndexEntry {
    let episode = zones
        .get(&info.zone_id)
        .and_then(|z| z.zone_name_title_current.as_deref())
        .and_then(|s| s.trim().parse::<i64>().ok());
    StageIndexEntry {
        preview: resolve_preview(art, &stage.stage_id, stage.level_id.as_deref(), &info.code),
        banner: resolve_banner(art, &info.zone_id, info.group, episode),
        stage_id: stage.stage_id.clone(),
        level_id: stage.level_id.clone(),
        code: info.code.clone(),
        name: info.stage_name.clone(),
        zone_order: zone_order(info.group, &info.zone_id, zones, activities),
        zone_id: info.zone_id.clone(),
        zone_name: info.zone_name.clone(),
        group: info.group.to_owned(),
        category: info.category.to_owned(),
        ap_cost: stage.ap_cost,
        boss: stage.boss_mark,
        is_hard: info.is_hard,
        difficulty: difficulty_str(&stage.difficulty).to_owned(),
        can_view,
    }
}

/// A procedural-mode node with no `stage_table` entry (IS / RA / CC / Paradox).
/// The URL-safe id for a procedural-mode node: its level filename (last path
/// segment), e.g. `obt/roguelike/ro1/level_rogue1_6-1` -> `level_rogue1_6-1`.
/// Unique across modes (the season prefix is in the name) and free of slashes,
/// so it works as a `/stages/{id}` route param.
fn mode_stage_id(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// A procedural-mode node with no `stage_table` entry (IS / RA / CC / Paradox).
/// These are viewable: `get_level` resolves the flat id back to its level file
/// via the mode-levels map (returned alongside the index).
fn entry_from_mode_level(
    rel: &str,
    info: &StageInfo,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
    art: &ArtIndex,
) -> StageIndexEntry {
    let id = mode_stage_id(rel);
    // IS per-node previews are keyed by the topic-table node id (`ro1_n_4_5`),
    // not the level filename — resolve that first, then fall through.
    let preview = art
        .is_node_ids
        .get(rel)
        .and_then(|nid| art.preview.get(nid).cloned())
        .or_else(|| resolve_preview(art, id, Some(rel), &info.code));
    StageIndexEntry {
        preview,
        banner: resolve_banner(art, &info.zone_id, info.group, None),
        stage_id: id.to_owned(),
        level_id: Some(rel.to_owned()),
        code: info.code.clone(),
        name: info.stage_name.clone(),
        zone_order: zone_order(info.group, &info.zone_id, zones, activities),
        zone_id: info.zone_id.clone(),
        zone_name: info.zone_name.clone(),
        group: info.group.to_owned(),
        category: info.category.to_owned(),
        ap_cost: 0,
        boss: false,
        is_hard: info.is_hard,
        difficulty: "NORMAL".to_owned(),
        can_view: true,
    }
}

/// Build the full Stage List catalogue.
pub fn build_stage_index(
    assets_dir: &Path,
    levels_dir: &Path,
    data_dir: &Path,
    stages: &HashMap<String, Stage>,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> (StageIndex, HashMap<String, String>) {
    let classifier = StageClassifier::new(data_dir, stages, zones, activities);
    let art = build_art_index(assets_dir, data_dir);

    let mut out: StageIndex = Vec::with_capacity(stages.len() + 512);
    // Flat mode-node id -> relative level path, so the map viewer can resolve a
    // procedural-mode stage that has no stage_table entry.
    let mut mode_levels: HashMap<String, String> = HashMap::new();

    // 1. Every stage_table stage, classified into its authoritative group.
    for stage in stages.values() {
        let info = classifier.classify_stage_id(&stage.stage_id);
        let can_view = level_exists(levels_dir, stage.level_id.as_deref());
        out.push(entry_from_stage(
            stage, &info, zones, activities, can_view, &art,
        ));
    }

    // 2. Procedural-mode nodes that live only in level files (IS / RA / CC /
    //    Paradox). Skip anything already covered by a stage_table stage, and any
    //    non-mode orphan (event sub-levels are represented by their stage_table
    //    stage above).
    let mut files = Vec::new();
    collect_level_files(levels_dir, &mut files);
    let mut seen_mode: std::collections::HashSet<(String, String)> =
        std::collections::HashSet::new();
    for path in &files {
        let Some(rel) = relative_level_id(levels_dir, path) else {
            continue;
        };
        if classifier.stage_for_level(&rel).is_some() {
            continue; // has a stage_table stage — already emitted above.
        }
        let info = classifier.classify_level(&rel);
        if info.category != "modes" {
            continue; // only fill the procedural modes here.
        }
        // Collapse layout variants that resolve to the same season + node code.
        if !seen_mode.insert((info.zone_id.clone(), info.code.clone())) {
            continue;
        }
        let entry = entry_from_mode_level(&rel, &info, zones, activities, &art);
        mode_levels
            .entry(entry.stage_id.clone())
            .or_insert_with(|| rel.clone());
        out.push(entry);
    }

    (out, mode_levels)
}
