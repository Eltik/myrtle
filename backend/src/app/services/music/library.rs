//! The account's music: likes, listening history and playlists.
//!
//! A track id is `siren:<cid>` or `game:<track id>` (see
//! `database::models::music`). The prefix is checked here; existence is not,
//! because a row must outlive a track the catalogue later drops, and because
//! the in-game index is per server while the account is not.

use std::collections::HashMap;

use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::app::validation::validate_length;
use crate::database::models::music::{
    HISTORY_LIMIT, MusicLibrary, MusicPlay, MusicPlayInput, MusicPlayUpdate, MusicPlaylist,
    MusicPlaylistAdd, MusicPlaylistCreate, MusicPlaylistPatch, MusicPlaylistRow, PLAYLIST_LIMIT,
    PLAYLIST_NAME_MAX_CHARS, PLAYLIST_TRACKS_MAX,
};
use crate::database::queries::music as queries;

/// The prefixes a track id may carry.
pub const TRACK_KINDS: [&str; 2] = ["siren:", "game:"];
/// The longest track id stored. In-game ids are clip stems (under 60 chars on
/// EN); the label's cids are 4 to 6 digits.
const TRACK_ID_MAX: usize = 120;
/// The longest a single play can claim, in seconds: a day. Guards the totals
/// against a stuck tab.
const LISTENED_MAX: f32 = 86_400.0;

/// # Errors
/// `400` for an id without a known prefix, with whitespace, empty or too long.
pub fn check_track_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() || id.len() > TRACK_ID_MAX {
        return Err(ApiError::BadRequest(format!(
            "track id must be 1 to {TRACK_ID_MAX} bytes"
        )));
    }
    if id.chars().any(char::is_whitespace) {
        return Err(ApiError::BadRequest(
            "track id must not contain whitespace".to_owned(),
        ));
    }
    let known = TRACK_KINDS
        .iter()
        .any(|p| id.starts_with(p) && id.len() > p.len());
    if !known {
        return Err(ApiError::BadRequest(format!(
            "track id must start with one of {}",
            TRACK_KINDS.join(", ")
        )));
    }
    Ok(())
}

fn check_track_ids(ids: &[String]) -> Result<Vec<String>, ApiError> {
    if ids.len() > PLAYLIST_TRACKS_MAX {
        return Err(ApiError::BadRequest(format!(
            "a playlist holds at most {PLAYLIST_TRACKS_MAX} tracks"
        )));
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

fn check_name(name: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::BadRequest("name must not be blank".to_owned()));
    }
    validate_length("name", name, PLAYLIST_NAME_MAX_CHARS)?;
    Ok(name.to_owned())
}

fn check_listened(seconds: f32) -> Result<f32, ApiError> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(ApiError::BadRequest(
            "listenedSeconds must be 0 or more".to_owned(),
        ));
    }
    Ok(seconds.min(LISTENED_MAX))
}

/// Everything the account keeps.
pub async fn get(state: &AppState, user_id: Uuid) -> Result<MusicLibrary, ApiError> {
    let mut conn = state.db.acquire().await?;
    let likes = queries::list_likes(&mut conn, user_id).await?;
    let history = queries::list_history(&mut conn, user_id, HISTORY_LIMIT).await?;
    let totals = queries::play_totals(&mut conn, user_id).await?;
    let first_played_at = queries::first_played_at(&mut conn, user_id).await?;
    let playlists = queries::list_playlists(&mut conn, user_id).await?;
    let tracks = queries::list_playlist_tracks(&mut conn, user_id).await?;

    let mut by_playlist: HashMap<Uuid, Vec<String>> = HashMap::new();
    for t in tracks {
        by_playlist
            .entry(t.playlist_id)
            .or_default()
            .push(t.track_id);
    }
    Ok(MusicLibrary {
        likes: likes.into_iter().map(|r| r.into_like()).collect(),
        history: history.into_iter().map(|r| r.into_play()).collect(),
        history_limit: HISTORY_LIMIT,
        totals: totals.into_iter().map(|r| r.into_total()).collect(),
        first_played_at,
        playlists: playlists
            .into_iter()
            .map(|p| {
                let ids = by_playlist.remove(&p.id).unwrap_or_default();
                p.into_playlist(ids)
            })
            .collect(),
        playlist_limit: PLAYLIST_LIMIT,
        playlist_tracks_max: i64::try_from(PLAYLIST_TRACKS_MAX).unwrap_or(i64::MAX),
    })
}

/// # Errors
/// `400` for a bad id.
pub async fn like(state: &AppState, user_id: Uuid, track_id: &str) -> Result<(), ApiError> {
    check_track_id(track_id)?;
    let mut conn = state.db.acquire().await?;
    queries::set_like(&mut conn, user_id, track_id).await?;
    Ok(())
}

