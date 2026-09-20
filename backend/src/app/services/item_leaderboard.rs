use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::{
        models::item_leaderboard::{ItemHoldingSummary, ItemLeaderboardEntry, ItemStanding},
        queries::item_leaderboard,
    },
};

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize, Deserialize)]
pub struct ItemLeaderboardPage {
    pub item_id: String,
    pub entries: Vec<ItemLeaderboardEntry>,
    /// Visible holders of the item: the population the page is cut from.
    #[ts(type = "number")]
    pub total: i64,
}

pub async fn get_item_leaderboard(
    state: &AppState,
    item_id: &str,
    server: Option<&str>,
    q: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<ItemLeaderboardPage, ApiError> {
    let key = CacheKey::ItemLeaderboard {
        item: item_id,
        server,
        q,
        limit,
        offset,
    };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let (entries, total) = tokio::try_join!(
        item_leaderboard::get_item_leaderboard(
            &state.db,
            item_id,
            server,
            q,
            i64::from(limit),
            i64::from(offset)
        ),
        item_leaderboard::count_item_holders(&state.db, item_id, server, q),
    )?;
    let page = ItemLeaderboardPage {
        item_id: item_id.to_owned(),
        entries,
        total,
    };
    state.cache.set(&key, &page).await;
    Ok(page)
}

pub async fn get_item_catalog(
    state: &AppState,
    server: Option<&str>,
) -> Result<Vec<ItemHoldingSummary>, ApiError> {
    let key = CacheKey::ItemLeaderboardCatalog { server };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let rows = item_leaderboard::get_item_catalog(&state.db, server).await?;
    state.cache.set(&key, &rows).await;
    Ok(rows)
}

pub async fn get_item_standing(
    state: &AppState,
    item_id: &str,
    uid: &str,
    server: &str,
) -> Result<ItemStanding, ApiError> {
    let key = CacheKey::ItemLeaderboardStanding {
        item: item_id,
        uid,
        server,
    };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let standing = item_leaderboard::get_item_standing(&state.db, item_id, uid, server)
        .await?
        .ok_or(ApiError::NotFound)?;
    state.cache.set(&key, &standing).await;
    Ok(standing)
}
