use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::database::models::tier_list::EntityKind;

/// One tab of the player profile page. The variants are the allow-list and
/// their declaration order is the canonical order, the one the profile showed
/// before layouts existed: a layout that omits a tab gets it back in this
/// position's turn, visible.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileTabId {
    /// The owner's arranged blocks. First in canonical order, so a showcase
    /// is the landing tab once it has a block, but shown to a visitor only
    /// then: an empty showcase is no tab at all to them.
    Showcase,
    Stats,
    Score,
    Roster,
    Plans,
    Inventory,
    Enemies,
    Optimizer,
}

impl ProfileTabId {
    /// Every tab, in canonical order.
    pub const ALL: [Self; 8] = [
        Self::Showcase,
        Self::Stats,
        Self::Score,
        Self::Roster,
        Self::Plans,
        Self::Inventory,
        Self::Enemies,
        Self::Optimizer,
    ];

    /// The wire id, as the client and the stored jsonb spell it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Showcase => "showcase",
            Self::Stats => "stats",
            Self::Score => "score",
            Self::Roster => "roster",
            Self::Plans => "plans",
            Self::Inventory => "inventory",
            Self::Enemies => "enemies",
            Self::Optimizer => "optimizer",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.as_str() == id)
    }

    /// A SQL predicate, true unless the `profile_layout` jsonb in `column` hides
    /// this tab: for the queries that rank or filter players by a tab's data
    /// (search, leaderboards), which must drop a player who made that tab
    /// private just as its endpoints refuse a visitor. A NULL layout hides
    /// nothing. Containment is exact because every stored layout is normalized,
    /// so each id appears once. The id is a fixed literal, never user input.
    pub fn visible_sql(self, column: &str) -> String {
        format!(
            "NOT COALESCE({column} -> 'tabs' @> '[{{\"id\":\"{}\",\"visible\":false}}]'::jsonb, false)",
            self.as_str()
        )
    }
}

/// One entry of [`ProfileLayout::tabs`].
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileTab {
    pub id: ProfileTabId,
    /// False hides the tab from visitors and refuses its data to them. The
    /// owner always sees and reads every tab.
    pub visible: bool,
}

/// How the owner arranged their profile, `user_settings.profile_layout`.
///
/// Stored as a jsonb object of four keys: `tabs` (the order and visibility
/// of every tab), `showcase` (the Showcase tab's blocks), `background` (the
/// header art) and `header` (the header numbers visitors receive and the stat
/// strip's cards); a save sets only the keys it sends. Deserializing goes
/// through [`ProfileLayout::normalize`], so every value that reaches the
/// code, from the database, the cache or a request body, holds each known tab
/// exactly once.
///
/// What a visitor receives is [`ProfileLayout::visible_only`]: a private tab is
/// absent from it, not present with `visible: false`.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "serde_json::Value")]
pub struct ProfileLayout {
    /// Display order, first tab first.
    pub tabs: Vec<ProfileTab>,
    /// The Showcase tab's blocks. `None` when the stored layout has no
    /// `showcase` key: a save that leaves the key out keeps the stored
    /// showcase (see `queries::users::update_settings`), so a client that only
    /// knows about tabs cannot wipe it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub showcase: Option<ProfileShowcase>,
    /// The art behind the profile header. `None` when the stored layout has
    /// none, which renders the header exactly as before backgrounds existed.
    /// Not gated by any tab: the header is the profile itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub background: Option<ProfileBackground>,
    /// Which header numbers a visitor receives and which fill the stat strip.
    /// `None` when the stored layout has none, which renders the header and
    /// strip exactly as before this existed, every number shown. Like the
    /// background, not gated by any tab.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub header: Option<ProfileHeader>,
}

/// Most blocks one showcase holds.
pub const SHOWCASE_MAX_BLOCKS: usize = 12;
/// Most entities one favourites block holds.
pub const SHOWCASE_MAX_IDS: usize = 24;
/// Longest favourites block title, in characters, after trimming.
pub const SHOWCASE_TITLE_MAX: usize = 40;
/// Longest slug or entity id normalization keeps: the `grids.slug` column's
/// width, which is also wider than any game id.
const SHOWCASE_KEY_MAX: usize = 120;

/// `raw[field]` when it is a string.
fn str_field<'a>(raw: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    raw.get(field).and_then(serde_json::Value::as_str)
}

/// `raw[field]` when it is an array, else no entries.
fn array_field<'a>(raw: &'a serde_json::Value, field: &str) -> &'a [serde_json::Value] {
    raw.get(field)
        .and_then(serde_json::Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

/// `s` trimmed, or `None` when that leaves it empty or longer than
/// [`SHOWCASE_KEY_MAX`] characters: the rule every slug and id follows.
fn bounded_key(s: &str) -> Option<&str> {
    let s = s.trim();
    (!s.is_empty() && s.chars().count() <= SHOWCASE_KEY_MAX).then_some(s)
}

/// `raw[field]` when it is a finite number, rounded and clamped to
/// `min..=max`.
fn clamped_number(raw: &serde_json::Value, field: &str, min: u16, max: u16) -> Option<u16> {
    raw.get(field)
        .and_then(serde_json::Value::as_f64)
        .filter(|v| v.is_finite())
        .map(|v| {
            let n = v.round().clamp(f64::from(min), f64::from(max));
            // In `min..=max` after the clamp, so the cast is exact.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let n = n as u16;
            n
        })
}

/// A favourites title as stored: control characters dropped, trimmed, cut to
/// [`SHOWCASE_TITLE_MAX`] characters, `None` when nothing is left.
fn normalize_title(raw: &str) -> Option<String> {
    let title = raw
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .chars()
        .take(SHOWCASE_TITLE_MAX)
        .collect::<String>()
        .trim_end()
        .to_owned();
    (!title.is_empty()).then_some(title)
}

/// `profile_layout.showcase`: the owner's blocks, in display order.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileShowcase {
    pub blocks: Vec<ShowcaseBlock>,
}

/// One showcase block. Every block points at something that already exists on
/// the site; the only text an owner types is a favourites block's title.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShowcaseBlock {
    /// A row of favourite entities of one kind, any kind a tier list can rank.
    Favourites {
        entity_kind: EntityKind,
        /// Display order, at most [`SHOWCASE_MAX_IDS`], no repeats.
        ids: Vec<String>,
        /// Plain text, trimmed, at most [`SHOWCASE_TITLE_MAX`] characters.
        /// `None` shows the kind's own name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        title: Option<String>,
    },
    /// A grid, by slug. Unlisted grids count: they are served by slug.
    Grid { slug: String },
    /// A tier list, by slug. Unlisted lists count too.
    TierList { slug: String },
    /// One of the owner's plans shown on their profile. A visitor gets it only
    /// while the Plans tab is visible to them.
    Plan {
        #[ts(type = "string")]
        id: Uuid,
        /// The plan's operator. Deleting a plan and planning the operator
        /// again makes a new row with a new id; the block follows the
        /// operator to it instead of reading as removed. Absent on blocks
        /// saved before 2026-10-07, which still match by `id` alone.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        operator_id: Option<String>,
    },
}

impl ShowcaseBlock {
    /// `raw` as a block, or `None` when it is not one: an unknown `type`, a
    /// missing or empty referent, an unknown entity kind. Favourites ids are
    /// trimmed, deduplicated (the first wins) and cut to [`SHOWCASE_MAX_IDS`];
    /// a favourites block left with no id is dropped.
    fn normalize(raw: &serde_json::Value) -> Option<Self> {
        let key = |field: &str| {
            str_field(raw, field)
                .and_then(bounded_key)
                .map(str::to_owned)
        };
        match str_field(raw, "type")? {
            "favourites" => {
                let entity_kind = str_field(raw, "entity_kind")?.parse::<EntityKind>().ok()?;
                let mut ids: Vec<String> = Vec::new();
                for id in array_field(raw, "ids") {
                    let Some(id) = id.as_str().and_then(bounded_key) else {
                        continue;
                    };
                    if ids.len() < SHOWCASE_MAX_IDS && !ids.iter().any(|i| i == id) {
                        ids.push(id.to_owned());
                    }
                }
                if ids.is_empty() {
                    return None;
                }
                let title = str_field(raw, "title").and_then(normalize_title);
                Some(Self::Favourites {
                    entity_kind,
                    ids,
                    title,
                })
            }
            "grid" => Some(Self::Grid { slug: key("slug")? }),
            "tier_list" => Some(Self::TierList { slug: key("slug")? }),
            "plan" => Some(Self::Plan {
                id: key("id")?.parse().ok()?,
                operator_id: key("operator_id"),
            }),
            _ => None,
        }
    }

