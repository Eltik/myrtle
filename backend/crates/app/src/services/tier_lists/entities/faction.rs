//! Factions, keyed by `handbook_team_table` power id: nations, groups and teams.

use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::handbook_team::{HandbookTeam, NO_FACTION};
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| faction(m, id).map(|t| summary(&m.assets, id, t)),
    known: |m, id| faction(m, id).is_some(),
    catalogue,
};

/// Nations, then groups, then teams, each in the handbook's order
fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.factions
            .iter()
            .filter(|(id, _)| id.as_str() != NO_FACTION)
            .map(|(id, t)| ((t.power_level, t.order_num, id), summary(&m.assets, id, t)))
            .collect(),
    )
}

fn faction<'a>(m: &'a GameModel, id: &str) -> Option<&'a HandbookTeam> {
    if id == NO_FACTION {
        return None;
    }
    m.factions.get(id)
}

/// The camp logo, matched lowercase (the pack spells `logo_Laterano`), else the tiny team icon
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
