//! Community music stats (`music_stat_events`, `music_track_stats`,
//! `music_stat_dedupe`; see v037). Every function takes a connection and the
//! moment to act at, so a caller or a test runs it inside its own transaction
//! and can step the clock across a dedupe window.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;
use uuid::Uuid;

use crate::database::models::music_stats::{
    MusicListenerEntry, MusicListenerSort, MusicStatKind, MusicTopTrack, MusicTrackCountsRow,
};
use crate::database::models::profile_layout::ProfileHeaderField;

/// Claims the dedupe slot for `(kind, client, track_id)` at `at`. True when
/// the call counts: the client never counted for the track, or last counted
/// at least `window_secs` before `at`. The window runs from the last COUNTED
/// call, so a client calling nonstop counts once per window. One statement,
/// so two racing calls cannot both count.
pub async fn claim_window(
    conn: &mut PgConnection,
    kind: MusicStatKind,
    client: &str,
    track_id: &str,
    at: DateTime<Utc>,
    window_secs: f64,
) -> Result<bool, sqlx::Error> {
    let claimed = sqlx::query_scalar::<_, bool>(
        "INSERT INTO music_stat_dedupe (kind, client, track_id, counted_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (kind, client, track_id) DO UPDATE SET counted_at = EXCLUDED.counted_at
         WHERE music_stat_dedupe.counted_at <= EXCLUDED.counted_at - make_interval(secs => $5)
         RETURNING true",
    )
    .bind(kind.as_str())
    .bind(client)
    .bind(track_id)
    .bind(at)
    .bind(window_secs)
    .fetch_optional(conn)
    .await?;
    Ok(claimed.is_some())
}

/// Records one counted event and raises the track's all-time counter, in
/// one statement.
pub async fn insert_event(
    conn: &mut PgConnection,
    kind: MusicStatKind,
    track_id: &str,
    user_id: Option<Uuid>,
    at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let (listens, downloads) = match kind {
        MusicStatKind::Listen => (1_i64, 0_i64),
        MusicStatKind::Download => (0, 1),
    };
    sqlx::query(
        "WITH ev AS (
             INSERT INTO music_stat_events (kind, track_id, user_id, created_at)
             VALUES ($1, $2, $3, $4)
         )
         INSERT INTO music_track_stats (track_id, listens, downloads, updated_at)
         VALUES ($2, $5, $6, $4)
         ON CONFLICT (track_id) DO UPDATE SET
             listens = music_track_stats.listens + EXCLUDED.listens,
             downloads = music_track_stats.downloads + EXCLUDED.downloads,
             updated_at = EXCLUDED.updated_at",
    )
    .bind(kind.as_str())
    .bind(track_id)
    .bind(user_id)
    .bind(at)
    .bind(listens)
    .bind(downloads)
    .execute(conn)
    .await?;
    Ok(())
}

/// Drops dedupe rows that counted before `before`: past every window, they
/// decide nothing.
pub async fn sweep_dedupe(
    conn: &mut PgConnection,
    before: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM music_stat_dedupe WHERE counted_at < $1")
        .bind(before)
        .execute(conn)
        .await?;
    Ok(result.rows_affected())
}

/// Whether the Monster Siren catalogue cache lists the song.
pub async fn siren_song_exists(conn: &mut PgConnection, cid: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM siren_songs WHERE cid = $1)")
        .bind(cid)
        .fetch_one(conn)
        .await
}

/// Every track's all-time counts.
pub async fn all_counts(conn: &mut PgConnection) -> Result<Vec<MusicTrackCountsRow>, sqlx::Error> {
    sqlx::query_as::<_, MusicTrackCountsRow>(
        "SELECT track_id, listens, downloads FROM music_track_stats
         WHERE listens > 0 OR downloads > 0",
    )
    .fetch_all(conn)
    .await
}

