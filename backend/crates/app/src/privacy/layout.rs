//! `user_settings.profile_layout`: tab order and visibility, showcase, header art and numbers.

use myrtle_types::EntityKind;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

/// Declaration order is the canonical tab order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileTabId {
    Showcase,
    Stats,
    Score,
    Roster,
    Plans,
    Inventory,
    Enemies,
    Optimizer,
    /// Roster data: its endpoints also need the Roster tab
    Lookback,
}

impl ProfileTabId {
    pub const ALL: [Self; 9] = [
        Self::Showcase,
        Self::Stats,
        Self::Score,
        Self::Roster,
        Self::Plans,
        Self::Inventory,
        Self::Enemies,
        Self::Optimizer,
        Self::Lookback,
    ];

    #[must_use]
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
            Self::Lookback => "lookback",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|tab| tab.as_str() == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutTab {
    pub id: ProfileTabId,
    pub visible: bool,
}

/// Deserializing normalizes, so every value holds each known tab exactly once
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Value")]
pub struct ProfileLayout {
    pub tabs: Vec<LayoutTab>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub showcase: Option<ProfileShowcase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<ProfileBackground>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<ProfileHeader>,
}

pub const SHOWCASE_MAX_BLOCKS: usize = 12;
pub const SHOWCASE_MAX_IDS: usize = 24;
pub const SHOWCASE_TITLE_MAX: usize = 40;
const SHOWCASE_KEY_MAX: usize = 120; // the `grids.slug` width

fn str_field<'a>(raw: &'a Value, field: &str) -> Option<&'a str> {
    raw.get(field).and_then(Value::as_str)
}

fn array_field<'a>(raw: &'a Value, field: &str) -> &'a [Value] {
    raw.get(field)
        .and_then(Value::as_array)
        .map_or(&[][..], Vec::as_slice)
}

fn bounded_key(s: &str) -> Option<&str> {
    let s = s.trim();
    (!s.is_empty() && s.chars().count() <= SHOWCASE_KEY_MAX).then_some(s)
}

