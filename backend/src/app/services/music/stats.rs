//! Community music stats: listens and downloads counted for everyone, signed
//! in or not, and the rankings read from them.
//!
//! A call counts at most once per window for the same caller and track:
//! [`LISTEN_WINDOW`] for a listen, [`DOWNLOAD_WINDOW`] for a download. A
//! signed-in caller is keyed by account, so their two ways in (the
//! play-history log and the public listen endpoint) share one slot and a play
//! reported through both counts once. An anonymous caller is keyed by an HMAC
//! of their address; see [`anonymous_client`].

use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use axum::http::{Extensions, HeaderMap};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::Sha256;
use sqlx::PgConnection;
use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::middleware::peer_ip;
use crate::app::services::music::game_track_ids;
use crate::app::services::music::library::check_track_id;
use crate::app::state::AppState;
use crate::database::models::music_stats::{
    MusicListenerPage, MusicListenerSort, MusicStatKind, MusicStatWindow, MusicStatsCatalog,
    MusicTopTracks, MusicTrackCounts,
};
use crate::database::queries::music_stats as queries;

/// A listen counts again after this long. About ten tracks: a favourite
/// played again after a break counts again, while a track on repeat, a page
/// reload or a player that re-reports a play does not.
pub const LISTEN_WINDOW: Duration = Duration::from_mins(30);
/// A download counts again after this long. A file once saved does not need
/// saving again, so a day absorbs retries, a failed first attempt and the
/// same file saved on a second tab, while a returning visitor still counts.
pub const DOWNLOAD_WINDOW: Duration = Duration::from_hours(24);

/// Counted calls between sweeps of the dedupe table.
const SWEEP_EVERY: u64 = 1_000;
static COUNTED: AtomicU64 = AtomicU64::new(0);

/// Tracks the top-tracks ranking returns when the caller names no limit.
pub const TOP_TRACKS_DEFAULT: u32 = 50;
/// Tracks the top-tracks ranking returns at most.
pub const TOP_TRACKS_MAX: u32 = 100;

type HmacSha256 = Hmac<Sha256>;

const fn window_of(kind: MusicStatKind) -> Duration {
    match kind {
        MusicStatKind::Listen => LISTEN_WINDOW,
        MusicStatKind::Download => DOWNLOAD_WINDOW,
    }
}

/// Who a call is counted against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatClient {
    /// A signed-in account.
    User(Uuid),
    /// An anonymous caller: `a:` and a hex HMAC of their address.
    Anonymous(String),
}

impl StatClient {
    /// The `music_stat_dedupe.client` value.
    pub fn key(&self) -> String {
        match self {
            Self::User(id) => format!("u:{id}"),
            Self::Anonymous(key) => key.clone(),
        }
    }

    const fn user_id(&self) -> Option<Uuid> {
        match self {
            Self::User(id) => Some(*id),
            Self::Anonymous(_) => None,
        }
    }
}

/// The address an anonymous caller is keyed on: the rate limiter's own
/// reading ([`peer_ip`]), the socket peer unless `TRUSTED_PROXY_HOPS` names a
/// forwarded entry. No `X-Forwarded-For` is trusted beyond that: a proxy on
/// the same host connects from loopback too, so trusting loopback's header
/// let any client name its own address. The frontend's server never reports
/// a stat for a visitor anonymously, so nothing needs its header.
pub fn client_addr(headers: &HeaderMap, extensions: &Extensions) -> Option<IpAddr> {
    peer_ip(headers, extensions)
}

/// The address as keyed: IPv4 as is (an IPv4-mapped IPv6 address read as
/// its IPv4), IPv6 cut to its /64. One IPv6 host is handed a whole /64 and
/// can pick a fresh address in it per request, so keying the full address
/// would let one machine count as many listeners as it likes.
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

/// An anonymous caller's key: `a:` and HMAC-SHA256 of the normalized address
/// under `secret` (the JWT secret, as the tier-list view dedupe uses), so a
/// stored key cannot be reversed to an address by trying every IPv4 address
/// without the secret. Only the address: a user agent is the caller's to
/// choose, and keying on it would let one address count once per made-up
/// agent. The cost is that people behind one shared address count as one
/// listener per window.
pub fn anonymous_client(secret: &str, ip: IpAddr) -> StatClient {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(b"music-stats:");
    mac.update(normalize_addr(ip).as_bytes());
    StatClient::Anonymous(format!("a:{}", hex::encode(mac.finalize().into_bytes())))
}

