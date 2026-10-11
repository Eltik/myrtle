//! The next EN game maintenance, read from Yostar's own announcements.
//!
//! # Sources
//!
//! - `announcement.meta.json`, at the `an` route of the EN `network_config`
//!   (`https://ark-us-static-online.yo-star.com/announce/{0}/announcement.meta.json`,
//!   `{0}` = `Android`). The list the in-game notice board shows: id, title,
//!   `webUrl`. Public: answers 200 to a bare GET with no game headers.
//! - Each notice's `webUrl`, an HTML page. Public, same evidence. The window
//!   is read out of its prose by [`parse`].
//! - The same board on every other Yostar server (JP, KR), read in the same
//!   tick and handed to [`crate::core::announcements`] for the
//!   `/announcements` routes. Only the EN board feeds the window: the prose
//!   grammar in [`parse`] is English, and only `SYSTEM` notices are parsed
//!   (see [`MAINTENANCE_GROUP`]).
//! - `preannouncement.meta.json` beside it, the notice the login screen shows.
//!   Public. `actived: false` on 2026-10-11, outside any window; whether it
//!   flips to `true` during a maintenance has not been observed yet.
//! - The EN service account (see [`crate::core::service_account`]). The only
//!   authenticated call: once a planned window has started and its end has
//!   passed (or it has none), one `account/login` per poll says whether the
//!   game is still down, which is how an overrun stays on screen instead of
//!   closing at the planned end.
//!
//! None of the announcement data needs the stored account; the login probe is
//! the one thing it is used for.
//!
//! `MAINTENANCE_POLL_SECS=0` turns the poller off, and the endpoint then
//! reports no window, as before this module existed; the announcement boards
//! then stay empty too.

pub mod parse;

use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    app::state::AppState,
    core::hypergryph::{
        config::config,
        constants::{Domain, Server},
        fetch::FetchError,
    },
};

use parse::{NoticeWindow, parse_notice};

use crate::core::announcements::{
    self, MetaItem,
    page::{self, PageParts},
    parse_meta,
};

const DEFAULT_ANNOUNCE_URL: &str =
    "https://ark-us-static-online.yo-star.com/announce/{0}/announcement.meta.json";
const ANNOUNCE_PLATFORM: &str = "Android";
const DEFAULT_POLL_SECS: u64 = 5 * 60;
/// While a window is running or about to, poll this often instead.
const ACTIVE_POLL_SECS: u64 = 60;
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// How long past its planned end a window can stay up on an overrun signal.
const OVERRUN_LIMIT_SECS: i64 = 12 * 60 * 60;
/// A window with no stated end closes this long after its start unless the
/// game is still answering as down.
const OPEN_ENDED_LIMIT_SECS: i64 = 24 * 60 * 60;
/// Notices further out than this are not shown yet.
const UPCOMING_HORIZON_SECS: i64 = 14 * 24 * 60 * 60;
/// Two notices whose windows start this close are one window restated.
const SAME_WINDOW_SLACK_SECS: i64 = 60 * 60;

/// Where a maintenance window stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MaintenancePhase {
    /// Announced, not started.
    Upcoming,
    /// Inside the announced window.
    Ongoing,
    /// Past the planned end and still down, or moved later by a newer notice.
    Extended,
}

/// One maintenance window, as Yostar's notices describe it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MaintenanceWindow {
    /// The game server this window is for. Only `en` is read today.
    pub server: String,
    pub phase: MaintenancePhase,
    /// Unix seconds.
    #[ts(type = "number")]
    pub starts_at: i64,
    /// Unix seconds; the latest end any notice gives. `null` when no notice
    /// names one. In the `extended` phase this can be in the past: the game is
    /// still down after it.
    #[ts(type = "number | null")]
    pub ends_at: Option<i64>,
    /// The official notice, the newest one that describes this window.
    pub notice_url: String,
    /// The notice's own headline.
    pub title: String,
}

