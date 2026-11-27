use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameDataConst {
    pub character_exp_map: Vec<ExpMapEntry>,
    pub character_upgrade_cost_map: Vec<UpgradeCostMapEntry>,
    pub evolve_gold_cost: Vec<EvolveGoldCostEntry>,
    /// Term glossary (`cc.bd_*` pool resources etc.), text behind a
    /// `<$cc.bd_wang_1>` reference
    #[serde(default)]
    pub term_description_dict: Vec<TermDescriptionEntry>,
    /// Minutes per point of natural sanity regeneration, 0 if absent
    #[serde(default)]
    pub player_ap_regen_speed: i64,
}

/// `TermDescriptionDict` row, e.g. `{"key": "cc.bd_wang_1", "value": {...}}`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TermDescriptionEntry {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: TermDescription,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TermDescription {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub term_id: String,
    #[serde(default)]
    pub term_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExpMapEntry {
    pub values: Vec<i32>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpgradeCostMapEntry {
    pub values: Vec<i32>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EvolveGoldCostEntry {
    pub values: Vec<i32>,
}
