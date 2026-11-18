use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct AlbumRow {
    pub cid: String,
    pub name: String,
    pub intro: String,
    pub belong: String,
    pub cover_url: String,
    pub cover_de_url: Option<String>,
    pub artists: Vec<String>,
    pub list_order: i32,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SongRow {
    pub cid: String,
    pub album_cid: String,
    pub name: String,
    pub artists: Vec<String>,
    pub lyric_url: Option<String>,
    pub mv_url: Option<String>,
    pub mv_cover_url: Option<String>,
    pub track_order: i32,
    pub has_lyric_text: bool,
}

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

#[derive(Debug, Clone, Copy)]
pub struct SongListing<'a> {
    pub cid: &'a str,
    pub album_cid: &'a str,
    pub name: &'a str,
    pub artists: &'a [String],
    pub track_order: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct LikeRow {
    pub track_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PlayRow {
    pub id: i64,
    pub track_id: String,
    pub listened_seconds: f32,
    pub played_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PlayTotalRow {
    pub track_id: String,
    pub plays: i64,
    pub seconds: f64,
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct PlaylistRow {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PlaylistTrackRow {
    pub playlist_id: Uuid,
    pub track_id: String,
    pub position: i32,
}

pub async fn list_albums(conn: &mut Conn) -> Result<Vec<AlbumRow>, DbError> {
    Ok(sqlx::query_as!(
        AlbumRow,
        "SELECT cid, name, intro, belong, cover_url, cover_de_url, artists, list_order, fetched_at
         FROM siren_albums ORDER BY list_order",
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_songs(conn: &mut Conn) -> Result<Vec<SongRow>, DbError> {
    Ok(sqlx::query_as!(
        SongRow,
        r#"SELECT cid, album_cid, name, artists, lyric_url, mv_url, mv_cover_url, track_order,
                  lyric_text IS NOT NULL AS "has_lyric_text!"
           FROM siren_songs ORDER BY album_cid, track_order"#,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn upsert_album(conn: &mut Conn, a: AlbumUpsert<'_>) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO siren_albums (cid, name, intro, belong, cover_url, cover_de_url, artists, list_order, fetched_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (cid) DO UPDATE
         SET name = EXCLUDED.name, intro = EXCLUDED.intro, belong = EXCLUDED.belong,
             cover_url = EXCLUDED.cover_url, cover_de_url = EXCLUDED.cover_de_url,
             artists = EXCLUDED.artists, list_order = EXCLUDED.list_order,
             fetched_at = now()",
        a.cid,
        a.name,
        a.intro,
        a.belong,
        a.cover_url,
        a.cover_de_url,
        a.artists,
        a.list_order,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn upsert_song(conn: &mut Conn, s: SongUpsert<'_>) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO siren_songs (cid, album_cid, name, artists, lyric_url, mv_url, mv_cover_url, track_order, fetched_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now())
         ON CONFLICT (cid) DO UPDATE
         SET album_cid = EXCLUDED.album_cid, name = EXCLUDED.name, artists = EXCLUDED.artists,
             lyric_url = EXCLUDED.lyric_url, mv_url = EXCLUDED.mv_url,
             mv_cover_url = EXCLUDED.mv_cover_url, track_order = EXCLUDED.track_order,
             lyric_text = CASE WHEN siren_songs.lyric_url IS DISTINCT FROM EXCLUDED.lyric_url THEN NULL ELSE siren_songs.lyric_text END,
             fetched_at = now()",
        s.cid,
        s.album_cid,
        s.name,
        s.artists,
        s.lyric_url,
        s.mv_url,
        s.mv_cover_url,
        s.track_order,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn update_song_listing(conn: &mut Conn, s: SongListing<'_>) -> Result<(), DbError> {
    let query = match s.track_order {
        Some(order) => sqlx::query!(
            "UPDATE siren_songs SET name = $2, album_cid = $3, artists = $4, track_order = $5 WHERE cid = $1",
            s.cid,
            s.name,
            s.album_cid,
            s.artists,
            order,
        ),
        None => sqlx::query!(
            "UPDATE siren_songs SET name = $2, album_cid = $3, artists = $4 WHERE cid = $1",
            s.cid,
            s.name,
            s.album_cid,
            s.artists,
        ),
    };
    query.execute(conn).await?;
    Ok(())
}

pub async fn prune_unlisted(
    conn: &mut Conn,
    album_cids: &[String],
    song_cids: &[String],
) -> Result<(u64, u64), DbError> {
    let pruned = sqlx::query!(
        r#"WITH songs AS (
               DELETE FROM siren_songs
               WHERE cardinality($2::text[]) > 0 AND NOT (cid = ANY($2)) RETURNING 1
           ),
           albums AS (
               DELETE FROM siren_albums
               WHERE cardinality($1::text[]) > 0 AND NOT (cid = ANY($1)) RETURNING 1
           )
           SELECT (SELECT COUNT(*) FROM albums) AS "albums!", (SELECT COUNT(*) FROM songs) AS "songs!""#,
        album_cids,
        song_cids,
    )
    .fetch_one(conn)
    .await?;
    Ok((pruned.albums.unsigned_abs(), pruned.songs.unsigned_abs()))
}

pub async fn lyric_text(conn: &mut Conn, cid: &str) -> Result<Option<String>, DbError> {
    Ok(
        sqlx::query_scalar!("SELECT lyric_text FROM siren_songs WHERE cid = $1", cid)
            .fetch_optional(conn)
            .await?
            .flatten(),
    )
}

pub async fn set_lyric_text(conn: &mut Conn, cid: &str, text: &str) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE siren_songs SET lyric_text = $2 WHERE cid = $1",
        cid,
        text,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn list_likes(conn: &mut Conn, user: UserId) -> Result<Vec<LikeRow>, DbError> {
    Ok(sqlx::query_as!(
        LikeRow,
        "SELECT track_id, created_at FROM music_likes WHERE user_id = $1 ORDER BY created_at DESC",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn set_like(conn: &mut Conn, user: UserId, track_id: &str) -> Result<LikeRow, DbError> {
    Ok(sqlx::query_as!(
        LikeRow,
        "INSERT INTO music_likes (user_id, track_id) VALUES ($1, $2)
         ON CONFLICT (user_id, track_id) DO UPDATE SET track_id = EXCLUDED.track_id
         RETURNING track_id, created_at",
        user.0,
        track_id,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn unset_like(conn: &mut Conn, user: UserId, track_id: &str) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM music_likes WHERE user_id = $1 AND track_id = $2",
        user.0,
        track_id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_history(
    conn: &mut Conn,
    user: UserId,
    limit: i64,
) -> Result<Vec<PlayRow>, DbError> {
    Ok(sqlx::query_as!(
        PlayRow,
        "SELECT id, track_id, listened_seconds, played_at FROM music_history
         WHERE user_id = $1 ORDER BY played_at DESC, id DESC LIMIT $2",
        user.0,
        limit,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn insert_play(
    conn: &mut Conn,
    user: UserId,
    track_id: &str,
    listened_seconds: f32,
) -> Result<PlayRow, DbError> {
    Ok(sqlx::query_as!(
        PlayRow,
        "INSERT INTO music_history (user_id, track_id, listened_seconds) VALUES ($1, $2, $3)
         RETURNING id, track_id, listened_seconds, played_at",
        user.0,
        track_id,
        listened_seconds,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn update_play(
    conn: &mut Conn,
    user: UserId,
    id: i64,
    listened_seconds: f32,
) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "UPDATE music_history SET listened_seconds = GREATEST(listened_seconds, $3)
         WHERE id = $1 AND user_id = $2",
        id,
        user.0,
        listened_seconds,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn play_totals(conn: &mut Conn, user: UserId) -> Result<Vec<PlayTotalRow>, DbError> {
    Ok(sqlx::query_as!(
        PlayTotalRow,
        r#"SELECT track_id, COUNT(*) AS "plays!",
                  COALESCE(SUM(listened_seconds), 0)::float8 AS "seconds!",
                  MAX(played_at) AS last_played_at
           FROM music_history WHERE user_id = $1
           GROUP BY track_id ORDER BY 2 DESC, 3 DESC, track_id"#,
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn first_played_at(
    conn: &mut Conn,
    user: UserId,
) -> Result<Option<DateTime<Utc>>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT MIN(played_at) FROM music_history WHERE user_id = $1",
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn list_playlists(conn: &mut Conn, user: UserId) -> Result<Vec<PlaylistRow>, DbError> {
    Ok(sqlx::query_as!(
        PlaylistRow,
        "SELECT id, name, created_at, updated_at FROM music_playlists
         WHERE user_id = $1 ORDER BY created_at, id",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn list_playlist_tracks(
    conn: &mut Conn,
    user: UserId,
) -> Result<Vec<PlaylistTrackRow>, DbError> {
    Ok(sqlx::query_as!(
        PlaylistTrackRow,
        "SELECT t.playlist_id, t.track_id, t.position
         FROM music_playlist_tracks t
         JOIN music_playlists p ON p.id = t.playlist_id
         WHERE p.user_id = $1 ORDER BY t.playlist_id, t.position, t.added_at",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count_playlists(conn: &mut Conn, user: UserId) -> Result<i64, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM music_playlists WHERE user_id = $1"#,
        user.0,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn create_playlist(
    conn: &mut Conn,
    user: UserId,
    name: &str,
) -> Result<PlaylistRow, DbError> {
    Ok(sqlx::query_as!(
        PlaylistRow,
        "INSERT INTO music_playlists (user_id, name) VALUES ($1, $2)
         RETURNING id, name, created_at, updated_at",
        user.0,
        name,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn get_playlist(
    conn: &mut Conn,
    user: UserId,
    id: Uuid,
) -> Result<Option<PlaylistRow>, DbError> {
    Ok(sqlx::query_as!(
        PlaylistRow,
        "SELECT id, name, created_at, updated_at FROM music_playlists
         WHERE user_id = $1 AND id = $2",
        user.0,
        id,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn rename_playlist(
    conn: &mut Conn,
    user: UserId,
    id: Uuid,
    name: &str,
) -> Result<Option<PlaylistRow>, DbError> {
    Ok(sqlx::query_as!(
        PlaylistRow,
        "UPDATE music_playlists SET name = $3 WHERE user_id = $1 AND id = $2
         RETURNING id, name, created_at, updated_at",
        user.0,
        id,
        name,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn delete_playlist(conn: &mut Conn, user: UserId, id: Uuid) -> Result<bool, DbError> {
    let result = sqlx::query!(
        "DELETE FROM music_playlists WHERE user_id = $1 AND id = $2",
        user.0,
        id,
    )
    .execute(conn)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn playlist_track_ids(
    conn: &mut Conn,
    playlist_id: Uuid,
) -> Result<Vec<String>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT track_id FROM music_playlist_tracks WHERE playlist_id = $1 ORDER BY position, added_at",
        playlist_id,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn replace_playlist_tracks(
    conn: &mut Conn,
    playlist_id: Uuid,
    track_ids: &[String],
) -> Result<(), DbError> {
    sqlx::query!(
        "WITH gone AS (
             DELETE FROM music_playlist_tracks WHERE playlist_id = $1 AND NOT (track_id = ANY($2))
         ),
         firsts AS (
             SELECT track_id, MIN(ord) AS ord
             FROM UNNEST($2::text[]) WITH ORDINALITY AS t(track_id, ord)
             GROUP BY track_id
         ),
         placed AS (
             INSERT INTO music_playlist_tracks (playlist_id, track_id, position)
             SELECT $1, track_id, (ROW_NUMBER() OVER (ORDER BY ord) - 1)::int4 FROM firsts
             ON CONFLICT (playlist_id, track_id) DO UPDATE SET position = EXCLUDED.position
         )
         UPDATE music_playlists SET updated_at = now() WHERE id = $1",
        playlist_id,
        track_ids,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn append_playlist_tracks(
    conn: &mut Conn,
    playlist_id: Uuid,
    track_ids: &[String],
) -> Result<(), DbError> {
    sqlx::query!(
        "WITH next AS (
             SELECT COALESCE(MAX(position) + 1, 0) AS first
             FROM music_playlist_tracks WHERE playlist_id = $1
         ),
         added AS (
             INSERT INTO music_playlist_tracks (playlist_id, track_id, position)
             SELECT $1, t.track_id, (next.first + t.ord - 1)::int4
             FROM UNNEST($2::text[]) WITH ORDINALITY AS t(track_id, ord), next
             ON CONFLICT (playlist_id, track_id) DO NOTHING
         )
         UPDATE music_playlists SET updated_at = now() WHERE id = $1",
        playlist_id,
        track_ids,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn remove_playlist_track(
    conn: &mut Conn,
    playlist_id: Uuid,
    track_id: &str,
) -> Result<bool, DbError> {
    Ok(sqlx::query_scalar!(
        r#"WITH gone AS (
               DELETE FROM music_playlist_tracks WHERE playlist_id = $1 AND track_id = $2 RETURNING 1
           ),
           touched AS (
               UPDATE music_playlists SET updated_at = now()
               WHERE id = $1 AND EXISTS (SELECT 1 FROM gone)
           )
           SELECT EXISTS (SELECT 1 FROM gone) AS "removed!""#,
        playlist_id,
        track_id,
    )
    .fetch_one(conn)
    .await?)
}