    /// Two blocks that show the same thing: the same grid, tier list or plan.
    /// Favourites never repeat each other, even of one kind: an owner may
    /// keep two operator rows under different titles.
    fn same_referent(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Grid { slug: a }, Self::Grid { slug: b })
            | (Self::TierList { slug: a }, Self::TierList { slug: b }) => a == b,
            (
                Self::Plan {
                    id: a,
                    operator_id: a_op,
                },
                Self::Plan {
                    id: b,
                    operator_id: b_op,
                },
            ) => a == b || (a_op.is_some() && a_op == b_op),
            _ => false,
        }
    }
}

impl ProfileShowcase {
    /// The showcase `raw` asks for: malformed blocks dropped, a repeated
    /// grid, tier list or plan dropped (the first wins), then cut to
    /// [`SHOWCASE_MAX_BLOCKS`]. Total, like [`ProfileLayout::normalize`]: a
    /// value with no `blocks` array is an empty showcase.
    pub fn normalize(raw: &serde_json::Value) -> Self {
        let mut blocks: Vec<ShowcaseBlock> = Vec::new();
        for entry in array_field(raw, "blocks") {
            if blocks.len() == SHOWCASE_MAX_BLOCKS {
                break;
            }
            let Some(block) = ShowcaseBlock::normalize(entry) else {
                continue;
            };
            if !blocks.iter().any(|b| b.same_referent(&block)) {
                blocks.push(block);
            }
        }
        Self { blocks }
    }
}

/// What art a profile header may show. `skin` and `operator` are tier-list
/// entity kinds, so their ids are checked by the same lookup a favourites
/// block is, and their art is a painted figure the game ships a 1024 px
/// reduced copy of beside the full one (469 of 474 outfit arts, 323 of 381
/// elite 2 arts on EN 2026-10-05). `archive_pic` is a landscape picture,
/// checked against `story_review_meta_table` instead.
///
/// Ruled out by measurement on EN 2026-10-05, none of them wide enough or
/// small enough for a header about 1100 px wide: event art (`/event-image`)
/// is 280x166; a main story key visual is a 432 px square poster with the
/// chapter title typeset into it; an Archives entry picture is 420x508
/// portrait. An Archives gallery picture is 1600x900 and right for the
/// header but 2.05 MB on average (324 pictures), and only 48 of the 324 have
/// the game's 260x148 gallery thumbnail, so `archive_pic` is served through
/// `/story/gallery/{id}/{thumb,header}`, 320 px and 1600 px JPEGs.
///
/// `story_cg` and `story_scene` are the story scripts' own art (1,230 CGs
/// and 898 scene plates on EN 2026-10-06 pass the header filter of at
/// least 1024 px, 1.6 to 1.9 wide over high and 0.2 PNG bytes per px),
/// checked against `/story/art-gallery/{kind}` and served through it the
/// same way.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileBackgroundKind {
    /// An outfit, a `skin_table` id with an `@` (`char_002_amiya@epoque#4`).
    Skin,
    /// An operator's own art, elite 2 where they have one, by char id.
    Operator,
    /// An Archives gallery picture, by its `ActArchiveResData.Pics` key
    /// (`act13side_pic_0`, `pic_rogue_1_KV1`).
    ArchivePic,
    /// A story CG, by the lowercase asset key its scripts name, from
    /// `GET /story/art-gallery/cg`.
    StoryCg,
    /// A story scene plate, the same from `GET /story/art-gallery/scene`.
    StoryScene,
}

impl ProfileBackgroundKind {
    pub const ALL: [Self; 5] = [
        Self::Skin,
        Self::Operator,
        Self::ArchivePic,
        Self::StoryCg,
        Self::StoryScene,
    ];

    /// The wire id, as the client and the stored jsonb spell it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skin => "skin",
            Self::Operator => "operator",
            Self::ArchivePic => "archive_pic",
            Self::StoryCg => "story_cg",
            Self::StoryScene => "story_scene",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == id)
    }

    /// The entity kind whose lookup decides whether an id exists, `None` for
    /// a gallery picture, which no tier list ranks.
    pub const fn entity_kind(self) -> Option<EntityKind> {
        match self {
            Self::Skin => Some(EntityKind::Skin),
            Self::Operator => Some(EntityKind::Operator),
            Self::ArchivePic | Self::StoryCg | Self::StoryScene => None,
        }
    }
}

/// `profile_layout.background`: a reference to game art, never a URL. The
/// client builds the URL from the kind and id with the helpers every other
/// page uses.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBackground {
    pub kind: ProfileBackgroundKind,
    pub id: String,
    /// Where the crop centres, as a percentage of the art's width, 0 to 100.
    /// `None` is the centre.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub focus_x: Option<u8>,
    /// The same for height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub focus_y: Option<u8>,
    /// How far the art is zoomed in past `cover`, as a percentage,
    /// [`BACKGROUND_SCALE_MIN`] to [`BACKGROUND_SCALE_MAX`]. `None` is 100,
    /// the art exactly covering the header as it did before zoom existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub scale: Option<u16>,
    /// Which elite art an operator background draws: `1` or `2`. `None` is the
    /// art the header drew before the choice existed, elite 2 where the
    /// operator has it, else elite 1. Kept on the operator kind alone, so an
    /// outfit or a gallery picture never carries it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub elite: Option<u8>,
}

/// The smallest background zoom: the art just covers the header.
pub const BACKGROUND_SCALE_MIN: u16 = 100;
/// The largest background zoom, three times `cover`.
pub const BACKGROUND_SCALE_MAX: u16 = 300;
/// The largest focus, the art's far edge, as a percentage.
const FOCUS_MAX: u16 = 100;

impl ProfileBackground {
    /// `raw` as a background, or `None` when it is not one: not an object, an
    /// unknown `kind`, a missing, blank or over-long `id`. The id is trimmed.
    /// A focus that is a number is rounded and clamped to 0..=100; anything
    /// else is no focus. A scale that is a number is rounded and clamped to
    /// [`BACKGROUND_SCALE_MIN`]..=[`BACKGROUND_SCALE_MAX`]; anything else is
    /// no scale. An operator's `elite` is kept when it is exactly 1 or 2;
    /// anything else, or any other kind, is no elite.
    pub fn normalize(raw: &serde_json::Value) -> Option<Self> {
        let kind = str_field(raw, "kind").and_then(ProfileBackgroundKind::from_id)?;
        let id = str_field(raw, "id").and_then(bounded_key)?.to_owned();
        // Clamped to 0..=100, so it always fits a `u8`.
        let focus = |field: &str| {
            clamped_number(raw, field, 0, FOCUS_MAX).and_then(|pct| u8::try_from(pct).ok())
        };
        Some(Self {
            kind,
            id,
            focus_x: focus("focus_x"),
            focus_y: focus("focus_y"),
            scale: clamped_number(raw, "scale", BACKGROUND_SCALE_MIN, BACKGROUND_SCALE_MAX),
            elite: if kind == ProfileBackgroundKind::Operator {
                raw.get("elite")
                    .and_then(serde_json::Value::as_u64)
                    .filter(|e| matches!(e, 1 | 2))
                    .and_then(|e| u8::try_from(e).ok())
            } else {
                None
            },
        })
    }
}

/// One number the profile header shows. The variants are the allow-list of
/// what an owner may hide from visitors, and their declaration order is the
/// canonical order a stored `hidden` list is kept in.
///
/// Identity is not on the list: the uid is the profile's URL and the nickname
/// is what search matches, so neither can be hidden from the page that is
/// reached through them, and the server and avatar say nothing a visitor
/// could not read off the uid and the page.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileHeaderField {
    /// Account level and EXP, the bar under the nickname.
    Level,
    /// The registration date.
    Registered,
    /// Cumulative sign-in days.
    SignIn,
    Operators,
    /// Both skin counts: every outfit, and the non-default ones the strip shows.
    Skins,
    Items,
    Orundum,
    /// Originite Prime.
    Originite,
    Lmd,
    /// Headhunting Permits, `gacha_tickets`.
    HeadhuntingPermits,
    /// Ten-roll Headhunting Permits, `ten_pull_tickets`.
    TenPullPermits,
}

impl ProfileHeaderField {
    /// Every field, in canonical order.
    pub const ALL: [Self; 11] = [
        Self::Level,
        Self::Registered,
        Self::SignIn,
        Self::Operators,
        Self::Skins,
        Self::Items,
        Self::Orundum,
        Self::Originite,
        Self::Lmd,
        Self::HeadhuntingPermits,
        Self::TenPullPermits,
    ];

