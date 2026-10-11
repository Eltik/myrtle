//! The music library's tables: the Monster Siren catalogue cache
//! (`siren_albums`, `siren_songs`) and the account's likes, listening history
//! and playlists (`music_likes`, `music_history`, `music_playlists`,
//! `music_playlist_tracks`). Every function takes a connection, so a caller or
//! a test runs it inside its own transaction.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::database::models::music::{
    MusicLikeRow, MusicPlayRow, MusicPlayTotalRow, MusicPlaylistRow, MusicPlaylistTrackRow,
    SirenAlbumRow, SirenSongRow,
};

// ---------------------------------------------------------------------------
// Catalogue cache
// ---------------------------------------------------------------------------

const ALBUM_COLUMNS: &str =
    "cid, name, intro, belong, cover_url, cover_de_url, artists, list_order, fetched_at";
const SONG_COLUMNS: &str = "cid, album_cid, name, artists, lyric_url, mv_url, mv_cover_url, track_order, lyric_text IS NOT NULL AS has_lyric_text";

pub async fn list_albums(conn: &mut PgConnection) -> Result<Vec<SirenAlbumRow>, sqlx::Error> {
    sqlx::query_as::<_, SirenAlbumRow>(&format!(
        "SELECT {ALBUM_COLUMNS} FROM siren_albums ORDER BY list_order"
    ))
    .fetch_all(conn)
    .await
}

pub async fn list_songs(conn: &mut PgConnection) -> Result<Vec<SirenSongRow>, sqlx::Error> {
    sqlx::query_as::<_, SirenSongRow>(&format!(
        "SELECT {SONG_COLUMNS} FROM siren_songs ORDER BY album_cid, track_order"
    ))
    .fetch_all(conn)
    .await
}

/// One release as the sync writes it. `intro`, `belong` and `cover_de_url`
/// are the detail call's; a caller that did not re-fetch the detail passes
/// the row it already holds.
#[derive(Debug, Clone, Copy)]
pub struct AlbumUpsert<'a> {
    pub cid: &'a str,
    pub name: &'a str,
    pub intro: &'a str,
    pub belong: &'a str,
    pub cover_url: &'a str,
    pub cover_de_url: Option<&'a str>,
    pub artists: &'a [String],
    pub list_order: i32,
}

/// Insert or refresh one release.
pub async fn upsert_album(conn: &mut PgConnection, a: AlbumUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO siren_albums (cid, name, intro, belong, cover_url, cover_de_url, artists, list_order, fetched_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (cid) DO UPDATE
         SET name = EXCLUDED.name, intro = EXCLUDED.intro, belong = EXCLUDED.belong,
             cover_url = EXCLUDED.cover_url, cover_de_url = EXCLUDED.cover_de_url,
             artists = EXCLUDED.artists, list_order = EXCLUDED.list_order,
             fetched_at = now()",
    )
    .bind(a.cid)
    .bind(a.name)
    .bind(a.intro)
    .bind(a.belong)
    .bind(a.cover_url)
    .bind(a.cover_de_url)
    .bind(a.artists)
    .bind(a.list_order)
    .execute(conn)
    .await?;
    Ok(())
}

/// One track as the sync writes it.
#[derive(Debug, Clone, Copy)]
pub struct SongUpsert<'a> {
    pub cid: &'a str,
    pub album_cid: &'a str,
    pub name: &'a str,
    pub artists: &'a [String],
    pub lyric_url: Option<&'a str>,
    pub mv_url: Option<&'a str>,
    pub mv_cover_url: Option<&'a str>,
    pub track_order: i32,
}

/// Insert or refresh one track. The lyrics text is kept across refreshes
/// unless the lyric URL moved, when it is dropped for a fresh fetch.
pub async fn upsert_song(conn: &mut PgConnection, s: SongUpsert<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO siren_songs (cid, album_cid, name, artists, lyric_url, mv_url, mv_cover_url, track_order, fetched_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (cid) DO UPDATE
         SET album_cid = EXCLUDED.album_cid, name = EXCLUDED.name, artists = EXCLUDED.artists,
             lyric_url = EXCLUDED.lyric_url, mv_url = EXCLUDED.mv_url,
             mv_cover_url = EXCLUDED.mv_cover_url, track_order = EXCLUDED.track_order,
             lyric_text = CASE WHEN siren_songs.lyric_url IS DISTINCT FROM EXCLUDED.lyric_url THEN NULL ELSE siren_songs.lyric_text END,
             fetched_at = now()",
    )
    .bind(s.cid)
    .bind(s.album_cid)
    .bind(s.name)
    .bind(s.artists)
    .bind(s.lyric_url)
    .bind(s.mv_url)
    .bind(s.mv_cover_url)
    .bind(s.track_order)
    .execute(conn)
    .await?;
    Ok(())
}

