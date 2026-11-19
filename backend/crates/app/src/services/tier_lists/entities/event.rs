//! Events, keyed by `activity_table.BasicInfo` id.

use std::cmp::Reverse;

use myrtle_gamedata::assets::AssetKind;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_types::EntityKind;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, non_empty};

pub(super) const SOURCE: KindSource = KindSource {
    resolve: |m, id| m.activities.get(id).map(|info| summary(m, id, info)),
    known: |m, id| m.activities.contains_key(id),
    catalogue,
};

/// Rankable events, newest first
fn catalogue(m: &GameModel) -> Vec<EntitySummary> {
    in_key_order(
        m.activities
            .iter()
            .filter(|(_, info)| is_rankable(info))
            .map(|(id, info)| ((Reverse(info.start_time), id), summary(m, id, info)))
            .collect(),
    )
}

/// Sign-in sheets, login bonuses and shop switches
fn is_filler(activity_type: &str) -> bool {
    activity_type.starts_with("CHECKIN_")
        || matches!(
            activity_type,
            "LOGIN_ONLY"
                | "PRAY_ONLY"
                | "COLLECTION"
                | "SWITCH_ONLY"
                | "RECRUIT_ONLY"
                | "UNIQUE_ONLY"
                | "BLESS_ONLY"
                | "APRIL_FOOL"
        )
}

/// Filed on an Archives shelf or has stages, and not filler
fn is_rankable(info: &ActivityBasicInfo) -> bool {
    ((info.display_type != "NONE" && !info.display_type.is_empty()) || info.has_stage)
        && !is_filler(&info.activity_type)
}

/// Whether the Story Collection holds the key visual and title a banner is made from
fn has_story_art(m: &GameModel, id: &str) -> bool {
    m.storyline_story_sets
        .values()
        .find(|s| s.covers_activity(id))
        .is_some_and(|set| {
            let art = |key: Option<&str>| {
                key.and_then(|k| m.assets.path(AssetKind::StorylineArt, k))
                    .is_some()
            };
            art(set.background_id()) && art(set.title_image_id.as_deref())
        })
}

/// `/event-image` when a banner exists or can be made, else the story group's Archives cover
fn icon(m: &GameModel, id: &str) -> Option<String> {
    if m.assets.event_banner_path(id).is_some() || has_story_art(m, id) {
        return Some(format!("/event-image/{id}"));
    }
    let pic = m.story_reviews.get(id)?.story_entry_pic_id.as_deref()?;
    let key = pic.strip_prefix("storyEntryPic_").unwrap_or(pic);
    m.assets.path(AssetKind::StoryEntryPic, key).map(asset_url)
}

fn summary(m: &GameModel, id: &str, info: &ActivityBasicInfo) -> EntitySummary {
    EntitySummary {
        kind: EntityKind::Event,
        id: id.to_owned(),
        name: info.name.clone(),
        icon: icon(m, id),
        href: None,
        facets: Facets::default()
            .one(
                "display_type",
                non_empty(&info.display_type).unwrap_or("NONE"),
            )
            .one_opt("type", non_empty(&info.activity_type))
            .one("start_time", info.start_time.to_string())
            .one_opt("rerun", info.is_replicate.then_some("true"))
            .into(),
    }
}
