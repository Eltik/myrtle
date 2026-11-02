use myrtle_gamedata_raw::tables::material::Materials;

use crate::assets::{AssetIndex, AssetKind};

pub mod audio;
pub mod chibi;
pub mod enemies;
pub mod enemy_stages;
pub mod gacha;
pub mod handbook;
pub mod modules;
pub mod operators;
pub mod profile;
pub mod stage_class;

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
