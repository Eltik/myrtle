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

pub async fn get_inventory(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<InventoryParams>,
) -> Result<Json<Vec<ItemEntry>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = items::get_inventory(&state.db, user_id).await?;
    Ok(Json(entries))
}
