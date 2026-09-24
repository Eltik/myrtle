//! The story reader's WIRE TYPES, and nothing that derives them.
//!
//! Every field's doc comment reaches the `OpenAPI` document and the generated TS
//! binding, so the measured numbers in them are the contract's own evidence.
//! The full census behind those numbers is `docs/story-reader.md`, section
//! "1. What is true about the data"; the shapes are section "2. Wire contract".

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::operator::OperatorProfession;
use crate::core::gamedata::types::voice::LangType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

/// Where a group's `coverUrl` came from. `EntryPic` is the game's own
/// Archives picture; `Background` is DERIVED, the first background of the
/// group's first scripted story, because most EN groups have no
/// `StoryEntryPicId` and all 17 mainline chapters are among them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum StoryCoverKind {
    EntryPic,
    Background,
}

/// A main chapter's names, from `zone_table` and `chapter_table`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name_first: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name_second: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name_third: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryEntry {
    pub id: String,
    pub name: String,
    /// The operation code (`GT-1`), absent on records and interludes.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub code: Option<String>,
    pub sort: i32,
    /// `Before Operation`, `After Operation` or `Interlude`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_tag: Option<String>,
    pub group_id: String,
    /// True when a script file answers the probe, so `GET /story/{id}` will
    /// return a script rather than a 404.
    pub has_script: bool,
    /// Whitespace-separated words of prose in the script, counted once at
    /// index build by the same rule as `StoryScript.wordCount`. 0 when the
    /// story has no script.
    pub word_count: u32,
    /// The script plays a `[Video]`. 1 of the 1,797 scripted EN stories is
    /// nothing but one (`main_14_level_main_14-20_beg`).
    pub has_video: bool,
    pub required_stages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryGroup {
    pub id: String,
    pub name: String,
    pub category: StoryCategory,
    pub entry_type: String,
    pub act_type: String,
    /// `activity_table.BasicInfo.DisplayType` for event groups (`SIDESTORY`,
    /// `BRANCHLINE`, `MINISTORY`, `NONE`); the game's own Archives shelf.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cover_url: Option<String>,
    /// Which source `coverUrl` came from, absent exactly when `coverUrl` is.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cover_kind: Option<StoryCoverKind>,
    /// The group's KEY VISUAL, `StorylineStorySets[].KvImageId` resolved
    /// through `spritepack/mixstory_kv_sprites_{0,1,2}`: the card art the
    /// game's own Story Collection draws. It is NOT `coverUrl`: a cover is the
    /// Archives entry picture (67 groups) or a derived first background (382),
    /// while a banner is the authored 532x456 plate and exists for exactly the
    /// 81 groups a storyline shelf lists, all 17 mainline chapters among them.
    /// For an event both exist and differ; the banner is the key visual.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub banner_url: Option<String>,
    /// The group's title as ART, `StorylineStorySets[].TitleImageId` resolved
    /// through `spritepack/mixstory_title_sprites_0`: the 516x260 logotype the
    /// game sets behind the written title, on all 81 groups a storyline shelf
    /// lists. It carries the same words as `name`, drawn, so a consumer that
    /// shows it still prints `name` for search and for a reader who cannot see
    /// the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title_image_url: Option<String>,
    /// The group's own small glyph, `MainlineData.DecoImageId` resolved
    /// through `spritepack/mixstory_deco_sprites_h2_0`. MAINLINE only: the
    /// table carries a deco for the 17 chapters and for nothing else, so an
    /// event group has none and the UI falls back to its title or code.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_url: Option<String>,
    /// 0 through 16 on the 17 mainline chapters, absent everywhere else. Read
    /// off the ZONE (`zone_table.ZoneNameTitleCurrent`, the "00".."16" the
    /// client prints under `EPISODE`), falling back to the `main_N` id. It is
    /// what a reader actually knows a chapter by, where `name` is an arc
    /// title nobody remembers.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_number: Option<u32>,
    /// Unix release time, `-1` when nothing dates the group. It is the
    /// group's own `StartTime`, except on a MAINLINE group, where every one of
    /// the 17 EN rows writes -1 and the date is the zone's
    /// `zone_table.MainlineAdditionInfo.ZoneOpenTime` instead (5 of 17 carry
    /// one: `main_10` 1666180800 through `main_14` 1730394000). No field says
    /// which source answered, because a release date is a release date and the
    /// consumer renders one line either way.
    #[ts(type = "number")]
    pub start_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub zone: Option<StoryZone>,
    /// The sum of this group's stories' `wordCount`.
    pub word_count: u32,
    /// Distinct backgrounds plus distinct CGs the group's scripts reference,
    /// the row count of the ILLUSTRATIONS tab. Counted over names, resolved
    /// or not.
    pub illustration_count: u32,
    /// Distinct sprite folders the group's scripts reference, the row count
    /// of the Sprites sub-tab.
    pub sprite_count: u32,
    /// The group's THEME, the track the game plays over its Archives entry:
    /// `music_3in1bg_main{N}` on a mainline chapter and the
    /// `sys.ON_ACTIVITY_LOADED.{id}` bank on an event. 86 of the 87 EN
    /// library groups carry one and 6 of those carry a title; the one that
    /// does not is `act24side`, whose bank names a loop clip the tree does
    /// not hold.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub music: Option<StoryMusic>,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct OperatorRecordGroup {
    pub char_id: String,
    /// The operator's name from the character table.
    pub name: String,
    /// 1-6 stars, the same `u8` the operators list's `rarity` carries; 0 when
    /// the character table has no such operator (0 of 315 on EN).
    pub rarity: u8,
    /// The character table's profession, the raw game value the operators
    /// list carries (`PIONEER`, `WARRIOR`, ...).
    pub profession: OperatorProfession,
    /// The `ui_char_avatar_*` sprite as an asset-index path
    /// (`/textures/spritepack/...`), the same form the operators list's
    /// `portrait` and a group's `coverUrl` carry: the frontend serves it
    /// under `/api/assets`. Empty when no avatar resolves (0 of 315 on EN).
    pub avatar_url: String,
    /// The sum of this operator's records' `wordCount`.
    pub word_count: u32,
    /// Distinct backgrounds plus distinct CGs over this operator's records.
    pub illustration_count: u32,
    /// Distinct sprite folders over this operator's records.
    pub sprite_count: u32,
    pub stories: Vec<StoryEntry>,
}

