use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::level::LevelTables;
use myrtle_gamedata_raw::tables::stage::{Stage, StageDifficulty};
use myrtle_gamedata_raw::tables::zone::Zone;

use crate::model::stage_index::{StageIndex, StageIndexEntry};

use super::enemy_stages::relative_level_id;
use super::stage_class::{StageClassifier, StageInfo};

const fn indexed_difficulty(d: &StageDifficulty) -> StageDifficulty {
    match d {
        StageDifficulty::FourStar => StageDifficulty::FourStar,
        StageDifficulty::SixStar => StageDifficulty::SixStar,
        StageDifficulty::Normal | StageDifficulty::Unknown => StageDifficulty::Normal,
    }
}

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
    if let Some(a) = activities.get(zone_id)
        && a.start_time > 0
    {
        return a.start_time;
    }
    zone.map_or(0, |z| i64::from(z.zone_index))
}

fn level_exists(level_files: &HashSet<&str>, level_id: Option<&str>) -> bool {
    let Some(level_id) = level_id else {
        return false;
    };
    let rel = level_id.to_lowercase().replace('\\', "/");
    level_files.contains(format!("{rel}.json").as_str())
}

struct ArtIndex<'a> {
    preview: HashMap<String, String>,
    banner: HashMap<String, String>,
    is_node_ids: &'a HashMap<String, String>,
    is_kv: &'a HashMap<String, String>,
}

fn png_in_subfolder<'p>(rel: &'p str, parent: &str) -> Option<(&'p str, &'p str)> {
    let (folder, file) = rel
        .strip_prefix(parent)?
        .strip_prefix('/')?
        .split_once('/')?;
    if file.contains('/') {
        return None;
    }
    let (stem, ext) = file.rsplit_once('.')?;
    (!stem.is_empty() && ext == "png").then_some((folder, stem))
}

fn index_pngs(
    asset_paths: &[String],
    parent: &str,
    folder_ok: impl Fn(&str) -> bool,
    out: &mut HashMap<String, String>,
) {
    for rel in asset_paths {
        let Some((folder, stem)) = png_in_subfolder(rel, parent) else {
            continue;
        };
        if !folder_ok(folder) {
            continue;
        }
        out.entry(stem.to_owned())
            .or_insert_with(|| format!("{parent}/{folder}/{stem}.png"));
    }
}

fn build_art_index<'a>(
    asset_paths: &[String],
    classifier: &'a StageClassifier<'_>,
) -> ArtIndex<'a> {
    let mut preview = HashMap::new();
    index_pngs(
        asset_paths,
        "textures/arts/ui",
        |f| f.contains("mappreview"),
        &mut preview,
    );
    index_pngs(
        asset_paths,
        "textures/spritepack",
        |f| f.contains("mappreview"),
        &mut preview,
    );

    let mut banner = HashMap::new();
    index_pngs(
        asset_paths,
        "textures/spritepack",
        |f| f.contains("banner_zone") || f.starts_with("ui_zone_home_theme_act"),
        &mut banner,
    );
    index_named_pngs(
        asset_paths,
        "textures/avg/imgs",
        |stem| stem.starts_with("pic_rogue_") && stem.contains("_KV"),
        &mut banner,
    );

    ArtIndex {
        preview,
        banner,
        is_node_ids: classifier.is_node_ids(),
        is_kv: classifier.is_kv(),
    }
}

fn index_named_pngs(
    asset_paths: &[String],
    parent: &str,
    stem_ok: impl Fn(&str) -> bool,
    out: &mut HashMap<String, String>,
) {
    for rel in asset_paths {
        let Some((folder, stem)) = png_in_subfolder(rel, parent) else {
            continue;
        };
        if stem_ok(stem) {
            out.entry(stem.to_owned())
                .or_insert_with(|| format!("{parent}/{folder}/{stem}.png"));
        }
    }
}

fn preview_keys(stage_id: &str, level_id: Option<&str>, code: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let base = stage_id.split('#').next().unwrap_or(stage_id);
    keys.push(stage_id.to_owned());
    if base != stage_id {
        keys.push(base.to_owned());
    }
    if let Some(b) = base.strip_suffix("_ex") {
        keys.push(b.to_owned());
    }
    if let Some(lid) = level_id {
        let file = lid.rsplit(['/', '\\']).next().unwrap_or(lid).to_lowercase();
        let node = file.strip_prefix("level_").unwrap_or(&file);
        if let Some(rest) = node.strip_prefix("memory_") {
            keys.push(format!("mem_{rest}"));
        }
        if let Some(rest) = node.strip_prefix("sandbox") {
            keys.push(format!("sandbox_{}", rest.trim_start_matches('_')));
        }
        keys.push(node.to_owned());
    }
    if !code.is_empty() {
        keys.push(format!("crisis_v2_{code}"));
    }
    keys
}

