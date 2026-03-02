use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::Serialize;

use crate::app::error::ApiError;
use crate::app::routes::operator_notes::get::OperatorNoteResponse;
use crate::app::routes::tier_lists::middleware::{require_any_admin_role, require_auth};
use crate::app::state::AppState;
use crate::database::models::operator_notes::{
    OperatorNote, OperatorNoteAuditLog, UpdateOperatorNote,
};

#[derive(Serialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub note: OperatorNoteResponse,
}

/// PUT /operator-notes/{operator_id}
/// Update notes for a specific operator (requires TierListEditor+)
pub async fn update_operator_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(operator_id): Path<String>,
    Json(input): Json<UpdateOperatorNote>,
) -> Result<Json<UpdateResponse>, ApiError> {
    // Auth: require TierListEditor or above
    let auth = require_auth(&headers, &state.jwt_secret)?;
    require_any_admin_role(&auth)?;

    // Validate operator exists in game data
    if !state.game_data.operators.contains_key(&operator_id) {
        return Err(ApiError::NotFound("Operator not found in game data".into()));
    }

    // Validate at least one field is provided
    if input.pros.is_none()
        && input.cons.is_none()
        && input.notes.is_none()
        && input.trivia.is_none()
        && input.summary.is_none()
        && input.tags.is_none()
    {
        return Err(ApiError::BadRequest(
            "At least one field must be provided".into(),
        ));
    }

    // Validate summary length
    if let Some(ref summary) = input.summary
        && summary.len() > 500
    {
        return Err(ApiError::BadRequest(
            "Summary must be 500 characters or fewer".into(),
        ));
    }

    // Validate tags is an array
    if let Some(ref tags) = input.tags
        && !tags.is_array()
    {
        return Err(ApiError::BadRequest("Tags must be an array".into()));
    }

    // Fetch current state for audit diff
    let current = OperatorNote::find_by_operator(&state.db, &operator_id)
        .await
        .map_err(|e| {
            eprintln!("Database error fetching current note: {e:?}");
            ApiError::Internal("Failed to fetch current note".into())
        })?
        .ok_or_else(|| ApiError::NotFound("Operator note not found".into()))?;

    // Perform the update
    let updated = OperatorNote::update(&state.db, &operator_id, &input)
        .await
        .map_err(|e| {
            eprintln!("Database error updating operator note: {e:?}");
            ApiError::Internal("Failed to update operator note".into())
        })?;

    // Create audit log entries for each changed field
    let audit_fields: Vec<(&str, &str, Option<&str>)> = vec![
        ("pros", &current.pros, input.pros.as_deref()),
        ("cons", &current.cons, input.cons.as_deref()),
        ("notes", &current.notes, input.notes.as_deref()),
        ("trivia", &current.trivia, input.trivia.as_deref()),
        ("summary", &current.summary, input.summary.as_deref()),
    ];

    for (field, old_val, new_val) in audit_fields {
        if let Some(new) = new_val
            && new != old_val
            && let Err(e) = OperatorNoteAuditLog::create(
                &state.db,
                &operator_id,
                field,
                Some(old_val),
                Some(new),
                auth.user_id,
            )
            .await
        {
            eprintln!("Warning: Failed to create audit log for {field}: {e:?}");
        }
    }

    // Audit tags separately (serialize to string for comparison)
    if let Some(ref new_tags) = input.tags {
        let old_tags_str = serde_json::to_string(&current.tags).unwrap_or_default();
        let new_tags_str = serde_json::to_string(new_tags).unwrap_or_default();
        if old_tags_str != new_tags_str
            && let Err(e) = OperatorNoteAuditLog::create(
                &state.db,
                &operator_id,
                "tags",
                Some(&old_tags_str),
                Some(&new_tags_str),
                auth.user_id,
            )
            .await
        {
            eprintln!("Warning: Failed to create audit log for tags: {e:?}");
        }
    }

    // Enrich response with game data
    let operator_data = state.game_data.operators.get(&operator_id);

    Ok(Json(UpdateResponse {
        success: true,
        note: OperatorNoteResponse {
            id: updated.id.to_string(),
            operator_id: updated.operator_id,
            pros: updated.pros,
            cons: updated.cons,
            notes: updated.notes,
            trivia: updated.trivia,
            summary: updated.summary,
            tags: updated.tags,
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
            created_at: updated.created_at.to_rfc3339(),
            updated_at: updated.updated_at.to_rfc3339(),
        },
    }))
}