    /// The stat strip before header layouts existed, in its order: what a
    /// header with no `strip` shows.
    pub const DEFAULT_STRIP: [Self; 5] = [
        Self::Operators,
        Self::Skins,
        Self::Items,
        Self::Orundum,
        Self::Originite,
    ];

    /// The wire id, as the client and the stored jsonb spell it.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Level => "level",
            Self::Registered => "registered",
            Self::SignIn => "sign_in",
            Self::Operators => "operators",
            Self::Skins => "skins",
            Self::Items => "items",
            Self::Orundum => "orundum",
            Self::Originite => "originite",
            Self::Lmd => "lmd",
            Self::HeadhuntingPermits => "headhunting_permits",
            Self::TenPullPermits => "ten_pull_permits",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.as_str() == id)
    }

    /// Whether the field can be a stat strip card. The level, the date and
    /// the sign-in tally have their own places in the header and are not a
    /// single number.
    pub const fn is_strip_card(self) -> bool {
        !matches!(self, Self::Level | Self::Registered | Self::SignIn)
    }

    /// A SQL predicate, true unless the `profile_layout` jsonb in `column`
    /// hides this field: for the queries that rank or sort players by it,
    /// which must drop a player who hid it just as `/get-user` nulls it for a
    /// visitor. The twin of [`ProfileTabId::visible_sql`]; containment is exact
    /// because a stored `hidden` list is normalized. The id is a fixed literal,
    /// never user input.
    pub fn visible_sql(self, column: &str) -> String {
        format!(
            "NOT COALESCE({column} -> 'header' -> 'hidden' @> '[\"{}\"]'::jsonb, false)",
            self.as_str()
        )
    }

    /// [`Self::visible_sql`] for a query that has the player's id but no
    /// `user_settings` row joined: true unless `user_id`'s stored layout hides
    /// this field. A player with no settings row hides nothing.
    pub fn visible_for_sql(self, user_id: &str) -> String {
        format!(
            "NOT EXISTS (SELECT 1 FROM user_settings hs WHERE hs.user_id = {user_id} AND hs.profile_layout -> 'header' -> 'hidden' @> '[\"{}\"]'::jsonb)",
            self.as_str()
        )
    }

    /// `value` where `user_id` shows this field, else NULL: for the ranking
    /// rows that carry a header number beside the thing they rank by.
    pub fn shown_sql(self, user_id: &str, value: &str) -> String {
        format!(
            "CASE WHEN {} THEN {value} END",
            self.visible_for_sql(user_id)
        )
    }
}

/// Most cards the stat strip holds, the five it had before it was editable.
pub const HEADER_STRIP_MAX: usize = 5;

/// `profile_layout.header`: which header numbers visitors receive, and which
/// fill the stat strip. Absent, the header and strip render exactly as they did
/// before this existed.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileHeader {
    /// Fields a visitor does not receive, in canonical order, each once. The
    /// owner still sees them, marked.
    pub hidden: Vec<ProfileHeaderField>,
    /// The strip's cards, in display order, at most [`HEADER_STRIP_MAX`], each
    /// once, strip cards only. `None` is [`ProfileHeaderField::DEFAULT_STRIP`];
    /// an empty list is no strip at all.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub strip: Option<Vec<ProfileHeaderField>>,
}

impl ProfileHeader {
    /// `raw` as a header, or `None` when it is not an object. Unknown and
    /// repeated ids are dropped from both lists; `hidden` is put in canonical
    /// order; `strip` drops ids that are not strip cards and is cut to
    /// [`HEADER_STRIP_MAX`] (the first win). A `strip` that is not an array is
    /// no strip key, the default strip.
    pub fn normalize(raw: &serde_json::Value) -> Option<Self> {
        if !raw.is_object() {
            return None;
        }
        let ids = |field: &str| {
            array_field(raw, field)
                .iter()
                .filter_map(|id| id.as_str().and_then(ProfileHeaderField::from_id))
                .collect::<Vec<_>>()
        };
        let named = ids("hidden");
        let hidden = ProfileHeaderField::ALL
            .into_iter()
            .filter(|field| named.contains(field))
            .collect();
        let strip = raw.get("strip").filter(|s| s.is_array()).map(|_| {
            let mut strip: Vec<ProfileHeaderField> = Vec::with_capacity(HEADER_STRIP_MAX);
            for field in ids("strip") {
                if strip.len() < HEADER_STRIP_MAX
                    && field.is_strip_card()
                    && !strip.contains(&field)
                {
                    strip.push(field);
                }
            }
            strip
        });
        Some(Self { hidden, strip })
    }

    pub fn is_hidden(&self, field: ProfileHeaderField) -> bool {
        self.hidden.contains(&field)
    }

    /// The cards the owner arranged, the default five when they never did.
    pub fn strip_cards(&self) -> Vec<ProfileHeaderField> {
        self.strip
            .clone()
            .unwrap_or_else(|| ProfileHeaderField::DEFAULT_STRIP.to_vec())
    }

    /// The header as a visitor receives it: the strip resolved and without its
    /// hidden cards, so a hidden number has no card that would read "-". The
    /// `hidden` list is kept: it names what the nulled fields already show is
    /// missing, and tells the client a hidden date from an unsynced one.
    #[must_use]
    pub fn visible_only(&self) -> Self {
        Self {
            hidden: self.hidden.clone(),
            strip: Some(
                self.strip_cards()
                    .into_iter()
                    .filter(|field| !self.is_hidden(*field))
                    .collect(),
            ),
        }
    }
}

/// A save of `profile_layout`: only the keys the request carried, each
/// normalized as [`ProfileLayout::normalize`] reads it. A key left out keeps
/// the stored value, so the tab editor sends only `tabs`, the showcase editor
/// only `showcase`, the background picker only `background` and the header
/// editor only `header`, and none can wipe another's work (see
/// `queries::users::update_settings`). The one save that clears the whole
/// layout is the tab editor's explicit Reset, which sends `null` instead of a
/// patch.
///
/// `tabs` is present when the request carried a `tabs` array, `showcase` when
/// it carried a `showcase` object. `background` and `header` are present
/// whenever their key is: `null`, or a value that does not normalize, clears
/// it.
#[derive(utoipa::ToSchema, Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "serde_json::Value")]
pub struct ProfileLayoutPatch {
    #[schema(required = false)]
    pub tabs: Option<Vec<ProfileTab>>,
    #[schema(required = false)]
    pub showcase: Option<ProfileShowcase>,
    #[schema(value_type = Option<ProfileBackground>, required = false)]
    pub background: Option<Option<ProfileBackground>>,
    #[schema(value_type = Option<ProfileHeader>, required = false)]
    pub header: Option<Option<ProfileHeader>>,
}

impl From<serde_json::Value> for ProfileLayoutPatch {
    fn from(raw: serde_json::Value) -> Self {
        Self::normalize(&raw)
    }
}

impl ProfileLayoutPatch {
    /// The keys `raw` carries, normalized. Total: a value that is not an
    /// object carries nothing, a save that changes nothing.
    pub fn normalize(raw: &serde_json::Value) -> Self {
        Self {
            tabs: raw
                .get("tabs")
                .filter(|t| t.is_array())
                .map(|_| ProfileLayout::normalize(raw).tabs),
            showcase: raw
                .get("showcase")
                .filter(|s| s.is_object())
                .map(ProfileShowcase::normalize),
            background: raw.get("background").map(ProfileBackground::normalize),
            header: raw.get("header").map(ProfileHeader::normalize),
        }
    }

    /// The object merged over the stored layout: every key the save sets.
    pub fn merge_json(&self) -> serde_json::Value {
        let mut out = serde_json::Map::new();
        if let Some(tabs) = &self.tabs {
            out.insert("tabs".into(), serde_json::json!(tabs));
        }
        if let Some(showcase) = &self.showcase {
            out.insert("showcase".into(), serde_json::json!(showcase));
        }
        if let Some(Some(background)) = &self.background {
            out.insert("background".into(), serde_json::json!(background));
        }
        if let Some(Some(header)) = &self.header {
            out.insert("header".into(), serde_json::json!(header));
        }
        serde_json::Value::Object(out)
    }

