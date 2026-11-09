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

#[cfg(test)]
mod tests {
    use super::{SandboxTemplate, SandboxUniverse};
    use serde_json::{Value, json};

    /// First rows of CN's `sandbox_perm_table`, both topics.
    fn cn_slice() -> Value {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/sandbox_perm_cn_slice.json"
        ))
        .expect("fixture parses")
    }

    #[test]
    fn every_basic_info_topic_is_read_under_its_own_template() {
        let u = SandboxUniverse::build(&cn_slice());
        let ids: Vec<(&str, &SandboxTemplate, i64)> = u
            .topics
            .iter()
            .map(|t| (t.topic_id.as_str(), &t.template, t.sort_id))
            .collect();
        assert_eq!(
            ids,
            vec![
                ("sandbox_1", &SandboxTemplate::V2, 1),
                ("sandbox_2", &SandboxTemplate::V3, 2),
            ]
        );

        let v3 = &u.topics[1].counts;
        assert_eq!(v3.max_achievements, 3, "AchievementData rows");
        assert_eq!(v3.max_nodes, 4, "MainMapData.Nodes without HOME");
        assert_eq!(v3.max_zones, 2, "zones of those nodes, not the HOME zone");
        assert_eq!(v3.node_zones["sandbox_2_node_04"], "z_2_2");
        assert_eq!(v3.max_tech_nodes, 4, "DevelopmentData");
        assert_eq!(v3.max_quests, 2, "ArchiveQuestData");
        assert_eq!(v3.max_stages, 2, "StageData");
        assert_eq!(v3.max_recipes, 2, "CookbookData");
        assert_eq!(v3.max_music, 2, "ArchiveMusicUnlockData");
        assert_eq!(v3.max_base_level, 3, "BasementUpdateDatas");
        assert_eq!(v3.max_blueprints, 0, "run-scoped recipes are not graded");
        assert_eq!(v3.max_band_levels, 6, "2 bands x 3 levels");
        assert_eq!(v3.max_rifts, 0, "V3 has no fixed rifts");
        assert_eq!(u.topics[1].name, "重启锚点", "BasicInfo.TopicName");
        assert!(v3.topics.is_empty());
    }

    #[test]
    fn flat_totals_stay_the_first_v2_topic() {
        let raw = cn_slice();
        let u = SandboxUniverse::build(&raw);
        // The same table without its V3 half: what the grader read before.
        let mut v2_only = raw;
        v2_only["Detail"]
            .as_object_mut()
            .expect("Detail")
            .remove("SANDBOX_V3");
        v2_only["BasicInfo"]
            .as_array_mut()
            .expect("BasicInfo")
            .truncate(1);
        let before = SandboxUniverse::build(&v2_only);

        let flat = |x: &SandboxUniverse| SandboxUniverse {
            topics: Vec::new(),
            ..x.clone()
        };
        assert_eq!(flat(&u), flat(&before), "V3 does not move a V2 total");
        assert_eq!(flat(&u), u.topics[0].counts, "flat fields are sandbox_1's");
        assert_eq!(u.max_nodes, 3, "the `_main` map only, not the hunt variant");
        assert_eq!(u.max_zones, 2, "tutorial zone excluded");
    }

    #[test]
    fn a_template_with_no_reader_is_listed_with_zero_totals() {
        let raw = json!({
            "BasicInfo": [{ "key": "sandbox_9", "value": {
                "TopicId": "sandbox_9", "TopicTemplate": "SANDBOX_V4", "SortId": 9,
            } }],
            "Detail": { "SANDBOX_V4": [{ "key": "sandbox_9", "value": { "AchievementData": [{}] } }] },
        });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(u.topics.len(), 1);
        assert_eq!(
            u.topics[0].template,
            SandboxTemplate::Other("SANDBOX_V4".into())
        );
        assert_eq!(u.topics[0].counts, SandboxUniverse::default());
    }

    #[test]
    fn no_basic_info_falls_back_to_the_detail_entries() {
        let raw = json!({ "Detail": {
            "SANDBOX_V3": [{ "key": "sandbox_2", "value": { "AchievementData": [{}, {}] } }],
        } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(u.topics.len(), 1);
        assert_eq!(u.topics[0].topic_id, "sandbox_2");
        assert_eq!(u.topics[0].counts.max_achievements, 2);
    }

    #[test]
    fn max_zones_excludes_the_tutorial_zone() {
        let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
            "ZoneData": [
                { "key": "z_1_0" }, { "key": "z_1_1" }, { "key": "z_1_tutorial" }, { "key": "z_1_2" },
            ],
        } } ] } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(
            u.max_zones, 3,
            "the tutorial zone is excluded from the zone total"
        );
    }

    #[test]
    fn max_blueprints_does_not_double_count_buildings() {
        // `BuildingItemData` ids all reappear in `CraftItemData`; the total is
        // the `CraftItemData` count, never the sum of the two.
        let raw = json!({ "Detail": { "SANDBOX_V2": [ { "value": {
            "BuildingItemData": [
                { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
            ],
            "CraftItemData": [
                { "key": "sandbox_1_building_1" }, { "key": "sandbox_1_building_2" },
                { "key": "sandbox_1_tactical_1" },
            ],
        } } ] } });
        let u = SandboxUniverse::build(&raw);
        assert_eq!(
            u.max_blueprints, 3,
            "blueprints total is the craftable set, not craft + building"
        );
    }
}
