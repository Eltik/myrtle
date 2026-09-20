use crate::core::gamedata::assets::AssetKind;
use crate::core::gamedata::{assets::AssetIndex, types::material::Materials};

pub mod audio;
pub mod chibi;
pub mod enemies;
pub mod enemy_stages;
pub mod gacha;
pub mod handbook;
pub mod modules;
pub mod operators;
pub mod profile;
pub mod skills;
pub mod skins;
pub mod stage_class;
pub mod stage_index;
pub mod voice;

pub fn resolve_item_icon(
    item_id: &str,
    materials: &Materials,
    assets: &AssetIndex,
) -> (Option<String>, Option<String>) {
    if let Some(item) = materials.items.get(item_id) {
        let path = assets.path(AssetKind::ItemIcon, &item.icon_id);
        return (Some(item.icon_id.clone()), path.map(str::to_owned));
    }

    if materials.exp_items.contains_key(item_id) {
        let path = assets.path(AssetKind::ItemIcon, item_id);
        return (Some(item_id.to_owned()), path.map(str::to_owned));
    }

    let path = assets.path(AssetKind::ItemIcon, item_id);
    (Some(item_id.to_owned()), path.map(str::to_owned))
}
