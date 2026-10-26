#[derive(Debug, Default)]
pub struct RoguelikeTables {}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::roguelike_entries::RoguelikeEntries;

#[derive(Debug, Clone, Default)]
pub struct RoguelikeGameData {
    pub themes: HashMap<String, RoguelikeThemeGameData>,
    /// Themes and their collectible items as tier list entries
    pub entries: RoguelikeEntries,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeThemeGameData {
    pub theme_id: String,
    pub theme_name: String,
    pub max_endings: i32,
    pub max_relics: i32,
    /// Only `rogue_1` has capsules
    pub max_capsules: i32,
    pub max_bands: i32,
    /// Archive relic ids (`ArchiveComp.Relic.Relic`), not the key set of `collect.relic`
    pub relic_ids: Vec<String>,
    /// Archive capsule ids (`ArchiveComp.Capsule.Capsule`)
    pub capsule_ids: Vec<String>,
    /// Band item ids: `Items` entries typed `BAND`
    pub band_ids: Vec<String>,
    pub max_challenges: i32,
    pub max_monthly_squads: i32,
    pub max_difficulty_grade: i32,
    /// Total BP levels (milestones)
    pub max_bp_levels: i32,
    /// Total across theme-specific archive categories (totem, chaos,
    /// disaster, fragment, wrath, copper, etc.)
    pub max_theme_collectibles: i32,
    /// Max endbook CG items
    pub max_endbook_items: i32,
    pub max_buffs: i32,
}

/// `roguelike_topic_table.json`
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoguelikeTopicTableFile {
    #[serde(alias = "Topics", alias = "topics", default)]
    pub topics: Value,
    #[serde(alias = "Details", alias = "details", default)]
    pub details: Value,
}

impl RoguelikeGameData {
    pub fn from_table(table: &RoguelikeTopicTableFile) -> Self {
        let mut data = Self {
            entries: RoguelikeEntries::from_tables(&table.topics, &table.details),
            ..Self::default()
        };

        let topics = kv_to_map(&table.topics);
        let details = kv_to_map(&table.details);

        for (theme_id, detail) in &details {
            let theme_name = topics
                .get(theme_id)
                .and_then(|t| get_str(t, &["Name", "name"]))
                .unwrap_or_else(|| theme_id.clone());

            let max_endings = get_kv_array_len(detail, &["Endings", "endings"]);
            let max_challenges = get_kv_array_len(detail, &["Challenges", "challenges"]);
            let max_bp_levels = get_array_len(detail, &["Milestones", "milestones"]);
            let max_monthly_squads = get_kv_array_len(detail, &["MonthSquad", "monthSquad"]);
            let band_ids = band_item_ids(detail);
            let max_bands = band_ids.len() as i32;

            let max_difficulty_grade = get_array(detail, &["Difficulties", "difficulties"])
                .iter()
                .filter_map(|d| {
                    d.get("Grade")
                        .or_else(|| d.get("grade"))
                        .and_then(Value::as_i64)
                })
                .max()
                .unwrap_or(0) as i32;

            let (relic_ids, capsule_ids, max_theme_collectibles, max_endbook_items, max_buffs) =
                parse_archive_comp(detail);
            let max_relics = relic_ids.len() as i32;
            let max_capsules = capsule_ids.len() as i32;

            data.themes.insert(
                theme_id.clone(),
                RoguelikeThemeGameData {
                    theme_id: theme_id.clone(),
                    theme_name,
                    max_endings,
                    max_relics,
                    max_capsules,
                    max_bands,
                    max_challenges,
                    max_monthly_squads,
                    max_difficulty_grade,
                    max_bp_levels,
                    max_theme_collectibles,
                    max_endbook_items,
                    max_buffs,
                    relic_ids,
                    capsule_ids,
                    band_ids,
                },
            );
        }

        data
    }

    pub fn total_max_endings(&self) -> i32 {
        self.themes.values().map(|t| t.max_endings).sum()
    }

    pub fn total_max_collectibles(&self) -> i32 {
        self.themes
            .values()
            .map(|t| t.max_relics + t.max_capsules + t.max_bands)
            .sum()
    }

    pub fn total_max_challenges(&self) -> i32 {
        self.themes.values().map(|t| t.max_challenges).sum()
    }

    pub fn total_max_monthly_squads(&self) -> i32 {
        self.themes.values().map(|t| t.max_monthly_squads).sum()
    }

