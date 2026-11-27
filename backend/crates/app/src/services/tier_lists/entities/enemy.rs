use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::enemy::Enemy;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, in_key_order, non_empty, wire_name};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| {
        m.enemies
            .enemy_data
            .get(id)
            .map(|e| summary(&m.assets, id, e))
    },
    known: |m, id| m.enemies.enemy_data.contains_key(id),
    catalogue,
};

fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.enemies
            .enemy_data
            .iter()
            .filter(|(_, e)| !e.hide_in_handbook)
            .map(|(id, e)| ((e.sort_id, id), summary(&m.assets, id, e)))
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
