use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Deserialize;

use crate::app::routes::static_data::handler::cached_handler;
use crate::app::state::AppState;
use crate::core::local::gamedata::trust::{TrustInfo, calculate_trust};

#[derive(Deserialize)]
pub struct TrustQuery {
    pub trust: Option<i32>,
}

/// GET /static/trust
/// Returns the full favor table
pub async fn get_favor_table(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = "static:trust:all".to_string();

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            Some(serde_json::json!({
                "favor": &state.game_data.favor
            }))
        },
    )
    .await
}

/// GET /static/trust/calculate?trust=1000
/// or GET /static/trust/calculate/{trust}
/// Returns the trust level for a given favor point value
pub async fn calculate_trust_level(
    State(state): State<AppState>,
    Query(params): Query<TrustQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let trust = params.trust.ok_or(StatusCode::BAD_REQUEST)?;

    let cache_key = format!("static:trust:calculate:{trust}");

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            let level = calculate_trust(&state.game_data.favor, trust);

            // Get detailed info if level found
            let info = level.and_then(|idx| {
                state
                    .game_data
                    .favor
                    .favor_frames
                    .get(idx)
                    .map(|frame| TrustInfo {
                        level: idx,
                        favor_point: frame.data.favor_point,
                        percent: frame.data.percent,
                        battle_phase: frame.data.battle_phase,
                    })
            });

            Some(serde_json::json!({
                "trust": trust,
                "level": level,
                "info": info
            }))
        },
    )
    .await
}

/// GET /static/trust/calculate/{trust}
pub async fn calculate_trust_by_path(
    State(state): State<AppState>,
    Path(trust): Path<i32>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let cache_key = format!("static:trust:calculate:{trust}");

    cached_handler(
        &mut state.redis.clone(),
        &cache_key,
        3600,
        &headers,
        || async {
            let level = calculate_trust(&state.game_data.favor, trust);

            let info = level.and_then(|idx| {
                state
                    .game_data
                    .favor
                    .favor_frames
                    .get(idx)
                    .map(|frame| TrustInfo {
                        level: idx,
                        favor_point: frame.data.favor_point,
                        percent: frame.data.percent,
                        battle_phase: frame.data.battle_phase,
                    })
            });

            Some(serde_json::json!({
                "trust": trust,
                "level": level,
                "info": info
            }))
        },
    )
    .await
}
