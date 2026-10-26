#[derive(Debug, Default)]
pub struct ItemTables {}

use serde::{Deserialize, Serialize};

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
    /// Recommend entry's `TagId`, e.g. `tag_1022`
    #[serde(default)]
    pub tag_id: Option<String>,
    #[serde(flatten)]
    pub kind: ListingKind,
}

/// Recommend-panel entry of any kind (outfits, packs, furniture)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendTag {
    pub tag_id: String,
    pub start_time: i64,
}

/// Number in `tag_NNNN`, `None` for any other spelling
pub fn tag_number(tag_id: &str) -> Option<u32> {
    let digits = tag_id.strip_prefix("tag_")?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}
