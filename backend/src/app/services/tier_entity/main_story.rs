//! Main story episodes, keyed by `story_review_table` group id (`main_14`).
//! Name, number, act and art are the story library's own: see
//! [`crate::app::services::story::mainline_chapters`].

use super::{EntitySummary, Facets, KindSource, asset_url};
use crate::app::services::story::{MainlineChapter, mainline_chapters};
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |gd, assets, id| {
        mainline_chapters(gd, assets)
            .iter()
            .find(|c| c.group_id == id)
            .map(summary)
    },
    known: |gd, assets, id| {
        mainline_chapters(gd, assets)
            .iter()
            .any(|c| c.group_id == id)
    },
    catalogue: |gd, assets| mainline_chapters(gd, assets).iter().map(summary).collect(),
};

fn summary(c: &MainlineChapter) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::MainStory,
        id: c.group_id.clone(),
        name: c.name.clone(),
        // The 432 px square key visual, the poster the library's card draws.
        icon: c.banner_url.as_deref().map(asset_url),
        // The library opens a chapter in a dialog, not at a route of its own.
        href: None,
        facets: Facets::default()
            .one_opt("episode", c.number.map(|n| n.to_string()))
            .one_opt("act", c.act.as_ref().map(|(i, _)| i.to_string()))
            .one_opt("act_name", c.act.as_ref().map(|(_, name)| name.clone()))
            .into(),
    }
}
