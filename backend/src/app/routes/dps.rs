use axum::Json;
use axum::extract::State;

use crate::app::error::ApiError;
use crate::app::services;
use crate::app::state::AppState;
use crate::dps::engine::DpsResult;

pub async fn operators() -> Json<Vec<services::dps::OperatorListEntry>> {
    Json(services::dps::list_operators())
}

pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    let result = services::dps::calculate(&state, body)?;
    Ok(Json(result))
}