/// Whether the catalogue lists the track: a `siren:` id the label's cached
/// catalogue holds, or a `game:` id in any loaded server's in-game index
/// (the default server's first; the others are built only on a miss).
///
/// # Errors
/// `400` for a malformed id; a database error.
pub async fn is_known_track(state: &AppState, track_id: &str) -> Result<bool, ApiError> {
    check_track_id(track_id)?;
    if let Some(cid) = track_id.strip_prefix("siren:") {
        let mut conn = state.db.acquire().await?;
        return Ok(queries::siren_song_exists(&mut conn, cid).await?);
    }
    let Some(id) = track_id.strip_prefix("game:") else {
        return Ok(false);
    };
    let mut servers = vec![state.default_server];
    servers.extend(
        state
            .pickable_servers()
            .into_iter()
            .filter(|s| *s != state.default_server),
    );
    for server in servers {
        match game_track_ids(state, server).await {
            Ok(ids) if ids.contains(id) => return Ok(true),
            Ok(_) | Err(ApiError::NotFound) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

/// Counts one event for `client` at `at` unless the client already counted
/// for the track inside the window. True when it counted. Runs on `conn`,
/// so a caller holding a transaction makes it part of that transaction.
/// Does not check the track id.
pub async fn record_on(
    conn: &mut PgConnection,
    kind: MusicStatKind,
    track_id: &str,
    client: &StatClient,
    at: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let window = window_of(kind).as_secs_f64();
    let counted = queries::claim_window(conn, kind, &client.key(), track_id, at, window).await?;
    if counted {
        queries::insert_event(conn, kind, track_id, client.user_id(), at).await?;
    }
    Ok(counted)
}

/// Sweeps the dedupe table every [`SWEEP_EVERY`] counted calls, off the
/// request: a row older than the longest window decides nothing.
pub fn note_counted(state: &AppState) {
    if !COUNTED
        .fetch_add(1, Ordering::Relaxed)
        .is_multiple_of(SWEEP_EVERY)
    {
        return;
    }
    let pool = state.db.clone();
    tokio::spawn(async move {
        let before = Utc::now() - DOWNLOAD_WINDOW;
        let swept = async {
            let mut conn = pool.acquire().await?;
            queries::sweep_dedupe(&mut conn, before).await
        }
        .await;
        match swept {
            Ok(rows) => tracing::debug!(rows, "music stat dedupe swept"),
            Err(e) => tracing::warn!(error = %e, "music stat dedupe sweep failed"),
        }
    });
}

/// Counts a listen or download from the public endpoints.
///
/// # Errors
/// `400` for a malformed id, `404` for a track the catalogue does not list.
pub async fn record(
    state: &AppState,
    kind: MusicStatKind,
    track_id: &str,
    client: &StatClient,
) -> Result<bool, ApiError> {
    if !is_known_track(state, track_id).await? {
        return Err(ApiError::NotFound);
    }
    let mut conn = state.db.acquire().await?;
    let counted = record_on(&mut conn, kind, track_id, client, Utc::now()).await?;
    if counted {
        note_counted(state);
    }
    Ok(counted)
}

/// The cached value under `key`, or `build`'s, stored on a miss.
async fn cached<T, F, Fut>(state: &AppState, key: &CacheKey<'_>, build: F) -> Result<T, ApiError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    if let Some(hit) = state.cache.get(key).await {
        return Ok(hit);
    }
    let out = build().await?;
    state.cache.set(key, &out).await;
    Ok(out)
}

/// Every track's all-time counts, cached for a minute.
pub async fn catalog(state: &AppState) -> Result<MusicStatsCatalog, ApiError> {
    cached(state, &CacheKey::MusicStats, || async {
        let mut conn = state.db.acquire().await?;
        let rows = queries::all_counts(&mut conn).await?;
        Ok(MusicStatsCatalog {
            tracks: rows
                .into_iter()
                .map(|r| {
                    (
                        r.track_id,
                        MusicTrackCounts {
                            listens: r.listens,
                            downloads: r.downloads,
                        },
                    )
                })
                .collect(),
        })
    })
    .await
}

/// The window's start, `None` for all time.
fn since(window: MusicStatWindow, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    window.days().map(|d| now - chrono::Duration::days(d))
}

/// The most listened tracks in the window.
pub async fn top_tracks(
    state: &AppState,
    window: MusicStatWindow,
    limit: u32,
) -> Result<MusicTopTracks, ApiError> {
    let limit = limit.clamp(1, TOP_TRACKS_MAX);
    let key = CacheKey::MusicTopTracks {
        window: window.as_str(),
        limit,
    };
    cached(state, &key, || async {
        let mut conn = state.db.acquire().await?;
        let entries =
            queries::top_tracks(&mut conn, since(window, Utc::now()), i64::from(limit)).await?;
        Ok(MusicTopTracks { window, entries })
    })
    .await
}

/// One page of visible listeners in the window, site-wide or of one track.
///
/// # Errors
/// `400` for a malformed track id.
pub async fn listeners(
    state: &AppState,
    window: MusicStatWindow,
    sort: MusicListenerSort,
    track_id: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<MusicListenerPage, ApiError> {
    if let Some(track) = track_id {
        check_track_id(track)?;
    }
    let key = CacheKey::MusicListeners {
        window: window.as_str(),
        sort: sort.as_str(),
        track: track_id,
        limit,
        offset,
    };
    cached(state, &key, || async {
        let from = since(window, Utc::now()).unwrap_or(DateTime::UNIX_EPOCH);
        let mut conn = state.db.acquire().await?;
        let entries = queries::listeners(
            &mut conn,
            from,
            track_id,
            sort,
            i64::from(limit),
            i64::from(offset),
        )
        .await?;
        let total = queries::count_listeners(&mut conn, from, track_id).await?;
        Ok(MusicListenerPage {
            track_id: track_id.map(str::to_owned),
            window,
            sort,
            entries,
            total,
        })
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv6_is_keyed_by_its_64() {
        let a: IpAddr = "2001:db8:1:2:aaaa::1".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:ffff:1:2:3".parse().unwrap();
        let c: IpAddr = "2001:db8:1:3::1".parse().unwrap();
        assert_eq!(anonymous_client("k", a), anonymous_client("k", b));
        assert_ne!(anonymous_client("k", a), anonymous_client("k", c));
    }

    #[test]
    fn a_mapped_ipv4_is_its_ipv4() {
        let v4: IpAddr = "203.0.113.7".parse().unwrap();
        let mapped: IpAddr = "::ffff:203.0.113.7".parse().unwrap();
        assert_eq!(anonymous_client("k", v4), anonymous_client("k", mapped));
    }

    #[test]
    fn the_key_never_holds_the_address_and_depends_on_the_secret() {
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        let StatClient::Anonymous(key) = anonymous_client("k", ip) else {
            panic!("anonymous");
        };
        assert!(key.starts_with("a:") && key.len() == 2 + 64);
        assert!(!key.contains("203.0.113.7"));
        assert_ne!(anonymous_client("k", ip), anonymous_client("other", ip));
    }

    #[test]
    fn users_and_addresses_never_share_a_key() {
        let user = StatClient::User(Uuid::nil()).key();
        assert!(user.starts_with("u:"));
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        assert!(anonymous_client("k", ip).key().starts_with("a:"));
    }

    #[test]
    fn a_forwarded_header_never_names_the_caller() {
        // A proxy on the same host connects from loopback; the header it
        // passes on is the client's to write, so the peer is the key.
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "1.1.1.1, 203.0.113.7".parse().unwrap());
        let mut ext = Extensions::new();
        let peer: std::net::SocketAddr = "127.0.0.1:40000".parse().unwrap();
        ext.insert(axum::extract::ConnectInfo(peer));
        assert_eq!(client_addr(&h, &ext), "127.0.0.1".parse().ok());
    }
}
