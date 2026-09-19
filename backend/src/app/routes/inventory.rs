use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id;
use crate::app::state::AppState;
use crate::database::queries::items;
use crate::database::queries::items::ItemEntry;

#[derive(Deserialize)]
pub struct InventoryParams {
    pub uid: Option<String>,
}

/// A player's item inventory.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/inventory",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Every item the player holds, with counts.", body = Vec<ItemEntry>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_inventory(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<InventoryParams>,
) -> Result<Json<Vec<ItemEntry>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = items::get_inventory(&state.db, user_id).await?;
    Ok(Json(entries))
}
