use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, NaiveDate, Utc};
use myrtle_db::repos::music::{
    self as repo, AlbumRow, AlbumUpsert, SongListing, SongRow, SongUpsert,
};
use myrtle_db::repos::music_stats::siren_song_exists;
use myrtle_hypergryph::UpstreamError;
use myrtle_hypergryph::siren::{
    ApiAlbumDetail, ApiAlbumSummary, ApiSongDetail, ApiSongSummary, SirenClient, cdn_day,
    signed_url_expiry,
};
use myrtle_jobs::{JobKind, JobSpec};
use serde::{Deserialize, Serialize};

use crate::cache::CachedJson;
use crate::{AppContext, AppError};

pub const CACHE_PREFIX: &str = "music:siren";
pub const CATALOG_KEY: &str = "music:siren:catalog";
const ALBUMS_KEY: &str = "music:siren:albums";
const CATALOG_TTL: Duration = Duration::from_hours(24);
const DETAIL_SPACING: Duration = Duration::from_millis(250);
const STREAM_SAFETY: u64 = 5 * 60;
const STREAM_FALLBACK_TTL: u64 = 30 * 60;
const INSTRUMENTAL_MARK: &str = "instrumental";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenAlbum {
    pub cid: String,
    pub name: String,
    pub cover_url: String,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub track_count: i32,
    pub lyrics_count: i32,
    pub instrumental_count: i32,
    pub added_on: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenSong {
    pub cid: String,
    pub name: String,
    pub title: String,
    pub album_cid: String,
    pub artists: Vec<String>,
    pub has_lyrics: bool,
    pub instrumental: bool,
    pub has_mv: bool,
    pub track_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenCatalog {
    pub albums: Vec<SirenAlbum>,
    pub songs: Vec<SirenSong>,
    pub synced_at: Option<DateTime<Utc>>,
    pub bulk_import_day: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SirenAlbumDetail {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: Option<String>,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub added_on: Option<NaiveDate>,
    pub songs: Vec<SirenSong>,
}

#[derive(Debug, Clone, Default)]
pub struct SirenFetch {
    pub albums: Vec<ApiAlbumSummary>,
    pub songs: Vec<ApiSongSummary>,
    pub album_details: HashMap<String, ApiAlbumDetail>,
    pub song_details: HashMap<String, ApiSongDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SirenStored {
    pub albums: usize,
    pub songs: usize,
    pub with_lyrics: usize,
    pub pruned_albums: u64,
    pub pruned_songs: u64,
}

#[must_use]
pub fn is_instrumental(name: &str) -> bool {
    name.to_lowercase().contains(INSTRUMENTAL_MARK)
}

#[must_use]
pub fn without_instrumental_mark(name: &str) -> String {
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

#[must_use]
pub fn clean_artists(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(list.len());
    for a in list {
        let a = a.trim();
        if !a.is_empty() && !out.iter().any(|x| x == a) {
            out.push(a.to_owned());
        }
    }
    out
}

#[must_use]
pub fn bulk_import_day(albums: &[SirenAlbum]) -> Option<NaiveDate> {
    let mut counts: HashMap<NaiveDate, usize> = HashMap::new();
    for d in albums.iter().filter_map(|a| a.added_on) {
        *counts.entry(d).or_default() += 1;
    }
    counts
        .into_iter()
        .find(|(_, n)| *n * 2 > albums.len())
        .map(|(d, _)| d)
}

fn count_i32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

fn song_dto(row: &SongRow) -> SirenSong {
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

#[must_use]
pub fn build_catalog(
    mut album_rows: Vec<AlbumRow>,
    song_rows: &[SongRow],
) -> (SirenCatalog, HashMap<String, SirenAlbumDetail>) {
    let mut by_album: HashMap<&str, Vec<SirenSong>> = HashMap::new();
    for row in song_rows {
        by_album
            .entry(&row.album_cid)
            .or_default()
            .push(song_dto(row));
    }
    for list in by_album.values_mut() {
        list.sort_by_key(|s| s.track_order);
    }
    let synced_at = album_rows.iter().map(|a| a.fetched_at).max();
    album_rows.sort_by_key(|a| a.list_order);
    let mut albums = Vec::with_capacity(album_rows.len());
    let mut details = HashMap::with_capacity(album_rows.len());
    let mut songs = Vec::with_capacity(song_rows.len());
    for row in album_rows {
        let tracks = by_album.remove(row.cid.as_str()).unwrap_or_default();
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
    let mut orphans: Vec<(&str, Vec<SirenSong>)> = by_album.into_iter().collect();
    orphans.sort_by_key(|(cid, _)| *cid);
    songs.extend(orphans.into_iter().flat_map(|(_, list)| list));
    let catalog = SirenCatalog {
        bulk_import_day: bulk_import_day(&albums),
        albums,
        songs,
        synced_at,
    };
    (catalog, details)
}

async fn load(
    ctx: &AppContext,
) -> Result<(SirenCatalog, HashMap<String, SirenAlbumDetail>), AppError> {
    let mut conn = ctx.db.conn().await?;
    let albums = repo::list_albums(&mut conn).await?;
    let songs = repo::list_songs(&mut conn).await?;
    drop(conn);
    if albums.is_empty() {
        let first_sync = JobSpec {
            dedupe_key: Some("siren:first_sync".to_owned()),
            ..JobSpec::new(JobKind::SirenSync)
        };
        if let Err(e) = ctx.queue.enqueue(first_sync).await {
            tracing::warn!(error = %e, "monster siren sync not queued");
        }
        return Err(AppError::Unavailable);
    }
    let (catalog, details) = build_catalog(albums, &songs);
    ctx.cache.set_json(ALBUMS_KEY, &details, CATALOG_TTL).await;
    Ok((catalog, details))
}

pub async fn catalog(ctx: &AppContext) -> Result<CachedJson, AppError> {
    ctx.cache
        .json(CATALOG_KEY, CATALOG_TTL, || async {
            let (catalog, _) = load(ctx).await?;
            serde_json::to_string(&catalog).map_err(AppError::internal)
        })
        .await
}

async fn albums(ctx: &AppContext) -> Result<HashMap<String, SirenAlbumDetail>, AppError> {
    if let Some(hit) = ctx.cache.get_json(ALBUMS_KEY).await {
        return Ok(hit);
    }
    Ok(load(ctx).await?.1)
}

pub async fn album(ctx: &AppContext, cid: &str) -> Result<SirenAlbumDetail, AppError> {
    albums(ctx)
        .await?
        .remove(cid)
        .ok_or(AppError::NotFound(None))
}

fn client(ctx: &AppContext) -> SirenClient<'_> {
    SirenClient::new(ctx.upstream.client(), &ctx.config.jobs.siren.api_base)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

pub async fn stream_url(ctx: &AppContext, cid: &str) -> Result<String, AppError> {
    let mut conn = ctx.db.conn().await?;
    if !siren_song_exists(&mut conn, cid).await? {
        return Err(AppError::NotFound(None));
    }
    drop(conn);
    let key = format!("music:siren:stream:{cid}");
    if let Some(url) = ctx.cache.get(&key).await {
        return Ok(url);
    }
    let detail = client(ctx).song_detail(cid).await.map_err(|e| {
        tracing::warn!(cid, error = %e, "monster siren song detail failed");
        AppError::Unavailable
    })?;
    let url = detail.source_url.ok_or(AppError::NotFound(None))?;
    let now = now_secs();
    let expiry = signed_url_expiry(&url).unwrap_or(now + STREAM_FALLBACK_TTL);
    let keep = expiry.saturating_sub(now + STREAM_SAFETY);
    if keep > 0 {
        ctx.cache.set(&key, &url, Duration::from_secs(keep)).await;
    }
    Ok(url)
}

pub async fn lyrics(ctx: &AppContext, cid: &str) -> Result<String, AppError> {
    let mut conn = ctx.db.conn().await?;
    if let Some(text) = repo::lyric_text(&mut conn, cid).await? {
        return Ok(text);
    }
    let url = repo::list_songs(&mut conn)
        .await?
        .into_iter()
        .find(|s| s.cid == cid)
        .and_then(|s| s.lyric_url)
        .ok_or(AppError::NotFound(None))?;
    drop(conn);
    let text = client(ctx).text(&url).await.map_err(|e| {
        tracing::warn!(cid, error = %e, "monster siren lyrics fetch failed");
        AppError::Unavailable
    })?;
    let mut conn = ctx.db.conn().await?;
    repo::set_lyric_text(&mut conn, cid, &text).await?;
    Ok(text)
}

#[must_use]
pub fn albums_to_fetch(
    albums: &[ApiAlbumSummary],
    songs: &[ApiSongSummary],
    known_albums: &HashSet<String>,
    known_songs: &HashSet<String>,
) -> HashSet<String> {
    albums
        .iter()
        .filter(|a| !known_albums.contains(&a.cid))
        .map(|a| a.cid.clone())
        .chain(
            songs
                .iter()
                .filter(|s| !known_songs.contains(&s.cid))
                .map(|s| s.album_cid.clone()),
        )
        .collect()
}

async fn with_retry<T, F, Fut>(mut call: F) -> Result<T, UpstreamError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, UpstreamError>>,
{
    let mut attempt = 0_u64;
    loop {
        match call().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt == 2 => return Err(e),
            Err(_) => {}
        }
        attempt += 1;
        tokio::time::sleep(Duration::from_secs(2 * attempt)).await;
    }
}

pub async fn sync(ctx: &AppContext) -> Result<String, AppError> {
    let started = std::time::Instant::now();
    let client = client(ctx);
    let mut fetch = SirenFetch {
        albums: client.albums().await?,
        songs: client.songs().await?,
        ..SirenFetch::default()
    };
    let mut conn = ctx.db.conn().await?;
    let known_albums: HashSet<String> = repo::list_albums(&mut conn)
        .await?
        .into_iter()
        .map(|r| r.cid)
        .collect();
    let known_songs: HashSet<String> = repo::list_songs(&mut conn)
        .await?
        .into_iter()
        .map(|r| r.cid)
        .collect();
    drop(conn);
    let wanted = albums_to_fetch(&fetch.albums, &fetch.songs, &known_albums, &known_songs);
    let mut failed = (0, 0);
    for a in fetch.albums.iter().filter(|a| wanted.contains(&a.cid)) {
        tokio::time::sleep(DETAIL_SPACING).await;
        match with_retry(|| client.album_detail(&a.cid)).await {
            Ok(d) => {
                fetch.album_details.insert(a.cid.clone(), d);
            }
            Err(e) => {
                failed.0 += 1;
                tracing::warn!(cid = %a.cid, error = %e, "monster siren album detail failed");
            }
        }
    }
    for s in fetch.songs.iter().filter(|s| !known_songs.contains(&s.cid)) {
        tokio::time::sleep(DETAIL_SPACING).await;
        match with_retry(|| client.song_detail(&s.cid)).await {
            Ok(d) => {
                fetch.song_details.insert(s.cid.clone(), d);
            }
            Err(e) => {
                failed.1 += 1;
                tracing::warn!(cid = %s.cid, error = %e, "monster siren song detail failed");
            }
        }
    }
    let stored = store(ctx, &fetch).await?;
    Ok(format!(
        "monster siren: {} albums, {} songs ({} with lyrics); fetched {} album and {} song details, {}+{} failed, {} pruned, {} ms",
        stored.albums,
        stored.songs,
        stored.with_lyrics,
        fetch.album_details.len(),
        fetch.song_details.len(),
        failed.0,
        failed.1,
        stored.pruned_albums + stored.pruned_songs,
        started.elapsed().as_millis()
    ))
}

pub async fn store(ctx: &AppContext, fetch: &SirenFetch) -> Result<SirenStored, AppError> {
    let mut conn = ctx.db.conn().await?;
    let known_albums: HashMap<String, AlbumRow> = repo::list_albums(&mut conn)
        .await?
        .into_iter()
        .map(|r| (r.cid.clone(), r))
        .collect();
    let known_songs: HashSet<String> = repo::list_songs(&mut conn)
        .await?
        .into_iter()
        .map(|r| r.cid)
        .collect();
    let track_orders: HashMap<&str, i32> = fetch
        .album_details
        .values()
        .flat_map(|d| d.songs.iter().enumerate())
        .map(|(i, s)| (s.cid.as_str(), count_i32(i)))
        .collect();
    for (order, a) in fetch.albums.iter().enumerate() {
        let (intro, belong, cover_de_url) =
            match (fetch.album_details.get(&a.cid), known_albums.get(&a.cid)) {
                (Some(d), _) => (d.intro.trim(), d.belong.trim(), d.cover_de_url.as_deref()),
                (None, Some(k)) => (
                    k.intro.as_str(),
                    k.belong.as_str(),
                    k.cover_de_url.as_deref(),
                ),
                (None, None) => ("", "", None),
            };
        repo::upsert_album(
            &mut conn,
            AlbumUpsert {
                cid: &a.cid,
                name: a.name.trim(),
                intro,
                belong,
                cover_url: &a.cover_url,
                cover_de_url,
                artists: &clean_artists(&a.artistes),
                list_order: count_i32(order),
            },
        )
        .await?;
    }
    for s in &fetch.songs {
        let artists = clean_artists(&s.artists);
        let track_order = track_orders.get(s.cid.as_str()).copied();
        if known_songs.contains(&s.cid) {
            repo::update_song_listing(
                &mut conn,
                SongListing {
                    cid: &s.cid,
                    album_cid: &s.album_cid,
                    name: s.name.trim(),
                    artists: &artists,
                    track_order,
                },
            )
            .await?;
        } else if let Some(d) = fetch.song_details.get(&s.cid) {
            repo::upsert_song(
                &mut conn,
                SongUpsert {
                    cid: &s.cid,
                    album_cid: &s.album_cid,
                    name: s.name.trim(),
                    artists: &artists,
                    lyric_url: d.lyric_url.as_deref(),
                    mv_url: d.mv_url.as_deref(),
                    mv_cover_url: d.mv_cover_url.as_deref(),
                    track_order: track_order.unwrap_or(0),
                },
            )
            .await?;
        }
    }
    let album_cids: Vec<String> = fetch.albums.iter().map(|a| a.cid.clone()).collect();
    let song_cids: Vec<String> = fetch.songs.iter().map(|s| s.cid.clone()).collect();
    let (pruned_albums, pruned_songs) =
        repo::prune_unlisted(&mut conn, &album_cids, &song_cids).await?;
    let albums = repo::list_albums(&mut conn).await?.len();
    let songs = repo::list_songs(&mut conn).await?;
    drop(conn);
    ctx.cache.invalidate_prefix(CACHE_PREFIX).await;
    Ok(SirenStored {
        albums,
        songs: songs.len(),
        with_lyrics: songs.iter().filter(|s| s.lyric_url.is_some()).count(),
        pruned_albums,
        pruned_songs,
    })
}
