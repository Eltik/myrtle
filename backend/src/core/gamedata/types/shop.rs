use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinWindow {
    pub skin_id: String,
    #[ts(type = "number")]
    pub start_time: i64,
    #[ts(type = "number")]
    pub end_time: i64,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum ListingKind {
    Group {
        name: String,
        skin_ids: Vec<String>,
        img_id: Option<String>,
    },
    Review,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinListing {
    #[ts(type = "number")]
    pub start_time: i64,
    #[ts(type = "number")]
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ShopTableFile {
    #[serde(default)]
    carousels: Vec<Carousel>,
    #[serde(default)]
    recommend_list: Vec<Recommend>,
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
}

/// The Fashion Review (every past outfit back on sale) ran under the plain
/// listing template until 2023-11 and again from 2025-10; its name is the
/// stable signal.
const REVIEW_NAMES: &[&str] = &["风尚回顾", "Fashion Review"];

pub fn is_review_name(tag_name: &str) -> bool {
    REVIEW_NAMES.iter().any(|n| tag_name.contains(n))
}

impl ShopTableFile {
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
        // The tag stays out of the comparison: two entries selling the same
        // thing on the same dates were one listing before tags were read,
        // and the first in table order keeps its tag.
        out.dedup_by(|a, b| {
            a.start_time == b.start_time && a.end_time == b.end_time && a.kind == b.kind
        });
        out
    }

    /// Every recommend entry with a tag and a start, whatever it sells.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_entries_become_listings() {
        let raw = serde_json::json!({
            "Carousels": [],
            "RecommendList": [
                {"TagId": "tag_1022", "StartDatetime": 100, "EndDatetime": 200, "TagName": "Coral Coast/IX", "TemplateType": "DEFAULT",
                 "GroupList": [{"DataList": [{"Cmd": "SKINSHOP", "SkinId": "char_a@summer#9", "Param1": "SS_char_a@summer#9_r5"}]}]},
                {"StartDatetime": 300, "EndDatetime": 400, "TagName": "Rhodes Fashion Review", "TemplateType": "RETURNSKIN",
                 "TemplateParam": {"ReturnSkinParam": {"ShowStartTs": 300, "ShowEndTs": 400}},
                 "GroupList": [{"DataList": [{"Cmd": "SKINSHOP"}]}]},
                {"StartDatetime": 450, "EndDatetime": 460, "TagName": "罗德岛风尚回顾", "TemplateType": "DEFAULT",
                 "GroupList": [{"DataList": [{"Cmd": "SKINSHOP"}]}]},
                {"StartDatetime": 500, "EndDatetime": 600, "TagName": "Test Collection/XIV", "TemplateType": "NORSKIN",
                 "TemplateParam": {"NormalSkinParam": {"SkinIds": ["char_b@sale#13"], "SkinGroupName": "Test Collection/XIV"}},
                 "GroupList": [{"DataList": [{"Cmd": "SKINSHOP"}]}]},
                {"TagId": "tag_1027", "StartDatetime": 700, "EndDatetime": 800, "TagName": "Packs", "GroupList": [{"DataList": [{"Cmd": "GIFTPACKAGE"}]}]}
            ]
        });
        let file: ShopTableFile = serde_json::from_value(raw).unwrap();
        let l = file.into_skin_listings();
        assert_eq!(l.len(), 4);
        assert_eq!(l[0].tag_id.as_deref(), Some("tag_1022"));
        assert_eq!(l[1].tag_id, None, "an entry without a tag");
        assert_eq!(
            file.recommend_tags(),
            vec![
                RecommendTag {
                    tag_id: "tag_1022".into(),
                    start_time: 100
                },
                RecommendTag {
                    tag_id: "tag_1027".into(),
                    start_time: 700
                },
            ],
            "every tagged entry, the gift pack included"
        );
        assert_eq!(
            l[0].kind,
            ListingKind::Group {
                name: "Coral Coast/IX".into(),
                skin_ids: vec!["char_a@summer#9".into()],
                img_id: None,
            }
        );
        assert_eq!(l[1].kind, ListingKind::Review);
        assert_eq!(
            l[2].kind,
            ListingKind::Review,
            "named review under the plain template"
        );
        assert_eq!(
            l[3].kind,
            ListingKind::Group {
                name: "Test Collection/XIV".into(),
                skin_ids: vec!["char_b@sale#13".into()],
                img_id: None,
            }
        );
    }

    #[test]
    fn only_tag_and_digits_is_a_tag_number() {
        assert_eq!(tag_number("tag_1022"), Some(1022));
        assert_eq!(tag_number("tag_0"), Some(0));
        for bad in ["", "tag_", "tag_10a", "tag_-1", "Tag_10", "1022", "tag_ 10"] {
            assert_eq!(tag_number(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn skinshop_windows_are_deduplicated_and_sorted() {
        let raw = serde_json::json!({
            "Carousels": [
                {"Items": [
                    {"Cmd": "SKINSHOP", "SkinId": "char_b@x#1", "StartTime": 200, "EndTime": 300},
                    {"Cmd": "GIFTPACKAGE", "SkinId": "", "StartTime": 1, "EndTime": 2}
                ]},
                {"Items": [
                    {"Cmd": "SKINSHOP", "SkinId": "char_a@x#1", "StartTime": 100, "EndTime": 150},
                    {"Cmd": "SKINSHOP", "SkinId": "char_b@x#1", "StartTime": 200, "EndTime": 300},
                    {"Cmd": "SKINSHOP", "SkinId": "char_b@x#1", "StartTime": 500, "EndTime": 600}
                ]}
            ]
        });
        let file: ShopTableFile = serde_json::from_value(raw).unwrap();
        let windows = file.into_skin_windows();
        assert_eq!(
            windows,
            vec![
                SkinWindow {
                    skin_id: "char_a@x#1".into(),
                    start_time: 100,
                    end_time: 150
                },
                SkinWindow {
                    skin_id: "char_b@x#1".into(),
                    start_time: 200,
                    end_time: 300
                },
                SkinWindow {
                    skin_id: "char_b@x#1".into(),
                    start_time: 500,
                    end_time: 600
                },
            ]
        );
    }
}
