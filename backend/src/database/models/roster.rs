use serde::Serialize;
use sqlx::types::Uuid;

/// One entry in a Doctor's published support roster (slot 0..2).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SupportUnit {
    pub slot: i16,
    pub operator_id: String,
    pub skin_id: Option<String>,
    pub skill_index: i16,
    pub current_equip: Option<String>,
    /// Joined from user_operators — null if the support refers to an op the
    /// player no longer owns (shouldn't happen, but the FK is soft).
    pub elite: Option<i16>,
    pub level: Option<i16>,
    pub potential: Option<i16>,
    pub skill_level: Option<i16>,
    pub favor_point: Option<i32>,
    /// Mastery (0-3) for the chosen skill_index.
    pub specialize_level: i16,
}

/// v_user_roster view
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
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
    pub masteries: serde_json::Value, // jsonb_agg result
    pub modules: serde_json::Value,   // jsonb_agg result
}
