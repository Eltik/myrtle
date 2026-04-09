use serde::{Deserialize, Serialize};

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::{models::score::LeaderboardEntry, queries::score},
};

#[derive(Serialize, Deserialize)]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
}

pub async fn get_leaderboard(
    state: &AppState,
    sort_by: &str,
    server: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<LeaderboardPage, ApiError> {
    // Cache key incorporates all query params
    let key = CacheKey::Leaderboard {
        sort: sort_by,
        server,
        page: offset / limit,
    };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    // Run count + data in parallel
    let (entries, total) = tokio::try_join!(
        score::get_leaderboard(&state.db, server, sort_by, limit as i64, offset as i64),
        score::count_leaderboard(&state.db, server),
    )?;

    let page = LeaderboardPage { entries, total };
    state.cache.set(&key, &page).await;
    Ok(page)
}
