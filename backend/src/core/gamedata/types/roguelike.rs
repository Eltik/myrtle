//! Roguelike (Integrated Strategies) game data types
//!
//! Contains types for parsing `roguelike_topic_table.json` and storing
//! processed data for use in user scoring calculations.
//!
//! The FlatBuffer-exported JSON uses `PascalCase` keys and `[{key, value}]`
//! arrays for dict types. We parse via `serde_json::Value` and extract manually.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Processed roguelike data for efficient lookups during scoring
#[derive(Debug, Clone, Default)]
pub struct RoguelikeGameData {
    /// Per-theme data indexed by `theme_id` (e.g., "`rogue_1`", "`rogue_2`", etc.)
    pub themes: HashMap<String, RoguelikeThemeGameData>,
}

/// Max available counts for a single roguelike theme
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeThemeGameData {
    pub theme_id: String,
    pub theme_name: String,
    /// Total unique endings available
    pub max_endings: i32,
    /// Total relics available (`relic_ids.len()`)
    pub max_relics: i32,
    /// Total capsules available (only `rogue_1` has these; `capsule_ids.len()`)
    pub max_capsules: i32,
    /// Total bands available (`band_ids.len()`)
    pub max_bands: i32,
    /// Archive relic ids (`ArchiveComp.Relic.Relic`). This is the in-game
    /// collectible archive, and it is NOT the same set as the keys the client
    /// writes to `collect.relic`: the archive lists the tool items
    /// (`*_active_tool_N`, `*_explore_tool_N`) which the client stores in other
    /// buckets, and `collect.relic` carries upgrade variants
    /// (`*_relic_legacy_N_a/_b/_c`) which the archive does not list. Counts must
    /// therefore iterate THIS list and look ids up, never count bucket keys.
    /// See [`RoguelikeThemeGameData::count_collected`].
    pub relic_ids: Vec<String>,
    /// Archive capsule ids (`ArchiveComp.Capsule.Capsule`)
    pub capsule_ids: Vec<String>,
    /// Band item ids: `Items` entries typed `BAND`. `BandRef` is the band
    /// upgrade-ref table, not the band list (empty for `rogue_1`/`rogue_3`, and
    /// only `band_11..22` for `rogue_2`), so it is the wrong denominator.
    pub band_ids: Vec<String>,
    /// Total challenge stages available
    pub max_challenges: i32,
    /// Total monthly squads available
    pub max_monthly_squads: i32,
    /// Highest difficulty grade available
    pub max_difficulty_grade: i32,
    /// Total BP levels (milestones)
    pub max_bp_levels: i32,
    /// Theme-specific collectibles (totem, chaos, disaster, fragment, wrath, copper, etc.)
    /// Total count across all theme-specific archive categories
    pub max_theme_collectibles: i32,
    /// Max endbook CG items
    pub max_endbook_items: i32,
    /// Max buff unlocks
    pub max_buffs: i32,
}

/// Root structure for `roguelike_topic_table.json` (`FlatBuffer` export format)
///
/// Top-level keys: Topics, Details, Modules, Constant, `CustomizeData`
/// Topics and Details are `[{key, value}]` arrays (`PascalCase`).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoguelikeTopicTableFile {
    #[serde(alias = "Topics", alias = "topics", default)]
    pub topics: serde_json::Value,
    #[serde(alias = "Details", alias = "details", default)]
    pub details: serde_json::Value,
}

impl RoguelikeGameData {
    /// Parse from `roguelike_topic_table.json`.
    ///
    /// Handles both formats:
    /// - `FlatBuffer` export: `PascalCase`, `[{key, value}]` arrays
    /// - CN-gamedata: camelCase, `{key: value}` dicts
    pub fn from_table(table: &RoguelikeTopicTableFile) -> Self {
        let mut data = Self::default();

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
                        .and_then(serde_json::Value::as_i64)
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

    /// Get total max endings across all themes
    pub fn total_max_endings(&self) -> i32 {
        self.themes.values().map(|t| t.max_endings).sum()
    }

    /// Get total max collectibles (relics + capsules + bands) across all themes
    pub fn total_max_collectibles(&self) -> i32 {
        self.themes
            .values()
            .map(|t| t.max_relics + t.max_capsules + t.max_bands)
            .sum()
    }

    /// Get total max challenges across all themes
    pub fn total_max_challenges(&self) -> i32 {
        self.themes.values().map(|t| t.max_challenges).sum()
    }

    /// Get total max monthly squads across all themes
    pub fn total_max_monthly_squads(&self) -> i32 {
        self.themes.values().map(|t| t.max_monthly_squads).sum()
    }

    /// Get number of themes
    pub fn theme_count(&self) -> i32 {
        self.themes.len() as i32
    }
}

/// Collected counts for one theme, measured against the archive id lists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CollectedCounts {
    pub relics: usize,
    pub capsules: usize,
    pub bands: usize,
}

