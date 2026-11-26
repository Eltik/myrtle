pub mod page;

use std::sync::LazyLock;
use std::time::Duration;

use chrono::{Datelike, NaiveDate};
use myrtle_hypergryph::Domain;
use myrtle_types::Server;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::cache::CachedJson;
use crate::{AppContext, AppError};
use page::PageParts;

const DEFAULT_ANNOUNCE_URL: &str =
    "https://ark-us-static-online.yo-star.com/announce/{0}/announcement.meta.json";
const ANNOUNCE_PLATFORM: &str = "Android";
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const FUTURE_SLACK_DAYS: i64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AnnouncementSummary {
    pub id: String,
    pub title: String,
    pub group: String,
    pub published_on: Option<String>,
    pub banner_url: Option<String>,
    pub notice_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct Announcement {
    #[serde(flatten)]
    #[ts(flatten)]
    pub summary: AnnouncementSummary,
    pub server: String,
    pub headline: Option<String>,
    pub body_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct AnnouncementList {
    pub server: String,
    pub servers: Vec<String>,
    pub announcements: Vec<AnnouncementSummary>,
    #[ts(type = "number | null")]
    pub checked_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub announcements: Vec<AnnouncementSummary>,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaItem {
    pub announce_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub web_url: String,
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub day: u32,
    #[serde(default)]
    pub month: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    #[serde(default)]
    announce_list: Vec<MetaItem>,
}

#[derive(Debug, Clone)]
pub struct Listed {
    pub item: MetaItem,
    pub banner_url: Option<String>,
    pub html: Option<String>,
}

static RE_IMG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<img\b[^>]*>").expect("static regex is valid"));
static RE_SRC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)\bsrc\s*=\s*"([^"]*)""#).expect("static regex is valid"));

#[must_use]
pub fn board_key(server: Server) -> String {
    format!("announcements:{}", server.as_str())
}

#[must_use]
pub fn detail_key(server: Server, id: &str) -> String {
    format!("announcements:{}:{id}", server.as_str())
}

#[must_use]
pub fn servers(ctx: &AppContext) -> Vec<Server> {
    let config = ctx.upstream.config();
    Server::ALL
        .into_iter()
        .filter(|&s| config.yostar_sdk(s).is_some())
        .collect()
}

pub fn parse_meta(raw: &str) -> Result<Vec<MetaItem>, serde_json::Error> {
    let meta: Meta = serde_json::from_str(raw)?;
    Ok(meta
        .announce_list
        .into_iter()
        .filter(|a| !a.web_url.is_empty() && !a.announce_id.is_empty())
        .collect())
}

#[must_use]
pub fn published_on(day: u32, month: u32, today: NaiveDate) -> Option<NaiveDate> {
    let limit = today + chrono::Duration::days(FUTURE_SLACK_DAYS);
    [today.year(), today.year() - 1]
        .into_iter()
        .filter_map(|y| NaiveDate::from_ymd_opt(y, month, day))
        .find(|d| *d <= limit)
}

pub fn summary(
    item: &MetaItem,
    banner_url: Option<String>,
    today: NaiveDate,
) -> AnnouncementSummary {
    AnnouncementSummary {
        id: item.announce_id.clone(),
        title: item
            .title
            .split('\n')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        group: item.group.clone(),
        published_on: published_on(item.day, item.month, today)
            .map(|d| d.format("%Y-%m-%d").to_string()),
        banner_url,
        notice_url: item.web_url.clone(),
    }
}

#[must_use]
pub fn announcement(
    server: Server,
    item: &MetaItem,
    parts: &PageParts,
    today: NaiveDate,
) -> Announcement {
    Announcement {
        summary: summary(item, parts.banner_url.clone(), today),
        server: server.as_str().to_owned(),
        headline: parts.headline.clone(),
        body_html: parts.body_html.clone(),
    }
}

pub fn sort_newest_first(list: &mut [AnnouncementSummary]) {
    list.sort_by_cached_key(|n| {
        std::cmp::Reverse((n.published_on.clone(), n.id.parse::<u64>().unwrap_or(0)))
    });
}

