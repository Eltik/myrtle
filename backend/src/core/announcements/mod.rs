//! The in-game notice board, per Yostar server, as the site serves it.
//!
//! Filled by the maintenance poller ([`crate::core::maintenance::spawn`]),
//! which already reads the board: one fetch of each server's
//! `announcement.meta.json` per tick and one per new notice page. This module
//! holds the types, the meta and page parsing, and the in-memory store the
//! `/announcements` routes answer from. Nothing here calls Yostar itself.
//!
//! # Servers
//!
//! Every server with a Yostar SDK domain ([`Server::yostar_domain`]) whose
//! `network_config` names an `an` route: EN, JP and KR on 2026-10-11, with 7,
//! 6 and 5 notices listed.
//!
//! Ruled out by measurement on 2026-10-11: CN and TW. Both `network_config`s
//! name an `an` route and the meta parses the same, but the boards are
//! abandoned: CN's newest notice is 2069, dated 05-08 and stamped
//! `2069_1746619406` (2025-05-07), 22 notices; TW's newest is 934, stamped
//! `t=1757990940` (2025-09-16), 8 notices. Those games publish elsewhere now.
//!
//! # Dates
//!
//! The meta gives each notice a `day` and `month` and no year, and the page
//! carries no date at all; the `?t=` on every `webUrl` is one stamp for the
//! whole board (EN `1791543600` on all 7), not a publish time. The year is
//! the latest one that does not put the date in the future, see
//! [`published_on`].

pub mod page;

use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{app::cache::SharedJson, core::hypergryph::constants::Server};

use page::PageParts;

/// One notice in the board list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AnnouncementSummary {
    /// Yostar's id, unique within one server.
    pub id: String,
    /// The board title, its line break turned into a space.
    pub title: String,
    /// The board's own category, as the meta names it (`ACTIVITY`, `SYSTEM`
    /// on 2026-10-11).
    pub group: String,
    /// `YYYY-MM-DD`; the year is inferred, see the module docs. `null` when
    /// the meta's day and month make no date.
    pub published_on: Option<String>,
    /// Absolute URL of the notice's banner image, if it has one.
    pub banner_url: Option<String>,
    /// The official notice page.
    pub notice_url: String,
}

/// `GET /announcements/{id}`: one notice with its sanitized body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct Announcement {
    #[serde(flatten)]
    #[ts(flatten)]
    pub summary: AnnouncementSummary,
    /// The game server this notice is from.
    pub server: String,
    /// The page's own headline, often longer than the board title. `null` on
    /// a banner-only notice.
    pub headline: Option<String>,
    /// The notice body after the allowlist sanitizer: text, emphasis, lists,
    /// tables, `hr`, images and `http(s)` links, every URL absolute. Coloured
    /// emphasis is `<span class="accent">`. Empty on a banner-only notice.
    pub body_html: String,
}

/// `GET /announcements`: the board, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct AnnouncementList {
    /// The server this board is from.
    pub server: String,
    /// Every server that has a board, for a picker.
    pub servers: Vec<String>,
    pub announcements: Vec<AnnouncementSummary>,
    /// Unix seconds of the last successful read of this board; `null` before
    /// the first one.
    #[ts(type = "number | null")]
    pub checked_at: Option<i64>,
}

/// The servers whose boards are read: Yostar's, see the module docs.
pub fn servers() -> impl Iterator<Item = Server> {
    Server::all()
        .iter()
        .copied()
        .filter(|s| s.yostar_domain().is_some())
}

/// One row of `announcement.meta.json`.
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

/// Every row that links a page, in board order.
pub fn parse_meta(raw: &str) -> Result<Vec<MetaItem>, serde_json::Error> {
    let meta: Meta = serde_json::from_str(raw)?;
    Ok(meta
        .announce_list
        .into_iter()
        .filter(|a| !a.web_url.is_empty() && !a.announce_id.is_empty())
        .collect())
}

/// How far ahead of `today` (UTC) a board date may sit before it is read as
/// last year's: the boards are stamped in the server's own zone, up to UTC+9
/// (JP/KR), so a notice posted on the 12th can be read on the 11th in UTC.
const FUTURE_SLACK_DAYS: i64 = 1;

