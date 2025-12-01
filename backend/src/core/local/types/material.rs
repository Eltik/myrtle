use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

// ============================================================================
// Enums
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum ItemRarity {
    Tier1 = 1,
    Tier2 = 2,
    Tier3 = 3,
    Tier4 = 4,
    Tier5 = 5,
    Tier6 = 6,
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
    pub stage_id: String,
    pub occ_per: ItemOccPer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildingProduct {
    pub room_type: BuildingRoomType,
    pub formula_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoucherRelate {
    pub voucher_id: String,
    pub voucher_item_type: VoucherItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniqueItem {
    pub id: String,
    pub count: i32,
    #[serde(rename = "type")]
    pub item_type: String, // Almost always just "FURN"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniCollectionInfo {
    pub uni_collection_item_id: String,
    pub unique_item: Vec<UniqueItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPackContent {
    pub id: String,
    pub count: i32,
    #[serde(rename = "type")]
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemPackInfo {
    pub pack_id: String,
    pub content: Vec<ItemPackContent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullPotentialCharacter {
    pub item_id: String,
    pub ts: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityPotentialCharacter {
    pub char_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavorCharacter {
    pub item_id: String,
    pub char_id: String,
    pub favor_add_amt: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpItem {
    pub id: String,
    pub gain_exp: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApSupply {
    pub id: String,
    pub ap: i32,
    pub has_ts: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharVoucherItem {
    pub id: String,
    pub display_type: VoucherDisplayType,
}

// ============================================================================
// Item
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub item_id: String,
    pub name: String,
    pub description: String,
    pub rarity: ItemRarity,
    pub icon_id: String,
    pub override_bkg: Option<String>,
    pub stack_icon_id: Option<String>,
    pub sort_id: i32,
    pub usage: String,
    pub obtain_approach: Option<String>,
    pub hide_in_item_get: bool,
    pub classify_type: ItemClass,
    pub item_type: ItemType,
    pub stage_drop_list: Vec<StageDrop>,
    pub building_product_list: Vec<BuildingProduct>,
    pub voucher_relate_list: Option<Vec<VoucherRelate>>,
    pub unique_info: HashMap<String, i32>,
    pub item_time_limit: HashMap<String, i64>,
    pub uni_collection_info: HashMap<String, UniCollectionInfo>,
    pub item_pack_infos: HashMap<String, ItemPackInfo>,
    pub full_potential_characters: HashMap<String, FullPotentialCharacter>,
    pub activity_potential_characters: HashMap<String, ActivityPotentialCharacter>,
    pub favor_characters: HashMap<String, FavorCharacter>,
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
