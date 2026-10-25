//! Integrated Strategies game data. The `FlatBuffer` export uses `PascalCase`
//! keys and `[{key, value}]` arrays for dicts, so it is read as
//! `serde_json::Value` and picked apart by hand.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RoguelikeGameData {
    pub themes: HashMap<String, RoguelikeThemeGameData>,
    /// Themes and their collectible items as tier list entries, see
    /// [`super::roguelike_entries`].
    pub entries: super::roguelike_entries::RoguelikeEntries,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeThemeGameData {
    pub theme_id: String,
    pub theme_name: String,
    pub max_endings: i32,
    pub max_relics: i32,
    /// Only `rogue_1` has capsules.
    pub max_capsules: i32,
    pub max_bands: i32,
    /// Archive relic ids (`ArchiveComp.Relic.Relic`). NOT the key set of
    /// `collect.relic`: the archive lists tool items (`*_active_tool_N`,
    /// `*_explore_tool_N`) the client stores in other buckets, and
    /// `collect.relic` holds upgrade variants (`*_relic_legacy_N_a/_b/_c`) the
    /// archive omits. Count by iterating this list, never bucket keys (see
    /// [`RoguelikeThemeGameData::count_collected`]).
    pub relic_ids: Vec<String>,
    /// Archive capsule ids (`ArchiveComp.Capsule.Capsule`)
    pub capsule_ids: Vec<String>,
    /// Band item ids: `Items` entries typed `BAND`. `BandRef` is the band
    /// upgrade-ref table, not the band list (empty for `rogue_1`/`rogue_3`, and
    /// only `band_11..22` for `rogue_2`), so it is the wrong denominator.
    pub band_ids: Vec<String>,
    pub max_challenges: i32,
    pub max_monthly_squads: i32,
    pub max_difficulty_grade: i32,
    /// Total BP levels (milestones)
    pub max_bp_levels: i32,
    /// Total across the theme-specific archive categories (totem, chaos,
    /// disaster, fragment, wrath, copper, ...).
    pub max_theme_collectibles: i32,
    /// Max endbook CG items
    pub max_endbook_items: i32,
    pub max_buffs: i32,
    /// Endings `Endings` lists outside the regular runs. `max_endings` still
    /// counts them (it is the array length, which grading divides by), so a
    /// theme with one reads one higher than the in-game tally of regular
    /// endings. Empty for every theme but `rogue_3` on EN (2026-10-25); see
    /// [`special_endings`].
    pub special_endings: Vec<SpecialEnding>,
}

/// An ending the game data lists that a regular run does not reach.
///
/// Carried so the UI can explain a count instead of the backend hiding one:
/// `rogue_3` (Expeditioner's Jǫklumarkar) lists 5 endings, 4 regular plus
/// `ro3_ending_c` "The Fleetest of Light", and players count 4 in game.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialEnding {
    pub id: String,
    pub name: String,
    /// The theme's challenge mode name, e.g. "Deep Investigation"; `None`
    /// when the theme lists no `CHALLENGE` difficulty.
    pub mode_name: Option<String>,
}

/// `roguelike_topic_table.json` (`FlatBuffer` export). Top-level keys: Topics,
/// Details, Modules, Constant, `CustomizeData`; Topics and Details are
/// `[{key, value}]` arrays.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoguelikeTopicTableFile {
    #[serde(alias = "Topics", alias = "topics", default)]
    pub topics: serde_json::Value,
    #[serde(alias = "Details", alias = "details", default)]
    pub details: serde_json::Value,
}

