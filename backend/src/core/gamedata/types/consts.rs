use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GameDataConst {
    pub character_exp_map: Vec<ExpMapEntry>,
    pub character_upgrade_cost_map: Vec<UpgradeCostMapEntry>,
    pub evolve_gold_cost: Vec<EvolveGoldCostEntry>,
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
