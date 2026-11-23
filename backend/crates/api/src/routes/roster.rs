use axum::Json;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::sync::roster;
use myrtle_types::roster::{RosterEntry, SupportUnit};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::MaybeCaller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/roster", get(get_roster))
        .route("/roster/{operator_id}", get(get_operator))
        .route("/get-user-supports", get(get_supports))
}

#[derive(Deserialize)]
struct RosterQuery {
    uid: Option<String>,
}

async fn get_roster(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<RosterQuery>,
) -> ApiResult<Json<Vec<RosterEntry>>> {
    Ok(Json(
        roster::operators(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}

async fn get_operator(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Path(operator_id): Path<String>,
    Query(query): Query<RosterQuery>,
) -> ApiResult<Json<RosterEntry>> {
    let entry = roster::operator(&ctx, &caller, query.uid.as_deref(), &operator_id).await?;
    Ok(Json(entry))
}

async fn get_supports(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<RosterQuery>,
) -> ApiResult<Json<Vec<SupportUnit>>> {
    Ok(Json(
        roster::supports(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}
