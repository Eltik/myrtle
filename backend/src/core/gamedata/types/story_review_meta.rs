//! `story_review_meta_table` types: the event ARCHIVE, the "from the archive"
//! screen the game hangs off a finished event (logs, landmarks, news, files,
//! a gallery and a soundtrack).
//!
//! The table is two halves. `ActArchiveData.Components` is the LAYOUT, one
//! entry per act, each naming the item ids that act shows in each slot with
//! the order it shows them in; `ActArchiveResData` is the CONTENT, eight
//! id -> item maps shared by every act. Nothing joins them but the id, so a
//! slot entry whose id is absent from the content map is dropped.
//!
//! On EN the layout carries 15 components over 9 distinct acts: every event
//! is listed twice, live (`act17side`) and retro (`act17sre`), with identical
//! slots, and the 5 `rogue_*` plus `sandbox_1` components name ids
//! `story_review_table` does not list at all, so 267 of the 324 gallery
//! pictures belong to acts the Archives library cannot reach.

use std::collections::HashMap;

use serde::Deserialize;

use super::serde_helpers::{FbKeyValue, deserialize_fb_map_or_default};

// ============================================================================
// Layout (`ActArchiveData`)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoryReviewMetaTableFile {
    #[serde(alias = "ActArchiveData", default)]
    pub act_archive_data: ActArchiveData,
    #[serde(alias = "ActArchiveResData", default)]
    pub act_archive_res_data: ActArchiveResData,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActArchiveData {
    #[serde(
        alias = "Components",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub components: HashMap<String, ActArchiveComponent>,
}

/// One act's slots. Every slot is optional and most acts carry two or three.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActArchiveComponent {
    #[serde(alias = "Avg", default)]
    pub avg: Option<ArchiveSlot>,
    #[serde(alias = "Music", default)]
    pub music: Option<ArchiveSlot>,
    #[serde(alias = "News", default)]
    pub news: Option<ArchiveSlot>,
    #[serde(alias = "Pic", default)]
    pub pic: Option<ArchiveSlot>,
    #[serde(alias = "Story", default)]
    pub story: Option<ArchiveSlot>,
    /// The one slot that is a bare list rather than a `{Xs: [...]}` wrapper.
    #[serde(alias = "Landmark", default)]
    pub landmark: Option<Vec<FbKeyValue<String, ArchiveSlotItem>>>,
    /// Logs are grouped into chapters, and the chapter carries its own name,
    /// icon and unlock line; the log ids inside it are what the content map
    /// answers.
    #[serde(alias = "Log", default)]
    pub log: Option<Vec<FbKeyValue<String, ArchiveLogChapter>>>,
}

/// A slot's items. The wrapper key is the slot's own plural (`Avgs`,
/// `Musics`, `News`, `Pics`, `Stories`), so one alias list reads all five.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchiveSlot {
    #[serde(
        alias = "Avgs",
        alias = "Musics",
        alias = "News",
        alias = "Pics",
        alias = "Stories",
        default
    )]
    pub items: Vec<FbKeyValue<String, ArchiveSlotItem>>,
}

/// One slot entry: an id into the content map and the act's own sort order.
/// Both fields are spelled per slot (`PicId`/`PicSortId`, `MusicId`/
/// `MusicSortId`, ...), never generically, so every spelling is an alias.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchiveSlotItem {
    #[serde(
        alias = "AvgId",
        alias = "MusicId",
        alias = "NewsId",
        alias = "PicId",
        alias = "StoryId",
        alias = "LandmarkId",
        default
    )]
    pub id: String,
    #[serde(
        alias = "AvgSortId",
        alias = "MusicSortId",
        alias = "NewsSortId",
        alias = "PicSortId",
        alias = "StorySortId",
        alias = "LandmarkSortId",
        default
    )]
    pub sort_id: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ArchiveLogChapter {
    #[serde(alias = "ChapterIcon", default)]
    pub chapter_icon: String,
    #[serde(alias = "ChapterName", default)]
    pub chapter_name: String,
    #[serde(alias = "DisplayId", default)]
    pub display_id: String,
    #[serde(alias = "Logs", default)]
    pub logs: Vec<String>,
    /// The source spells it `UnlockDes`, not `UnlockDesc`.
    #[serde(alias = "UnlockDes", alias = "UnlockDesc", default)]
    pub unlock_des: String,
}

