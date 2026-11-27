#![allow(clippy::unwrap_used, clippy::float_cmp)]

mod common;

use chrono::{DateTime, Duration, SubsecRound, Utc};
use myrtle_db::Conn;
use myrtle_db::repos::music;
use myrtle_db::repos::music_stats::{self, ListenerSort, StatKind};
use myrtle_types::{Server, UserId};
use sqlx::PgPool;

const LISTEN_WINDOW: f64 = 1_800.0;
const DOWNLOAD_WINDOW: f64 = 86_400.0;

/// Claims the window and records the event when it counts
async fn record(
    conn: &mut Conn,
    kind: StatKind,
    track_id: &str,
    client: &str,
    user: Option<UserId>,
    at: DateTime<Utc>,
) -> bool {
    let window = match kind {
        StatKind::Listen => LISTEN_WINDOW,
        StatKind::Download => DOWNLOAD_WINDOW,
    };
    let counted = music_stats::claim_window(conn, kind, client, track_id, at, window)
        .await
        .unwrap();
    if counted {
        music_stats::insert_event(conn, kind, track_id, user, at)
            .await
            .unwrap();
    }
    counted
}

async fn listen(conn: &mut Conn, track_id: &str, user: Option<UserId>, at: DateTime<Utc>) {
    music_stats::insert_event(conn, StatKind::Listen, track_id, user, at)
        .await
        .unwrap();
}

async fn settings(conn: &mut Conn, user: UserId, public: bool) {
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, $2)")
        .bind(user.0)
        .bind(public)
        .execute(conn)
        .await
        .unwrap();
}

async fn counts(conn: &mut Conn, track_id: &str) -> (i64, i64) {
    sqlx::query_as("SELECT listens, downloads FROM music_track_stats WHERE track_id = $1")
        .bind(track_id)
        .fetch_optional(conn)
        .await
        .unwrap()
        .unwrap_or((0, 0))
}

