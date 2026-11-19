//! Operators, keyed by char id (`char_002_amiya`).

use std::cmp::Reverse;

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::handbook::OperatorRace;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::EntityKind;

use super::{
    EntitySummary, Facets, KindSource, in_key_order, non_empty, operator_href, stars, wire_name,
};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.operators.get(id).map(|op| summary(id, op)),
    known: |m, id| m.operators.contains_key(id),
    catalogue,
};

/// Obtainable, rarest first, then name, then id (Amiya's forms share a name and rarity)
fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.operators
            .iter()
            .filter(|(_, op)| !op.is_not_obtainable)
            .map(|(id, op)| ((Reverse(stars(&op.rarity)), &op.name, id), summary(id, op)))
            .collect(),
    )
}

/// The operator index's spelling; `Unknown` sends nothing, as the race filter offers no such value
fn race(op: &Operator) -> Option<String> {
    op.profile
        .as_ref()
        .map(|p| &p.basic_info.race)
        .filter(|r| **r != OperatorRace::Unknown)
        .and_then(wire_name)
}

fn summary(id: &str, op: &Operator) -> EntitySummary {
    let facets = Facets::default()
        .one("rarity", stars(&op.rarity).to_string())
        .one_opt("profession", wire_name(&op.profession))
        .one("sub_profession_id", op.sub_profession_id.clone())
        .one_opt("position", wire_name(&op.position))
        .one_opt("nation_id", non_empty(&op.nation_id))
        .one_opt("profession_name", op.profession_name.clone())
        .one_opt("sub_profession_name", op.sub_profession_name.clone())
        .one_opt("nation_name", op.nation_name.clone())
        .one_opt("group_id", op.group_id.as_deref().and_then(non_empty))
        .one_opt("group_name", op.group_name.clone())
        .one_opt("team_id", op.team_id.as_deref().and_then(non_empty))
        .one_opt("team_name", op.team_name.clone())
        .one_opt("race", race(op))
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
