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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnemyApplyWay {
    #[default]
    Melee,
    Ranged,
    None,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnemyMotion {
    #[default]
    Walk,
    Fly,
    #[serde(other)]
    Unknown,
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
// Enemy Stats (from enemy_database.json)
// ============================================================================

/// Wrapper for optional values in enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MaybeValue<T: Default> {
    #[serde(rename = "M_defined")]
    pub defined: bool,
    #[serde(rename = "M_value", default)]
    pub value: T,
}

impl<T: Default> MaybeValue<T> {
    pub fn get(&self) -> Option<&T> {
        if self.defined {
            Some(&self.value)
        } else {
            None
        }
    }

    pub fn into_option(self) -> Option<T> {
        if self.defined { Some(self.value) } else { None }
    }
}

/// Enemy attributes from enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawEnemyAttributes {
    pub max_hp: MaybeValue<i32>,
    pub atk: MaybeValue<i32>,
    pub def: MaybeValue<i32>,
    pub magic_resistance: MaybeValue<f64>,
    pub move_speed: MaybeValue<f64>,
    pub attack_speed: MaybeValue<f64>,
    pub base_attack_time: MaybeValue<f64>,
    pub mass_level: MaybeValue<i32>,
    pub hp_recovery_per_sec: MaybeValue<f64>,
    pub stun_immune: MaybeValue<bool>,
    pub silence_immune: MaybeValue<bool>,
    pub sleep_immune: MaybeValue<bool>,
    pub frozen_immune: MaybeValue<bool>,
    pub levitate_immune: MaybeValue<bool>,
    pub palsy_immune: MaybeValue<bool>,
    pub feared_immune: MaybeValue<bool>,
    pub ep_damage_resistance: MaybeValue<f64>,
    pub ep_resistance: MaybeValue<f64>,
}

/// Skill blackboard entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillBlackboardEntry {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub value_str: Option<String>,
}

/// Enemy skill from enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawEnemySkill {
    pub prefab_key: String,
    pub priority: i32,
    pub cooldown: f64,
    pub init_cooldown: f64,
    pub sp_cost: i32,
    #[serde(default)]
    pub blackboard: Vec<SkillBlackboardEntry>,
}

/// Enemy data entry from enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawEnemyData {
    pub name: MaybeValue<String>,
    pub description: MaybeValue<String>,
    pub prefab_key: MaybeValue<String>,
    pub attributes: RawEnemyAttributes,
    pub apply_way: MaybeValue<String>,
    pub motion: MaybeValue<String>,
    pub level_type: MaybeValue<String>,
    pub range_radius: MaybeValue<f64>,
    pub life_point_reduce: MaybeValue<i32>,
    #[serde(default)]
    pub skills: Vec<RawEnemySkill>,
}

/// Level entry in enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawEnemyLevelEntry {
    pub level: i32,
    pub enemy_data: RawEnemyData,
}

/// Enemy entry in enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RawEnemyDatabaseEntry {
    pub key: String,
    pub value: Vec<RawEnemyLevelEntry>,
}

/// Root structure for enemy_database.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EnemyDatabaseFile {
    pub enemies: Vec<RawEnemyDatabaseEntry>,
}

// ============================================================================
// Enriched Enemy Stats (output format)
// ============================================================================

/// Processed enemy attributes for API output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyAttributes {
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub magic_resistance: f64,
    pub move_speed: f64,
    pub attack_speed: f64,
    pub base_attack_time: f64,
    pub mass_level: i32,
    pub hp_recovery_per_sec: f64,
    pub stun_immune: bool,
    pub silence_immune: bool,
    pub sleep_immune: bool,
    pub frozen_immune: bool,
    pub levitate_immune: bool,
}

/// Processed enemy skill for API output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemySkill {
    pub prefab_key: String,
    pub priority: i32,
    pub cooldown: f64,
    pub init_cooldown: f64,
    pub sp_cost: i32,
    pub blackboard: Vec<SkillBlackboardEntry>,
}

/// Per-level stats for an enemy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyLevelStats {
    pub level: i32,
    pub attributes: EnemyAttributes,
    pub apply_way: Option<String>,
    pub motion: Option<String>,
    pub range_radius: Option<f64>,
    pub life_point_reduce: i32,
    pub skills: Vec<EnemySkill>,
}

/// All stats for an enemy (all levels)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyStats {
    pub levels: Vec<EnemyLevelStats>,
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
    /// Enriched stats from enemy_database.json
    #[serde(default, skip_deserializing)]
    pub stats: Option<EnemyStats>,
    /// Enemy portrait/icon path
    #[serde(default, skip_deserializing)]
    pub portrait: Option<String>,
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
