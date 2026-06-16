//! Stage classification: the single source of truth for turning a level file
//! (or `stage_table` stage) into canonical display info - its zone/event,
//! coarse category, fine group, code and name.
//!
//! This is shared by the enemy -> stages index and is designed to back a
//! future stage viewer, so all of the per-mode quirk handling (Integrated
//! Strategies seasons, Reclamation Algorithm, Paradox Simulation, Contingency
//! Contract, Annihilation, event EX zones, ...) lives here rather than being
//! re-derived by callers or the frontend.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::path::Path;

use serde_json::Value;

use crate::core::gamedata::types::activity::ActivityBasicInfo;
use crate::core::gamedata::types::stage::{Stage, StageDifficulty};
use crate::core::gamedata::types::zone::{Zone, ZoneType};

/// Canonical, display-ready info for one stage / level.
pub struct StageInfo {
    /// Short per-stage label (in-game stage code, IS/RA node name, etc.).
    pub code: String,
    /// Grouping key the UI buckets by (event/chapter/season/mode id).
    pub zone_id: String,
    /// Resolved zone/event display name; `None` falls back to `zone_id`.
    pub zone_name: Option<String>,
    /// Longer per-stage name, when distinct from `code`.
    pub stage_name: Option<String>,
    /// Coarse bucket: `"stages"`, `"events"`, or `"modes"`.
    pub category: &'static str,
    /// Fine group: `story` / `events` / `annihilation` / `is` / `ra` / `sss` /
    /// `paradox` / `cc` / `supplies` / `other`.
    pub group: &'static str,
    /// Adverse / hard-mode variant.
    pub is_hard: bool,
}

// ============================================================================
// Raw gamedata reading
// ============================================================================

/// Read `data_dir/<name>.json` as a loose JSON value.
///
/// Some tables (e.g. `roguelike_topic_table`, `activity_table`) carry invalid
/// UTF-8 in flavor text, which makes `serde_json::from_slice` reject the whole
/// file. Lossy-decode first so the table still parses - the bad bytes only ever
/// live in descriptions, never in the keys/names/level ids read here.
pub fn read_json(data_dir: &Path, name: &str) -> Option<Value> {
    let bytes = std::fs::read(data_dir.join(format!("{name}.json"))).ok()?;
    serde_json::from_str(&String::from_utf8_lossy(&bytes)).ok()
}

/// Pull `{key -> value.<field>}` out of a FlatBuffer-style map (a JSON array of
/// `{ "key": ..., "value": {...} }` objects).
fn fb_map_names(table: &Value, list_key: &str, field: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Some(arr) = table.get(list_key).and_then(Value::as_array) {
        for item in arr {
            if let (Some(k), Some(name)) = (
                item.get("key").and_then(Value::as_str),
                item.pointer(&format!("/value/{field}"))
                    .and_then(Value::as_str),
            ) && !name.is_empty()
            {
                out.insert(k.to_owned(), name.to_owned());
            }
        }
    }
    out
}

