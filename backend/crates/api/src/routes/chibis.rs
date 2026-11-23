use axum::extract::{Path, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::static_data;

use crate::error::ApiResult;
use crate::extract::ServerSel;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/chibis/{operator_id}", get(chibi_detail))
}

async fn chibi_detail(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Path(operator_id): Path<String>,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::chibi(&ctx, server, &operator_id).await?,
    ))
}
