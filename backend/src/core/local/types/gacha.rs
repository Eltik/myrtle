use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaPoolClient {
    #[serde(alias = "GachaPoolId")]
    pub gacha_pool_id: String,
    #[serde(alias = "GachaIndex")]
    pub gacha_index: i32,
    #[serde(alias = "OpenTime")]
    pub open_time: i64,
    #[serde(alias = "EndTime")]
    pub end_time: i64,
    #[serde(alias = "GachaPoolName")]
    pub gacha_pool_name: String,
    #[serde(alias = "GachaPoolSummary")]
    pub gacha_pool_summary: String,
    #[serde(alias = "GachaPoolDetail")]
    pub gacha_pool_detail: Option<String>,
    #[serde(alias = "Guarantee5Avail")]
    pub guarantee5_avail: i32,
    #[serde(alias = "Guarantee5Count")]
    pub guarantee5_count: i32,
    #[serde(alias = "GachaRuleType")]
    pub gacha_rule_type: String,
    #[serde(alias = "LMTGSID")]
    pub lmtgsid: Option<String>,
    #[serde(alias = "CDPrimColor")]
    pub cd_prim_color: Option<String>,
    #[serde(alias = "CDSecColor")]
    pub cd_sec_color: Option<String>,
    #[serde(alias = "LimitParam")]
    pub limit_param: Option<serde_json::Value>,
    #[serde(alias = "LinkageParam")]
    pub linkage_param: Option<serde_json::Value>,
    #[serde(alias = "LinkageRuleId")]
    pub linkage_rule_id: Option<String>,
    #[serde(alias = "DynMeta")]
    pub dyn_meta: Option<serde_json::Value>,
    #[serde(alias = "FreeBackColor")]
    pub free_back_color: Option<String>,
    #[serde(alias = "GuaranteeName")]
    pub guarantee_name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewbeeGachaPoolClient {
    #[serde(alias = "GachaPoolId")]
    pub gacha_pool_id: String,
    #[serde(alias = "GachaIndex")]
    pub gacha_index: i32,
    #[serde(alias = "GachaPoolName")]
    pub gacha_pool_name: String,
    #[serde(alias = "GachaPoolDetail")]
    pub gacha_pool_detail: Option<String>,
    #[serde(alias = "GachaPrice")]
    pub gacha_price: i32,
    #[serde(alias = "GachaTimes")]
    pub gacha_times: i32,
    #[serde(alias = "GachaOffset")]
    pub gacha_offset: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaTag {
    #[serde(alias = "TagId")]
    pub tag_id: i32,
    #[serde(alias = "TagName")]
    pub tag_name: String,
    #[serde(alias = "TagGroup")]
    pub tag_group: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitTimeEntry {
    #[serde(alias = "RecruitPrice")]
    pub recruit_price: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitPool {
    #[serde(alias = "RecruitTimeTable")]
    pub recruit_time_table: Vec<RecruitTimeEntry>,
    // Add other fields as needed
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitRarityEntry {
    #[serde(alias = "RarityStart")]
    pub rarity_start: i32,
    #[serde(alias = "RarityEnd")]
    pub rarity_end: i32,
}

// ============================================================================
// Container Type
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaData {
    pub gacha_pool_client: Vec<GachaPoolClient>,
    pub newbee_gacha_pool_client: Vec<NewbeeGachaPoolClient>,
    pub special_recruit_pool: Vec<serde_json::Value>,
    pub gacha_tags: Vec<GachaTag>,
    pub recruit_pool: RecruitPool,
    pub potential_material_converter: serde_json::Value,
    pub classic_potential_material_converter: serde_json::Value,
    pub recruit_rarity_table: HashMap<i32, RecruitRarityEntry>,
    pub special_tag_rarity_table: HashMap<i32, Vec<i32>>,
    pub recruit_detail: String,
    pub show_gacha_log_entry: bool,
    pub carousel: Vec<serde_json::Value>,
    pub free_gacha: Vec<serde_json::Value>,
    pub limit_ten_gacha_item: Vec<serde_json::Value>,
    pub linkage_ten_gacha_item: Vec<serde_json::Value>,
    pub normal_gacha_item: Vec<serde_json::Value>,
    pub fes_gacha_pool_relate_item: HashMap<String, FesGachaPoolRelateEntry>,
    pub dic_recruit6_star_hint: HashMap<String, String>,
    pub special_gacha_percent_dict: HashMap<i32, f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FesGachaPoolRelateEntry {
    #[serde(alias = "RarityRank5ItemId")]
    pub rarity_rank5_item_id: String,
    #[serde(alias = "RarityRank6ItemId")]
    pub rarity_rank6_item_id: String,
}

// ============================================================================
// Table File Wrapper
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GachaTableFile {
    pub gacha_pool_client: Vec<GachaPoolClient>,
    pub newbee_gacha_pool_client: Vec<NewbeeGachaPoolClient>,
    #[serde(default)]
    pub special_recruit_pool: Vec<serde_json::Value>,
    pub gacha_tags: Vec<GachaTag>,
    pub recruit_pool: RecruitPool,
    #[serde(default)]
    pub potential_material_converter: serde_json::Value,
    #[serde(default)]
    pub classic_potential_material_converter: serde_json::Value,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub recruit_rarity_table: HashMap<i32, RecruitRarityEntry>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub special_tag_rarity_table: HashMap<i32, Vec<i32>>,
    #[serde(default)]
    pub recruit_detail: String,
    #[serde(default)]
    pub show_gacha_log_entry: bool,
    #[serde(default)]
    pub carousel: Vec<serde_json::Value>,
    #[serde(default)]
    pub free_gacha: Vec<serde_json::Value>,
    #[serde(default)]
    pub limit_ten_gacha_item: Vec<serde_json::Value>,
    #[serde(default)]
    pub linkage_ten_gacha_item: Vec<serde_json::Value>,
    #[serde(default)]
    pub normal_gacha_item: Vec<serde_json::Value>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub fes_gacha_pool_relate_item: HashMap<String, FesGachaPoolRelateEntry>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub dic_recruit6_star_hint: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub special_gacha_percent_dict: HashMap<i32, f64>,
}
