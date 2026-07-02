use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::error::ApiError;
use crate::app::routes::static_data::json_response;
use crate::app::services;
use crate::app::state::AppState;
use crate::dps::engine::{DpsResult, HpsResult};

pub async fn operators(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let cached = services::dps::list_operators_json(&state).await?;
    Ok(json_response(cached, &headers))
}

pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    let result = services::dps::calculate(&state, body)?;
    Ok(Json(result))
}

pub async fn healers(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let cached = services::dps::list_healers_json(&state).await?;
    Ok(json_response(cached, &headers))
}

pub async fn calculate_hps(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<HpsResult>, ApiError> {
    let result = services::dps::calculate_hps(&state, body)?;
    Ok(Json(result))
}