async fn events(conn: &mut Conn, track_id: &str, kind: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND kind = $2")
        .bind(track_id)
        .bind(kind)
        .fetch_one(conn)
        .await
        .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_dedupe_window_counts_each_client_once_per_window(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let account = common::user(&mut conn, 1).await;
    let signed_in = format!("u:{}", account.0);
    let t0 = Utc::now().trunc_subsecs(0);
    let window = Duration::minutes(30);
    let day = Duration::hours(24);
    let (song, other_song) = ("game:song", "game:other");
    let (anon, neighbour) = ("a:anon", "a:neighbour");
    let (l, d) = (StatKind::Listen, StatKind::Download);

    assert!(record(&mut conn, l, song, anon, None, t0).await);
    assert!(!record(&mut conn, l, song, anon, None, t0 + Duration::seconds(5)).await);
    assert!(
        !record(
            &mut conn,
            l,
            song,
            anon,
            None,
            t0 + window - Duration::seconds(1)
        )
        .await
    );
    assert!(record(&mut conn, l, song, anon, None, t0 + window).await);
    assert!(
        !record(
            &mut conn,
            l,
            song,
            anon,
            None,
            t0 + window + Duration::minutes(29)
        )
        .await
    );
    assert!(record(&mut conn, l, other_song, anon, None, t0).await);
    assert!(record(&mut conn, l, song, neighbour, None, t0).await);
    assert!(record(&mut conn, l, song, &signed_in, Some(account), t0).await);
    assert!(
        !record(
            &mut conn,
            l,
            song,
            &signed_in,
            Some(account),
            t0 + Duration::seconds(2)
        )
        .await
    );

    assert!(record(&mut conn, d, song, anon, None, t0).await);
    assert!(
        !record(
            &mut conn,
            d,
            song,
            anon,
            None,
            t0 + day - Duration::minutes(1)
        )
        .await
    );
    assert!(record(&mut conn, d, song, anon, None, t0 + day).await);

    assert_eq!(events(&mut conn, song, "listen").await, 4);
    assert_eq!(events(&mut conn, song, "download").await, 2);
    assert_eq!(counts(&mut conn, song).await, (4, 2));
    assert_eq!(counts(&mut conn, other_song).await, (1, 0));
    let user_rows: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM music_stat_events WHERE track_id = $1 AND user_id = $2",
    )
    .bind(song)
    .bind(account.0)
    .fetch_one(&mut *conn)
    .await
    .unwrap();
    assert_eq!(user_rows, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn the_sweep_drops_only_rows_counted_before_the_cutoff(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let t0 = Utc::now().trunc_subsecs(0);
    let day = Duration::hours(24);
    record(&mut conn, StatKind::Listen, "game:a", "a:1", None, t0).await;
    record(&mut conn, StatKind::Download, "game:a", "a:1", None, t0).await;
    record(
        &mut conn,
        StatKind::Download,
        "game:a",
        "a:1",
        None,
        t0 + day,
    )
    .await;

    assert_eq!(
        music_stats::sweep_dedupe(&mut conn, t0 + day)
            .await
            .unwrap(),
        1
    );
    let left: Vec<String> = sqlx::query_scalar("SELECT kind FROM music_stat_dedupe ORDER BY kind")
        .fetch_all(&mut *conn)
        .await
        .unwrap();
    assert_eq!(left, ["download"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_siren_song_exists_only_once_the_catalogue_lists_it(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    sqlx::raw_sql(
        "INSERT INTO siren_albums (cid, name, cover_url, list_order) VALUES ('8913', 'a', 'u', 0);
         INSERT INTO siren_songs (cid, album_cid, name) VALUES ('048756', '8913', 's')",
    )
    .execute(&mut *conn)
    .await
    .unwrap();

    assert!(
        music_stats::siren_song_exists(&mut conn, "048756")
            .await
            .unwrap()
    );
    assert!(
        !music_stats::siren_song_exists(&mut conn, "8913")
            .await
            .unwrap()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn all_counts_skips_a_track_with_neither_count(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    listen(&mut conn, "game:heard", None, now).await;
    music_stats::insert_event(&mut conn, StatKind::Download, "game:saved", None, now)
        .await
        .unwrap();
    sqlx::query("INSERT INTO music_track_stats (track_id) VALUES ('game:none')")
        .execute(&mut *conn)
        .await
        .unwrap();

    let mut rows: Vec<(String, i64, i64)> = music_stats::all_counts(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.track_id, r.listens, r.downloads))
        .collect();
    rows.sort();
    assert_eq!(
        rows,
        [
            ("game:heard".to_owned(), 1, 0),
            ("game:saved".to_owned(), 0, 1)
        ]
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn top_tracks_follow_the_window(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let (hot, steady, old) = ("game:hot", "game:steady", "game:old");
    for _ in 0..3 {
        listen(&mut conn, hot, None, now - Duration::days(1)).await;
    }
    for d in [2, 10, 20, 25] {
        listen(&mut conn, steady, None, now - Duration::days(d)).await;
    }
    for _ in 0..6 {
        listen(&mut conn, old, None, now - Duration::days(90)).await;
    }
    music_stats::insert_event(&mut conn, StatKind::Download, "game:saved", None, now)
        .await
        .unwrap();

    let top = |rows: Vec<music_stats::TopTrackRow>| {
        rows.into_iter()
            .map(|r| (r.rank, r.track_id, r.listens))
            .collect::<Vec<_>>()
    };
    let week = top(
        music_stats::top_tracks(&mut conn, Some(now - Duration::days(7)), 100)
            .await
            .unwrap(),
    );
    assert_eq!(week, [(1, hot.to_owned(), 3), (2, steady.to_owned(), 1)]);
    let month = top(
        music_stats::top_tracks(&mut conn, Some(now - Duration::days(30)), 100)
            .await
            .unwrap(),
    );
    assert_eq!(month, [(1, steady.to_owned(), 4), (2, hot.to_owned(), 3)]);
    let all = top(music_stats::top_tracks(&mut conn, None, 100).await.unwrap());
    assert_eq!(
        all,
        [
            (1, old.to_owned(), 6),
            (2, steady.to_owned(), 4),
            (3, hot.to_owned(), 3)
        ]
    );
    let first = top(music_stats::top_tracks(&mut conn, None, 1).await.unwrap());
    assert_eq!(first, [(1, old.to_owned(), 6)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn tied_tracks_share_a_rank_and_order_by_id(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    for track in ["game:b", "game:a", "game:c", "game:c"] {
        listen(&mut conn, track, None, now).await;
    }

    for since in [Some(now - Duration::days(1)), None] {
        let rows: Vec<(i64, String)> = music_stats::top_tracks(&mut conn, since, 10)
            .await
            .unwrap()
            .into_iter()
            .map(|r| (r.rank, r.track_id))
            .collect();
        assert_eq!(
            rows,
            [
                (1, "game:c".to_owned()),
                (2, "game:a".to_owned()),
                (2, "game:b".to_owned())
            ]
        );
    }
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn listener_rankings_gate_on_public_profiles(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let (song, other) = ("game:song", "game:other");
    let fan = common::user(&mut conn, 1).await;
    let casual = common::user(&mut conn, 2).await;
    let hidden = common::user(&mut conn, 3).await;
    settings(&mut conn, fan, true).await;
    settings(&mut conn, casual, true).await;
    settings(&mut conn, hidden, false).await;

    for h in [2, 1, 0] {
        listen(&mut conn, song, Some(fan), now - Duration::hours(h)).await;
        music::insert_play(&mut conn, fan, song, 40.0)
            .await
            .unwrap();
    }
    listen(&mut conn, other, Some(casual), now).await;
    // One stuck play claiming 50 minutes counts 20.
    music::insert_play(&mut conn, casual, other, 3_000.0)
        .await
        .unwrap();
    // A track no counter knows adds no time.
    music::insert_play(&mut conn, casual, "game:unknown", 600.0)
        .await
        .unwrap();
    listen(&mut conn, song, Some(casual), now - Duration::days(40)).await;
    listen(&mut conn, song, None, now).await;
    for h in [4, 3, 2, 1, 0] {
        listen(&mut conn, song, Some(hidden), now - Duration::hours(h)).await;
    }
    music::insert_play(&mut conn, hidden, song, 9_000.0)
        .await
        .unwrap();

    let week = now - Duration::days(7);
    let rows = |rows: Vec<music_stats::ListenerRow>| {
        rows.into_iter()
            .map(|r| (r.rank, r.uid, r.listens, r.listened_seconds))
            .collect::<Vec<_>>()
    };
    let by_listens = rows(
        music_stats::listeners(&mut conn, week, None, ListenerSort::Listens, 100, 0)
            .await
            .unwrap(),
    );
    assert_eq!(
        by_listens,
        [
            (1, "1".to_owned(), 3, 120.0),
            (2, "2".to_owned(), 1, 1_200.0)
        ]
    );
    let by_time = rows(
        music_stats::listeners(&mut conn, week, None, ListenerSort::Time, 100, 0)
            .await
            .unwrap(),
    );
    assert_eq!(
        by_time,
        [
            (1, "2".to_owned(), 1, 1_200.0),
            (2, "1".to_owned(), 3, 120.0)
        ]
    );

    let song_week = rows(
        music_stats::listeners(&mut conn, week, Some(song), ListenerSort::Listens, 100, 0)
            .await
            .unwrap(),
    );
    assert_eq!(song_week, [(1, "1".to_owned(), 3, 120.0)]);
    let song_all = rows(
        music_stats::listeners(
            &mut conn,
            DateTime::UNIX_EPOCH,
            Some(song),
            ListenerSort::Listens,
            100,
            0,
        )
        .await
        .unwrap(),
    );
    assert_eq!(
        song_all,
        [(1, "1".to_owned(), 3, 120.0), (2, "2".to_owned(), 1, 0.0)]
    );
    assert_eq!(
        music_stats::count_listeners(&mut conn, DateTime::UNIX_EPOCH, Some(song))
            .await
            .unwrap(),
        2
    );
    assert_eq!(
        music_stats::count_listeners(&mut conn, week, Some(song))
            .await
            .unwrap(),
        1
    );

    let second = rows(
        music_stats::listeners(&mut conn, week, None, ListenerSort::Listens, 1, 1)
            .await
            .unwrap(),
    );
    assert_eq!(second, [(2, "2".to_owned(), 1, 1_200.0)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_listener_row_carries_identity_and_masks_a_hidden_level(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    let shown = common::user(&mut conn, 1).await;
    let masked = common::user(&mut conn, 2).await;
    settings(&mut conn, shown, true).await;
    settings(&mut conn, masked, true).await;
    sqlx::query("UPDATE users SET level = 90, nick_number = '1234' WHERE id = ANY($1)")
        .bind(vec![shown.0, masked.0])
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query(
        r#"UPDATE user_settings SET profile_layout = '{"header": {"hidden": ["level"]}}'
           WHERE user_id = $1"#,
    )
    .bind(masked.0)
    .execute(&mut *conn)
    .await
    .unwrap();
    listen(&mut conn, "game:a", Some(shown), now).await;
    listen(&mut conn, "game:a", Some(shown), now).await;
    listen(&mut conn, "game:a", Some(masked), now).await;

    let rows = music_stats::listeners(
        &mut conn,
        DateTime::UNIX_EPOCH,
        None,
        ListenerSort::Listens,
        10,
        0,
    )
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].uid, "1");
    assert_eq!(rows[0].nickname.as_deref(), Some("user1"));
    assert_eq!(rows[0].nick_number.as_deref(), Some("1234"));
    assert_eq!(rows[0].server, Server::EN.as_str());
    assert_eq!(rows[0].level, Some(90));
    assert_eq!(rows[1].uid, "2");
    assert_eq!(rows[1].level, None);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn listener_ranks_break_ties_on_the_other_measure_and_share_a_rank_on_both(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let now = Utc::now();
    // (uid, listens, seconds): 1 and 2 tie on listens, 1 and 5 on seconds, 3 and 4 on both.
    // Each pair puts the lower uid on the lower secondary, so a dropped key reorders.
    for (n, listens, seconds) in [
        (1, 2, 50.0),
        (2, 2, 100.0),
        (3, 1, 300.0),
        (4, 1, 300.0),
        (5, 3, 50.0),
    ] {
        let user = common::user(&mut conn, n).await;
        settings(&mut conn, user, true).await;
        for _ in 0..listens {
            listen(&mut conn, "game:a", Some(user), now).await;
        }
        music::insert_play(&mut conn, user, "game:a", seconds)
            .await
            .unwrap();
    }

    let ranks = |rows: Vec<music_stats::ListenerRow>| {
        rows.into_iter()
            .map(|r| (r.rank, r.uid))
            .collect::<Vec<_>>()
    };
    let expect = |rows: &[(i64, &str)]| {
        rows.iter()
            .map(|(rank, uid)| (*rank, (*uid).to_owned()))
            .collect::<Vec<_>>()
    };
    let by_listens = ranks(
        music_stats::listeners(
            &mut conn,
            DateTime::UNIX_EPOCH,
            None,
            ListenerSort::Listens,
            10,
            0,
        )
        .await
        .unwrap(),
    );
    assert_eq!(
        by_listens,
        expect(&[(1, "5"), (2, "2"), (3, "1"), (4, "3"), (4, "4")])
    );
    let by_time = ranks(
        music_stats::listeners(
            &mut conn,
            DateTime::UNIX_EPOCH,
            None,
            ListenerSort::Time,
            10,
            0,
        )
        .await
        .unwrap(),
    );
    assert_eq!(
        by_time,
        expect(&[(1, "3"), (1, "4"), (3, "2"), (4, "5"), (5, "1")])
    );
}
