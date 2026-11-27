use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SandboxUniverse {
    pub max_achievements: usize,
    pub max_nodes: usize,
    pub max_zones: usize,
    pub max_tech_nodes: usize,
    pub max_quests: usize,
    pub max_stages: usize,
    pub max_recipes: usize,
    pub max_music: usize,
    pub max_base_level: usize,
    pub max_blueprints: usize,
    pub max_rifts: usize,
    pub max_band_levels: usize,
    pub node_zones: BTreeMap<String, String>,
    pub topics: Vec<SandboxTopicUniverse>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SandboxTopicUniverse {
    pub topic_id: String,
    pub name: String,
    pub template: SandboxTemplate,
    pub sort_id: i64,
    pub counts: SandboxUniverse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SandboxTemplate {
    V2,
    V3,
    /// No reader yet
    Other(String),
}

impl SandboxTemplate {
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "SANDBOX_V2" => Self::V2,
            "SANDBOX_V3" => Self::V3,
            other => Self::Other(other.to_owned()),
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Self::V2 => "SANDBOX_V2",
            Self::V3 => "SANDBOX_V3",
            Self::Other(name) => name,
        }
    }

    fn reader(&self) -> Option<fn(&Value) -> SandboxUniverse> {
        match self {
            Self::V2 => Some(SandboxUniverse::count_v2),
            Self::V3 => Some(SandboxUniverse::count_v3),
            Self::Other(_) => None,
        }
    }
}

impl SandboxUniverse {
    #[must_use]
    pub fn build(raw: &Value) -> Self {
        Self::build_reporting(raw, &mut Vec::new())
    }

    pub fn build_reporting(raw: &Value, warnings: &mut Vec<String>) -> Self {
        let topics = Self::build_topics(raw, warnings);
        // The flat totals are the first `Detail.SANDBOX_V2` entry, keyed or
        // not, as main and gamedata's copy of this type read them.
        let Some(sandbox_data) = Self::extract_sandbox_v2(raw) else {
            warnings.push("SandboxUniverse: no SANDBOX_V2 data found".to_owned());
            return Self {
                topics,
                ..Self::default()
            };
        };
        Self {
            topics,
            ..Self::count_v2(sandbox_data)
        }
    }

    fn extract_sandbox_v2(raw: &Value) -> Option<&Value> {
        raw.get("Detail")?
            .get("SANDBOX_V2")?
            .as_array()?
            .first()?
            .get("value")
    }

    fn build_topics(raw: &Value, warnings: &mut Vec<String>) -> Vec<SandboxTopicUniverse> {
        let Some(detail) = raw.get("Detail") else {
            return Vec::new();
        };

        let mut topics: Vec<SandboxTopicUniverse> = keyed_entries(raw.get("BasicInfo"))
            .into_iter()
            .map(|(key, info)| {
                let field = |name: &str| info.get(name).and_then(Value::as_str);
                SandboxTopicUniverse {
                    topic_id: field("TopicId").unwrap_or(key).to_owned(),
                    name: field("TopicName").unwrap_or_default().to_owned(),
                    template: SandboxTemplate::from_name(
                        field("TopicTemplate").unwrap_or_default(),
                    ),
                    sort_id: info
                        .get("SortId")
                        .and_then(Value::as_i64)
                        .unwrap_or(i64::MAX),
                    counts: Self::default(),
                }
            })
            .collect();
        if topics.is_empty() {
            topics = detail
                .as_object()
                .into_iter()
                .flatten()
                .flat_map(|(template, entries)| {
                    keyed_entries(Some(entries))
                        .into_iter()
                        .map(move |(key, _)| (template, key))
                })
                .zip(0..)
                .map(|((template, key), sort_id)| SandboxTopicUniverse {
                    topic_id: key.to_owned(),
                    name: String::new(),
                    template: SandboxTemplate::from_name(template),
                    sort_id,
                    counts: Self::default(),
                })
                .collect();
        }

        for topic in &mut topics {
            let data = keyed_entries(detail.get(topic.template.key()))
                .into_iter()
                .find(|(key, _)| *key == topic.topic_id)
                .map(|(_, value)| value);
            match (topic.template.reader(), data) {
                (Some(read), Some(data)) => topic.counts = read(data),
                _ => warnings.push(format!(
                    "SandboxUniverse: topic {} ({}) has no reader or no Detail entry; not graded",
                    topic.topic_id,
                    topic.template.key()
                )),
            }
        }
        topics.sort_by_key(|t| t.sort_id);
        topics
    }

