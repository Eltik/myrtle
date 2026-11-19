//! Main story episodes, keyed by `story_review_table` group id (`main_14`).

use myrtle_engine_story::index::{MainlineChapter, mainline_chapters};
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| {
        mainline_chapters(m, &m.assets)
            .iter()
            .find(|c| c.group_id == id)
            .map(summary)
    },
    known: |m, id| {
        mainline_chapters(m, &m.assets)
            .iter()
            .any(|c| c.group_id == id)
    },
    catalogue: |m| {
        mainline_chapters(m, &m.assets)
            .iter()
            .map(summary)
            .collect()
    },
};

fn summary(c: &MainlineChapter) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::MainStory,
        id: c.group_id.clone(),
        name: c.name.clone(),
        icon: c.banner_url.as_deref().map(asset_url),
        href: None, // the library opens a chapter in a dialog, not at a route
        facets: Facets::default()
            .one_opt("episode", c.number.map(|n| n.to_string()))
            .one_opt("act", c.act.as_ref().map(|(i, _)| i.to_string()))
            .one_opt("act_name", c.act.as_ref().map(|(_, name)| name.clone()))
            .into(),
    }
}
