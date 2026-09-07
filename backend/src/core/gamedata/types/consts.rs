use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameDataConst {
    pub character_exp_map: Vec<ExpMapEntry>,
    pub character_upgrade_cost_map: Vec<UpgradeCostMapEntry>,
    pub evolve_gold_cost: Vec<EvolveGoldCostEntry>,
    /// The game's term glossary (`cc.bd_*` pool resources etc.): the text
    /// behind a `<$cc.bd_wang_1>` reference, e.g. "For every Trading Post and
    /// Power Plant, Influence +1" - the only place a layout-counted resource
    /// is defined.
    #[serde(default)]
    pub term_description_dict: Vec<TermDescriptionEntry>,
}

/// One `TermDescriptionDict` row: `{"key": "cc.bd_wang_1", "value": {...}}`.
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
