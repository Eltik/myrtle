use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::models::roster::RosterEntry;
use crate::database::queries::{roster, users};

#[derive(Deserialize)]
pub struct RosterParams {
    pub uid: Option<String>,
}

/// Resolve the target user_id from either a `uid` query param (public access)
/// or the authenticated user's token (private access).
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

            // Allow if it's the caller's own profile
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

pub async fn get_roster(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<RosterParams>,
) -> Result<Json<Vec<RosterEntry>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = roster::get_roster(&state.db, user_id).await?;
    Ok(Json(entries))
}

pub async fn get_operator(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Path(operator_id): Path<String>,
    Query(params): Query<RosterParams>,
) -> Result<Json<RosterEntry>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entry = roster::get_operator(&state.db, user_id, &operator_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(entry))
}
