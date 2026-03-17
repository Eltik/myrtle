use std::collections::HashMap;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::skin::{EnrichedSkin, Skin, SkinData, SkinImages},
};

pub fn get_artists(char_id: &str, skins: &SkinData) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    skins
        .char_skins
        .values()
        .filter(|skin| skin.char_id == char_id)
        .flat_map(|skin| &skin.display_skin.drawer_list)
        .filter(|artist| seen.insert((*artist).clone()))
        .cloned()
        .collect()
}

pub fn enrich_all_skins(
    char_skins: &HashMap<String, Skin>,
    assets: &AssetIndex,
) -> HashMap<String, EnrichedSkin> {
    char_skins
        .iter()
        .map(|(id, skin)| {
            let images = get_skin_images(skin, assets);
            let enriched = EnrichedSkin {
                id: id.clone(),
                skin: skin.clone(),
                images,
            };
            (id.clone(), enriched)
        })
        .collect()
}

fn get_skin_images(skin: &Skin, assets: &AssetIndex) -> SkinImages {
    let avatar = assets
        .path(
            crate::core::gamedata::assets::AssetKind::Avatar,
            &skin.avatar_id,
        )
        .map(str::to_owned);

    let portrait = if skin.skin_id.contains('@') || skin.skin_id.contains('#') {
        assets
            .path(
                crate::core::gamedata::assets::AssetKind::SkinPortrait,
                &skin.portrait_id,
            )
            .map(str::to_owned)
    } else {
        assets
            .path(
                crate::core::gamedata::assets::AssetKind::Avatar,
                &skin.portrait_id,
            )
            .map(str::to_owned)
    };

    let skin_url = format_skin_url(&skin.skin_id, &skin.char_id);

    SkinImages {
        avatar: avatar.unwrap_or_default(),
        portrait: portrait.unwrap_or_default(),
        skin: skin_url,
    }
}

fn format_skin_url(skin_id: &str, char_id: &str) -> String {
    if skin_id.contains('@') {
        let formatted = skin_id.replace('@', "_");
        format!("/textures/skinpack/{char_id}/{formatted}.png")
    } else if skin_id.contains('#') {
        let formatted = skin_id.replace('#', "_");
        format!("/textures/chararts/{char_id}/{formatted}.png")
    } else {
        format!("/textures/chararts/{char_id}/{skin_id}.png")
    }
}
