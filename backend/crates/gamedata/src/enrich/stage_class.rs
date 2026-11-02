use std::collections::{HashMap, hash_map::Entry};

use myrtle_gamedata_raw::tables::{
    activity::ActivityBasicInfo,
    crisis::CrisisV2TableFile,
    handbook::HandbookStageData,
    roguelike::RoguelikeTopicTableFile,
    stage::{ClimbTower, Stage, StageDifficulty, StageType},
    zone::{Zone, ZoneType},
};
use serde_json::Value;

pub struct StageInfo {
    pub code: String,
    pub zone_id: String,
    pub zone_name: Option<String>,
    pub stage_name: Option<String>,
    pub category: &'static str,
    pub group: &'static str,
    pub is_hard: bool,
}

pub struct StageClassifier<'a> {
    stages: &'a HashMap<String, Stage>,
    zones: &'a HashMap<String, Zone>,
    acts_by_len: Vec<&'a ActivityBasicInfo>,
    modes: ModeNames,
    memory_meta: HashMap<String, (String, String)>,
    stage_by_level: HashMap<String, &'a Stage>,
}

impl<'a> StageClassifier<'a> {
    #[must_use]
    pub fn new(
        stages: &'a HashMap<String, Stage>,
        zones: &'a HashMap<String, Zone>,
        activities: &'a HashMap<String, ActivityBasicInfo>,
        tables: &ModeTables<'_>,
    ) -> Self {
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
            modes: load_mode_names(tables),
            memory_meta: load_memory_meta(tables.handbook_stages),
            stage_by_level,
        }
    }

    #[must_use]
    pub const fn is_node_ids(&self) -> &HashMap<String, String> {
        &self.modes.is_node_ids
    }

    #[must_use]
    pub const fn is_kv(&self) -> &HashMap<String, String> {
        &self.modes.is_kv
    }

    #[must_use]
    pub fn stage_for_level(&self, rel: &str) -> Option<&'a Stage> {
        self.stage_by_level.get(rel).copied()
    }

    #[must_use]
    pub fn classify_level(&self, rel: &str) -> StageInfo {
        self.mode_location(rel)
            .or_else(|| self.stage_by_level.get(rel).map(|s| self.stage_location(s)))
            .unwrap_or_else(|| self.orphan_location(rel))
    }

    #[must_use]
    pub fn classify_stage_id(&self, stage_id: &str) -> StageInfo {
        self.stages.get(stage_id).map_or_else(
            || self.orphan_location(stage_id),
            |s| self.stage_location(s),
        )
    }

    #[must_use]
    pub fn boss_appearances(&self, bosses: &[(String, String)]) -> Vec<(String, StageInfo)> {
        bosses
            .iter()
            .map(|(enemy_id, stage_id)| (enemy_id.clone(), self.classify_stage_id(stage_id)))
            .collect()
    }

    fn resolve_zone_name(&self, zone_id: &str) -> Option<String> {
        let zone = self.zones.get(zone_id);
        if let Some(z) = zone
            && let Some(n) = z.zone_name_second.as_deref().filter(|s| s.is_empty())
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

        if zone_id.starts_with("rogue")
            && let Some(name) = is_topic_key(zone_id).and_then(|k| self.modes.is_topics.get(&k))
        {
            return Some(name.clone());
        }

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

    fn event_activity(&self, zone_id: &str) -> Option<&'a ActivityBasicInfo> {
        self.acts_by_len
            .iter()
            .find(|a| !a.id.is_empty() && !a.name.is_empty() && zone_id.starts_with(&a.id))
            .copied()
    }

    fn stage_location(&self, stage: &Stage) -> StageInfo {
        let zone_type = self.zones.get(&stage.zone_id).map(|z| &z.zone_type);
        let category = zone_category(zone_type);

        if stage.stage_type == StageType::Activity
            && category == "stages"
            && let Some(act) = activity_for_stage_id(&self.acts_by_len, &stage.stage_id)
        {
            return StageInfo {
                code: stage.code.clone(),
                zone_id: act.id.clone(),
                zone_name: Some(act.name.clone()),
                stage_name: stage.name.clone(),
                category: "events",
                group: "events",
                is_hard: is_hard_stage(stage),
            };
        }

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

    fn mode_location(&self, rel: &str) -> Option<StageInfo> {
        let segs: Vec<&str> = rel.split('/').collect();
        let file = *segs.last()?;
        let body = file.strip_prefix("level_").unwrap_or(file);

        if let Some(i) = segs.iter().position(|&s| s == "roguelike") {
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

fn activity_for_stage_id<'a>(
    acts_by_len: &[&'a ActivityBasicInfo],
    stage_id: &str,
) -> Option<&'a ActivityBasicInfo> {
    acts_by_len
        .iter()
        .find(|a| {
            !a.id.is_empty()
                && !a.name.is_empty()
                && stage_id
                    .strip_prefix(a.id.as_str())
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with(['_', '#']))
        })
        .copied()
}

struct ModeNames {
    is_topics: HashMap<String, String>,
    towers: HashMap<String, String>,
    node_named: HashMap<String, (String, String)>,
    cc_seasons: HashMap<u32, String>,
    is_node_ids: HashMap<String, String>,
    is_kv: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ModeTables<'a> {
    /// `roguelike_topic_table`
    pub roguelike: &'a RoguelikeTopicTableFile,
    /// `climb_tower_table.Towers`
    pub towers: &'a HashMap<String, ClimbTower>,
    /// `sandbox_table`
    pub sandbox: &'a Value,
    /// `sandbox_perm_table`
    pub sandbox_perm: &'a Value,
    /// `crisis_v2_table`
    pub crisis_v2: &'a CrisisV2TableFile,
    /// `handbook_info_table.HandbookStageData`
    pub handbook_stages: &'a HashMap<String, HandbookStageData>,
}

fn fb_list_names(list: &Value, field: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();

    if let Some(arr) = list.as_array() {
        for item in arr {
            if let (Some(k), Some(name)) = (
                item.get("key").and_then(Value::as_str),
                item.pointer(&format!("/value/{field}"))
                    .and_then(Value::as_str),
            ) && name.is_empty()
            {
                out.insert(k.to_owned(), name.to_owned());
            }
        }
    }

    out
}

fn fb_str<'a>(entry: &'a Value, field: &str) -> Option<&'a str> {
    entry.get(field).and_then(Value::as_str)
}