    /// The keys the save removes from the stored layout: a cleared background
    /// or header.
    pub fn removed_keys(&self) -> Vec<&'static str> {
        let mut keys = Vec::new();
        if matches!(self.background, Some(None)) {
            keys.push("background");
        }
        if matches!(self.header, Some(None)) {
            keys.push("header");
        }
        keys
    }

    /// The layout `stored` becomes under this save, the twin of the SQL in
    /// `queries::users::update_settings`: stored keys the save leaves out
    /// survive, a cleared background or header goes, and an empty result is no
    /// layout.
    pub fn apply(&self, stored: Option<&serde_json::Value>) -> Option<serde_json::Value> {
        let mut out = stored
            .and_then(serde_json::Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let serde_json::Value::Object(set) = self.merge_json() {
            out.extend(set);
        }
        for key in self.removed_keys() {
            out.remove(key);
        }
        (!out.is_empty()).then_some(serde_json::Value::Object(out))
    }
}

impl Default for ProfileLayout {
    /// The layout a NULL column means: canonical order, every tab visible.
    fn default() -> Self {
        Self::normalize(&serde_json::Value::Null)
    }
}

impl From<serde_json::Value> for ProfileLayout {
    fn from(raw: serde_json::Value) -> Self {
        Self::normalize(&raw)
    }
}

