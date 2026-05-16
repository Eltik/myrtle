use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::queries::skins::OwnedSkin;
use crate::database::queries::{skins, users};

#[derive(Deserialize)]
pub struct SkinsParams {
    pub uid: Option<String>,
}

async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    match uid_param {
        Some(uid) => {
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
        }
        None => {
            let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
            auth.user_id.parse().map_err(|_| ApiError::Unauthorized)
        }
    }
}

pub async fn get_owned_skins(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<SkinsParams>,
) -> Result<Json<Vec<OwnedSkin>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = skins::get_owned_skins(&state.db, user_id).await?;
    Ok(Json(entries))
}