/// Recursively collect every object that carries a `levelId`/`LevelId` plus a
/// `code`/`name`, keyed by lowercased level id. Robust to the differing shapes
/// of the roguelike (`Details[].Stages[]`) and sandbox (`stageDatas[]`) tables.
fn collect_level_named(v: &Value, out: &mut HashMap<String, (String, String)>) {
    match v {
        Value::Object(map) => {
            if let Some(lid) = map
                .get("LevelId")
                .or_else(|| map.get("levelId"))
                .and_then(Value::as_str)
            {
                let code = map
                    .get("Code")
                    .or_else(|| map.get("code"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let name = map
                    .get("Name")
                    .or_else(|| map.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if !code.is_empty() || !name.is_empty() {
                    out.entry(lid.to_lowercase())
                        .or_insert_with(|| (code.to_owned(), name.to_owned()));
                }
            }
            for val in map.values() {
                collect_level_named(val, out);
            }
        }
        Value::Array(arr) => {
            for x in arr {
                collect_level_named(x, out);
            }
        }
        _ => {}
    }
}

/// Parse `crisis_v2_table.SeasonInfoDataMap` into `season number -> name`
/// (key `crisis_v2_season_2_1` -> `2`).
fn load_cc_seasons(data_dir: &Path) -> HashMap<u32, String> {
    let mut out = HashMap::new();
    let Some(table) = read_json(data_dir, "crisis_v2_table") else {
        return out;
    };
    let entries: Vec<(&str, &Value)> = match table.get("SeasonInfoDataMap") {
        Some(Value::Object(m)) => m.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|it| Some((it.get("key")?.as_str()?, it.get("value")?)))
            .collect(),
        _ => return out,
    };
    for (key, v) in entries {
        if let Some(n) = key.split('_').nth(3).and_then(|s| s.parse::<u32>().ok())
            && let Some(name) = v
                .get("Name")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
        {
            out.entry(n).or_insert_with(|| name.to_owned());
        }
    }
    out
}

/// Per-operator Paradox Simulation stage metadata `(code, name)`, keyed by the
/// lowercased `LevelId`.
fn load_memory_meta(data_dir: &Path) -> HashMap<String, (String, String)> {
    let mut out = HashMap::new();
    let Some(table) = read_json(data_dir, "handbook_info_table") else {
        return out;
    };
    let Some(arr) = table.get("HandbookStageData").and_then(Value::as_array) else {
        return out;
    };
    for item in arr {
        let v = item.get("value").unwrap_or(item);
        if let (Some(lid), Some(name)) = (
            v.get("LevelId").and_then(Value::as_str),
            v.get("Name").and_then(Value::as_str),
        ) {
            let code = v
                .get("Code")
                .and_then(Value::as_str)
                .unwrap_or(name)
                .to_owned();
            out.insert(lid.to_lowercase(), (code, name.to_owned()));
        }
    }
    out
}

/// Names for the permanent game modes, pulled from each mode's own table.
struct ModeNames {
    /// IS topic id (`rogue_1`) -> season name.
    is_topics: HashMap<String, String>,
    /// Climb-tower zone id (`tower_n_18`) -> tower name.
    towers: HashMap<String, String>,
    /// Lowercased `LevelId` -> `(code, name)` for procedural-mode nodes, so a
    /// node file resolves to its real in-game stage code/name instead of the
    /// raw file node id.
    node_named: HashMap<String, (String, String)>,
    /// Contingency Contract (`crisis_v2`) season number -> display name.
    cc_seasons: HashMap<u32, String>,
}

fn load_mode_names(data_dir: &Path) -> ModeNames {
    let roguelike = read_json(data_dir, "roguelike_topic_table");
    let is_topics = roguelike
        .as_ref()
        .map(|t| fb_map_names(t, "Topics", "Name"))
        .unwrap_or_default();
    let towers = read_json(data_dir, "climb_tower_table")
        .map(|t| fb_map_names(&t, "Towers", "Name"))
        .unwrap_or_default();

    let mut node_named = HashMap::new();
    if let Some(t) = &roguelike {
        collect_level_named(t, &mut node_named);
    }
    if let Some(t) = read_json(data_dir, "sandbox_table") {
        collect_level_named(&t, &mut node_named);
    }
    // RA stage names that match the extracted level paths live in the perm
    // table (keyed by the full level id under `obt/sandbox/`).
    if let Some(t) = read_json(data_dir, "sandbox_perm_table") {
        collect_level_named(&t, &mut node_named);
    }

    ModeNames {
        is_topics,
        towers,
        node_named,
        cc_seasons: load_cc_seasons(data_dir),
    }
}

/// Collect `(enemy_id, stage_id)` from any `BossData.EnemyId` declared inside a
/// `FlatBuffer` map entry (`{key: <stage_id>, value: {... BossData: {EnemyId}}}`)
/// anywhere in `activity_table` - some bosses are never in a level file.
fn collect_boss_stages(v: &Value, out: &mut Vec<(String, String)>) {
    match v {
        Value::Array(arr) => {
            for item in arr {
                if let (Some(k), Some(eid)) = (
                    item.get("key").and_then(Value::as_str),
                    item.pointer("/value/BossData/EnemyId")
                        .and_then(Value::as_str),
                ) && eid.starts_with("enemy_")
                {
                    out.push((eid.to_owned(), k.to_owned()));
                }
                collect_boss_stages(item, out);
            }
        }
        Value::Object(map) => {
            for val in map.values() {
                collect_boss_stages(val, out);
            }
        }
        _ => {}
    }
}

// ============================================================================
// Pure classification helpers
// ============================================================================

fn is_hard_path(level_id: &str) -> bool {
    let l = level_id.to_lowercase();
    l.contains("tough") || l.contains("/hard/") || l.contains("_hard_")
}

fn is_hard_stage(stage: &Stage) -> bool {
    is_hard_path(stage.level_id.as_deref().unwrap_or(""))
        || !matches!(stage.difficulty, StageDifficulty::Normal)
}

/// Rank a stage as the representative for its level file (lower wins). Prefers
/// non-Challenge-Mode ids (a `#` marks a CM that reuses its base level) then
/// Normal difficulty.
fn canonical_rank(stage: &Stage) -> (u8, u8) {
    (
        u8::from(stage.stage_id.contains('#')),
        u8::from(!matches!(stage.difficulty, StageDifficulty::Normal)),
    )
}

/// Map an Integrated Strategies zone/season id (`rogue_4`, or a level prefix
/// like `rogue4`) to its topic key (`rogue_4`).
fn is_topic_key(season_id: &str) -> Option<String> {
    let digits = season_id.trim_start_matches(|c: char| !c.is_ascii_digit());
    let n: u32 = digits.parse().ok()?;
    Some(format!("rogue_{n}"))
}

/// The node id within a procedural-mode filename, i.e. everything after the
/// mode prefix: `rogue4_b-4-c` -> `b-4-c`, `sandbox2_04` -> `04`.
fn node_code(body: &str) -> String {
    body.split_once('_')
        .map_or(body, |(_, rest)| rest)
        .to_owned()
}

/// Resolve a procedural node to `(chip_label, tooltip_code)`. Prefers the
/// stage's real name with its code in the tooltip; falls back to the code, then
/// the raw file node id.
fn named_node(modes: &ModeNames, rel: &str, body: &str) -> (String, Option<String>) {
    if let Some((code, name)) = modes.node_named.get(rel) {
        if !name.is_empty() {
            return (name.clone(), (!code.is_empty()).then(|| code.clone()));
        }
        if !code.is_empty() {
            return (code.clone(), None);
        }
    }
    (node_code(body), None)
}

/// Coarse UI bucket for a zone, derived from its type.
const fn zone_category(zone_type: Option<&ZoneType>) -> &'static str {
    match zone_type {
        Some(
            ZoneType::Mainline
            | ZoneType::MainlineActivity
            | ZoneType::MainlineRetro
            | ZoneType::Guide,
        ) => "stages",
        Some(ZoneType::Activity | ZoneType::Sidestory | ZoneType::Branchline) => "events",
        _ => "modes",
    }
}

/// Fine group for a `stage_table`/path-derived zone (the procedural modes set
/// their group directly when classifying).
const fn group_for(category: &str, zone_type: Option<&ZoneType>) -> &'static str {
    match category.as_bytes() {
        b"stages" => "story",
        b"events" => "events",
        _ => match zone_type {
            Some(ZoneType::Campaign) => "annihilation",
            Some(ZoneType::ClimbTower) => "sss",
            Some(ZoneType::Roguelike) => "is",
            Some(ZoneType::Weekly) => "supplies",
            _ => "other",
        },
    }
}

