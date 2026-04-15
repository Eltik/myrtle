use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::operator_notes::{OperatorNote, OperatorNoteAuditEntry};
use crate::database::queries::operator_notes as queries;

pub async fn get_all(state: &AppState) -> Result<Vec<OperatorNote>, ApiError> {
    queries::get_all(&state.db).await.map_err(|e| e.into())
}

pub async fn get_by_operator(
    state: &AppState,
    operator_id: &str,
) -> Result<OperatorNote, ApiError> {
    queries::get_by_operator(&state.db, operator_id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn get_audit_log(
    state: &AppState,
    operator_id: &str,
) -> Result<Vec<OperatorNoteAuditEntry>, ApiError> {
    queries::get_audit_log(&state.db, operator_id)
        .await
        .map_err(|e| e.into())
}

pub struct UpdateFields {
    pub pros: Option<String>,
    pub cons: Option<String>,
    pub notes: Option<String>,
    pub trivia: Option<String>,
    pub summary: Option<String>,
    pub tags: Option<serde_json::Value>,
}

pub async fn update(
    state: &AppState,
    operator_id: &str,
    changed_by: Uuid,
    fields: UpdateFields,
) -> Result<OperatorNote, ApiError> {
    let existing = queries::get_by_operator(&state.db, operator_id).await?;

    let note = queries::upsert(
        &state.db,
        operator_id,
        fields.pros.as_deref(),
        fields.cons.as_deref(),
        fields.notes.as_deref(),
        fields.trivia.as_deref(),
        fields.summary.as_deref(),
        fields.tags.as_ref(),
    )
    .await?;

    let diffs: Vec<(&str, Option<&str>, Option<&str>)> = vec![
        (
            "pros",
            existing.as_ref().and_then(|e| e.pros.as_deref()),
            fields.pros.as_deref(),
        ),
        (
            "cons",
            existing.as_ref().and_then(|e| e.cons.as_deref()),
            fields.cons.as_deref(),
        ),
        (
            "notes",
            existing.as_ref().and_then(|e| e.notes.as_deref()),
            fields.notes.as_deref(),
        ),
        (
            "trivia",
            existing.as_ref().and_then(|e| e.trivia.as_deref()),
            fields.trivia.as_deref(),
        ),
        (
            "summary",
            existing.as_ref().and_then(|e| e.summary.as_deref()),
            fields.summary.as_deref(),
        ),
    ];

    for (field, old, new) in diffs {
        if let Some(new_val) = new
            && old != Some(new_val)
        {
            queries::insert_audit(&state.db, note.id, field, old, Some(new_val), changed_by)
                .await?;
        }
    }

    Ok(note)
}
