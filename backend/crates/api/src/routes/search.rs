use axum::Json;
use axum::extract::{Query, State};
use axum::routing::get;
use myrtle_app::AppContext;
use myrtle_app::services::leaderboard::search::{self, SearchPage, SearchParams};
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::Paging;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new().route("/search", get(search))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    has: Option<String>,
    support: Option<String>,
    all: Option<String>,
}

async fn search(
    State(ctx): State<AppContext>,
    Query(query): Query<SearchQuery>,
    Paging(page): Paging,
) -> ApiResult<Json<SearchPage>> {
    let params = SearchParams {
        q: query.q,
        sort: query.sort,
        dir: query.dir,
        has: query.has,
        support: query.support,
        all: query.all,
        limit: page.limit,
        offset: page.offset,
    };
    Ok(Json(search::search(&ctx, &params).await?))
}