/// The date a `day`/`month` pair names, seen on `today`: this year's, unless
/// that is in the future, then last year's.
pub fn published_on(day: u32, month: u32, today: NaiveDate) -> Option<NaiveDate> {
    let limit = today + chrono::Duration::days(FUTURE_SLACK_DAYS);
    [today.year(), today.year() - 1]
        .into_iter()
        .filter_map(|y| NaiveDate::from_ymd_opt(y, month, day))
        .find(|d| *d <= limit)
}

/// A notice as the store keeps it.
pub fn build(server: Server, item: &MetaItem, parts: &PageParts, today: NaiveDate) -> Announcement {
    Announcement {
        summary: AnnouncementSummary {
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
            banner_url: parts.banner_url.clone(),
            notice_url: item.web_url.clone(),
        },
        server: server.as_str().to_owned(),
        headline: parts.headline.clone(),
        body_html: parts.body_html.clone(),
    }
}

/// Newest first: by date, then by id (the higher id is the later notice on
/// every board read). An undated notice sorts last.
pub fn sort_newest_first(list: &mut [Announcement]) {
    list.sort_by(|a, b| {
        let key = |n: &Announcement| {
            (
                n.summary.published_on.clone(),
                n.summary.id.parse::<u64>().unwrap_or(0),
            )
        };
        key(b).cmp(&key(a))
    });
}

/// One server's board, pre-serialized.
struct Board {
    notices: Vec<Announcement>,
    checked_at: i64,
    list: SharedJson,
    details: HashMap<String, SharedJson>,
}

static BOARDS: LazyLock<RwLock<HashMap<Server, Board>>> = LazyLock::new(RwLock::default);

fn serialize<T: Serialize>(value: &T) -> SharedJson {
    // These types serialize infallibly: strings, numbers and options.
    SharedJson::new(serde_json::to_vec(value).unwrap_or_default())
}

fn list_body(
    server: Server,
    notices: &[Announcement],
    checked_at: Option<i64>,
) -> AnnouncementList {
    AnnouncementList {
        server: server.as_str().to_owned(),
        servers: servers().map(|s| s.as_str().to_owned()).collect(),
        announcements: notices.iter().map(|n| n.summary.clone()).collect(),
        checked_at,
    }
}

/// Store `notices` (already sorted) as `server`'s board. Re-encodes only
/// what changed; a poll that finds the same board just moves `checked_at`.
/// Brotli on a few kB per notice: run it off the async workers.
pub fn publish(server: Server, notices: Vec<Announcement>, checked_at: i64) {
    let Ok(mut boards) = BOARDS.write() else {
        return;
    };
    if let Some(board) = boards.get_mut(&server)
        && board.notices == notices
    {
        board.checked_at = checked_at;
        board.list = serialize(&list_body(server, &notices, Some(checked_at)));
        return;
    }
    let details = notices
        .iter()
        .map(|n| (n.summary.id.clone(), serialize(n)))
        .collect();
    let list = serialize(&list_body(server, &notices, Some(checked_at)));
    boards.insert(
        server,
        Board {
            notices,
            checked_at,
            list,
            details,
        },
    );
}

/// `server`'s board, or an empty one before the first read. `None` for a
/// server with no board.
pub fn list(server: Server) -> Option<SharedJson> {
    servers().any(|s| s == server).then(|| {
        BOARDS
            .read()
            .ok()
            .and_then(|b| b.get(&server).map(|board| board.list.clone()))
            .unwrap_or_else(|| serialize(&list_body(server, &[], None)))
    })
}

/// One notice from `server`'s board.
pub fn detail(server: Server, id: &str) -> Option<SharedJson> {
    BOARDS.read().ok()?.get(&server)?.details.get(id).cloned()
}

