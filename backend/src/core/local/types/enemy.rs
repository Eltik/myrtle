use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum EnemyLevel {
    Normal,
    Elite,
    Boss,
}

impl Default for EnemyLevel {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DamageType {
    Physic,
    Magic,
    NoDamage,
}

impl Default for DamageType {
    fn default() -> Self {
        Self::Physic
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyInfoList {
    pub class_level: String,
    pub attack: StatRange,
    pub def: StatRange,
    pub magic_res: StatRange,
    #[serde(rename = "maxHP")]
    pub max_hp: StatRange,
    pub move_speed: StatRange,
    pub attack_speed: StatRange,
    pub enemy_damage_res: StatRange,
    pub enemy_res: StatRange,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceData {
    pub id: String,
    pub race_name: String,
    pub sort_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityInfo {
    pub text: String,
    pub text_format: String,
}

// ============================================================================
// Enemy
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enemy {
    pub enemy_id: String,
    pub enemy_index: String,
    pub enemy_tags: Option<Vec<String>>,
    pub sort_id: i32,
    pub name: String,
    pub enemy_level: EnemyLevel,
    pub description: String,
    pub attack_type: Option<String>,
    pub ability: Option<String>,
    pub is_invalid_killed: bool,
    pub override_kill_cnt_infos: Option<serde_json::Value>, // unknown type
    pub hide_in_handbook: bool,
    pub hide_in_stage: bool,
    pub ability_list: Vec<AbilityInfo>,
    pub link_enemies: Vec<String>,
    pub damage_type: Vec<DamageType>,
    pub invisible_detail: bool,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyHandbook {
    pub level_info_list: Vec<EnemyInfoList>,
    pub enemy_data: HashMap<String, Enemy>,
    pub race_data: HashMap<String, RaceData>,
}