/// `GET /maintenance`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct MaintenanceStatus {
    /// The nearest window that is upcoming, running or overrunning; `null`
    /// when there is none.
    pub window: Option<MaintenanceWindow>,
    /// Unix seconds of the last successful announcement read; `null` before
    /// the first one.
    #[ts(type = "number | null")]
    pub checked_at: Option<i64>,
}

/// A parsed notice and where it came from.
#[derive(Debug, Clone)]
pub struct Notice {
    pub id: u64,
    pub url: String,
    pub window: NoticeWindow,
}

/// What the poller last saw. Read on every request, written once per poll.
#[derive(Debug, Default, Clone)]
struct Snapshot {
    notices: Vec<Notice>,
    prean_active: bool,
    /// The last service-account probe: `(down, at)`.
    probe: Option<(bool, i64)>,
    checked_at: Option<i64>,
}

static SNAPSHOT: LazyLock<RwLock<Snapshot>> = LazyLock::new(RwLock::default);

/// Signals that the game is still down, past what the notices say.
#[derive(Debug, Default, Clone, Copy)]
pub struct Signals {
    /// The login-screen notice is live.
    pub prean_active: bool,
    /// The service account's last login: `Some(true)` refused, `Some(false)`
    /// answered, `None` not asked or not conclusive.
    pub game_down: Option<bool>,
}

impl Signals {
    const fn still_down(self) -> bool {
        self.prean_active || matches!(self.game_down, Some(true))
    }
}

/// One window after merging every notice that restates or extends it.
#[derive(Debug, Clone)]
struct Merged {
    starts_at: i64,
    ends_at: Option<i64>,
    extended: bool,
    url: String,
    title: String,
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
        match same {
            Some(m) => {
                // A newer notice for the same window wins on its end; one that
                // pushes the end later is an extension even if it doesn't say so.
                let later = match (m.ends_at, w.ends_at) {
                    (Some(old), Some(new)) => new > old,
                    _ => false,
                };
                m.extended |= w.extended || later;
                if w.ends_at.is_some() {
                    m.ends_at = w.ends_at;
                }
                m.starts_at = m.starts_at.min(w.starts_at);
                m.url.clone_from(&n.url);
                m.title.clone_from(&w.title);
            }
            None => merged.push(Merged {
                starts_at: w.starts_at,
                ends_at: w.ends_at,
                extended: w.extended,
                url: n.url.clone(),
                title: w.title.clone(),
            }),
        }
    }
    merged.sort_by_key(|m| m.starts_at);
    merged
}

