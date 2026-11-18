use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TierListType {
    Official,
    Community,
}

impl TierListType {
    pub const ALL: [Self; 2] = [Self::Official, Self::Community];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "official",
            Self::Community => "community",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    Operator,
    Class,
    Subclass,
    Enemy,
    Event,
    Faction,
    StrongholdBond,
    Skin,
    Module,
    /// `{char_id}:{skill_id}`
    Skill,
    IntegratedStrategies,
    StorySprite,
    MainStory,
    Background,
    StoryCg,
    StoryScene,
    ArchivePic,
}

impl EntityKind {
    pub const ALL: [Self; 17] = [
        Self::Operator,
        Self::Class,
        Self::Subclass,
        Self::Enemy,
        Self::Event,
        Self::Faction,
        Self::StrongholdBond,
        Self::Skin,
        Self::Module,
        Self::Skill,
        Self::IntegratedStrategies,
        Self::StorySprite,
        Self::MainStory,
        Self::Background,
        Self::StoryCg,
        Self::StoryScene,
        Self::ArchivePic,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Class => "class",
            Self::Subclass => "subclass",
            Self::Enemy => "enemy",
            Self::Event => "event",
            Self::Faction => "faction",
            Self::StrongholdBond => "stronghold_bond",
            Self::Skin => "skin",
            Self::Module => "module",
            Self::Skill => "skill",
            Self::IntegratedStrategies => "integrated_strategies",
            Self::StorySprite => "story_sprite",
            Self::MainStory => "main_story",
            Self::Background => "background",
            Self::StoryCg => "story_cg",
            Self::StoryScene => "story_scene",
            Self::ArchivePic => "archive_pic",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentTargetKind {
    TierList,
    Grid,
}

impl ContentTargetKind {
    pub const ALL: [Self; 2] = [Self::TierList, Self::Grid];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TierList => "tier_list",
            Self::Grid => "grid",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentAuditAction {
    Create,
    Fork,
    Edit,
    Publish,
    Visibility,
    Flair,
    Grant,
    Revoke,
    Delete,
}

impl ContentAuditAction {
    pub const ALL: [Self; 9] = [
        Self::Create,
        Self::Fork,
        Self::Edit,
        Self::Publish,
        Self::Visibility,
        Self::Flair,
        Self::Grant,
        Self::Revoke,
        Self::Delete,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Fork => "fork",
            Self::Edit => "edit",
            Self::Publish => "publish",
            Self::Visibility => "visibility",
            Self::Flair => "flair",
            Self::Grant => "grant",
            Self::Revoke => "revoke",
            Self::Delete => "delete",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.as_str() == s)
    }
}
