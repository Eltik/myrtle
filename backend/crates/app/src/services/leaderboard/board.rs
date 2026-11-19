//! The score leaderboard: pages, movers, server shares, one player's standing.

use std::time::Duration;

use chrono::{DateTime, Utc};
use myrtle_db::repos::leaderboard::{
    self, LeaderboardRow, LeaderboardSort, Movement, MoverDirection, MoverRow,
};
use myrtle_db::repos::scores;
use myrtle_types::{PageRequest, Server};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{server_filter, trimmed};
use crate::privacy::{self, layout::ProfileTabId};
use crate::{AppContext, AppError, Subject};

const PAGE_TTL: Duration = Duration::from_mins(5);
const MOVERS_TTL: Duration = Duration::from_mins(15);
const DISTRIBUTION_TTL: Duration = Duration::from_mins(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub server: String,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub operator_score: Option<f64>,
    pub stage_score: Option<f64>,
    pub roguelike_score: Option<f64>,
    pub sandbox_score: Option<f64>,
    pub medal_score: Option<f64>,
    pub base_score: Option<f64>,
    pub skin_score: Option<f64>,
    pub rank_global: Option<i64>,
    pub rank_server: Option<i64>,
    /// Positive = climbed; `None` without a movement interval or a baseline
    pub rank_delta: Option<i64>,
}

