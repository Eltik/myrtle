use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    types::skin::{EnrichedSkin, Skin, SkinData, SkinImages},
};

pub fn get_artists(char_id: &str, skins: &SkinData) -> Vec<String> {
    let mut artists: Vec<String> = skins
        .char_skins
        .values()
        .filter(|skin| skin.char_id == char_id)
        .flat_map(|skin| skin.display_skin.drawer_list.clone())
        .collect();

    let mut seen = std::collections::HashSet::new();
    artists.retain(|artist| seen.insert(artist.clone()));

    artists
}

/// Generate image paths for a skin using local assets
pub fn get_skin_images(skin: &Skin, mappings: &AssetMappings) -> SkinImages {
    let avatar = mappings.get_avatar_path(&skin.avatar_id);

    // For portrait, check if it's a skin (has @ or #) or default
    let portrait = if skin.skin_id.contains('@') || skin.skin_id.contains('#') {
        mappings.get_skin_portrait_path(&skin.portrait_id)
    } else {
        mappings.get_avatar_path(&skin.portrait_id)
    };

    // Skin image path (chararts/skinpack)
    let skin_url = format_skin_url(&skin.skin_id, &skin.char_id);

    SkinImages {
        avatar,
        portrait,
        skin: skin_url,
    }
}

/// Format skin URL based on skinId pattern
/// - Contains `@`: Special skin in skinpack (replace @ with _)
/// - Only has `#`: Base skin in chararts (replace # with _)
fn format_skin_url(skin_id: &str, char_id: &str) -> String {
    if skin_id.contains('@') {
        // Special skin: char_002_amiya@epoque#4 → /upk/skinpack/char_002_amiya/char_002_amiya_epoque#4.png
        let formatted = skin_id.replace('@', "_");
        format!("/upk/skinpack/{char_id}/{formatted}.png")
    } else if skin_id.contains('#') {
        // Base skin: char_285_medic2#1 → /upk/chararts/char_285_medic2/char_285_medic2_1.png
        let formatted = skin_id.replace('#', "_");
        format!("/upk/chararts/{char_id}/{formatted}.png")
    } else {
        // Fallback (shouldn't happen)
        format!("/upk/chararts/{char_id}/{skin_id}.png")
    }
}

/// Enrich all skins with image URLs
pub fn enrich_all_skins(
    char_skins: &HashMap<String, Skin>,
    mappings: &AssetMappings,
) -> HashMap<String, EnrichedSkin> {
    char_skins
        .iter()
        .map(|(id, skin)| {
            let enriched = EnrichedSkin {
                id: id.clone(),
                skin: skin.clone(),
                images: get_skin_images(skin, mappings),
            };
            (id.clone(), enriched)
        })
        .collect()
}

/// Get all skins for a specific character, using pre-enriched data
pub fn get_character_skins<'a>(char_id: &str, skins: &'a SkinData) -> Vec<&'a EnrichedSkin> {
    skins
        .enriched_skins
        .values()
        .filter(|enriched| enriched.skin.char_id == char_id)
        .collect()
}