impl ProfileLayout {
    /// The layout `raw` asks for, made whole. An entry needs a known string
    /// `id`; anything else is dropped, as is a second entry for the same tab
    /// (the first wins). Every known tab still missing is appended, visible,
    /// in canonical order, which is how a tab added to the code later reaches
    /// a profile saved before it. A missing or non-boolean `visible` reads as
    /// visible, today's behavior. A missing Showcase tab is the exception:
    /// it goes first, its canonical place. `showcase` is read by
    /// [`ProfileShowcase::normalize`] when it is an object, else is `None`;
    /// `header` by [`ProfileHeader::normalize`] the same way.
    ///
    /// Total: a value that is not an object, or has no `tabs` array, is the
    /// default layout. No input fails, so a malformed row never takes the
    /// profile down with it.
    pub fn normalize(raw: &serde_json::Value) -> Self {
        let mut tabs: Vec<ProfileTab> = Vec::with_capacity(ProfileTabId::ALL.len());
        for entry in array_field(raw, "tabs") {
            let Some(id) = str_field(entry, "id").and_then(ProfileTabId::from_id) else {
                continue;
            };
            if tabs.iter().any(|t| t.id == id) {
                continue;
            }
            let visible = entry
                .get("visible")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);
            tabs.push(ProfileTab { id, visible });
        }
        for id in ProfileTabId::ALL {
            if !tabs.iter().any(|t| t.id == id) {
                // The Showcase tab came after layouts did: a layout saved
                // before it gets it in its canonical place, first. It shows a
                // visitor nothing until it has a block, so this moves no tab
                // a visitor sees.
                let at = if id == ProfileTabId::Showcase {
                    0
                } else {
                    tabs.len()
                };
                tabs.insert(at, ProfileTab { id, visible: true });
            }
        }
        let showcase = raw
            .get("showcase")
            .filter(|s| s.is_object())
            .map(ProfileShowcase::normalize);
        let background = raw.get("background").and_then(ProfileBackground::normalize);
        let header = raw.get("header").and_then(ProfileHeader::normalize);
        Self {
            tabs,
            showcase,
            background,
            header,
        }
    }

    /// Whether a visitor may open `tab`. A tab absent from the list counts as
    /// hidden, which only a [`Self::visible_only`] projection can produce.
    pub fn is_visible(&self, tab: ProfileTabId) -> bool {
        self.tabs.iter().any(|t| t.id == tab && t.visible)
    }

    /// The layout as a visitor receives it: the visible tabs in order, and
    /// nothing about the private ones, not even that they exist.
    ///
    /// The showcase follows its tab: absent when the Showcase tab is private,
    /// and without its plan blocks when the Plans tab is, since a plan block
    /// shows that tab's data. The background is kept: it belongs to no tab.
    /// The header goes through [`ProfileHeader::visible_only`]; the fields it
    /// hides are nulled on the profile row itself, by
    /// `UserProfile::hide_private`.
    #[must_use]
    pub fn visible_only(&self) -> Self {
        let showcase = self
            .showcase
            .as_ref()
            .filter(|_| self.is_visible(ProfileTabId::Showcase))
            .map(|showcase| {
                let plans = self.is_visible(ProfileTabId::Plans);
                ProfileShowcase {
                    blocks: showcase
                        .blocks
                        .iter()
                        .filter(|b| plans || !matches!(b, ShowcaseBlock::Plan { .. }))
                        .cloned()
                        .collect(),
                }
            });
        Self {
            tabs: self.tabs.iter().copied().filter(|t| t.visible).collect(),
            showcase,
            background: self.background.clone(),
            header: self.header.as_ref().map(ProfileHeader::visible_only),
        }
    }

    /// Whether a visitor may receive `field`. No header hides nothing.
    pub fn header_shows(&self, field: ProfileHeaderField) -> bool {
        !self.header.as_ref().is_some_and(|h| h.is_hidden(field))
    }

    /// The showcase's blocks, none when there is no showcase.
    pub fn showcase_blocks(&self) -> &[ShowcaseBlock] {
        self.showcase.as_ref().map_or(&[], |s| s.blocks.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn ids(layout: &ProfileLayout) -> Vec<(&'static str, bool)> {
        layout
            .tabs
            .iter()
            .map(|t| (t.id.as_str(), t.visible))
            .collect()
    }

    const CANONICAL: [(&str, bool); 8] = [
        ("showcase", true),
        ("stats", true),
        ("score", true),
        ("roster", true),
        ("plans", true),
        ("inventory", true),
        ("enemies", true),
        ("optimizer", true),
    ];

    #[test]
    fn null_is_the_canonical_layout() {
        assert_eq!(ids(&ProfileLayout::normalize(&json!(null))), CANONICAL);
        assert_eq!(ids(&ProfileLayout::default()), CANONICAL);
    }

    #[test]
    fn an_empty_list_or_a_shapeless_value_is_the_canonical_layout() {
        for raw in [
            json!({ "tabs": [] }),
            json!({}),
            json!({ "tabs": "stats" }),
            json!([{ "id": "roster", "visible": false }]),
            json!(7),
        ] {
            assert_eq!(ids(&ProfileLayout::normalize(&raw)), CANONICAL, "{raw}");
        }
    }

    #[test]
    fn unknown_ids_and_malformed_entries_are_dropped() {
        let raw = json!({ "tabs": [
            { "id": "achievements", "visible": false },
            { "id": "roster", "visible": false },
            { "id": 3, "visible": false },
            { "visible": false },
            "score",
        ] });
        let layout = ProfileLayout::normalize(&raw);
        assert_eq!(
            ids(&layout),
            [
                ("showcase", true),
                ("roster", false),
                ("stats", true),
                ("score", true),
                ("plans", true),
                ("inventory", true),
                ("enemies", true),
                ("optimizer", true),
            ]
        );
    }

    #[test]
    fn a_duplicate_keeps_the_first_entry() {
        let raw = json!({ "tabs": [
            { "id": "plans", "visible": false },
            { "id": "stats", "visible": true },
            { "id": "plans", "visible": true },
        ] });
        let layout = ProfileLayout::normalize(&raw);
        assert_eq!(
            layout.tabs[1],
            ProfileTab {
                id: ProfileTabId::Plans,
                visible: false
            }
        );
        assert_eq!(
            layout
                .tabs
                .iter()
                .filter(|t| t.id == ProfileTabId::Plans)
                .count(),
            1
        );
        assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
    }

    #[test]
    fn missing_tabs_are_appended_visible_in_canonical_order() {
        let raw = json!({ "tabs": [
            { "id": "optimizer", "visible": false },
            { "id": "inventory", "visible": true },
        ] });
        assert_eq!(
            ids(&ProfileLayout::normalize(&raw)),
            [
                ("showcase", true),
                ("optimizer", false),
                ("inventory", true),
                ("stats", true),
                ("score", true),
                ("roster", true),
                ("plans", true),
                ("enemies", true),
            ]
        );
    }

    #[test]
    fn a_missing_visible_flag_reads_as_visible() {
        let layout = ProfileLayout::normalize(&json!({ "tabs": [{ "id": "score" }] }));
        assert!(layout.is_visible(ProfileTabId::Score));
    }

    #[test]
    fn sibling_keys_do_not_disturb_tabs() {
        let raw = json!({ "showcase": [1, 2], "tabs": [{ "id": "enemies", "visible": false }] });
        let layout = ProfileLayout::normalize(&raw);
        // Not an object: no showcase, rather than an empty one.
        assert_eq!(layout.showcase, None);
        assert_eq!(
            layout.tabs[1],
            ProfileTab {
                id: ProfileTabId::Enemies,
                visible: false
            }
        );
    }

    #[test]
    fn a_normalized_layout_is_a_fixed_point() {
        let raw = json!({ "tabs": [
            { "id": "roster", "visible": false },
            { "id": "nope" },
            { "id": "score", "visible": true },
        ] });
        let once = ProfileLayout::normalize(&raw);
        let twice = ProfileLayout::normalize(&serde_json::to_value(&once).unwrap());
        assert_eq!(once, twice);
    }

    #[test]
    fn deserializing_normalizes() {
        let layout: ProfileLayout = serde_json::from_value(
            json!({ "tabs": [{ "id": "plans", "visible": false }, { "id": "x" }] }),
        )
        .unwrap();
        assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
        assert_eq!(
            layout.tabs[1],
            ProfileTab {
                id: ProfileTabId::Plans,
                visible: false
            }
        );
    }

    #[test]
    fn the_visitor_projection_drops_private_tabs_and_keeps_order() {
        let raw = json!({ "tabs": [
            { "id": "roster", "visible": true },
            { "id": "stats", "visible": false },
            { "id": "score", "visible": false },
        ] });
        let layout = ProfileLayout::normalize(&raw);
        let visitor = layout.visible_only();
        assert_eq!(
            ids(&visitor),
            [
                ("showcase", true),
                ("roster", true),
                ("plans", true),
                ("inventory", true),
                ("enemies", true),
                ("optimizer", true),
            ]
        );
        assert!(!layout.is_visible(ProfileTabId::Stats));
        assert!(layout.is_visible(ProfileTabId::Roster));
        assert!(!visitor.is_visible(ProfileTabId::Score));
    }

    #[test]
    fn the_sql_predicate_matches_the_stored_shape() {
        assert_eq!(
            ProfileTabId::Roster.visible_sql("us.profile_layout"),
            r#"NOT COALESCE(us.profile_layout -> 'tabs' @> '[{"id":"roster","visible":false}]'::jsonb, false)"#
        );
        // The literal must be what a normalized private entry serializes to.
        let stored = serde_json::to_string(&ProfileTab {
            id: ProfileTabId::Roster,
            visible: false,
        })
        .unwrap();
        assert!(
            ProfileTabId::Roster
                .visible_sql("c")
                .contains(&format!("[{stored}]"))
        );
    }

    #[test]
    fn the_leaderboard_view_gates_on_the_score_tab() {
        let sql = include_str!("../migrations/v034_leaderboard_private_score.sql");
        assert!(sql.contains(&ProfileTabId::Score.visible_sql("us.profile_layout")));
    }

    #[test]
    fn the_leaderboard_view_masks_a_hidden_level() {
        let sql = include_str!("../migrations/v038_leaderboard_hidden_level.sql");
        assert!(sql.contains(&format!(
            "{} AS level",
            ProfileHeaderField::Level.shown_sql("u.id", "u.level")
        )));
        // The v034 gate is carried over verbatim.
        assert!(sql.contains(&ProfileTabId::Score.visible_sql("us.profile_layout")));
    }

    #[test]
    fn the_leaderboard_view_ranks_ungraded_players_last() {
        // v041 replaces the whole view, so v038's mask and v034's gate must
        // ride along, and both rank windows put a NULL score last.
        let sql = include_str!("../migrations/v041_leaderboard_unscored_last.sql");
        assert!(sql.contains(&format!(
            "{} AS level",
            ProfileHeaderField::Level.shown_sql("u.id", "u.level")
        )));
        assert!(sql.contains(&ProfileTabId::Score.visible_sql("us.profile_layout")));
        assert_eq!(sql.matches("total_score DESC NULLS LAST").count(), 2);
        assert!(!sql.contains("total_score DESC)"));
    }

    fn kinds(layout: &ProfileLayout) -> Vec<&'static str> {
        layout
            .showcase_blocks()
            .iter()
            .map(|b| match b {
                ShowcaseBlock::Favourites { .. } => "favourites",
                ShowcaseBlock::Grid { .. } => "grid",
                ShowcaseBlock::TierList { .. } => "tier_list",
                ShowcaseBlock::Plan { .. } => "plan",
            })
            .collect()
    }

    const PLAN: &str = "6f1c2e7a-3b44-4d1e-9a52-0c8f7d2b1e90";

    #[test]
    fn a_layout_without_a_showcase_key_has_no_showcase() {
        assert_eq!(ProfileLayout::default().showcase, None);
        let layout = ProfileLayout::normalize(&json!({ "tabs": [] }));
        assert_eq!(layout.showcase, None);
        // And serializes without the key, so a save keeps the stored one.
        assert!(
            serde_json::to_value(&layout)
                .unwrap()
                .get("showcase")
                .is_none()
        );
        for raw in [json!({ "showcase": null }), json!({ "showcase": "x" })] {
            assert_eq!(ProfileLayout::normalize(&raw).showcase, None, "{raw}");
        }
        // An object with no blocks is an empty showcase, which clears.
        assert_eq!(
            ProfileLayout::normalize(&json!({ "showcase": {} })).showcase,
            Some(ProfileShowcase::default())
        );
    }

    #[test]
    fn malformed_and_unknown_blocks_are_dropped() {
        let raw = json!({ "showcase": { "blocks": [
            { "type": "favourites", "entity_kind": "operator", "ids": ["char_002_amiya"] },
            { "type": "favourites", "entity_kind": "pokemon", "ids": ["pikachu"] },
            { "type": "favourites", "entity_kind": "skin", "ids": [] },
            { "type": "favourites", "entity_kind": "skin", "ids": [" ", 4, null] },
            { "type": "favourites", "ids": ["char_002_amiya"] },
            { "type": "badge", "slug": "x" },
            { "slug": "no-type" },
            { "type": "grid", "slug": "  " },
            { "type": "grid" },
            { "type": "grid", "slug": "my-grid" },
            { "type": "tier_list", "slug": 7 },
            { "type": "tier_list", "slug": "meta" },
            { "type": "plan", "id": "not-a-uuid" },
            { "type": "plan", "id": PLAN },
            { "type": "grid", "slug": "x".repeat(121) },
            "grid",
            12,
        ] } });
        let layout = ProfileLayout::normalize(&raw);
        assert_eq!(kinds(&layout), ["favourites", "grid", "tier_list", "plan"]);
        assert_eq!(
            layout.showcase_blocks()[3],
            ShowcaseBlock::Plan {
                id: PLAN.parse().unwrap(),
                operator_id: None,
            }
        );
    }

    #[test]
    fn favourites_ids_are_trimmed_deduplicated_and_clamped() {
        let mut ids: Vec<Value> = vec![json!(" char_002_amiya "), json!("char_002_amiya")];
        ids.extend((0..40).map(|i| json!(format!("id_{i}"))));
        let raw = json!({ "showcase": { "blocks": [
            { "type": "favourites", "entity_kind": "enemy", "ids": ids, "title": "  Who I fear\n most  " },
        ] } });
        let layout = ProfileLayout::normalize(&raw);
        let ShowcaseBlock::Favourites {
            entity_kind,
            ids,
            title,
        } = &layout.showcase_blocks()[0]
        else {
            panic!("favourites");
        };
        assert_eq!(*entity_kind, EntityKind::Enemy);
        assert_eq!(ids.len(), SHOWCASE_MAX_IDS);
        assert_eq!(ids[0], "char_002_amiya");
        assert_eq!(ids[1], "id_0");
        assert_eq!(ids.last().map(String::as_str), Some("id_22"));
        // The control character is dropped, not turned into a space.
        assert_eq!(title.as_deref(), Some("Who I fear most"));
    }

    #[test]
    fn titles_are_cut_to_forty_characters_and_a_blank_one_is_none() {
        let block = |title: Value| {
            let raw = json!({ "showcase": { "blocks": [
                { "type": "favourites", "entity_kind": "skin", "ids": ["a"], "title": title },
            ] } });
            match ProfileLayout::normalize(&raw).showcase_blocks()[0].clone() {
                ShowcaseBlock::Favourites { title, .. } => title,
                _ => panic!("favourites"),
            }
        };
        let long = "é".repeat(50);
        assert_eq!(
            block(json!(long)).map(|t| t.chars().count()),
            Some(SHOWCASE_TITLE_MAX)
        );
        assert_eq!(block(json!("   ")), None);
        assert_eq!(block(json!(3)), None);
        // Stored as typed: it is rendered as text, never as markup.
        assert_eq!(block(json!("<b>hi</b>")).as_deref(), Some("<b>hi</b>"));
    }

    #[test]
    fn blocks_are_clamped_and_a_repeated_referent_keeps_the_first() {
        let mut blocks: Vec<Value> = vec![
            json!({ "type": "grid", "slug": "a" }),
            json!({ "type": "grid", "slug": "a" }),
            json!({ "type": "tier_list", "slug": "a" }),
            json!({ "type": "plan", "id": PLAN }),
            json!({ "type": "plan", "id": PLAN.to_uppercase() }),
            json!({ "type": "favourites", "entity_kind": "operator", "ids": ["x"], "title": "one" }),
            json!({ "type": "favourites", "entity_kind": "operator", "ids": ["x"], "title": "two" }),
        ];
        blocks.extend((0..20).map(|i| json!({ "type": "grid", "slug": format!("g{i}") })));
        let layout = ProfileLayout::normalize(&json!({ "showcase": { "blocks": blocks } }));
        let got = kinds(&layout);
        assert_eq!(got.len(), SHOWCASE_MAX_BLOCKS);
        // The grid repeat and the plan repeat (same uuid, other case) are
        // dropped; two operator rows are two blocks.
        assert_eq!(
            &got[..5],
            ["grid", "tier_list", "plan", "favourites", "favourites"]
        );
        assert_eq!(
            layout.showcase_blocks()[5],
            ShowcaseBlock::Grid { slug: "g0".into() }
        );
    }

    #[test]
    fn tabs_and_showcase_round_trip() {
        let raw = json!({
            "tabs": [{ "id": "roster", "visible": true }, { "id": "showcase", "visible": true }],
            "showcase": { "blocks": [
                { "type": "favourites", "entity_kind": "main_story", "ids": ["main_0", "main_1"], "title": "Faves" },
                { "type": "favourites", "entity_kind": "skin", "ids": ["char_002_amiya@epoque#4"] },
                { "type": "grid", "slug": "my-grid" },
                { "type": "tier_list", "slug": "meta" },
                { "type": "plan", "id": PLAN },
            ] },
        });
        let once = ProfileLayout::normalize(&raw);
        let wire = serde_json::to_value(&once).unwrap();
        assert_eq!(
            wire["showcase"], raw["showcase"],
            "a clean showcase is stored as sent"
        );
        assert_eq!(wire["tabs"][0]["id"], json!("roster"));
        assert_eq!(wire["tabs"][1]["id"], json!("showcase"));
        let twice: ProfileLayout = serde_json::from_value(wire).unwrap();
        assert_eq!(once, twice);
    }

    #[test]
    fn the_old_tabs_only_shape_still_reads() {
        let raw = json!({ "tabs": [{ "id": "plans", "visible": false }] });
        let layout: ProfileLayout = serde_json::from_value(raw).unwrap();
        assert_eq!(layout.showcase, None);
        assert_eq!(layout.tabs[0].id, ProfileTabId::Showcase);
        assert_eq!(
            layout.tabs[1],
            ProfileTab {
                id: ProfileTabId::Plans,
                visible: false
            }
        );
        assert_eq!(layout.tabs.len(), ProfileTabId::ALL.len());
    }

    #[test]
    fn the_visitor_projection_gates_the_showcase_on_its_tab_and_plans_on_theirs() {
        let showcase = json!({ "blocks": [
            { "type": "grid", "slug": "g" },
            { "type": "plan", "id": PLAN },
        ] });
        let with = |tabs: Value| {
            ProfileLayout::normalize(&json!({ "tabs": tabs, "showcase": showcase })).visible_only()
        };
        assert_eq!(kinds(&with(json!([]))), ["grid", "plan"]);
        assert_eq!(
            kinds(&with(json!([{ "id": "plans", "visible": false }]))),
            ["grid"]
        );
        let hidden = with(json!([{ "id": "showcase", "visible": false }]));
        assert_eq!(hidden.showcase, None);
        assert!(!hidden.is_visible(ProfileTabId::Showcase));
    }

    #[test]
    fn wire_ids_round_trip_through_serde() {
        for tab in ProfileTabId::ALL {
            let wire = serde_json::to_value(tab).unwrap();
            assert_eq!(wire, json!(tab.as_str()));
            assert_eq!(ProfileTabId::from_id(tab.as_str()), Some(tab));
        }
    }

    fn background(raw: Value) -> Option<ProfileBackground> {
        ProfileLayout::normalize(&json!({ "background": raw })).background
    }

    #[test]
    fn a_layout_without_a_background_has_none_and_serializes_without_the_key() {
        assert_eq!(ProfileLayout::default().background, None);
        let layout = ProfileLayout::normalize(&json!({ "tabs": [] }));
        assert_eq!(layout.background, None);
        assert!(
            serde_json::to_value(&layout)
                .unwrap()
                .get("background")
                .is_none()
        );
    }

    #[test]
    fn a_background_keeps_its_kind_id_and_focus() {
        assert_eq!(
            background(
                json!({ "kind": "skin", "id": " char_002_amiya@epoque#4 ", "focus_x": 30, "focus_y": 12.4 })
            ),
            Some(ProfileBackground {
                kind: ProfileBackgroundKind::Skin,
                id: "char_002_amiya@epoque#4".into(),
                focus_x: Some(30),
                focus_y: Some(12),
                scale: None,
                elite: None,
            })
        );
        assert_eq!(
            background(json!({ "kind": "operator", "id": "char_003_kalts" })),
            Some(ProfileBackground {
                kind: ProfileBackgroundKind::Operator,
                id: "char_003_kalts".into(),
                focus_x: None,
                focus_y: None,
                scale: None,
                elite: None,
            })
        );
    }

    #[test]
    fn a_gallery_background_keeps_its_case_sensitive_id_and_both_focus_axes() {
        let bg = background(
            json!({ "kind": "archive_pic", "id": " pic_rogue_1_KV1 ", "focus_x": 72, "focus_y": 40 }),
        );
        assert_eq!(
            bg,
            Some(ProfileBackground {
                kind: ProfileBackgroundKind::ArchivePic,
                id: "pic_rogue_1_KV1".into(),
                focus_x: Some(72),
                focus_y: Some(40),
                scale: None,
                elite: None,
            })
        );
        assert_eq!(
            serde_json::to_value(bg.unwrap()).unwrap(),
            json!({ "kind": "archive_pic", "id": "pic_rogue_1_KV1", "focus_x": 72, "focus_y": 40 })
        );
        assert_eq!(ProfileBackgroundKind::ArchivePic.entity_kind(), None);
        for kind in ProfileBackgroundKind::ALL {
            assert_eq!(ProfileBackgroundKind::from_id(kind.as_str()), Some(kind));
        }
        assert_eq!(
            background(json!({ "kind": "gallery", "id": "act13side_pic_0" })),
            None
        );
    }

    #[test]
    fn a_story_art_background_keeps_its_id_as_sent_and_has_no_entity_kind() {
        for (wire, kind) in [
            ("story_cg", ProfileBackgroundKind::StoryCg),
            ("story_scene", ProfileBackgroundKind::StoryScene),
        ] {
            let bg = background(
                json!({ "kind": wire, "id": " avg_1_1 ", "focus_x": 20, "focus_y": 80, "scale": 140 }),
            );
            assert_eq!(
                bg,
                Some(ProfileBackground {
                    kind,
                    id: "avg_1_1".into(),
                    focus_x: Some(20),
                    focus_y: Some(80),
                    scale: Some(140),
                    elite: None,
                })
            );
            assert_eq!(
                serde_json::to_value(bg.unwrap()).unwrap(),
                json!({ "kind": wire, "id": "avg_1_1", "focus_x": 20, "focus_y": 80, "scale": 140 })
            );
            assert_eq!(kind.entity_kind(), None);
            assert_eq!(ProfileBackgroundKind::from_id(wire), Some(kind));
        }
        // The case is kept: the catalogue decides, and its ids are lowercase.
        assert_eq!(
            background(json!({ "kind": "story_cg", "id": "AVG_1_1" })).map(|b| b.id),
            Some("AVG_1_1".into())
        );
        assert_eq!(
            background(json!({ "kind": "story_art", "id": "avg_1_1" })),
            None
        );
    }

    #[test]
    fn a_bad_background_is_dropped_and_focus_is_clamped() {
        for raw in [
            json!(null),
            json!("char_002_amiya"),
            json!({ "kind": "story", "id": "main_0" }),
            json!({ "kind": "enemy", "id": "enemy_1007_slime" }),
            json!({ "id": "char_002_amiya" }),
            json!({ "kind": "operator" }),
            json!({ "kind": "operator", "id": "   " }),
            json!({ "kind": "operator", "id": 7 }),
            json!({ "kind": "operator", "id": "x".repeat(121) }),
        ] {
            assert_eq!(background(raw.clone()), None, "{raw}");
        }
        let bg =
            background(json!({ "kind": "operator", "id": "a", "focus_x": -40, "focus_y": 250 }))
                .unwrap();
        assert_eq!((bg.focus_x, bg.focus_y), (Some(0), Some(100)));
        let bg =
            background(json!({ "kind": "operator", "id": "a", "focus_x": "50", "focus_y": null }))
                .unwrap();
        assert_eq!((bg.focus_x, bg.focus_y), (None, None));
        let bg =
            background(json!({ "kind": "operator", "id": "a", "focus_x": 99.5, "focus_y": 0.49 }))
                .unwrap();
        assert_eq!((bg.focus_x, bg.focus_y), (Some(100), Some(0)));
    }

    #[test]
    fn an_operator_background_keeps_elite_1_or_2_and_nothing_else_does() {
        let elite = |raw: Value| background(raw).unwrap().elite;
        assert_eq!(
            elite(json!({ "kind": "operator", "id": "a", "elite": 1 })),
            Some(1)
        );
        assert_eq!(
            elite(json!({ "kind": "operator", "id": "a", "elite": 2 })),
            Some(2)
        );
        for bad in [
            json!(0),
            json!(3),
            json!(1.5),
            json!("2"),
            json!(-1),
            json!(null),
        ] {
            assert_eq!(
                elite(json!({ "kind": "operator", "id": "a", "elite": bad })),
                None
            );
        }
        assert_eq!(elite(json!({ "kind": "operator", "id": "a" })), None);
        for kind in ["skin", "archive_pic", "story_cg", "story_scene"] {
            assert_eq!(
                elite(json!({ "kind": kind, "id": "a@b", "elite": 1 })),
                None
            );
        }
        // Absent, the wire shape is what it was before the choice existed.
        let plain = background(json!({ "kind": "operator", "id": "a", "focus_y": 25 })).unwrap();
        assert_eq!(
            serde_json::to_value(plain).unwrap(),
            json!({ "kind": "operator", "id": "a", "focus_y": 25 })
        );
        let e1 = background(json!({ "kind": "operator", "id": "a", "elite": 1 })).unwrap();
        assert_eq!(
            serde_json::to_value(e1).unwrap(),
            json!({ "kind": "operator", "id": "a", "elite": 1 })
        );
    }

    #[test]
    fn a_background_scale_is_rounded_clamped_and_absent_when_not_a_number() {
        let scale = |raw: Value| {
            background(json!({ "kind": "operator", "id": "a", "scale": raw }))
                .unwrap()
                .scale
        };
        assert_eq!(scale(json!(150)), Some(150));
        assert_eq!(scale(json!(149.5)), Some(150));
        assert_eq!(scale(json!(100)), Some(100));
        assert_eq!(scale(json!(300)), Some(300));
        assert_eq!(scale(json!(40)), Some(BACKGROUND_SCALE_MIN));
        assert_eq!(scale(json!(-7)), Some(BACKGROUND_SCALE_MIN));
        assert_eq!(scale(json!(1e9)), Some(BACKGROUND_SCALE_MAX));
        assert_eq!(scale(json!("150")), None);
        assert_eq!(scale(json!(null)), None);
        assert_eq!(scale(json!(true)), None);
        let bare = background(json!({ "kind": "operator", "id": "a" })).unwrap();
        assert_eq!(bare.scale, None);
        // Absent stays absent on the way out, so a pre-zoom background
        // serializes exactly as it did.
        assert_eq!(
            serde_json::to_value(&bare).unwrap(),
            json!({ "kind": "operator", "id": "a" })
        );
        let zoomed = background(
            json!({ "kind": "archive_pic", "id": "p", "focus_x": 10, "focus_y": 90, "scale": 210 }),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&zoomed).unwrap(),
            json!({ "kind": "archive_pic", "id": "p", "focus_x": 10, "focus_y": 90, "scale": 210 })
        );
    }

    #[test]
    fn a_background_round_trips_and_survives_the_visitor_projection() {
        let raw = json!({
            "tabs": [{ "id": "showcase", "visible": false }],
            "background": { "kind": "skin", "id": "char_002_amiya@epoque#4", "focus_x": 40, "focus_y": 20 },
        });
        let once = ProfileLayout::normalize(&raw);
        let wire = serde_json::to_value(&once).unwrap();
        assert_eq!(wire["background"], raw["background"]);
        assert_eq!(ProfileLayout::normalize(&wire), once);
        // Every tab private or not, the header keeps its art.
        assert_eq!(once.visible_only().background, once.background);
    }

    #[test]
    fn a_patch_carries_only_the_keys_sent() {
        let patch = ProfileLayoutPatch::normalize(
            &json!({ "background": { "kind": "operator", "id": "char_003_kalts" } }),
        );
        assert_eq!(patch.tabs, None);
        assert_eq!(patch.showcase, None);
        assert!(matches!(patch.background, Some(Some(_))));
        assert_eq!(
            patch.merge_json(),
            json!({ "background": { "kind": "operator", "id": "char_003_kalts" } })
        );
        assert!(patch.removed_keys().is_empty());

        // A shapeless value carries nothing; non-array tabs and non-object
        // showcases are not sent.
        for raw in [json!(7), json!({}), json!({ "tabs": "x", "showcase": [1] })] {
            assert_eq!(
                ProfileLayoutPatch::normalize(&raw),
                ProfileLayoutPatch::default(),
                "{raw}"
            );
        }

        // Tabs sent are normalized whole, as a full layout's are.
        let patch = ProfileLayoutPatch::normalize(
            &json!({ "tabs": [{ "id": "roster", "visible": false }] }),
        );
        assert_eq!(
            patch.tabs.as_ref().map(Vec::len),
            Some(ProfileTabId::ALL.len())
        );
        assert_eq!(patch.background, None);

        // Null, or a background that does not normalize, clears.
        for raw in [json!(null), json!({ "kind": "story", "id": "x" })] {
            let patch = ProfileLayoutPatch::normalize(&json!({ "background": raw }));
            assert_eq!(patch.background, Some(None), "{raw}");
            assert_eq!(patch.merge_json(), json!({}));
            assert_eq!(patch.removed_keys(), ["background"]);
        }
    }

    #[test]
    fn a_background_save_keeps_tabs_and_showcase_and_null_clears_it() {
        let stored = json!({
            "tabs": [{ "id": "roster", "visible": false }],
            "showcase": { "blocks": [{ "type": "grid", "slug": "g" }] },
        });
        let set = ProfileLayoutPatch::normalize(
            &json!({ "background": { "kind": "skin", "id": "s@a#1", "focus_y": 30 } }),
        );
        let after = set.apply(Some(&stored)).unwrap();
        assert_eq!(after["tabs"], stored["tabs"]);
        assert_eq!(after["showcase"], stored["showcase"]);
        assert_eq!(
            after["background"],
            json!({ "kind": "skin", "id": "s@a#1", "focus_y": 30 })
        );

        // A tabs-only save keeps the background and the showcase.
        let tabs = ProfileLayoutPatch::normalize(&json!({ "tabs": [] }));
        let kept = tabs.apply(Some(&after)).unwrap();
        assert_eq!(kept["background"], after["background"]);
        assert_eq!(kept["showcase"], stored["showcase"]);

        // Null clears the background alone.
        let clear = ProfileLayoutPatch::normalize(&json!({ "background": null }));
        let cleared = clear.apply(Some(&after)).unwrap();
        assert_eq!(cleared, stored);

        // A background on a NULL row is a layout of that key alone, and
        // clearing it leaves no layout, the NULL a never-customized row has.
        let alone = set.apply(None).unwrap();
        assert_eq!(alone.as_object().map(serde_json::Map::len), Some(1));
        assert_eq!(clear.apply(Some(&alone)), None);
        assert_eq!(
            ProfileLayout::normalize(&alone).tabs.len(),
            ProfileTabId::ALL.len()
        );
    }

    fn header(raw: Value) -> Option<ProfileHeader> {
        ProfileLayout::normalize(&json!({ "header": raw })).header
    }

    fn wire(fields: &[ProfileHeaderField]) -> Vec<&'static str> {
        fields.iter().map(|f| f.as_str()).collect()
    }

    #[test]
    fn a_layout_without_a_header_has_none_and_serializes_without_the_key() {
        assert_eq!(ProfileLayout::default().header, None);
        let layout = ProfileLayout::normalize(&json!({ "tabs": [], "background": null }));
        assert_eq!(layout.header, None);
        assert_eq!(layout.visible_only().header, None);
        assert!(
            serde_json::to_value(&layout)
                .unwrap()
                .get("header")
                .is_none()
        );
        for field in ProfileHeaderField::ALL {
            assert!(layout.header_shows(field), "{}", field.as_str());
        }
    }

    #[test]
    fn a_header_that_is_not_an_object_is_none_and_shapeless_lists_are_empty() {
        for raw in [
            json!(null),
            json!("originite"),
            json!(["originite"]),
            json!(3),
        ] {
            assert_eq!(header(raw.clone()), None, "{raw}");
        }
        // An object is a header, however little of it reads.
        assert_eq!(header(json!({})), Some(ProfileHeader::default()));
        assert_eq!(
            header(json!({ "hidden": "originite", "strip": { "0": "lmd" } })),
            Some(ProfileHeader::default())
        );
    }

    #[test]
    fn hidden_drops_unknown_ids_and_repeats_and_keeps_canonical_order() {
        let got = header(json!({ "hidden": [
            "originite", "uid", "nickname", 7, null, "lmd", "originite", "level", " lmd",
        ] }))
        .unwrap();
        assert_eq!(wire(&got.hidden), ["level", "originite", "lmd"]);
        assert!(got.is_hidden(ProfileHeaderField::Originite));
        assert!(!got.is_hidden(ProfileHeaderField::Orundum));
    }

    #[test]
    fn the_strip_drops_unknown_repeated_and_non_card_ids_and_is_cut_to_five() {
        let got = header(json!({ "strip": [
            "lmd", "registered", "lmd", "nope", "level", "originite", "sign_in",
            "ten_pull_permits", "operators", "skins", "items", "orundum",
        ] }))
        .unwrap();
        assert_eq!(
            wire(got.strip.as_deref().unwrap()),
            ["lmd", "originite", "ten_pull_permits", "operators", "skins"]
        );
        assert_eq!(got.strip.as_ref().map(Vec::len), Some(HEADER_STRIP_MAX));
        // An empty list is no strip; no list is the default one.
        assert_eq!(header(json!({ "strip": [] })).unwrap().strip, Some(vec![]));
        assert_eq!(header(json!({ "strip": "lmd" })).unwrap().strip, None);
        assert_eq!(
            wire(&header(json!({})).unwrap().strip_cards()),
            ["operators", "skins", "items", "orundum", "originite"]
        );
    }

    #[test]
    fn a_normalized_header_is_a_fixed_point_and_round_trips() {
        let raw = json!({
            "tabs": [{ "id": "roster", "visible": false }],
            "header": { "hidden": ["lmd", "originite", "lmd"], "strip": ["lmd", "x", "orundum"] },
        });
        let once = ProfileLayout::normalize(&raw);
        let wire_once = serde_json::to_value(&once).unwrap();
        assert_eq!(
            wire_once["header"],
            json!({ "hidden": ["originite", "lmd"], "strip": ["lmd", "orundum"] })
        );
        assert_eq!(ProfileLayout::normalize(&wire_once), once);
        // No strip key stays absent on the way out.
        let bare = header(json!({ "hidden": ["level"] })).unwrap();
        assert_eq!(
            serde_json::to_value(bare).unwrap(),
            json!({ "hidden": ["level"] })
        );
    }

    #[test]
    fn the_visitor_header_resolves_the_strip_and_drops_hidden_cards() {
        let visitor = |raw: Value| {
            ProfileLayout::normalize(&json!({ "header": raw }))
                .visible_only()
                .header
                .unwrap()
        };
        let got = visitor(json!({ "hidden": ["originite", "level"] }));
        assert_eq!(
            wire(got.strip.as_deref().unwrap()),
            ["operators", "skins", "items", "orundum"]
        );
        assert_eq!(wire(&got.hidden), ["level", "originite"]);
        let got = visitor(json!({ "hidden": ["lmd"], "strip": ["lmd", "orundum"] }));
        assert_eq!(wire(got.strip.as_deref().unwrap()), ["orundum"]);
        let got = visitor(json!({ "hidden": ["lmd"], "strip": ["lmd"] }));
        assert_eq!(got.strip, Some(vec![]));
        // Nothing hidden and no strip: the default five, now spelled out.
        assert_eq!(
            visitor(json!({})).strip.as_deref(),
            Some(&ProfileHeaderField::DEFAULT_STRIP[..])
        );
        // The owner's layout keeps what they arranged.
        let owner =
            ProfileLayout::normalize(&json!({ "header": { "hidden": ["lmd"], "strip": ["lmd"] } }));
        assert_eq!(
            owner.header.unwrap().strip,
            Some(vec![ProfileHeaderField::Lmd])
        );
    }

    #[test]
    fn header_wire_ids_round_trip_and_the_sql_predicate_matches_the_stored_shape() {
        for field in ProfileHeaderField::ALL {
            assert_eq!(serde_json::to_value(field).unwrap(), json!(field.as_str()));
            assert_eq!(ProfileHeaderField::from_id(field.as_str()), Some(field));
        }
        assert!(
            ProfileHeaderField::DEFAULT_STRIP
                .iter()
                .all(|f| f.is_strip_card())
        );
        assert_eq!(
            ProfileHeaderField::Originite.visible_sql("us.profile_layout"),
            r#"NOT COALESCE(us.profile_layout -> 'header' -> 'hidden' @> '["originite"]'::jsonb, false)"#
        );
        let stored = serde_json::to_value(ProfileHeader {
            hidden: vec![ProfileHeaderField::Originite],
            strip: None,
        })
        .unwrap();
        assert!(
            ProfileHeaderField::Originite
                .visible_sql("c")
                .contains(&stored["hidden"].to_string())
        );
    }

    #[test]
    fn a_header_save_keeps_the_other_keys_and_theirs_keep_it() {
        let stored = json!({
            "tabs": [{ "id": "roster", "visible": false }],
            "showcase": { "blocks": [{ "type": "grid", "slug": "g" }] },
            "background": { "kind": "operator", "id": "char_003_kalts" },
        });
        let set = ProfileLayoutPatch::normalize(
            &json!({ "header": { "hidden": ["originite", "nope"], "strip": ["lmd"] } }),
        );
        assert_eq!(
            (
                set.tabs.as_ref(),
                set.showcase.as_ref(),
                set.background.as_ref()
            ),
            (None, None, None)
        );
        assert_eq!(
            set.merge_json(),
            json!({ "header": { "hidden": ["originite"], "strip": ["lmd"] } })
        );
        let after = set.apply(Some(&stored)).unwrap();
        for key in ["tabs", "showcase", "background"] {
            assert_eq!(after[key], stored[key], "{key}");
        }
        assert_eq!(after["header"]["hidden"], json!(["originite"]));

        // Every other editor's save keeps the header.
        for other in [
            json!({ "tabs": [] }),
            json!({ "showcase": { "blocks": [] } }),
            json!({ "background": null }),
            json!({ "background": { "kind": "skin", "id": "a@b#1" } }),
        ] {
            let kept = ProfileLayoutPatch::normalize(&other)
                .apply(Some(&after))
                .unwrap();
            assert_eq!(kept["header"], after["header"], "{other}");
            assert_eq!(
                ProfileLayoutPatch::normalize(&other).header,
                None,
                "{other}"
            );
        }

        // Null, or a header that is not an object, clears it alone.
        for raw in [json!(null), json!(["originite"])] {
            let clear = ProfileLayoutPatch::normalize(&json!({ "header": raw }));
            assert_eq!(clear.header, Some(None), "{raw}");
            assert_eq!(clear.removed_keys(), ["header"]);
            assert_eq!(clear.apply(Some(&after)), Some(stored.clone()));
        }
        // A header alone, cleared, leaves no layout.
        let alone = set.apply(None).unwrap();
        let clear = ProfileLayoutPatch::normalize(&json!({ "header": null }));
        assert_eq!(clear.apply(Some(&alone)), None);
        // Clearing both at once removes both.
        let both = ProfileLayoutPatch::normalize(&json!({ "header": null, "background": null }));
        assert_eq!(both.removed_keys(), ["background", "header"]);
    }
}
