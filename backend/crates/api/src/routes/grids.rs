use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use myrtle_app::AppContext;
use myrtle_app::services::community::grids::{
    self, Grid, GridInput, GridListResponse, GridSummary,
};
use myrtle_db::repos::grids::ListOrder;
use serde::Deserialize;

use crate::error::ApiResult;
use crate::extract::{self, Caller, MaybeCaller, ServerSel};
use crate::router::ApiRouter;

pub fn routes() -> ApiRouter {
    ApiRouter::new()
        .route("/grids", get(list).post(create))
        .route("/grids/mine", get(mine))
        .route("/grids/{slug}", get(one).put(update).delete(remove))
        .route("/grids/{slug}/fork", post(fork))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum Sort {
    #[default]
    Recent,
    Popular,
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    sort: Sort,
    q: Option<String>,
    page: Option<i32>,
    per_page: Option<i32>,
}

async fn list(
    State(ctx): State<AppContext>,
    ServerSel(server): ServerSel,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<GridListResponse>> {
    let order = match query.sort {
        Sort::Recent => ListOrder::Recent,
        Sort::Popular => ListOrder::Popular,
    };
    let page = grids::list(
        &ctx,
        order,
        query.q.as_deref(),
        query.page,
        query.per_page,
        Some(server),
    )
    .await?;
    Ok(Json(page))
}

async fn mine(
    State(ctx): State<AppContext>,
    caller: Caller,
    ServerSel(server): ServerSel,
) -> ApiResult<Json<Vec<GridSummary>>> {
    Ok(Json(grids::mine(&ctx, &caller, Some(server)).await?))
}

async fn one(
    State(ctx): State<AppContext>,
    caller: MaybeCaller,
    ServerSel(server): ServerSel,
    Path(slug): Path<String>,
) -> ApiResult<Json<Grid>> {
    Ok(Json(grids::get(&ctx, &caller, &slug, Some(server)).await?))
}

async fn create(
    State(ctx): State<AppContext>,
    caller: Caller,
    ServerSel(server): ServerSel,
    extract::Json(body): extract::Json<GridInput>,
) -> ApiResult<(StatusCode, Json<Grid>)> {
    let grid = grids::create(&ctx, &caller, body, Some(server)).await?;
    Ok((StatusCode::CREATED, Json(grid)))
}

async fn update(
    State(ctx): State<AppContext>,
    caller: Caller,
    ServerSel(server): ServerSel,
    Path(slug): Path<String>,
    extract::Json(body): extract::Json<GridInput>,
) -> ApiResult<Json<Grid>> {
    Ok(Json(
        grids::update(&ctx, &caller, &slug, body, Some(server)).await?,
    ))
}

async fn remove(
    State(ctx): State<AppContext>,
    caller: Caller,
    Path(slug): Path<String>,
) -> ApiResult<StatusCode> {
    grids::delete(&ctx, &caller, &slug).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn fork(
    State(ctx): State<AppContext>,
    caller: Caller,
    ServerSel(server): ServerSel,
    Path(slug): Path<String>,
) -> ApiResult<(StatusCode, Json<Grid>)> {
    let grid = grids::fork(&ctx, &caller, &slug, Some(server)).await?;
    Ok((StatusCode::CREATED, Json(grid)))
}