impl From<LeaderboardRow> for LeaderboardEntry {
    fn from(r: LeaderboardRow) -> Self {
        Self {
            id: r.id,
            uid: r.uid,
            nickname: r.nickname,
            nick_number: r.nick_number,
            level: r.level,
            avatar_id: r.avatar_id,
            secretary: r.secretary,
            secretary_skin_id: r.secretary_skin_id,
            server: r.server,
            total_score: r.total_score,
            grade: r.grade,
            operator_score: r.operator_score,
            stage_score: r.stage_score,
            roguelike_score: r.roguelike_score,
            sandbox_score: r.sandbox_score,
            medal_score: r.medal_score,
            base_score: r.base_score,
            skin_score: r.skin_score,
            rank_global: r.rank_global,
            rank_server: r.rank_server,
            rank_delta: r.rank_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total: i64,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardMover {
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub current_rank: i64,
    pub previous_rank: i64,
    pub rank_delta: i64,
    pub current_score: Option<f64>,
    pub score_delta: Option<f64>,
}

impl From<MoverRow> for LeaderboardMover {
    fn from(r: MoverRow) -> Self {
        Self {
            uid: r.uid,
            nickname: r.nickname,
            nick_number: r.nick_number,
            avatar_id: r.avatar_id,
            server: r.server,
            current_rank: r.current_rank,
            previous_rank: r.previous_rank,
            rank_delta: r.rank_delta,
            current_score: r.current_score,
            score_delta: r.score_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerShare {
    pub server: String,
    pub players: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerStanding {
    pub player: LeaderboardEntry,
    pub neighbors: Vec<LeaderboardEntry>,
    /// 0.0 = top, 1.0 = bottom
    pub percentile: f64,
    pub rank_delta: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreHistoryPoint {
    pub taken_at: DateTime<Utc>,
    pub total_score: Option<f64>,
    pub rank_global: i32,
    pub rank_server: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LeaderboardParams {
    pub sort: Option<String>,
    pub server: Option<String>,
    pub movement_interval: Option<String>,
    #[serde(default)]
    pub movement_only: bool,
    pub q: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MoversParams {
    pub direction: Option<String>,
    pub interval: Option<String>,
    pub server: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StandingParams {
    pub uid: String,
    pub server: String,
    pub window: Option<u32>,
    pub interval: Option<String>,
}

#[must_use]
pub fn is_valid_interval(interval: &str) -> bool {
    matches!(interval, "1 day" | "7 days" | "30 days")
}

/// Unknown columns rank by total score
fn sort_of(sort: Option<&str>) -> LeaderboardSort {
    use LeaderboardSort as S;
    [
        S::Operator,
        S::Stage,
        S::Roguelike,
        S::Sandbox,
        S::Medal,
        S::Base,
        S::Skin,
    ]
    .into_iter()
    .find(|s| Some(s.column()) == sort)
    .unwrap_or(S::Total)
}

fn interval_or_week(interval: Option<&str>, message: &str) -> Result<String, AppError> {
    let interval = interval.unwrap_or("7 days");
    if is_valid_interval(interval) {
        Ok(interval.to_owned())
    } else {
        Err(AppError::bad_request(message))
    }
}

pub async fn page(
    ctx: &AppContext,
    params: &LeaderboardParams,
) -> Result<LeaderboardPage, AppError> {
    let interval = params.movement_interval.as_deref();
    match interval {
        Some(i) if !is_valid_interval(i) => {
            return Err(AppError::bad_request(
                "movement_interval must be '1 day', '7 days', or '30 days'",
            ));
        }
        None if params.movement_only => {
            return Err(AppError::bad_request(
                "movement_only requires movement_interval",
            ));
        }
        _ => {}
    }
    let sort = sort_of(params.sort.as_deref());
    let server = server_filter(params.server.as_deref())?;
    let q = trimmed(params.q.as_deref());
    let paging = PageRequest {
        limit: params.limit,
        offset: params.offset,
    };
    let (limit, offset) = (paging.limit(), paging.offset());
    let key = format!(
        "leaderboard:{}:{}:{}:{}:{}:{limit}:{offset}",
        sort.column(),
        server.map_or("all", Server::as_str),
        interval.map_or_else(|| "none".to_owned(), |i| i.replace(' ', "_")),
        if params.movement_only { "only" } else { "all" },
        q.unwrap_or(""),
    );
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let movement = interval.map(|interval| Movement {
        interval,
        moved_only: params.movement_only,
    });
    let mut conn = ctx.db.conn().await?;
    let rows = leaderboard::page(
        &mut conn,
        sort,
        server,
        q,
        movement,
        i64::from(limit),
        i64::from(offset),
    )
    .await?;
    let total = leaderboard::count(&mut conn, server, q, movement).await?;
    let updated_at = scores::last_calculated_at(&mut conn, server).await?;
    let page = LeaderboardPage {
        entries: rows.into_iter().map(Into::into).collect(),
        total,
        updated_at,
    };
    ctx.cache.set_json(&key, &page, PAGE_TTL).await;
    Ok(page)
}

pub async fn top_movers(
    ctx: &AppContext,
    params: &MoversParams,
) -> Result<Vec<LeaderboardMover>, AppError> {
    let (direction, word) = match params.direction.as_deref().unwrap_or("up") {
        "up" => (MoverDirection::Up, "up"),
        "down" => (MoverDirection::Down, "down"),
        _ => return Err(AppError::bad_request("direction must be 'up' or 'down'")),
    };
    let interval = interval_or_week(params.interval.as_deref(), "invalid interval")?;
    let limit = params.limit.unwrap_or(50).min(100);
    let server = server_filter(params.server.as_deref())?;
    let key = format!(
        "leaderboard:movers:{word}:{}:{}:{limit}",
        interval.replace(' ', "_"),
        server.map_or("all", Server::as_str),
    );
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let movers: Vec<LeaderboardMover> =
        leaderboard::top_movers(&mut conn, &interval, direction, server, i64::from(limit))
            .await?
            .into_iter()
            .map(Into::into)
            .collect();
    ctx.cache.set_json(&key, &movers, MOVERS_TTL).await;
    Ok(movers)
}

/// How the top `top` (250 when absent, at most 10 000) players split across servers
pub async fn distribution(
    ctx: &AppContext,
    top: Option<u32>,
) -> Result<Vec<ServerShare>, AppError> {
    let top_n = top.unwrap_or(250).min(10_000);
    let key = format!("leaderboard:distribution:{top_n}");
    if let Some(cached) = ctx.cache.get_json(&key).await {
        return Ok(cached);
    }
    let mut conn = ctx.db.conn().await?;
    let shares: Vec<ServerShare> = scores::server_shares(&mut conn, i64::from(top_n))
        .await?
        .into_iter()
        .map(|r| ServerShare {
            server: r.server,
            players: r.players,
        })
        .collect();
    ctx.cache.set_json(&key, &shares, DISTRIBUTION_TTL).await;
    Ok(shares)
}

/// 404 for the owner too while the Score tab is private: the board leaves them out
pub async fn standing(
    ctx: &AppContext,
    subject: &Subject,
    params: &StandingParams,
) -> Result<PlayerStanding, AppError> {
    let window = params.window.unwrap_or(5).min(50);
    let interval = interval_or_week(
        params.interval.as_deref(),
        "interval must be '1 day', '7 days', or '30 days'",
    )?;
    let server =
        Server::parse(&params.server).ok_or_else(|| AppError::bad_request("unknown server"))?;
    let uid = privacy::resolve_uid(ctx, subject, Some(&params.uid), ProfileTabId::Score).await?;
    let mut conn = ctx.db.conn().await?;
    let rows = leaderboard::standing(&mut conn, &uid, server, i64::from(window), &interval)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let rank = rows.player.rank_global.unwrap_or(0);
    let percentile = if rows.ranked > 0 {
        rank as f64 / rows.ranked as f64
    } else {
        0.0
    };
    Ok(PlayerStanding {
        player: rows.player.into(),
        neighbors: rows.neighbours.into_iter().map(Into::into).collect(),
        percentile,
        rank_delta: rows.previous_rank.map(|p| i64::from(p) - rank),
    })
}

/// Every snapshot the player appears in, oldest first
pub async fn score_history(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
) -> Result<Vec<ScoreHistoryPoint>, AppError> {
    let uid = privacy::resolve_uid(ctx, subject, Some(uid), ProfileTabId::Score).await?;
    let mut conn = ctx.db.conn().await?;
    Ok(scores::history(&mut conn, &uid, None)
        .await?
        .into_iter()
        .map(|r| ScoreHistoryPoint {
            taken_at: r.taken_at,
            total_score: r.total_score,
            rank_global: r.rank_global,
            rank_server: r.rank_server,
        })
        .collect())
}
