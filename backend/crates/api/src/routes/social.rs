use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::community::social;
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiResult;
use crate::extract::Caller;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/friends", get(get_friends))
        .route("/players/search", get(search_players))
}

#[derive(Deserialize)]
struct FriendsQuery {
    limit: Option<usize>,
}

async fn get_friends(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<FriendsQuery>,
) -> ApiResult<Json<Value>> {
    Ok(Json(social::friends(&ctx, &caller, query.limit).await?))
}

#[derive(Deserialize)]
struct SearchPlayersQuery {
    q: String,
    limit: Option<usize>,
}

async fn search_players(
    State(ctx): State<AppContext>,
    caller: Caller,
    Query(query): Query<SearchPlayersQuery>,
) -> ApiResult<Json<Value>> {
    let data = social::search_players(&ctx, &caller, &query.q, query.limit).await?;
    Ok(Json(data))
}
