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
                .map(|a| a.len())
                .unwrap_or(0)
        };

        let universe = Self {
            max_achievements: count("AchievementData"),
            max_nodes: Self::count_map_nodes(sandbox_data),
            max_zones: count("ZoneData"),
            max_tech_nodes: count("DevelopmentData"),
            max_quests: count("QuestData"),
            max_stages: count("StageData"),
            max_recipes: count("FoodData"),
            max_music: count("ArchiveMusicUnlockData"),
            max_base_level: count("BaseUpdate"),
            max_blueprints: count("BuildingItemData") + count("CraftItemData"),
            max_rifts: count("FixedRiftData"),
        };

        eprintln!(
            "SandboxUniverse: achievements={}, nodes={}, zones={}, tech={}, quests={}, stages={}, recipes={}, music={}, base_levels={}, blueprints={}, rifts={}",
            universe.max_achievements,
            universe.max_nodes,
            universe.max_zones,
            universe.max_tech_nodes,
            universe.max_quests,
            universe.max_stages,
            universe.max_recipes,
            universe.max_music,
            universe.max_base_level,
            universe.max_blueprints,
            universe.max_rifts,
        );

        universe
    }

    fn extract_sandbox_v2(raw: &Value) -> Option<&Value> {
        let sandbox_v2 = raw.get("Detail")?.get("SANDBOX_V2")?.as_array()?;
        let first = sandbox_v2.first()?;
        first.get("value")
    }

    fn count_map_nodes(sandbox_data: &Value) -> usize {
        let Some(map_data) = sandbox_data.get("MapData").and_then(|v| v.as_array()) else {
            return 0;
        };

        let mut total = 0;
        for entry in map_data {
            let Some(value) = entry.get("value") else {
                continue;
            };
            if let Some(nodes) = value.get("Nodes").and_then(|v| v.as_array()) {
                total += nodes.len();
            }
        }
        total
    }
}
