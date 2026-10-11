//! Monster Siren Records, the game's own label, as a catalogue this backend
//! serves: every release and track, each album's blurb and banner, a signed
//! stream URL per track and the LRC lyrics.
//!
//! The label's API is read on a CLOCK, never per request. [`spawn`] loads the
//! cached catalogue from Postgres at boot, then every `SIREN_REFRESH_SECS`
//! (default 6 h) re-reads the two list endpoints (2 requests) and fetches the
//! detail of a release or track it has NOT seen before, spaced
//! [`DETAIL_SPACING`] apart. A first sync of the whole catalogue is 298 + 911
//! detail calls, about five minutes at that spacing, and it happens once: the
//! rows persist, so a restart costs 2 requests. The in-memory snapshot the
//! routes read is rebuilt from the rows after each sync.
//!
//! Two things are read on demand because they cannot be cached ahead:
//!
//! - the STREAM URL, which the label signs for about three hours. One song
//!   detail call per track per ~3 h, kept in [`STREAMS`] until five minutes
//!   before it expires;
//! - the LYRICS file, fetched from the CDN the first time a reader asks and
//!   written into `siren_songs.lyric_text`, so it is fetched once ever.

pub mod client;
pub mod dto;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use arc_swap::ArcSwapOption;
use axum::body::Bytes;
use chrono::{DateTime, NaiveDate, Utc};
use dashmap::DashMap;
use std::sync::LazyLock;
use tokio::sync::Mutex;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::database::models::music::{SirenAlbumRow, SirenSongRow};
use crate::database::queries::music as queries;

pub use client::{SirenClient, cdn_day, signed_url_expiry};
pub use dto::*;

/// Gap between two detail calls during a sync.
const DETAIL_SPACING: Duration = Duration::from_millis(250);
/// Default interval between syncs.
const DEFAULT_REFRESH_SECS: u64 = 6 * 60 * 60;
/// A signed stream URL is re-fetched this long before it expires.
const STREAM_SAFETY: u64 = 5 * 60;
/// When a stream URL carries no readable expiry, trust it for this long.
const STREAM_FALLBACK_TTL: u64 = 30 * 60;
/// The title fragment that marks an instrumental, matched case-insensitively.
/// The API has no flag; the label writes `(Instrumental)` or `（Instrumental）`
/// (ASCII or full-width parentheses: 116 + 12 = 128 tracks on 2026-10-11).
const INSTRUMENTAL_MARK: &str = "instrumental";

/// What the routes read: the catalogue, serialised once, plus the per-album
/// detail and the per-song row for the on-demand calls.
pub struct Snapshot {
    pub catalog: SirenCatalog,
    pub json: Bytes,
    albums: HashMap<String, SirenAlbumDetail>,
    songs: HashMap<String, SirenSongRow>,
}

static SNAPSHOT: ArcSwapOption<Snapshot> = ArcSwapOption::const_empty();
/// One sync at a time; a second caller finds the lock held and moves on.
static SYNC_LOCK: Mutex<()> = Mutex::const_new(());
/// Signed stream URLs by cid with the unix second they are dropped.
static STREAMS: LazyLock<DashMap<String, (String, u64)>> = LazyLock::new(DashMap::new);
/// Lyrics text by cid, so a repeat reader does not touch the database.
static LYRICS: LazyLock<DashMap<String, Arc<str>>> = LazyLock::new(DashMap::new);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn is_instrumental(name: &str) -> bool {
    name.to_lowercase().contains(INSTRUMENTAL_MARK)
}

