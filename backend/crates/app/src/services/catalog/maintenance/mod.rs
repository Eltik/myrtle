pub mod parse;

use std::time::Duration;

use myrtle_hypergryph::UpstreamError;
use myrtle_types::Server;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::services::catalog::announcements::{self, Board, MetaItem};
use crate::{AppContext, AppError};

pub use parse::{NoticeWindow, parse_notice};

const DEFAULT_POLL_SECS: u64 = 5 * 60;
const ACTIVE_POLL_SECS: u64 = 60;
pub const OVERRUN_LIMIT_SECS: i64 = 12 * 60 * 60;
pub const OPEN_ENDED_LIMIT_SECS: i64 = 24 * 60 * 60;
const UPCOMING_HORIZON_SECS: i64 = 14 * 24 * 60 * 60;
const SAME_WINDOW_SLACK_SECS: i64 = 60 * 60;
const PULLED_NOTICE_GRACE_SECS: i64 = 60 * 60;
const MAINTENANCE_GROUP: &str = "SYSTEM";
const PREAN_META: &str = "preannouncement.meta.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum MaintenancePhase {
    Upcoming,
    Ongoing,
    Extended,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MaintenanceWindow {
    pub server: String,
    pub phase: MaintenancePhase,
    #[ts(type = "number")]
    pub starts_at: i64,
    #[ts(type = "number | null")]
    pub ends_at: Option<i64>,
    pub notice_url: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct MaintenanceStatus {
    pub window: Option<MaintenanceWindow>,
    #[ts(type = "number | null")]
    pub checked_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub id: u64,
    pub url: String,
    pub window: NoticeWindow,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceState {
    pub notices: Vec<Notice>,
    pub prean_active: bool,
    pub probe: Option<(bool, i64)>,
    pub checked_at: Option<i64>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Signals {
    pub prean_active: bool,
    pub game_down: Option<bool>,
}

impl Signals {
    const fn still_down(self) -> bool {
        self.prean_active || matches!(self.game_down, Some(true))
    }
}

#[derive(Deserialize)]
struct PreAnnounceMeta {
    #[serde(default)]
    actived: bool,
}

struct Merged {
    starts_at: i64,
    ends_at: Option<i64>,
    extended: bool,
    url: String,
    title: String,
}

#[must_use]
pub fn state_key(server: Server) -> String {
    format!("maintenance:{}", server.as_str())
}

fn state_ttl(ctx: &AppContext) -> Duration {
    Duration::from_secs(ctx.config.jobs.maintenance.poll_secs.max(DEFAULT_POLL_SECS) * 4)
}

fn merge(notices: &[Notice]) -> Vec<Merged> {
    let mut sorted: Vec<&Notice> = notices.iter().collect();
    sorted.sort_by_key(|n| n.id);
    let mut merged: Vec<Merged> = Vec::new();
    for n in sorted {
        let w = &n.window;
        let same = merged.iter_mut().find(|m| {
            (w.starts_at - m.starts_at).abs() <= SAME_WINDOW_SLACK_SECS
                || (w.starts_at >= m.starts_at && m.ends_at.is_some_and(|e| w.starts_at <= e))
        });
        if let Some(m) = same {
            let later = matches!((m.ends_at, w.ends_at), (Some(old), Some(new)) if new > old);
            m.extended |= w.extended || later;
            if w.ends_at.is_some() {
                m.ends_at = w.ends_at;
            }
            m.starts_at = m.starts_at.min(w.starts_at);
            m.url.clone_from(&n.url);
            m.title.clone_from(&w.title);
        } else {
            merged.push(Merged {
                starts_at: w.starts_at,
                ends_at: w.ends_at,
                extended: w.extended,
                url: n.url.clone(),
                title: w.title.clone(),
            });
        }
    }
    merged.sort_by_key(|m| m.starts_at);
    merged
}

#[must_use]
pub fn resolve(notices: &[Notice], now: i64, signals: Signals) -> Option<MaintenanceWindow> {
    merge(notices).into_iter().find_map(|m| {
        let phase = if now < m.starts_at {
            (m.starts_at - now <= UPCOMING_HORIZON_SECS).then_some(MaintenancePhase::Upcoming)?
        } else {
            match m.ends_at {
                Some(end) if now < end && m.extended => MaintenancePhase::Extended,
                Some(end) if now < end => MaintenancePhase::Ongoing,
                Some(end) => (signals.still_down() && now - end < OVERRUN_LIMIT_SECS)
                    .then_some(MaintenancePhase::Extended)?,
                None => {
                    let answered = signals.game_down == Some(false) && !signals.prean_active;
                    let expired =
                        now - m.starts_at >= OPEN_ENDED_LIMIT_SECS && !signals.still_down();
                    if answered || expired {
                        return None;
                    }
                    MaintenancePhase::Ongoing
                }
            }
        };
        Some(MaintenanceWindow {
            server: Server::EN.as_str().to_owned(),
            phase,
            starts_at: m.starts_at,
            ends_at: m.ends_at,
            notice_url: m.url,
            title: m.title,
        })
    })
}

#[must_use]
pub fn wants_probe(notices: &[Notice], now: i64) -> bool {
    merge(notices).iter().any(|m| {
        now >= m.starts_at
            && match m.ends_at {
                Some(end) => now >= end && now - end < OVERRUN_LIMIT_SECS,
                None => now - m.starts_at < OPEN_ENDED_LIMIT_SECS * 2,
            }
    })
}

#[must_use]
pub fn carry_over(previous: &[Notice], mut fresh: Vec<Notice>, now: i64) -> Vec<Notice> {
    let kept: Vec<Notice> = previous
        .iter()
        .filter(|old| !fresh.iter().any(|n| n.id == old.id))
        .filter(|old| {
            let w = &old.window;
            now + PULLED_NOTICE_GRACE_SECS >= w.starts_at
                && match w.ends_at {
                    Some(end) => now - end < OVERRUN_LIMIT_SECS,
                    None => now - w.starts_at < OPEN_ENDED_LIMIT_SECS * 2,
                }
        })
        .cloned()
        .collect();
    fresh.extend(kept);
    fresh
}

#[must_use]
pub fn status_at(state: &MaintenanceState, now: i64) -> MaintenanceStatus {
    let fresh = i64::try_from(DEFAULT_POLL_SECS * 2).unwrap_or(i64::MAX);
    let signals = Signals {
        prean_active: state.prean_active,
        game_down: state
            .probe
            .filter(|(_, at)| now - at <= fresh)
            .map(|(down, _)| down),
    };
    MaintenanceStatus {
        window: resolve(&state.notices, now, signals),
        checked_at: state.checked_at,
    }
}

#[must_use]
pub fn may_be_maintenance(item: &MetaItem) -> bool {
    item.group.is_empty() || item.group == MAINTENANCE_GROUP
}

pub fn prean_url(meta_url: &str) -> Option<String> {
    Url::parse(meta_url)
        .ok()?
        .join(PREAN_META)
        .ok()
        .map(String::from)
}

#[must_use]
pub fn prean_is_active(raw: &str) -> bool {
    serde_json::from_str::<PreAnnounceMeta>(raw).is_ok_and(|p| p.actived)
}

pub async fn status(ctx: &AppContext) -> MaintenanceStatus {
    let state = ctx
        .cache
        .get_json::<MaintenanceState>(&state_key(Server::EN))
        .await
        .unwrap_or_default();
    status_at(&state, chrono::Utc::now().timestamp())
}

pub async fn blocking_window(ctx: &AppContext, server: Server) -> Option<MaintenanceWindow> {
    if server != Server::EN {
        return None;
    }
    status(ctx)
        .await
        .window
        .filter(|w| w.phase != MaintenancePhase::Upcoming)
}

async fn prean_active(ctx: &AppContext, meta_url: &str) -> bool {
    let Some(url) = prean_url(meta_url) else {
        return false;
    };
    match announcements::get_text(ctx, &url).await {
        Ok(raw) => prean_is_active(&raw),
        Err(e) => {
            tracing::debug!(error = %e, "maintenance: pre-announcement fetch failed");
            false
        }
    }
}

async fn probe_game(ctx: &AppContext) -> Option<(bool, i64)> {
    let account = ctx.accounts.get(Server::EN)?;
    match account.probe(&ctx.upstream).await {
        Ok(()) => Some((false, chrono::Utc::now().timestamp())),
        Err(UpstreamError::NotLoggedIn) => None,
        Err(e) => {
            tracing::info!(error = %e, "maintenance: EN login refused, still down");
            Some((true, chrono::Utc::now().timestamp()))
        }
    }
}

async fn refresh_board(
    ctx: &AppContext,
    server: Server,
    now: i64,
    ttl: Duration,
) -> Result<(String, Vec<announcements::Listed>), AppError> {
    let previous = ctx
        .cache
        .get_json::<Board>(&announcements::board_key(server))
        .await;
    let (meta_url, listed) = announcements::read_board(ctx, server, previous.as_ref()).await?;
    announcements::publish(ctx, server, &listed, now, ttl).await;
    Ok((meta_url, listed))
}

pub async fn poll(ctx: &AppContext) -> Result<Duration, AppError> {
    let interval = ctx.config.jobs.maintenance.poll_secs;
    if interval == 0 {
        return Ok(Duration::ZERO);
    }
    let now = chrono::Utc::now().timestamp();
    let ttl = state_ttl(ctx);

    for server in announcements::servers(ctx)
        .into_iter()
        .filter(|&s| s != Server::EN)
    {
        if let Err(e) = refresh_board(ctx, server, now, ttl).await {
            tracing::warn!(server = server.as_str(), error = %e, "announcements: board read failed");
        }
    }

    let key = state_key(Server::EN);
    let previous = ctx
        .cache
        .get_json::<MaintenanceState>(&key)
        .await
        .unwrap_or_default();
    let (meta_url, listed) = refresh_board(ctx, Server::EN, now, ttl).await?;
    let fresh = listed
        .into_iter()
        .filter(|l| may_be_maintenance(&l.item))
        .filter_map(|l| {
            let window = match &l.html {
                Some(html) => parse_notice(html, now)?,
                None => previous
                    .notices
                    .iter()
                    .find(|n| n.url == l.item.web_url)?
                    .window
                    .clone(),
            };
            Some(Notice {
                id: l.item.announce_id.parse().ok()?,
                url: l.item.web_url,
                window,
            })
        })
        .collect();
    let notices = carry_over(&previous.notices, fresh, now);

    let prean_active = prean_active(ctx, &meta_url).await;
    let probe = if wants_probe(&notices, now) {
        probe_game(ctx).await
    } else {
        None
    };
    if previous.notices.len() != notices.len() || previous.prean_active != prean_active {
        tracing::info!(
            notices = notices.len(),
            prean_active,
            "maintenance notices updated"
        );
    }
    let state = MaintenanceState {
        notices,
        prean_active,
        probe: probe.or(previous.probe),
        checked_at: Some(now),
    };
    ctx.cache.set_json(&key, &state, ttl).await;

    let busy = status_at(&state, now).window.is_some_and(|w| {
        w.phase != MaintenancePhase::Upcoming
            || w.starts_at - now <= i64::try_from(DEFAULT_POLL_SECS).unwrap_or(0)
    });
    let secs = if busy {
        interval.min(ACTIVE_POLL_SECS)
    } else {
        interval
    };
    Ok(Duration::from_secs(secs))
}
