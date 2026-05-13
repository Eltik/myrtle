use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::{
    app::{
        error::ApiError,
        extractors::pagination::Pagination,
        services::{self, leaderboard::LeaderboardPage},
        state::AppState,
    },
    database::models::score::{LeaderboardMover, PlayerStanding, ServerShare},
};

#[derive(Deserialize)]
pub struct LeaderboardParams {
    pub sort: Option<String>,   // defaults to "total_score"
    pub server: Option<String>, // optional server filter
    /// When set, each row is enriched with `rank_delta` vs. the most recent
    /// snapshot taken before this interval. Valid: "1 day" | "7 days" | "30 days".
    pub movement_interval: Option<String>,
    /// When true (requires `movement_interval`), the result is restricted to
    /// users whose rank has changed since the baseline.
    #[serde(default)]
    pub movement_only: bool,
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<LeaderboardPage>, ApiError> {
    let sort = params.sort.as_deref().unwrap_or("total_score");
    let movement_interval = params.movement_interval.as_deref();
    if let Some(interval) = movement_interval {
        if !matches!(interval, "1 day" | "7 days" | "30 days") {
            return Err(ApiError::BadRequest(
                "movement_interval must be '1 day', '7 days', or '30 days'".into(),
            ));
        }
    } else if params.movement_only {
        return Err(ApiError::BadRequest(
            "movement_only requires movement_interval".into(),
        ));
    }
    let page = services::leaderboard::get_leaderboard(
        &state,
        sort,
        params.server.as_deref(),
        movement_interval,
        params.movement_only,
        params.pagination.limit(),
        params.pagination.offset(),
    )
    .await?;
    Ok(Json(page))
}

#[derive(Deserialize)]
pub struct MoversParams {
    pub direction: Option<String>, // "up" (default) or "down"
    pub interval: Option<String>,  // "1 day", "7 days" (default), "30 days"
    pub server: Option<String>,
    pub limit: Option<u32>,
}

pub async fn top_movers(
    State(state): State<AppState>,
    Query(params): Query<MoversParams>,
) -> Result<Json<Vec<LeaderboardMover>>, ApiError> {
    let direction = params.direction.as_deref().unwrap_or("up");
    if direction != "up" && direction != "down" {
        return Err(ApiError::BadRequest(
            "direction must be 'up' or 'down'".into(),
        ));
    }
    let interval = params.interval.as_deref().unwrap_or("7 days");
    if !matches!(interval, "1 day" | "7 days" | "30 days") {
        return Err(ApiError::BadRequest("invalid interval".into()));
    }
    let limit = params.limit.unwrap_or(50).min(100);
    let movers = services::leaderboard::get_top_movers(
        &state,
        direction,
        interval,
        params.server.as_deref(),
        limit,
    )
    .await?;
    Ok(Json(movers))
}

#[derive(Deserialize)]
pub struct DistributionParams {
    pub top: Option<u32>,
}

pub async fn distribution(
    State(state): State<AppState>,
    Query(params): Query<DistributionParams>,
) -> Result<Json<Vec<ServerShare>>, ApiError> {
    let top_n = params.top.unwrap_or(250).min(10_000);
    let dist = services::leaderboard::get_distribution(&state, top_n).await?;
    Ok(Json(dist))
}

#[derive(Deserialize)]
pub struct StandingParams {
    pub uid: String,
    pub server: String,
    pub window: Option<u32>,
    pub interval: Option<String>,
}

pub async fn standing(
    State(state): State<AppState>,
    Query(params): Query<StandingParams>,
) -> Result<Json<PlayerStanding>, ApiError> {
    let window = params.window.unwrap_or(5).min(50);
    let interval = params.interval.as_deref().unwrap_or("7 days");
    if !matches!(interval, "1 day" | "7 days" | "30 days") {
        return Err(ApiError::BadRequest(
            "interval must be '1 day', '7 days', or '30 days'".into(),
        ));
    }
    let standing =
        services::leaderboard::get_standing(&state, &params.uid, &params.server, window, interval)
            .await?;
    Ok(Json(standing))
}
