use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "schema")]
use utoipa::ToSchema;
use uuid::Uuid;

use crate::ids::ItemId;

#[cfg_attr(feature = "schema", derive(ToSchema))]
#[derive(Debug, Clone, Serialize)]
pub struct SupportUnit {
    pub slot: i16,
    pub operator_id: String,
    pub skin_id: Option<String>,
    pub skill_index: i16,
    pub current_equip: Option<String>,
    pub elite: Option<i16>,
    pub level: Option<i16>,
    pub potential: Option<i16>,
    pub skill_level: Option<i16>,
    pub favor_point: Option<i32>,
    pub specialize_level: i16,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<(ItemId, i64)>,
}

#[cfg_attr(feature = "schema", derive(ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterEntry {
    pub user_id: Uuid,
    pub operator_id: String,
    pub elite: i16,
    pub level: i16,
    pub exp: i32,
    pub potential: i16,
    pub skill_level: i16,
    pub favor_point: i32,
    pub skin_id: Option<String>,
    pub default_skill: Option<i16>,
    pub voice_lan: Option<String>,
    pub current_equip: Option<String>,
    pub current_tmpl: Option<String>,
    pub obtained_at: Option<i64>,
    pub masteries: Value,
    pub modules: Value,
}
