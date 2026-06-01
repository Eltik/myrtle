use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::queries::items::ItemEntry;
use crate::database::queries::{items, users};

#[derive(Deserialize)]
pub struct InventoryParams {
    pub uid: Option<String>,
}

async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    if let Some(uid) = uid_param {
        let profile = users::find_by_uid(&state.db, uid)
            .await?
            .ok_or(ApiError::NotFound)?;

        let is_own = auth
            .0
            .as_ref()
            .and_then(|a| a.user_id.parse::<Uuid>().ok())
            .is_some_and(|id| id == profile.id);

        if !is_own && profile.public_profile != Some(true) {
            return Err(ApiError::Forbidden);
        }

        Ok(profile.id)
    } else {
        let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
        auth.user_uuid()
    }
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
