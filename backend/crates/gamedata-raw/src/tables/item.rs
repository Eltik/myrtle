//! Items, shops and constants: `item_table`, shop tables, `gamedata_const`, the
//! derived `derived/event-shops.json` written by the event-shop job.
//!
//! Moves in: `backend/src/core/gamedata/types/{shop.rs (368), event_shop.rs (162)}`.
//! `material.rs`, `obtain.rs` and `consts.rs` went to modules of their own names
//! (lift-2), as files.csv maps them. `event_shop_path`
//! (`event_shop.rs:26`, reads `EVENT_SHOP_DIR`) becomes a `TableSource` concern,
//! not an env read.

#[derive(Debug, Default)]
pub struct ItemTables {}

// ---- Lifted for engine-release (lift pilot, Oct 25 2026). Copied from backend/src/core/gamedata/types/obtain.rs:68-90 and shop.rs:4-13,35-79 (the ShopTableFile parse left),
// ts-rs dropped, utoipa behind the `schema` feature.
use serde::{Deserialize, Serialize};

/// Moved to `obtain.rs` with the rest of `types/obtain.rs` (lift-2); re-exported
/// so `engine-release` keeps its import path.
pub use super::obtain::SkinChannel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkinWindow {
    pub skin_id: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListingKind {
    Group {
        name: String,
        skin_ids: Vec<String>,
        img_id: Option<String>,
    },
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
#[serde(rename_all = "camelCase")]
pub struct SkinListing {
    pub start_time: i64,
    pub end_time: i64,
    /// The recommend entry's `TagId` (`tag_1022`). CN and EN number the
    /// same entry with the same id, so a CN tag EN never carries is a
    /// listing EN skipped; see `release::skipped`.
    #[serde(default)]
    pub tag_id: Option<String>,
    #[serde(flatten)]
    pub kind: ListingKind,
}

/// One recommend-panel entry of any kind (outfits, packs, furniture), by
/// tag and start: the record of how far a server's shop has shipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendTag {
    pub tag_id: String,
    pub start_time: i64,
}

/// The number in `tag_NNNN`; any other spelling is `None`.
pub fn tag_number(tag_id: &str) -> Option<u32> {
    let digits = tag_id.strip_prefix("tag_")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}
