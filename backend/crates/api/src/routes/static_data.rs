use axum::extract::{Path, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::static_data;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/static/{resource}", get(get_static))
}

async fn get_static(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(resource): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::resource(&ctx, server, &resource).await?,
    ))
}
