//! Events, keyed by `activity_table.BasicInfo` id.

use std::cmp::Reverse;

use super::{EntitySummary, Facets, KindSource, asset_url, in_key_order, non_empty};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::activity::ActivityBasicInfo;
use crate::database::models::tier_list::EntityKind;

pub(super) const SOURCE: KindSource = KindSource {
    resolve,
    known,
    catalogue,
};

fn resolve(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<EntitySummary> {
    gd.activities
        .get(id)
        .map(|info| summary(gd, assets, id, info))
}

fn known(gd: &GameData, _: &AssetIndex, id: &str) -> bool {
    gd.activities.contains_key(id)
}

/// Rankable events, newest first: the order a reader remembers them in.
fn catalogue(gd: &GameData, assets: &AssetIndex) -> Vec<EntitySummary> {
    in_key_order(
        gd.activities
            .iter()
            .filter(|(_, info)| is_rankable(info))
            .map(|(id, info)| {
                (
                    (Reverse(info.start_time), id),
                    summary(gd, assets, id, info),
                )
            })
            .collect(),
    )
}

/// Activity types that are a sign-in sheet, a login bonus or a shop switch,
/// not an event anyone ranks. None of them carries a display type or a stage
/// on EN 2026-09-05, so the rule drops nothing there today; it is the guard
/// for when one does.
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

/// Whether an activity is offered in the event pool: it is filed on an
/// Archives shelf or has stages, and it is not filler.
fn is_rankable(info: &ActivityBasicInfo) -> bool {
    ((info.display_type != "NONE" && !info.display_type.is_empty()) || info.has_stage)
        && !is_filler(&info.activity_type)
}

/// The event's art: its banner by activity id, else the Archives cover its
/// story group names. Grani's `1stact` files its cover as `act1d0`, so the
/// activity-id lookup alone misses it.
fn icon(gd: &GameData, assets: &AssetIndex, id: &str) -> Option<String> {
    if assets.event_banner_path(id).is_some() {
        return Some(format!("/event-image/{id}"));
    }
    let pic = gd.story_reviews.get(id)?.story_entry_pic_id.as_deref()?;
    let key = pic.strip_prefix("storyEntryPic_").unwrap_or(pic);
    assets.path(AssetKind::StoryEntryPic, key).map(asset_url)
}

fn summary(
    gd: &GameData,
    assets: &AssetIndex,
    id: &str,
    info: &ActivityBasicInfo,
) -> EntitySummary {
    let display = non_empty(&info.display_type).unwrap_or("NONE");
    EntitySummary {
        kind: EntityKind::Event,
        id: id.to_owned(),
        name: info.name.clone(),
        icon: icon(gd, assets, id),
        // No page of its own: the site has no event route.
        href: None,
        facets: Facets::default()
            .one("display_type", display)
            .one_opt("type", non_empty(&info.activity_type))
            .one("start_time", info.start_time.to_string())
            .one_opt("rerun", info.is_replicate.then_some("true"))
            .into(),
    }
}
