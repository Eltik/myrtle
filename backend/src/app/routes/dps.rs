use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::cache::keys::CacheKey;
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

/// Hash a request body into a cache key component.
///
/// `DefaultHasher` is seeded deterministically in Rust, so the value is stable
/// across processes and safe to share through Redis. `RandomState` would not be.
fn body_hash(body: &services::dps::CalculateRequest) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_string(body)
        .unwrap_or_default()
        .hash(&mut hasher);
    hasher.finish()
}

/// The simulation is pure CPU on an unauthenticated route, so the body is
/// bounds-checked first and the work then runs on the blocking pool under
/// admission control.
///
/// Memoised, because this is the ideal candidate for it: the result is a pure
/// function of the body and the loaded game data, with no clock, no RNG and no
/// user scoping, so two people posting the same configuration must get the same
/// three floats. The UI compares configurations by firing a burst of these at
/// once, which is what was filling the admission queue; a burst of repeats now
/// costs one simulation.
///
/// Validation stays AHEAD of the cache read so an out-of-range body is rejected
/// rather than keyed, and the cache read sits ahead of `cpu::run` so a hit never
/// consumes a permit.
pub async fn calculate(
    State(state): State<AppState>,
    Json(body): Json<services::dps::CalculateRequest>,
) -> Result<Json<DpsResult>, ApiError> {
    body.validate()?;
    let key = CacheKey::DpsCalculate {
        kind: "dps",
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<DpsResult>(&key).await {
        return Ok(Json(hit));
    }
    // `cpu::run` needs an owned closure, so the compute side takes its own handle.
    // AppState is an Arc behind the scenes, so this is a refcount bump, and it
    // leaves `state` available for the cache write once the work comes back.
    let compute_state = state.clone();
    let result = cpu::run("dps_calculate", move || {
        services::dps::calculate(&compute_state, body)
    })
    .await??;
    state.cache.set(&key, &result).await;
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
    let key = CacheKey::DpsCalculate {
        kind: "hps",
        body_hash: body_hash(&body),
    };
    if let Some(hit) = state.cache.get::<HpsResult>(&key).await {
        return Ok(Json(hit));
    }
    // `cpu::run` needs an owned closure, so the compute side takes its own handle.
    // AppState is an Arc behind the scenes, so this is a refcount bump, and it
    // leaves `state` available for the cache write once the work comes back.
    let compute_state = state.clone();
    let result = cpu::run("hps_calculate", move || {
        services::dps::calculate_hps(&compute_state, body)
    })
    .await??;
    state.cache.set(&key, &result).await;
    Ok(Json(result))
}
