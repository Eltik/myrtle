use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::{models::user::UserProfile, queries::users};

pub async fn get_user(state: &AppState, uid: &str) -> Result<UserProfile, ApiError> {
    let key = CacheKey::User { uid };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    let profile = users::find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;

    state.cache.set(&key, &profile).await;
    Ok(profile)
}
