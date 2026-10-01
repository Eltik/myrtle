//! One operator's skill slot, keyed `{char_id}:{skill_id}`
//! (`char_002_amiya:skchr_amiya_2`). A skill id alone is not an entry,
//! because generic skills (`skcom_*`) are shared.

use std::cmp::Reverse;

use super::class::class_order;
use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, operator_href};
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

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    operator_skill(gd, id).map(|(char_id, op, slot)| summary(assets, char_id, op, slot))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    operator_skill(gd, id).is_some()
}

/// Every slot of every obtainable playable operator, grouped by operator in
/// the operator pool's order, then by slot.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    let mut skills = Vec::new();
    for (char_id, op) in &gd.operators {
        if op.is_not_obtainable || class_order(&op.profession).is_none() {
            continue;
        }
        let stars = Reverse(rarity_to_stars(&op.rarity));
        for slot in 0..op.skills.len() {
            skills.push((
                (stars, op.name.as_str(), char_id.as_str(), slot),
                summary(assets, char_id, op, slot),
            ));
        }
    }
    in_key_order(skills)
}

/// `{char_id}:{skill_id}` -> the operator and the slot holding that skill.
fn operator_skill<'a>(gd: &'a GameData, id: &'a str) -> Option<(&'a str, &'a Operator, usize)> {
    let (char_id, skill_id) = id.split_once(':')?;
    let op = gd.operators.get(char_id)?;
    let slot = op.skills.iter().position(|s| s.skill_id == skill_id)?;
    Some((char_id, op, slot))
}

/// The pool's name for how a skill charges, from `spData.spType`; `None` for
/// a passive.
fn sp_type(raw: &str) -> Option<&'static str> {
    match raw {
        "INCREASE_WITH_TIME" => Some("auto"),
        "INCREASE_WHEN_ATTACK" => Some("offensive"),
        "INCREASE_WHEN_TAKEN_DAMAGE" => Some("defensive"),
        _ => None,
    }
}

fn summary(assets: &AssetIndex, char_id: &str, op: &Operator, slot: usize) -> EntitySummary {
    let skill = &op.skills[slot];
    let static_data = skill.static_data.as_ref();
    // The top level carries the skill's display name and its type.
    let top = static_data.and_then(|s| s.levels.last());
    let icon = static_data
        .and_then(|s| s.icon_id.as_deref())
        .filter(|s| !s.is_empty())
        .and_then(|i| assets.skill_icon_path(i))
        .or_else(|| assets.skill_icon_path(&skill.skill_id))
        .map(asset_url);
    EntitySummary {
        kind: EntityKind::Skill,
        id: format!("{char_id}:{}", skill.skill_id),
        name: top.map_or_else(|| skill.skill_id.clone(), |l| l.name.clone()),
        icon,
        href: Some(operator_href(char_id)),
        facets: Facets::default()
            .owner(char_id, Some(op))
            .one("slot", (slot + 1).to_string())
            .one_opt("skill_type", top.map(|l| l.skill_type.to_ascii_lowercase()))
            .one_opt("sp_type", top.and_then(|l| sp_type(&l.sp_data.sp_type)))
            .into(),
    }
}
