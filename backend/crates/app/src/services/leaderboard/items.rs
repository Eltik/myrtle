//! The inventory leaderboard: who holds the most of one item or currency.

use std::time::Duration;

use myrtle_db::repos::items::{self, Holding};
use myrtle_types::{PageRequest, Server};
use serde::{Deserialize, Serialize};

use super::{server_filter, trimmed};
use crate::privacy::{self, layout::ProfileTabId};
use crate::{AppContext, AppError, Subject};

const PAGE_TTL: Duration = Duration::from_mins(5);
const CATALOG_TTL: Duration = Duration::from_mins(10);
const STANDING_TTL: Duration = Duration::from_mins(1);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemLeaderboardEntry {
    /// Ties share a rank
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub grade: Option<String>,
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemLeaderboardPage {
    pub item_id: String,
    pub entries: Vec<ItemLeaderboardEntry>,
    /// Visible holders of the item
    pub total: i64,
    pub total_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemHoldingSummary {
    pub item_id: String,
    pub holders: i64,
    pub top: i64,
    pub total_quantity: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemCatalog {
    /// Visible players under the same filter as `holders`
    pub population: i64,
    /// Most holders first
    pub items: Vec<ItemHoldingSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStanding {
    pub item_id: String,
    pub quantity: i64,
    pub rank_global: i64,
    pub holders_global: i64,
    pub rank_server: i64,
    pub holders_server: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ItemLeaderboardParams {
    pub item: String,
    pub server: Option<String>,
    pub q: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemStandingParams {
    pub item: String,
    pub uid: String,
    pub server: String,
}

#[must_use]
pub fn is_valid_item_id(item_id: &str) -> bool {
    !item_id.is_empty()
        && item_id.len() <= 50
        && item_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

fn validated_item_id(item: &str) -> Result<&str, AppError> {
    let item = item.trim();
    if is_valid_item_id(item) {
        Ok(item)
    } else {
        Err(AppError::bad_request(
            "item must be a game item id: 1 to 50 characters of [A-Za-z0-9_]",
        ))
    }
}

pub async fn page(
    ctx: &AppContext,
    params: &ItemLeaderboardParams,
) -> Result<ItemLeaderboardPage, AppError> {
    let item = validated_item_id(&params.item)?;
    let server = server_filter(params.server.as_deref())?;
    let q = trimmed(params.q.as_deref());
    let paging = PageRequest {
        limit: params.limit,
        offset: params.offset,
    };
    let (limit, offset) = (paging.limit(), paging.offset());
    let key = format!(
        "leaderboard:items:{item}:{}:{}:{limit}:{offset}",
        server.map_or("all", Server::as_str),
        q.unwrap_or(""),
    );
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let holding = Holding::of(item);
    let mut conn = ctx.db.conn().await?;
    let rows = items::leaderboard(
        &mut conn,
        holding,
        server,
        q,
        i64::from(limit),
        i64::from(offset),
    )
    .await?;
    let totals = items::holding_totals(&mut conn, holding, server, q).await?;
    let page = ItemLeaderboardPage {
        item_id: item.to_owned(),
        entries: rows
            .into_iter()
            .map(|r| ItemLeaderboardEntry {
                rank: r.rank,
                uid: r.uid,
                nickname: r.nickname,
                nick_number: r.nick_number,
                level: r.level,
                avatar_id: r.avatar_id,
                server: r.server,
                grade: r.grade,
                quantity: r.quantity,
            })
            .collect(),
        total: totals.holders,
        total_quantity: totals.quantity,
    };
    ctx.cache.set_json(&key, &page, PAGE_TTL).await;
    Ok(page)
}

pub async fn catalog(ctx: &AppContext, server: Option<&str>) -> Result<ItemCatalog, AppError> {
    let server = server_filter(server)?;
    let key = format!(
        "leaderboard:items:catalog:{}",
        server.map_or("all", Server::as_str)
    );
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let rows = items::catalog(&mut conn, server).await?;
    let population = items::count_visible_players(&mut conn, server).await?;
    let catalog = ItemCatalog {
        population,
        items: rows
            .into_iter()
            .map(|r| ItemHoldingSummary {
                item_id: r.item_id,
                holders: r.holders,
                top: r.top,
                total_quantity: r.total_quantity,
            })
            .collect(),
    };
    ctx.cache.set_json(&key, &catalog, CATALOG_TTL).await;
    Ok(catalog)
}

pub async fn standing(
    ctx: &AppContext,
    subject: &Subject,
    params: &ItemStandingParams,
) -> Result<ItemStanding, AppError> {
    let item = validated_item_id(&params.item)?;
    let server =
        Server::parse(&params.server).ok_or_else(|| AppError::bad_request("unknown server"))?;
    let uid =
        privacy::resolve_uid(ctx, subject, Some(&params.uid), ProfileTabId::Inventory).await?;
    let key = format!(
        "leaderboard:items:standing:{item}:{}:{uid}",
        server.as_str(),
    );
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let row = items::standing(&mut conn, Holding::of(item), &uid, server)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let standing = ItemStanding {
        item_id: row.item_id,
        quantity: row.quantity,
        rank_global: row.rank_global,
        holders_global: row.holders_global,
        rank_server: row.rank_server,
        holders_server: row.holders_server,
    };
    ctx.cache.set_json(&key, &standing, STANDING_TTL).await;
    Ok(standing)
}
