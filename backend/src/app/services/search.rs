use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::{models::user::UserProfile, queries::users},
};

pub async fn search_users(
    state: &AppState,
    query: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<UserProfile>, ApiError> {
    let mut hasher = DefaultHasher::new();
    (query, limit, offset).hash(&mut hasher);
    let key = CacheKey::Search {
        query_hash: hasher.finish(),
    };

    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    let results = users::search_by_nickname(&state.db, query, limit as i64, offset as i64).await?;
    state.cache.set(&key, &results).await;
    Ok(results)
}
