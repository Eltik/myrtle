//! The music library's storage against the real schema: likes, plays and
//! playlists round-trip for one account, a play's listened time only grows, a
//! playlist's track order is replaced and appended as asked, and the Monster
//! Siren cache rows keep a fetched lyrics file across a refresh that does not
//! move its URL and drop it when the URL moves.
//!
//! Runs inside one transaction on the local Postgres and rolls it back, so
//! nothing persists. When the local database predates v036 the migration is
//! applied inside that same transaction. Ignored by default:
//! `cargo test --test music_library_storage_test -- --ignored --nocapture`

use backend::database::queries::music as q;
use sqlx::{Connection, PgConnection};
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";

/// The music tables (applying v036 when the local database predates it) and a
/// throwaway user, all inside `conn`'s open transaction.
async fn setup(conn: &mut PgConnection) -> Uuid {
    let migrated: bool = sqlx::query_scalar("SELECT to_regclass('music_playlists') IS NOT NULL")
        .fetch_one(&mut *conn)
        .await
        .expect("catalog");
    if !migrated {
        sqlx::raw_sql(include_str!(
            "../src/database/migrations/v036_music_library.sql"
        ))
        .execute(&mut *conn)
        .await
        .expect("v036 applies");
    }
    let uid = format!("mus{}", &Uuid::new_v4().simple().to_string()[..12]);
    sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
        .bind(&uid)
        .fetch_one(&mut *conn)
        .await
        .expect("throwaway user")
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn likes_and_plays_round_trip() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local postgres");
    let mut tx = conn.begin().await.expect("begin");
    let user = setup(&mut tx).await;

    q::set_like(&mut tx, user, "siren:048756")
        .await
        .expect("like");
    let first = q::set_like(&mut tx, user, "siren:048756")
        .await
        .expect("like again");
    q::set_like(&mut tx, user, "game:m_bat_act27side")
        .await
        .expect("like game");
    let likes = q::list_likes(&mut tx, user).await.expect("list");
    assert_eq!(likes.len(), 2, "two distinct likes, the repeat collapsed");
    assert_eq!(
        likes
            .iter()
            .filter(|l| l.track_id == "siren:048756")
            .count(),
        1
    );
    assert_eq!(
        likes
            .iter()
            .find(|l| l.track_id == "siren:048756")
            .map(|l| l.created_at),
        Some(first.created_at),
        "re-liking keeps the first like's time"
    );
    assert!(
        q::unset_like(&mut tx, user, "siren:048756")
            .await
            .expect("unlike")
    );
    assert!(
        !q::unset_like(&mut tx, user, "siren:048756")
            .await
            .expect("unlike again"),
        "a second unlike finds nothing"
    );

    let play = q::insert_play(&mut tx, user, "siren:048756", 12.5)
        .await
        .expect("play");
    assert!(
        q::update_play(&mut tx, user, play.id, 40.0)
            .await
            .expect("raise")
    );
    assert!(
        q::update_play(&mut tx, user, play.id, 20.0)
            .await
            .expect("lower")
    );
    let history = q::list_history(&mut tx, user, 10).await.expect("history");
    assert_eq!(history.len(), 1);
    assert_eq!(
        history[0].listened_seconds, 40.0,
        "a smaller update never lowers the stored time"
    );
    assert!(
        !q::update_play(&mut tx, Uuid::new_v4(), play.id, 99.0)
            .await
            .expect("other user"),
        "another account cannot touch the play"
    );
    q::insert_play(&mut tx, user, "siren:048756", 5.0)
        .await
        .expect("play 2");
    q::insert_play(&mut tx, user, "game:m_bat_act27side", 7.0)
        .await
        .expect("play 3");
    let totals = q::play_totals(&mut tx, user).await.expect("totals");
    assert_eq!(totals.len(), 2);
    assert_eq!(totals[0].track_id, "siren:048756");
    assert_eq!(totals[0].plays, 2);
    assert!(
        (totals[0].seconds - 45.0).abs() < 1e-6,
        "40 + 5 = {}",
        totals[0].seconds
    );
    assert!(
        q::first_played_at(&mut tx, user)
            .await
            .expect("first")
            .is_some()
    );

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn playlists_keep_the_order_asked() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local postgres");
    let mut tx = conn.begin().await.expect("begin");
    let user = setup(&mut tx).await;

    let pl = q::create_playlist(&mut tx, user, "Late night")
        .await
        .expect("create");
    assert_eq!(q::count_playlists(&mut tx, user).await.expect("count"), 1);
    let ids = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    q::replace_playlist_tracks(&mut tx, pl.id, &ids(&["siren:1", "siren:2", "siren:3"]))
        .await
        .expect("replace");
    assert_eq!(
        q::playlist_track_ids(&mut tx, pl.id).await.expect("ids"),
        ids(&["siren:1", "siren:2", "siren:3"])
    );
    q::append_playlist_tracks(&mut tx, pl.id, &ids(&["siren:2", "siren:4"]))
        .await
        .expect("append");
    assert_eq!(
        q::playlist_track_ids(&mut tx, pl.id).await.expect("ids"),
        ids(&["siren:1", "siren:2", "siren:3", "siren:4"]),
        "an append skips a track already in and lands after the last"
    );
    q::replace_playlist_tracks(&mut tx, pl.id, &ids(&["siren:4", "siren:1"]))
        .await
        .expect("reorder");
    assert_eq!(
        q::playlist_track_ids(&mut tx, pl.id).await.expect("ids"),
        ids(&["siren:4", "siren:1"]),
        "a replace is the full new order; tracks left out go"
    );
    assert!(
        q::remove_playlist_track(&mut tx, pl.id, "siren:4")
            .await
            .expect("remove")
    );
    assert!(
        !q::remove_playlist_track(&mut tx, pl.id, "siren:4")
            .await
            .expect("remove again")
    );
    let renamed = q::rename_playlist(&mut tx, user, pl.id, "Later night")
        .await
        .expect("rename")
        .expect("owned");
    assert_eq!(renamed.name, "Later night");
    assert!(
        q::rename_playlist(&mut tx, Uuid::new_v4(), pl.id, "x")
            .await
            .expect("rename other")
            .is_none(),
        "another account's rename finds nothing"
    );
    let all = q::list_playlist_tracks(&mut tx, user)
        .await
        .expect("all tracks");
    assert_eq!(all.len(), 1);
    assert!(
        q::delete_playlist(&mut tx, user, pl.id)
            .await
            .expect("delete")
    );
    assert!(
        q::list_playlist_tracks(&mut tx, user)
            .await
            .expect("all")
            .is_empty(),
        "the tracks cascade"
    );

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn siren_rows_keep_lyrics_until_the_url_moves() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local postgres");
    let mut tx = conn.begin().await.expect("begin");
    setup(&mut tx).await;

    let artists = vec!["塞壬唱片-MSR".to_owned()];
    q::upsert_album(
        &mut tx,
        q::AlbumUpsert {
            cid: "8913",
            name: "昨日海OST",
            intro: "intro",
            belong: "arknights",
            cover_url: "https://web.hycdn.cn/siren/pic/20261010/a.jpg",
            cover_de_url: Some("https://web.hycdn.cn/siren/pic/20261010/b.jpg"),
            artists: &artists,
            list_order: 0,
        },
    )
    .await
    .expect("album");
    q::upsert_song(
        &mut tx,
        q::SongUpsert {
            cid: "048756",
            album_cid: "8913",
            name: "Song of the Nameless",
            artists: &artists,
            lyric_url: Some("https://web.hycdn.cn/siren/lyric/x.lrc"),
            mv_url: None,
            mv_cover_url: None,
            track_order: 4,
        },
    )
    .await
    .expect("song");
    q::set_lyric_text(&mut tx, "048756", "[00:01.00]hello")
        .await
        .expect("lyrics");
    q::upsert_song(
        &mut tx,
        q::SongUpsert {
            cid: "048756",
            album_cid: "8913",
            name: "Song of the Nameless",
            artists: &artists,
            lyric_url: Some("https://web.hycdn.cn/siren/lyric/x.lrc"),
            mv_url: None,
            mv_cover_url: None,
            track_order: 4,
        },
    )
    .await
    .expect("refresh same url");
    assert_eq!(
        q::lyric_text(&mut tx, "048756")
            .await
            .expect("read")
            .as_deref(),
        Some("[00:01.00]hello"),
        "a refresh with the same lyric URL keeps the file"
    );
    q::upsert_song(
        &mut tx,
        q::SongUpsert {
            cid: "048756",
            album_cid: "8913",
            name: "Song of the Nameless",
            artists: &artists,
            lyric_url: Some("https://web.hycdn.cn/siren/lyric/y.lrc"),
            mv_url: None,
            mv_cover_url: None,
            track_order: 4,
        },
    )
    .await
    .expect("refresh moved url");
    assert_eq!(
        q::lyric_text(&mut tx, "048756")
            .await
            .expect("read")
            .as_deref(),
        None,
        "a moved lyric URL drops the stale file"
    );
    let songs = q::list_songs(&mut tx).await.expect("songs");
    assert!(songs.iter().any(|s| s.cid == "048756" && !s.has_lyric_text));

    q::upsert_song(
        &mut tx,
        q::SongUpsert {
            cid: "gone",
            album_cid: "8913",
            name: "Gone",
            artists: &artists,
            lyric_url: None,
            mv_url: None,
            mv_cover_url: None,
            track_order: 5,
        },
    )
    .await
    .expect("song 2");
    let (albums, songs_pruned) =
        q::prune_unlisted(&mut tx, &["8913".to_owned()], &["048756".to_owned()])
            .await
            .expect("prune");
    assert_eq!(
        (albums, songs_pruned),
        (0, 1),
        "only the unlisted song goes"
    );

    tx.rollback().await.expect("rollback");
}