impl RoguelikeThemeGameData {
    /// Count how many archive relics/capsules/bands a player's theme progress
    /// has unlocked (`state >= 1`).
    ///
    /// Each count iterates the archive id list and looks the id up, so an id
    /// that lives outside the bucket named after it is still found and an id
    /// in the bucket that the archive does not list is never counted. Measured
    /// on 2,630 real accounts (2026-09-17), counting `collect.relic` keys
    /// instead gave 256/262 as the ceiling on `rogue_2` (the six active tools
    /// are in `collect.activeTool`) and 333/294 on `rogue_5` (45 `_a/_b/_c`
    /// upgrade variants), which the old `.min(max)` cap then hid as "complete"
    /// for 1,441 accounts.
    ///
    /// Relic ids are read from `collect.relic`, then `collect.activeTool`, then
    /// `challenge.collect.exploreTool` (`rogue_3` only). Every bucket has the
    /// same `{id: {state, progress}}` shape.
    pub fn count_collected(&self, progress: &serde_json::Value) -> CollectedCounts {
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

/// Number of `ids` whose entry in any of `buckets` has `state >= 1`.
fn count_in_buckets(ids: &[String], buckets: &[Option<&serde_json::Value>]) -> usize {
    ids.iter()
        .filter(|id| {
            buckets.iter().flatten().any(|bucket| {
                bucket
                    .get(id.as_str())
                    .and_then(|e| e.get("state"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0)
                    >= 1
            })
        })
        .count()
}

/// Ids of `Items` entries typed `BAND`.
///
/// `FlatBuffer` export: `Items: [{key, value: {Type_: "BAND", ...}}]` (the
/// exporter renames `type` to `Type_`). CN-gamedata: `items: {id: {type: "BAND"}}`.
fn band_item_ids(detail: &serde_json::Value) -> Vec<String> {
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

/// Extracts collectible ids and max counts from `ArchiveComp`.
///
/// `FlatBuffer` format: `{ "Relic": { "Relic": [{key, value}, ...] }, ... }`
/// CN-gamedata format: `{ "relic": { "relic": { id: ... } }, ... }`
///
/// Returns `(relic_ids, capsule_ids, max_theme_collectibles, max_endbook_items, max_buffs)`.
fn parse_archive_comp(detail: &serde_json::Value) -> (Vec<String>, Vec<String>, i32, i32, i32) {
    let Some(archive) = detail
        .get("ArchiveComp")
        .or_else(|| detail.get("archiveComp"))
    else {
        return (Vec::new(), Vec::new(), 0, 0, 0);
    };

    // Ids of every inner entry of a category, e.g. `Relic.Relic[*].key`.
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
            // Each inner key maps to either a dict (CN) or a [{key,value}] array (FBS)
            obj.values()
                .map(|v| {
                    if let Some(arr) = v.as_array() {
                        // FBS format: [{key, value}, ...]
                        arr.len() as i32
                    } else if let Some(inner_obj) = v.as_object() {
                        // CN format: {id: ...}
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

// ── JSON helpers ───────────────────────────────────────────────

/// Convert a `[{key, value}]` array or `{key: value}` dict to a `HashMap`.
fn kv_to_map(val: &serde_json::Value) -> HashMap<String, serde_json::Value> {
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

/// Get a string field, trying multiple key variants.
fn get_str(val: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
}

/// Count elements in a `[{key, value}]` array or `{key: value}` dict.
fn get_kv_array_len(val: &serde_json::Value, keys: &[&str]) -> i32 {
    let field = keys.iter().find_map(|k| val.get(*k));
    match field {
        Some(v) if v.is_array() => v.as_array().map_or(0, |a| a.len() as i32),
        Some(v) if v.is_object() => v.as_object().map_or(0, |o| o.len() as i32),
        _ => 0,
    }
}

/// Count elements in a plain array.
fn get_array_len(val: &serde_json::Value, keys: &[&str]) -> i32 {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .map_or(0, |a| a.len() as i32)
}

/// Get a plain array.
fn get_array(val: &serde_json::Value, keys: &[&str]) -> Vec<serde_json::Value> {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}
