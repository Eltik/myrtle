use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::improvements;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::MaybeCaller;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/user/improvements", get(get_user_improvements))
}

#[derive(Deserialize)]
struct ImprovementsParams {
    uid: Option<String>,
}

async fn get_user_improvements(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<ImprovementsParams>,
) -> ApiResult<Cached> {
    let json = improvements::improvements(&ctx, &caller, params.uid.as_deref()).await?;
    Ok(Cached::gamedata(json))
}
