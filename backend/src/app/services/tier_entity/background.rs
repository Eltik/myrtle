//! Home-screen backgrounds, keyed by `display_meta_table` `BgId`.

use super::{EntitySummary, Facets, KindSource, asset_url, non_empty};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::home_background::HomeBackground;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |gd, assets, id| find(gd, id).map(|bg| summary(assets, bg)),
    known: |gd, _, id| find(gd, id).is_some(),
    // The lobby's own order, which `home_backgrounds` already is.
    catalogue: |gd, assets| {
        gd.home_backgrounds
            .iter()
            .map(|bg| summary(assets, bg))
            .collect()
    },
};

fn find<'a>(gd: &'a GameData, id: &str) -> Option<&'a HomeBackground> {
    gd.home_backgrounds.iter().find(|bg| bg.bg_id == id)
}

fn summary(assets: &AssetIndex, bg: &HomeBackground) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::Background,
        id: bg.bg_id.clone(),
        name: bg.bg_name.clone(),
        // The lobby's preview strip, about 3:1, drawn on a wide tile.
        icon: assets
            .path(AssetKind::HomeBackground, &bg.bg_id)
            .map(asset_url),
        // No page of its own: the site has no background route.
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
