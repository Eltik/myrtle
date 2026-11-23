use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::items::{
    self, ItemCatalog, ItemLeaderboardPage, ItemLeaderboardParams, ItemStanding, ItemStandingParams,
};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{MaybeCaller, Paging};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/leaderboard/items", get(item_leaderboard))
        .route("/leaderboard/items/catalog", get(item_catalog))
        .route("/leaderboard/items/standing", get(item_standing))
}

#[derive(Deserialize)]
struct ItemLeaderboardQuery {
    item: String,
    server: Option<String>,
    q: Option<String>,
}

async fn item_leaderboard(
    State(ctx): State<AppContext>,
    Query(query): Query<ItemLeaderboardQuery>,
    Paging(page): Paging,
) -> ApiResult<Json<ItemLeaderboardPage>> {
    let params = ItemLeaderboardParams {
        item: query.item,
        server: query.server,
        q: query.q,
        limit: page.limit,
        offset: page.offset,
    };
    Ok(Json(items::page(&ctx, &params).await?))
}

#[derive(Deserialize)]
struct ItemCatalogQuery {
    server: Option<String>,
}

async fn item_catalog(
    State(ctx): State<AppContext>,
    Query(query): Query<ItemCatalogQuery>,
) -> ApiResult<Json<ItemCatalog>> {
    Ok(Json(items::catalog(&ctx, query.server.as_deref()).await?))
}

async fn item_standing(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    Query(params): Query<ItemStandingParams>,
) -> ApiResult<Json<ItemStanding>> {
    Ok(Json(items::standing(&ctx, &caller, &params).await?))
}
