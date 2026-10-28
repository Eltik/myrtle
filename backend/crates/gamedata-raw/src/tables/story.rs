use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Error;
use serde_json::Value;
use serde_json::from_slice;

use crate::serde_helpers::{FbKeyValue, deserialize_fb_map_or_default};

#[derive(Debug, Default)]
pub struct StoryTables {
    /// `story_review_table`
    pub review: StoryReviewTableFile,
    /// `story_review_meta_table`
    pub review_meta: StoryReviewMetaTableFile,
    /// `gamedata/story/**/*.txt`, the scripts that read as UTF-8
    pub scripts: Vec<String>,
    /// `textures/avg/characters/`, the story sprite sets
    pub sprites: StorySpriteFiles,
}

/// `textures/avg/characters/`
pub const AVG_CHARACTERS: &str = "textures/avg/characters/";

/// Bytes a PNG header needs to give the image size
pub const PNG_HEAD_LEN: usize = 24;

/// What the story sprite index reads off the sprite sets beside their paths
#[derive(Debug, Default)]
pub struct StorySpriteFiles {
    /// Folder -> its `hub.json`, the ones that parse
    pub hubs: HashMap<String, Value>,
    /// `textures/avg/characters/{folder}/{stem}.png` -> its first [`PNG_HEAD_LEN`] bytes
    pub png_heads: HashMap<String, Vec<u8>>,
}

/// `gamedata/story/story_variables.json.json`, `$name` -> value table scripts
/// reference (e.g. `[PlayMusic(intro="$escape_intro", key="$escape_loop")]`).
/// Values are audio paths, avatar ids or numbers, all kept as strings
#[derive(Debug, Default, Clone)]
pub struct StoryVariables {
    map: HashMap<String, String>,
}

impl StoryVariables {
    #[must_use]
    pub fn path(server_assets_dir: &Path) -> PathBuf {
        server_assets_dir.join("gamedata/story/story_variables.json.json")
    }

    pub fn parse(raw: &[u8]) -> Result<Self, Error> {
        let parsed: HashMap<String, Value> = from_slice(raw)?;
        let map = parsed
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, s)
            })
            .collect();
        Ok(Self { map })
    }

    #[must_use]
    pub const fn from_map(map: HashMap<String, String>) -> Self {
        Self { map }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// `$key` reads the table (`None` if absent), a bare value is returned as itself
    #[must_use]
    pub fn resolve<'a>(&'a self, reference: &'a str) -> Option<&'a str> {
        let r = reference.trim();
        match r.strip_prefix('$') {
            Some(key) => self.map.get(key).map(String::as_str),
            None => Some(r),
        }
    }
}

/// Stage gate on a story (`RequiredStages[]`)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryRequiredStage {
    #[serde(alias = "StageId", default)]
    pub stage_id: String,
    #[serde(alias = "MinState", default)]
    pub min_state: String,
    #[serde(alias = "MaxState", default)]
    pub max_state: String,
}

/// Story under a group (`InfoUnlockDatas[]`)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryUnlockData {
    #[serde(alias = "StoryId", default)]
    pub story_id: String,
    #[serde(alias = "StoryGroup", default)]
    pub story_group: String,
    #[serde(alias = "StoryName", default)]
    pub story_name: String,
    /// Operation code (`GT-1`), absent on records and interludes
    #[serde(alias = "StoryCode", default)]
    pub story_code: Option<String>,
    #[serde(alias = "StorySort", default)]
    pub story_sort: i32,
    /// `Before Operation`, `After Operation` or `Interlude`
    #[serde(alias = "AvgTag", default)]
    pub avg_tag: Option<String>,
    /// Script path under `gamedata/story/`, without extension
    #[serde(alias = "StoryTxt", default)]
    pub story_txt: String,
    /// Summary path under `gamedata/story/`, without extension
    #[serde(alias = "StoryInfo", default)]
    pub story_info: Option<String>,
    #[serde(alias = "StoryDependence", default)]
    pub story_dependence: Option<String>,
    #[serde(alias = "RequiredStages", default)]
    pub required_stages: Vec<StoryRequiredStage>,
    #[serde(alias = "UnLockType", default)]
    pub un_lock_type: String,
    #[serde(alias = "StoryReviewType", default)]
    pub story_review_type: String,
}

/// Archives group: main chapter, event or operator record set
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryReviewGroup {
    #[serde(alias = "Id", default)]
    pub id: String,
    #[serde(alias = "Name", default)]
    pub name: String,
    /// `MAINLINE`, `ACTIVITY`, `MINI_ACTIVITY` or `NONE`
    #[serde(alias = "EntryType", default)]
    pub entry_type: String,
    /// `MAIN_STORY`, `ACTIVITY_STORY`, `MINI_STORY` or `NONE`
    #[serde(alias = "ActType", default)]
    pub act_type: String,
    /// `storyEntryPic_{id}`, the Archives cover
    #[serde(alias = "StoryEntryPicId", default)]
    pub story_entry_pic_id: Option<String>,
    #[serde(alias = "StartTime", default)]
    pub start_time: i64,
    #[serde(alias = "InfoUnlockDatas", default)]
    pub info_unlock_datas: Vec<StoryUnlockData>,
}