/// `name` without a trailing parenthesised `INSTRUMENTAL_MARK` (ASCII or
/// full-width parentheses, any case, inner spaces allowed); `name` when the
/// marker is not at the end or is the whole title.
fn without_instrumental_mark(name: &str) -> String {
    let name = name.trim();
    let strip = || {
        let body = name
            .strip_suffix(')')
            .or_else(|| name.strip_suffix('\u{ff09}'))?
            .trim_end();
        let head = body.get(..body.len().checked_sub(INSTRUMENTAL_MARK.len())?)?;
        if !body[head.len()..].eq_ignore_ascii_case(INSTRUMENTAL_MARK) {
            return None;
        }
        let head = head.trim_end();
        let head = head
            .strip_suffix('(')
            .or_else(|| head.strip_suffix('\u{ff08}'))?
            .trim_end();
        (!head.is_empty()).then(|| head.to_owned())
    };
    strip().unwrap_or_else(|| name.to_owned())
}

/// A count as the `i32` the wire carries, saturating.
fn count_i32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// Artist names trimmed, blanks and repeats dropped, first spelling kept.
fn clean_artists(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(list.len());
    for a in list {
        let a = a.trim();
        if !a.is_empty() && !out.iter().any(|x| x == a) {
            out.push(a.to_owned());
        }
    }
    out
}

fn song_dto(row: &SirenSongRow) -> SirenSong {
    SirenSong {
        cid: row.cid.clone(),
        name: row.name.trim().to_owned(),
        title: without_instrumental_mark(&row.name),
        album_cid: row.album_cid.clone(),
        artists: clean_artists(&row.artists),
        has_lyrics: row.lyric_url.is_some(),
        instrumental: is_instrumental(&row.name),
        has_mv: row.mv_url.is_some(),
        track_order: row.track_order,
    }
}

