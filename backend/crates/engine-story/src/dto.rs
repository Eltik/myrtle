use myrtle_gamedata_raw::tables::operator::OperatorProfession;
use myrtle_gamedata_raw::tables::voice::LangType;
use serde::{Deserialize, Serialize};

use crate::assets::CharacterSprite;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StoryCategory {
    Main,
    Side,
    Vignette,
    Is,
    Reclamation,
    SideContent,
    Record,
}

impl StoryCategory {
    pub const ALL: [Self; 7] = [
        Self::Main,
        Self::Side,
        Self::Vignette,
        Self::Is,
        Self::Reclamation,
        Self::SideContent,
        Self::Record,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Side => "side",
            Self::Vignette => "vignette",
            Self::Is => "is",
            Self::Reclamation => "reclamation",
            Self::SideContent => "sideContent",
            Self::Record => "record",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StoryCoverKind {
    EntryPic,
    Background,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_second: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_third: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEntry {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub sort: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_tag: Option<String>,
    pub group_id: String,
    pub has_script: bool,
    pub word_count: u32,
    pub has_video: bool,
    pub required_stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryGroup {
    pub id: String,
    pub name: String,
    pub category: StoryCategory,
    pub entry_type: String,
    pub act_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_kind: Option<StoryCoverKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_number: Option<u32>,
    pub start_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<StoryZone>,
    pub word_count: u32,
    pub illustration_count: u32,
    pub sprite_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music: Option<StoryMusic>,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorRecordGroup {
    pub char_id: String,
    pub name: String,
    pub rarity: u8,
    pub profession: OperatorProfession,
    pub profession_name: Option<String>,
    pub avatar_url: String,
    pub word_count: u32,
    pub illustration_count: u32,
    pub sprite_count: u32,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorylineArc {
    pub name: String,
    pub sort: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    pub group_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_range: Option<ChapterRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterRange {
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storyline {
    pub id: String,
    pub name: String,
    pub sort: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_range: Option<ChapterRange>,
    pub group_ids: Vec<String>,
    pub own_group_ids: Vec<String>,
    pub arcs: Vec<StorylineArc>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryIndex {
    pub groups: Vec<StoryGroup>,
    pub records: Vec<OperatorRecordGroup>,
    pub storylines: Vec<Storyline>,
    pub totals: StoryTotals,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryTotals {
    pub stories: u32,
    pub with_script: u32,
    pub words: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IllustrationItem {
    pub name: String,
    pub url: Option<String>,
    pub story_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteItem {
    pub base: String,
    pub body_url: Option<String>,
    pub story_ids: Vec<String>,
    pub faces: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryIllustrations {
    pub group_id: String,
    pub backgrounds: Vec<IllustrationItem>,
    pub images: Vec<IllustrationItem>,
    pub sprites: Vec<SpriteItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryMusic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro_url: Option<String>,
    pub loop_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum StoryArchiveSection {
    Logs {
        count: u32,
        chapters: Vec<ArchiveLogChapter>,
    },
    Landmarks {
        count: u32,
        landmarks: Vec<ArchiveLandmark>,
    },
    News {
        count: u32,
        news: Vec<ArchiveNewsItem>,
    },
    Files {
        count: u32,
        files: Vec<ArchiveFile>,
    },
    Gallery {
        count: u32,
        pictures: Vec<ArchivePicture>,
    },
    Music {
        count: u32,
        tracks: Vec<ArchiveTrack>,
    },
    Recordings {
        count: u32,
        nodes: Vec<ArchiveRecordingNode>,
        hidden: Vec<ArchiveClip>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLogChapter {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_desc: Option<String>,
    pub logs: Vec<ArchiveLog>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLog {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLandmark {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eng_name: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNewsItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ArchiveNewsFormat>,
    pub lines: Vec<ArchiveNewsLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNewsFormat {
    pub type_id: String,
    pub type_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_logo_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ArchiveNewsLine {
    Text { text: String },
    Image { name: String, url: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFile {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_picture_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePicture {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub picture_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveTrack {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro_url: Option<String>,
    pub loop_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRecordingNode {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlock_desc: Option<String>,
    pub clips: Vec<ArchiveClip>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveClip {
    pub char_id: String,
    pub voice_id: String,
    pub index: i32,
    pub tracks: Vec<ArchiveClipTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveClipTrack {
    pub language: LangType,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryArchive {
    pub group_id: String,
    pub sections: Vec<StoryArchiveSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StorySpriteKind {
    Operator,
    Npc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteName {
    pub name: String,
    pub count: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteVariant {
    pub key: String,
    #[serde(flatten)]
    pub sprite: CharacterSprite,
    pub whole_body: bool,
    pub uses: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteEntry {
    pub base: String,
    pub kind: StorySpriteKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub char_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub names: Vec<StorySpriteName>,
    pub noise: f64,
    pub lines: f64,
    pub story_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_seen: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_story: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_order: Option<u32>,
    pub variant_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<StorySpriteVariant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteIndex {
    pub sprites: Vec<StorySpriteEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteStory {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub group_id: String,
    pub group_name: String,
    pub lines: f64,
    pub uses: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteDetail {
    pub sprite: StorySpriteEntry,
    pub names: Vec<StorySpriteNameDetail>,
    pub stray_names: Vec<StorySpriteNameDetail>,
    pub variants: Vec<StorySpriteVariant>,
    pub stories: Vec<StorySpriteStory>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteNameStory {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub lines: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteExample {
    pub story_id: String,
    pub story_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub line: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySpriteNameDetail {
    pub name: String,
    pub count: f64,
    pub stories: Vec<StorySpriteNameStory>,
    pub more_stories: u32,
    pub examples: Vec<StorySpriteExample>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryGallery {
    pub groups: Vec<GalleryGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryGroup {
    pub id: String,
    pub name: String,
    pub pictures: Vec<GalleryPicture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryPicture {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub picture_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoryArtKind {
    Cg,
    Scene,
}

impl StoryArtKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cg => "cg",
            Self::Scene => "scene",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryArtGallery {
    pub kind: StoryArtKind,
    pub groups: Vec<StoryArtGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryArtGroup {
    pub id: String,
    pub name: String,
    pub category: StoryCategory,
    pub pictures: Vec<StoryArtPicture>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryArtPicture {
    pub id: String,
    pub width: u32,
    pub height: u32,
}
