//! Classes (professions, `WARRIOR`) and subclasses (sub-professions,
//! `charger`), the two kinds read off the operator roster's classification.

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::OperatorProfession;
use crate::database::models::tier_list::EntityKind;

pub(super) const CLASS: KindSource = KindSource {
    resolve: class_summary,
    known: |_, _, id| class_of(id).is_some(),
    catalogue: |gd, assets| {
        CLASSES
            .iter()
            .filter_map(|(p, _)| class_summary(gd, assets, p.to_raw_str()))
            .collect()
    },
};

pub(super) const SUBCLASS: KindSource = KindSource {
    resolve: subclass_summary,
    known: |gd, _, id| subclass_profession(gd, id).is_some(),
    catalogue: subclass_catalogue,
};

/// The eight playable classes in the game's own order, with the wire id the
/// operator index sends and the English name. The name is only the fallback:
/// a class reads as the server's own gacha tag (`GameData::profession_names`)
/// when the tag list carries it.
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

/// A playable class's place in [`CLASSES`]; `None` for tokens, traps and
/// unknown professions, which no editor ranks.
pub(super) fn class_order(p: &OperatorProfession) -> Option<usize> {
    CLASSES.iter().position(|(c, _)| c == p)
}

fn class_summary(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    let (profession, english) = class_of(id)?;
    let raw = profession.to_raw_str();
    let name = gd
        .profession_names
        .get(raw)
        .map_or(*english, String::as_str);
    let icon = assets
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

/// The class of a playable sub-profession, read off the operators that hold
/// it. `uniequip_table.SubProfDict` also names `none1`, `none2`, `notchar1`
/// and `notchar2`, which no playable operator carries; on EN 2026-09-05 this
/// leaves 71, exactly the keys of `SubProfToProfDict`.
fn subclass_profession<'a>(gd: &'a GameData, id: &str) -> Option<&'a OperatorProfession> {
    if !gd.modules.sub_prof_dict.contains_key(id) {
        return None;
    }
    gd.operators
        .values()
        .find(|op| op.sub_profession_id == id && class_order(&op.profession).is_some())
        .map(|op| &op.profession)
}

fn subclass_summary(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    let profession = subclass_profession(gd, id)?;
    let sub = gd.modules.sub_prof_dict.get(id)?;
    Some(EntitySummary {
        kind: EntityKind::Subclass,
        id: id.to_owned(),
        name: sub.sub_profession_name.clone(),
        icon: assets
            .path(AssetKind::SubProfessionIcon, &format!("sub_{id}_icon"))
            .map(asset_url),
        href: None,
        facets: Facets::default()
            .one("profession", profession.to_raw_str())
            .one_opt(
                "profession_name",
                gd.profession_names.get(profession.to_raw_str()).cloned(),
            )
            .into(),
    })
}

/// Every subclass a playable operator holds, grouped by class in
/// [`CLASSES`] order, then by name.
fn subclass_catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    let mut ids: Vec<(usize, &str)> = Vec::new();
    for op in gd.operators.values() {
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
                let s = subclass_summary(gd, assets, id)?;
                Some(((order, s.name.clone()), s))
            })
            .collect(),
    )
}
