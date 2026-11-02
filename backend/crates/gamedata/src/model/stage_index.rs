use myrtle_gamedata_raw::tables::stage::StageDifficulty;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageIndexEntry {
    pub stage_id: String,
    pub level_id: Option<String>,
    pub code: String,
    pub name: Option<String>,
    pub zone_id: String,
    pub zone_name: Option<String>,
    pub zone_order: i64,
    pub group: String,
    pub category: String,
    pub ap_cost: i32, // Sanity cost
    pub boss: bool,
    pub originite_prime: i32,
    pub is_hard: bool, // Hard / Adverse / Challenge Mode
    pub difficulty: StageDifficulty,
    pub can_view: bool,
    pub preview: Option<String>,
    pub banner: Option<String>,
}

pub type StageIndex = Vec<StageIndexEntry>;