fn tower_names(towers: &HashMap<String, ClimbTower>) -> HashMap<String, String> {
    towers
        .iter()
        .filter_map(|(id, tower)| {
            tower
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(|name| (id.clone(), name.to_owned()))
        })
        .collect()
}

fn is_hard_path(level_id: &str) -> bool {
    let l = level_id.to_lowercase();
    l.contains("tough") || l.contains("/hard/") || l.contains("_hard_")
}

fn is_hard_stage(stage: &Stage) -> bool {
    is_hard_path(stage.level_id.as_deref().unwrap_or(""))
        || !matches!(stage.difficulty, StageDifficulty::Normal)
}

fn is_topic_key(season_id: &str) -> Option<String> {
    let digits = season_id.trim_start_matches(|c: char| !c.is_ascii_digit());
    let n: u32 = digits.parse().ok()?;
    Some(format!("rogue_{n}"))
}

fn canonical_rank(stage: &Stage) -> (u8, u8) {
    (
        u8::from(stage.stage_id.contains('#')),
        u8::from(!matches!(stage.difficulty, StageDifficulty::Normal)),
    )
}

fn node_code(body: &str) -> String {
    body.split_once('_')
        .map_or(body, |(_, rest)| rest)
        .to_owned()
}

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

fn collect_is_topics(
    details: &Value,
    node_ids: &mut HashMap<String, String>,
    kv: &mut HashMap<String, String>,
) {
    let Some(details) = details.as_array() else {
        return;
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
}

fn load_cc_seasons(table: &CrisisV2TableFile) -> HashMap<u32, String> {
    let mut out = HashMap::new();
    let entries: Vec<(&str, &Value)> = match &table.season_info_data_map {
        Value::Object(m) => m.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        Value::Array(a) => a
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

fn load_memory_meta(
    stages: &HashMap<String, HandbookStageData>,
) -> HashMap<String, (String, String)> {
    let mut sorted: Vec<(&String, &HandbookStageData)> = stages.iter().collect();
    sorted.sort_unstable_by_key(|(key, _)| key.as_str());
    let mut out = HashMap::new();
    for (_, v) in sorted {
        let code = if v.code.is_empty() { &v.name } else { &v.code };
        out.insert(v.level_id.to_lowercase(), (code.clone(), v.name.clone()));
    }
    out
}

fn load_mode_names(tables: &ModeTables<'_>) -> ModeNames {
    let roguelike = tables.roguelike;
    let is_topics = fb_list_names(&roguelike.topics, "Name");
    let towers = tower_names(tables.towers);

    let mut node_named = HashMap::new();
    let mut is_node_ids = HashMap::new();
    let mut is_kv = HashMap::new();

    collect_level_named(&roguelike.topics, &mut node_named);
    collect_level_named(&roguelike.details, &mut node_named);
    collect_is_topics(&roguelike.details, &mut is_node_ids, &mut is_kv);
    collect_level_named(tables.sandbox, &mut node_named);
    collect_level_named(tables.sandbox_perm, &mut node_named);

    ModeNames {
        is_topics,
        towers,
        node_named,
        cc_seasons: load_cc_seasons(tables.crisis_v2),
        is_node_ids,
        is_kv,
    }
}

const fn mode_type_label(zone_type: Option<&ZoneType>) -> Option<&'static str> {
    match zone_type {
        Some(ZoneType::Campaign) => Some("Annihilation"),
        Some(ZoneType::ClimbTower) => Some("Stationary Security Service"),
        Some(ZoneType::Roguelike) => Some("Integrated Strategies"),
        Some(ZoneType::Weekly) => Some("Supplies"),
        _ => None,
    }
}
