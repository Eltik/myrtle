use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::{AuthUser, MaybeAuthUser};
use crate::app::routes::tier_lists::ServerQuery;
use crate::app::services::grid::{self, Grid, GridInput, GridListResponse, GridSummary};
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::grids::ListOrder;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub sort: ListOrder,
    pub q: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub server: Option<Server>,
}

/// Listed grids, one page at a time.
#[utoipa::path(
    get,
    path = "/grids",
    operation_id = "grids_list",
    tag = "grids",
    params(
        ("sort" = Option<String>, Query, description = "`recent` (latest edit first, the default) or `popular` (most forked first, then latest edit)."),
        ("q" = Option<String>, Query, description = "Case-insensitive substring of the title."),
        ("page" = Option<i32>, Query, description = "1-based page number. Defaults to 1."),
        ("per_page" = Option<i32>, Query, description = "Grids per page, 1 to 60. Defaults to 24."),
        ("server" = Option<String>, Query, description = "Game server whose data resolves each preview icon: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server.")
    ),
    responses(
        (status = 200, description = "One page of listed grids and the total that matched.", body = GridListResponse),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<GridListResponse>, ApiError> {
    let server = query.server.unwrap_or(state.default_server);
    let page = grid::list(
        &state,
        query.sort,
        query.q.as_deref(),
        query.page,
        query.per_page,
        server,
    )
    .await?;
    Ok(Json(page))
}

/// Every grid the caller owns, listed or not, latest edit first.
#[utoipa::path(
    get,
    path = "/grids/mine",
    operation_id = "grids_mine",
    tag = "grids",
    params(
        ("server" = Option<String>, Query, description = "Game server whose data resolves each preview icon. Defaults to the deployment's default server.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The caller's grids.", body = Vec<GridSummary>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn mine(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ServerQuery>,
) -> Result<Json<Vec<GridSummary>>, ApiError> {
    let grids = grid::mine(&state, auth.user_uuid()?, query.or_default(&state)).await?;
    Ok(Json(grids))
}

/// One grid with every cell's entity resolved. An unlisted grid is still
/// served to anyone with its slug.
#[utoipa::path(
    get,
    path = "/grids/{slug}",
    operation_id = "grid_get",
    tag = "grids",
    params(
        ("slug" = String, Path, description = "Grid slug, as it appears in its URL."),
        ("server" = Option<String>, Query, description = "Game server whose data resolves each cell's `entity`. Defaults to the deployment's default server.")
    ),
    responses(
        (status = 200, description = "The grid. `can_edit` is true for its owner and site admins.", body = Grid),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Path(slug): Path<String>,
    Query(query): Query<ServerQuery>,
) -> Result<Json<Grid>, ApiError> {
    let viewer = auth
        .0
        .as_ref()
        .and_then(|a| a.user_uuid().ok().map(|id| (id, a.role)));
    let grid = grid::get(&state, &slug, query.or_default(&state), viewer).await?;
    Ok(Json(grid))
}

/// Create a grid. The caller becomes its owner; one account keeps at most 50.
#[utoipa::path(
    post,
    path = "/grids",
    operation_id = "grid_create",
    tag = "grids",
    params(
        ("server" = Option<String>, Query, description = "Game server whose data resolves each cell's `entity` in the response. Defaults to the deployment's default server.")
    ),
    request_body = GridInput,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 201, description = "The created grid.", body = Grid),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ServerQuery>,
    Json(body): Json<GridInput>,
) -> Result<(StatusCode, Json<Grid>), ApiError> {
    let grid = grid::create(
        &state,
        auth.user_uuid()?,
        auth.role,
        body,
        query.or_default(&state),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(grid)))
}

/// Replace a grid's title, description, size, cells and listing. Owner or
/// site admin only.
#[utoipa::path(
    put,
    path = "/grids/{slug}",
    operation_id = "grid_update",
    tag = "grids",
    params(
        ("slug" = String, Path, description = "Grid slug, as it appears in its URL."),
        ("server" = Option<String>, Query, description = "Game server whose data resolves each cell's `entity` in the response. Defaults to the deployment's default server.")
    ),
    request_body = GridInput,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The grid as stored.", body = Grid),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 422, response = crate::app::openapi::responses::ValidationFailed),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Query(query): Query<ServerQuery>,
    Json(body): Json<GridInput>,
) -> Result<Json<Grid>, ApiError> {
    let grid = grid::update(
        &state,
        &slug,
        auth.user_uuid()?,
        auth.role,
        body,
        query.or_default(&state),
    )
    .await?;
    Ok(Json(grid))
}

/// Delete a grid. Owner or site admin only. Its forks are kept and lose their
/// `template_of`.
#[utoipa::path(
    delete,
    path = "/grids/{slug}",
    operation_id = "grid_delete",
    tag = "grids",
    params(
        ("slug" = String, Path, description = "Grid slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 204, description = "The grid is gone."),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, ApiError> {
    grid::delete(&state, &slug, auth.user_uuid()?, auth.role).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Use a grid as a template: a new grid owned by the caller with its title,
/// description, size and labels, every cell empty, and listed. Counts toward
/// the caller's 50.
#[utoipa::path(
    post,
    path = "/grids/{slug}/fork",
    operation_id = "grid_fork",
    tag = "grids",
    params(
        ("slug" = String, Path, description = "Slug of the grid to copy."),
        ("server" = Option<String>, Query, description = "Game server whose data resolves the response. Defaults to the deployment's default server.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 201, description = "The new grid.", body = Grid),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn fork(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Query(query): Query<ServerQuery>,
) -> Result<(StatusCode, Json<Grid>), ApiError> {
    let grid = grid::fork(
        &state,
        &slug,
        auth.user_uuid()?,
        auth.role,
        query.or_default(&state),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(grid)))
}