    fn count_v2(data: &Value) -> Self {
        let zones = data
            .get("ZoneData")
            .and_then(Value::as_array)
            .map_or(0, |rows| {
                rows.iter()
                    .filter(|e| {
                        e.get("key")
                            .and_then(Value::as_str)
                            .is_none_or(|k| !k.contains("tutorial"))
                    })
                    .count()
            });

        Self {
            max_achievements: rows(data, "AchievementData"),
            max_nodes: Self::count_map_nodes(data),
            max_zones: zones,
            max_tech_nodes: rows(data, "DevelopmentData"),
            max_quests: rows(data, "ArchiveQuestData"),
            max_stages: rows(data, "StageData"),
            max_recipes: rows(data, "FoodData"),
            max_music: rows(data, "ArchiveMusicUnlockData"),
            max_base_level: rows(data, "BaseUpdate"),
            max_blueprints: rows(data, "CraftItemData"),
            max_rifts: rows(data, "FixedRiftData"),
            ..Self::default()
        }
    }

    fn count_v3(data: &Value) -> Self {
        // The save never marks HOME explored, so it is not counted
        let node_zones: BTreeMap<String, String> =
            keyed_entries(data.get("MainMapData").and_then(|m| m.get("Nodes")))
                .into_iter()
                .filter(|(_, node)| node.get("NodeType").and_then(Value::as_str) != Some("HOME"))
                .map(|(id, node)| {
                    let zone = node
                        .get("ZoneId")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    (id.to_owned(), zone.to_owned())
                })
                .collect();
        let max_zones = node_zones
            .values()
            .filter(|z| !z.is_empty())
            .collect::<BTreeSet<_>>()
            .len();
        let max_band_levels = keyed_entries(data.get("BandDataMap"))
            .into_iter()
            .map(|(_, band)| {
                band.get("LevelList")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len)
            })
            .sum();

        Self {
            max_achievements: rows(data, "AchievementData"),
            max_nodes: node_zones.len(),
            max_zones,
            max_tech_nodes: rows(data, "DevelopmentData"),
            max_quests: rows(data, "ArchiveQuestData"),
            max_stages: rows(data, "StageData"),
            max_recipes: rows(data, "CookbookData"),
            max_music: rows(data, "ArchiveMusicUnlockData"),
            max_base_level: rows(data, "BasementUpdateDatas"),
            max_blueprints: 0,
            max_rifts: 0,
            max_band_levels,
            node_zones,
            topics: Vec::new(),
        }
    }

    /// Explorable map nodes for a single playthrough
    fn count_map_nodes(data: &Value) -> usize {
        let Some(map_data) = data.get("MapData").and_then(Value::as_array) else {
            return 0;
        };

        let node_count = |entry: &Value| -> usize {
            entry
                .get("value")
                .and_then(|v| v.get("Nodes"))
                .and_then(Value::as_array)
                .map_or(0, Vec::len)
        };
        let key_of = |entry: &Value| -> String {
            entry
                .get("key")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };

        let overworld: usize = map_data
            .iter()
            .filter(|e| key_of(e).contains("_main"))
            .map(&node_count)
            .sum();
        if overworld > 0 {
            return overworld;
        }

        let mut per_group: HashMap<String, usize> = HashMap::new();
        for entry in map_data {
            let group = Self::base_layout_group(&key_of(entry));
            let slot = per_group.entry(group).or_insert(0);
            *slot = (*slot).max(node_count(entry));
        }
        per_group.values().sum()
    }

    /// Strip a trailing `_<digits>` variant suffix, e.g. `sandbox_1_hunt_normal_7` -> `sandbox_1_hunt_normal`
    fn base_layout_group(key: &str) -> String {
        match key.rsplit_once('_') {
            Some((prefix, suffix))
                if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) =>
            {
                prefix.to_string()
            }
            _ => key.to_string(),
        }
    }
}

fn rows(data: &Value, key: &str) -> usize {
    data.get(key).and_then(Value::as_array).map_or(0, Vec::len)
}

fn keyed_entries(v: Option<&Value>) -> Vec<(&str, &Value)> {
    match v {
        Some(Value::Array(rows)) => rows
            .iter()
            .filter_map(|row| Some((row.get("key")?.as_str()?, row.get("value")?)))
            .collect(),
        Some(Value::Object(map)) => map.iter().map(|(k, v)| (k.as_str(), v)).collect(),
        _ => Vec::new(),
    }
}
