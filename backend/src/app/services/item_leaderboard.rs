use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app::{cache::keys::CacheKey, error::ApiError, state::AppState},
    database::{
        models::item_leaderboard::{ItemCatalog, ItemLeaderboardEntry, ItemStanding},
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
    /// Every holding in that population summed: how much of the item exists
    /// across the ranked players.
    #[ts(type = "number")]
    pub total_quantity: i64,
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
    let (entries, totals) = tokio::try_join!(
        item_leaderboard::get_item_leaderboard(
            &state.db,
            item_id,
            server,
            q,
            i64::from(limit),
            i64::from(offset)
        ),
        item_leaderboard::item_holding_totals(&state.db, item_id, server, q),
    )?;
    let page = ItemLeaderboardPage {
        item_id: item_id.to_owned(),
        entries,
        total: totals.holders,
        total_quantity: totals.quantity,
    };
    state.cache.set(&key, &page).await;
    Ok(page)
}

pub async fn get_item_catalog(
    state: &AppState,
    server: Option<&str>,
) -> Result<ItemCatalog, ApiError> {
    let key = CacheKey::ItemLeaderboardCatalog { server };
    if let Some(cached) = state.cache.get(&key).await {
        return Ok(cached);
    }
    let (items, population) = tokio::try_join!(
        item_leaderboard::get_item_catalog(&state.db, server),
        item_leaderboard::count_visible_players(&state.db, server),
    )?;
    let catalog = ItemCatalog { population, items };
    state.cache.set(&key, &catalog).await;
    Ok(catalog)
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