/// One arc inside a storyline: the run of groups between two of the game's
/// own `MAINLINE_SPLIT` headers. Only `mainLine` carries any on EN (4:
/// `HOUR OF AN AWAKENING`, `SHATTER OF A VISION`, `SHADOW OF A DYING SUN`,
/// `NEXUS POINT OF FUTURE`); every `ssLine_*` has an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StorylineArc {
    pub name: String,
    pub sort: i32,
    /// The arc header's own icon, `MainlineSplitData.IconId` resolved through
    /// `spritepack/mixstory_deco_sprites_h2_0` (`act_0` to `act_3`, 184x52).
    /// All 4 EN mainline arcs carry one and nothing else declares an arc.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_url: Option<String>,
    pub group_ids: Vec<String>,
    /// The chapters the arc spans: the min and max
    /// `StoryGroup.chapterNumber` over the groups on it that carry one.
    /// Absent on an arc that holds no numbered chapter at all.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_range: Option<ChapterRange>,
}

/// An inclusive run of mainline chapter numbers, `from <= to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ChapterRange {
    pub from: u32,
    pub to: u32,
}

/// One of the game's own themed shelves over the Archives
/// (`stage_table.Storylines`), with the `StoryGroup.id`s it holds, in the
/// order the shelf lists them.
///
/// The join is `StorylineStorySets`, NOT the group id: a location names a
/// `RelevantStorySetId`, that set is `MAINLINE` and names a zone
/// (`MainlineData.ZoneId`, which is the mainline group's own id) or `SS` /
/// `COLLECT` and names an activity (`RelevantActivityId`, which is the event
/// group's own id). On EN 81 of 81 sets resolve and 79 of them name a group
/// the Archives list; the other 2 (`setId_mainline_3_1` -> `act2mainss`,
/// `setId_mainline_3_2` -> `act3mainss`) name ids `story_review_table` does
/// not carry and are dropped.
///
/// A group can sit on SEVERAL shelves (17 do on EN), so `groupIds` over all
/// storylines is not a partition: it lists 104 slots over 81 distinct groups.
/// Picking one shelf per group is the consumer's job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Storyline {
    pub id: String,
    pub name: String,
    pub sort: i32,
    /// The shelf's own glyph: `StorylineIconId`
    /// (`spritepack/mixstory_abbr_sprites_h2_0`, 44x36, 13 of the 14 EN
    /// shelves) and, where that is absent, `StorylineLogoId`
    /// (`spritepack/mixstory_logo_sprites_0`, 14 of 14). `mainLine` is the one
    /// shelf with no abbreviation glyph, so it draws its logo.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub icon_url: Option<String>,
    /// The shelf's LOGO, `StorylineLogoId` (`spritepack/mixstory_logo_sprites_0`,
    /// 108x108, 14 of 14 EN shelves): the emblem the game's Story Collection
    /// draws for the shelf, monochrome white on transparent. The jump bar draws
    /// this so every shelf chip carries the same kind of mark as the arcs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub logo_url: Option<String>,
    /// The mainline chapters this shelf spans: the min and max
    /// `StoryGroup.chapterNumber` over its groups. `mainLine` is 0 to 16, and
    /// a themed shelf carries one when it holds mainline chapters of its own
    /// (`ssLine_1` holds `main_7` through `main_14`); it is absent on the 7 EN
    /// shelves that hold none.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_range: Option<ChapterRange>,
    pub group_ids: Vec<String>,
    pub arcs: Vec<StorylineArc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryIndex {
    pub groups: Vec<StoryGroup>,
    pub records: Vec<OperatorRecordGroup>,
    /// The game's themed shelves, in `SortId` order. Empty when the loaded
    /// `stage_table` carries no `Storylines` (every CN/EN tree since 2024
    /// does).
    pub storylines: Vec<Storyline>,
    pub totals: StoryTotals,
}

