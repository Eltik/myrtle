use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::serde_helpers::deserialize_fb_map;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemRarity {
    #[serde(rename = "TIER_1")]
    Tier1,
    #[serde(rename = "TIER_2")]
    Tier2,
    #[serde(rename = "TIER_3")]
    Tier3,
    #[serde(rename = "TIER_4")]
    Tier4,
    #[serde(rename = "TIER_5")]
    Tier5,
    #[serde(rename = "TIER_6")]
    Tier6,
}

impl Default for ItemRarity {
    fn default() -> Self {
        Self::Tier1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemClass {
    #[serde(rename = "MATERIAL")]
    Material,
    #[serde(rename = "CONSUME")]
    Consumable,
    #[serde(rename = "NORMAL")]
    Normal,
    #[serde(rename = "NONE")]
    None,
}

impl Default for ItemClass {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ItemType {
    Gold,
    CardExp,
    Material,
    Diamond,
    DiamondShd,
    HggShd,
    LggShd,
    ExpPlayer,
    PlayerAvatar,
    TktTry,
    TktryRecruit,
    TktInstFin,
    TktGacha,
    TktGacha10,
    SocialPt,
    ApGameplay,
    ApBase,
    TktGachaPrsv,
    LmtgsCoin,
    EpgsCoin,
    RepCoin,
    CrsShopCoin,
    CrsShopCoinV2,
    RetroCoin,
    RenamingCard,
    ApSupply,
    ExterminationAgent,
    LimitedTktGacha10,
    LinkageTktGacha10,
    VoucherPick,
    VoucherLevelmax6,
    VoucherLevelmax5,
    #[serde(rename = "VOUCHER_ELITE_II_6")]
    VoucherEliteIi6,
    #[serde(rename = "VOUCHER_ELITE_II_5")]
    VoucherEliteIi5,
    VoucherSkin,
    VoucherCgacha,
    OptionalVoucherPick,
    ItemPack,
    VoucherMgacha,
    VoucherFullPotential,
    UniCollection,
    ApItem,
    CrsRuneCoin,
    ActivityCoin,
    ActivityItem,
    EtStage,
    RlCoin,
    ReturnCredit,
    Medal,
    ActivityPotential,
    FavorAddItem,
    ClassicShd,
    ClassicTktGacha,
    ClassicTktGacha10,
    LimitedBuff,
    ClassicFesPickTier5,
    ClassicFesPickTier6,
    ReturnProgress,
    NewProgress,
    McardVoucher,
    MaterialIssueVoucher,
    SandboxToken,
    ExclusiveTktGacha,
    ExclusiveTktGacha10,
    #[serde(other)]
    Unknown,
}

impl Default for ItemType {
    fn default() -> Self {
        Self::Material
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ItemOccPer {
    Usual,
    Almost,
    Always,
    Sometimes,
}

impl Default for ItemOccPer {
    fn default() -> Self {
        Self::Usual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum BuildingRoomType {
    Workshop,
    Manufacture,
}

impl Default for BuildingRoomType {
    fn default() -> Self {
        Self::Workshop
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VoucherItemType {
    OptionalVoucherPick,
    MaterialIssueVoucher,
}

impl Default for VoucherItemType {
    fn default() -> Self {
        Self::OptionalVoucherPick
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum VoucherDisplayType {
    None,
    Divide,
}

impl Default for VoucherDisplayType {
    fn default() -> Self {
        Self::None
    }
}

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDrop {
    #[serde(alias = "StageId")]
    pub stage_id: String,
    #[serde(alias = "OccPer")]
    pub occ_per: ItemOccPer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingProduct {
    #[serde(alias = "RoomType")]
    pub room_type: BuildingRoomType,
    #[serde(alias = "FormulaId")]
    pub formula_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoucherRelate {
    #[serde(alias = "VoucherId")]
    pub voucher_id: String,
    #[serde(alias = "VoucherItemType")]
    pub voucher_item_type: VoucherItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniqueItem {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Count")]
    pub count: i32,
    #[serde(rename = "type", alias = "Type_")]
    pub item_type: String, // Almost always just "FURN"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniCollectionInfo {
    #[serde(alias = "UniCollectionItemId")]
    pub uni_collection_item_id: String,
    #[serde(alias = "UniqueItem")]
    pub unique_item: Vec<UniqueItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPackContent {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Count")]
    pub count: i32,
    #[serde(rename = "type", alias = "Type_")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPackInfo {
    #[serde(alias = "PackId")]
    pub pack_id: String,
    #[serde(alias = "Content")]
    pub content: Vec<ItemPackContent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPotentialCharacter {
    #[serde(alias = "ItemId")]
    pub item_id: String,
    #[serde(alias = "Ts")]
    pub ts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPotentialCharacter {
    #[serde(alias = "CharId")]
    pub char_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavorCharacter {
    #[serde(alias = "ItemId")]
    pub item_id: String,
    #[serde(alias = "CharId")]
    pub char_id: String,
    #[serde(alias = "FavorAddAmt")]
    pub favor_add_amt: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpItem {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "GainExp")]
    pub gain_exp: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApSupply {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "Ap")]
    pub ap: i32,
    #[serde(alias = "HasTs")]
    pub has_ts: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharVoucherItem {
    #[serde(alias = "Id")]
    pub id: String,
    #[serde(alias = "DisplayType")]
    pub display_type: VoucherDisplayType,
}

// ============================================================================
// Item
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(alias = "ItemId")]
    pub item_id: String,
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Description")]
    pub description: String,
    #[serde(alias = "Rarity")]
    pub rarity: ItemRarity,
    #[serde(alias = "IconId")]
    pub icon_id: String,
    #[serde(alias = "OverrideBkg")]
    pub override_bkg: Option<String>,
    #[serde(alias = "StackIconId")]
    pub stack_icon_id: Option<String>,
    #[serde(alias = "SortId")]
    pub sort_id: i32,
    #[serde(alias = "Usage")]
    pub usage: String,
    #[serde(alias = "ObtainApproach")]
    pub obtain_approach: Option<String>,
    #[serde(alias = "HideInItemGet")]
    pub hide_in_item_get: bool,
    #[serde(alias = "ClassifyType")]
    pub classify_type: ItemClass,
    #[serde(alias = "ItemType")]
    pub item_type: ItemType,
    #[serde(alias = "StageDropList", default)]
    pub stage_drop_list: Vec<StageDrop>,
    #[serde(alias = "BuildingProductList", default)]
    pub building_product_list: Vec<BuildingProduct>,
    #[serde(alias = "VoucherRelateList")]
    pub voucher_relate_list: Option<Vec<VoucherRelate>>,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Materials {
    pub items: HashMap<String, Item>,
    pub exp_items: HashMap<String, ExpItem>,
    pub potential_items: HashMap<String, HashMap<String, String>>,
    pub ap_supplies: HashMap<String, ApSupply>,
    pub char_voucher_items: HashMap<String, CharVoucherItem>,
}

// ============================================================================
// Table File Wrapper (for loading from FlatBuffer JSON)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemTableFile {
    #[serde(deserialize_with = "deserialize_fb_map")]
    pub items: HashMap<String, Item>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub exp_items: HashMap<String, ExpItem>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub potential_items: HashMap<String, serde_json::Value>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub ap_supply_out_of_date_dict: HashMap<String, ApSupply>,
    #[serde(deserialize_with = "deserialize_fb_map", default)]
    pub char_voucher_items: HashMap<String, CharVoucherItem>,
}
