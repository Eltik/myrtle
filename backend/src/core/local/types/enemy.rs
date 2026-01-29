use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[derive(Default)]
pub enum EnemyLevel {
    #[default]
    Normal,
    Elite,
    Boss,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum DamageType {
    #[default]
    Physic,
    Magic,
    NoDamage,
    Heal,
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatRange {
    #[serde(alias = "Min")]
    pub min: f64,
    #[serde(alias = "Max")]
    pub max: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyInfoList {
    #[serde(alias = "ClassLevel")]
    pub class_level: String,
    #[serde(alias = "Attack")]
    pub attack: StatRange,
    #[serde(alias = "Def")]
    pub def: StatRange,
    #[serde(alias = "MagicRes")]
    pub magic_res: StatRange,
    #[serde(alias = "MaxHP")]
    pub max_hp: StatRange,
    #[serde(alias = "MoveSpeed")]
    pub move_speed: StatRange,
    #[serde(alias = "AttackSpeed")]
    pub attack_speed: StatRange,
    #[serde(alias = "EnemyDamageRes")]
    pub enemy_damage_res: StatRange,
    #[serde(alias = "EnemyRes")]
    pub enemy_res: StatRange,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaceData {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "RaceName")]
    pub race_name: String,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityInfo {
    #[serde(alias = "Text")]
    pub text: String,
    #[serde(alias = "TextFormat")]
    pub text_format: String,
}

// ============================================================================
// Enemy
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enemy {
    #[serde(alias = "EnemyId")]
    pub enemy_id: String,
    #[serde(alias = "EnemyIndex")]
    pub enemy_index: String,
    #[serde(alias = "EnemyTags")]
    pub enemy_tags: Option<Vec<String>>,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "EnemyLevel")]
    pub enemy_level: EnemyLevel,
    #[serde(alias = "Description")]
    pub description: String,
    #[serde(alias = "AttackType")]
    pub attack_type: Option<String>,
    #[serde(alias = "Ability")]
    pub ability: Option<String>,
    #[serde(alias = "IsInvalidKilled")]
    pub is_invalid_killed: bool,
    #[serde(alias = "OverrideKillCntInfos")]
    pub override_kill_cnt_infos: Option<serde_json::Value>,
    #[serde(alias = "HideInHandbook")]
    pub hide_in_handbook: bool,
    #[serde(alias = "HideInStage")]
    pub hide_in_stage: bool,
    #[serde(alias = "AbilityList")]
    pub ability_list: Vec<AbilityInfo>,
    #[serde(alias = "LinkEnemies")]
    pub link_enemies: Vec<String>,
    #[serde(alias = "DamageType")]
    pub damage_type: Vec<DamageType>,
    #[serde(alias = "InvisibleDetail")]
    pub invisible_detail: bool,
}

// ============================================================================
// Container Types (used in GameData)
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyHandbook {
    pub level_info_list: Vec<EnemyInfoList>,
    pub enemy_data: HashMap<String, Enemy>,
    pub race_data: HashMap<String, RaceData>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EnemyHandbookTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub enemy_data: HashMap<String, Enemy>,
    pub level_info_list: Vec<EnemyInfoList>,
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub race_data: HashMap<String, RaceData>,
}

impl From<EnemyHandbookTableFile> for EnemyHandbook {
    fn from(table: EnemyHandbookTableFile) -> Self {
        Self {
            level_info_list: table.level_info_list,
            enemy_data: table.enemy_data,
            race_data: table.race_data,
        }
    }
}
