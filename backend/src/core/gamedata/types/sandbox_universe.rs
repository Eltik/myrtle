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

        Self {
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
        }
    }

    fn extract_sandbox_v2(raw: &Value) -> Option<&Value> {
        let sandbox_v2 = raw.get("Detail")?.get("SANDBOX_V2")?.as_array()?;
        let first = sandbox_v2.first()?;
        first.get("value")
    }

    /// Count the explorable map nodes for a *single playthrough*.
    ///
    /// `MapData` does NOT hold one node list per region. It holds:
    ///   - the persistent overworld map(s) (`sandbox_1_main`, ≈127 nodes) - the
    ///     map the player actually explores, mirrored in user progress under
    ///     `main.map.node`; and
    ///   - dozens of randomized layout *variants* of instanced hunt/rift
    ///     encounters (`sandbox_1_hunt_normal_1` .. `_10`, etc.), only one of
    ///     which is ever loaded at a time.
    ///
    /// The old code summed Nodes across *every* entry, producing ≈1032 across 59
    /// variants - the bogus ">1,000" total. That figure both showed up as the
    /// "Map nodes" UI total and made the exploration sub-score impossible to
    /// fill (its denominator was 10× too large), deflating the whole RA grade.
    ///
    /// Since the exploration score is keyed off the overworld (`main.map.node`),
    /// we count the overworld map(s) - keys containing `_main`. If none are
    /// present (defensive, e.g. data shape changes), we fall back to summing one
    /// representative variant per layout group so we never regress to the old
    /// over-count.
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

        // Primary: the persistent overworld map(s) the player explores.
        let overworld: usize = map_data
            .iter()
            .filter(|e| key_of(e).contains("_main"))
            .map(&node_count)
            .sum();
        if overworld > 0 {
            return overworld;
        }

        // Fallback: one representative (largest) variant per layout group.
        let mut per_group: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for entry in map_data {
            let group = Self::base_layout_group(&key_of(entry));
            let slot = per_group.entry(group).or_insert(0);
            *slot = (*slot).max(node_count(entry));
        }
        per_group.values().sum()
    }

    /// Strip a trailing `_<digits>` variant suffix so randomized layouts of the
    /// same region collapse to one group. `sandbox_1_hunt_normal_7` →
    /// `sandbox_1_hunt_normal`; ids without a numeric suffix are returned as-is.
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
