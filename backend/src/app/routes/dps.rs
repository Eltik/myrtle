use axum::Json;
use axum::extract::State;

use crate::app::error::ApiError;
use crate::app::services;
use crate::app::state::AppState;
use crate::dps::engine::{DpsResult, HpsResult};

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

pub async fn healers(State(state): State<AppState>) -> Json<Vec<services::dps::OperatorListEntry>> {
    Json(services::dps::list_healers(&state))
}

pub async fn calculate_hps(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<HpsResult>, ApiError> {
    let result = services::dps::calculate_hps(&state, body)?;
    Ok(Json(result))
}