/// The most listened tracks since `since`, or of all time when `since` is
/// `None` (read from the counters, which the events sum to).
pub async fn top_tracks(
    conn: &mut PgConnection,
    since: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<MusicTopTrack>, sqlx::Error> {
    match since {
        Some(since) => {
            sqlx::query_as::<_, MusicTopTrack>(
                "SELECT rank() OVER (ORDER BY count(*) DESC) AS rank, track_id, count(*) AS listens
                 FROM music_stat_events
                 WHERE kind = 'listen' AND created_at >= $1
                 GROUP BY track_id
                 ORDER BY listens DESC, track_id
                 LIMIT $2",
            )
            .bind(since)
            .bind(limit)
            .fetch_all(conn)
            .await
        }
        None => {
            sqlx::query_as::<_, MusicTopTrack>(
                "SELECT rank() OVER (ORDER BY listens DESC) AS rank, track_id, listens
                 FROM music_track_stats
                 WHERE listens > 0
                 ORDER BY listens DESC, track_id
                 LIMIT $1",
            )
            .bind(limit)
            .fetch_all(conn)
            .await
        }
    }
}

/// The same gate as the score and inventory leaderboards: only public
/// profiles are ranked. The profile has no music tab to hide. Expects the
/// `users` alias `u`.
const VISIBLE: &str =
    "EXISTS (SELECT 1 FROM user_settings us WHERE us.user_id = u.id AND us.public_profile)";

/// The most one play adds to a listener's time, in seconds: 20 minutes.
/// `listened_seconds` is the client's to report (and to raise later, up to a
/// day per play), and the server knows no track's length, so a stuck or
/// scripted tab would otherwise top the time ranking. Tracks run 2 to 5
/// minutes; a looped in-game track past 20 minutes counts 20.
const PLAY_SECONDS_MAX: f32 = 1_200.0;

/// `listeners`: every visible signed-in listener since `$1` (and of track
/// `$2` when `per_track`), with counted listens from the events and seconds
/// heard from the play history. A listener with plays but no counted listen
/// (or the reverse) is still a row, with 0 for the missing side.
///
/// Seconds count only plays of tracks the catalogue knows: a history row is
/// stored whatever its id, but only a known track ever counts a listen, so a
/// `music_track_stats` row marks a known one. Each play counts at most
/// [`PLAY_SECONDS_MAX`].
fn listeners_cte(per_track: bool) -> String {
    let track = if per_track { " AND track_id = $2" } else { "" };
    format!(
        r"
        WITH ev AS (
            SELECT user_id, count(*) AS listens
            FROM music_stat_events
            WHERE kind = 'listen' AND user_id IS NOT NULL AND created_at >= $1{track}
            GROUP BY user_id
        ),
        hist AS (
            SELECT h.user_id, sum(least(h.listened_seconds, {PLAY_SECONDS_MAX}))::float8 AS seconds
            FROM music_history h
            WHERE h.played_at >= $1{track}
              AND EXISTS (SELECT 1 FROM music_track_stats ts WHERE ts.track_id = h.track_id)
            GROUP BY h.user_id
        ),
        listeners AS (
            SELECT coalesce(ev.user_id, hist.user_id) AS user_id,
                   coalesce(ev.listens, 0) AS listens,
                   coalesce(hist.seconds, 0)::float8 AS seconds
            FROM ev FULL JOIN hist ON hist.user_id = ev.user_id
        )"
    )
}

/// One page of listeners since `since`, site-wide or of one track, ranked by
/// `sort` with the other metric breaking ties. Pass `UNIX_EPOCH` as `since`
/// for all time: a bound keeps the time index usable.
pub async fn listeners(
    conn: &mut PgConnection,
    since: DateTime<Utc>,
    track_id: Option<&str>,
    sort: MusicListenerSort,
    limit: i64,
    offset: i64,
) -> Result<Vec<MusicListenerEntry>, sqlx::Error> {
    let order = match sort {
        MusicListenerSort::Listens => "l.listens DESC, l.seconds DESC",
        MusicListenerSort::Time => "l.seconds DESC, l.listens DESC",
    };
    let cte = listeners_cte(track_id.is_some());
    // `$2` is the track when there is one, so the page binds follow it.
    let limit_idx = if track_id.is_some() { 3 } else { 2 };
    let sql = format!(
        r"
        {cte}
        SELECT rank() OVER (ORDER BY {order}) AS rank,
               u.uid, u.nickname, u.nick_number, {level} AS level, u.avatar_id,
               s.code AS server, l.listens, l.seconds AS listened_seconds
        FROM listeners l
        JOIN users u ON u.id = l.user_id
        JOIN servers s ON s.id = u.server_id
        WHERE {VISIBLE}
        ORDER BY {order}, u.uid
        LIMIT ${limit_idx} OFFSET ${offset_idx}
        ",
        offset_idx = limit_idx + 1,
        // A level the player hid from their profile header is not shown here.
        level = ProfileHeaderField::Level.shown_sql("u.id", "u.level"),
    );
    let mut q = sqlx::query_as::<_, MusicListenerEntry>(&sql).bind(since);
    if let Some(track) = track_id {
        q = q.bind(track);
    }
    q.bind(limit).bind(offset).fetch_all(conn).await
}

/// How many rows `listeners` ranks for the same window and track.
pub async fn count_listeners(
    conn: &mut PgConnection,
    since: DateTime<Utc>,
    track_id: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let cte = listeners_cte(track_id.is_some());
    let sql = format!(
        r"
        {cte}
        SELECT count(*) FROM listeners l
        JOIN users u ON u.id = l.user_id
        WHERE {VISIBLE}
        "
    );
    let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(since);
    if let Some(track) = track_id {
        q = q.bind(track);
    }
    q.fetch_one(conn).await
}
