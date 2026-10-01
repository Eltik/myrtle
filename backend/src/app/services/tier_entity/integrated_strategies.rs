//! Integrated Strategies themes and items, keyed by `roguelike_topic_table`
//! id. Which items are offered, and in what order, is decided where the table
//! is read: see [`crate::core::gamedata::types::roguelike_entries`].

use super::{EntitySummary, Facets, KindSource, asset_url};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::roguelike_entries::{RoguelikeEntry, THEME_TYPE};
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |gd, assets, id| gd.roguelike.entries.get(id).map(|e| summary(assets, e)),
    known: |gd, _, id| gd.roguelike.entries.get(id).is_some(),
    catalogue: |gd, assets| {
        gd.roguelike
            .entries
            .entries
            .iter()
            .map(|e| summary(assets, e))
            .collect()
    },
};

fn summary(assets: &AssetIndex, e: &RoguelikeEntry) -> EntitySummary {
    let kind = if e.item_type == THEME_TYPE {
        AssetKind::RoguelikeTheme
    } else {
        AssetKind::RoguelikeItem
    };
    let icon = e
        .icon_stems
        .iter()
        .find_map(|stem| assets.path(kind, stem))
        .map(asset_url);
    EntitySummary {
        kind: EntityKind::IntegratedStrategies,
        id: e.id.clone(),
        name: e.name.clone(),
        icon,
        href: None,
        facets: Facets::default()
            .one("theme", e.theme_id.clone())
            .one("item_type", e.item_type.to_ascii_lowercase())
            .into(),
    }
}