fn resolve_preview(
    art: &ArtIndex<'_>,
    stage_id: &str,
    level_id: Option<&str>,
    code: &str,
) -> Option<String> {
    preview_keys(stage_id, level_id, code)
        .into_iter()
        .find_map(|k| art.preview.get(&k).cloned())
}

fn resolve_banner(
    art: &ArtIndex<'_>,
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
    if group == "is" {
        if let Some(kv) = art.is_kv.get(zone_id) {
            keys.push(kv.clone());
        }
        keys.push(format!("pic_{zone_id}_KV2"));
        keys.push(format!("pic_{zone_id}_KV1"));
    }
    keys.into_iter().find_map(|k| art.banner.get(&k).cloned())
}

fn entry_from_stage(
    stage: &Stage,
    info: &StageInfo,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
    can_view: bool,
    art: &ArtIndex<'_>,
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
        originite_prime: stage.diamond_once_drop.max(0),
        is_hard: info.is_hard,
        difficulty: indexed_difficulty(&stage.difficulty),
        can_view,
    }
}

fn mode_stage_id(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

fn entry_from_mode_level(
    rel: &str,
    info: &StageInfo,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
    art: &ArtIndex<'_>,
) -> StageIndexEntry {
    let id = mode_stage_id(rel);
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
        originite_prime: 0,
        is_hard: info.is_hard,
        difficulty: StageDifficulty::Normal,
        can_view: true,
    }
}

pub fn build_stage_index(
    asset_paths: &[String],
    levels: &LevelTables,
    classifier: &StageClassifier<'_>,
    stages: &HashMap<String, Stage>,
    zones: &HashMap<String, Zone>,
    activities: &HashMap<String, ActivityBasicInfo>,
) -> (StageIndex, HashMap<String, String>) {
    let art = build_art_index(asset_paths, classifier);
    let level_files: HashSet<&str> = levels.files.iter().map(String::as_str).collect();

    let mut out: StageIndex = Vec::with_capacity(stages.len() + 512);
    let mut mode_levels: HashMap<String, String> = HashMap::new();

    for stage in stages.values() {
        let info = classifier.classify_stage_id(&stage.stage_id);
        let can_view = level_exists(&level_files, stage.level_id.as_deref());
        out.push(entry_from_stage(
            stage, &info, zones, activities, can_view, &art,
        ));
    }

    let mut seen_mode: HashSet<(String, String)> = HashSet::new();
    for (path, _) in &levels.levels {
        let rel = relative_level_id(path);
        if classifier.stage_for_level(&rel).is_some() {
            continue;
        }
        let info = classifier.classify_level(&rel);
        if info.category != "modes" {
            continue;
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

    out.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));
    (out, mode_levels)
}

#[cfg(test)]
mod tests {
    use super::{mode_stage_id, preview_keys};

    #[test]
    fn plain_stage_is_its_own_only_key() {
        assert_eq!(preview_keys("main_01-07", None, ""), vec!["main_01-07"]);
    }

    #[test]
    fn challenge_mode_and_ex_variants_fall_back_to_the_base_stage() {
        assert_eq!(
            preview_keys("main_11-01#f#", None, ""),
            vec!["main_11-01#f#", "main_11-01"]
        );
        assert_eq!(
            preview_keys("lt_08_02_ex", None, ""),
            vec!["lt_08_02_ex", "lt_08_02"]
        );
        assert_eq!(
            preview_keys("act1_ex#f#", None, ""),
            vec!["act1_ex#f#", "act1_ex", "act1"]
        );
    }

    #[test]
    fn procedural_nodes_use_their_level_file() {
        assert_eq!(
            preview_keys("ro4_n_1", Some("Obt/Roguelike/RO4/level_rogue4_b-4-c"), ""),
            vec!["ro4_n_1", "rogue4_b-4-c"]
        );
        assert_eq!(
            preview_keys("mem_x", Some("Obt/Memory/level_memory_gdglow_1"), ""),
            vec!["mem_x", "mem_gdglow_1", "memory_gdglow_1"]
        );
        assert_eq!(
            preview_keys("sb", Some("obt/sandbox/level_sandbox1_04"), ""),
            vec!["sb", "sandbox_1_04", "sandbox1_04"]
        );
        // Windows separators and case are normalized.
        assert_eq!(
            preview_keys("x", Some("Obt\\Main\\Level_Main_01-07"), ""),
            vec!["x", "main_01-07"]
        );
    }

    #[test]
    fn a_contract_code_adds_the_crisis_key_last() {
        assert_eq!(
            preview_keys("cc_x", Some("obt/crisis/level_crisis_v2_01-02"), "01-02"),
            vec!["cc_x", "crisis_v2_01-02", "crisis_v2_01-02"]
        );
    }

    #[test]
    fn mode_stage_id_is_the_last_path_segment() {
        assert_eq!(
            mode_stage_id("obt/roguelike/ro1/level_rogue1_6-1"),
            "level_rogue1_6-1"
        );
        assert_eq!(mode_stage_id("level_only"), "level_only");
        assert_eq!(mode_stage_id("trailing/"), "");
    }
}