/// Fallback mode label keyed off the gamedata zone *type*. Only used when a
/// mode has no per-instance name in its table.
const fn mode_type_label(zone_type: Option<&ZoneType>) -> Option<&'static str> {
    match zone_type {
        Some(ZoneType::Campaign) => Some("Annihilation"),
        Some(ZoneType::ClimbTower) => Some("Stationary Security Service"),
        Some(ZoneType::Roguelike) => Some("Integrated Strategies"),
        Some(ZoneType::Weekly) => Some("Supplies"),
        _ => None,
    }
}

// ============================================================================
// Classifier
// ============================================================================

/// Holds the gamedata tables needed to classify any stage/level. Build once,
/// then call `classify_level` / `classify_stage_id` per item.
pub struct StageClassifier<'a> {
    stages: &'a HashMap<String, Stage>,
    zones: &'a HashMap<String, Zone>,
    acts_by_len: Vec<&'a ActivityBasicInfo>,
    modes: ModeNames,
    memory_meta: HashMap<String, (String, String)>,
    /// Lowercased level id -> the canonical `stage_table` stage at that file.
    stage_by_level: HashMap<String, &'a Stage>,
}

impl<'a> StageClassifier<'a> {
    pub fn new(
        data_dir: &Path,
        stages: &'a HashMap<String, Stage>,
        zones: &'a HashMap<String, Zone>,
        activities: &'a HashMap<String, ActivityBasicInfo>,
    ) -> Self {
        // Canonical stage per level file (a CM `#f#` reuses its base level).
        let mut stage_by_level: HashMap<String, &Stage> = HashMap::new();
        for stage in stages.values() {
            let Some(level_id) = stage.level_id.as_deref() else {
                continue;
            };
            match stage_by_level.entry(level_id.to_lowercase()) {
                Entry::Vacant(e) => {
                    e.insert(stage);
                }
                Entry::Occupied(mut e) => {
                    let better = canonical_rank(stage) < canonical_rank(e.get())
                        || (canonical_rank(stage) == canonical_rank(e.get())
                            && stage.stage_id < e.get().stage_id);
                    if better {
                        e.insert(stage);
                    }
                }
            }
        }

        let mut acts_by_len: Vec<&ActivityBasicInfo> = activities.values().collect();
        acts_by_len.sort_by_key(|a| std::cmp::Reverse(a.id.len()));

        Self {
            stages,
            zones,
            acts_by_len,
            modes: load_mode_names(data_dir),
            memory_meta: load_memory_meta(data_dir),
            stage_by_level,
        }
    }

