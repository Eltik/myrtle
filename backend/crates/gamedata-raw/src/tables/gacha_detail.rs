use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::gacha::WeightUpChar;

/// `rarityRank` of a 6★ in `gacha/getPoolDetail`, which counts from 0
pub const RARITY_RANK_6: i32 = 5;
/// `rarityRank` of a 5★
pub const RARITY_RANK_5: i32 = 4;

/// Sidecar layout version, another version is discarded on load
pub const POOL_DETAIL_FILE_VERSION: u32 = 1;
/// Sidecar path under the server root, written by the pool-detail job
pub const POOL_DETAIL_REL_PATH: &str = "derived/gacha_pool_details.json";

/// `derived/gacha_pool_details.json`, every `gacha/getPoolDetail` body fetched, keyed by pool id
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDetailFile {
    #[serde(default)]
    pub version: u32,
    /// Unix seconds of the last successful refresh
    #[serde(default)]
    pub fetched_at: i64,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub pools: HashMap<String, GachaPoolDetail>,
}

/// One `gacha/getPoolDetail` response body
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaPoolDetail {
    #[serde(default)]
    pub detail_info: DetailInfo,
    #[serde(default)]
    pub has_rate_up: bool,
    #[serde(default)]
    pub gacha_obj_group_type: i32,
}

impl GachaPoolDetail {
    /// Player-pick banner (`SPECIAL` pickup), by layout discriminator rather than rule type
    pub fn is_pickup(&self) -> bool {
        self.detail_info.gacha_obj_list.as_ref().is_some_and(|l| {
            l.iter()
                .any(|o| o.gacha_object.starts_with("SPECIAL_PICKUP"))
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailInfo {
    /// Rate-ups proper, empty on FESCLASSIC / SPECIAL / `CLASSIC_ATTAIN`
    #[serde(default)]
    pub up_char_info: Option<UpCharInfo>,
    /// Soft rate-boosted units, a larger share of the base 6★ rate
    #[serde(default)]
    pub weight_up_char_info_list: Option<Vec<WeightUpChar>>,
    #[serde(default)]
    pub limited_char: Option<Vec<String>>,
    #[serde(default)]
    pub avail_char_info: Option<AvailCharInfo>,
    #[serde(default)]
    pub gacha_obj_list: Option<Vec<GachaObj>>,
    #[serde(default)]
    pub gacha_obj_groups: Option<Vec<GachaObjGroup>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpCharInfo {
    #[serde(default)]
    pub per_char_list: Vec<PerCharEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerCharEntry {
    #[serde(default)]
    pub rarity_rank: i32,
    /// Share of the rarity's rate taken by each operator, a 0..1 fraction
    #[serde(default)]
    pub percent: f64,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub char_id_list: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailCharInfo {
    #[serde(default)]
    pub per_avail_list: Vec<PerAvailEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerAvailEntry {
    #[serde(default)]
    pub rarity_rank: i32,
    /// Headline rate for the whole rarity band, e.g. `0.02` for 6★
    #[serde(default)]
    pub total_percent: f64,
    #[serde(default)]
    pub char_id_list: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaObj {
    /// Layout discriminator, e.g. `UP_CHAR`, `FES_CLASSIC_UP_CHAR`, `SPECIAL_PICKUP_SELECT_CHAR`
    #[serde(default)]
    pub gacha_object: String,
    #[serde(default)]
    pub image_type: i32,
    /// Free text for `TEXT` blocks, `null` for character blocks
    #[serde(default)]
    pub param: Option<String>,
    #[serde(rename = "type", default)]
    pub obj_type: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaObjGroup {
    #[serde(default)]
    pub group_type: i32,
    #[serde(default)]
    pub start_index: i32,
    #[serde(default)]
    pub end_index: i32,
}
