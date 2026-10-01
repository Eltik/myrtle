//! Stronghold Protocol bonds, keyed by bond id.

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::autochess::AutoChessBond;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    gd.autochess_bonds.get(id).map(|b| summary(assets, id, b))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    gd.autochess_bonds.contains_key(id)
}

/// Season (faction) bonds first, then trait bonds, each in the game's order.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.autochess_bonds
            .iter()
            .map(|(id, b)| ((!is_season(b), b.bond_order, id), summary(assets, id, b)))
            .collect(),
    )
}

fn is_season(b: &AutoChessBond) -> bool {
    b.bond_type == "SEASON"
}

fn summary(assets: &AssetIndex, id: &str, b: &AutoChessBond) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::StrongholdBond,
        id: id.to_owned(),
        name: b.name.clone(),
        icon: assets
            .path(AssetKind::AutoChessIcon, &b.icon)
            .map(asset_url),
        href: None,
        facets: Facets::default()
            .one("bond_type", if is_season(b) { "season" } else { "regular" })
            .many("power_ids", &b.power_id_list)
            .into(),
    }
}
