use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::header;

use crate::app::error::ApiError;
use crate::app::services::tier_entity::{EntitySummary, catalogue};
use crate::app::state::AppState;
use crate::database::models::tier_list::EntityKind;

/// The catalogue changes only on a game-data reload; five minutes bounds how
/// long a browser keeps the old one.
const CACHE_CONTROL: &str = "public, max-age=300";

/// Everything of one kind a tier list editor may place, in pool order.
/// Static game data: the same for every list and every caller.
#[utoipa::path(
    get,
    path = "/tier-lists/catalogue/{kind}",
    tag = "tier-lists",
    params(
        ("kind" = EntityKind, Path, description = "Entity kind, e.g. `operator`."),
        ("server" = Option<String>, Query, description = "Game server whose data to read: `en`, `jp`, `kr`, `cn`, `tw` or `bili`. Defaults to the deployment's default server.")
    ),
    responses(
        (status = 200, description = "The kind's placeable entities, with `Cache-Control: public, max-age=300`.", body = Vec<EntitySummary>),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get(
    State(state): State<AppState>,
    Path(kind): Path<EntityKind>,
    Query(query): Query<super::ServerQuery>,
) -> Result<
    (
        [(header::HeaderName, &'static str); 1],
        Json<Vec<EntitySummary>>,
    ),
    ApiError,
> {
    let server = query.or_default(&state);
    let gd = state.game_data(server);
    let assets = state.asset_index(server);
    Ok((
        [(header::CACHE_CONTROL, CACHE_CONTROL)],
        Json(catalogue(&gd, &assets, kind)),
    ))
}
