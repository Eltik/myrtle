#[derive(Debug, Default)]
pub struct GachaTables {}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "schema")]
use utoipa::ToSchema;

use crate::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
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
    pub limit_param: Option<Value>,
    #[serde(alias = "LinkageParam")]
    pub linkage_param: Option<Value>,
    #[serde(alias = "LinkageRuleId")]
    pub linkage_rule_id: Option<String>,
    #[serde(alias = "DynMeta")]
    pub dyn_meta: Option<Value>,
    #[serde(alias = "FreeBackColor")]
    pub free_back_color: Option<String>,
    #[serde(alias = "GuaranteeName")]
    pub guarantee_name: Option<String>,

    #[serde(default, skip_deserializing)]
    pub featured6: Vec<String>,
    #[serde(default, skip_deserializing)]
    pub featured5: Vec<String>,

    /// Rate-boosted operators, from the pool-detail sidecar
    #[serde(default, skip_deserializing)]
    pub weight_up: Vec<WeightUpChar>,
    /// Headline per-rarity rates
    #[serde(default, skip_deserializing)]
    pub avail_rates: Vec<RarityRate>,
    /// Selectable 6* candidates on player-pick banners
    #[serde(default, skip_deserializing)]
    pub pickup6: Vec<String>,
    /// Featured list source: `"static"` (BSON blobs only) or `"static+api"` (pool detail merged)
    #[serde(default, skip_deserializing)]
    pub featured_source: String,
    /// Returning-player pool; `end_time` is not a real end date
    #[serde(default, skip_deserializing)]
    pub returning: bool,
    /// Per-operator 6★ rate-up share as a 0..1 fraction of the 6★ band
    /// (`upCharInfo.perCharList[rarityRank=5].percent`), `None` without a 6★ rate-up
    #[serde(skip)]
    pub declared_share6: Option<f64>,
    /// Rolls per cycle of a repeating collab handover (`LinkageParam.loopTarget6Count`),
    /// `None` otherwise
    #[serde(skip)]
    pub linkage_loop_at: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
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
#[cfg_attr(feature = "schema", derive(ToSchema))]
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
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct RecruitTimeEntry {
    #[serde(alias = "RecruitPrice")]
    pub recruit_price: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct RecruitPool {
    #[serde(alias = "RecruitTimeTable")]
    pub recruit_time_table: Vec<RecruitTimeEntry>,
}

/// Gacha tag id of each class, keyed by profession code
pub const PROFESSION_TAG_IDS: [(&str, i32); 8] = [
    ("WARRIOR", 1),
    ("SNIPER", 2),
    ("TANK", 3),
    ("MEDIC", 4),
    ("SUPPORT", 5),
    ("CASTER", 6),
    ("SPECIAL", 7),
    ("PIONEER", 8),
];

/// Gacha tag ids of the two positions and the two handbook genders
pub const MELEE_TAG_ID: i32 = 9;
pub const RANGED_TAG_ID: i32 = 10;
pub const MALE_TAG_ID: i32 = 1012;
pub const FEMALE_TAG_ID: i32 = 1013;

/// Profession code -> server's class name from its gacha tags
pub fn profession_names(tags: &[GachaTag]) -> HashMap<String, String> {
    PROFESSION_TAG_IDS
        .iter()
        .filter_map(|(code, id)| {
            let tag = tags.iter().find(|tag| tag.tag_id == *id)?;
            (!tag.tag_name.is_empty()).then(|| ((*code).to_owned(), tag.tag_name.clone()))
        })
        .collect()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct RecruitRarityEntry {
    #[serde(alias = "RarityStart")]
    pub rarity_start: i32,
    #[serde(alias = "RarityEnd")]
    pub rarity_end: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct GachaData {
    pub gacha_pool_client: Vec<GachaPoolClient>,
    pub newbee_gacha_pool_client: Vec<NewbeeGachaPoolClient>,
    pub special_recruit_pool: Vec<Value>,
    pub gacha_tags: Vec<GachaTag>,
    pub recruit_pool: RecruitPool,
    pub potential_material_converter: Value,
    pub classic_potential_material_converter: Value,
    pub recruit_rarity_table: HashMap<i32, RecruitRarityEntry>,
    pub special_tag_rarity_table: HashMap<i32, Vec<i32>>,
    pub recruit_detail: String,
    pub show_gacha_log_entry: bool,
    pub carousel: Vec<Value>,
    pub free_gacha: Vec<Value>,
    pub limit_ten_gacha_item: Vec<Value>,
    pub linkage_ten_gacha_item: Vec<Value>,
    pub normal_gacha_item: Vec<Value>,
    pub fes_gacha_pool_relate_item: HashMap<String, FesGachaPoolRelateEntry>,
    pub dic_recruit6_star_hint: HashMap<String, String>,
    pub special_gacha_percent_dict: HashMap<i32, f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct FesGachaPoolRelateEntry {
    #[serde(alias = "RarityRank5ItemId")]
    pub rarity_rank5_item_id: String,
    #[serde(alias = "RarityRank6ItemId")]
    pub rarity_rank6_item_id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GachaTableFile {
    pub gacha_pool_client: Vec<GachaPoolClient>,
    pub newbee_gacha_pool_client: Vec<NewbeeGachaPoolClient>,
    #[serde(default)]
    pub special_recruit_pool: Vec<Value>,
    pub gacha_tags: Vec<GachaTag>,
    pub recruit_pool: RecruitPool,
    #[serde(default)]
    pub potential_material_converter: Value,
    #[serde(default)]
    pub classic_potential_material_converter: Value,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub recruit_rarity_table: HashMap<i32, RecruitRarityEntry>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub special_tag_rarity_table: HashMap<i32, Vec<i32>>,
    #[serde(default)]
    pub recruit_detail: String,
    #[serde(default)]
    pub show_gacha_log_entry: bool,
    #[serde(default)]
    pub carousel: Vec<Value>,
    #[serde(default)]
    pub free_gacha: Vec<Value>,
    #[serde(default)]
    pub limit_ten_gacha_item: Vec<Value>,
    #[serde(default)]
    pub linkage_ten_gacha_item: Vec<Value>,
    #[serde(default)]
    pub normal_gacha_item: Vec<Value>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub fes_gacha_pool_relate_item: HashMap<String, FesGachaPoolRelateEntry>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub dic_recruit6_star_hint: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub special_gacha_percent_dict: HashMap<i32, f64>,
}

impl GachaTableFile {
    pub fn into_gacha_data(self) -> GachaData {
        GachaData {
            gacha_pool_client: self.gacha_pool_client,
            newbee_gacha_pool_client: self.newbee_gacha_pool_client,
            special_recruit_pool: self.special_recruit_pool,
            gacha_tags: self.gacha_tags,
            recruit_pool: self.recruit_pool,
            potential_material_converter: self.potential_material_converter,
            classic_potential_material_converter: self.classic_potential_material_converter,
            recruit_rarity_table: self.recruit_rarity_table,
            special_tag_rarity_table: self.special_tag_rarity_table,
            recruit_detail: self.recruit_detail,
            show_gacha_log_entry: self.show_gacha_log_entry,
            carousel: self.carousel,
            free_gacha: self.free_gacha,
            limit_ten_gacha_item: self.limit_ten_gacha_item,
            linkage_ten_gacha_item: self.linkage_ten_gacha_item,
            normal_gacha_item: self.normal_gacha_item,
            fes_gacha_pool_relate_item: self.fes_gacha_pool_relate_item,
            dic_recruit6_star_hint: self.dic_recruit6_star_hint,
            special_gacha_percent_dict: self.special_gacha_percent_dict,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct WeightUpChar {
    #[serde(default)]
    pub char_id: String,
    #[serde(default)]
    pub rarity_rank: i32,
    #[serde(default)]
    pub weight: i64,
}

/// Per-rarity headline rate for a banner
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct RarityRate {
    pub rarity_rank: i32,
    pub total_percent: f64,
    /// Operator count in this rarity band
    pub pool_size: usize,
}
