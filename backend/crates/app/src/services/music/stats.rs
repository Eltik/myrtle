use std::collections::BTreeMap;
use std::net::IpAddr;
use std::time::Duration;

use crate::cache::hmac_hex;
use chrono::{DateTime, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::music_stats::{self as repo, ListenerSort, StatKind};
use myrtle_types::UserId;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::library::check_track_id;
use crate::{AppContext, AppError, Subject};

pub const LISTEN_WINDOW: Duration = Duration::from_mins(30);
pub const DOWNLOAD_WINDOW: Duration = Duration::from_hours(24);
pub const TOP_TRACKS_DEFAULT: u32 = 50;
pub const TOP_TRACKS_MAX: u32 = 100;
pub const LISTENERS_DEFAULT: u32 = 20;
pub const LISTENERS_MAX: u32 = 100;

pub const STATS_KEY: &str = "music:stats";
const STATS_TTL: Duration = Duration::from_mins(1);
const RANKING_TTL: Duration = Duration::from_mins(5);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum MusicStatWindow {
    #[default]
    Week,
    Month,
    All,
}

impl MusicStatWindow {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
            Self::All => "all",
        }
    }

    fn since(self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let days = match self {
            Self::Week => 7,
            Self::Month => 30,
            Self::All => return None,
        };
        Some(now - chrono::Duration::days(days))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum MusicListenerSort {
    #[default]
    Listens,
    Time,
}

impl MusicListenerSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Listens => "listens",
            Self::Time => "time",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatInput {
    pub track_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatRecorded {
    pub counted: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicTrackCounts {
    #[ts(type = "number")]
    pub listens: i64,
    #[ts(type = "number")]
    pub downloads: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicStatsCatalog {
    pub tracks: BTreeMap<String, MusicTrackCounts>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicTopTrack {
    #[ts(type = "number")]
    pub rank: i64,
    pub track_id: String,
    #[ts(type = "number")]
    pub listens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicTopTracks {
    pub window: MusicStatWindow,
    pub entries: Vec<MusicTopTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicListenerEntry {
    #[ts(type = "number")]
    pub rank: i64,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub server: String,
    #[ts(type = "number")]
    pub listens: i64,
    pub listened_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct MusicListenerPage {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub track_id: Option<String>,
    pub window: MusicStatWindow,
    pub sort: MusicListenerSort,
    pub entries: Vec<MusicListenerEntry>,
    #[ts(type = "number")]
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatClient {
    User(UserId),
    Anonymous(String),
}

impl StatClient {
    #[must_use]
    pub fn key(&self) -> String {
        match self {
            Self::User(id) => format!("u:{}", id.0),
            Self::Anonymous(key) => key.clone(),
        }
    }

    #[must_use]
    pub fn of(ctx: &AppContext, subject: &Subject, addr: Option<IpAddr>) -> Option<Self> {
        match subject.user_id() {
            Some(user) => Some(Self::User(user)),
            None => addr.map(|ip| anonymous_client(&ctx.secrets.jwt, ip)),
        }
    }
}

const fn window_of(kind: StatKind) -> Duration {
    match kind {
        StatKind::Listen => LISTEN_WINDOW,
        StatKind::Download => DOWNLOAD_WINDOW,
    }
}

#[must_use]
pub fn normalize_addr(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                v4.to_string()
            } else {
                let s = v6.segments();
                format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
            }
        }
    }
}

#[must_use]
pub fn anonymous_client(secret: &str, ip: IpAddr) -> StatClient {
    let message = format!("music-stats:{}", normalize_addr(ip));
    StatClient::Anonymous(format!(
        "a:{}",
        hmac_hex(secret.as_bytes(), message.as_bytes())
    ))
}

pub async fn is_known_track(ctx: &AppContext, track_id: &str) -> Result<bool, AppError> {
    check_track_id(track_id)?;
    if let Some(cid) = track_id.strip_prefix("siren:") {
        let mut conn = ctx.db.conn().await?;
        return Ok(repo::siren_song_exists(&mut conn, cid).await?);
    }
    let Some(id) = track_id.strip_prefix("game:") else {
        return Ok(false);
    };
    for server in ctx.snapshots.loaded() {
        match super::game_track_ids(ctx, server).await {
            Ok(ids) if ids.contains(id) => return Ok(true),
            Ok(_) | Err(AppError::NotFound(_)) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

pub async fn record_on(
    conn: &mut Conn,
    kind: StatKind,
    track_id: &str,
    client: &StatClient,
    at: DateTime<Utc>,
) -> Result<bool, AppError> {
    let window = window_of(kind).as_secs_f64();
    let counted = repo::claim_window(conn, kind, &client.key(), track_id, at, window).await?;
    if counted {
        let user = match client {
            StatClient::User(id) => Some(*id),
            StatClient::Anonymous(_) => None,
        };
        repo::insert_event(conn, kind, track_id, user, at).await?;
    }
    Ok(counted)
}

pub async fn record(
    ctx: &AppContext,
    kind: StatKind,
    track_id: &str,
    client: Option<&StatClient>,
) -> Result<MusicStatRecorded, AppError> {
    if !is_known_track(ctx, track_id).await? {
        return Err(AppError::NotFound(None));
    }
    let Some(client) = client else {
        return Ok(MusicStatRecorded { counted: false });
    };
    let mut conn = ctx.db.conn().await?;
    let counted = record_on(&mut conn, kind, track_id, client, Utc::now()).await?;
    Ok(MusicStatRecorded { counted })
}

pub async fn sweep_dedupe(ctx: &AppContext) -> Result<String, AppError> {
    let before = Utc::now() - DOWNLOAD_WINDOW;
    let mut conn = ctx.db.conn().await?;
    let rows = repo::sweep_dedupe(&mut conn, before).await?;
    Ok(format!("{rows} music stat dedupe rows swept"))
}

async fn cached<T, F, Fut>(
    ctx: &AppContext,
    key: &str,
    ttl: Duration,
    build: F,
) -> Result<T, AppError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    if let Some(hit) = ctx.cache.get_json(key).await {
        return Ok(hit);
    }
    let out = build().await?;
    ctx.cache.set_json(key, &out, ttl).await;
    Ok(out)
}

#[must_use]
pub fn top_tracks_key(window: MusicStatWindow, limit: u32) -> String {
    format!("music:stats:top_tracks:{}:{limit}", window.as_str())
}

#[must_use]
pub fn listeners_key(
    window: MusicStatWindow,
    sort: MusicListenerSort,
    track: Option<&str>,
    limit: u32,
    offset: u32,
) -> String {
    format!(
        "music:stats:listeners:{}:{}:{}:{limit}:{offset}",
        track.unwrap_or("all"),
        window.as_str(),
        sort.as_str()
    )
}

pub async fn catalog(ctx: &AppContext) -> Result<MusicStatsCatalog, AppError> {
    cached(ctx, STATS_KEY, STATS_TTL, || async {
        let mut conn = ctx.db.conn().await?;
        Ok(MusicStatsCatalog {
            tracks: repo::all_counts(&mut conn)
                .await?
                .into_iter()
                .map(|r| {
                    let counts = MusicTrackCounts {
                        listens: r.listens,
                        downloads: r.downloads,
                    };
                    (r.track_id, counts)
                })
                .collect(),
        })
    })
    .await
}

pub async fn top_tracks(
    ctx: &AppContext,
    window: MusicStatWindow,
    limit: Option<u32>,
) -> Result<MusicTopTracks, AppError> {
    let limit = limit.unwrap_or(TOP_TRACKS_DEFAULT).clamp(1, TOP_TRACKS_MAX);
    cached(ctx, &top_tracks_key(window, limit), RANKING_TTL, || async {
        let mut conn = ctx.db.conn().await?;
        let rows = repo::top_tracks(&mut conn, window.since(Utc::now()), i64::from(limit)).await?;
        Ok(MusicTopTracks {
            window,
            entries: rows
                .into_iter()
                .map(|r| MusicTopTrack {
                    rank: r.rank,
                    track_id: r.track_id,
                    listens: r.listens,
                })
                .collect(),
        })
    })
    .await
}

pub async fn listeners(
    ctx: &AppContext,
    window: MusicStatWindow,
    sort: MusicListenerSort,
    track_id: Option<&str>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<MusicListenerPage, AppError> {
    if let Some(track) = track_id {
        check_track_id(track)?;
    }
    let limit = limit.unwrap_or(LISTENERS_DEFAULT).min(LISTENERS_MAX);
    let offset = offset.unwrap_or(0);
    let key = listeners_key(window, sort, track_id, limit, offset);
    cached(ctx, &key, RANKING_TTL, || async {
        let from = window.since(Utc::now()).unwrap_or(DateTime::UNIX_EPOCH);
        let db_sort = match sort {
            MusicListenerSort::Listens => ListenerSort::Listens,
            MusicListenerSort::Time => ListenerSort::Time,
        };
        let mut conn = ctx.db.conn().await?;
        let rows = repo::listeners(
            &mut conn,
            from,
            track_id,
            db_sort,
            i64::from(limit),
            i64::from(offset),
        )
        .await?;
        let total = repo::count_listeners(&mut conn, from, track_id).await?;
        Ok(MusicListenerPage {
            track_id: track_id.map(str::to_owned),
            window,
            sort,
            entries: rows
                .into_iter()
                .map(|r| MusicListenerEntry {
                    rank: r.rank,
                    uid: r.uid,
                    nickname: r.nickname,
                    nick_number: r.nick_number,
                    level: r.level,
                    avatar_id: r.avatar_id,
                    server: r.server,
                    listens: r.listens,
                    listened_seconds: r.listened_seconds,
                })
                .collect(),
            total,
        })
    })
    .await
}
