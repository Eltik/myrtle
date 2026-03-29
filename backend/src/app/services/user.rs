use crate::app::cache::{keys::CacheKey, store};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::{models::user::UserProfile, queries::users};

pub async fn get_user(state: &AppState, uid: &str) -> Result<UserProfile, ApiError> {
    // Check cache
    let key = CacheKey::User { uid };
    if let Some(cached) = store::get(&mut state.redis.clone(), &key).await {
        return Ok(cached);
    }

    // DB lookup
    let profile = users::find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Cache and return
    store::set(&mut state.redis.clone(), &key, &profile).await;
    Ok(profile)
}
