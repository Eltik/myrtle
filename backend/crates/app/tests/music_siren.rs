#![allow(clippy::unwrap_used)]

mod common;

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;
use myrtle_app::AppError;
use myrtle_app::services::music::siren::{
    self, CATALOG_KEY, SirenAlbum, SirenCatalog, SirenFetch, albums_to_fetch, bulk_import_day,
    clean_artists, is_instrumental, without_instrumental_mark,
};
use myrtle_db::repos::music as repo;
use myrtle_hypergryph::siren::{
    ApiAlbumDetail, ApiAlbumSong, ApiAlbumSummary, ApiSongDetail, ApiSongSummary,
};

fn album(cid: &str, name: &str) -> ApiAlbumSummary {
    ApiAlbumSummary {
        cid: cid.into(),
        name: format!(" {name} "),
        cover_url: format!("https://web.hycdn.cn/siren/pic/20210322/{cid}.jpg"),
        artistes: vec![" MSR".into(), "MSR".into()],
    }
}

fn song(cid: &str, album: &str, name: &str) -> ApiSongSummary {
    ApiSongSummary {
        cid: cid.into(),
        name: name.into(),
        album_cid: album.into(),
        artists: vec!["MSR".into(), " ".into()],
    }
}

fn album_detail(cid: &str, intro: &str, songs: &[&str]) -> ApiAlbumDetail {
    ApiAlbumDetail {
        cid: cid.into(),
        name: cid.into(),
        intro: format!(" {intro} "),
        belong: "arknights".into(),
        cover_url: String::new(),
        cover_de_url: Some(format!("https://web.hycdn.cn/siren/pic/{cid}_de.jpg")),
        songs: songs
            .iter()
            .map(|s| ApiAlbumSong {
                cid: (*s).into(),
                name: (*s).into(),
                artistes: vec![],
            })
            .collect(),
    }
}

fn song_detail(cid: &str, album: &str, lyric: Option<&str>) -> ApiSongDetail {
    ApiSongDetail {
        cid: cid.into(),
        name: cid.into(),
        album_cid: album.into(),
        source_url: None,
        lyric_url: lyric.map(Into::into),
        mv_url: None,
        mv_cover_url: None,
        artists: vec![],
    }
}

fn first_sync() -> SirenFetch {
    SirenFetch {
        albums: vec![album("A2", "Second"), album("A1", "First")],
        songs: vec![
            song("S2", "A1", "Radiant (Instrumental)"),
            song("S1", "A1", "Radiant"),
            song("S3", "A2", "Lone"),
        ],
        album_details: HashMap::from([
            ("A1".into(), album_detail("A1", "blurb one", &["S1", "S2"])),
            ("A2".into(), album_detail("A2", "blurb two", &["S3"])),
        ]),
        song_details: HashMap::from([
            (
                "S1".into(),
                song_detail("S1", "A1", Some("https://cdn/s1.lrc")),
            ),
            ("S2".into(), song_detail("S2", "A1", None)),
            ("S3".into(), song_detail("S3", "A2", None)),
        ]),
    }
}