/// `story_review_table`, the Archives library index
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StoryReviewTableFile {
    /// The unpacker writes this key as `Story_reviews`, not `StoryReviews`
    #[serde(
        alias = "Story_reviews",
        alias = "StoryReviews",
        alias = "storyreviewtable",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub story_reviews: HashMap<String, StoryReviewGroup>,
}

/// `story_review_meta_table`, per-act layout (`ActArchiveData`) and content by id (`ActArchiveResData`)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoryReviewMetaTableFile {
    #[serde(alias = "ActArchiveData", default)]
    pub act_archive_data: ActArchiveData,
    #[serde(alias = "ActArchiveResData", default)]
    pub act_archive_res_data: ActArchiveResData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActArchiveData {
    #[serde(
        alias = "Components",
        deserialize_with = "deserialize_fb_map_or_default",
        default
    )]
    pub components: HashMap<String, ActArchiveComponent>,
}

/// Act's archive slots, all optional
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// Bare list, not a `{Xs: [...]}` wrapper
    #[serde(alias = "Landmark", default)]
    pub landmark: Option<Vec<FbKeyValue<String, ArchiveSlotItem>>>,
    /// Logs grouped into chapters
    #[serde(alias = "Log", default)]
    pub log: Option<Vec<FbKeyValue<String, ArchiveLogChapter>>>,
}

/// Slot items under the slot's plural key (`Avgs`, `Musics`, `News`, `Pics`, `Stories`)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// Slot entry: content id and sort order, spelled per slot (`PicId`/`PicSortId`)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchiveLogChapter {
    #[serde(alias = "ChapterIcon", default)]
    pub chapter_icon: String,
    #[serde(alias = "ChapterName", default)]
    pub chapter_name: String,
    #[serde(alias = "DisplayId", default)]
    pub display_id: String,
    #[serde(alias = "Logs", default)]
    pub logs: Vec<String>,
    /// Spelled `UnlockDes` in the source
    #[serde(alias = "UnlockDes", alias = "UnlockDesc", default)]
    pub unlock_des: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveLog {
    #[serde(alias = "LogId", default)]
    pub log_id: String,
    #[serde(alias = "LogDesc", default)]
    pub log_desc: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveLandmark {
    #[serde(alias = "LandmarkId", default)]
    pub landmark_id: String,
    #[serde(alias = "LandmarkName", default)]
    pub landmark_name: String,
    #[serde(alias = "LandmarkEngName", default)]
    pub landmark_eng_name: String,
    #[serde(alias = "LandmarkDesc", default)]
    pub landmark_desc: String,
    /// Picture stem under `textures/ui/`
    #[serde(alias = "LandmarkPic", default)]
    pub landmark_pic: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveNews {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// Headline
    #[serde(alias = "Desc", default)]
    pub desc: String,
    #[serde(alias = "NewsAuthor", default)]
    pub news_author: String,
    #[serde(alias = "NewsType", default)]
    pub news_type: String,
    #[serde(alias = "NewsFormat", default)]
    pub news_format: Option<RawArchiveNewsFormat>,
    /// Body split into text and image lines
    #[serde(alias = "NewsLines", default)]
    pub news_lines: Vec<RawArchiveNewsLine>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveNewsFormat {
    #[serde(alias = "TypeId", default)]
    pub type_id: String,
    #[serde(alias = "TypeName", default)]
    pub type_name: String,
    #[serde(alias = "TypeLogo", default)]
    pub type_logo: String,
    #[serde(alias = "TypeMainLogo", default)]
    pub type_main_logo: String,
    /// Masthead stem, on disk as `<stem>_bkg` and `<stem>_title`
    #[serde(alias = "TypeMainSealing", default)]
    pub type_main_sealing: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveNewsLine {
    /// Prose on a `TextContent` line, an image stem on an `ImageContent` one
    #[serde(alias = "Content", default)]
    pub content: String,
    #[serde(alias = "LineType", default)]
    pub line_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveStory {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// File title
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchivePic {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// Picture title
    #[serde(alias = "Desc", default)]
    pub desc: String,
    #[serde(alias = "PicDescription", default)]
    pub pic_description: String,
    /// AVG image stem under `textures/avg/imgs` (`27_i25`)
    #[serde(alias = "AssetPath", default)]
    pub asset_path: String,
    #[serde(alias = "Type_", alias = "Type", default)]
    pub type_: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawArchiveAudio {
    #[serde(alias = "Id", default)]
    pub id: String,
    /// Track title, a single space when untitled
    #[serde(alias = "Name", default)]
    pub name: String,
    #[serde(alias = "Desc", default)]
    pub desc: String,
}