/// When `server`'s board was last read.
pub fn checked_at(server: Server) -> Option<i64> {
    BOARDS.read().ok()?.get(&server).map(|b| b.checked_at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("date")
    }

    #[test]
    fn yostar_servers_only() {
        let s: Vec<&str> = servers().map(|s| s.as_str()).collect();
        assert_eq!(s, ["en", "jp", "kr"]);
    }

    #[test]
    fn real_metas_parse_with_groups() {
        for (raw, n, ids) in [
            (
                include_str!("../maintenance/fixtures/meta_android.json"),
                7,
                "1608",
            ),
            (
                include_str!("../maintenance/fixtures/meta_jp.json"),
                6,
                "1428",
            ),
            (
                include_str!("../maintenance/fixtures/meta_kr.json"),
                5,
                "1391",
            ),
        ] {
            let items = parse_meta(raw).expect("parses");
            assert_eq!(items.len(), n);
            assert_eq!(items[0].announce_id, ids);
            assert!(items.iter().all(|i| !i.group.is_empty() && i.day > 0));
        }
        let en = parse_meta(include_str!("../maintenance/fixtures/meta_android.json")).expect("en");
        let groups: Vec<&str> = en.iter().map(|i| i.group.as_str()).collect();
        assert_eq!(
            groups,
            [
                "ACTIVITY", "ACTIVITY", "ACTIVITY", "ACTIVITY", "ACTIVITY", "SYSTEM", "SYSTEM"
            ]
        );
    }

    #[test]
    fn year_is_the_latest_not_in_the_future() {
        let today = day("2026-10-11");
        assert_eq!(published_on(9, 10, today), Some(day("2026-10-09")));
        assert_eq!(
            published_on(12, 10, today),
            Some(day("2026-10-12")),
            "a zone ahead"
        );
        assert_eq!(published_on(13, 10, today), Some(day("2025-10-13")));
        assert_eq!(published_on(31, 7, today), Some(day("2026-07-31")));
        assert_eq!(
            published_on(31, 12, day("2027-01-02")),
            Some(day("2026-12-31"))
        );
        assert_eq!(published_on(0, 0, today), None);
        assert_eq!(
            published_on(29, 2, day("2029-03-01")),
            Some(day("2028-02-29"))
        );
    }

    #[test]
    fn real_board_sorts_newest_first() {
        let today = day("2026-10-11");
        let items =
            parse_meta(include_str!("../maintenance/fixtures/meta_android.json")).expect("en");
        let parts = PageParts {
            headline: None,
            banner_url: None,
            body_html: String::new(),
        };
        let mut built: Vec<Announcement> = items
            .iter()
            .map(|i| build(Server::EN, i, &parts, today))
            .collect();
        sort_newest_first(&mut built);
        let ids: Vec<&str> = built.iter().map(|n| n.summary.id.as_str()).collect();
        assert_eq!(
            ids,
            ["1608", "1629", "1627", "1599", "1624", "1107", "1090"]
        );
        assert_eq!(built[0].summary.title, "Displayed Operators Rate Up !!!");
        assert_eq!(built[0].summary.published_on.as_deref(), Some("2026-10-09"));
        assert_eq!(built[6].summary.published_on.as_deref(), Some("2026-07-31"));
    }

    #[test]
    fn store_serves_list_and_detail() {
        let today = day("2026-10-11");
        let items = parse_meta(include_str!("../maintenance/fixtures/meta_kr.json")).expect("kr");
        let parts = PageParts {
            headline: Some("h".into()),
            banner_url: None,
            body_html: "<p>x</p>".into(),
        };
        let built: Vec<Announcement> = items
            .iter()
            .map(|i| build(Server::KR, i, &parts, today))
            .collect();
        publish(Server::KR, built, 42);
        let board: AnnouncementList =
            serde_json::from_slice(&list(Server::KR).expect("kr board").body).expect("json");
        assert_eq!(board.announcements.len(), 5);
        assert_eq!(board.servers, ["en", "jp", "kr"]);
        assert_eq!(board.checked_at, Some(42));
        let one: serde_json::Value =
            serde_json::from_slice(&detail(Server::KR, "1391").expect("detail").body)
                .expect("json");
        assert_eq!(one["id"], "1391");
        assert_eq!(one["body_html"], "<p>x</p>");
        assert!(detail(Server::KR, "nope").is_none());
        assert!(list(Server::CN).is_none());
        assert_eq!(checked_at(Server::KR), Some(42));
    }
}
