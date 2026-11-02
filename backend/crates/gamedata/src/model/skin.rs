use std::collections::HashMap;

use myrtle_gamedata_raw::tables::character::{Brand, Skin, SkinTableFile, SpecialSkinInfo};
use myrtle_gamedata_raw::tables::obtain::SkinChannel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinData {
    pub char_skins: HashMap<String, Skin>,
    pub buildin_evolve_map: HashMap<String, HashMap<String, String>>,
    pub buildin_patch_map: HashMap<String, HashMap<String, String>>, // Amiya is special
    pub brand_list: HashMap<String, Brand>,
    pub special_skin_info_list: Vec<SpecialSkinInfo>,

    #[serde(skip)]
    pub enriched_skins: HashMap<String, EnrichedSkin>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinImages {
    pub avatar: String,
    pub portrait: String,
    pub skin: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedSkin {
    pub id: String,
    #[serde(flatten)]
    pub skin: Skin,
    pub images: SkinImages,
}

#[must_use]
pub fn skin_data(mut file: SkinTableFile) -> SkinData {
    for skin in file.char_skins.values_mut() {
        skin.obtain_channel = SkinChannel::classify(skin.display_skin.display_tag_id.as_deref());
    }
    SkinData {
        char_skins: file.char_skins,
        buildin_evolve_map: HashMap::new(),
        buildin_patch_map: HashMap::new(),
        brand_list: file.brand_list,
        special_skin_info_list: file.special_skin_info_list,
        enriched_skins: HashMap::new(),
    }
}