/// The window to show at `now`, if any. Pure: everything it reads is passed in.
pub fn resolve(notices: &[Notice], now: i64, signals: Signals) -> Option<MaintenanceWindow> {
    merge(notices).into_iter().find_map(|m| {
        let phase = if now < m.starts_at {
            (m.starts_at - now <= UPCOMING_HORIZON_SECS).then_some(MaintenancePhase::Upcoming)?
        } else {
            match m.ends_at {
                Some(end) if now < end => {
                    if m.extended {
                        MaintenancePhase::Extended
                    } else {
                        MaintenancePhase::Ongoing
                    }
                }
                Some(end) => (signals.still_down() && now - end < OVERRUN_LIMIT_SECS)
                    .then_some(MaintenancePhase::Extended)?,
                None => {
                    if signals.game_down == Some(false) && !signals.prean_active {
                        return None;
                    }
                    if now - m.starts_at >= OPEN_ENDED_LIMIT_SECS && !signals.still_down() {
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

/// Whether the login probe is worth asking: a window has started and either
/// has no end or its end has passed recently.
fn wants_probe(notices: &[Notice], now: i64) -> bool {
    merge(notices).iter().any(|m| {
        now >= m.starts_at
            && match m.ends_at {
                Some(end) => now >= end && now - end < OVERRUN_LIMIT_SECS,
                None => now - m.starts_at < OPEN_ENDED_LIMIT_SECS * 2,
            }
    })
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

fn poll_interval() -> u64 {
    std::env::var("MAINTENANCE_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_POLL_SECS)
}

/// The probe result is trusted for two default poll intervals.
fn signals(snapshot: &Snapshot, now: i64) -> Signals {
    let fresh = i64::try_from(DEFAULT_POLL_SECS * 2).unwrap_or(i64::MAX);
    Signals {
        prean_active: snapshot.prean_active,
        game_down: snapshot
            .probe
            .filter(|(_, at)| now - at <= fresh)
            .map(|(down, _)| down),
    }
}

/// What `GET /maintenance` answers.
pub fn current() -> MaintenanceStatus {
    let now = now_secs();
    let snapshot = SNAPSHOT.read().map(|s| s.clone()).unwrap_or_default();
    MaintenanceStatus {
        window: resolve(&snapshot.notices, now, signals(&snapshot, now)),
        checked_at: snapshot.checked_at,
    }
}

/// The running or overrunning window that should stop a sync on `server`.
pub fn blocking_window(server: Server) -> Option<MaintenanceWindow> {
    if server != Server::EN {
        return None;
    }
    current()
        .window
        .filter(|w| w.phase != MaintenancePhase::Upcoming)
}

/// Whether a game-server error reads as maintenance. The EN maintenance
/// answer has not been captured, so this matches the words rather than a
/// code: "maintenance" in any field, or the CN/JP words for it.
pub fn is_maintenance_error(err: &FetchError) -> bool {
    let FetchError::Upstream(e) = err else {
        return false;
    };
    [&e.error, &e.message, &e.msg, &e.info].iter().any(|s| {
        let lower = s.to_lowercase();
        lower.contains("maintenance") || lower.contains("维护") || lower.contains("メンテナンス")
    })
}

#[derive(Deserialize)]
struct PreAnnounceMeta {
    #[serde(default)]
    actived: bool,
}

/// The board group maintenance notices are filed under. Every notice with a
/// maintenance headline on the four live boards read on 2026-10-11 is in it:
/// JP 1424 `メンテナンスのお知らせ`, CN 2069 `闪断更新公告`, TW 934 `閃斷更新`;
/// none of the 34 `ACTIVITY` notices across EN/JP/KR/CN/TW is one. The rest
/// of the group is statements (EN 1107, 1090), which the prose filter in
/// [`parse`] already rejects. A notice with no group is still read.
const MAINTENANCE_GROUP: &str = "SYSTEM";

fn may_be_maintenance(item: &MetaItem) -> bool {
    item.group.is_empty() || item.group == MAINTENANCE_GROUP
}

/// The pre-announcement meta's file name. It sits beside the board's meta.
/// Not the `prean` route of `network_config`: that names a webview host
/// (`https://ak-webview.arknights.global`, a 302 to `/gameBulletin`), where
/// `/preannouncement.meta.json`, `/Android/preannouncement.meta.json` and
/// `/announce/preannouncement.meta.json` all answer 404 (2026-10-11); the
/// file beside the `an` meta answers 200.
const PREAN_META: &str = "preannouncement.meta.json";

async fn get_text(client: &reqwest::Client, url: &str) -> anyhow::Result<String> {
    let res = client
        .get(url)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await?
        .error_for_status()?;
    Ok(res.text().await?)
}

/// `server`'s board meta, from its `network_config` `an` route. EN falls back
/// to the route last seen there when the config has not loaded; the others
/// are skipped until it has.
async fn announce_url(server: Server) -> Option<String> {
    let routed = config()
        .read()
        .await
        .domain(server, Domain::AN)
        .map(str::to_owned);
    let template = match routed {
        Some(t) => t,
        None if server == Server::EN => DEFAULT_ANNOUNCE_URL.to_owned(),
        None => return None,
    };
    Some(template.replace("{0}", ANNOUNCE_PLATFORM))
}

fn prean_url(meta_url: &str) -> Option<String> {
    reqwest::Url::parse(meta_url)
        .ok()?
        .join(PREAN_META)
        .ok()
        .map(String::from)
}

/// What one notice page yielded: its parts, and on the EN board its window.
#[derive(Debug, Clone)]
struct Page {
    parts: PageParts,
    window: Option<NoticeWindow>,
}

/// Pages by `webUrl`. The URL carries a `?t=` board stamp, so a rebuilt board
/// is a new key and is read again.
type PageCache = HashMap<String, Page>;

/// Read `server`'s board: the meta every tick, a page only when its URL is new.
async fn read_board(
    client: &reqwest::Client,
    server: Server,
    pages: &mut PageCache,
    now: i64,
) -> anyhow::Result<(String, Vec<(MetaItem, Page)>)> {
    let meta_url = announce_url(server)
        .await
        .ok_or_else(|| anyhow::anyhow!("no `an` route for {}", server.as_str()))?;
    let listed = parse_meta(&get_text(client, &meta_url).await?)?;
    pages.retain(|url, _| listed.iter().any(|i| &i.web_url == url));
    let mut board = Vec::with_capacity(listed.len());
    for item in listed {
        if !pages.contains_key(&item.web_url) {
            match get_text(client, &item.web_url).await {
                Ok(html) => {
                    // The window grammar is English: only the EN board has one.
                    let window = (server == Server::EN && may_be_maintenance(&item))
                        .then(|| parse_notice(&html, now))
                        .flatten();
                    let parts = page::extract(&html, &item.web_url);
                    pages.insert(item.web_url.clone(), Page { parts, window });
                }
                Err(e) => {
                    tracing::warn!(
                        server = server.as_str(),
                        id = item.announce_id,
                        error = %e,
                        "announcements: notice fetch failed"
                    );
                    continue;
                }
            }
        }
        if let Some(page) = pages.get(&item.web_url) {
            board.push((item, page.clone()));
        }
    }
    Ok((meta_url, board))
}

/// Store a board for the `/announcements` routes. Brotli runs off the workers.
async fn publish_board(server: Server, board: &[(MetaItem, Page)], now: i64) {
    let today = chrono::DateTime::from_timestamp(now, 0)
        .unwrap_or_default()
        .date_naive();
    let mut notices: Vec<_> = board
        .iter()
        .map(|(item, page)| announcements::build(server, item, &page.parts, today))
        .collect();
    announcements::sort_newest_first(&mut notices);
    if let Err(e) =
        tokio::task::spawn_blocking(move || announcements::publish(server, notices, now)).await
    {
        tracing::error!(error = %e, "announcements: publish panicked");
    }
}

/// Whether the login-screen notice beside `meta_url` is live. A failed read
/// counts as not live.
async fn prean_active(client: &reqwest::Client, meta_url: &str) -> bool {
    let Some(url) = prean_url(meta_url) else {
        return false;
    };
    match get_text(client, &url).await {
        Ok(raw) => serde_json::from_str::<PreAnnounceMeta>(&raw).is_ok_and(|p| p.actived),
        Err(e) => {
            tracing::debug!(error = %e, "maintenance: pre-announcement fetch failed");
            false
        }
    }
}

/// One EN service-account login: `Some((down, at))`, or `None` when there is
/// no account or its answer says nothing about the game.
async fn probe_game(state: &AppState) -> Option<(bool, i64)> {
    let account = state.service_accounts.get(Server::EN)?;
    match account.probe(&state.http_client).await {
        Ok(()) => Some((false, now_secs())),
        // A dead durable token says nothing about the game.
        Err(FetchError::NotLoggedIn) => None,
        Err(e) => {
            tracing::info!(error = ?e, "maintenance: EN login refused, still down");
            Some((true, now_secs()))
        }
    }
}

async fn poll(state: &AppState, caches: &mut HashMap<Server, PageCache>) -> anyhow::Result<()> {
    let client = &state.http_client;
    let now = now_secs();

    // The other boards first: a failure there is logged and never stops the
    // EN read the maintenance window depends on.
    for server in announcements::servers().filter(|s| *s != Server::EN) {
        let pages = caches.entry(server).or_default();
        match read_board(client, server, pages, now).await {
            Ok((_, board)) => publish_board(server, &board, now).await,
            Err(e) => {
                tracing::warn!(server = server.as_str(), error = %e, "announcements: board read failed");
            }
        }
    }

    let pages = caches.entry(Server::EN).or_default();
    let (meta_url, board) = read_board(client, Server::EN, pages, now).await?;
    publish_board(Server::EN, &board, now).await;
    let notices: Vec<Notice> = board
        .into_iter()
        .filter_map(|(item, page)| {
            Some(Notice {
                id: item.announce_id.parse().ok()?,
                url: item.web_url,
                window: page.window?,
            })
        })
        .collect();

    let prean_active = prean_active(client, &meta_url).await;
    let probe = if wants_probe(&notices, now) {
        probe_game(state).await
    } else {
        None
    };

    let mut snapshot = SNAPSHOT.write().map_err(|_| anyhow::anyhow!("poisoned"))?;
    let changed = snapshot.notices.len() != notices.len() || snapshot.prean_active != prean_active;
    snapshot.notices = notices;
    snapshot.prean_active = prean_active;
    if probe.is_some() {
        snapshot.probe = probe;
    }
    snapshot.checked_at = Some(now);
    if changed {
        tracing::info!(
            notices = snapshot.notices.len(),
            prean_active,
            "maintenance notices updated"
        );
    }
    Ok(())
}

/// Start the poller. Not staggered: it costs one small GET per tick plus one
/// per new notice, and the top bar wants an answer soon after boot.
pub fn spawn(state: AppState) {
    let interval = poll_interval();
    if interval == 0 {
        tracing::info!("maintenance poller disabled (MAINTENANCE_POLL_SECS=0)");
        return;
    }
    tokio::spawn(async move {
        let mut caches = HashMap::new();
        loop {
            if let Err(e) = poll(&state, &mut caches).await {
                tracing::warn!(error = %e, "maintenance poll failed");
            }
            let busy = current().window.is_some_and(|w| {
                w.phase != MaintenancePhase::Upcoming
                    || w.starts_at - now_secs() <= i64::try_from(DEFAULT_POLL_SECS).unwrap_or(0)
            });
            let secs = if busy {
                interval.min(ACTIVE_POLL_SECS)
            } else {
                interval
            };
            tokio::time::sleep(Duration::from_secs(secs)).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn notice(id: u64, start: i64, end: Option<i64>, extended: bool) -> Notice {
        Notice {
            id,
            url: format!("https://example.invalid/{id}.html"),
            window: NoticeWindow {
                starts_at: start,
                ends_at: end,
                extended,
                title: format!("notice {id}"),
            },
        }
    }

    const H: i64 = 3600;
    const T0: i64 = 1_800_000_000;

    #[test]
    fn real_meta_lists_every_notice() {
        let listed = parse_meta(include_str!("fixtures/meta_android.json")).expect("parses");
        let ids: Vec<&str> = listed.iter().map(|i| i.announce_id.as_str()).collect();
        assert_eq!(
            ids,
            ["1608", "1629", "1627", "1599", "1624", "1107", "1090"]
        );
        assert!(
            listed[0]
                .web_url
                .ends_with("/announcement/1608.html?t=1791543600")
        );
        // The group filter keeps the two statements and drops five events.
        let kept: Vec<&str> = listed
            .iter()
            .filter(|i| may_be_maintenance(i))
            .map(|i| i.announce_id.as_str())
            .collect();
        assert_eq!(kept, ["1107", "1090"]);
    }

    #[test]
    fn prean_meta_sits_beside_the_board() {
        assert_eq!(
            prean_url(
                "https://ark-us-static-online.yo-star.com/announce/Android/announcement.meta.json"
            )
            .as_deref(),
            Some(
                "https://ark-us-static-online.yo-star.com/announce/Android/preannouncement.meta.json"
            )
        );
    }

    #[test]
    fn real_preannouncement_is_inactive() {
        let p: PreAnnounceMeta =
            serde_json::from_str(include_str!("fixtures/prean.json")).expect("parses");
        assert!(!p.actived);
    }

    #[test]
    fn phases_across_a_window() {
        let n = [notice(1, T0, Some(T0 + 4 * H), false)];
        let none = Signals::default();
        let at = |t| resolve(&n, t, none).map(|w| w.phase);
        assert_eq!(at(T0 - H), Some(MaintenancePhase::Upcoming));
        assert_eq!(at(T0), Some(MaintenancePhase::Ongoing));
        assert_eq!(
            at(T0 + 4 * H),
            None,
            "closes at the planned end with no overrun signal"
        );
        assert_eq!(at(T0 - 15 * 24 * H), None, "beyond the horizon");
    }

    #[test]
    fn overrun_stays_up_while_the_game_is_down() {
        let n = [notice(1, T0, Some(T0 + H), false)];
        let down = Signals {
            prean_active: false,
            game_down: Some(true),
        };
        let w = resolve(&n, T0 + 2 * H, down).expect("still shown");
        assert_eq!(w.phase, MaintenancePhase::Extended);
        assert_eq!(w.ends_at, Some(T0 + H));
        let prean = Signals {
            prean_active: true,
            game_down: None,
        };
        assert!(resolve(&n, T0 + 2 * H, prean).is_some());
        assert!(
            resolve(&n, T0 + H + OVERRUN_LIMIT_SECS, down).is_none(),
            "gives up eventually"
        );
        let up = Signals {
            prean_active: false,
            game_down: Some(false),
        };
        assert!(resolve(&n, T0 + 2 * H, up).is_none());
    }

    #[test]
    fn newer_notice_extends_the_same_window() {
        let n = [
            notice(1626, T0, Some(T0 + H), false),
            notice(1630, T0, Some(T0 + 3 * H), false),
        ];
        let w = resolve(&n, T0 + 2 * H, Signals::default()).expect("shown");
        assert_eq!(w.phase, MaintenancePhase::Extended);
        assert_eq!(w.ends_at, Some(T0 + 3 * H));
        assert!(w.notice_url.ends_with("1630.html"));
        // Before the window, the restated end is what counts down.
        let early = resolve(&n, T0 - H, Signals::default()).expect("shown");
        assert_eq!(early.phase, MaintenancePhase::Upcoming);
        assert_eq!(early.ends_at, Some(T0 + 3 * H));
    }

    #[test]
    fn nearest_of_two_separate_windows() {
        let n = [
            notice(2, T0 + 7 * 24 * H, Some(T0 + 7 * 24 * H + H), false),
            notice(1, T0, Some(T0 + H), false),
        ];
        let w = resolve(&n, T0 - H, Signals::default()).expect("shown");
        assert_eq!(w.starts_at, T0);
        let later = resolve(&n, T0 + 2 * H, Signals::default()).expect("next one");
        assert_eq!(later.starts_at, T0 + 7 * 24 * H);
    }

    #[test]
    fn open_ended_window_closes_when_the_game_answers() {
        let n = [notice(1, T0, None, false)];
        assert_eq!(
            resolve(&n, T0 + H, Signals::default()).map(|w| w.phase),
            Some(MaintenancePhase::Ongoing)
        );
        let up = Signals {
            prean_active: false,
            game_down: Some(false),
        };
        assert!(resolve(&n, T0 + H, up).is_none());
        assert!(resolve(&n, T0 + OPEN_ENDED_LIMIT_SECS, Signals::default()).is_none());
    }

    #[test]
    fn probe_only_after_the_planned_end() {
        let n = [notice(1, T0, Some(T0 + H), false)];
        assert!(!wants_probe(&n, T0 - H));
        assert!(!wants_probe(&n, T0 + H / 2));
        assert!(wants_probe(&n, T0 + H));
        assert!(!wants_probe(&n, T0 + H + OVERRUN_LIMIT_SECS));
    }

    #[test]
    fn maintenance_error_words() {
        let err = |msg: &str| {
            FetchError::Upstream(crate::core::hypergryph::fetch::UpstreamError {
                status_code: 503,
                code: 0,
                error: String::new(),
                message: String::new(),
                msg: msg.to_owned(),
                info: String::new(),
            })
        };
        assert!(is_maintenance_error(&err("Server under Maintenance")));
        assert!(is_maintenance_error(&err("服务器维护中")));
        assert!(!is_maintenance_error(&err("invalid shop id")));
        assert!(!is_maintenance_error(&FetchError::NotLoggedIn));
    }
}