/// # Errors
/// `404` when the track was not liked.
pub async fn unlike(state: &AppState, user_id: Uuid, track_id: &str) -> Result<(), ApiError> {
    check_track_id(track_id)?;
    let mut conn = state.db.acquire().await?;
    if queries::unset_like(&mut conn, user_id, track_id).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Log a play.
///
/// # Errors
/// `400` for a bad id or a negative time.
pub async fn log_play(
    state: &AppState,
    user_id: Uuid,
    input: MusicPlayInput,
) -> Result<MusicPlay, ApiError> {
    check_track_id(&input.track_id)?;
    let listened = check_listened(input.listened_seconds)?;
    let mut conn = state.db.acquire().await?;
    let row = queries::insert_play(&mut conn, user_id, &input.track_id, listened).await?;
    Ok(row.into_play())
}

/// Raise a play's listened time.
///
/// # Errors
/// `404` when the play is not the caller's.
pub async fn update_play(
    state: &AppState,
    user_id: Uuid,
    id: i64,
    input: MusicPlayUpdate,
) -> Result<(), ApiError> {
    let listened = check_listened(input.listened_seconds)?;
    let mut conn = state.db.acquire().await?;
    if queries::update_play(&mut conn, user_id, id, listened).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

async fn with_tracks(
    conn: &mut sqlx::PgConnection,
    row: MusicPlaylistRow,
) -> Result<MusicPlaylist, ApiError> {
    let ids = queries::playlist_track_ids(conn, row.id).await?;
    Ok(row.into_playlist(ids))
}

/// # Errors
/// `400` for a bad name or ids, `409` at the playlist limit.
pub async fn create_playlist(
    state: &AppState,
    user_id: Uuid,
    input: MusicPlaylistCreate,
) -> Result<MusicPlaylist, ApiError> {
    let name = check_name(&input.name)?;
    let ids = check_track_ids(&input.track_ids)?;
    let mut tx = state.db.begin().await?;
    if queries::count_playlists(&mut tx, user_id).await? >= PLAYLIST_LIMIT {
        return Err(ApiError::Conflict(format!(
            "an account keeps at most {PLAYLIST_LIMIT} playlists"
        )));
    }
    let row = queries::create_playlist(&mut tx, user_id, &name).await?;
    if !ids.is_empty() {
        queries::replace_playlist_tracks(&mut tx, row.id, &ids).await?;
    }
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

/// Rename, reorder or both.
///
/// # Errors
/// `404` when the playlist is not the caller's.
pub async fn patch_playlist(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
    input: MusicPlaylistPatch,
) -> Result<MusicPlaylist, ApiError> {
    let name = input.name.as_deref().map(check_name).transpose()?;
    let ids = input
        .track_ids
        .as_deref()
        .map(check_track_ids)
        .transpose()?;
    let mut tx = state.db.begin().await?;
    let mut row = queries::get_playlist(&mut tx, user_id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if let Some(name) = name {
        row = queries::rename_playlist(&mut tx, user_id, id, &name)
            .await?
            .ok_or(ApiError::NotFound)?;
    }
    if let Some(ids) = ids {
        queries::replace_playlist_tracks(&mut tx, id, &ids).await?;
        row = queries::get_playlist(&mut tx, user_id, id)
            .await?
            .ok_or(ApiError::NotFound)?;
    }
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

/// # Errors
/// `404` when the playlist is not the caller's.
pub async fn delete_playlist(state: &AppState, user_id: Uuid, id: Uuid) -> Result<(), ApiError> {
    let mut conn = state.db.acquire().await?;
    if queries::delete_playlist(&mut conn, user_id, id).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Append tracks, skipping those already in.
///
/// # Errors
/// `404` when the playlist is not the caller's, `400` when the result would
/// pass the per-playlist cap.
pub async fn add_tracks(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
    input: MusicPlaylistAdd,
) -> Result<MusicPlaylist, ApiError> {
    let ids = check_track_ids(&input.track_ids)?;
    let mut tx = state.db.begin().await?;
    let row = queries::get_playlist(&mut tx, user_id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let current = queries::playlist_track_ids(&mut tx, id).await?;
    let new = ids.iter().filter(|t| !current.contains(t)).count();
    if current.len() + new > PLAYLIST_TRACKS_MAX {
        return Err(ApiError::BadRequest(format!(
            "a playlist holds at most {PLAYLIST_TRACKS_MAX} tracks"
        )));
    }
    if new > 0 {
        queries::append_playlist_tracks(&mut tx, id, &ids).await?;
    }
    let row = queries::get_playlist(&mut tx, user_id, id)
        .await?
        .unwrap_or(row);
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

/// # Errors
/// `404` when the playlist is not the caller's or the track is not in it.
pub async fn remove_track(
    state: &AppState,
    user_id: Uuid,
    id: Uuid,
    track_id: &str,
) -> Result<MusicPlaylist, ApiError> {
    let mut tx = state.db.begin().await?;
    queries::get_playlist(&mut tx, user_id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if !queries::remove_playlist_track(&mut tx, id, track_id).await? {
        return Err(ApiError::NotFound);
    }
    let row = queries::get_playlist(&mut tx, user_id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let out = with_tracks(&mut tx, row).await?;
    tx.commit().await?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_ids_need_a_known_prefix() {
        assert!(check_track_id("siren:048756").is_ok());
        assert!(check_track_id("game:m_bat_act27side").is_ok());
        assert!(check_track_id("siren:").is_err());
        assert!(check_track_id("spotify:123").is_err());
        assert!(check_track_id("siren:0487 56").is_err());
        assert!(check_track_id("").is_err());
    }

    #[test]
    fn listened_time_is_clamped_to_a_day() {
        assert_eq!(check_listened(12.5).unwrap(), 12.5);
        assert_eq!(check_listened(1e9).unwrap(), LISTENED_MAX);
        assert!(check_listened(-1.0).is_err());
        assert!(check_listened(f32::NAN).is_err());
    }
}