/// The day shared by more than half the releases, if any: the bulk import.
fn bulk_import_day(albums: &[SirenAlbum]) -> Option<NaiveDate> {
    let mut counts: HashMap<NaiveDate, usize> = HashMap::new();
    for a in albums {
        if let Some(d) = a.added_on {
            *counts.entry(d).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .filter(|(_, n)| *n * 2 > albums.len())
        .map(|(d, _)| d)
        .next()
}

fn build_snapshot(
    album_rows: Vec<SirenAlbumRow>,
    song_rows: Vec<SirenSongRow>,
    synced_at: Option<DateTime<Utc>>,
) -> Result<Snapshot, ApiError> {
    let mut by_album: HashMap<String, Vec<SirenSong>> = HashMap::new();
    let mut songs_map: HashMap<String, SirenSongRow> = HashMap::with_capacity(song_rows.len());
    for row in song_rows {
        by_album
            .entry(row.album_cid.clone())
            .or_default()
            .push(song_dto(&row));
        songs_map.insert(row.cid.clone(), row);
    }
    for list in by_album.values_mut() {
        list.sort_by_key(|s| s.track_order);
    }

    let mut albums: Vec<SirenAlbum> = Vec::with_capacity(album_rows.len());
    let mut details: HashMap<String, SirenAlbumDetail> = HashMap::with_capacity(album_rows.len());
    let mut songs: Vec<SirenSong> = Vec::with_capacity(songs_map.len());
    let mut album_rows = album_rows;
    album_rows.sort_by_key(|a| a.list_order);
    for row in album_rows {
        let tracks = by_album.remove(&row.cid).unwrap_or_default();
        let artists = clean_artists(&row.artists);
        let added_on = cdn_day(&row.cover_url);
        albums.push(SirenAlbum {
            cid: row.cid.clone(),
            name: row.name.trim().to_owned(),
            cover_url: row.cover_url.clone(),
            artists: artists.clone(),
            list_order: row.list_order,
            track_count: count_i32(tracks.len()),
            lyrics_count: count_i32(tracks.iter().filter(|s| s.has_lyrics).count()),
            instrumental_count: count_i32(tracks.iter().filter(|s| s.instrumental).count()),
            added_on,
        });
        songs.extend(tracks.iter().cloned());
        details.insert(
            row.cid.clone(),
            SirenAlbumDetail {
                cid: row.cid,
                name: row.name.trim().to_owned(),
                intro: row.intro,
                belong: row.belong,
                cover_url: row.cover_url,
                cover_de_url: row.cover_de_url,
                artists,
                list_order: row.list_order,
                added_on,
                songs: tracks,
            },
        );
    }
    // A song whose album row is missing (never, unless the album call failed)
    // is still listed, at the end.
    for (_, mut orphans) in by_album {
        orphans.sort_by_key(|s| s.track_order);
        songs.extend(orphans);
    }

    let catalog = SirenCatalog {
        bulk_import_day: bulk_import_day(&albums),
        albums,
        songs,
        synced_at,
    };
    let json = serde_json::to_vec(&catalog).map_err(|e| ApiError::Internal(e.into()))?;
    Ok(Snapshot {
        catalog,
        json: Bytes::from(json),
        albums: details,
        songs: songs_map,
    })
}

/// Rebuild the snapshot from the rows.
async fn load_from_db(
    state: &AppState,
    synced_at: Option<DateTime<Utc>>,
) -> Result<Arc<Snapshot>, ApiError> {
    let mut conn = state.db.acquire().await?;
    let albums = queries::list_albums(&mut conn).await?;
    let songs = queries::list_songs(&mut conn).await?;
    let synced_at =
        synced_at.or_else(|| SNAPSHOT.load().as_ref().and_then(|s| s.catalog.synced_at));
    let snap = Arc::new(build_snapshot(albums, songs, synced_at)?);
    SNAPSHOT.store(Some(Arc::clone(&snap)));
    Ok(snap)
}

/// The snapshot, loading it from the database on the first call. `None` when
/// the database holds no catalogue yet; the caller then starts a sync.
async fn snapshot(state: &AppState) -> Result<Option<Arc<Snapshot>>, ApiError> {
    if let Some(s) = SNAPSHOT.load_full() {
        return Ok(Some(s));
    }
    let s = load_from_db(state, None).await?;
    if s.catalog.albums.is_empty() {
        return Ok(None);
    }
    Ok(Some(s))
}

/// The snapshot, or `503` with a sync started when the database holds no
/// catalogue yet.
async fn ready_snapshot(state: &AppState) -> Result<Arc<Snapshot>, ApiError> {
    let Some(snap) = snapshot(state).await? else {
        kick_sync(state);
        return Err(ApiError::ServiceUnavailable);
    };
    Ok(snap)
}

/// Start a sync in the background unless one is running.
fn kick_sync(state: &AppState) {
    if SYNC_LOCK.try_lock().is_err() {
        return;
    }
    let state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = sync(&state).await {
            tracing::warn!(error = %e, "monster siren sync failed");
        }
    });
}

