use std::cmp::Reverse;

use myrtle_gamedata::assets::AssetKind;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::Module;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, non_empty, operator_href};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| module(m, id).map(|module| summary(m, id, module)),
    known: |m, id| module(m, id).is_some(),
    catalogue,
};

fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.modules
            .equip_dict
            .iter()
            .filter(|(id, module_row)| {
                module(m, id).is_some()
                    && m.operators
                        .get(&module_row.char_id)
                        .is_some_and(|op| !op.is_not_obtainable)
            })
            .map(|(id, module_row)| {
                (
                    (Reverse(module_row.uni_equip_get_time), id),
                    summary(m, id, module_row),
                )
            })
            .collect(),
    )
}

fn module<'a>(m: &'a GameModel, id: &str) -> Option<&'a Module> {
    m.modules
        .equip_dict
        .get(id)
        .filter(|row| row.type_icon != "original" && !row.uni_equip_id.starts_with("uniequip_001_"))
}

fn summary(m: &GameModel, id: &str, module: &Module) -> EntitySummary {
    let op = m.operators.get(&module.char_id);
    let icon = m
        .assets
        .module_big_path(&module.uni_equip_icon)
        .or_else(|| m.assets.path(AssetKind::ModuleType, &module.type_icon))
        .map(asset_url);
    EntitySummary {
        kind: EntityKind::Module,
        id: id.to_owned(),
        name: module.uni_equip_name.clone(),
        icon,
        href: op.map(|_| operator_href(&module.char_id)),
        facets: Facets::default()
            .owner(&module.char_id, op)
            .one_opt(
                "module_type",
                module.type_name2.as_deref().and_then(non_empty),
            )
            .one_opt("type_code", non_empty(&module.type_name1))
            .into(),
    }
}
