use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::release::{
    prices::SkinPrice,
    skins::{AnniversaryStats, RerunBasis, SaleWindow},
};
use crate::core::{
    gamedata::types::activity::{FarmStage, OpStage},
    translate::AutoName,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum Resolution {
    Confirmed {
        en_id: String,
        #[ts(type = "number")]
        en_start: i64,
        #[ts(type = "number")]
        en_end: i64,
    },
    Override {
        en_id: Option<String>,
        #[ts(type = "number")]
        en_start: i64,
        #[ts(type = "number | null")]
        en_end: Option<i64>,
        source: String,
        note: String,
    },
    Estimated {
        #[ts(type = "number")]
        en_start: i64,
        #[ts(type = "number")]
        lo: i64,
        #[ts(type = "number")]
        hi: i64,
    },
    Unmodelled,
    Independent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LagSample {
    pub cn_id: String,
    pub name: String,
    #[ts(type = "number")]
    pub cn_start: i64,
    #[ts(type = "number")]
    pub en_start: i64,
    pub lag_days: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LagModel {
    pub window: usize,
    pub n: usize,
    pub median_days: f64,
    pub p25_days: f64,
    pub p75_days: f64,
    pub samples: Vec<LagSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct YearlyModel {
    pub types: Vec<String>,
    pub model: LagModel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Backtest {
    pub window: usize,
    pub n: usize,
    pub median_abs_err_days: f64,
    pub p75_abs_err_days: f64,
    pub p90_abs_err_days: f64,
    pub max_abs_err_days: f64,
    pub band_hit_rate: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LagResponse {
    pub model: LagModel,
    pub yearly: YearlyModel,
    pub backtest: Backtest,
}

/// What a shop good is, for grouping; the server's `item_type` is kept beside
/// it for anything finer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ShopGoodKind {
    Outfit,
    Furniture,
    Material,
    Currency,
    Exp,
    Ticket,
    Other,
}

impl ShopGoodKind {
    pub fn of(item_type: &str) -> Self {
        match item_type {
            "CHAR_SKIN" => Self::Outfit,
            "FURN" | "HOME_THEME" => Self::Furniture,
            "MATERIAL" => Self::Material,
            "GOLD" | "DIAMOND_SHD" | "AP_SUPPLY" => Self::Currency,
            "CARD_EXP" => Self::Exp,
            t if t.starts_with("TKT_") => Self::Ticket,
            _ => Self::Other,
        }
    }
}

/// An item as the shop names it: the client table's name and icon, EN when
/// the item exists there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ShopItem {
    pub item_id: String,
    pub name: String,
    pub name_en: Option<String>,
    pub icon_path: Option<String>,
    pub rarity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ShopGood {
    pub good_id: String,
    pub item_type: String,
    pub kind: ShopGoodKind,
    pub item: ShopItem,
    pub count: i32,
    pub price: i32,
    /// Purchase limit; `-1` is unlimited stock, which the buy-out leaves out.
    pub avail_count: i32,
}

/// An event's token shop as the planner shows it: the game server's goods
/// (see `gamedata::types::event_shop`) with the names and icons the client
/// tables give them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventShop {
    /// Which server's shop this is: CN when that account has fetched it,
    /// else EN.
    pub server: String,
    pub shop_name: String,
    pub token: ShopItem,
    #[ts(type = "number")]
    pub start_time: i64,
    #[ts(type = "number")]
    pub end_time: i64,
    /// Token cost of every limited good at full stock, the server's figure.
    pub max_price: i32,
    pub goods: Vec<ShopGood>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReleaseEvent {
    pub cn_id: String,
    pub name_cn: String,
    pub name_en: Option<String>,
    pub activity_type: String,
    pub has_stage: bool,
    #[ts(type = "number")]
    pub cn_start: i64,
    #[ts(type = "number")]
    pub cn_end: i64,
    pub op_stages: Vec<OpStage>,
    pub farm_stages: Vec<FarmStage>,
    pub mission_tokens: i32,
    pub shop: Option<EventShop>,
    pub name_en_auto: Option<AutoName>,
    pub image_path: Option<String>,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventsResponse {
    pub model: LagModel,
    pub yearly: YearlyModel,
    pub events: Vec<ReleaseEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AlignMethod {
    Content,
    Anchor,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PoolAlignment {
    pub en_pool_id: Option<String>,
    pub method: AlignMethod,
}

pub const STANDING_POOL_SECS: i64 = 365 * 86_400;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReleaseBanner {
    pub cn_pool_id: String,
    pub rule_type: String,
    pub name_cn: String,
    #[ts(type = "number")]
    pub cn_open: i64,
    #[ts(type = "number")]
    pub cn_end: i64,
    pub standing: bool,
    pub featured6: Vec<String>,
    pub featured5: Vec<String>,
    pub debut_chars: Vec<String>,
    pub anchor_activity: Option<String>,
    pub alignment: PoolAlignment,
    pub en_featured6: Vec<String>,
    pub override_featured: Vec<String>,
    pub name_en_auto: Option<AutoName>,
    pub image_path: Option<String>,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BannersResponse {
    pub model: LagModel,
    pub yearly: YearlyModel,
    pub banners: Vec<ReleaseBanner>,
    pub char_names: HashMap<String, AutoName>,
    pub rule_independence: Vec<crate::core::release::align::RuleIndependence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NewSkin {
    pub skin_id: String,
    pub char_id: String,
    pub skin_name: String,
    pub skin_group_id: String,
    pub skin_group_name: String,
    pub is_buy_skin: bool,
    pub obtain_approach: Option<String>,
    #[ts(type = "number")]
    pub cn_get_time: i64,
    pub skin_name_auto: Option<AutoName>,
    pub skin_group_name_auto: Option<AutoName>,
    pub char_name: Option<AutoName>,
    pub portrait_path: Option<String>,
    pub colors: Vec<String>,
    pub price: SkinPrice,
    pub anchor: Option<EventAnchor>,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventAnchor {
    pub cn_id: String,
    pub name_cn: String,
    #[ts(type = "number")]
    pub cn_start: i64,
    #[ts(type = "number")]
    pub cn_end: i64,
    #[ts(type = "number")]
    pub offset_secs: i64,
    pub name_en: Option<String>,
    pub name_en_auto: Option<AutoName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinTile {
    pub skin_id: String,
    pub char_id: String,
    pub skin_name: String,
    pub char_name: Option<AutoName>,
    pub portrait_path: Option<String>,
    pub colors: Vec<String>,
    pub price: SkinPrice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinGroupArt {
    pub skin_group_id: String,
    pub brand_id: String,
    pub brand_name: String,
    pub kv_path: Option<String>,
    pub logo_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BatchForecast {
    pub name: String,
    pub name_en_auto: Option<AutoName>,
    #[ts(type = "number")]
    pub cn_start: i64,
    #[ts(type = "number")]
    pub cn_end: i64,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RerunForecast {
    pub skin_group_id: String,
    pub skin_group_name: String,
    pub skin_ids: Vec<String>,
    pub skins: Vec<SkinTile>,
    pub windows: Vec<SaleWindow>,
    #[ts(type = "number")]
    pub last_seen: i64,
    pub next: Resolution,
    pub basis: RerunBasis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReviewWindow {
    #[ts(type = "number")]
    pub cn_start: i64,
    #[ts(type = "number")]
    pub cn_end: i64,
    /// Newest CN release date this edition stocks; the pool is every
    /// `ReviewOutfit` with `cn_get_time` at or before it.
    #[ts(type = "number")]
    pub pool_cutoff: i64,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReviewOutfit {
    #[serde(flatten)]
    #[ts(flatten)]
    pub tile: SkinTile,
    #[ts(type = "number")]
    pub cn_get_time: i64,
    #[ts(type = "number | null")]
    pub en_get_time: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinsResponse {
    pub model: LagModel,
    pub yearly: YearlyModel,
    pub anniversaries: Vec<AnniversaryStats>,
    pub batches: Vec<BatchForecast>,
    pub new_skins: Vec<NewSkin>,
    pub rerun_forecasts: Vec<RerunForecast>,
    pub reviews: Vec<ReviewWindow>,
    pub review_pool: Vec<ReviewOutfit>,
    pub group_art: HashMap<String, SkinGroupArt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReleaseOverride {
    pub kind: String,
    pub cn_id: String,
    pub en_id: Option<String>,
    pub en_name: Option<String>,
    #[ts(type = "number | null")]
    pub en_start: Option<i64>,
    #[ts(type = "number | null")]
    pub en_end: Option<i64>,
    pub featured_chars: Option<Vec<String>>,
    pub source: String,
    pub note: String,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PutOverride {
    pub kind: String,
    pub cn_id: String,
    #[serde(default)]
    pub en_id: Option<String>,
    #[serde(default)]
    pub en_name: Option<String>,
    #[serde(default)]
    pub featured_chars: Option<Vec<String>>,
    #[ts(type = "number | null")]
    #[serde(default)]
    pub en_start: Option<i64>,
    #[ts(type = "number | null")]
    #[serde(default)]
    pub en_end: Option<i64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ReleasePlan {
    pub initial: i32,
    pub initial_manual: bool,
    pub picks: Vec<String>,
    pub stages: HashMap<String, HashMap<String, bool>>,
    #[ts(type = "number")]
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PutReleasePlan {
    pub initial: i32,
    pub initial_manual: bool,
    pub picks: Vec<String>,
    pub stages: HashMap<String, HashMap<String, bool>>,
}
