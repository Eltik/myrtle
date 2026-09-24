//! Every `AssetIndex` lookup the story index turns into a SERVED path.
//!
//! One shape answers all three: a kind plus an id, resolved to the
//! `/textures/...` path the frontend fetches under `/api/assets`. The helpers
//! below differ only in which kind they ask for and how they massage the id
//! first, so the lookup itself is written once.
//!
//! This is the `AssetIndex` half, over the game's own classified sprite packs.
//! The reader's OWN tree (`textures/avg`, plus the audio and video walks) is
//! resolved by [`crate::core::story::StoryAssetIndex`], which is a different
//! index over a different tree and shares no body with this one.

use crate::core::gamedata::assets::{AssetIndex, AssetKind};

/// One asset-index lookup as a served path. The id is passed through verbatim:
/// each caller decides what, if anything, to strip from it first.
fn served(assets: &AssetIndex, kind: AssetKind, id: &str) -> Option<String> {
    assets.path(kind, id).map(str::to_owned)
}

/// The operator's avatar sprite by the same `_2`/`_1` fallback the
/// `/api/avatar/{id}` route uses: a handful of operators (Medic Amiya,
/// Closure) ship no bare-id file.
pub(super) fn avatar_for(assets: &AssetIndex, char_id: &str) -> Option<String> {
    ["", "_2", "_1"]
        .into_iter()
        .find_map(|suffix| served(assets, AssetKind::Avatar, &format!("{char_id}{suffix}")))
}

/// The Story Collection art for one id (`kv_*`, `deco_*`, `storyline_abbr_*`,
/// `act_0`), as a served path. One flat namespace, because the seven
/// `mixstory_*` bundles never reuse a stem.
pub(super) fn storyline_art_for(assets: &AssetIndex, id: Option<&str>) -> Option<String> {
    served(assets, AssetKind::StorylineArt, id?.trim())
}

/// A group's Archives entry picture. The table writes the id with a
/// `storyEntryPic_` prefix the sprite pack does not carry.
pub(super) fn cover_for(assets: &AssetIndex, pic_id: Option<&str>) -> Option<String> {
    let pic = pic_id?;
    let key = pic.strip_prefix("storyEntryPic_").unwrap_or(pic);
    served(assets, AssetKind::StoryEntryPic, key)
}
