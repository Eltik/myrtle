use crate::core::gamedata::{assets::AssetIndex, types::material::Materials};

pub mod chibi;
pub mod enemies;
pub mod handbook;
pub mod modules;
pub mod operators;
pub mod profile;
pub mod skills;
pub mod skins;
pub mod voice;

pub fn resolve_item_icon(
    item_id: &str,
    materials: &Materials,
    assets: &AssetIndex,
) -> (Option<String>, Option<String>) {
    // Try items table first
    if let Some(item) = materials.items.get(item_id) {
        let path = assets.path(
            crate::core::gamedata::assets::AssetKind::ItemIcon,
            &item.icon_id,
        );
        return (Some(item.icon_id.clone()), path.map(str::to_owned));
    }

    // Try exp_items table
    if materials.exp_items.contains_key(item_id) {
        let path = assets.path(crate::core::gamedata::assets::AssetKind::ItemIcon, item_id);
        return (Some(item_id.to_owned()), path.map(str::to_owned));
    }

    // Fallback: use item_id as icon_id
    let path = assets.path(crate::core::gamedata::assets::AssetKind::ItemIcon, item_id);
    (Some(item_id.to_owned()), path.map(str::to_owned))
}
