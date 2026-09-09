use serde::Serialize;
use sqlx::types::{
    Uuid,
    chrono::{DateTime, Utc},
};
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    #[ts(type = "number")]
    pub id: i64,
    pub table_name: String,
    pub record_id: String,
    pub action: String,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
}
