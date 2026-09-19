use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::services::leaderboard::get_distribution;
use crate::app::services::leaderboard::get_leaderboard;
use crate::app::services::leaderboard::get_standing;
use crate::app::services::leaderboard::get_top_movers;
use crate::app::validation::is_valid_interval;
use crate::{
    app::{
        error::ApiError, extractors::pagination::Pagination,
        services::leaderboard::LeaderboardPage, state::AppState,
    },
    database::models::score::{LeaderboardMover, PlayerStanding, ScoreHistoryPoint, ServerShare},
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
    /// Free-text filter applied to nickname / uid via ILIKE.
    pub q: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

/// One page of the global ranking.
///
/// Rows come from the `v_leaderboard` view, which is refreshed on a schedule
/// rather than per request; `updated_at` is when that snapshot was taken.
#[utoipa::path(
    get,
    path = "/leaderboard",
    tag = "leaderboard",
    params(
        ("sort" = Option<String>, Query, description = "Column to rank by. Defaults to `total_score`."),
        ("server" = Option<String>, Query, description = "Restrict the page to one game server."),
        ("movement_interval" = Option<String>, Query, description = "Enrich each row with `rank_delta` against the newest snapshot older than this interval. One of `1 day`, `7 days`, `30 days`."),
        ("movement_only" = Option<bool>, Query, description = "Keep only rows whose rank changed since the baseline. Requires `movement_interval`."),
        ("q" = Option<String>, Query, description = "Free-text filter applied to nickname and uid."),
        ("limit" = Option<u32>, Query, description = "Page size. Defaults to 20, capped at 100."),
        ("offset" = Option<u32>, Query, description = "Rows to skip. Defaults to 0.")
    ),
    responses(
        (status = 200, description = "A page of ranked players, the total row count, and the snapshot time.", body = LeaderboardPage),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardParams>,
) -> Result<Json<LeaderboardPage>, ApiError> {
    let sort = params.sort.as_deref().unwrap_or("total_score");
    let movement_interval = params.movement_interval.as_deref();
    if let Some(interval) = movement_interval {
        if !is_valid_interval(interval) {
            return Err(ApiError::BadRequest(
                "movement_interval must be '1 day', '7 days', or '30 days'".into(),
            ));
        }
    } else if params.movement_only {
        return Err(ApiError::BadRequest(
            "movement_only requires movement_interval".into(),
        ));
    }
    let q = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let page = get_leaderboard(
        &state,
        sort,
        params.server.as_deref(),
        movement_interval,
        params.movement_only,
        q,
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

/// The players whose rank moved the most over an interval.
#[utoipa::path(
    get,
    path = "/leaderboard/movers",
    tag = "leaderboard",
    params(
        ("direction" = Option<String>, Query, description = "`up` (default) for climbers, `down` for fallers."),
        ("interval" = Option<String>, Query, description = "Comparison window: `1 day`, `7 days` (default) or `30 days`."),
        ("server" = Option<String>, Query, description = "Restrict to one game server."),
        ("limit" = Option<u32>, Query, description = "How many movers to return. Defaults to 50, capped at 100.")
    ),
    responses(
        (status = 200, description = "Movers ordered by the size of the rank change.", body = Vec<LeaderboardMover>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
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
    if !is_valid_interval(interval) {
        return Err(ApiError::BadRequest("invalid interval".into()));
    }
    let limit = params.limit.unwrap_or(50).min(100);
    let movers =
        get_top_movers(&state, direction, interval, params.server.as_deref(), limit).await?;
    Ok(Json(movers))
}

#[derive(Deserialize)]
pub struct DistributionParams {
    pub top: Option<u32>,
}

/// How the top N ranked players are split across game servers.
#[utoipa::path(
    get,
    path = "/leaderboard/distribution",
    tag = "leaderboard",
    params(
        ("top" = Option<u32>, Query, description = "Size of the prefix to measure. Defaults to 250, capped at 10000.")
    ),
    responses(
        (status = 200, description = "One row per server with its share of the prefix.", body = Vec<ServerShare>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn distribution(
    State(state): State<AppState>,
    Query(params): Query<DistributionParams>,
) -> Result<Json<Vec<ServerShare>>, ApiError> {
    let top_n = params.top.unwrap_or(250).min(10_000);
    let dist = get_distribution(&state, top_n).await?;
    Ok(Json(dist))
}

#[derive(Deserialize)]
pub struct StandingParams {
    pub uid: String,
    pub server: String,
    pub window: Option<u32>,
    pub interval: Option<String>,
}

/// One player's rank, percentile, and the players immediately around them.
///
/// Subject to the same privacy gate as every other by-uid endpoint: a private
/// profile is visible only to its owner.
#[utoipa::path(
    get,
    path = "/leaderboard/standing",
    tag = "leaderboard",
    params(
        ("uid" = String, Query, description = "The player to locate."),
        ("server" = String, Query, description = "The player's game server."),
        ("window" = Option<u32>, Query, description = "How many neighbours to return on each side. Defaults to 5, capped at 50."),
        ("interval" = Option<String>, Query, description = "Window for `rank_delta`: `1 day`, `7 days` (default) or `30 days`.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The player, their neighbours, and their percentile.", body = PlayerStanding),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn standing(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<StandingParams>,
) -> Result<Json<PlayerStanding>, ApiError> {
    let window = params.window.unwrap_or(5).min(50);
    let interval = params.interval.as_deref().unwrap_or("7 days");
    if !is_valid_interval(interval) {
        return Err(ApiError::BadRequest(
            "interval must be '1 day', '7 days', or '30 days'".into(),
        ));
    }
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let standing = get_standing(&state, &uid, &params.server, window, interval).await?;
    Ok(Json(standing))
}

#[derive(Deserialize)]
pub struct HistoryParams {
    pub uid: String,
}

/// A user's score/rank across every leaderboard snapshot, oldest first - the
/// Score tab's history chart.
///
/// Takes the same privacy gate as every other by-uid endpoint: a public
/// profile's history is public, a private one's is not, and a player always
/// sees their own.
#[utoipa::path(
    get,
    path = "/leaderboard/history",
    tag = "leaderboard",
    params(
        ("uid" = String, Query, description = "The player whose history to read.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Every snapshot the player appears in, oldest first.", body = Vec<ScoreHistoryPoint>),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn score_history(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<HistoryParams>,
) -> Result<Json<Vec<ScoreHistoryPoint>>, ApiError> {
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let points = crate::database::queries::score::get_score_history(&state.db, &uid).await?;
    Ok(Json(points))
}
