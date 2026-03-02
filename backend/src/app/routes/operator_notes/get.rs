use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::operator_notes::OperatorNote;

#[derive(Serialize)]
pub struct OperatorNoteResponse {
    pub id: String,
    pub operator_id: String,
    pub pros: String,
    pub cons: String,
    pub notes: String,
    pub trivia: String,
    pub summary: String,
    pub tags: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_rarity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_profession: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn note_to_response(note: OperatorNote, state: &AppState) -> OperatorNoteResponse {
    let operator_data = state.game_data.operators.get(&note.operator_id);

    OperatorNoteResponse {
        id: note.id.to_string(),
        operator_id: note.operator_id,
        pros: note.pros,
        cons: note.cons,
        notes: note.notes,
        trivia: note.trivia,
        summary: note.summary,
        tags: note.tags,
        operator_name: operator_data.map(|o| o.name.clone()),
        operator_rarity: operator_data.and_then(|o| {
            serde_json::to_value(&o.rarity)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }),
        operator_profession: operator_data.and_then(|o| {
            serde_json::to_value(&o.profession)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }),
        created_at: note.created_at.to_rfc3339(),
        updated_at: note.updated_at.to_rfc3339(),
    }
}

/// GET /operator-notes
/// List all operator notes that have content
pub async fn list_operator_notes(
    State(state): State<AppState>,
) -> Result<Json<Vec<OperatorNoteResponse>>, ApiError> {
    let notes = OperatorNote::find_all_with_content(&state.db)
        .await
        .map_err(|e| {
            eprintln!("Database error listing operator notes: {e:?}");
            ApiError::Internal("Failed to fetch operator notes".into())
        })?;

    let response: Vec<OperatorNoteResponse> = notes
        .into_iter()
        .map(|n| note_to_response(n, &state))
        .collect();

    Ok(Json(response))
}

/// GET /operator-notes/{operator_id}
/// Get notes for a specific operator
pub async fn get_operator_note(
    State(state): State<AppState>,
    Path(operator_id): Path<String>,
) -> Result<Json<OperatorNoteResponse>, ApiError> {
    let note = OperatorNote::find_by_operator(&state.db, &operator_id)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching operator note: {e:?}");
            ApiError::Internal("Failed to fetch operator note".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Operator note not found".into()))?;

    Ok(Json(note_to_response(note, &state)))
}
