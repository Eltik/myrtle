use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use super::gacha_detail::{RarityRate, WeightUpChar};
use super::serde_helpers::deserialize_fb_map;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct GachaPoolClient {
    #[serde(alias = "GachaPoolId")]
    pub gacha_pool_id: String,
    #[serde(alias = "GachaIndex")]
    pub gacha_index: i32,
    #[serde(alias = "OpenTime")]
    #[ts(type = "number")]
    pub open_time: i64,
    #[serde(alias = "EndTime")]
    #[ts(type = "number")]
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

    // Decoded from `LimitParam.Base64` / `DynMeta.Base64` during enrichment:
    // base64-wrapped BSON whose shape varies by `gacha_rule_type` (LIMITED has
    // `limitedCharId`; CLASSIC_DOUBLE main/sub 6★ + rare5CharList; FESCLASSIC /
    // SPECIAL nest `rarityPickCharDict` with TIER_5/TIER_6; ATTAIN variants have
    // `attainRare6CharList`). Empty when the banner has none this way (NORMAL /
    // SINGLE pools carry only art, LINKAGE pools use a separate LinkageRuleId).
    #[serde(default, skip_deserializing)]
    pub featured6: Vec<String>,
    #[serde(default, skip_deserializing)]
    pub featured5: Vec<String>,

    /// Rate-boosted operators, merged in from the pool-detail sidecar.
    ///
    /// Kept separate from `featured6` on purpose: a weight-up operator keeps the
    /// band's base rate and merely takes a larger share of it, which is a
    /// different promise to the player than a rate-up.
    #[serde(default, skip_deserializing)]
    pub weight_up: Vec<WeightUpChar>,
    /// Headline per-rarity rates for this banner.
    #[serde(default, skip_deserializing)]
    pub avail_rates: Vec<RarityRate>,
    /// Selectable 6* candidates on player-pick banners. Empty elsewhere.
    #[serde(default, skip_deserializing)]
    pub pickup6: Vec<String>,
    /// Provenance of the featured lists: `"static"` when only the BSON blobs
    /// were available, `"static+api"` once pool detail has been merged in.
    #[serde(default, skip_deserializing)]
    pub featured_source: String,
    /// Shown only to returning players: a pool some `open_server_table`
    /// `PlayerReturn` tier grants (see `open_server`). Its `gacha_table` window
    /// is years long because the game times it per player, so it is never
    /// "running now for everyone" and its `end_time` is not a real end date.
    #[serde(default, skip_deserializing)]
    pub returning: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RecruitTimeEntry {
    #[serde(alias = "RecruitPrice")]
    pub recruit_price: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RecruitPool {
    #[serde(alias = "RecruitTimeTable")]
    pub recruit_time_table: Vec<RecruitTimeEntry>,
}

/// The gacha tag id of each class, keyed by profession code. Every client
/// ships the 8 class names as recruitment tags under these fixed ids, in the
/// same order (checked on en, cn, kr and jp on 2026-10-06: 1 is Guard,
/// 近卫干员, 가드 and 前衛タイプ), so a class reads in the server's own wording
/// with no translation table and a server added later needs no code. The
/// frontend recruitment tool keeps the same numbering as `PROFESSION_TAG_ID`
/// in `components/tools/recruitment/impl/constants.ts`.
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

/// Profession code -> the server's own class name, read off its gacha tags
/// verbatim (CN's 干员 and JP's タイプ suffixes are the game's wording, kept).
/// A tag id the list lacks, or an empty name, leaves that class out.
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
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct RecruitRarityEntry {
    #[serde(alias = "RarityStart")]
    pub rarity_start: i32,
    #[serde(alias = "RarityEnd")]
    pub rarity_end: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct GachaData {
    pub gacha_pool_client: Vec<GachaPoolClient>,
    pub newbee_gacha_pool_client: Vec<NewbeeGachaPoolClient>,
    pub special_recruit_pool: Vec<serde_json::Value>,
    pub gacha_tags: Vec<GachaTag>,
    pub recruit_pool: RecruitPool,
    pub potential_material_converter: serde_json::Value,
    pub classic_potential_material_converter: serde_json::Value,
    #[ts(as = "HashMap<String, RecruitRarityEntry>")]
    pub recruit_rarity_table: HashMap<i32, RecruitRarityEntry>,
    #[ts(as = "HashMap<String, Vec<i32>>")]
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
    #[ts(as = "HashMap<String, f64>")]
    pub special_gacha_percent_dict: HashMap<i32, f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
