use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::operator_notes::OperatorNoteAuditLog;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct AuditLogResponse {
    pub id: String,
    pub operator_id: String,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Option<String>,
    pub changed_at: String,
}

fn audit_to_response(log: OperatorNoteAuditLog) -> AuditLogResponse {
    AuditLogResponse {
        id: log.id.to_string(),
        operator_id: log.operator_id,
        field_name: log.field_name,
        old_value: log.old_value,
        new_value: log.new_value,
        changed_by: log.changed_by.map(|id| id.to_string()),
        changed_at: log.changed_at.to_rfc3339(),
    }
}

/// GET /operator-notes/{operator_id}/audit
/// Get edit history for a specific operator
pub async fn get_operator_audit(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogResponse>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(200);

    let logs = OperatorNoteAuditLog::find_by_operator(&state.db, &operator_id, limit)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching audit log: {e:?}");
            ApiError::Internal("Failed to fetch audit log".into())
        })?;

    Ok(Json(logs.into_iter().map(audit_to_response).collect()))
}

/// GET /operator-notes/audit/recent
/// Get recent edits across all operators
pub async fn get_recent_audit(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogResponse>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(200);

    let logs = OperatorNoteAuditLog::find_recent(&state.db, limit)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching recent audit log: {e:?}");
            ApiError::Internal("Failed to fetch recent audit log".into())
        })?;

    Ok(Json(logs.into_iter().map(audit_to_response).collect()))
}
