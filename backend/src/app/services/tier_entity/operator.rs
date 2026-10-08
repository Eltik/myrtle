//! Operators, keyed by char id (`char_002_amiya`).

use std::cmp::Reverse;

use super::{EntitySummary, Facets, KindSource, in_key_order, non_empty, operator_href, wire_name};
use crate::app::services::operators::rarity_to_stars;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::handbook::OperatorRace;
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

/// The profile's race in the operator index's spelling (`Sankta`, `Cautus/Chimera`),
/// `None` with no profile or an `Unknown` one, which the /operators race filter
/// does not offer either.
fn race(op: &Operator) -> Option<String> {
    op.profile
        .as_ref()
        .map(|p| &p.basic_info.race)
        .filter(|r| **r != OperatorRace::Unknown)
        .and_then(wire_name)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::services::tier_entity::FacetValue;
    use crate::core::gamedata::types::handbook::{BasicInfo, OperatorProfile};

    fn one<'a>(s: &'a EntitySummary, key: &str) -> Option<&'a str> {
        match s.facets.get(key)? {
            FacetValue::One(v) => Some(v),
            FacetValue::Many(_) => None,
        }
    }

    fn with_race(race: OperatorRace) -> Operator {
        Operator {
            name: "Exusiai".to_owned(),
            profile: Some(OperatorProfile {
                basic_info: BasicInfo {
                    race,
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn race_is_sent_in_the_operator_index_spelling() {
        let s = summary("char_103_angel", &with_race(OperatorRace::Sankta));
        assert_eq!(one(&s, "race"), Some("Sankta"));
        let s = summary("x", &with_race(OperatorRace::CautusChimera));
        assert_eq!(one(&s, "race"), Some("Cautus/Chimera"));
    }

    #[test]
    fn an_unknown_or_missing_race_sends_no_facet() {
        let s = summary("x", &with_race(OperatorRace::Unknown));
        assert!(!s.facets.contains_key("race"));
        let s = summary("x", &Operator::default());
        assert!(!s.facets.contains_key("race"));
        // Undisclosed is a value the /operators filter offers, so it is sent.
        let s = summary("x", &with_race(OperatorRace::Undisclosed));
        assert_eq!(one(&s, "race"), Some("Undisclosed"));
    }

    #[test]
    fn group_and_team_are_sent_with_their_names() {
        let op = Operator {
            nation_id: "kazimierz".to_owned(),
            group_id: Some("pinus".to_owned()),
            group_name: Some("Pinus Sylvestris".to_owned()),
            team_id: Some(String::new()),
            ..Default::default()
        };
        let s = summary("char_x", &op);
        assert_eq!(one(&s, "nation_id"), Some("kazimierz"));
        assert_eq!(one(&s, "group_id"), Some("pinus"));
        assert_eq!(one(&s, "group_name"), Some("Pinus Sylvestris"));
        // An empty id is no facet, as with `nation_id`.
        assert!(!s.facets.contains_key("team_id"));
        assert!(!s.facets.contains_key("team_name"));
    }
}