/// One sync: the two lists, the details not yet held, the rows, the snapshot.
/// Returns a one-line summary for the log.
///
/// # Errors
/// A list call that fails aborts the sync with nothing written; a detail call
/// that fails after retries skips that release or track until the next sync.
pub async fn sync(state: &AppState) -> anyhow::Result<String> {
    let Ok(_guard) = SYNC_LOCK.try_lock() else {
        return Ok("a sync is already running".to_owned());
    };
    let started = std::time::Instant::now();
    let client = SirenClient::new(&state.http_client);
    let albums = client.albums().await?;
    let songs = client.songs().await?;

    let known_albums: HashMap<String, SirenAlbumRow> = {
        let mut conn = state.db.acquire().await?;
        queries::list_albums(&mut conn)
            .await?
            .into_iter()
            .map(|r| (r.cid.clone(), r))
            .collect()
    };
    let known_songs: HashSet<String> = {
        let mut conn = state.db.acquire().await?;
        queries::list_songs(&mut conn)
            .await?
            .into_iter()
            .map(|r| r.cid)
            .collect()
    };

    // An album is re-read when it is new or when one of its listed tracks is
    // new (the track order comes from the album detail).
    let mut albums_to_fetch: HashSet<String> = HashSet::new();
    for a in &albums {
        if !known_albums.contains_key(&a.cid) {
            albums_to_fetch.insert(a.cid.clone());
        }
    }
    for s in &songs {
        if !known_songs.contains(&s.cid) {
            albums_to_fetch.insert(s.album_cid.clone());
        }
    }

    let mut fetched_albums = 0usize;
    let mut failed_albums = 0usize;
    let mut track_orders: HashMap<String, i32> = HashMap::new();
    {
        // A connection per write, not one held across the whole crawl: a
        // first sync sleeps for minutes between calls and must not pin a
        // pool slot the request path needs.
        for (order, a) in albums.iter().enumerate() {
            let mut conn = state.db.acquire().await?;
            let list_order = count_i32(order);
            let artists = clean_artists(&a.artistes);
            let mut detail = None;
            if albums_to_fetch.contains(&a.cid) {
                tokio::time::sleep(DETAIL_SPACING).await;
                match fetch_with_retry(|| client.album_detail(&a.cid)).await {
                    Ok(d) => {
                        fetched_albums += 1;
                        for (i, s) in d.songs.iter().enumerate() {
                            track_orders.insert(s.cid.clone(), count_i32(i));
                        }
                        detail = Some(d);
                    }
                    Err(e) => {
                        failed_albums += 1;
                        tracing::warn!(cid = %a.cid, error = %e, "monster siren album detail failed");
                    }
                }
            }
            // The detail fields come from the call just made, else from the
            // row already held. A release listed but never detailed is stored
            // from its listing with an empty blurb, so its tracks have an album.
            let (intro, belong, cover_de_url) = match (&detail, known_albums.get(&a.cid)) {
                (Some(d), _) => (d.intro.trim(), d.belong.trim(), d.cover_de_url.as_deref()),
                (None, Some(known)) => (
                    known.intro.as_str(),
                    known.belong.as_str(),
                    known.cover_de_url.as_deref(),
                ),
                (None, None) if albums_to_fetch.contains(&a.cid) => ("", "", None),
                (None, None) => continue,
            };
            queries::upsert_album(
                &mut conn,
                queries::AlbumUpsert {
                    cid: &a.cid,
                    name: a.name.trim(),
                    intro,
                    belong,
                    cover_url: &a.cover_url,
                    cover_de_url,
                    artists: &artists,
                    list_order,
                },
            )
            .await?;
        }
    }

    let mut fetched_songs = 0usize;
    let mut failed_songs = 0usize;
    {
        for s in &songs {
            let mut conn = state.db.acquire().await?;
            if known_songs.contains(&s.cid) {
                // Listing fields may move (a rename, an album move); the
                // detail fields stay as fetched.
                queries::update_song_listing(
                    &mut conn,
                    queries::SongListing {
                        cid: &s.cid,
                        album_cid: &s.album_cid,
                        name: s.name.trim(),
                        artists: &clean_artists(&s.artists),
                        track_order: track_orders.get(&s.cid).copied(),
                    },
                )
                .await?;
                continue;
            }
            tokio::time::sleep(DETAIL_SPACING).await;
            let order = track_orders.get(&s.cid).copied().unwrap_or(0);
            match fetch_with_retry(|| client.song_detail(&s.cid)).await {
                Ok(d) => {
                    fetched_songs += 1;
                    queries::upsert_song(
                        &mut conn,
                        queries::SongUpsert {
                            cid: &s.cid,
                            album_cid: &s.album_cid,
                            name: s.name.trim(),
                            artists: &clean_artists(&s.artists),
                            lyric_url: d.lyric_url.as_deref(),
                            mv_url: d.mv_url.as_deref(),
                            mv_cover_url: d.mv_cover_url.as_deref(),
                            track_order: order,
                        },
                    )
                    .await?;
                }
                Err(e) => {
                    failed_songs += 1;
                    tracing::warn!(cid = %s.cid, error = %e, "monster siren song detail failed");
                }
            }
        }
        let mut conn = state.db.acquire().await?;
        let album_cids: Vec<String> = albums.iter().map(|a| a.cid.clone()).collect();
        let song_cids: Vec<String> = songs.iter().map(|s| s.cid.clone()).collect();
        let (pruned_albums, pruned_songs) =
            queries::prune_unlisted(&mut conn, &album_cids, &song_cids).await?;
        if pruned_albums + pruned_songs > 0 {
            tracing::info!(
                pruned_albums,
                pruned_songs,
                "monster siren rows no longer listed"
            );
        }
    }

    let snap = load_from_db(state, Some(Utc::now())).await?;
    let summary = format!(
        "monster siren: {} albums, {} songs ({} with lyrics); fetched {fetched_albums} album and {fetched_songs} song details, {failed_albums}+{failed_songs} failed, {} ms",
        snap.catalog.albums.len(),
        snap.catalog.songs.len(),
        snap.catalog.songs.iter().filter(|s| s.has_lyrics).count(),
        started.elapsed().as_millis()
    );
    tracing::info!("{summary}");
    Ok(summary)
}

