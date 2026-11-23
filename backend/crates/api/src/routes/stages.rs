use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::static_data;
use myrtle_app::services::sync::roster::{self, StageClearEntry};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{MaybeCaller, ServerSel};
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/stages/{stage_id}/detail", get(stage_detail))
        .route("/stage-clears", get(stage_clears))
}

async fn stage_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(stage_id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::stage_detail(&ctx, server, &stage_id).await?,
    ))
}

#[derive(Deserialize)]
struct StageClearsQuery {
    uid: Option<String>,
}

async fn stage_clears(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<StageClearsQuery>,
) -> ApiResult<Json<HashMap<String, StageClearEntry>>> {
    let clears = roster::stage_clears(&ctx, &caller, query.uid.as_deref()).await?;
    Ok(Json(clears))
}
