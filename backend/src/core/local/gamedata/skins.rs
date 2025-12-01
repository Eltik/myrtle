use crate::core::local::types::skin::SkinData;

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