/// Drop the releases and tracks the label no longer lists. Returns (albums,
/// songs) removed.
pub async fn prune_unlisted(
    conn: &mut PgConnection,
    album_cids: &[String],
    song_cids: &[String],
) -> Result<(u64, u64), sqlx::Error> {
    let songs = sqlx::query("DELETE FROM siren_songs WHERE NOT (cid = ANY($1))")
        .bind(song_cids)
        .execute(&mut *conn)
        .await?
        .rows_affected();
    let albums = sqlx::query("DELETE FROM siren_albums WHERE NOT (cid = ANY($1))")
        .bind(album_cids)
        .execute(conn)
        .await?
        .rows_affected();
    Ok((albums, songs))
}

pub async fn lyric_text(conn: &mut PgConnection, cid: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>("SELECT lyric_text FROM siren_songs WHERE cid = $1")
        .bind(cid)
        .fetch_optional(conn)
        .await
        .map(Option::flatten)
}

pub async fn set_lyric_text(
    conn: &mut PgConnection,
    cid: &str,
    text: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE siren_songs SET lyric_text = $2 WHERE cid = $1")
        .bind(cid)
        .bind(text)
        .execute(conn)
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Likes
// ---------------------------------------------------------------------------

pub async fn list_likes(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<MusicLikeRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicLikeRow>(
        "SELECT track_id, created_at FROM music_likes WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
}

/// Like a track; liking it again keeps the first like's time.
pub async fn set_like(
    conn: &mut PgConnection,
    user_id: Uuid,
    track_id: &str,
) -> Result<MusicLikeRow, sqlx::Error> {
    sqlx::query_as::<_, MusicLikeRow>(
        "INSERT INTO music_likes (user_id, track_id) VALUES ($1, $2)
         ON CONFLICT (user_id, track_id) DO UPDATE SET track_id = EXCLUDED.track_id
         RETURNING track_id, created_at",
    )
    .bind(user_id)
    .bind(track_id)
    .fetch_one(conn)
    .await
}

/// `true` when the like existed.
pub async fn unset_like(
    conn: &mut PgConnection,
    user_id: Uuid,
    track_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM music_likes WHERE user_id = $1 AND track_id = $2")
        .bind(user_id)
        .bind(track_id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

pub async fn list_history(
    conn: &mut PgConnection,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<MusicPlayRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlayRow>(
        "SELECT id, track_id, listened_seconds, played_at FROM music_history
         WHERE user_id = $1 ORDER BY played_at DESC, id DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(conn)
    .await
}

pub async fn insert_play(
    conn: &mut PgConnection,
    user_id: Uuid,
    track_id: &str,
    listened_seconds: f32,
) -> Result<MusicPlayRow, sqlx::Error> {
    sqlx::query_as::<_, MusicPlayRow>(
        "INSERT INTO music_history (user_id, track_id, listened_seconds) VALUES ($1, $2, $3)
         RETURNING id, track_id, listened_seconds, played_at",
    )
    .bind(user_id)
    .bind(track_id)
    .bind(listened_seconds)
    .fetch_one(conn)
    .await
}

/// Raise a play's listened time. The time only grows: a late update with a
/// smaller value (a stale tab) is kept out. `true` when the play is the
/// caller's.
pub async fn update_play(
    conn: &mut PgConnection,
    user_id: Uuid,
    id: i64,
    listened_seconds: f32,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE music_history SET listened_seconds = GREATEST(listened_seconds, $3)
         WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user_id)
    .bind(listened_seconds)
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Plays and listened seconds per track over the whole history.
pub async fn play_totals(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<MusicPlayTotalRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlayTotalRow>(
        "SELECT track_id, COUNT(*) AS plays, COALESCE(SUM(listened_seconds), 0)::float8 AS seconds,
                MAX(played_at) AS last_played_at
         FROM music_history WHERE user_id = $1
         GROUP BY track_id ORDER BY plays DESC, seconds DESC, track_id",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
}

pub async fn first_played_at(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT MIN(played_at) FROM music_history WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(conn)
    .await
}

// ---------------------------------------------------------------------------
// Playlists
// ---------------------------------------------------------------------------

const PLAYLIST_COLUMNS: &str = "id, name, created_at, updated_at";

pub async fn list_playlists(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<MusicPlaylistRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlaylistRow>(&format!(
        "SELECT {PLAYLIST_COLUMNS} FROM music_playlists WHERE user_id = $1 ORDER BY created_at, id"
    ))
    .bind(user_id)
    .fetch_all(conn)
    .await
}

/// Every track of every playlist the user owns, in playlist position order.
pub async fn list_playlist_tracks(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<Vec<MusicPlaylistTrackRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlaylistTrackRow>(
        "SELECT t.playlist_id, t.track_id, t.position
         FROM music_playlist_tracks t
         JOIN music_playlists p ON p.id = t.playlist_id
         WHERE p.user_id = $1 ORDER BY t.playlist_id, t.position, t.added_at",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await
}

pub async fn count_playlists(conn: &mut PgConnection, user_id: Uuid) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM music_playlists WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(conn)
        .await
}

pub async fn create_playlist(
    conn: &mut PgConnection,
    user_id: Uuid,
    name: &str,
) -> Result<MusicPlaylistRow, sqlx::Error> {
    sqlx::query_as::<_, MusicPlaylistRow>(&format!(
        "INSERT INTO music_playlists (user_id, name) VALUES ($1, $2) RETURNING {PLAYLIST_COLUMNS}"
    ))
    .bind(user_id)
    .bind(name)
    .fetch_one(conn)
    .await
}

/// The playlist when the user owns it.
pub async fn get_playlist(
    conn: &mut PgConnection,
    user_id: Uuid,
    id: Uuid,
) -> Result<Option<MusicPlaylistRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlaylistRow>(&format!(
        "SELECT {PLAYLIST_COLUMNS} FROM music_playlists WHERE user_id = $1 AND id = $2"
    ))
    .bind(user_id)
    .bind(id)
    .fetch_optional(conn)
    .await
}

pub async fn rename_playlist(
    conn: &mut PgConnection,
    user_id: Uuid,
    id: Uuid,
    name: &str,
) -> Result<Option<MusicPlaylistRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicPlaylistRow>(&format!(
        "UPDATE music_playlists SET name = $3 WHERE user_id = $1 AND id = $2 RETURNING {PLAYLIST_COLUMNS}"
    ))
    .bind(user_id)
    .bind(id)
    .bind(name)
    .fetch_optional(conn)
    .await
}

/// `true` when the playlist was the user's.
pub async fn delete_playlist(
    conn: &mut PgConnection,
    user_id: Uuid,
    id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM music_playlists WHERE user_id = $1 AND id = $2")
        .bind(user_id)
        .bind(id)
        .execute(conn)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn playlist_track_ids(
    conn: &mut PgConnection,
    playlist_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT track_id FROM music_playlist_tracks WHERE playlist_id = $1 ORDER BY position, added_at",
    )
    .bind(playlist_id)
    .fetch_all(conn)
    .await
}

/// Make the playlist hold exactly `track_ids`, in that order. Tracks already
/// in it keep their `added_at`.
pub async fn replace_playlist_tracks(
    conn: &mut PgConnection,
    playlist_id: Uuid,
    track_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM music_playlist_tracks WHERE playlist_id = $1 AND NOT (track_id = ANY($2))",
    )
    .bind(playlist_id)
    .bind(track_ids)
    .execute(&mut *conn)
    .await?;
    let positions: Vec<i32> = (0..track_ids.len())
        .map(|i| i32::try_from(i).unwrap_or(i32::MAX))
        .collect();
    sqlx::query(
        "INSERT INTO music_playlist_tracks (playlist_id, track_id, position)
         SELECT $1, t.track_id, t.position
         FROM UNNEST($2::text[], $3::int4[]) AS t(track_id, position)
         ON CONFLICT (playlist_id, track_id) DO UPDATE SET position = EXCLUDED.position",
    )
    .bind(playlist_id)
    .bind(track_ids)
    .bind(&positions)
    .execute(&mut *conn)
    .await?;
    touch_playlist(conn, playlist_id).await
}

/// Append the tracks not already in the playlist, after the last one.
pub async fn append_playlist_tracks(
    conn: &mut PgConnection,
    playlist_id: Uuid,
    track_ids: &[String],
) -> Result<(), sqlx::Error> {
    let next: i32 = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(position) FROM music_playlist_tracks WHERE playlist_id = $1",
    )
    .bind(playlist_id)
    .fetch_one(&mut *conn)
    .await?
    .map_or(0, |p| p.saturating_add(1));
    let positions: Vec<i32> = (0..track_ids.len())
        .map(|i| next.saturating_add(i32::try_from(i).unwrap_or(i32::MAX)))
        .collect();
    sqlx::query(
        "INSERT INTO music_playlist_tracks (playlist_id, track_id, position)
         SELECT $1, t.track_id, t.position
         FROM UNNEST($2::text[], $3::int4[]) AS t(track_id, position)
         ON CONFLICT (playlist_id, track_id) DO NOTHING",
    )
    .bind(playlist_id)
    .bind(track_ids)
    .bind(&positions)
    .execute(&mut *conn)
    .await?;
    touch_playlist(conn, playlist_id).await
}

/// `true` when the track was in the playlist.
pub async fn remove_playlist_track(
    conn: &mut PgConnection,
    playlist_id: Uuid,
    track_id: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM music_playlist_tracks WHERE playlist_id = $1 AND track_id = $2")
            .bind(playlist_id)
            .bind(track_id)
            .execute(&mut *conn)
            .await?;
    if result.rows_affected() > 0 {
        touch_playlist(conn, playlist_id).await?;
    }
    Ok(result.rows_affected() > 0)
}

/// Move `updated_at`, so a membership change shows on the playlist itself.
async fn touch_playlist(conn: &mut PgConnection, playlist_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE music_playlists SET updated_at = now() WHERE id = $1")
        .bind(playlist_id)
        .execute(conn)
        .await?;
    Ok(())
}