/// The library's size, over the DISTINCT story ids in the index: the 364
/// `record` groups and the 315 operator records list the same 367 stories,
/// so a naive sum of both halves would count those twice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryTotals {
    pub stories: u32,
    pub with_script: u32,
    pub words: u32,
}

/// One background or CG a group's stories reference, with the stories that
/// reference it. `url` is `null` when the name resolves to no file on disk:
/// the tab lists the name and says so rather than dropping it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct IllustrationItem {
    pub name: String,
    pub url: Option<String>,
    pub story_ids: Vec<String>,
}

/// One character sprite folder a group's stories reference. `bodyUrl` is the
/// `$1` body, the same file `GET /story/{id}` resolves a bare name to, and
/// `faces` counts the DISTINCT `#N` variants the group's scripts ask for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct SpriteItem {
    pub base: String,
    pub body_url: Option<String>,
    pub story_ids: Vec<String>,
    pub faces: u32,
}

/// Every illustration one group or one operator's records reference, in
/// first-appearance order over the group's stories in library order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryIllustrations {
    /// The id as asked for: a `StoryGroup.id` or an operator `charId`.
    pub group_id: String,
    pub backgrounds: Vec<IllustrationItem>,
    pub images: Vec<IllustrationItem>,
    pub sprites: Vec<SpriteItem>,
}

/// The theme a story group plays, resolved to clips.
///
/// `loopUrl` is the only required half: 59 of the 86 EN groups that have a
/// theme also carry a one-shot `introUrl` in front of it, and a group whose
/// loop clip is not on disk carries no `music` at all rather than a URL that
/// 404s. `title` is the `Musics` row's `Name`, which 101 of the 208 EN rows
/// write as a single space, so it is trimmed and dropped when empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryMusic {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intro_url: Option<String>,
    pub loop_url: String,
}