#[tokio::test]
async fn a_first_sync_writes_every_album_and_song_from_the_fetched_details() {
    let (ctx, db) = common::context().await;
    let stored = siren::store(&ctx, &first_sync()).await.unwrap();
    assert_eq!((stored.albums, stored.songs, stored.with_lyrics), (2, 3, 1));

    let catalog: SirenCatalog =
        serde_json::from_str(&siren::catalog(&ctx).await.unwrap().body).unwrap();
    let order: Vec<&str> = catalog.albums.iter().map(|a| a.cid.as_str()).collect();
    assert_eq!(order, ["A2", "A1"], "list order is the label's");
    let a1 = &catalog.albums[1];
    assert_eq!(a1.name, "First");
    assert_eq!(a1.artists, ["MSR"]);
    assert_eq!(
        (a1.track_count, a1.lyrics_count, a1.instrumental_count),
        (2, 1, 1)
    );
    assert_eq!(a1.added_on, NaiveDate::from_ymd_opt(2021, 3, 22));
    assert!(catalog.synced_at.is_some());
    let songs: Vec<&str> = catalog.songs.iter().map(|s| s.cid.as_str()).collect();
    assert_eq!(
        songs,
        ["S3", "S1", "S2"],
        "album order, then the detail's track order"
    );
    let s2 = catalog.songs.iter().find(|s| s.cid == "S2").unwrap();
    assert_eq!(s2.title, "Radiant");
    assert!(s2.instrumental && !s2.has_lyrics);

    let detail = siren::album(&ctx, "A1").await.unwrap();
    assert_eq!(detail.intro, "blurb one");
    assert_eq!(detail.belong, "arknights");
    assert!(detail.cover_de_url.is_some());
    assert!(matches!(
        siren::album(&ctx, "nope").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_later_sync_keeps_details_updates_listings_and_prunes_the_unlisted() {
    let (ctx, db) = common::context().await;
    siren::store(&ctx, &first_sync()).await.unwrap();
    siren::catalog(&ctx).await.unwrap();
    let mut conn = ctx.db.conn().await.unwrap();
    repo::set_lyric_text(&mut conn, "S1", "[00:00]la")
        .await
        .unwrap();
    drop(conn);

    let later = SirenFetch {
        albums: vec![album("A3", "Third"), album("A1", "First Renamed")],
        songs: vec![
            song("S1", "A1", "Radiant v2"),
            song("S4", "A3", "New"),
            song("S5", "A3", "Failed detail"),
        ],
        album_details: HashMap::new(),
        song_details: HashMap::from([("S4".into(), song_detail("S4", "A3", None))]),
    };
    let stored = siren::store(&ctx, &later).await.unwrap();
    assert_eq!((stored.pruned_albums, stored.pruned_songs), (1, 2));
    assert_eq!(
        (stored.albums, stored.songs),
        (2, 2),
        "S5 waits for its detail"
    );

    let catalog: SirenCatalog =
        serde_json::from_str(&siren::catalog(&ctx).await.unwrap().body).unwrap();
    assert_eq!(
        catalog.albums[0].cid, "A3",
        "the write dropped the cached catalogue"
    );
    let a1 = siren::album(&ctx, "A1").await.unwrap();
    assert_eq!(a1.name, "First Renamed");
    assert_eq!(
        a1.intro, "blurb one",
        "a held album keeps its detail fields"
    );
    assert_eq!(a1.songs[0].name, "Radiant v2");
    assert_eq!(
        a1.songs[0].track_order, 0,
        "a listing with no detail keeps its order"
    );
    let a3 = siren::album(&ctx, "A3").await.unwrap();
    assert_eq!(
        a3.intro, "",
        "an album never detailed is stored from its listing"
    );
    assert_eq!(siren::lyrics(&ctx, "S1").await.unwrap(), "[00:00]la");
    assert!(matches!(
        siren::lyrics(&ctx, "S4").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn an_empty_catalogue_is_unavailable_and_reads_no_stream() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        siren::catalog(&ctx).await,
        Err(AppError::Unavailable)
    ));
    assert!(ctx.cache.get(CATALOG_KEY).await.is_none());
    assert!(matches!(
        siren::album(&ctx, "A1").await,
        Err(AppError::Unavailable)
    ));
    assert!(matches!(
        siren::stream_url(&ctx, "S1").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[test]
fn only_new_albums_and_albums_of_new_tracks_are_detailed() {
    let albums = [album("A1", "x"), album("A2", "y"), album("A3", "z")];
    let songs = [song("S1", "A1", "a"), song("S9", "A2", "b")];
    let known_albums = HashSet::from(["A1".to_owned(), "A2".to_owned()]);
    let known_songs = HashSet::from(["S1".to_owned()]);
    let got = albums_to_fetch(&albums, &songs, &known_albums, &known_songs);
    assert_eq!(got, HashSet::from(["A2".to_owned(), "A3".to_owned()]));
}

#[test]
fn instrumental_mark_reads_both_parenthesis_styles() {
    assert!(is_instrumental("Radiant (Instrumental)"));
    assert!(is_instrumental("Vows of the Sea（Instrumental）"));
    assert!(is_instrumental("x (instrumental)"));
    assert!(!is_instrumental("Instruments of War"));
}

#[test]
fn display_title_drops_only_a_trailing_mark() {
    let t = without_instrumental_mark;
    assert_eq!(t("Speed of Light (Instrumental)"), "Speed of Light");
    assert_eq!(t("Speed of Light (instrumental) "), "Speed of Light");
    assert_eq!(t("Speed of Light ( Instrumental )"), "Speed of Light");
    assert_eq!(t("光（Instrumental）"), "光");
    assert_eq!(t("(Instrumental)"), "(Instrumental)");
    assert_eq!(t("A (Instrumental) Mix"), "A (Instrumental) Mix");
    assert_eq!(t("Instruments of War"), "Instruments of War");
}

#[test]
fn artists_are_trimmed_and_deduplicated() {
    let got = clean_artists(&[
        " Emperor".into(),
        "Emperor".into(),
        String::new(),
        " D.D.D.".into(),
    ]);
    assert_eq!(got, ["Emperor", "D.D.D."]);
}

#[test]
fn bulk_import_day_needs_a_majority() {
    let day = |d: u32| NaiveDate::from_ymd_opt(2021, 3, d);
    let mk = |added_on: Option<NaiveDate>| SirenAlbum {
        cid: String::new(),
        name: String::new(),
        cover_url: String::new(),
        artists: vec![],
        list_order: 0,
        track_count: 0,
        lyrics_count: 0,
        instrumental_count: 0,
        added_on,
    };
    let majority = [mk(day(22)), mk(day(22)), mk(day(22)), mk(day(23))];
    assert_eq!(bulk_import_day(&majority), day(22));
    assert_eq!(bulk_import_day(&[mk(day(22)), mk(day(23))]), None);
}
