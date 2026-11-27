use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata_raw::tables::autochess::AutoChessBond;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.autochess_bonds.get(id).map(|b| summary(&m.assets, id, b)),
    known: |m, id| m.autochess_bonds.contains_key(id),
    catalogue: |m| {
        in_key_order(
            m.autochess_bonds
                .iter()
                .map(|(id, b)| ((!is_season(b), b.bond_order, id), summary(&m.assets, id, b)))
                .collect(),
        )
    },
};

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
