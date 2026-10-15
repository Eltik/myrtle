//! Community music stats against the real schema: the dedupe window per
//! client and kind, the counters agreeing with the events, the top tracks
//! per window, and the listener rankings (site-wide and per track) with the
//! public-profile gate.
//!
//! Runs inside one transaction on the local Postgres and rolls it back, so
//! nothing persists. When the local database predates v037 the migration is
//! applied inside that same transaction, which is why the tests run one at a
//! time (concurrent transactions creating the same tables wait on each
//! other). Ignored by default:
//! `cargo test --test music_stats_storage_test -- --ignored --test-threads=1`

use std::net::IpAddr;

use backend::app::services::music::stats::{
    DOWNLOAD_WINDOW, LISTEN_WINDOW, StatClient, anonymous_client, record_on,
};
use backend::database::models::music_stats::{MusicListenerSort, MusicStatKind};
use backend::database::queries::music as music_q;
use backend::database::queries::music_stats as q;
use chrono::{DateTime, Duration, Utc};
use sqlx::{Connection, PgConnection, Postgres, Transaction};
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

async fn migrate(conn: &mut PgConnection) {
    let migrated: bool = sqlx::query_scalar("SELECT to_regclass('music_stat_events') IS NOT NULL")
        .fetch_one(&mut *conn)
        .await
        .expect("catalog");
    if !migrated {
        sqlx::raw_sql(include_str!(
            "../src/database/migrations/v037_music_stats.sql"
        ))
        .execute(&mut *conn)
        .await
        .expect("v037 applies");
    }
}

async fn connect() -> PgConnection {
    PgConnection::connect(DATABASE_URL)
        .await
        .expect("local postgres")
}

/// The transaction every test runs in and never commits, with v037 applied.
async fn begin_migrated(conn: &mut PgConnection) -> Transaction<'_, Postgres> {
    let mut tx = conn.begin().await.expect("begin");
    migrate(&mut tx).await;
    tx
}

/// A throwaway user with a settings row; `public` is their profile flag.
async fn user(conn: &mut PgConnection, public: bool) -> (Uuid, String) {
    let uid = format!("mst{}", &Uuid::new_v4().simple().to_string()[..12]);
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
            .bind(&uid)
            .fetch_one(&mut *conn)
            .await
            .expect("throwaway user");
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, $2)")
        .bind(id)
        .bind(public)
        .execute(&mut *conn)
        .await
        .expect("settings");
    (id, uid)
}

fn track() -> String {
    format!(
        "game:zz_test_{}",
        &Uuid::new_v4().simple().to_string()[..12]
    )
}

async fn counts(conn: &mut PgConnection, track_id: &str) -> (i64, i64) {
    sqlx::query_as("SELECT listens, downloads FROM music_track_stats WHERE track_id = $1")
        .bind(track_id)
        .fetch_optional(&mut *conn)
        .await
        .expect("counts")
        .unwrap_or((0, 0))
}

async fn events(conn: &mut PgConnection, track_id: &str, kind: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND kind = $2")
        .bind(track_id)
        .bind(kind)
        .fetch_one(&mut *conn)
        .await
        .expect("events")
}

