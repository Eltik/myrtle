//! Outfits, keyed by `skin_table.CharSkins` id (`char_002_amiya@epoque#4`).

use std::cmp::Reverse;

use myrtle_gamedata::assets::AssetKind;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::character::Skin;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, in_key_order, operator_href, stars};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.skins.char_skins.get(id).map(|skin| summary(m, id, skin)),
    known: |m, id| m.skins.char_skins.contains_key(id),
    catalogue,
};

/// Newest first, the store's order
fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.skins
            .char_skins
            .iter()
            .filter(|(id, skin)| is_outfit(m, id, skin))
            .map(|(id, skin)| {
                (
                    (Reverse(skin.display_skin.get_time), id),
                    summary(m, id, skin),
                )
            })
            .collect(),
    )
}

/// An `@` id worn by a known operator; the `#1`/`#2` default arts resolve but are not offered
fn is_outfit(m: &GameModel, id: &str, skin: &Skin) -> bool {
    id.contains('@') && m.operators.contains_key(wearer(skin))
}

/// Amiya's forms share `char_id` and name the form in `tmpl_id`
fn wearer(skin: &Skin) -> &str {
    skin.tmpl_id.as_deref().unwrap_or(&skin.char_id)
}

/// Collaboration groups (`2021#rainbow6`) have none
fn brand<'a>(m: &'a GameModel, group_id: &str) -> Option<&'a str> {
    m.skins
        .brand_list
        .values()
        .find(|b| b.group_list.iter().any(|g| g.skin_group_id == group_id))
        .map(|b| b.brand_name.as_str())
}

fn summary(m: &GameModel, id: &str, skin: &Skin) -> EntitySummary {
    let wearer = wearer(skin);
    let op = m.operators.get(wearer);
    let name = skin
        .display_skin
        .skin_name
        .clone()
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| id.to_owned());
    EntitySummary {
        kind: EntityKind::Skin,
        id: id.to_owned(),
        name,
        icon: m
            .assets
            .path(AssetKind::Avatar, &skin.avatar_id)
            .map(|_| format!("/avatar/{}", skin.avatar_id.replace('#', "%23"))),
        href: op.map(|_| operator_href(wearer)),
        facets: Facets::default()
            .owner(wearer, op)
            .one_opt("rarity", op.map(|op| stars(&op.rarity).to_string()))
            .one_opt("brand", brand(m, &skin.display_skin.skin_group_id))
            .into(),
    }
}
