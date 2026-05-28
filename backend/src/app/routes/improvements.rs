use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::services::improvements::{ImprovementsResponse, get_improvements};
use crate::app::state::AppState;
use crate::database::queries::users;

#[derive(Deserialize)]
pub struct ImprovementsParams {
    pub uid: Option<String>,
}

pub async fn get_user_improvements(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<ImprovementsParams>,
) -> Result<Json<ImprovementsResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    let body = get_improvements(&state, &uid).await?;
    Ok(Json(body))
}

async fn resolve_uid(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<String, ApiError> {
    if let Some(uid) = uid_param {
        let profile = users::find_by_uid(&state.db, uid)
            .await?
            .ok_or(ApiError::NotFound)?;

        let is_own = auth
            .0
            .as_ref()
            .and_then(|a| a.user_id.parse::<uuid::Uuid>().ok())
            .is_some_and(|id| id == profile.id);

        if !is_own && profile.public_profile != Some(true) {
            return Err(ApiError::Forbidden);
        }

        Ok(profile.uid)
    } else {
        let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
        let user_uuid: uuid::Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
        let profile = users::find_by_id(&state.db, user_uuid)
            .await?
            .ok_or(ApiError::Unauthorized)?;
        Ok(profile.uid)
    }
}
