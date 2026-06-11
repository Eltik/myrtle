use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::database::queries::users::count_by_nickname;
use crate::database::queries::users::search_by_nickname;
use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::models::user::UserProfile,
};

#[derive(Serialize, Deserialize)]
pub struct SearchPage {
    pub entries: Vec<UserProfile>,
    pub total: i64,
}

pub async fn search_users(
    state: &AppState,
    query: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<SearchPage, ApiError> {
    let mut hasher = DefaultHasher::new();
    (query, limit, offset).hash(&mut hasher);
    let key = CacheKey::Search {
        query_hash: hasher.finish(),
    };

    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    let (entries, total) = tokio::try_join!(
        search_by_nickname(&state.db, query, i64::from(limit), i64::from(offset)),
        count_by_nickname(&state.db, query),
    )?;

    let page = SearchPage { entries, total };
    state.cache.set(&key, &page).await;
    Ok(page)
}