fn clamped_number(raw: &Value, field: &str, min: u16, max: u16) -> Option<u16> {
    raw.get(field)
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite())
        .map(|v| v.round().clamp(f64::from(min), f64::from(max)) as u16)
}

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileShowcase {
    pub blocks: Vec<ShowcaseBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShowcaseBlock {
    Favourites {
        entity_kind: EntityKind,
        ids: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    Grid {
        slug: String,
    },
    TierList {
        slug: String,
    },
    /// Shown to a visitor only while the Plans tab is visible
    Plan {
        id: Uuid,
        /// Lets the block follow the operator to a re-created plan
        #[serde(default, skip_serializing_if = "Option::is_none")]
        operator_id: Option<String>,
    },
}

impl ShowcaseBlock {
    fn normalize(raw: &Value) -> Option<Self> {
        let key = |field: &str| {
            str_field(raw, field)
                .and_then(bounded_key)
                .map(str::to_owned)
        };
        match str_field(raw, "type")? {
            "favourites" => {
                let entity_kind = EntityKind::parse(str_field(raw, "entity_kind")?)?;
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
                Some(Self::Favourites {
                    entity_kind,
                    ids,
                    title: str_field(raw, "title").and_then(normalize_title),
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

    /// Favourites never repeat each other: two rows of one kind may carry different titles
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
    #[must_use]
    pub fn normalize(raw: &Value) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileBackgroundKind {
    Skin,
    Operator,
    ArchivePic,
    StoryCg,
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

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skin => "skin",
            Self::Operator => "operator",
            Self::ArchivePic => "archive_pic",
            Self::StoryCg => "story_cg",
            Self::StoryScene => "story_scene",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_str() == id)
    }

    /// The tier list kind whose lookup decides whether an id exists
    #[must_use]
    pub const fn entity_kind(self) -> Option<EntityKind> {
        match self {
            Self::Skin => Some(EntityKind::Skin),
            Self::Operator => Some(EntityKind::Operator),
            Self::ArchivePic | Self::StoryCg | Self::StoryScene => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileBackground {
    pub kind: ProfileBackgroundKind,
    pub id: String,
    /// Crop centre, percent of the art's width
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_x: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_y: Option<u8>,
    /// Zoom past `cover`, percent
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<u16>,
    /// Operator kind only: 1 or 2
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elite: Option<u8>,
}

pub const BACKGROUND_SCALE_MIN: u16 = 100;
pub const BACKGROUND_SCALE_MAX: u16 = 300;
const FOCUS_MAX: u16 = 100;

impl ProfileBackground {
    #[must_use]
    pub fn normalize(raw: &Value) -> Option<Self> {
        let kind = str_field(raw, "kind").and_then(ProfileBackgroundKind::from_id)?;
        let id = str_field(raw, "id").and_then(bounded_key)?.to_owned();
        let focus = |field: &str| {
            clamped_number(raw, field, 0, FOCUS_MAX).and_then(|pct| u8::try_from(pct).ok())
        };
        let elite = raw
            .get("elite")
            .and_then(Value::as_u64)
            .filter(|e| matches!(e, 1 | 2) && kind == ProfileBackgroundKind::Operator)
            .and_then(|e| u8::try_from(e).ok());
        Some(Self {
            kind,
            id,
            focus_x: focus("focus_x"),
            focus_y: focus("focus_y"),
            scale: clamped_number(raw, "scale", BACKGROUND_SCALE_MIN, BACKGROUND_SCALE_MAX),
            elite,
        })
    }
}

/// What an owner may hide from visitors; declaration order is the canonical order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileHeaderField {
    Level,
    Registered,
    SignIn,
    Operators,
    Skins,
    Items,
    Orundum,
    Originite,
    Lmd,
    HeadhuntingPermits,
    TenPullPermits,
}

impl ProfileHeaderField {
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

    /// The strip a header with no `strip` shows
    pub const DEFAULT_STRIP: [Self; 5] = [
        Self::Operators,
        Self::Skins,
        Self::Items,
        Self::Orundum,
        Self::Originite,
    ];

    #[must_use]
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

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.as_str() == id)
    }

    #[must_use]
    pub const fn is_strip_card(self) -> bool {
        !matches!(self, Self::Level | Self::Registered | Self::SignIn)
    }
}

pub const HEADER_STRIP_MAX: usize = 5;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileHeader {
    /// Canonical order, each once
    pub hidden: Vec<ProfileHeaderField>,
    /// `None` is [`ProfileHeaderField::DEFAULT_STRIP`], empty is no strip
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strip: Option<Vec<ProfileHeaderField>>,
}

impl ProfileHeader {
    #[must_use]
    pub fn normalize(raw: &Value) -> Option<Self> {
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

    #[must_use]
    pub fn is_hidden(&self, field: ProfileHeaderField) -> bool {
        self.hidden.contains(&field)
    }

    #[must_use]
    pub fn strip_cards(&self) -> Vec<ProfileHeaderField> {
        self.strip
            .clone()
            .unwrap_or_else(|| ProfileHeaderField::DEFAULT_STRIP.to_vec())
    }

    /// The strip resolved and without hidden cards; `hidden` is kept
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

/// A save: only the keys sent, so no editor wipes another's work
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "Value")]
pub struct ProfileLayoutPatch {
    pub tabs: Option<Vec<LayoutTab>>,
    pub showcase: Option<ProfileShowcase>,
    pub background: Option<Option<ProfileBackground>>,
    pub header: Option<Option<ProfileHeader>>,
}

impl From<Value> for ProfileLayoutPatch {
    fn from(raw: Value) -> Self {
        Self::normalize(&raw)
    }
}

impl ProfileLayoutPatch {
    #[must_use]
    pub fn normalize(raw: &Value) -> Self {
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

    /// The object merged over the stored layout
    #[must_use]
    pub fn merge_json(&self) -> Value {
        let mut out = Map::new();
        if let Some(tabs) = &self.tabs {
            out.insert("tabs".into(), json!(tabs));
        }
        if let Some(showcase) = &self.showcase {
            out.insert("showcase".into(), json!(showcase));
        }
        if let Some(Some(background)) = &self.background {
            out.insert("background".into(), json!(background));
        }
        if let Some(Some(header)) = &self.header {
            out.insert("header".into(), json!(header));
        }
        Value::Object(out)
    }

    #[must_use]
    pub fn removed_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        if matches!(self.background, Some(None)) {
            keys.push("background".to_owned());
        }
        if matches!(self.header, Some(None)) {
            keys.push("header".to_owned());
        }
        keys
    }

    /// What `stored` becomes under this save, the twin of the SQL in `users::update_settings`
    #[must_use]
    pub fn apply(&self, stored: Option<&Value>) -> Option<Value> {
        let mut out = stored
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if let Value::Object(set) = self.merge_json() {
            out.extend(set);
        }
        for key in self.removed_keys() {
            out.remove(&key);
        }
        (!out.is_empty()).then_some(Value::Object(out))
    }
}

impl Default for ProfileLayout {
    fn default() -> Self {
        Self::normalize(&Value::Null)
    }
}

impl From<Value> for ProfileLayout {
    fn from(raw: Value) -> Self {
        Self::normalize(&raw)
    }
}

impl ProfileLayout {
    /// Total: unknown and repeated tabs dropped, missing ones added visible
    #[must_use]
    pub fn normalize(raw: &Value) -> Self {
        let mut tabs: Vec<LayoutTab> = Vec::with_capacity(ProfileTabId::ALL.len());
        for entry in array_field(raw, "tabs") {
            let Some(id) = str_field(entry, "id").and_then(ProfileTabId::from_id) else {
                continue;
            };
            if tabs.iter().any(|t| t.id == id) {
                continue;
            }
            let visible = entry
                .get("visible")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            tabs.push(LayoutTab { id, visible });
        }
        for id in ProfileTabId::ALL {
            if !tabs.iter().any(|t| t.id == id) {
                let at = if id == ProfileTabId::Showcase {
                    0
                } else {
                    tabs.len()
                };
                tabs.insert(at, LayoutTab { id, visible: true }); // layouts saved before Showcase existed get it first
            }
        }
        Self {
            tabs,
            showcase: raw
                .get("showcase")
                .filter(|s| s.is_object())
                .map(ProfileShowcase::normalize),
            background: raw.get("background").and_then(ProfileBackground::normalize),
            header: raw.get("header").and_then(ProfileHeader::normalize),
        }
    }

    /// A tab absent from the list counts as hidden
    #[must_use]
    pub fn is_visible(&self, tab: ProfileTabId) -> bool {
        self.tabs.iter().any(|t| t.id == tab && t.visible)
    }

    /// What a visitor receives: private tabs absent, the showcase gated on its tab and plans on theirs
    #[must_use]
    pub fn visible_only(&self) -> Self {
        let plans = self.is_visible(ProfileTabId::Plans);
        let showcase = self
            .showcase
            .as_ref()
            .filter(|_| self.is_visible(ProfileTabId::Showcase))
            .map(|showcase| ProfileShowcase {
                blocks: showcase
                    .blocks
                    .iter()
                    .filter(|b| plans || !matches!(b, ShowcaseBlock::Plan { .. }))
                    .cloned()
                    .collect(),
            });
        Self {
            tabs: self.tabs.iter().copied().filter(|t| t.visible).collect(),
            showcase,
            background: self.background.clone(),
            header: self.header.as_ref().map(ProfileHeader::visible_only),
        }
    }

    #[must_use]
    pub fn header_shows(&self, field: ProfileHeaderField) -> bool {
        !self.header.as_ref().is_some_and(|h| h.is_hidden(field))
    }

    #[must_use]
    pub fn showcase_blocks(&self) -> &[ShowcaseBlock] {
        self.showcase.as_ref().map_or(&[], |s| s.blocks.as_slice())
    }
}
