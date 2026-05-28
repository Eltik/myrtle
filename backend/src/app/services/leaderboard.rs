use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::{
        models::score::{LeaderboardEntry, LeaderboardMover, PlayerStanding, ServerShare},
        queries::score,
    },
};

#[derive(Serialize, Deserialize)]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
    pub updated_at: Option<DateTime<Utc>>,
}

#[allow(clippy::too_many_arguments)]
pub async fn get_leaderboard(
    state: &AppState,
    sort_by: &str,
    server: Option<&str>,
    movement_interval: Option<&str>,
    movement_only: bool,
    q: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<LeaderboardPage, ApiError> {
    // Cache key incorporates all query params
    let key = CacheKey::Leaderboard {
        sort: sort_by,
        server,
        movement_interval,
        movement_only,
        q,
        limit,
        offset,
    };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }

    // Run count + data in parallel
    let (entries, total, updated_at) = tokio::try_join!(
        score::get_leaderboard(
            &state.db,
            server,
            sort_by,
            movement_interval,
            movement_only,
            q,
            i64::from(limit),
            i64::from(offset)
        ),
        score::count_leaderboard(&state.db, server, movement_interval, movement_only, q),
        score::get_last_updated(&state.db, server),
    )?;

    let page = LeaderboardPage {
        entries,
        total,
        updated_at,
    };
    state.cache.set(&key, &page).await;
    Ok(page)
}

pub async fn get_top_movers(
    state: &AppState,
    direction: &str,
    interval: &str,
    server: Option<&str>,
    limit: u32,
) -> Result<Vec<LeaderboardMover>, ApiError> {
    let key = CacheKey::LeaderboardMovers {
        direction,
        interval,
        server,
    };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let movers =
        score::get_top_movers(&state.db, interval, direction, server, i64::from(limit)).await?;
    state.cache.set(&key, &movers).await;
    Ok(movers)
}

pub async fn get_distribution(state: &AppState, top_n: u32) -> Result<Vec<ServerShare>, ApiError> {
    let key = CacheKey::LeaderboardDistribution { top_n };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let dist = score::get_server_distribution(&state.db, i64::from(top_n)).await?;
    state.cache.set(&key, &dist).await;
    Ok(dist)
}

pub async fn get_standing(
    state: &AppState,
    uid: &str,
    server: &str,
    window: u32,
    interval: &str,
) -> Result<PlayerStanding, ApiError> {
    score::get_player_standing(&state.db, uid, server, i64::from(window), interval)
        .await?
        .ok_or(ApiError::NotFound)
}
