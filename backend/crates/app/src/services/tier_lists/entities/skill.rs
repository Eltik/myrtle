use std::cmp::Reverse;

use myrtle_gamedata::assets::AssetIndex;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::EntityKind;

use super::class::class_order;
use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, operator_href, stars};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| {
        operator_skill(m, id).map(|(char_id, op, slot)| summary(&m.assets, char_id, op, slot))
    },
    known: |m, id| operator_skill(m, id).is_some(),
    catalogue,
};

fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    let mut skills = Vec::new();
    for (char_id, op) in &m.operators {
        if op.is_not_obtainable || class_order(&op.profession).is_none() {
            continue;
        }
        let rank = Reverse(stars(&op.rarity));
        for slot in 0..op.skills.len() {
            skills.push((
                (rank, op.name.as_str(), char_id.as_str(), slot),
                summary(&m.assets, char_id, op, slot),
            ));
        }
    }
    in_key_order(skills)
}

fn operator_skill<'a>(m: &'a GameModel, id: &'a str) -> Option<(&'a str, &'a Operator, usize)> {
    let (char_id, skill_id) = id.split_once(':')?;
    let op = m.operators.get(char_id)?;
    let slot = op.skills.iter().position(|s| s.skill_id == skill_id)?;
    Some((char_id, op, slot))
}

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