#[tokio::test]
#[ignore = "writes the local Postgres inside a rolled-back transaction"]
async fn siren_listing_refresh_keeps_the_detail_fields() {
    let mut conn = PgConnection::connect(DATABASE_URL)
        .await
        .expect("local postgres");
    let mut tx = conn.begin().await.expect("begin");
    setup(&mut tx).await;

    let artists = vec!["塞壬唱片-MSR".to_owned()];
    for cid in ["8913", "8914"] {
        q::upsert_album(
            &mut tx,
            q::AlbumUpsert {
                cid,
                name: "昨日海OST",
                intro: "",
                belong: "",
                cover_url: "https://web.hycdn.cn/siren/pic/20261010/a.jpg",
                cover_de_url: None,
                artists: &artists,
                list_order: 0,
            },
        )
        .await
        .expect("album");
    }
    q::upsert_song(
        &mut tx,
        q::SongUpsert {
            cid: "048756",
            album_cid: "8913",
            name: "Song of the Nameless",
            artists: &artists,
            lyric_url: Some("https://web.hycdn.cn/siren/lyric/x.lrc"),
            mv_url: Some("https://web.hycdn.cn/siren/mv/x.mp4"),
            mv_cover_url: None,
            track_order: 4,
        },
    )
    .await
    .expect("song");
    let song = |rows: Vec<backend::database::models::music::SirenSongRow>| {
        rows.into_iter()
            .find(|s| s.cid == "048756")
            .expect("listed")
    };

    let renamed = vec!["Emperor".to_owned()];
    q::update_song_listing(
        &mut tx,
        q::SongListing {
            cid: "048756",
            album_cid: "8914",
            name: "Song of the Named",
            artists: &renamed,
            track_order: None,
        },
    )
    .await
    .expect("refresh without an order");
    let row = song(q::list_songs(&mut tx).await.expect("songs"));
    assert_eq!(
        (
            row.name.as_str(),
            row.album_cid.as_str(),
            row.artists.clone()
        ),
        ("Song of the Named", "8914", renamed.clone())
    );
    assert_eq!(row.track_order, 4, "no order keeps the stored one");
    assert_eq!(
        row.lyric_url.as_deref(),
        Some("https://web.hycdn.cn/siren/lyric/x.lrc"),
        "the detail fields stay"
    );
    assert!(row.mv_url.is_some());

    q::update_song_listing(
        &mut tx,
        q::SongListing {
            cid: "048756",
            album_cid: "8914",
            name: "Song of the Named",
            artists: &renamed,
            track_order: Some(1),
        },
    )
    .await
    .expect("refresh with an order");
    let row = song(q::list_songs(&mut tx).await.expect("songs"));
    assert_eq!(row.track_order, 1, "an order read this sync replaces it");

    tx.rollback().await.expect("rollback");
}
