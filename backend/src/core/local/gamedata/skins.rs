use std::collections::HashMap;

use crate::core::local::types::skin::{EnrichedSkin, Skin, SkinData, SkinImages};

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
pub fn get_skin_images(skin: &Skin, char_id: &str) -> SkinImages {
    // Avatar - using spritepack (you may need to adjust based on your actual directory)
    let avatar = format!("/spritepack/{}.png", skin.avatar_id);

    // Portrait - using chararts directory
    let portrait = format!("/chararts/{}/{}.png", char_id, skin.portrait_id);

    // Skin image - depends on skin type
    let skin_url = format_skin_url(&skin.skin_id, char_id);

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
        // Special skin: char_002_amiya@epoque#4 → /skinpack/char_002_amiya/char_002_amiya_epoque#4.png
        let formatted = skin_id.replace('@', "_");
        format!("/skinpack/{}/{}.png", char_id, formatted)
    } else if skin_id.contains('#') {
        // Base skin: char_285_medic2#1 → /chararts/char_285_medic2/char_285_medic2_1.png
        let formatted = skin_id.replace('#', "_");
        format!("/chararts/{}/{}.png", char_id, formatted)
    } else {
        // Fallback (shouldn't happen)
        format!("/chararts/{}/{}.png", char_id, skin_id)
    }
}

/// Enrich all skins with image URLs
pub fn enrich_all_skins(char_skins: &HashMap<String, Skin>) -> HashMap<String, EnrichedSkin> {
    char_skins
        .iter()
        .map(|(id, skin)| {
            let enriched = EnrichedSkin {
                id: id.clone(),
                skin: skin.clone(),
                images: get_skin_images(skin, &skin.char_id),
            };
            (id.clone(), enriched)
        })
        .collect()
}

/// Get all skins for a specific character, enriched with images
pub fn get_character_skins(char_id: &str, skins: &SkinData) -> Vec<EnrichedSkin> {
    skins
        .char_skins
        .iter()
        .filter(|(_, skin)| skin.char_id == char_id)
        .map(|(id, skin)| EnrichedSkin {
            id: id.clone(),
            skin: skin.clone(),
            images: get_skin_images(skin, &skin.char_id),
        })
        .collect()
}
