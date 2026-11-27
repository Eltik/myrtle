use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::board::{
    self, LeaderboardMover, LeaderboardPage, LeaderboardParams, MoversParams, PlayerStanding,
    ScoreHistoryPoint, ServerShare, StandingParams,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{MaybeCaller, Paging};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/leaderboard", get(leaderboard))
        .route("/leaderboard/movers", get(top_movers))
        .route("/leaderboard/distribution", get(distribution))
        .route("/leaderboard/standing", get(standing))
        .route("/leaderboard/history", get(score_history))
}

#[derive(Deserialize)]
struct LeaderboardQuery {
    sort: Option<String>,
    server: Option<String>,
    movement_interval: Option<String>,
    #[serde(default)]
    movement_only: bool,
    q: Option<String>,
}

async fn leaderboard(
    State(ctx): State<AppContext>,
    Query(query): Query<LeaderboardQuery>,
    Paging(page): Paging,
) -> ApiResult<Json<LeaderboardPage>> {
    let params = LeaderboardParams {
        sort: query.sort,
        server: query.server,
        movement_interval: query.movement_interval,
        movement_only: query.movement_only,
        q: query.q,
        limit: page.limit,
        offset: page.offset,
    };
    Ok(Json(board::page(&ctx, &params).await?))
}

async fn top_movers(
    State(ctx): State<AppContext>,
    Query(params): Query<MoversParams>,
) -> ApiResult<Json<Vec<LeaderboardMover>>> {
    Ok(Json(board::top_movers(&ctx, &params).await?))
}

#[derive(Deserialize)]
struct DistributionQuery {
    top: Option<u32>,
}

async fn distribution(
    State(ctx): State<AppContext>,
    Query(query): Query<DistributionQuery>,
) -> ApiResult<Json<Vec<ServerShare>>> {
    Ok(Json(board::distribution(&ctx, query.top).await?))
}

async fn standing(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<StandingParams>,
) -> ApiResult<Json<PlayerStanding>> {
    Ok(Json(board::standing(&ctx, &caller, &params).await?))
}

#[derive(Deserialize)]
struct HistoryQuery {
    uid: String,
}

async fn score_history(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<ScoreHistoryPoint>>> {
    Ok(Json(board::score_history(&ctx, &caller, &query.uid).await?))
}
