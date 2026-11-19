//! Classes (`WARRIOR`) and subclasses (`charger`), read off the roster's classification.

use myrtle_gamedata::assets::AssetKind;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::OperatorProfession;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};

pub(super) const CLASS: KindSource = KindSource {
    resolve: class_summary,
    known: |_, id| class_of(id).is_some(),
    catalogue: |m| {
        CLASSES
            .iter()
            .filter_map(|(p, _)| class_summary(m, p.to_raw_str()))
            .collect()
    },
};

pub(super) const SUBCLASS: KindSource = KindSource {
    resolve: subclass_summary,
    known: |m, id| subclass_profession(m, id).is_some(),
    catalogue: subclass_catalogue,
};

/// The game's order; the English name is the fallback when the server's tag list lacks one
const CLASSES: [(OperatorProfession, &str); 8] = [
    (OperatorProfession::Vanguard, "Vanguard"),
    (OperatorProfession::Guard, "Guard"),
    (OperatorProfession::Defender, "Defender"),
    (OperatorProfession::Sniper, "Sniper"),
    (OperatorProfession::Caster, "Caster"),
    (OperatorProfession::Medic, "Medic"),
    (OperatorProfession::Supporter, "Supporter"),
    (OperatorProfession::Specialist, "Specialist"),
];

fn class_of(id: &str) -> Option<&'static (OperatorProfession, &'static str)> {
    CLASSES.iter().find(|(p, _)| p.to_raw_str() == id)
}

/// `None` for tokens, traps and unknown professions
pub(super) fn class_order(p: &OperatorProfession) -> Option<usize> {
    CLASSES.iter().position(|(c, _)| c == p)
}

fn class_summary(m: &GameModel, id: &str) -> Option<EntitySummary> {
    let (profession, english) = class_of(id)?;
    let raw = profession.to_raw_str();
    let name = m.profession_names.get(raw).map_or(*english, String::as_str);
    let icon = m
        .assets
        .path(
            AssetKind::ProfessionIcon,
            &format!("icon_profession_{}", raw.to_ascii_lowercase()),
        )
        .map(asset_url);
    Some(EntitySummary {
        kind: EntityKind::Class,
        id: raw.to_owned(),
        name: name.to_owned(),
        icon,
        href: None,
        facets: Facets::default().into(),
    })
}

/// Read off the operators that hold it: `SubProfDict` also names `none1`, `notchar1` and the like
fn subclass_profession<'a>(m: &'a GameModel, id: &str) -> Option<&'a OperatorProfession> {
    if !m.modules.sub_prof_dict.contains_key(id) {
        return None;
    }
    m.operators
        .values()
        .find(|op| op.sub_profession_id == id && class_order(&op.profession).is_some())
        .map(|op| &op.profession)
}

fn subclass_summary(m: &GameModel, id: &str) -> Option<EntitySummary> {
    let profession = subclass_profession(m, id)?;
    let sub = m.modules.sub_prof_dict.get(id)?;
    Some(EntitySummary {
        kind: EntityKind::Subclass,
        id: id.to_owned(),
        name: sub.sub_profession_name.clone(),
        icon: m
            .assets
            .path(AssetKind::SubProfessionIcon, &format!("sub_{id}_icon"))
            .map(asset_url),
        href: None,
        facets: Facets::default()
            .one("profession", profession.to_raw_str())
            .one_opt(
                "profession_name",
                m.profession_names.get(profession.to_raw_str()).cloned(),
            )
            .into(),
    })
}

/// Grouped by class in [`CLASSES`] order, then by name
fn subclass_catalogue(m: &GameModel) -> Vec<EntitySummary> {
    let mut ids: Vec<(usize, &str)> = Vec::new();
    for op in m.operators.values() {
        let Some(order) = class_order(&op.profession) else {
            continue;
        };
        let sub = op.sub_profession_id.as_str();
        if !ids.iter().any(|(_, s)| *s == sub) {
            ids.push((order, sub));
        }
    }
    in_key_order(
        ids.into_iter()
            .filter_map(|(order, id)| {
                let s = subclass_summary(m, id)?;
                Some(((order, s.name.clone()), s))
            })
            .collect(),
    )
}
