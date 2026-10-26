use serde::{Deserialize, Serialize};

use super::display_meta::DisplayMetaTableFile;
use super::material::ItemTableFile;
pub use super::obtain::SkinChannel;

#[derive(Debug, Default)]
pub struct ItemTables {
    /// `item_table`
    pub items: ItemTableFile,
    /// `shop_client_table`
    pub shop: ShopTableFile,
    /// `display_meta_table`
    pub display_meta: DisplayMetaTableFile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkinWindow {
    pub skin_id: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CarouselItem {
    #[serde(default)]
    cmd: String,
    #[serde(default)]
    skin_id: String,
    #[serde(default)]
    start_time: i64,
    #[serde(default)]
    end_time: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Carousel {
    #[serde(default)]
    items: Vec<CarouselItem>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RecommendData {
    #[serde(default)]
    cmd: String,
    #[serde(default)]
    skin_id: Option<String>,
    #[serde(default)]
    img_id: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RecommendGroup {
    #[serde(default)]
    data_list: Vec<RecommendData>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NormalSkinParam {
    #[serde(default)]
    skin_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TemplateParam {
    #[serde(default)]
    normal_skin_param: Option<NormalSkinParam>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Recommend {
    #[serde(default)]
    tag_id: String,
    #[serde(default)]
    start_datetime: i64,
    #[serde(default)]
    end_datetime: i64,
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    template_type: Option<String>,
    #[serde(default)]
    template_param: Option<TemplateParam>,
    #[serde(default)]
    group_list: Vec<RecommendGroup>,
}

/// `shop_client_table`
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShopTableFile {
    #[serde(default)]
    carousels: Vec<Carousel>,
    #[serde(default)]
    recommend_list: Vec<Recommend>,
}

/// Fashion Review, identified by name since its listing template changes
const REVIEW_NAMES: &[&str] = &["风尚回顾", "Fashion Review"];

pub fn is_review_name(tag_name: &str) -> bool {
    REVIEW_NAMES.iter().any(|n| tag_name.contains(n))
}

impl ShopTableFile {
    pub fn into_skin_windows(self) -> Vec<SkinWindow> {
        let mut out: Vec<SkinWindow> = self
            .carousels
            .into_iter()
            .flat_map(|c| c.items)
            .filter(|it| it.cmd == "SKINSHOP" && !it.skin_id.is_empty() && it.start_time > 0)
            .map(|it| SkinWindow {
                skin_id: it.skin_id,
                start_time: it.start_time,
                end_time: it.end_time,
            })
            .collect();
        out.sort_by(|a, b| {
            a.skin_id
                .cmp(&b.skin_id)
                .then(a.start_time.cmp(&b.start_time))
                .then(a.end_time.cmp(&b.end_time))
        });
        out.dedup();
        out
    }

    pub fn into_skin_listings(&self) -> Vec<SkinListing> {
        let mut out: Vec<SkinListing> = self
            .recommend_list
            .iter()
            .filter(|r| r.start_datetime > 0)
            .filter_map(|r| {
                let is_skin = r
                    .group_list
                    .iter()
                    .flat_map(|g| g.data_list.iter())
                    .any(|d| d.cmd == "SKINSHOP");
                if !is_skin {
                    return None;
                }
                let kind = if r.template_type.as_deref() == Some("RETURNSKIN")
                    || is_review_name(&r.tag_name)
                {
                    ListingKind::Review
                } else {
                    let mut skin_ids: Vec<String> = r
                        .group_list
                        .iter()
                        .flat_map(|g| g.data_list.iter())
                        .filter_map(|d| d.skin_id.clone())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if let Some(p) = r
                        .template_param
                        .as_ref()
                        .and_then(|t| t.normal_skin_param.as_ref())
                    {
                        skin_ids.extend(p.skin_ids.iter().cloned());
                    }
                    skin_ids.sort();
                    skin_ids.dedup();
                    ListingKind::Group {
                        name: r.tag_name.trim().to_string(),
                        skin_ids,
                        img_id: r
                            .group_list
                            .iter()
                            .flat_map(|g| g.data_list.iter())
                            .find_map(|d| d.img_id.clone())
                            .filter(|i| !i.is_empty()),
                    }
                };
                Some(SkinListing {
                    start_time: r.start_datetime,
                    end_time: r.end_datetime,
                    tag_id: Some(r.tag_id.clone()).filter(|t| !t.is_empty()),
                    kind,
                })
            })
            .collect();
        out.sort_by(|a, b| {
            a.start_time
                .cmp(&b.start_time)
                .then(a.end_time.cmp(&b.end_time))
        });
        out.dedup_by(|a, b| {
            a.start_time == b.start_time && a.end_time == b.end_time && a.kind == b.kind
        });
        out
    }

    /// Recommend entries with a tag and a start
    pub fn recommend_tags(&self) -> Vec<RecommendTag> {
        self.recommend_list
            .iter()
            .filter(|r| !r.tag_id.is_empty() && r.start_datetime > 0)
            .map(|r| RecommendTag {
                tag_id: r.tag_id.clone(),
                start_time: r.start_datetime,
            })
            .collect()
    }
}
