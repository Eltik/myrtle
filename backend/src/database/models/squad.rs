//! Saved squads (four slots on the account) and the in-game squads captured at
//! roster sync, as the Squad Builder page reads and writes them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

pub use crate::core::squad::policy::{NAME_MAX_CHARS, SLOT_LIMIT, SQUAD_SIZE};

/// An explicit build on a seat. `None` on a `SquadMember` means "follow my
/// roster": the page resolves the build from the synced roster when it loads,
/// and from a full build when the operator is not owned.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadBuild {
    pub phase: i32,
    pub level: i32,
    /// 0-based (0 = potential 1).
    pub potential: i32,
    /// 1 to the operator's top skill rank (`AllSkillLvlup` count + 1).
    pub skill_level: i32,
    /// On top of the top rank, up to the skill's `LevelUpCostCond` count.
    pub mastery: i32,
    /// 1 to the module's rank count when a module is set, else 0.
    pub module_level: i32,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SquadMember {
    pub char_id: String,
    /// 0-based.
    pub skill_index: i32,
    pub module_id: Option<String>,
    pub build: Option<SquadBuild>,
}

/// `PUT /squads/{slot}`. `members` is at most twelve entries, `None` for an
/// empty seat; shorter lists are padded on read.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSquadInput {
    pub name: String,
    pub members: Vec<Option<SquadMember>>,
    pub support: Option<SquadMember>,
    pub stage_id: Option<String>,
}

#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSquad {
    /// 1-based, 1..=4.
    pub slot: i32,
    pub name: String,
    /// Exactly twelve entries.
    pub members: Vec<Option<SquadMember>>,
    pub support: Option<SquadMember>,
    pub stage_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One seat of an in-game squad as the game stores it: the operator, its
/// chosen skill and the module it had equipped at sync time. The build is the
/// roster's.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSquadSlot {
    pub char_id: String,
    pub skill_index: i32,
    pub module_id: Option<String>,
}

/// An in-game squad from the last roster sync, read only.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSquad {
    /// The game's `squadId` as a number, 0-based.
    pub index: i32,
    pub name: String,
    /// Exactly twelve entries.
    pub slots: Vec<Option<GameSquadSlot>>,
    pub synced_at: DateTime<Utc>,
}

/// `GET /squads`.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadsResponse {
    /// Exactly `slot_limit` entries, `None` for an empty slot.
    pub saved: Vec<Option<SavedSquad>>,
    /// Empty until the account syncs after this feature shipped.
    pub synced: Vec<GameSquad>,
    pub slot_limit: i32,
    pub squad_size: i32,
}

/// A `user_squads` row. `members` and `support` stay raw JSON so a row written
/// by an older shape still loads: [`SavedSquadRow::into_saved`] drops a seat
/// it cannot read, never the squad.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SavedSquadRow {
    pub slot: i16,
    pub name: String,
    pub members: sqlx::types::Json<serde_json::Value>,
    pub support: Option<sqlx::types::Json<serde_json::Value>>,
    pub stage_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SavedSquadRow {
    pub fn into_saved(self) -> SavedSquad {
        SavedSquad {
            slot: i32::from(self.slot),
            name: self.name,
            members: seats(self.members.0),
            support: self.support.and_then(|s| serde_json::from_value(s.0).ok()),
            stage_id: self.stage_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// A `user_game_squads` row.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GameSquadRow {
    pub squad_index: i16,
    pub name: String,
    pub slots: sqlx::types::Json<serde_json::Value>,
    pub synced_at: DateTime<Utc>,
}

impl GameSquadRow {
    pub fn into_game_squad(self) -> GameSquad {
        GameSquad {
            index: i32::from(self.squad_index),
            name: self.name,
            slots: seats(self.slots.0),
            synced_at: self.synced_at,
        }
    }
}

/// A stored seat array as exactly [`SQUAD_SIZE`] entries: an entry that does
/// not parse becomes an empty seat, a short array is padded and a long one cut.
fn seats<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Vec<Option<T>> {
    let mut out: Vec<Option<T>> = match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .take(SQUAD_SIZE)
            .map(|item| serde_json::from_value::<Option<T>>(item).ok().flatten())
            .collect(),
        _ => Vec::new(),
    };
    out.resize_with(SQUAD_SIZE, || None);
    out
}
