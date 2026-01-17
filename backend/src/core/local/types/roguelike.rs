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
    /// Total difficulty levels available
    pub max_difficulties: i32,
    /// Highest difficulty grade available
    pub max_difficulty_grade: i32,
    /// Total BP levels (milestones)
    pub max_bp_levels: i32,
}

/// Root structure for roguelike_topic_table.json parsing
#[derive(Debug, Clone, Deserialize)]
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
    pub relics: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub capsule_dict: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub band_ref: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub challenges: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub difficulties: Option<Vec<RoguelikeDifficultyEntry>>,
    #[serde(default)]
    pub milestones: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub month_squad: Option<HashMap<String, serde_json::Value>>,
}

/// Difficulty entry for extracting max grade
#[derive(Debug, Clone, Deserialize)]
pub struct RoguelikeDifficultyEntry {
    #[serde(default)]
    pub grade: i32,
}

impl RoguelikeGameData {
    /// Create roguelike game data with known max values
    ///
    /// The roguelike JSON files have a complex nested structure that doesn't
    /// easily map to per-theme data. These values are based on game data analysis.
    pub fn with_known_values() -> Self {
        let mut data = RoguelikeGameData::default();

        // rogue_1: Phantom & Crimson Solitaire
        data.themes.insert(
            "rogue_1".to_string(),
            RoguelikeThemeGameData {
                theme_id: "rogue_1".to_string(),
                theme_name: "Phantom & Crimson Solitaire".to_string(),
                max_endings: 4,
                max_relics: 265,
                max_capsules: 26,
                max_bands: 0,
                max_challenges: 12,
                max_monthly_squads: 8,
                max_difficulties: 15,
                max_difficulty_grade: 15,
                max_bp_levels: 120,
            },
        );

        // rogue_2: Mizuki & Caerula Arbor
        data.themes.insert(
            "rogue_2".to_string(),
            RoguelikeThemeGameData {
                theme_id: "rogue_2".to_string(),
                theme_name: "Mizuki & Caerula Arbor".to_string(),
                max_endings: 4,
                max_relics: 294,
                max_capsules: 0,
                max_bands: 12,
                max_challenges: 12,
                max_monthly_squads: 8,
                max_difficulties: 18,
                max_difficulty_grade: 18,
                max_bp_levels: 170,
            },
        );

        // rogue_3: Sami & Expeditioner's Joklumarkar
        data.themes.insert(
            "rogue_3".to_string(),
            RoguelikeThemeGameData {
                theme_id: "rogue_3".to_string(),
                theme_name: "Expeditioner's Joklumarkar".to_string(),
                max_endings: 5,
                max_relics: 380,
                max_capsules: 0,
                max_bands: 0,
                max_challenges: 13,
                max_monthly_squads: 8,
                max_difficulties: 15,
                max_difficulty_grade: 15,
                max_bp_levels: 155,
            },
        );

        // rogue_4: Sarkaz & Ceobe's Fungimist
        data.themes.insert(
            "rogue_4".to_string(),
            RoguelikeThemeGameData {
                theme_id: "rogue_4".to_string(),
                theme_name: "Sarkaz Reclamation".to_string(),
                max_endings: 5,
                max_relics: 431,
                max_capsules: 0,
                max_bands: 55,
                max_challenges: 12,
                max_monthly_squads: 8,
                max_difficulties: 18,
                max_difficulty_grade: 18,
                max_bp_levels: 200,
            },
        );

        // rogue_5: Babel
        data.themes.insert(
            "rogue_5".to_string(),
            RoguelikeThemeGameData {
                theme_id: "rogue_5".to_string(),
                theme_name: "Chronicles of Babel".to_string(),
                max_endings: 4,
                max_relics: 481,
                max_capsules: 0,
                max_bands: 40,
                max_challenges: 0, // No challenge mode in rogue_5
                max_monthly_squads: 7,
                max_difficulties: 15,
                max_difficulty_grade: 15,
                max_bp_levels: 150,
            },
        );

        data
    }

    /// Process raw roguelike table data into indexed structures (legacy, unused)
    #[allow(dead_code)]
    pub fn from_table(_table: RoguelikeTopicTableFile) -> Self {
        // The JSON structure doesn't match what we need, use hardcoded values instead
        Self::with_known_values()
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