    pub fn theme_count(&self) -> i32 {
        self.themes.len() as i32
    }
}

/// Collected counts for one theme
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CollectedCounts {
    pub relics: usize,
    pub capsules: usize,
    pub bands: usize,
}

impl RoguelikeThemeGameData {
    /// Archive relics/capsules/bands a player's theme progress has unlocked (`state >= 1`)
    pub fn count_collected(&self, progress: &Value) -> CollectedCounts {
        let collect = progress.get("collect");
        let relic_buckets = [
            collect.and_then(|c| c.get("relic")),
            collect.and_then(|c| c.get("activeTool")),
            progress
                .get("challenge")
                .and_then(|c| c.get("collect"))
                .and_then(|c| c.get("exploreTool")),
        ];
        let capsule_buckets = [collect.and_then(|c| c.get("capsule"))];
        let band_buckets = [collect.and_then(|c| c.get("band"))];

        CollectedCounts {
            relics: count_in_buckets(&self.relic_ids, &relic_buckets),
            capsules: count_in_buckets(&self.capsule_ids, &capsule_buckets),
            bands: count_in_buckets(&self.band_ids, &band_buckets),
        }
    }
}

/// Number of `ids` whose entry in any of `buckets` has `state >= 1`
fn count_in_buckets(ids: &[String], buckets: &[Option<&Value>]) -> usize {
    ids.iter()
        .filter(|id| {
            buckets.iter().flatten().any(|bucket| {
                bucket
                    .get(id.as_str())
                    .and_then(|e| e.get("state"))
                    .and_then(Value::as_i64)
                    .unwrap_or(0)
                    >= 1
            })
        })
        .count()
}

fn band_item_ids(detail: &Value) -> Vec<String> {
    let items = detail
        .get("Items")
        .or_else(|| detail.get("items"))
        .map(kv_to_map)
        .unwrap_or_default();
    let mut ids: Vec<String> = items
        .into_iter()
        .filter(|(_, v)| get_str(v, &["Type_", "Type", "type"]).as_deref() == Some("BAND"))
        .map(|(k, _)| k)
        .collect();
    ids.sort();
    ids
}

/// Returns `(relic_ids, capsule_ids, max_theme_collectibles, max_endbook_items, max_buffs)`
fn parse_archive_comp(detail: &Value) -> (Vec<String>, Vec<String>, i32, i32, i32) {
    let Some(archive) = detail
        .get("ArchiveComp")
        .or_else(|| detail.get("archiveComp"))
    else {
        return (Vec::new(), Vec::new(), 0, 0, 0);
    };

    let ids_inner = |category_variants: &[&str]| -> Vec<String> {
        let Some(cat_val) = category_variants.iter().find_map(|c| archive.get(*c)) else {
            return Vec::new();
        };
        let Some(obj) = cat_val.as_object() else {
            return Vec::new();
        };
        let mut ids: Vec<String> = obj
            .values()
            .flat_map(|v| kv_to_map(v).into_keys())
            .collect();
        ids.sort();
        ids
    };

    let count_inner = |category_variants: &[&str]| -> i32 {
        let cat = category_variants.iter().find_map(|c| archive.get(*c));
        let Some(cat_val) = cat else { return 0 };

        if let Some(obj) = cat_val.as_object() {
            obj.values()
                .map(|v| {
                    if let Some(arr) = v.as_array() {
                        arr.len() as i32
                    } else if let Some(inner_obj) = v.as_object() {
                        inner_obj.len() as i32
                    } else {
                        0
                    }
                })
                .sum()
        } else {
            0
        }
    };

    let relic_ids = ids_inner(&["Relic", "relic"]);
    let capsule_ids = ids_inner(&["Capsule", "capsule"]);
    let max_endbook_items = count_inner(&["Endbook", "endbook"]);
    let max_buffs = count_inner(&["Buff", "buff"]);

    let theme_specific = [
        ["Totem", "totem"],
        ["Chaos", "chaos"],
        ["Fragment", "fragment"],
        ["Disaster", "disaster"],
        ["Wrath", "wrath"],
        ["Copper", "copper"],
    ];
    let max_theme_collectibles: i32 = theme_specific
        .iter()
        .map(|variants| count_inner(variants))
        .sum();

    (
        relic_ids,
        capsule_ids,
        max_theme_collectibles,
        max_endbook_items,
        max_buffs,
    )
}

/// `[{key, value}]` array or `{key: value}` dict to a `HashMap`
fn kv_to_map(val: &Value) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    if let Some(arr) = val.as_array() {
        for item in arr {
            if let (Some(k), Some(v)) =
                (item.get("key").and_then(|k| k.as_str()), item.get("value"))
            {
                map.insert(k.to_string(), v.clone());
            }
        }
    } else if let Some(obj) = val.as_object() {
        for (k, v) in obj {
            map.insert(k.clone(), v.clone());
        }
    }
    map
}

fn get_str(val: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
}

/// Count elements in a `[{key, value}]` array or `{key: value}` dict
fn get_kv_array_len(val: &Value, keys: &[&str]) -> i32 {
    let field = keys.iter().find_map(|k| val.get(*k));
    match field {
        Some(v) if v.is_array() => v.as_array().map_or(0, |a| a.len() as i32),
        Some(v) if v.is_object() => v.as_object().map_or(0, |o| o.len() as i32),
        _ => 0,
    }
}

fn get_array_len(val: &Value, keys: &[&str]) -> i32 {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .map_or(0, |a| a.len() as i32)
}

fn get_array(val: &Value, keys: &[&str]) -> Vec<Value> {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}
