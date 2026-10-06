//! Operators, keyed by char id (`char_002_amiya`).

use std::cmp::Reverse;

use super::{EntitySummary, Facets, KindSource, in_key_order, non_empty, operator_href, wire_name};
use crate::app::services::operators::rarity_to_stars;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::Operator;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, _: &AssetIndex, id: &str) -> Option<EntitySummary> {
    gd.operators.get(id).map(|op| summary(id, op))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    gd.operators.contains_key(id)
}

/// Obtainable operators, rarest first, then by name, then by id (Amiya's
/// forms share a name and rarity, and the map iterates in no fixed order).
fn catalogue(gd: &GameData, _: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.operators
            .iter()
            .filter(|(_, op)| !op.is_not_obtainable)
            .map(|(id, op)| {
                (
                    (Reverse(rarity_to_stars(&op.rarity)), &op.name, id),
                    summary(id, op),
                )
            })
            .collect(),
    )
}

fn summary(id: &str, op: &Operator) -> EntitySummary {
    let facets = Facets::default()
        .one("rarity", rarity_to_stars(&op.rarity).to_string())
        .one_opt("profession", wire_name(&op.profession))
        .one("sub_profession_id", op.sub_profession_id.clone())
        .one_opt("position", wire_name(&op.position))
        .one_opt("nation_id", non_empty(&op.nation_id))
        .one_opt("profession_name", op.profession_name.clone())
        .one_opt("sub_profession_name", op.sub_profession_name.clone())
        .one_opt("nation_name", op.nation_name.clone())
        .one_opt("appellation", non_empty(&op.appellation));
    EntitySummary {
        kind: EntityKind::Operator,
        id: id.to_owned(),
        name: op.name.clone(),
        icon: Some(format!("/avatar/{id}")),
        href: Some(operator_href(id)),
        facets: facets.into(),
    }
}