/// Three tries with a growing pause: the label's API drops connections under
/// a steady crawl (6 of 298 album calls needed a retry on 2026-10-11).
async fn fetch_with_retry<T, F, Fut>(mut call: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut last = None;
    for attempt in 0..3u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(2 * u64::from(attempt))).await;
        }
        match call().await {
            Ok(v) => return Ok(v),
            Err(e) => last = Some(e),
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("no attempt made")))
}

/// Load the cached catalogue, then sync on the clock. `SIREN_REFRESH_SECS=0`
/// disables the clock (the catalogue is then whatever the database holds, or
/// is synced once on the first request when it holds nothing).
pub fn spawn(state: AppState) {
    let interval_secs: u64 = std::env::var("SIREN_REFRESH_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_REFRESH_SECS);
    tokio::spawn(async move {
        match load_from_db(&state, None).await {
            Ok(s) => tracing::info!(
                albums = s.catalog.albums.len(),
                songs = s.catalog.songs.len(),
                "monster siren catalogue loaded from the database"
            ),
            Err(e) => tracing::warn!(error = %e, "monster siren catalogue load failed"),
        }
        if interval_secs == 0 {
            tracing::info!("SIREN_REFRESH_SECS=0: monster siren sync clock off");
            return;
        }
        crate::core::jobs::stagger("monster_siren").await;
        loop {
            if let Err(e) = sync(&state).await {
                tracing::warn!(error = %e, "monster siren sync failed");
            }
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    });
}

/// The catalogue as JSON bytes.
///
/// # Errors
/// `503` while the first sync has not finished: the call starts it.
pub async fn catalog(state: &AppState) -> Result<Bytes, ApiError> {
    Ok(ready_snapshot(state).await?.json.clone())
}

/// One release with its tracks.
///
/// # Errors
/// `404` for an unknown cid, `503` before the first sync.
pub async fn album(state: &AppState, cid: &str) -> Result<SirenAlbumDetail, ApiError> {
    let snap = ready_snapshot(state).await?;
    snap.albums.get(cid).cloned().ok_or(ApiError::NotFound)
}

/// The signed CDN URL for a track's audio, fetched from the label when the
/// held one is within [`STREAM_SAFETY`] of expiring.
///
/// # Errors
/// `404` for a cid the catalogue does not list (so a guessed id never reaches
/// the label), `503` when the label does not answer.
pub async fn stream_url(state: &AppState, cid: &str) -> Result<String, ApiError> {
    let snap = ready_snapshot(state).await?;
    if !snap.songs.contains_key(cid) {
        return Err(ApiError::NotFound);
    }
    let now = now_secs();
    if let Some(hit) = STREAMS.get(cid)
        && hit.1 > now + STREAM_SAFETY
    {
        return Ok(hit.0.clone());
    }
    let client = SirenClient::new(&state.http_client);
    let detail = client.song_detail(cid).await.map_err(|e| {
        tracing::warn!(cid, error = %e, "monster siren song detail failed");
        ApiError::ServiceUnavailable
    })?;
    let url = detail.source_url.ok_or(ApiError::NotFound)?;
    let expiry = signed_url_expiry(&url).unwrap_or(now + STREAM_FALLBACK_TTL);
    STREAMS.insert(cid.to_owned(), (url.clone(), expiry));
    Ok(url)
}

/// A track's LRC lyrics, from memory, then the row, then the label's CDN
/// (written back into the row).
///
/// # Errors
/// `404` for a track without lyrics or an unknown cid, `503` when the CDN does
/// not answer.
pub async fn lyrics(state: &AppState, cid: &str) -> Result<Arc<str>, ApiError> {
    if let Some(hit) = LYRICS.get(cid) {
        return Ok(Arc::clone(&hit));
    }
    let snap = ready_snapshot(state).await?;
    let row = snap.songs.get(cid).ok_or(ApiError::NotFound)?;
    let url = row.lyric_url.as_deref().ok_or(ApiError::NotFound)?;
    let mut conn = state.db.acquire().await?;
    let text = match queries::lyric_text(&mut conn, cid).await? {
        Some(t) => t,
        None => {
            let client = SirenClient::new(&state.http_client);
            let t = client.text(url).await.map_err(|e| {
                tracing::warn!(cid, error = %e, "monster siren lyrics fetch failed");
                ApiError::ServiceUnavailable
            })?;
            queries::set_lyric_text(&mut conn, cid, &t).await?;
            t
        }
    };
    let text: Arc<str> = Arc::from(text);
    LYRICS.insert(cid.to_owned(), Arc::clone(&text));
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instrumental_mark_reads_both_parenthesis_styles() {
        assert!(is_instrumental("Radiant (Instrumental)"));
        assert!(is_instrumental("Vows of the Sea（Instrumental）"));
        assert!(is_instrumental("x (instrumental)"));
        assert!(!is_instrumental("Instruments of War"));
    }

    #[test]
    fn display_title_drops_only_a_trailing_mark() {
        let t = without_instrumental_mark;
        assert_eq!(t("Speed of Light (Instrumental)"), "Speed of Light");
        assert_eq!(t("Speed of Light (instrumental) "), "Speed of Light");
        assert_eq!(t("Speed of Light ( Instrumental )"), "Speed of Light");
        assert_eq!(t("光（Instrumental）"), "光");
        assert_eq!(t("(Instrumental)"), "(Instrumental)");
        assert_eq!(t("A (Instrumental) Mix"), "A (Instrumental) Mix");
        assert_eq!(t("Instruments of War"), "Instruments of War");
    }

    #[test]
    fn artists_are_trimmed_and_deduplicated() {
        let got = clean_artists(&[
            " Emperor".into(),
            "Emperor".into(),
            "".into(),
            " D.D.D.".into(),
        ]);
        assert_eq!(got, vec!["Emperor".to_owned(), "D.D.D.".to_owned()]);
    }

    #[test]
    fn bulk_import_day_needs_a_majority() {
        let day = |d: u32| NaiveDate::from_ymd_opt(2021, 3, d);
        let mk = |added: Option<NaiveDate>| SirenAlbum {
            cid: String::new(),
            name: String::new(),
            cover_url: String::new(),
            artists: vec![],
            list_order: 0,
            track_count: 0,
            lyrics_count: 0,
            instrumental_count: 0,
            added_on: added,
        };
        let majority = vec![mk(day(22)), mk(day(22)), mk(day(22)), mk(day(23))];
        assert_eq!(bulk_import_day(&majority), day(22));
        let split = vec![mk(day(22)), mk(day(23))];
        assert_eq!(bulk_import_day(&split), None);
    }
}
