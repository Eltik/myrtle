use serde::{Deserialize, Serialize};
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

/// What a tier list placement ranks. The id space of each kind is its own, so a
/// placement is keyed by (kind, id), and the variants are the allow-list: an
/// unknown kind fails to parse at the route and never reaches the database.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    #[default]
    Operator,
    /// A profession (`WARRIOR`, `PIONEER`), the eight playable ones.
    Class,
    /// A sub-profession id (`pioneer`, `charger`), the playable ones.
    Subclass,
    /// An `enemy_handbook_table` id.
    Enemy,
    /// An `activity_table.BasicInfo` id.
    Event,
    /// A `handbook_team_table` power id: a nation, group or team.
    Faction,
    /// A Stronghold Protocol bond, `activity_table.AutoChessData.BondInfoDict`.
    StrongholdBond,
    /// A purchasable or reward outfit, a `skin_table.CharSkins` id with an `@`.
    Skin,
    /// An operator module, a `uniequip_table.EquipDict` id that is not the
    /// ORIGINAL placeholder.
    Module,
    /// One operator's skill slot, `{char_id}:{skill_id}`: a skill id alone is
    /// not one entry, because generic skills (`skcom_*`) are shared.
    Skill,
    /// An Integrated Strategies theme (`rogue_N`) or one of its items: relics,
    /// tools, squads, Plays, Foldartals, Thoughts, Wraths and Tongbao.
    IntegratedStrategies,
    /// A story character sprite set, keyed by its folder under
    /// `textures/avg/characters` with the trailing variant number cut.
    StorySprite,
}

impl EntityKind {
    pub const ALL: &'static [Self] = &[
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
    ];

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
        }
    }
}

impl std::str::FromStr for EntityKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| format!("unknown entity kind `{s}`"))
    }
}

impl TryFrom<String> for EntityKind {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

/// One placement's key within a list: the kind, and the id in that kind's space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRef<'a> {
    pub kind: EntityKind,
    pub id: &'a str,
}

impl TierPlacement {
    pub fn entity(&self) -> EntityRef<'_> {
        EntityRef {
            kind: self.entity_kind,
            id: &self.entity_id,
        }
    }
}

/// A list's offered kinds, as stored in `tier_lists.entity_kinds`.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityKinds(pub Vec<EntityKind>);

/// Lenient on purpose: the column checks only that the set is non-empty, not
/// its values, so a kind this build does not know (a hand edit, a rolled-back
/// release) is skipped with a warning rather than failing every query that
/// reads the list. Nothing left reads as operator-only.
impl From<Vec<String>> for EntityKinds {
    fn from(raw: Vec<String>) -> Self {
        let kinds: Vec<EntityKind> = raw
            .into_iter()
            .filter_map(|s| match s.parse() {
                Ok(kind) => Some(kind),
                Err(err) => {
                    tracing::warn!("tier_lists.entity_kinds: skipping {err}");
                    None
                }
            })
            .collect();
        if kinds.is_empty() {
            operator_only()
        } else {
            Self(kinds)
        }
    }
}

fn operator_only() -> EntityKinds {
    EntityKinds(vec![EntityKind::Operator])
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierList {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub list_type: String,
    pub created_by: Option<Uuid>,
    pub is_active: bool,
    pub is_listed: bool,
    pub flair_id: Option<i16>,
    /// The kinds the list's editor offers. Lists cached or snapshotted before
    /// the column existed read as operator-only, which is what they were.
    #[serde(default = "operator_only")]
    #[sqlx(try_from = "Vec<String>")]
    pub entity_kinds: EntityKinds,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListFlair {
    pub id: i16,
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: i16,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListStats {
    pub tier_list_id: Uuid,
    #[ts(type = "number")]
    pub view_count: i64,
    #[ts(type = "number")]
    pub unique_view_count: i64,
    pub favorite_count: i32,
    pub share_count: i32,
    pub is_trending: bool,
    pub trending_score: f64,
    pub views_last_24h: i32,
    pub views_last_7d: i32,
    pub last_viewed_at: Option<DateTime<Utc>>,
    pub stats_updated_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListFavorite {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub favorited_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tier {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierPlacement {
    pub tier_id: Uuid,
    /// Absent from snapshots and cache entries written before kinds existed,
    /// all of which were operators.
    #[serde(default)]
    #[sqlx(try_from = "String")]
    pub entity_kind: EntityKind,
    /// Was `operator_id`; old snapshots still spell it that way.
    #[serde(alias = "operator_id")]
    pub entity_id: String,
    pub sub_order: i16,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// A `tier_placements` row as stored, its kind not yet checked. The list reads
/// go through this so one row of a kind this build does not know drops that
/// placement instead of failing the whole list.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TierPlacementRow {
    pub tier_id: Uuid,
    pub entity_kind: String,
    pub entity_id: String,
    pub sub_order: i16,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl TierPlacementRow {
    /// The placement, or `None` (with a warning) when its kind is unknown.
    pub fn into_known(self) -> Option<TierPlacement> {
        match self.entity_kind.parse() {
            Ok(entity_kind) => Some(TierPlacement {
                tier_id: self.tier_id,
                entity_kind,
                entity_id: self.entity_id,
                sub_order: self.sub_order,
                description: self.description,
                updated_at: self.updated_at,
            }),
            Err(err) => {
                tracing::warn!(
                    tier_id = %self.tier_id,
                    entity_id = %self.entity_id,
                    "tier_placements: dropping a placement, {err}"
                );
                None
            }
        }
    }
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListVersion {
    pub id: Uuid,
    pub tier_list_id: Uuid,
    pub version: i32,
    pub snapshot: serde_json::Value,
    pub changelog: Option<String>,
    pub published_by: Option<Uuid>,
    pub published_at: DateTime<Utc>,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TierListPermission {
    pub tier_list_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(kind: &str) -> TierPlacementRow {
        TierPlacementRow {
            tier_id: Uuid::nil(),
            entity_kind: kind.into(),
            entity_id: "char_002_amiya".into(),
            sub_order: 0,
            description: None,
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }

    #[test]
    fn unknown_offered_kinds_are_skipped() {
        let kinds = EntityKinds::from(vec!["enemy".into(), "boss".into(), "operator".into()]);
        assert_eq!(
            kinds,
            EntityKinds(vec![EntityKind::Enemy, EntityKind::Operator])
        );
    }

    #[test]
    fn nothing_known_offered_reads_as_operator_only() {
        assert_eq!(EntityKinds::from(vec!["boss".into()]), operator_only());
        assert_eq!(EntityKinds::from(Vec::new()), operator_only());
    }

    #[test]
    fn a_placement_of_an_unknown_kind_is_dropped() {
        assert!(row("boss").into_known().is_none());
        let known = row("stronghold_bond")
            .into_known()
            .expect("a known kind decodes");
        assert_eq!(known.entity_kind, EntityKind::StrongholdBond);
        assert_eq!(known.entity_id, "char_002_amiya");
    }
}
