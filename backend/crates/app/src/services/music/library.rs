use std::collections::HashMap;

use chrono::{DateTime, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::music::{self as repo, PlaylistRow};
use myrtle_db::repos::music_stats::StatKind;
use myrtle_types::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::stats::{self, StatClient};
use crate::{AppContext, AppError, Subject};

pub const PLAYLIST_NAME_MAX_CHARS: usize = 60;
pub const PLAYLIST_LIMIT: i64 = 100;
pub const PLAYLIST_TRACKS_MAX: usize = 1000;
pub const HISTORY_LIMIT: i64 = 200;
pub const TRACK_KINDS: [&str; 2] = ["siren:", "game:"];
const TRACK_ID_MAX: usize = 120;
const LISTENED_MAX: f32 = 86_400.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicLike {
    pub track_id: String,
    pub liked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlay {
    pub id: i64,
    pub track_id: String,
    pub listened_seconds: f32,
    pub played_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayInput {
    pub track_id: String,
    pub listened_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayUpdate {
    pub listened_seconds: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlayTotal {
    pub track_id: String,
    pub plays: i64,
    pub seconds: f64,
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylist {
    pub id: Uuid,
    pub name: String,
    pub track_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylistCreate {
    pub name: String,
    #[serde(default)]
    pub track_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylistPatch {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub track_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicPlaylistAdd {
    pub track_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicLibrary {
    pub likes: Vec<MusicLike>,
    pub history: Vec<MusicPlay>,
    pub history_limit: i64,
    pub totals: Vec<MusicPlayTotal>,
    pub first_played_at: Option<DateTime<Utc>>,
    pub playlists: Vec<MusicPlaylist>,
    pub playlist_limit: i64,
    pub playlist_tracks_max: i64,
}

pub fn check_track_id(id: &str) -> Result<(), AppError> {
    if id.is_empty() || id.len() > TRACK_ID_MAX {
        return Err(AppError::bad_request(format!(
            "track id must be 1 to {TRACK_ID_MAX} bytes"
        )));
    }
    if id.chars().any(char::is_whitespace) {
        return Err(AppError::bad_request(
            "track id must not contain whitespace",
        ));
    }
    if !TRACK_KINDS
        .iter()
        .any(|p| id.starts_with(p) && id.len() > p.len())
    {
        return Err(AppError::bad_request(format!(
            "track id must start with one of {}",
            TRACK_KINDS.join(", ")
        )));
    }
    Ok(())
}

fn check_track_ids(ids: &[String]) -> Result<Vec<String>, AppError> {
    if ids.len() > PLAYLIST_TRACKS_MAX {
        return Err(playlist_full());
    }
    let mut out: Vec<String> = Vec::with_capacity(ids.len());
    for id in ids {
        check_track_id(id)?;
        if !out.contains(id) {
            out.push(id.clone());
        }
    }
    Ok(out)
}

fn check_name(name: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("name must not be blank"));
    }
    if name.chars().count() > PLAYLIST_NAME_MAX_CHARS {
        return Err(AppError::bad_request(format!(
            "name max {PLAYLIST_NAME_MAX_CHARS} chars"
        )));
    }
    Ok(name.to_owned())
}

pub fn check_listened(seconds: f32) -> Result<f32, AppError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(AppError::bad_request("listenedSeconds must be 0 or more"));
    }
    Ok(seconds.min(LISTENED_MAX))
}

fn playlist_full() -> AppError {
    AppError::bad_request(format!(
        "a playlist holds at most {PLAYLIST_TRACKS_MAX} tracks"
    ))
}

const fn found(touched: bool) -> Result<(), AppError> {
    if touched {
        Ok(())
    } else {
        Err(AppError::NotFound(None))
    }
}

fn signed_in(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

async fn owned_playlist(conn: &mut Conn, user: UserId, id: Uuid) -> Result<PlaylistRow, AppError> {
    repo::get_playlist(conn, user, id)
        .await?
        .ok_or(AppError::NotFound(None))
}

fn playlist(row: PlaylistRow, track_ids: Vec<String>) -> MusicPlaylist {
    MusicPlaylist {
        id: row.id,
        name: row.name,
        track_ids,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

async fn with_tracks(conn: &mut Conn, row: PlaylistRow) -> Result<MusicPlaylist, AppError> {
    let ids = repo::playlist_track_ids(conn, row.id).await?;
    Ok(playlist(row, ids))
}

pub async fn get(ctx: &AppContext, subject: &Subject) -> Result<MusicLibrary, AppError> {
    let user = signed_in(subject)?;
    let mut conn = ctx.db.conn().await?;
    let likes = repo::list_likes(&mut conn, user).await?;
    let history = repo::list_history(&mut conn, user, HISTORY_LIMIT).await?;
    let totals = repo::play_totals(&mut conn, user).await?;
    let first_played_at = repo::first_played_at(&mut conn, user).await?;
    let playlists = repo::list_playlists(&mut conn, user).await?;
    let mut by_playlist: HashMap<Uuid, Vec<String>> = HashMap::new();
    for t in repo::list_playlist_tracks(&mut conn, user).await? {
        by_playlist
            .entry(t.playlist_id)
            .or_default()
            .push(t.track_id);
    }
    Ok(MusicLibrary {
        likes: likes
            .into_iter()
            .map(|r| MusicLike {
                track_id: r.track_id,
                liked_at: r.created_at,
            })
            .collect(),
        history: history.into_iter().map(play).collect(),
        history_limit: HISTORY_LIMIT,
        totals: totals
            .into_iter()
            .map(|r| MusicPlayTotal {
                track_id: r.track_id,
                plays: r.plays,
                seconds: r.seconds,
                last_played_at: r.last_played_at,
            })
            .collect(),
        first_played_at,
        playlists: playlists
            .into_iter()
            .map(|p| {
                let ids = by_playlist.remove(&p.id).unwrap_or_default();
                playlist(p, ids)
            })
            .collect(),
        playlist_limit: PLAYLIST_LIMIT,
        playlist_tracks_max: i64::try_from(PLAYLIST_TRACKS_MAX).unwrap_or(i64::MAX),
    })
}

fn play(r: repo::PlayRow) -> MusicPlay {
    MusicPlay {
        id: r.id,
        track_id: r.track_id,
        listened_seconds: r.listened_seconds,
        played_at: r.played_at,
    }
}

pub async fn like(ctx: &AppContext, subject: &Subject, track_id: &str) -> Result<(), AppError> {
    let user = signed_in(subject)?;
    check_track_id(track_id)?;
    let mut conn = ctx.db.conn().await?;
    repo::set_like(&mut conn, user, track_id).await?;
    Ok(())
}

pub async fn unlike(ctx: &AppContext, subject: &Subject, track_id: &str) -> Result<(), AppError> {
    let user = signed_in(subject)?;
    check_track_id(track_id)?;
    let mut conn = ctx.db.conn().await?;
    found(repo::unset_like(&mut conn, user, track_id).await?)
}

pub async fn log_play(
    ctx: &AppContext,
    subject: &Subject,
    input: MusicPlayInput,
) -> Result<MusicPlay, AppError> {
    let user = signed_in(subject)?;
    check_track_id(&input.track_id)?;
    let listened = check_listened(input.listened_seconds)?;
    let known = stats::is_known_track(ctx, &input.track_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, track = %input.track_id, "music stats: track check failed");
            false
        });
    let mut tx = ctx.db.begin().await?;
    let row = repo::insert_play(&mut tx, user, &input.track_id, listened).await?;
    if known {
        stats::record_on(
            &mut tx,
            StatKind::Listen,
            &input.track_id,
            &StatClient::User(user),
            Utc::now(),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(play(row))
}

pub async fn update_play(
    ctx: &AppContext,
    subject: &Subject,
    id: i64,
    input: MusicPlayUpdate,
) -> Result<(), AppError> {
    let user = signed_in(subject)?;
    let listened = check_listened(input.listened_seconds)?;
    let mut conn = ctx.db.conn().await?;
    found(repo::update_play(&mut conn, user, id, listened).await?)
}

pub async fn create_playlist(
    ctx: &AppContext,
    subject: &Subject,
    input: MusicPlaylistCreate,
) -> Result<MusicPlaylist, AppError> {
    let user = signed_in(subject)?;
    let name = check_name(&input.name)?;
    let ids = check_track_ids(&input.track_ids)?;
    let mut tx = ctx.db.begin().await?;
    if repo::count_playlists(&mut tx, user).await? >= PLAYLIST_LIMIT {
        return Err(AppError::Conflict(format!(
            "an account keeps at most {PLAYLIST_LIMIT} playlists"
        )));
    }
    let row = repo::create_playlist(&mut tx, user, &name).await?;
    if !ids.is_empty() {
        repo::replace_playlist_tracks(&mut tx, row.id, &ids).await?;
    }
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

pub async fn patch_playlist(
    ctx: &AppContext,
    subject: &Subject,
    id: Uuid,
    input: MusicPlaylistPatch,
) -> Result<MusicPlaylist, AppError> {
    let user = signed_in(subject)?;
    let name = input.name.as_deref().map(check_name).transpose()?;
    let ids = input
        .track_ids
        .as_deref()
        .map(check_track_ids)
        .transpose()?;
    let mut tx = ctx.db.begin().await?;
    let mut row = owned_playlist(&mut tx, user, id).await?;
    if let Some(name) = name {
        row = repo::rename_playlist(&mut tx, user, id, &name)
            .await?
            .ok_or(AppError::NotFound(None))?;
    }
    if let Some(ids) = ids {
        repo::replace_playlist_tracks(&mut tx, id, &ids).await?;
        row = owned_playlist(&mut tx, user, id).await?;
    }
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

pub async fn delete_playlist(
    ctx: &AppContext,
    subject: &Subject,
    id: Uuid,
) -> Result<(), AppError> {
    let user = signed_in(subject)?;
    let mut conn = ctx.db.conn().await?;
    found(repo::delete_playlist(&mut conn, user, id).await?)
}

pub async fn add_tracks(
    ctx: &AppContext,
    subject: &Subject,
    id: Uuid,
    input: MusicPlaylistAdd,
) -> Result<MusicPlaylist, AppError> {
    let user = signed_in(subject)?;
    let ids = check_track_ids(&input.track_ids)?;
    let mut tx = ctx.db.begin().await?;
    owned_playlist(&mut tx, user, id).await?;
    let current = repo::playlist_track_ids(&mut tx, id).await?;
    let new = ids.iter().filter(|t| !current.contains(t)).count();
    if current.len() + new > PLAYLIST_TRACKS_MAX {
        return Err(playlist_full());
    }
    if new > 0 {
        repo::append_playlist_tracks(&mut tx, id, &ids).await?;
    }
    let row = owned_playlist(&mut tx, user, id).await?;
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

pub async fn remove_track(
    ctx: &AppContext,
    subject: &Subject,
    id: Uuid,
    track_id: &str,
) -> Result<MusicPlaylist, AppError> {
    let user = signed_in(subject)?;
    let mut tx = ctx.db.begin().await?;
    owned_playlist(&mut tx, user, id).await?;
    if !repo::remove_playlist_track(&mut tx, id, track_id).await? {
        return Err(AppError::NotFound(None));
    }
    let row = owned_playlist(&mut tx, user, id).await?;
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}
