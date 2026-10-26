use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Clone, Default)]
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
}

impl SandboxUniverse {
    pub fn build(raw: &Value) -> Self {
        let Some(sandbox_data) = Self::extract_sandbox_v2(raw) else {
            eprintln!("SandboxUniverse: no SANDBOX_V2 data found");
            return Self::default();
        };

        let count = |key: &str| -> usize {
            sandbox_data
                .get(key)
                .and_then(|v| v.as_array())
                .map_or(0, std::vec::Vec::len)
        };

        let zone_count = sandbox_data
            .get("ZoneData")
            .and_then(|v| v.as_array())
            .map_or(0, |arr| {
                arr.iter()
                    .filter(|e| {
                        e.get("key")
                            .and_then(serde_json::Value::as_str)
                            .is_none_or(|k| !k.contains("tutorial"))
                    })
                    .count()
            });

        Self {
            max_achievements: count("AchievementData"),
            max_nodes: Self::count_map_nodes(sandbox_data),
            max_zones: zone_count,
            max_tech_nodes: count("DevelopmentData"),
            max_quests: count("ArchiveQuestData"),
            max_stages: count("StageData"),
            max_recipes: count("FoodData"),
            max_music: count("ArchiveMusicUnlockData"),
            max_base_level: count("BaseUpdate"),
            max_blueprints: count("CraftItemData"),
            max_rifts: count("FixedRiftData"),
        }
    }

    fn extract_sandbox_v2(raw: &Value) -> Option<&Value> {
        let sandbox_v2 = raw.get("Detail")?.get("SANDBOX_V2")?.as_array()?;
        let first = sandbox_v2.first()?;
        first.get("value")
    }

    /// Explorable map nodes for a single playthrough
    fn count_map_nodes(sandbox_data: &Value) -> usize {
        let Some(map_data) = sandbox_data.get("MapData").and_then(|v| v.as_array()) else {
            return 0;
        };

        let node_count = |entry: &Value| -> usize {
            entry
                .get("value")
                .and_then(|v| v.get("Nodes"))
                .and_then(|v| v.as_array())
                .map_or(0, std::vec::Vec::len)
        };
        let key_of = |entry: &Value| -> String {
            entry
                .get("key")
                .and_then(|v| v.as_str())
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
