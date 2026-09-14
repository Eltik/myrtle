use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::cpu;
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

/// The simulation is pure CPU on an unauthenticated route, so the body is
/// bounds-checked first and the work then runs on the blocking pool under
/// admission control: over the limit is a 503 rather than an occupied worker.
pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    body.validate()?;
    let result = cpu::run("dps_calculate", move || {
        services::dps::calculate(&state, body)
    })
    .await??;
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
    body.validate()?;
    let result = cpu::run("hps_calculate", move || {
        services::dps::calculate_hps(&state, body)
    })
    .await??;
    Ok(Json(result))
}