/// One item of a group's archive: a log entry, a landmark, a news story, a
/// file, a gallery picture, a track or a recording.
///
/// The sections are a TAGGED list rather than a fixed record because an
/// archive carries two or three of them and never all seven: on EN
/// `act13side` is news, files and a gallery, `act17side` is logs, landmarks
/// and a gallery, `act25side` is files and a gallery, `main_14` is nothing
/// but recordings. `count` is the LEAF count, the number of rows the game's
/// own screen lists, so `logs` counts logs and not the chapters they are
/// grouped into, and `recordings` counts clips and not nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
        /// The clips the shelf hides until they are unlocked, which belong to
        /// no node (8 of `main_14`'s 43).
        hidden: Vec<ArchiveClip>,
    },
}

/// A run of log entries under one of the archive's own chapter headers, in
/// the order the chapter lists them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveLogChapter {
    pub id: String,
    pub name: String,
    /// The number the chapter header prints (`01`), a string because the
    /// table writes it padded.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub display_id: Option<String>,
    /// `NORMAL` or `SPECIAL`, the header's own glyph. Kept raw: no art is
    /// joined to it, because the two values are the whole vocabulary.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub chapter_icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unlock_desc: Option<String>,
    pub logs: Vec<ArchiveLog>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveLog {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveLandmark {
    pub id: String,
    pub name: String,
    /// The landmark's name in the setting's own Latin spelling, dropped when
    /// it repeats `name` (15 of the 26 EN landmarks repeat it).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub eng_name: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub picture_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveNewsItem {
    pub id: String,
    /// The headline.
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub author: Option<String>,
    /// The masthead this story ran under.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub format: Option<ArchiveNewsFormat>,
    /// The body, already split by the table into prose and pictures.
    pub lines: Vec<ArchiveNewsLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveNewsFormat {
    pub type_id: String,
    pub type_name: String,
    /// The small masthead glyph, and the big one the story is headed with.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub main_logo_url: Option<String>,
}

/// One line of a news body: prose, or a picture that runs in the column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub enum ArchiveNewsLine {
    Text {
        text: String,
    },
    Image {
        /// The picture's name, kept so a line with no file on disk still says
        /// what is missing.
        name: String,
        url: Option<String>,
    },
}

/// One of the archive's files or diaries: a page of prose with its own plate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveFile {
    pub id: String,
    pub title: String,
    /// The in-setting date the file is stamped with (`1097.10.27`).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub date: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub picture_url: Option<String>,
    /// The drawn title over the page, on 4 of the 16 EN files.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title_picture_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchivePicture {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    /// `IMAGE`, `ENDING_IMAGE` or `ROGUE_IMAGE`, raw from the table. All 57
    /// pictures on the EN library groups are `IMAGE`; the other two values
    /// belong to the `rogue_*` archives the library does not list.
    pub picture_type: String,
    /// The AVG image, resolved through the same tree a script's `[Image]`
    /// name resolves through.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveTrack {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub intro_url: Option<String>,
    pub loop_url: String,
}

/// One node of the recordings shelf: a titled set of clips the player unlocks
/// together.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveRecordingNode {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unlock_desc: Option<String>,
    pub clips: Vec<ArchiveClip>,
}

/// One recorded line, in every language whose file is on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveClip {
    pub char_id: String,
    pub voice_id: String,
    pub index: i32,
    pub tracks: Vec<ArchiveClipTrack>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ArchiveClipTrack {
    pub language: LangType,
    pub url: String,
}

/// Everything the game's own "from the archive" screen shows for one group.
///
/// An archive is a group's, not a story's: it belongs to the event and is
/// unlocked by playing it. A group with no archive answers 200 with an empty
/// `sections`, because "this event kept no archive" is an answer and a 404 is
/// not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct StoryArchive {
    pub group_id: String,
    pub sections: Vec<StoryArchiveSection>,
}
