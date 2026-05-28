use std::collections::HashMap;

use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
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
        auth.user_id.parse().map_err(|_| ApiError::Unauthorized)
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinPopularityResponse {
    /// Number of users counted in the denominator — every user that has any
    /// skin record (i.e. has imported their data at least once).
    pub total_users: i64,
    /// Map of `skin_id` → number of owners. Only non-default skins (`skin_id`
    /// containing `@`) are included; absent IDs imply zero owners.
    pub counts: HashMap<String, i64>,
    pub computed_at: String,
}

pub async fn get_skin_popularity(
    State(state): State<AppState>,
) -> Result<Json<SkinPopularityResponse>, ApiError> {
    let key = CacheKey::SkinPopularity;
    if let Some(cached) = state.cache.get::<SkinPopularityResponse>(&key).await {
        return Ok(Json(cached));
    }

    let (total_users, rows) = skins::get_skin_popularity(&state.db).await?;
    let counts: HashMap<String, i64> = rows.into_iter().map(|r| (r.skin_id, r.owners)).collect();

    let response = SkinPopularityResponse {
        total_users,
        counts,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(Json(response))
}
