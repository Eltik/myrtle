use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::services::operator_notes::GlobalAuditLogResponse;
use crate::app::services::operator_notes::get_all;
use crate::app::services::operator_notes::get_audit_log;
use crate::app::services::operator_notes::get_by_operator;
use crate::app::services::operator_notes::get_global_audit_log;
use crate::app::state::AppState;
use crate::database::models::operator_notes::{OperatorNote, OperatorNoteAuditEntry};

/// Every editorial operator note.
#[utoipa::path(
    get,
    path = "/operator-notes",
    operation_id = "operator_notes_list",
    tag = "notes",
    responses(
        (status = 200, description = "All notes.", body = Vec<OperatorNote>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<OperatorNote>>, ApiError> {
    let notes = get_all(&state).await?;
    Ok(Json(notes))
}

/// One operator's editorial note.
#[utoipa::path(
    get,
    path = "/operator-notes/{operator_id}",
    operation_id = "operator_note_get",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "The note.", body = OperatorNote),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<OperatorNote>, ApiError> {
    let note = get_by_operator(&state, &operator_id).await?;
    Ok(Json(note))
}

/// Edit history for one operator's note.
#[utoipa::path(
    get,
    path = "/operator-notes/{operator_id}/audit",
    operation_id = "operator_note_audit_log",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    responses(
        (status = 200, description = "Audit entries, newest first.", body = Vec<OperatorNoteAuditEntry>),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn audit_log(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<Vec<OperatorNoteAuditEntry>>, ApiError> {
    let log = get_audit_log(&state, &operator_id).await?;
    Ok(Json(log))
}

#[derive(Deserialize)]
pub struct GlobalAuditQuery {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Edit history across every operator note.
#[utoipa::path(
    get,
    path = "/admin/operator-notes/audit",
    tag = "admin",
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "Audit entries, newest first.", body = GlobalAuditLogResponse),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn global_audit_log(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(params): Query<GlobalAuditQuery>,
) -> Result<Json<GlobalAuditLogResponse>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let limit = params.limit.unwrap_or(100);
    let response = get_global_audit_log(&state, limit, params.before).await?;
    Ok(Json(response))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateNoteRequest {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
}

/// Write an operator's editorial note.
///
/// Every write is recorded in the note's audit log.
#[utoipa::path(
    put,
    path = "/operator-notes/{operator_id}",
    operation_id = "operator_note_update",
    tag = "notes",
    params(
        ("operator_id" = String, Path, description = "Operator id.")
    ),
    request_body = UpdateNoteRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The stored note.", body = OperatorNote),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<OperatorNote>, ApiError> {
    if !auth.role.is_any_admin_role() {
        return Err(ApiError::Forbidden);
    }

    let user_id: uuid::Uuid = auth.user_uuid()?;

    let note = services::operator_notes::update(
        &state,
        &operator_id,
        user_id,
        services::operator_notes::UpdateFields {
            pros: body.pros,
            cons: body.cons,
            notes: body.notes,
            trivia: body.trivia,
            summary: body.summary,
            tags: body.tags,
        },
    )
    .await?;
    Ok(Json(note))
}
