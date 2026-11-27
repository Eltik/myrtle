#![allow(clippy::unwrap_used, clippy::float_cmp)]

mod common;

use myrtle_db::Conn;
use myrtle_db::repos::music::{self, AlbumUpsert, SongListing, SongUpsert};
use myrtle_types::UserId;
use sqlx::PgPool;
use uuid::Uuid;

fn ids(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

const fn album<'a>(cid: &'a str, list_order: i32, artists: &'a [String]) -> AlbumUpsert<'a> {
    AlbumUpsert {
        cid,
        name: "昨日海OST",
        intro: "",
        belong: "",
        cover_url: "https://web.hycdn.cn/siren/pic/a.jpg",
        cover_de_url: None,
        artists,
        list_order,
    }
}

const fn song<'a>(
    cid: &'a str,
    album_cid: &'a str,
    lyric_url: Option<&'a str>,
    artists: &'a [String],
) -> SongUpsert<'a> {
    SongUpsert {
        cid,
        album_cid,
        name: "Song of the Nameless",
        artists,
        lyric_url,
        mv_url: None,
        mv_cover_url: None,
        track_order: 4,
    }
}

async fn song_row(conn: &mut Conn, cid: &str) -> music::SongRow {
    music::list_songs(conn)
        .await
        .unwrap()
        .into_iter()
        .find(|s| s.cid == cid)
        .unwrap()
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn albums_list_in_label_order_and_an_upsert_refreshes_the_row(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["塞壬唱片-MSR"]);

    music::upsert_album(&mut conn, album("8914", 1, &artists))
        .await
        .unwrap();
    music::upsert_album(&mut conn, album("8913", 0, &artists))
        .await
        .unwrap();
    music::upsert_album(
        &mut conn,
        AlbumUpsert {
            name: "Renamed",
            intro: "intro",
            cover_de_url: Some("https://web.hycdn.cn/siren/pic/b.jpg"),
            ..album("8914", 1, &artists)
        },
    )
    .await
    .unwrap();

    let rows = music::list_albums(&mut conn).await.unwrap();
    let cids: Vec<&str> = rows.iter().map(|a| a.cid.as_str()).collect();
    assert_eq!(cids, ["8913", "8914"]);
    assert_eq!(rows[1].name, "Renamed");
    assert_eq!(rows[1].intro, "intro");
    assert_eq!(
        rows[1].cover_de_url.as_deref(),
        Some("https://web.hycdn.cn/siren/pic/b.jpg")
    );
    assert_eq!(rows[1].artists, artists);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_song_keeps_its_lyrics_until_the_lyric_url_moves(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["塞壬唱片-MSR"]);
    let x = Some("https://web.hycdn.cn/siren/lyric/x.lrc");
    music::upsert_album(&mut conn, album("8913", 0, &artists))
        .await
        .unwrap();
    music::upsert_song(&mut conn, song("048756", "8913", x, &artists))
        .await
        .unwrap();
    assert_eq!(music::lyric_text(&mut conn, "048756").await.unwrap(), None);
    assert_eq!(music::lyric_text(&mut conn, "missing").await.unwrap(), None);

    music::set_lyric_text(&mut conn, "048756", "[00:01.00]hello")
        .await
        .unwrap();
    assert!(song_row(&mut conn, "048756").await.has_lyric_text);
    music::upsert_song(&mut conn, song("048756", "8913", x, &artists))
        .await
        .unwrap();
    assert_eq!(
        music::lyric_text(&mut conn, "048756")
            .await
            .unwrap()
            .as_deref(),
        Some("[00:01.00]hello")
    );

    music::upsert_song(
        &mut conn,
        song(
            "048756",
            "8913",
            Some("https://web.hycdn.cn/siren/lyric/y.lrc"),
            &artists,
        ),
    )
    .await
    .unwrap();
    assert_eq!(music::lyric_text(&mut conn, "048756").await.unwrap(), None);
    assert!(!song_row(&mut conn, "048756").await.has_lyric_text);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn songs_list_by_album_then_track_order(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["a"]);
    for cid in ["8913", "8914"] {
        music::upsert_album(&mut conn, album(cid, 0, &artists))
            .await
            .unwrap();
    }
    for (cid, album_cid, order) in [("s3", "8914", 0), ("s2", "8913", 2), ("s1", "8913", 1)] {
        music::upsert_song(
            &mut conn,
            SongUpsert {
                track_order: order,
                ..song(cid, album_cid, None, &artists)
            },
        )
        .await
        .unwrap();
    }

    let cids: Vec<String> = music::list_songs(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.cid)
        .collect();
    assert_eq!(cids, ["s1", "s2", "s3"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_listing_refresh_keeps_the_detail_fields_and_an_absent_order(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["塞壬唱片-MSR"]);
    for cid in ["8913", "8914"] {
        music::upsert_album(&mut conn, album(cid, 0, &artists))
            .await
            .unwrap();
    }
    music::upsert_song(
        &mut conn,
        SongUpsert {
            mv_url: Some("https://web.hycdn.cn/siren/mv/x.mp4"),
            ..song(
                "048756",
                "8913",
                Some("https://web.hycdn.cn/siren/lyric/x.lrc"),
                &artists,
            )
        },
    )
    .await
    .unwrap();

    let renamed = ids(&["Emperor"]);
    let listing = SongListing {
        cid: "048756",
        album_cid: "8914",
        name: "Song of the Named",
        artists: &renamed,
        track_order: None,
    };
    music::update_song_listing(&mut conn, listing)
        .await
        .unwrap();
    let row = song_row(&mut conn, "048756").await;
    assert_eq!(
        (row.name.as_str(), row.album_cid.as_str(), &row.artists),
        ("Song of the Named", "8914", &renamed)
    );
    assert_eq!(row.track_order, 4);
    assert_eq!(
        row.lyric_url.as_deref(),
        Some("https://web.hycdn.cn/siren/lyric/x.lrc")
    );
    assert!(row.mv_url.is_some());

    music::update_song_listing(
        &mut conn,
        SongListing {
            track_order: Some(1),
            ..listing
        },
    )
    .await
    .unwrap();
    assert_eq!(song_row(&mut conn, "048756").await.track_order, 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn prune_drops_unlisted_songs_and_albums_and_counts_each(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["a"]);
    for cid in ["kept", "gone"] {
        music::upsert_album(&mut conn, album(cid, 0, &artists))
            .await
            .unwrap();
    }
    for (cid, album_cid) in [("a1", "kept"), ("a2", "kept"), ("b1", "gone")] {
        music::upsert_song(&mut conn, song(cid, album_cid, None, &artists))
            .await
            .unwrap();
    }

    let pruned = music::prune_unlisted(&mut conn, &ids(&["kept"]), &ids(&["a1"]))
        .await
        .unwrap();
    assert_eq!(pruned, (1, 2));
    let albums: Vec<String> = music::list_albums(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|a| a.cid)
        .collect();
    assert_eq!(albums, ["kept"]);
    let songs: Vec<String> = music::list_songs(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|s| s.cid)
        .collect();
    assert_eq!(songs, ["a1"]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_empty_listing_prunes_nothing_from_its_table(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let artists = ids(&["a"]);
    music::upsert_album(&mut conn, album("kept", 0, &artists))
        .await
        .unwrap();
    music::upsert_album(&mut conn, album("gone", 1, &artists))
        .await
        .unwrap();
    music::upsert_song(&mut conn, song("a1", "kept", None, &artists))
        .await
        .unwrap();
    music::upsert_song(&mut conn, song("b1", "gone", None, &artists))
        .await
        .unwrap();

    let pruned = music::prune_unlisted(&mut conn, &[], &[]).await.unwrap();
    assert_eq!(pruned, (0, 0));
    assert_eq!(music::list_albums(&mut conn).await.unwrap().len(), 2);
    assert_eq!(music::list_songs(&mut conn).await.unwrap().len(), 2);

    let pruned = music::prune_unlisted(&mut conn, &ids(&["kept", "gone"]), &[])
        .await
        .unwrap();
    assert_eq!(pruned, (0, 0));
    let pruned = music::prune_unlisted(&mut conn, &[], &ids(&["a1"]))
        .await
        .unwrap();
    assert_eq!(pruned, (0, 1));
    assert_eq!(music::list_albums(&mut conn).await.unwrap().len(), 2);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_repeat_like_keeps_the_first_time_and_unlike_reports_a_hit(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;

    let first = music::set_like(&mut conn, user, "siren:048756")
        .await
        .unwrap();
    let again = music::set_like(&mut conn, user, "siren:048756")
        .await
        .unwrap();
    assert_eq!(again.created_at, first.created_at);
    music::set_like(&mut conn, user, "game:m_bat_act27side")
        .await
        .unwrap();
    music::set_like(&mut conn, other, "siren:1").await.unwrap();

    let likes: Vec<String> = music::list_likes(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|l| l.track_id)
        .collect();
    assert_eq!(likes, ["game:m_bat_act27side", "siren:048756"]);

    assert!(
        music::unset_like(&mut conn, user, "siren:048756")
            .await
            .unwrap()
    );
    assert!(
        !music::unset_like(&mut conn, user, "siren:048756")
            .await
            .unwrap()
    );
    assert_eq!(music::list_likes(&mut conn, other).await.unwrap().len(), 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_play_only_grows_and_only_for_its_owner(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    assert_eq!(music::first_played_at(&mut conn, user).await.unwrap(), None);

    let play = music::insert_play(&mut conn, user, "siren:048756", 12.5)
        .await
        .unwrap();
    assert_eq!(play.listened_seconds, 12.5);
    assert!(
        music::update_play(&mut conn, user, play.id, 40.0)
            .await
            .unwrap()
    );
    assert!(
        music::update_play(&mut conn, user, play.id, 20.0)
            .await
            .unwrap()
    );
    assert!(
        !music::update_play(&mut conn, UserId(Uuid::new_v4()), play.id, 99.0)
            .await
            .unwrap()
    );

    let history = music::list_history(&mut conn, user, 10).await.unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].listened_seconds, 40.0);
    assert_eq!(
        music::first_played_at(&mut conn, user).await.unwrap(),
        Some(play.played_at)
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn history_is_newest_first_and_capped_by_the_limit(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;
    let mut played = Vec::new();
    for track in ["a", "b", "c"] {
        played.push(
            music::insert_play(&mut conn, user, track, 1.0)
                .await
                .unwrap()
                .id,
        );
    }
    music::insert_play(&mut conn, other, "z", 1.0)
        .await
        .unwrap();

    let history: Vec<i64> = music::list_history(&mut conn, user, 2)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.id)
        .collect();
    assert_eq!(history, [played[2], played[1]]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn totals_rank_by_plays_then_seconds_then_track(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    for (track, seconds) in [
        ("siren:1", 40.0),
        ("siren:1", 5.0),
        ("game:b", 30.0),
        ("game:a", 30.0),
        ("game:c", 31.0),
    ] {
        music::insert_play(&mut conn, user, track, seconds)
            .await
            .unwrap();
    }

    let totals = music::play_totals(&mut conn, user).await.unwrap();
    let order: Vec<(&str, i64, f64)> = totals
        .iter()
        .map(|t| (t.track_id.as_str(), t.plays, t.seconds))
        .collect();
    assert_eq!(
        order,
        [
            ("siren:1", 2, 45.0),
            ("game:c", 1, 31.0),
            ("game:a", 1, 30.0),
            ("game:b", 1, 30.0),
        ]
    );
    assert!(totals.iter().all(|t| t.last_played_at.is_some()));
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn playlists_are_scoped_to_their_owner(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let other = common::user(&mut conn, 2).await;

    let first = music::create_playlist(&mut conn, user, "Late night")
        .await
        .unwrap();
    let second = music::create_playlist(&mut conn, user, "Morning")
        .await
        .unwrap();
    let theirs = music::create_playlist(&mut conn, other, "Theirs")
        .await
        .unwrap();
    music::replace_playlist_tracks(&mut conn, theirs.id, &ids(&["siren:9"]))
        .await
        .unwrap();
    assert_eq!(music::count_playlists(&mut conn, user).await.unwrap(), 2);

    let names: Vec<String> = music::list_playlists(&mut conn, user)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(names, ["Late night", "Morning"]);
    assert_eq!(
        music::get_playlist(&mut conn, user, first.id)
            .await
            .unwrap()
            .map(|p| p.name)
            .as_deref(),
        Some("Late night")
    );
    assert!(
        music::get_playlist(&mut conn, other, first.id)
            .await
            .unwrap()
            .is_none()
    );

    let renamed = music::rename_playlist(&mut conn, user, first.id, "Later night")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(renamed.name, "Later night");
    assert!(
        music::rename_playlist(&mut conn, other, first.id, "x")
            .await
            .unwrap()
            .is_none()
    );

    assert!(
        !music::delete_playlist(&mut conn, other, second.id)
            .await
            .unwrap()
    );
    assert!(
        music::delete_playlist(&mut conn, user, second.id)
            .await
            .unwrap()
    );
    assert_eq!(music::count_playlists(&mut conn, user).await.unwrap(), 1);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn playlist_tracks_keep_the_order_asked(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let pl = music::create_playlist(&mut conn, user, "Late night")
        .await
        .unwrap();

    music::replace_playlist_tracks(&mut conn, pl.id, &ids(&["siren:1", "siren:2", "siren:3"]))
        .await
        .unwrap();
    assert_eq!(
        music::playlist_track_ids(&mut conn, pl.id).await.unwrap(),
        ids(&["siren:1", "siren:2", "siren:3"])
    );

    music::append_playlist_tracks(&mut conn, pl.id, &ids(&["siren:2", "siren:4"]))
        .await
        .unwrap();
    assert_eq!(
        music::playlist_track_ids(&mut conn, pl.id).await.unwrap(),
        ids(&["siren:1", "siren:2", "siren:3", "siren:4"])
    );

    music::replace_playlist_tracks(&mut conn, pl.id, &ids(&["siren:4", "siren:1"]))
        .await
        .unwrap();
    assert_eq!(
        music::playlist_track_ids(&mut conn, pl.id).await.unwrap(),
        ids(&["siren:4", "siren:1"])
    );

    assert!(
        music::remove_playlist_track(&mut conn, pl.id, "siren:4")
            .await
            .unwrap()
    );
    assert!(
        !music::remove_playlist_track(&mut conn, pl.id, "siren:4")
            .await
            .unwrap()
    );
    let all = music::list_playlist_tracks(&mut conn, user).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        (all[0].playlist_id, all[0].track_id.as_str()),
        (pl.id, "siren:1")
    );

    assert!(
        music::delete_playlist(&mut conn, user, pl.id)
            .await
            .unwrap()
    );
    assert!(
        music::list_playlist_tracks(&mut conn, user)
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn a_repeated_track_keeps_its_first_position(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let pl = music::create_playlist(&mut conn, user, "Loop")
        .await
        .unwrap();

    music::replace_playlist_tracks(
        &mut conn,
        pl.id,
        &ids(&["siren:2", "siren:1", "siren:2", "siren:3", "siren:1"]),
    )
    .await
    .unwrap();
    let tracks = music::list_playlist_tracks(&mut conn, user).await.unwrap();
    let got: Vec<(&str, i32)> = tracks
        .iter()
        .map(|t| (t.track_id.as_str(), t.position))
        .collect();
    assert_eq!(got, [("siren:2", 0), ("siren:1", 1), ("siren:3", 2)]);
}

#[sqlx::test(migrator = "myrtle_db::migrate::MIGRATOR")]
async fn an_append_to_an_empty_playlist_starts_at_zero_and_moves_updated_at(pool: PgPool) {
    let db = common::db(pool).await;
    let mut conn = db.conn().await.unwrap();
    let user = common::user(&mut conn, 1).await;
    let pl = music::create_playlist(&mut conn, user, "Empty")
        .await
        .unwrap();

    music::append_playlist_tracks(&mut conn, pl.id, &ids(&["siren:1", "siren:2"]))
        .await
        .unwrap();
    let tracks = music::list_playlist_tracks(&mut conn, user).await.unwrap();
    let positions: Vec<i32> = tracks.iter().map(|t| t.position).collect();
    assert_eq!(positions, [0, 1]);
    let after = music::get_playlist(&mut conn, user, pl.id)
        .await
        .unwrap()
        .unwrap();
    assert!(after.updated_at > pl.updated_at);
    assert_eq!(after.created_at, pl.created_at);
}
