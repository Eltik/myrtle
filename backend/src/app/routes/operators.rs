use axum::{Json, extract::State};

use crate::app::{error::ApiError, services, state::AppState};

pub async fn index(
    State(state): State<AppState>,
) -> Result<Json<Vec<services::operators::OperatorIndexEntry>>, ApiError> {
    let entries = services::operators::get_index(&state).await?;
    Ok(Json(entries))
}
