use axum::extract::{Path, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::levels;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/level/{stage_id}", get(get_level_map))
}

async fn get_level_map(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(stage_id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        levels::level(&ctx, server, &stage_id).await?,
    ))
}
