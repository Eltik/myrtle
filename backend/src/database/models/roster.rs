use serde::Serialize;
use sqlx::types::Uuid;

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
