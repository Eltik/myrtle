//! Factions, keyed by `handbook_team_table` power id: nations, groups and teams.

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::handbook_team::{HandbookTeam, NO_FACTION};
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    faction(gd, id).map(|t| summary(assets, id, t))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    faction(gd, id).is_some()
}

/// Nations, then groups, then teams, each in the handbook's own order.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.factions
            .iter()
            .filter(|(id, _)| id.as_str() != NO_FACTION)
            .map(|(id, t)| ((t.power_level, t.order_num, id), summary(assets, id, t)))
            .collect(),
    )
}

fn faction<'a>(gd: &'a GameData, id: &str) -> Option<&'a HandbookTeam> {
    if id == NO_FACTION {
        return None;
    }
    gd.factions.get(id)
}

/// A faction's art: the camp logo, matched case-insensitively (the pack spells
/// `logo_Laterano`), else the tiny team icon (`rainbow`, `laios` and `mujica`
/// have only that on EN 2026-09-05).
fn icon(assets: &AssetIndex, id: &str) -> Option<String> {
    let lower = id.to_ascii_lowercase();
    assets
        .path(AssetKind::CampLogo, &format!("logo_{lower}"))
        .or_else(|| assets.path(AssetKind::TeamIcon, &format!("org_{lower}_tiny")))
        .map(asset_url)
}

fn summary(assets: &AssetIndex, id: &str, t: &HandbookTeam) -> EntitySummary {
    let level = match t.power_level {
        0 => "nation",
        1 => "group",
        _ => "team",
    };
    EntitySummary {
        kind: EntityKind::Faction,
        id: id.to_owned(),
        name: t.power_name.clone(),
        icon: icon(assets, id),
        href: None,
        facets: Facets::default().one("power_level", level).into(),
    }
}