impl RoguelikeGameData {
    /// Reads either format:
    /// - `FlatBuffer` export: `PascalCase`, `[{key, value}]` arrays
    /// - CN-gamedata: camelCase, `{key: value}` dicts
    pub fn from_table(table: &RoguelikeTopicTableFile) -> Self {
        let mut data = Self {
            entries: super::roguelike_entries::RoguelikeEntries::from_tables(
                &table.topics,
                &table.details,
            ),
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
            let special_endings = special_endings(detail);
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
                    special_endings,
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

/// Endings whose `Priority` sits outside the 0..n-1 ranking regular endings share.
///
/// Measured on the EN table (2026-10-25): the 23 regular endings of all five
/// themes carry `Priority` 0..n-1 in display order, and the one extra ending,
/// `ro3_ending_c`, carries 99 with n = 5. It is also the only ending with one
/// untiered icon (Level 0) where the rest of `rogue_3` carry four (Levels
/// 0/4/10/12). A rule on the icon count would misfire on `rogue_1`/`rogue_2`,
/// whose regular endings carry one icon each, so the rank is the signal.
///
/// Nothing in the table names the mode an ending belongs to. The mode name is
/// the theme's `CHALLENGE` difficulty ("Deep Investigation" on `rogue_3`): a
/// HYPOTHESIS resting on the `_c` id suffix, the untiered icon (that mode has
/// no difficulty grades) and what players report, not on a field.
fn special_endings(detail: &serde_json::Value) -> Vec<SpecialEnding> {
    let endings = detail
        .get("Endings")
        .or_else(|| detail.get("endings"))
        .map(kv_to_map)
        .unwrap_or_default();
    let count = endings.len() as i64;
    let mode_name = get_array(detail, &["Difficulties", "difficulties"])
        .iter()
        .find(|d| {
            get_str(d, &["ModeDifficulty", "modeDifficulty", "mode"]).as_deref()
                == Some("CHALLENGE")
        })
        .and_then(|d| get_str(d, &["Name", "name"]));

    let mut special: Vec<SpecialEnding> = endings
        .iter()
        .filter(|(_, e)| {
            // A missing rank is read as regular: no rank, no evidence.
            e.get("Priority")
                .or_else(|| e.get("priority"))
                .and_then(serde_json::Value::as_i64)
                .is_some_and(|p| p >= count)
        })
        .map(|(id, e)| SpecialEnding {
            id: id.clone(),
            name: get_str(e, &["Name", "name"]).unwrap_or_else(|| id.clone()),
            mode_name: mode_name.clone(),
        })
        .collect();
    special.sort_by(|a, b| a.id.cmp(&b.id));
    special
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

fn get_array_len(val: &serde_json::Value, keys: &[&str]) -> i32 {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .map_or(0, |a| a.len() as i32)
}

fn get_array(val: &serde_json::Value, keys: &[&str]) -> Vec<serde_json::Value> {
    keys.iter()
        .find_map(|k| val.get(*k))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn detail(endings: serde_json::Value) -> serde_json::Value {
        json!({
            "Endings": endings,
            "Difficulties": [
                { "Name": "Regular", "ModeDifficulty": "NORMAL" },
                { "Name": "Deep Investigation", "ModeDifficulty": "CHALLENGE" },
            ],
        })
    }

    fn ending(id: &str, name: &str, priority: i64) -> serde_json::Value {
        json!({ "key": id, "value": { "Id": id, "Name": name, "Priority": priority } })
    }

    #[test]
    fn an_ending_ranked_past_the_list_is_special() {
        let d = detail(json!([
            ending("e1", "One", 0),
            ending("e2", "Two", 1),
            ending("e_c", "Extra", 99),
            ending("e3", "Three", 2),
        ]));
        assert_eq!(
            special_endings(&d),
            vec![SpecialEnding {
                id: "e_c".into(),
                name: "Extra".into(),
                mode_name: Some("Deep Investigation".into()),
            }]
        );
    }

    #[test]
    fn a_contiguous_ranking_has_no_special_ending() {
        let d = detail(json!([
            ending("e1", "One", 0),
            ending("e2", "Two", 1),
            ending("e3", "Three", 2)
        ]));
        assert!(special_endings(&d).is_empty());
    }

    #[test]
    fn a_missing_rank_reads_as_regular_and_cn_dicts_parse() {
        // CN-gamedata shape: camelCase keys and `{id: ending}` dicts, no difficulties.
        let d = json!({ "endings": {
            "e1": { "name": "One", "priority": 0 },
            "e2": { "name": "Two" },
            "e3": { "name": "Extra", "priority": 7 },
        }});
        assert_eq!(
            special_endings(&d),
            vec![SpecialEnding {
                id: "e3".into(),
                name: "Extra".into(),
                mode_name: None
            }]
        );
    }

    /// The EN table: `GAME_DATA_DIR`, else `../assets/output/en/gamedata/excel`.
    fn load_en() -> Option<RoguelikeGameData> {
        let dir = std::env::var("GAME_DATA_DIR").map_or_else(
            |_| {
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../assets/output/en/gamedata/excel")
            },
            std::path::PathBuf::from,
        );
        if !dir.join("roguelike_topic_table.json").exists() {
            return None;
        }
        let file: RoguelikeTopicTableFile =
            crate::core::gamedata::tables::load_table(&dir, "roguelike_topic_table").ok()?;
        Some(RoguelikeGameData::from_table(&file))
    }

    #[test]
    fn en_lists_one_special_ending_on_rogue_3_and_keeps_every_count() {
        let Some(data) = load_en() else {
            eprintln!("skipping: EN roguelike_topic_table.json not present");
            return;
        };
        let rogue_3 = &data.themes["rogue_3"];
        assert_eq!(
            rogue_3.special_endings,
            vec![SpecialEnding {
                id: "ro3_ending_c".into(),
                name: "The Fleetest of Light".into(),
                mode_name: Some("Deep Investigation".into()),
            }]
        );
        // The count is the array length either way: 4 regular + 1 special.
        assert_eq!(rogue_3.max_endings, 5);
        // `rogue_4`/`rogue_5` list five REGULAR endings each (Priority 0..4),
        // so they get no note.
        for (theme_id, max) in [
            ("rogue_1", 4),
            ("rogue_2", 4),
            ("rogue_4", 5),
            ("rogue_5", 5),
        ] {
            let theme = &data.themes[theme_id];
            assert!(
                theme.special_endings.is_empty(),
                "{theme_id}: {:?}",
                theme.special_endings
            );
            assert_eq!(theme.max_endings, max, "{theme_id}");
        }
    }
}
