//! Roguelike (Integrated Strategies) game data types
//!
//! Contains types for parsing roguelike_topic_table.json and storing
//! processed data for use in user scoring calculations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Processed roguelike data for efficient lookups during scoring
#[derive(Debug, Clone, Default)]
pub struct RoguelikeGameData {
    /// Per-theme data indexed by theme_id (e.g., "rogue_1", "rogue_2", etc.)
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
    /// Total relics available
    pub max_relics: i32,
    /// Total capsules available (only rogue_1 has these)
    pub max_capsules: i32,
    /// Total bands available
    pub max_bands: i32,
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

/// Root structure for roguelike_topic_table.json parsing
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RoguelikeTopicTableFile {
    pub topics: HashMap<String, RoguelikeTopicEntry>,
    pub details: HashMap<String, RoguelikeDetailEntry>,
}

/// Topic entry (theme metadata)
#[derive(Debug, Clone, Deserialize)]
pub struct RoguelikeTopicEntry {
    pub id: String,
    pub name: String,
}

/// Detail entry containing collectibles and challenges
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoguelikeDetailEntry {
    #[serde(default)]
    pub endings: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub challenges: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub difficulties: Option<Vec<RoguelikeDifficultyEntry>>,
    #[serde(default)]
    pub milestones: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub month_squad: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub archive_comp: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub band_ref: Option<HashMap<String, serde_json::Value>>,
}

/// Difficulty entry for extracting max grade
#[derive(Debug, Clone, Deserialize)]
pub struct RoguelikeDifficultyEntry {
    #[serde(default)]
    pub grade: i32,
}

impl RoguelikeGameData {
    /// Parse from roguelike_topic_table.json
    /// Uses archiveComp for collectible counts (eg. max collectible count)
    pub fn from_table(table: &RoguelikeTopicTableFile) -> Self {
        let mut data = RoguelikeGameData::default();

        for (theme_id, detail) in &table.details {
            let topic = table.topics.get(theme_id);
            let theme_name = topic.map_or(theme_id.clone(), |t| t.name.clone());

            let max_endings = detail.endings.as_ref().map_or(0, |e| e.len() as i32);

            let max_challenges = detail.challenges.as_ref().map_or(0, |c| c.len() as i32);

            let max_difficulty_grade = detail
                .difficulties
                .as_ref()
                .map_or(0, |diffs| diffs.iter().map(|d| d.grade).max().unwrap_or(0));

            let max_bp_levels = detail.milestones.as_ref().map_or(0, |m| m.len() as i32);

            let max_monthly_squads = detail.month_squad.as_ref().map_or(0, |s| s.len() as i32);

            let (
                max_relics,
                max_capsules,
                max_bands,
                max_theme_collectibles,
                max_endbook_items,
                max_buffs,
            ) = parse_archive_comp(detail);

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

/// Extracts collectible max counts from archiveComp.
///
/// archiveComp structure: { "relic": {"relic": {id: ...}}, "capsule": {"capsule": {...}}, ... }
/// Each category has an inner key (usually same name) containing a dict of items.
///
/// Categories we care about:
/// - relic, capsule: standard collectibles (separate fields)
/// - band: tracked via detail.bandRef, but user data has bands in collect.band
/// - endbook: CG scene unlocks
/// - buff: permanent buff purchases
/// - totem, chaos, fragment, disaster, wrath, copper: theme-specific → summed into max_theme_collectibles
fn parse_archive_comp(detail: &RoguelikeDetailEntry) -> (i32, i32, i32, i32, i32, i32) {
    let archive = match &detail.archive_comp {
        Some(a) => a,
        None => return (0, 0, 0, 0, 0, 0),
    };

    let count_inner = |category: &str| -> i32 {
        archive
            .get(category)
            .and_then(|v| v.as_object())
            .map_or(0, |obj| {
                // The inner dict has one or more keys; count items in the
                // first dict-valued key (the collectible items dict)
                obj.values()
                    .filter_map(|v| v.as_object())
                    .map(|inner| inner.len() as i32)
                    .sum()
            })
    };

    let max_relics = count_inner("relic");
    let max_capsules = count_inner("capsule");

    let max_bands = detail.band_ref.as_ref().map_or(0, |b| b.len() as i32);

    let max_endbook_items = count_inner("endbook");
    let max_buffs = count_inner("buff");

    // Theme-specific collectibles: everything in archiveComp that isn't
    // relic, capsule, trap, chat, endbook, buff
    let theme_specific_categories = ["totem", "chaos", "fragment", "disaster", "wrath", "copper"];
    let max_theme_collectibles: i32 = theme_specific_categories
        .iter()
        .map(|cat| count_inner(cat))
        .sum();

    (
        max_relics,
        max_capsules,
        max_bands,
        max_theme_collectibles,
        max_endbook_items,
        max_buffs,
    )
}
