//! Outfits, keyed by `skin_table.CharSkins` id (`char_002_amiya@epoque#4`).

use std::cmp::Reverse;

use super::{EntitySummary, Facets, KindSource, in_key_order, operator_href};
use crate::app::services::operators::rarity_to_stars;
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::skin::Skin;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    gd.skins
        .char_skins
        .get(id)
        .map(|skin| summary(gd, assets, id, skin))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    gd.skins.char_skins.contains_key(id)
}

/// Outfits, newest first: the order the store lists them.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.skins
            .char_skins
            .iter()
            .filter(|(id, skin)| is_outfit(gd, id, skin))
            .map(|(id, skin)| {
                (
                    (Reverse(skin.display_skin.get_time), id),
                    summary(gd, assets, id, skin),
                )
            })
            .collect(),
    )
}

/// A skin offered in the pool: a real outfit (`@` in the id: bought, or an
/// event reward), worn by a known operator. The `#1`/`#2` ids are the
/// default and Elite 2 art every operator has, and resolve but are not
/// offered.
fn is_outfit(gd: &GameData, id: &str, skin: &Skin) -> bool {
    id.contains('@') && gd.operators.contains_key(wearer(skin))
}

/// The operator a skin belongs to: Amiya's forms share `char_002_amiya` as
/// `char_id` and name the form in `tmpl_id`.
fn wearer(skin: &Skin) -> &str {
    skin.tmpl_id.as_deref().unwrap_or(&skin.char_id)
}

/// The brand a skin group is filed under (`EPOQUE`), read off the brand
/// list's group ids. Collaboration groups (`2021#rainbow6`) have none.
fn brand<'a>(gd: &'a GameData, group_id: &str) -> Option<&'a str> {
    gd.skins
        .brand_list
        .values()
        .find(|b| b.group_list.iter().any(|g| g.skin_group_id == group_id))
        .map(|b| b.brand_name.as_str())
}

fn summary(gd: &GameData, assets: &AssetIndex, id: &str, skin: &Skin) -> EntitySummary {
    let wearer = wearer(skin);
    let op = gd.operators.get(wearer);
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
        icon: assets
            .path(AssetKind::Avatar, &skin.avatar_id)
            .map(|_| format!("/avatar/{}", skin.avatar_id.replace('#', "%23"))),
        href: op.map(|_| operator_href(wearer)),
        facets: Facets::default()
            .owner(wearer, op)
            .one_opt(
                "rarity",
                op.map(|op| rarity_to_stars(&op.rarity).to_string()),
            )
            .one_opt("brand", brand(gd, &skin.display_skin.skin_group_id))
            .into(),
    }
}