pub fn banner_url(html: &str, page_url: &str) -> Option<String> {
    let src = RE_IMG
        .find_iter(html)
        .map(|m| m.as_str())
        .find(|tag| tag.contains("banner-image"))
        .and_then(|tag| RE_SRC.captures(tag))?[1]
        .replace("&amp;", "&");
    let url = match Url::parse(&src) {
        Ok(url) => url,
        Err(_) => Url::parse(page_url).ok()?.join(&src).ok()?,
    };
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

pub fn board_server(ctx: &AppContext, requested: Option<Server>) -> Result<Server, AppError> {
    let servers = servers(ctx);
    match requested {
        Some(server) if servers.contains(&server) => Ok(server),
        Some(server) => Err(AppError::NotFound(Some(format!(
            "no notice board is read for `{}`",
            server.as_str()
        )))),
        None => {
            let default = ctx.snapshots.default_server();
            Ok(servers
                .iter()
                .copied()
                .find(|&s| s == default)
                .or_else(|| servers.first().copied())
                .unwrap_or(Server::EN))
        }
    }
}

pub async fn list(ctx: &AppContext, server: Option<Server>) -> Result<AnnouncementList, AppError> {
    let server = board_server(ctx, server)?;
    let board = ctx.cache.get_json::<Board>(&board_key(server)).await;
    Ok(AnnouncementList {
        server: server.as_str().to_owned(),
        servers: servers(ctx).iter().map(|s| s.as_str().to_owned()).collect(),
        checked_at: board.as_ref().map(|b| b.checked_at),
        announcements: board.map(|b| b.announcements).unwrap_or_default(),
    })
}

pub async fn detail(
    ctx: &AppContext,
    server: Option<Server>,
    id: &str,
) -> Result<CachedJson, AppError> {
    let server = board_server(ctx, server)?;
    let on_board = ctx
        .cache
        .get_json::<Board>(&board_key(server))
        .await
        .is_some_and(|b| b.announcements.iter().any(|a| a.id == id));
    if !on_board {
        return Err(AppError::NotFound(None));
    }
    ctx.cache
        .get_json(&detail_key(server, id))
        .await
        .ok_or(AppError::NotFound(None))
}

pub async fn get_text(ctx: &AppContext, url: &str) -> Result<String, AppError> {
    let response = ctx
        .upstream
        .client()
        .get(url)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(myrtle_hypergryph::UpstreamError::from)?;
    Ok(response
        .text()
        .await
        .map_err(myrtle_hypergryph::UpstreamError::from)?)
}

#[must_use]
pub fn announce_url(ctx: &AppContext, server: Server) -> Option<String> {
    let config = ctx.upstream.config();
    let template = match config.domain(server, Domain::AN) {
        Some(t) => t,
        None if server == Server::EN => DEFAULT_ANNOUNCE_URL,
        None => return None,
    };
    Some(template.replace("{0}", ANNOUNCE_PLATFORM))
}

pub async fn read_board(
    ctx: &AppContext,
    server: Server,
    previous: Option<&Board>,
) -> Result<(String, Vec<Listed>), AppError> {
    let meta_url = announce_url(ctx, server)
        .ok_or_else(|| AppError::internal(format!("no `an` route for {}", server.as_str())))?;
    let listed = parse_meta(&get_text(ctx, &meta_url).await?).map_err(AppError::internal)?;
    let mut board = Vec::with_capacity(listed.len());
    for item in listed {
        let known = previous.and_then(|b| {
            b.announcements
                .iter()
                .find(|a| a.notice_url == item.web_url)
        });
        let detail = detail_key(server, &item.announce_id);
        if let Some(known) = known
            && ctx.cache.get(&detail).await.is_some()
        {
            let banner_url = known.banner_url.clone();
            board.push(Listed {
                item,
                banner_url,
                html: None,
            });
            continue;
        }
        match get_text(ctx, &item.web_url).await {
            Ok(html) => board.push(Listed {
                banner_url: banner_url(&html, &item.web_url),
                html: Some(html),
                item,
            }),
            Err(e) => tracing::warn!(
                server = server.as_str(),
                id = item.announce_id,
                error = %e,
                "announcements: notice fetch failed"
            ),
        }
    }
    Ok((meta_url, board))
}

pub async fn publish(ctx: &AppContext, server: Server, listed: &[Listed], now: i64, ttl: Duration) {
    let today = chrono::DateTime::from_timestamp(now, 0)
        .unwrap_or_default()
        .date_naive();
    let pages: Vec<(MetaItem, String)> = listed
        .iter()
        .filter_map(|l| Some((l.item.clone(), l.html.clone()?)))
        .collect();
    let built = ctx
        .cpu
        .run("announcements", move || {
            pages
                .iter()
                .map(|(item, html)| {
                    let parts = page::extract(html, &item.web_url);
                    announcement(server, item, &parts, today)
                })
                .collect::<Vec<_>>()
        })
        .await
        .inspect_err(|e| tracing::warn!(server = server.as_str(), error = %e, "announcements: pages not cut"))
        .unwrap_or_default();
    for notice in built {
        if let Ok(body) = serde_json::to_string(&notice) {
            let key = detail_key(server, &notice.summary.id);
            ctx.cache.set_json(&key, &CachedJson::new(body), ttl).await;
        }
    }
    for l in listed.iter().filter(|l| l.html.is_none()) {
        let key = detail_key(server, &l.item.announce_id);
        if let Some(kept) = ctx.cache.get(&key).await {
            ctx.cache.set(&key, &kept, ttl).await;
        }
    }
    let mut announcements: Vec<AnnouncementSummary> = listed
        .iter()
        .map(|l| summary(&l.item, l.banner_url.clone(), today))
        .collect();
    sort_newest_first(&mut announcements);
    let board = Board {
        announcements,
        checked_at: now,
    };
    ctx.cache.set_json(&board_key(server), &board, ttl).await;
}
