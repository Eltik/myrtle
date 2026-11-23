use axum::Json;
use axum::extract::State;
use axum::routing::{get, post};
use myrtle_app::AppContext;
use myrtle_app::services::story::progress::{self, StoryProgressDocument, StoryProgressResponse};

use crate::error::ApiResult;
use crate::extract::{self, Caller};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/user/story-progress", get(fetch).put(store))
        .route("/user/story-progress/import", post(import))
}

async fn fetch(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<StoryProgressResponse>> {
    Ok(Json(progress::get(&ctx, &caller).await?))
}

async fn store(
    State(ctx): State<AppContext>,
    caller: Caller,
    extract::Json(body): extract::Json<StoryProgressDocument>,
) -> ApiResult<Json<StoryProgressResponse>> {
    Ok(Json(progress::put(&ctx, &caller, body).await?))
}

async fn import(
    State(ctx): State<AppContext>,
    caller: Caller,
) -> ApiResult<Json<StoryProgressResponse>> {
    Ok(Json(progress::import(&ctx, &caller).await?))
}
