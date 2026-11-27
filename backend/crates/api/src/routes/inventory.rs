use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::sync::roster::{self, ItemEntry};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::MaybeCaller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/inventory", get(get_inventory))
}

#[derive(Deserialize)]
struct InventoryQuery {
    uid: Option<String>,
}

async fn get_inventory(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(query): Query<InventoryQuery>,
) -> ApiResult<Json<Vec<ItemEntry>>> {
    Ok(Json(
        roster::inventory(&ctx, &caller, query.uid.as_deref()).await?,
    ))
}
