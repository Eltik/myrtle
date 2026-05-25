use axum::Json;
use axum::extract::State;

use crate::app::error::ApiError;
use crate::app::services;
use crate::app::state::AppState;
use crate::dps::engine::DpsResult;

pub async fn operators(
    State(state): State<AppState>,
) -> Json<Vec<services::dps::OperatorListEntry>> {
    Json(services::dps::list_operators(&state))
}

pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    let result = services::dps::calculate(&state, body)?;
    Ok(Json(result))
}
