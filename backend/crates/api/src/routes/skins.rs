use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::catalog::static_data;
use myrtle_app::services::sync::roster::{self, OwnedSkin};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{MaybeCaller, ServerSel};
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/skins/index", get(skins_index))
        .route("/skins/popularity", get(skin_popularity))
        .route("/user-skins", get(owned_skins))
}

async fn skins_index(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
) -> ApiResult<Cached> {
    Ok(Cached::gamedata(
        static_data::skins_index(&ctx, server).await?,
    ))
}

async fn skin_popularity(State(ctx): State<AppContext>) -> ApiResult<Cached> {
    Ok(static_data::skin_popularity(&ctx).await?.into())
}

#[derive(Deserialize)]
struct UidQuery {
    uid: Option<String>,
}

async fn owned_skins(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<UidQuery>,
) -> ApiResult<Json<Vec<OwnedSkin>>> {
    Ok(Json(
        roster::owned_skins(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}
