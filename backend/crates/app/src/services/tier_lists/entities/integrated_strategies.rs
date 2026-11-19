//! Integrated Strategies themes and items, keyed by `roguelike_topic_table` id.

use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata_raw::tables::roguelike_entries::{RoguelikeEntry, THEME_TYPE};
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.roguelike.entries.get(id).map(|e| summary(&m.assets, e)),
    known: |m, id| m.roguelike.entries.get(id).is_some(),
    catalogue: |m| {
        m.roguelike
            .entries
            .entries
            .iter()
            .map(|e| summary(&m.assets, e))
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
