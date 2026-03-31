use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;
use crate::database::models::operator_notes::{OperatorNote, OperatorNoteAuditEntry};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<OperatorNote>>, ApiError> {
    let notes = services::operator_notes::get_all(&state).await?;
    Ok(Json(notes))
}

pub async fn get(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<OperatorNote>, ApiError> {
    let note = services::operator_notes::get_by_operator(&state, &operator_id).await?;
    Ok(Json(note))
}

pub async fn audit_log(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<Vec<OperatorNoteAuditEntry>>, ApiError> {
    let log = services::operator_notes::get_audit_log(&state, &operator_id).await?;
    Ok(Json(log))
}

#[derive(Deserialize)]
pub struct UpdateNoteRequest {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
    Json(body): Json<UpdateNoteRequest>,
) -> Result<Json<OperatorNote>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }

    let user_id: uuid::Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;

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
