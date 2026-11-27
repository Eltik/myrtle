use axum::extract::{Path, Query, State};
use axum::routing::get;
use myrtle_app::cache::CachedJson;
use myrtle_app::services::catalog::announcements;
use myrtle_app::{AppContext, AppError};
use myrtle_types::Server;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::respond::Cached;
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/announcements", get(list))
        .route("/announcements/{id}", get(detail))
}

#[derive(Deserialize)]
struct BoardQuery {
    server: Option<String>,
}

fn requested(query: &BoardQuery) -> Result<Option<Server>, AppError> {
    query
        .server
        .as_deref()
        .map(|s| Server::parse(s).ok_or(AppError::NotFound(None)))
        .transpose()
}

async fn list(State(ctx): State<AppContext>, Query(query): Query<BoardQuery>) -> ApiResult<Cached> {
    let board = announcements::list(&ctx, requested(&query)?).await?;
    let body = serde_json::to_string(&board).map_err(AppError::internal)?;
    Ok(Cached::gamedata(CachedJson::new(body)))
}

async fn detail(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Query(query): Query<BoardQuery>,
) -> ApiResult<Cached> {
    let json = announcements::detail(&ctx, requested(&query)?, &id).await?;
    Ok(Cached::gamedata(json))
}
