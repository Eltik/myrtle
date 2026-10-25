use serde_json::Value;

/// Content totals of the Reclamation Algorithm, the denominators of the RA grade.
///
/// The flat `max_*` fields are the first `SANDBOX_V2` topic's (`sandbox_1`,
/// "Tales Within the Sand"), read exactly as before topics existed, so the V2
/// grade does not move. `topics` holds every topic `BasicInfo` lists, each with
/// its own totals, dispatched on `TopicTemplate`: CN lists 2 (`sandbox_1` on
/// `SANDBOX_V2`, `sandbox_2` "Relaunch Anchor" on `SANDBOX_V3`), EN lists 1
/// until the 2026-10-28 update.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    /// Every topic in `BasicInfo`, in `SortId` order. Empty inside a topic's
    /// own `counts`.
    pub topics: Vec<SandboxTopicUniverse>,
}

/// The `TopicTemplate` a topic's `Detail` entry lives under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SandboxTemplate {
    V2,
    V3,
    /// A template no reader exists for yet; its totals stay 0 and it is not graded.
    Other(String),
}

impl SandboxTemplate {
    fn from_name(name: &str) -> Self {
        match name {
            "SANDBOX_V2" => Self::V2,
            "SANDBOX_V3" => Self::V3,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The key under `Detail` in the table and under `template` in a
    /// player's `sandboxPerm`.
    pub fn key(&self) -> &str {
        match self {
            Self::V2 => "SANDBOX_V2",
            Self::V3 => "SANDBOX_V3",
            Self::Other(name) => name,
        }
    }
}

/// One Reclamation Algorithm topic and its own content totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxTopicUniverse {
    pub topic_id: String,
    pub template: SandboxTemplate,
    pub sort_id: i64,
    pub counts: SandboxUniverse,
}

impl SandboxUniverse {
    pub fn build(raw: &Value) -> Self {
        let topics = Self::build_topics(raw);
        let Some(sandbox_data) = Self::extract_sandbox_v2(raw) else {
            eprintln!("SandboxUniverse: no SANDBOX_V2 data found");
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

    /// Totals of one `SANDBOX_V2` topic's `Detail` value.
    fn count_v2(sandbox_data: &Value) -> Self {
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
            // `BuildingItemData` is a strict SUBSET of `CraftItemData`, not a
            // disjoint table: every one of its 43 ids reappears in
            // `CraftItemData` as a `BASE_BUILDING` (19) or `COMBAT_BUILDING`
            // (24) entry, alongside 53 `TACTICAL` ones. Summing the two counted
            // every building blueprint twice and put the total at 139 when the
            // real universe is 96, the exact key set of `build.book` in the
            // progress of players who have collected them all.
            max_blueprints: count("CraftItemData"),
            max_rifts: count("FixedRiftData"),
            topics: Vec::new(),
        }
    }

    /// Totals of one `SANDBOX_V3` topic's `Detail` value. Counts on CN
    /// `sandbox_2` (extract of 2026-10-06) in brackets.
    ///
    /// Each V3 table below is the one whose rows play the role the V2 table
    /// played, matched by shape, not yet by a player's progress: no V3
    /// `sandboxPerm` blob has been seen (the only dump is EN and the DB backups
    /// hold no CN account), so which of these a player's progress counts
    /// against is open until the first EN sync after 2026-10-28.
    fn count_v3(sandbox_data: &Value) -> Self {
        let count = |key: &str| -> usize {
            sandbox_data
                .get(key)
                .and_then(|v| v.as_array())
                .map_or(0, std::vec::Vec::len)
        };
        // V3 has one fixed overworld, `MainMapData`, an object with `Nodes`
        // [30: 25 STORY, 4 EXPLORE, 1 HOME] and `Zones` [6, `z_2_0` the home
        // block through `z_2_5`], where V2 had `MapData` variants and `ZoneData`.
        // OPEN: both totals include the HOME node and the `z_2_0` home zone.
        // V2 counts its home zone `z_1_0` too and drops only the tutorial, so
        // this matches V2's rule; if a fresh V3 save starts with both unlocked
        // it gets 1/30 nodes and 1/6 zones for free. A logged V3 shape with
        // per-node `state` settles it.
        let main_map = |key: &str| -> usize {
            sandbox_data
                .get("MainMapData")
                .and_then(|m| m.get(key))
                .and_then(|v| v.as_array())
                .map_or(0, std::vec::Vec::len)
        };

        Self {
            max_achievements: count("AchievementData"), // [59]
            max_nodes: main_map("Nodes"),               // [30]
            max_zones: main_map("Zones"),               // [6]
            max_tech_nodes: count("DevelopmentData"),   // [57]
            max_quests: count("ArchiveQuestData"),      // [7]
            max_stages: count("StageData"),             // [29]
            // `CookbookData` [21] replaces V2's `FoodData`.
            max_recipes: count("CookbookData"),
            max_music: count("ArchiveMusicUnlockData"), // [9]
            // `BasementUpdateDatas` [12 rows, `BaseLevel` 1..12] replaces V2's
            // `BaseUpdate`.
            max_base_level: count("BasementUpdateDatas"),
            // `BuildRecipeData` [63] replaces V2's `CraftItemData`.
            max_blueprints: count("BuildRecipeData"),
            // V3 has no `FixedRiftData`. `ExploreStageData` [4] is the nearest
            // repeatable-stage table, but nothing says a player's progress
            // records clears of it, so the rift piece is left out of the V3
            // grade (0 here) rather than guessed.
            max_rifts: 0,
            topics: Vec::new(),
        }
    }

    /// Every topic `BasicInfo` lists, with the totals of its `Detail` entry
    /// under its `TopicTemplate`. A table with no `BasicInfo` falls back to
    /// the `Detail` entries themselves, in table order.
    fn build_topics(raw: &Value) -> Vec<SandboxTopicUniverse> {
        let Some(detail) = raw.get("Detail") else {
            return Vec::new();
        };

        let mut declared: Vec<(String, String, i64)> = keyed_entries(raw.get("BasicInfo"))
            .into_iter()
            .map(|(key, info)| {
                let topic_id = info
                    .get("TopicId")
                    .and_then(Value::as_str)
                    .unwrap_or(key)
                    .to_owned();
                let template = info
                    .get("TopicTemplate")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let sort_id = info
                    .get("SortId")
                    .and_then(Value::as_i64)
                    .unwrap_or(i64::MAX);
                (topic_id, template, sort_id)
            })
            .collect();
        if declared.is_empty() {
            let mut position = 0;
            for (template, entries) in detail.as_object().into_iter().flatten() {
                for (key, _) in keyed_entries(Some(entries)) {
                    declared.push((key.to_owned(), template.clone(), position));
                    position += 1;
                }
            }
        }

        let mut topics: Vec<SandboxTopicUniverse> = declared
            .into_iter()
            .map(|(topic_id, template_name, sort_id)| {
                let template = SandboxTemplate::from_name(&template_name);
                let data = keyed_entries(detail.get(&template_name))
                    .into_iter()
                    .find(|(key, _)| *key == topic_id)
                    .map(|(_, value)| value);
                let counts = match (&template, data) {
                    (SandboxTemplate::V2, Some(d)) => Self::count_v2(d),
                    (SandboxTemplate::V3, Some(d)) => Self::count_v3(d),
                    _ => {
                        eprintln!(
                            "SandboxUniverse: topic {topic_id} ({template_name}) has no reader \
                             or no Detail entry; its totals are 0 and it is not graded"
                        );
                        Self::default()
                    }
                };
                SandboxTopicUniverse {
                    topic_id,
                    template,
                    sort_id,
                    counts,
                }
            })
            .collect();
        topics.sort_by_key(|t| t.sort_id);
        topics
    }

    fn extract_sandbox_v2(raw: &Value) -> Option<&Value> {
        let sandbox_v2 = raw.get("Detail")?.get("SANDBOX_V2")?.as_array()?;
        let first = sandbox_v2.first()?;
        first.get("value")
    }

    /// Explorable map nodes for a single playthrough.
    ///
    /// `MapData` holds the persistent overworld map(s) (`sandbox_1_main`, ≈127
    /// nodes, mirrored in progress as `main.map.node`) plus dozens of randomized
    /// hunt/rift layout variants (`sandbox_1_hunt_normal_1` .. `_10`, ...), only
    /// one of which is ever loaded. Summing them all gave ≈1032 over 59 variants,
    /// the bogus ">1,000" "Map nodes" total that left the exploration sub-score
    /// unfillable and deflated the whole RA grade.
    ///
    /// Counts the `_main` maps, since exploration keys off `main.map.node`. With
    /// none (a shape change), falls back to one variant per layout group so it
    /// never regresses to the over-count.
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
    /// same region collapse to one group. `sandbox_1_hunt_normal_7` ->
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

/// `[{key, value}]` rows or a plain object, as `(key, value)` pairs in order.
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
            "../../../../tests/fixtures/sandbox_perm_cn_slice.json"
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
        assert_eq!(v3.max_nodes, 4, "MainMapData.Nodes");
        assert_eq!(v3.max_zones, 2, "MainMapData.Zones");
        assert_eq!(v3.max_tech_nodes, 4, "DevelopmentData");
        assert_eq!(v3.max_quests, 2, "ArchiveQuestData");
        assert_eq!(v3.max_stages, 2, "StageData");
        assert_eq!(v3.max_recipes, 2, "CookbookData");
        assert_eq!(v3.max_music, 2, "ArchiveMusicUnlockData");
        assert_eq!(v3.max_base_level, 3, "BasementUpdateDatas");
        assert_eq!(v3.max_blueprints, 2, "BuildRecipeData");
        assert_eq!(v3.max_rifts, 0, "V3 has no fixed rifts");
        assert!(v3.topics.is_empty());
    }

    #[test]
    fn flat_totals_stay_the_first_v2_topic() {
        let raw = cn_slice();
        let u = SandboxUniverse::build(&raw);
        // The same table without its V3 half: what the grader read before.
        let mut v2_only = raw.clone();
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
