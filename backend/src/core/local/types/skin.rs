use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Nested Structs
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenSkinMapEntry {
    pub token_id: String,
    pub token_skin_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleSkin {
    pub overwrite_prefab: bool,
    pub skin_or_prefab_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySkin {
    pub skin_name: String,
    pub color_list: Vec<String>,
    pub title_list: Vec<String>,
    pub model_name: String,
    pub drawer_list: Vec<String>,
    pub designer_list: Option<Vec<String>>,
    pub skin_group_id: String,
    pub skin_group_name: String,
    pub skin_group_sort_index: i32,
    pub content: String,
    pub dialog: String,
    pub usage: String,
    pub description: String,
    pub obtain_approach: String,
    pub sort_id: i32,
    pub display_tag_id: Option<String>,
    pub get_time: i64,
    pub on_year: i32,
    pub on_period: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandGroup {
    pub skin_group_id: String,
    pub publish_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandKvImg {
    pub kv_img_id: String,
    pub linked_skin_group_id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub brand_id: String,
    pub group_list: Vec<BrandGroup>,
    pub kv_img_id_list: Vec<BrandKvImg>,
    pub brand_name: String,
    pub brand_capital_name: String,
    pub description: String,
    pub publish_time: i64,
    pub sort_id: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialSkinInfo {
    pub skin_id: String,
    pub start_time: i64,
    pub end_time: i64,
}

// ============================================================================
// Skin
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skin {
    pub skin_id: String,
    pub char_id: String,
    pub token_skin_map: Option<Vec<TokenSkinMapEntry>>,
    pub illust_id: String,
    pub dyn_illust_id: Option<String>,
    pub avatar_id: String,
    pub portrait_id: String,
    pub dyn_portrait_id: Option<String>,
    pub dyn_entrance_id: Option<String>,
    pub building_id: Option<String>,
    pub battle_skin: BattleSkin,
    pub is_buy_skin: bool,
    pub tmpl_id: Option<String>,
    pub voice_id: Option<String>,
    pub voice_type: String,
    pub display_skin: DisplaySkin,
}

// ============================================================================
// Container Types
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinData {
    pub char_skins: HashMap<String, Skin>,
    pub buildin_evolve_map: HashMap<String, HashMap<String, String>>,
    pub buildin_patch_map: HashMap<String, HashMap<String, String>>, // Amiya is special
    pub brand_list: HashMap<String, Brand>,
    pub special_skin_info_list: Vec<SpecialSkinInfo>,
}
