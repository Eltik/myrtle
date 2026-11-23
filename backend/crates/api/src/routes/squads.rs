use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use myrtle_app::AppContext;
use myrtle_app::services::community::squads::{
    self, SavedSquad, SavedSquadInput, SquadRules, SquadsResponse,
};
use myrtle_engine_squad::{SquadAnalysis, SquadAnalyzeRequest};

use crate::error::ApiResult;
use crate::extract::{self, Caller, ServerSel};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/squads", get(list))
        .route("/squads/{slot}", put(save).delete(remove))
        .route("/squads/analyze", post(analyze))
        .route("/squads/rules", get(rules))
}

async fn list(State(ctx): State<AppContext>, caller: Caller) -> ApiResult<Json<SquadsResponse>> {
    Ok(Json(squads::get(&ctx, &caller).await?))
}

async fn save(
    State(ctx): State<AppContext>,
    caller: Caller,
    ServerSel(server): ServerSel,
    Path(slot): Path<i32>,
    extract::Json(body): extract::Json<SavedSquadInput>,
) -> ApiResult<Json<SavedSquad>> {
    Ok(Json(squads::save(&ctx, &caller, slot, body, server).await?))
}

async fn remove(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slot): Path<i32>,
) -> ApiResult<StatusCode> {
    squads::delete(&ctx, &caller, slot).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn analyze(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    extract::Json(body): extract::Json<SquadAnalyzeRequest>,
) -> ApiResult<Json<SquadAnalysis>> {
    Ok(Json(squads::analyze(&ctx, body, server).await?))
}

async fn rules(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Json<SquadRules>> {
    Ok(Json(squads::rules(&ctx, server)?))
}