fn chrono_span(d: std::time::Duration) -> Duration {
    Duration::from_std(d).expect("window fits")
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn the_dedupe_window_counts_each_client_once_per_window() {
    let mut conn = connect().await;
    let mut tx = begin_migrated(&mut conn).await;

    let t0: DateTime<Utc> = Utc::now();
    let song = track();
    let other_song = track();
    let ip: IpAddr = "203.0.113.7".parse().unwrap();
    let anon = anonymous_client("secret", ip);
    let neighbour = anonymous_client("secret", "203.0.113.8".parse().unwrap());
    let (account, _) = user(&mut tx, true).await;
    let signed_in = StatClient::User(account);
    let listen = MusicStatKind::Listen;
    let download = MusicStatKind::Download;
    let window = chrono_span(LISTEN_WINDOW);

    assert!(
        record_on(&mut tx, listen, &song, &anon, t0).await.unwrap(),
        "first listen counts"
    );
    assert!(
        !record_on(&mut tx, listen, &song, &anon, t0 + Duration::seconds(5))
            .await
            .unwrap(),
        "a replay seconds later does not"
    );
    assert!(
        !record_on(
            &mut tx,
            listen,
            &song,
            &anon,
            t0 + window - Duration::seconds(1)
        )
        .await
        .unwrap(),
        "nor one a second before the window closes"
    );
    assert!(
        record_on(&mut tx, listen, &song, &anon, t0 + window)
            .await
            .unwrap(),
        "the window has passed"
    );
    assert!(
        !record_on(
            &mut tx,
            listen,
            &song,
            &anon,
            t0 + window + Duration::minutes(29)
        )
        .await
        .unwrap(),
        "the window restarts from the last COUNTED listen"
    );
    assert!(
        record_on(&mut tx, listen, &other_song, &anon, t0)
            .await
            .unwrap(),
        "another track is its own slot"
    );
    assert!(
        record_on(&mut tx, listen, &song, &neighbour, t0)
            .await
            .unwrap(),
        "another address is its own slot"
    );

    // A signed-in play reported through the history log and the public
    // endpoint share the account's slot: once.
    assert!(
        record_on(&mut tx, listen, &song, &signed_in, t0)
            .await
            .unwrap()
    );
    assert!(
        !record_on(
            &mut tx,
            listen,
            &song,
            &signed_in,
            t0 + Duration::seconds(2)
        )
        .await
        .unwrap(),
        "the second report of the same play does not count"
    );

    // Downloads: a 24 hour window, independent of listens.
    let day = chrono_span(DOWNLOAD_WINDOW);
    assert!(
        record_on(&mut tx, download, &song, &anon, t0)
            .await
            .unwrap(),
        "a download is not a listen"
    );
    assert!(
        !record_on(
            &mut tx,
            download,
            &song,
            &anon,
            t0 + day - Duration::minutes(1)
        )
        .await
        .unwrap()
    );
    assert!(
        record_on(&mut tx, download, &song, &anon, t0 + day)
            .await
            .unwrap()
    );

    assert_eq!(
        events(&mut tx, &song, "listen").await,
        4,
        "anon x2, neighbour, account"
    );
    assert_eq!(events(&mut tx, &song, "download").await, 2);
    assert_eq!(
        counts(&mut tx, &song).await,
        (4, 2),
        "the counter is the events summed"
    );
    assert_eq!(counts(&mut tx, &other_song).await, (1, 0));

    let user_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND user_id = $2",
    )
    .bind(&song)
    .bind(account)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(user_rows, 1, "the account's listen is kept against it");
    let anon_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND user_id IS NULL",
    )
    .bind(&song)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(
        anon_rows, 5,
        "anonymous rows carry no identity: 3 listens, 2 downloads"
    );

    // The sweep drops only rows past the longest window.
    let swept = q::sweep_dedupe(&mut tx, t0 + day).await.unwrap();
    assert!(swept >= 1, "rows counted at t0 are past a day at t0 + 24h");
    let left: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM music_stat_dedupe WHERE track_id = $1 AND kind = 'download'",
    )
    .bind(&song)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(left, 1, "the download counted at t0 + 24h stays");

    tx.rollback().await.expect("rollback");
}