// ============================================================================
// Content (`ActArchiveResData`)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ActArchiveResData {
    #[serde(
        alias = "Logs",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub logs: HashMap<String, RawArchiveLog>,
    #[serde(
        alias = "Landmarks",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub landmarks: HashMap<String, RawArchiveLandmark>,
    #[serde(
        alias = "News",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub news: HashMap<String, RawArchiveNews>,
    #[serde(
        alias = "Stories",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub stories: HashMap<String, RawArchiveStory>,
    #[serde(
        alias = "Pics",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub pics: HashMap<String, RawArchivePic>,
    #[serde(
        alias = "Audios",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub audios: HashMap<String, RawArchiveAudio>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveLog {
    #[serde(alias = "LogId", default)]
    pub log_id: String,
    #[serde(alias = "LogDesc", default)]
    pub log_desc: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveLandmark {
    #[serde(alias = "LandmarkId", default)]
    pub landmark_id: String,
    #[serde(alias = "LandmarkName", default)]
    pub landmark_name: String,
    #[serde(alias = "LandmarkEngName", default)]
    pub landmark_eng_name: String,
    #[serde(alias = "LandmarkDesc", default)]
    pub landmark_desc: String,
    /// The picture's stem, which is a file under `textures/ui/`, not a
    /// spritepack: all 26 EN landmarks live in `[uc]deepsea`.
    #[serde(alias = "LandmarkPic", default)]
    pub landmark_pic: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveNews {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// The headline.
    #[serde(alias = "Desc", default)]
    pub desc: String,
    #[serde(alias = "NewsAuthor", default)]
    pub news_author: String,
    #[serde(alias = "NewsType", default)]
    pub news_type: String,
    #[serde(alias = "NewsFormat", default)]
    pub news_format: Option<RawArchiveNewsFormat>,
    /// The body, already split into text and image lines. `NewsText` carries
    /// the same body as one string with `<newsimg>` markers, so it is not
    /// read here.
    #[serde(alias = "NewsLines", default)]
    pub news_lines: Vec<RawArchiveNewsLine>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveNewsFormat {
    #[serde(alias = "TypeId", default)]
    pub type_id: String,
    #[serde(alias = "TypeName", default)]
    pub type_name: String,
    #[serde(alias = "TypeLogo", default)]
    pub type_logo: String,
    #[serde(alias = "TypeMainLogo", default)]
    pub type_main_logo: String,
    /// The masthead's sealing. It is the one format stem with no file of its
    /// own: each value is on disk as `<stem>_bkg` and `<stem>_title`, two
    /// pieces, so nothing is served for it.
    #[serde(alias = "TypeMainSealing", default)]
    pub type_main_sealing: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveNewsLine {
    /// Prose on a `TextContent` line, an image stem on an `ImageContent` one.
    #[serde(alias = "Content", default)]
    pub content: String,
    #[serde(alias = "LineType", default)]
    pub line_type: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveStory {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// The file's title.
    #[serde(alias = "Desc", default)]
    pub desc: String,
    #[serde(alias = "Date", default)]
    pub date: String,
    #[serde(alias = "Text", default)]
    pub text: String,
    #[serde(alias = "Pic", default)]
    pub pic: String,
    #[serde(alias = "TitlePic", default)]
    pub title_pic: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchivePic {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// The picture's title.
    #[serde(alias = "Desc", default)]
    pub desc: String,
    #[serde(alias = "PicDescription", default)]
    pub pic_description: String,
    /// An AVG image stem (`27_i25`, `23_kv`), which resolves through the same
    /// `textures/avg/imgs` tree a script's `[Image]` name does.
    #[serde(alias = "AssetPath", default)]
    pub asset_path: String,
    #[serde(alias = "Type_", alias = "Type", default)]
    pub type_: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawArchiveAudio {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// The track's title. It is the `Musics` row's title too, and blank (a
    /// single space) on the rows that have none.
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Desc", default)]
    pub desc: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_reads_every_slot_shape() {
        let raw = r#"{
            "ActArchiveData": {"Components": [
                {"key": "act17side", "value": {
                    "Landmark": [
                        {"key": "l2", "value": {"LandmarkId": "l2", "LandmarkSortId": 2}},
                        {"key": "l1", "value": {"LandmarkId": "l1", "LandmarkSortId": 1}}
                    ],
                    "Log": [{"key": "chapter_1", "value": {
                        "ChapterIcon": "NORMAL", "ChapterName": "Entering Gran Faro",
                        "DisplayId": "01", "Logs": ["log_1", "log_2"],
                        "UnlockDes": "Keep exploring"
                    }}],
                    "Music": {"Musics": [
                        {"key": "m0", "value": {"MusicId": "m0", "MusicSortId": 1}}
                    ]},
                    "Pic": {"Pics": [
                        {"key": "p0", "value": {"PicId": "p0", "PicSortId": 1}}
                    ]}
                }}
            ]},
            "ActArchiveResData": {
                "Logs": [{"key": "log_1", "value": {"LogId": "log_1", "LogDesc": "a sign"}}],
                "Landmarks": [{"key": "l1", "value": {
                    "LandmarkId": "l1", "LandmarkName": "Tattered Map",
                    "LandmarkEngName": "Tattered Map", "LandmarkDesc": "torn",
                    "LandmarkPic": "act17side_data_1_1_pic"
                }}],
                "Audios": [{"key": "m0", "value": {"Id": "m0", "Name": "Legacy", "Desc": "m0"}}],
                "Pics": [{"key": "p0", "value": {
                    "Id": "p0", "Desc": "Long Night", "PicDescription": "",
                    "AssetPath": "23_kv", "Type_": "IMAGE"
                }}]
            }
        }"#;
        let file: StoryReviewMetaTableFile = serde_json::from_str(raw).unwrap();
        let c = &file.act_archive_data.components["act17side"];
        let landmark = c.landmark.as_ref().unwrap();
        assert_eq!(landmark.len(), 2);
        assert_eq!(landmark[0].value.sort_id, 2);
        let log = c.log.as_ref().unwrap();
        assert_eq!(log[0].value.logs, ["log_1", "log_2"]);
        assert_eq!(log[0].value.unlock_des, "Keep exploring");
        assert_eq!(c.music.as_ref().unwrap().items[0].value.id, "m0");
        assert_eq!(c.pic.as_ref().unwrap().items[0].value.id, "p0");
        assert!(c.story.is_none());
        let res = &file.act_archive_res_data;
        assert_eq!(res.logs["log_1"].log_desc, "a sign");
        assert_eq!(res.landmarks["l1"].landmark_pic, "act17side_data_1_1_pic");
        assert_eq!(res.audios["m0"].name, "Legacy");
        assert_eq!(res.pics["p0"].asset_path, "23_kv");
        assert!(res.news.is_empty());
        assert!(res.stories.is_empty());
    }

    #[test]
    fn a_missing_table_is_an_empty_one() {
        let file: StoryReviewMetaTableFile = serde_json::from_str("{}").unwrap();
        assert!(file.act_archive_data.components.is_empty());
        assert!(file.act_archive_res_data.logs.is_empty());
    }
}
