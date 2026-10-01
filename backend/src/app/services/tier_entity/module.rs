//! Operator modules, keyed by `uniequip_table.EquipDict` id.

use std::cmp::Reverse;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, non_empty, operator_href};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::Module;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    module(gd, id).map(|m| summary(gd, assets, id, m))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    module(gd, id).is_some()
}

/// Real modules of obtainable operators, newest first.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.modules
            .equip_dict
            .iter()
            .filter(|(id, m)| {
                module(gd, id).is_some()
                    && gd
                        .operators
                        .get(&m.char_id)
                        .is_some_and(|op| !op.is_not_obtainable)
            })
            .map(|(id, m)| {
                (
                    (Reverse(m.uni_equip_get_time), id),
                    summary(gd, assets, id, m),
                )
            })
            .collect(),
    )
}

/// A real module: not the ORIGINAL placeholder every moduled operator has.
fn module<'a>(gd: &'a GameData, id: &str) -> Option<&'a Module> {
    gd.modules
        .equip_dict
        .get(id)
        .filter(|m| m.type_icon != "original" && !m.uni_equip_id.starts_with("uniequip_001_"))
}

fn summary(gd: &GameData, assets: &AssetIndex, id: &str, m: &Module) -> EntitySummary {
    let op = gd.operators.get(&m.char_id);
    // The module's own art, the picture the operator page shows; the type
    // badge (shared by every module of a subclass and letter) only when the
    // extract lacks it.
    let icon = assets
        .module_big_path(&m.uni_equip_icon)
        .or_else(|| assets.path(AssetKind::ModuleType, &m.type_icon))
        .map(asset_url);
    EntitySummary {
        kind: EntityKind::Module,
        id: id.to_owned(),
        name: m.uni_equip_name.clone(),
        icon,
        href: op.map(|_| operator_href(&m.char_id)),
        facets: Facets::default()
            .owner(&m.char_id, op)
            .one_opt("module_type", m.type_name2.as_deref().and_then(non_empty))
            .one_opt("type_code", non_empty(&m.type_name1))
            .into(),
    }
}
