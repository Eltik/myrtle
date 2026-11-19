//! Home-screen backgrounds, keyed by `display_meta_table` `BgId`.

use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata_raw::tables::display_meta::HomeBackground;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, non_empty};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| {
        m.home_backgrounds
            .iter()
            .find(|bg| bg.bg_id == id)
            .map(|bg| summary(&m.assets, bg))
    },
    known: |m, id| m.home_backgrounds.iter().any(|bg| bg.bg_id == id),
    catalogue: |m| {
        m.home_backgrounds
            .iter()
            .map(|bg| summary(&m.assets, bg))
            .collect()
    },
};

fn summary(assets: &AssetIndex, bg: &HomeBackground) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::Background,
        id: bg.bg_id.clone(),
        name: bg.bg_name.clone(),
        icon: assets
            .path(AssetKind::HomeBackground, &bg.bg_id)
            .map(asset_url),
        href: None,
        facets: Facets::default()
            .one(
                "bg_type",
                if bg.bg_type == "VARIABLE" {
                    "dynamic"
                } else {
                    "base"
                },
            )
            .one_opt("obtain", non_empty(&bg.obtain_approach))
            .one_opt(
                "start_time",
                (bg.bg_start_time > 0).then(|| bg.bg_start_time.to_string()),
            )
            .into(),
    }
}