/// One counted listen of `track_id` at `at` from an address no other call
/// used.
async fn fresh_listen(conn: &mut PgConnection, n: &mut u32, track_id: &str, at: DateTime<Utc>) {
    *n += 1;
    let [_, _, hi, lo] = n.to_be_bytes();
    let client = anonymous_client("s", IpAddr::from([198, 51, hi, lo]));
    assert!(
        record_on(conn, MusicStatKind::Listen, track_id, &client, at)
            .await
            .unwrap()
    );
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn top_tracks_follow_the_window() {
    let mut conn = connect().await;
    let mut tx = begin_migrated(&mut conn).await;

    let now = Utc::now();
    let (hot, steady, old) = (track(), track(), track());
    // Each listen from a fresh address, so each counts.
    let mut n = 0_u32;
    for _ in 0..3 {
        fresh_listen(&mut tx, &mut n, &hot, now - Duration::days(1)).await;
    }
    for d in [2, 10, 20, 25] {
        fresh_listen(&mut tx, &mut n, &steady, now - Duration::days(d)).await;
    }
    for _ in 0..6 {
        fresh_listen(&mut tx, &mut n, &old, now - Duration::days(90)).await;
    }

    let ours = |rows: Vec<backend::database::models::music_stats::MusicTopTrack>| {
        rows.into_iter()
            .filter(|r| [&hot, &steady, &old].contains(&&r.track_id))
            .map(|r| (r.track_id, r.listens))
            .collect::<Vec<_>>()
    };
    let week = ours(
        q::top_tracks(&mut tx, Some(now - Duration::days(7)), 1000)
            .await
            .unwrap(),
    );
    assert_eq!(week, vec![(hot.clone(), 3), (steady.clone(), 1)]);
    let month = ours(
        q::top_tracks(&mut tx, Some(now - Duration::days(30)), 1000)
            .await
            .unwrap(),
    );
    assert_eq!(month, vec![(steady.clone(), 4), (hot.clone(), 3)]);
    let all = ours(q::top_tracks(&mut tx, None, 1000).await.unwrap());
    assert_eq!(
        all,
        vec![(old.clone(), 6), (steady.clone(), 4), (hot.clone(), 3)]
    );

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn listener_rankings_gate_on_public_profiles() {
    let mut conn = connect().await;
    let mut tx = begin_migrated(&mut conn).await;

    let now = Utc::now();
    let (song, other) = (track(), track());
    let (fan, fan_uid) = user(&mut tx, true).await;
    let (casual, casual_uid) = user(&mut tx, true).await;
    let (hidden, hidden_uid) = user(&mut tx, false).await;
    let listen = MusicStatKind::Listen;

    // fan: 3 counted listens of `song` this week, little time.
    // Oldest first: the window runs forward from the last counted listen.
    for h in [2, 1, 0] {
        let at = now - Duration::hours(h);
        record_on(&mut tx, listen, &song, &StatClient::User(fan), at)
            .await
            .unwrap();
        music_q::insert_play(&mut tx, fan, &song, 40.0)
            .await
            .unwrap();
    }
    // casual: 1 counted listen of `other`, long sessions.
    record_on(&mut tx, listen, &other, &StatClient::User(casual), now)
        .await
        .unwrap();
    // A stuck tab: one play claiming 50 minutes counts 20.
    music_q::insert_play(&mut tx, casual, &other, 3_000.0)
        .await
        .unwrap();
    // A play of a track the catalogue never counted adds no time.
    music_q::insert_play(&mut tx, casual, &track(), 600.0)
        .await
        .unwrap();
    // casual heard `song` too, 40 days ago: outside the month.
    record_on(
        &mut tx,
        listen,
        &song,
        &StatClient::User(casual),
        now - Duration::days(40),
    )
    .await
    .unwrap();
    // hidden: the most of everything, but a private profile.
    for h in [4, 3, 2, 1, 0] {
        record_on(
            &mut tx,
            listen,
            &song,
            &StatClient::User(hidden),
            now - Duration::hours(h),
        )
        .await
        .unwrap();
    }
    music_q::insert_play(&mut tx, hidden, &song, 9_000.0)
        .await
        .unwrap();

    let week = now - Duration::days(7);
    let ours = |rows: Vec<backend::database::models::music_stats::MusicListenerEntry>| {
        rows.into_iter()
            .filter(|r| [&fan_uid, &casual_uid, &hidden_uid].contains(&&r.uid))
            .map(|r| (r.uid, r.listens, r.listened_seconds))
            .collect::<Vec<_>>()
    };

    let by_listens = ours(
        q::listeners(&mut tx, week, None, MusicListenerSort::Listens, 1000, 0)
            .await
            .unwrap(),
    );
    assert_eq!(
        by_listens,
        vec![
            (fan_uid.clone(), 3, 120.0),
            (casual_uid.clone(), 1, 1_200.0)
        ],
        "the private profile is not ranked"
    );
    let by_time = ours(
        q::listeners(&mut tx, week, None, MusicListenerSort::Time, 1000, 0)
            .await
            .unwrap(),
    );
    assert_eq!(by_time[0].0, casual_uid, "time ranks by seconds heard");

    let song_week = ours(
        q::listeners(
            &mut tx,
            week,
            Some(&song),
            MusicListenerSort::Listens,
            1000,
            0,
        )
        .await
        .unwrap(),
    );
    assert_eq!(
        song_week,
        vec![(fan_uid.clone(), 3, 120.0)],
        "casual's listen is 40 days old"
    );
    let song_all = ours(
        q::listeners(
            &mut tx,
            DateTime::UNIX_EPOCH,
            Some(&song),
            MusicListenerSort::Listens,
            1000,
            0,
        )
        .await
        .unwrap(),
    );
    assert_eq!(
        song_all,
        vec![(fan_uid.clone(), 3, 120.0), (casual_uid.clone(), 1, 0.0)]
    );

    let count = q::count_listeners(&mut tx, DateTime::UNIX_EPOCH, Some(&song))
        .await
        .unwrap();
    let page = q::listeners(
        &mut tx,
        DateTime::UNIX_EPOCH,
        Some(&song),
        MusicListenerSort::Listens,
        1000,
        0,
    )
    .await
    .unwrap();
    assert_eq!(
        count,
        i64::try_from(page.len()).unwrap(),
        "the total is the ranked population"
    );

    // Ranks close over the hidden listener: the top visible row is rank 1.
    let top = q::listeners(&mut tx, week, Some(&song), MusicListenerSort::Listens, 1, 0)
        .await
        .unwrap();
    assert_eq!((top[0].uid.as_str(), top[0].rank), (fan_uid.as_str(), 1));

    // Deleting an account keeps its listens as anonymous ones.
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(fan)
        .execute(&mut *tx)
        .await
        .unwrap();
    let left: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND user_id IS NULL",
    )
    .bind(&song)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    assert_eq!(left, 3);

    tx.rollback().await.expect("rollback");
}
