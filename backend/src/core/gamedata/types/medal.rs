//! Medal game data types
//!
//! Contains types for parsing medal_table.json and storing processed medal data
//! for use in user scoring calculations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for medal_table.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTableFile {
    pub medal_list: Vec<MedalDefinition>,
    pub medal_type_data: Vec<MedalTypeEntry>,
}

/// Individual medal definition from game data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalDefinition {
    pub medal_id: String,
    pub medal_name: String,
    pub medal_type: String,
    pub slot_id: i32,
    #[serde(default)]
    pub pre_medal_id_list: Vec<String>,
    pub rarity: String,
    #[serde(default)]
    pub template: String, // Some medals don't have this
    #[serde(default)]
    pub unlock_param: Vec<String>,
    #[serde(default)]
    pub get_method: String, // Some medals don't have this
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub display_time: i64,
    #[serde(default)]
    pub expire_times: Vec<ExpireTime>,
    #[serde(default)]
    pub medal_reward_group: serde_json::Value, // Complex nested structure, use Value for flexibility
    #[serde(default)]
    pub is_hidden: bool,
}

/// Time window for medal availability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExpireTime {
    pub start: i64,
    pub end: i64,
    #[serde(rename = "Type_")]
    pub expire_type: String,
}

/// Medal type/category entry with groups
#[derive(Debug, Clone, Deserialize)]
pub struct MedalTypeEntry {
    pub key: String,
    pub value: MedalTypeData,
}

/// Medal category metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTypeData {
    pub medal_group_id: String,
    pub sort_id: i32,
    pub medal_name: String,
    #[serde(default)]
    pub group_data: Vec<MedalGroup>,
}

/// Medal group (themed set of medals)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalGroup {
    pub group_id: String,
    pub group_name: String,
    pub group_desc: String,
    pub medal_id: Vec<String>,
    pub sort_id: i32,
    pub group_back_color: String,
    pub group_get_time: i64,
    #[serde(default)]
    pub shared_expire_times: Vec<ExpireTime>,
}

/// Processed medal data for efficient lookups during scoring
#[derive(Debug, Clone, Default)]
pub struct MedalData {
    /// All medals indexed by medal_id
    pub medals: HashMap<String, MedalDefinition>,
    /// All medal groups indexed by group_id
    pub groups: HashMap<String, MedalGroup>,
    /// Medals organized by type/category (e.g., "playerMedal" -> vec of medal_ids)
    pub medals_by_type: HashMap<String, Vec<String>>,
    /// Medals organized by rarity (e.g., "T1" -> vec of medal_ids)
    pub medals_by_rarity: HashMap<String, Vec<String>>,
    /// Category display names (e.g., "playerMedal" -> "Records Medal")
    pub category_names: HashMap<String, String>,
}

impl MedalData {
    /// Process raw medal table data into indexed structures
    pub fn from_table(table: MedalTableFile) -> Self {
        let mut data = MedalData::default();

        // Index all medals
        for medal in table.medal_list {
            let medal_id = medal.medal_id.clone();
            let medal_type = medal.medal_type.clone();
            let rarity = medal.rarity.clone();

            // Add to medals_by_type
            data.medals_by_type
                .entry(medal_type)
                .or_default()
                .push(medal_id.clone());

            // Add to medals_by_rarity
            data.medals_by_rarity
                .entry(rarity)
                .or_default()
                .push(medal_id.clone());

            // Add to main index
            data.medals.insert(medal_id, medal);
        }

        // Process type data for category names and groups
        for type_entry in table.medal_type_data {
            // Store category display name
            data.category_names
                .insert(type_entry.key.clone(), type_entry.value.medal_name);

            // Index all groups from this type
            for group in type_entry.value.group_data {
                data.groups.insert(group.group_id.clone(), group);
            }
        }

        data
    }

    /// Get the display name for a medal category
    pub fn get_category_name(&self, category: &str) -> String {
        self.category_names
            .get(category)
            .cloned()
            .unwrap_or_else(|| category.to_string())
    }
}
