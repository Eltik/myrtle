//! Enemies, keyed by `enemy_handbook_table` id.

use super::{EntitySummary, Facets, KindSource, in_key_order, non_empty, wire_name};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::enemy::Enemy;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    gd.enemies
        .enemy_data
        .get(id)
        .map(|e| summary(assets, id, e))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    gd.enemies.enemy_data.contains_key(id)
}

/// Enemies the handbook shows, in its own order.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.enemies
            .enemy_data
            .iter()
            .filter(|(_, e)| !e.hide_in_handbook)
            .map(|(id, e)| ((e.sort_id, id), summary(assets, id, e)))
            .collect(),
    )
}

fn summary(assets: &AssetIndex, id: &str, e: &Enemy) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::Enemy,
        id: id.to_owned(),
        name: e.name.clone(),
        icon: assets
            .path(AssetKind::EnemyIcon, id)
            .map(|_| format!("/enemy-icon/{id}")),
        href: Some(format!("/enemies/{id}")),
        facets: Facets::default()
            .one_opt("enemy_level", wire_name(&e.enemy_level))
            .one_opt("enemy_index", non_empty(&e.enemy_index))
            .into(),
    }
}
