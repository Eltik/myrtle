use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    types::enemy::{Enemy, EnemyHandbook},
};

/// Enrich enemies with asset paths (images, etc.)
/// Currently a placeholder - can be extended to add image paths when available
pub fn enrich_all_enemies(
    handbook: &EnemyHandbook,
    _asset_mappings: &AssetMappings,
) -> HashMap<String, Enemy> {
    handbook
        .enemy_data
        .iter()
        .map(|(id, enemy)| {
            let enriched = enemy.clone();
            // TODO: Add image paths when asset mapping for enemies is implemented
            // See `/assets/Unpacked/upk/spritepack/icon_enemies<num>` for enemy icons
            // enriched.image = asset_mappings.get_enemy_image_path(&enemy.enemy_id);
            (id.clone(), enriched)
        })
        .collect()
}