    /// The canonical `stage_table` stage at a level file, if any.
    pub fn stage_for_level(&self, rel: &str) -> Option<&'a Stage> {
        self.stage_by_level.get(rel).copied()
    }

    /// Classify a level file (lowercased relative id, no extension). Procedural
    /// modes group under their season/mode zone; otherwise resolve via
    /// `stage_table` or the path.
    pub fn classify_level(&self, rel: &str) -> StageInfo {
        self.mode_location(rel)
            .or_else(|| self.stage_by_level.get(rel).map(|s| self.stage_location(s)))
            .unwrap_or_else(|| self.orphan_location(rel))
    }

    /// Classify by `stage_table` id (used for boss declarations / a stage-table
    /// driven viewer); falls back to path derivation for ids without a stage.
    pub fn classify_stage_id(&self, stage_id: &str) -> StageInfo {
        self.stages.get(stage_id).map_or_else(
            || self.orphan_location(stage_id),
            |s| self.stage_location(s),
        )
    }

    /// Boss appearances declared only in `activity_table` as `(enemy_id, info)`.
    pub fn boss_appearances(&self, data_dir: &Path) -> Vec<(String, StageInfo)> {
        let Some(activity_table) = read_json(data_dir, "activity_table") else {
            return Vec::new();
        };
        let mut bosses = Vec::new();
        collect_boss_stages(&activity_table, &mut bosses);
        bosses
            .into_iter()
            .map(|(enemy_id, stage_id)| (enemy_id, self.classify_stage_id(&stage_id)))
            .collect()
    }

    fn resolve_zone_name(&self, zone_id: &str) -> Option<String> {
        let zone = self.zones.get(zone_id);
        if let Some(z) = zone
            && let Some(n) = z.zone_name_second.as_deref().filter(|s| !s.is_empty())
        {
            return Some(n.to_owned());
        }
        if let Some(act) = self
            .acts_by_len
            .iter()
            .find(|a| !a.id.is_empty() && zone_id.starts_with(&a.id))
            && !act.name.is_empty()
        {
            return Some(act.name.clone());
        }
        // Integrated Strategies season name (only for roguelike ids, so a digit
        // in e.g. `sandbox_2` / `tower_n_18` can't be mistaken for a topic).
        if zone_id.starts_with("rogue")
            && let Some(name) = is_topic_key(zone_id).and_then(|k| self.modes.is_topics.get(&k))
        {
            return Some(name.clone());
        }
        // Stationary Security Service tower (zone id == tower key).
        if let Some(name) = self.modes.towers.get(zone_id) {
            return Some(name.clone());
        }
        if let Some(z) = zone
            && let Some(n) = z.zone_name_first.as_deref().filter(|s| !s.is_empty())
        {
            return Some(n.to_owned());
        }
        mode_type_label(zone.map(|z| &z.zone_type)).map(str::to_owned)
    }

    /// The owning activity for a zone id, used to group an event's stages (its
    /// normal and EX/sub zones) under one parent rather than per-zone.
    fn event_activity(&self, zone_id: &str) -> Option<&'a ActivityBasicInfo> {
        self.acts_by_len
            .iter()
            .find(|a| !a.id.is_empty() && !a.name.is_empty() && zone_id.starts_with(&a.id))
            .copied()
    }

    fn stage_location(&self, stage: &Stage) -> StageInfo {
        let zone_type = self.zones.get(&stage.zone_id).map(|z| &z.zone_type);
        let category = zone_category(zone_type);

        // Events group by their parent activity so an event's normal and EX/sub
        // stages live under one entry instead of their separate in-event zone
        // names.
        let (zone_id, zone_name) = match category {
            "events" => self.event_activity(&stage.zone_id).map_or_else(
                || {
                    (
                        stage.zone_id.clone(),
                        self.resolve_zone_name(&stage.zone_id),
                    )
                },
                |act| (act.id.clone(), Some(act.name.clone())),
            ),
            _ => (
                stage.zone_id.clone(),
                self.resolve_zone_name(&stage.zone_id),
            ),
        };

        StageInfo {
            code: stage.code.clone(),
            zone_id,
            zone_name,
            stage_name: stage.name.clone(),
            category,
            group: group_for(category, zone_type),
            is_hard: is_hard_stage(stage),
        }
    }

    /// Derive info for an orphan level file (no `stage_table` entry) from its
    /// path/filename.
    fn orphan_location(&self, rel: &str) -> StageInfo {
        let segments: Vec<&str> = rel.split('/').collect();
        let file_stem = segments.last().copied().unwrap_or(rel);
        let body = file_stem.strip_prefix("level_").unwrap_or(file_stem);

        if let Some(act) = self
            .acts_by_len
            .iter()
            .find(|a| !a.id.is_empty() && body.starts_with(&a.id))
            .copied()
        {
            // Orphan event sub-levels group under the activity (same key as a
            // stage-table event) so they merge with the event's regular stages.
            let code = body
                .strip_prefix(&act.id)
                .unwrap_or(body)
                .trim_start_matches('_');
            return StageInfo {
                code: code.to_owned(),
                zone_id: act.id.clone(),
                zone_name: (!act.name.is_empty()).then(|| act.name.clone()),
                stage_name: None,
                category: "events",
                group: "events",
                is_hard: is_hard_path(rel),
            };
        }

        let family = segments.iter().rev().nth(1).copied().unwrap_or("unknown");
        let zone_type = self.zones.get(family).map(|z| &z.zone_type);
        let category = zone_category(zone_type);
        StageInfo {
            code: body.to_owned(),
            zone_id: family.to_owned(),
            zone_name: self.resolve_zone_name(family),
            stage_name: None,
            category,
            group: group_for(category, zone_type),
            is_hard: is_hard_path(rel),
        }
    }

    /// Classify a procedural-mode level file (IS / RA / Paradox / CC). Each node
    /// is kept as its own entry under the season/mode zone.
    fn mode_location(&self, rel: &str) -> Option<StageInfo> {
        let segs: Vec<&str> = rel.split('/').collect();
        let file = *segs.last()?;
        let body = file.strip_prefix("level_").unwrap_or(file);

        if let Some(i) = segs.iter().position(|&s| s == "roguelike") {
            // .../roguelike/ro4/level_rogue4_b-4-c -> season rogue_4, named node.
            let season = is_topic_key(segs.get(i + 1).copied().unwrap_or(""))?;
            let (code, stage_name) = named_node(&self.modes, rel, body);
            return Some(StageInfo {
                code,
                zone_name: self.resolve_zone_name(&season),
                zone_id: season,
                stage_name,
                category: "modes",
                group: "is",
                is_hard: is_hard_path(rel),
            });
        }
        if let Some(i) = segs.iter().position(|&s| s == "sandbox") {
            // .../sandbox/sandbox_2/level_sandbox2_04 -> season sandbox_2.
            let season = segs.get(i + 1).copied().unwrap_or("sandbox").to_owned();
            let zone_name = self
                .resolve_zone_name(&season)
                .or_else(|| Some("Reclamation Algorithm".to_owned()));
            let (code, stage_name) = named_node(&self.modes, rel, body);
            return Some(StageInfo {
                code,
                zone_id: season,
                zone_name,
                stage_name,
                category: "modes",
                group: "ra",
                is_hard: is_hard_path(rel),
            });
        }
        if segs.contains(&"crisis") {
            // Contingency Contract (crisis_v2): obt/crisis/v2/level_crisis_v2_02-03
            // -> season 2 ("Battleplan Underdawn"), stage code 02-03.
            let rest = body.strip_prefix("crisis_v2_").unwrap_or(body);
            let season = rest
                .split(['-', '_'])
                .next()
                .and_then(|s| s.parse::<u32>().ok());
            let name = season.and_then(|n| self.modes.cc_seasons.get(&n).cloned());
            return Some(StageInfo {
                code: rest.to_owned(),
                zone_id: season
                    .map_or_else(|| "crisis_v2".to_owned(), |n| format!("crisis_v2_{n}")),
                zone_name: Some(name.unwrap_or_else(|| "Contingency Contract".to_owned())),
                stage_name: None,
                category: "modes",
                group: "cc",
                is_hard: is_hard_path(rel),
            });
        }
        if segs.contains(&"recalrune") {
            // Contingency Contract reminiscence: story stage replays with runes.
            let (code, stage_name) = named_node(&self.modes, rel, body);
            return Some(StageInfo {
                code,
                zone_id: "recalrune".to_owned(),
                zone_name: Some("Contingency Contract".to_owned()),
                stage_name,
                category: "modes",
                group: "cc",
                is_hard: is_hard_path(rel),
            });
        }
        if segs.contains(&"memory") {
            // Paradox Simulation - one entry per operator record.
            let (code, name) = self
                .memory_meta
                .get(rel)
                .cloned()
                .unwrap_or_else(|| (node_code(body), String::new()));
            return Some(StageInfo {
                code,
                zone_id: "memory".to_owned(),
                zone_name: Some("Paradox Simulation".to_owned()),
                stage_name: (!name.is_empty()).then_some(name),
                category: "modes",
                group: "paradox",
                is_hard: false,
            });
        }
        None
    }
}
