#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{Duration, Utc};
use myrtle_app::services::music::library::{
    self, MusicPlayInput, MusicPlayUpdate, MusicPlaylistAdd, MusicPlaylistCreate,
    MusicPlaylistPatch, PLAYLIST_LIMIT, PLAYLIST_TRACKS_MAX, check_listened, check_track_id,
};
use myrtle_app::services::music::siren::{self, SirenFetch};
use myrtle_app::services::music::stats::{
    self, MusicListenerSort, MusicStatWindow, STATS_KEY, StatClient, anonymous_client,
    listeners_key, top_tracks_key,
};
use myrtle_app::{AppContext, AppError, Subject};
use myrtle_db::repos::music_stats::{StatKind, claim_window};
use myrtle_db::repos::users::{SettingsUpdate, set_base_facts, update_settings};
use myrtle_hypergryph::siren::{ApiAlbumSummary, ApiSongDetail, ApiSongSummary};
use myrtle_types::{Role, Server, UserId};
use uuid::Uuid;

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

async fn seed_siren(ctx: &AppContext, cids: &[&str]) {
    let fetch = SirenFetch {
        albums: vec![ApiAlbumSummary {
            cid: "A1".into(),
            name: "Album".into(),
            cover_url: String::new(),
            artistes: vec![],
        }],
        songs: cids
            .iter()
            .map(|c| ApiSongSummary {
                cid: (*c).into(),
                name: (*c).into(),
                album_cid: "A1".into(),
                artists: vec![],
            })
            .collect(),
        album_details: HashMap::new(),
        song_details: cids
            .iter()
            .map(|c| {
                let detail = ApiSongDetail {
                    cid: (*c).into(),
                    name: (*c).into(),
                    album_cid: "A1".into(),
                    source_url: None,
                    lyric_url: None,
                    mv_url: None,
                    mv_cover_url: None,
                    artists: vec![],
                };
                ((*c).to_owned(), detail)
            })
            .collect(),
    };
    siren::store(ctx, &fetch).await.unwrap();
}

async fn public(ctx: &AppContext, user: UserId, public: bool) {
    let mut conn = ctx.db.conn().await.unwrap();
    set_base_facts(&mut conn, user, &serde_json::json!({}))
        .await
        .unwrap();
    let update = SettingsUpdate {
        public_profile: Some(public),
        ..SettingsUpdate::default()
    };
    update_settings(&mut conn, user, &update).await.unwrap();
}

#[test]
fn track_ids_need_a_known_prefix() {
    assert!(check_track_id("siren:048756").is_ok());
    assert!(check_track_id("game:m_bat_act27side").is_ok());
    assert!(check_track_id("siren:").is_err());
    assert!(check_track_id("spotify:123").is_err());
    assert!(check_track_id("siren:0487 56").is_err());
    assert!(check_track_id("").is_err());
    assert!(check_track_id(&format!("game:{}", "x".repeat(120))).is_err());
}

#[test]
fn listened_time_is_clamped_to_a_day() {
    assert!((check_listened(12.5).unwrap() - 12.5).abs() < f32::EPSILON);
    assert!((check_listened(1e9).unwrap() - 86_400.0).abs() < f32::EPSILON);
    assert!(check_listened(-1.0).is_err());
    assert!(check_listened(f32::NAN).is_err());
}

#[test]
fn ipv6_is_keyed_by_its_64_and_mapped_ipv4_by_its_ipv4() {
    let ip = |s: &str| s.parse::<IpAddr>().unwrap();
    let a = anonymous_client("k", ip("2001:db8:1:2:aaaa::1"));
    assert_eq!(a, anonymous_client("k", ip("2001:db8:1:2:ffff:1:2:3")));
    assert_ne!(a, anonymous_client("k", ip("2001:db8:1:3::1")));
    assert_eq!(
        anonymous_client("k", ip("203.0.113.7")),
        anonymous_client("k", ip("::ffff:203.0.113.7"))
    );
}

#[test]
fn the_anonymous_key_is_an_hmac_that_depends_on_the_secret() {
    let ip: IpAddr = "203.0.113.7".parse().unwrap();
    let StatClient::Anonymous(key) = anonymous_client("k", ip) else {
        panic!("anonymous");
    };
    assert_eq!(
        key,
        "a:256d6b5e67be32a3aad762d0a32962642f1c748b9ef2190b23c3887b91c2fdc2"
    );
    let long_secret = anonymous_client(&"x".repeat(100), "2001:db8:1:2::9".parse().unwrap());
    assert_eq!(
        long_secret.key(),
        "a:77b27bee46e6ad2ae4718998fa80949d6678a1732a9a75a449ccbb840efb76b3"
    );
    assert_ne!(anonymous_client("k", ip), anonymous_client("other", ip));
    assert!(
        StatClient::User(UserId(Uuid::nil()))
            .key()
            .starts_with("u:")
    );
}

#[test]
fn stats_keys_keep_their_text() {
    assert_eq!(STATS_KEY, "music:stats");
    assert_eq!(
        top_tracks_key(MusicStatWindow::Week, 50),
        "music:stats:top_tracks:week:50"
    );
    assert_eq!(
        listeners_key(MusicStatWindow::All, MusicListenerSort::Time, None, 20, 40),
        "music:stats:listeners:all:all:time:20:40"
    );
    assert_eq!(
        listeners_key(
            MusicStatWindow::Month,
            MusicListenerSort::Listens,
            Some("siren:1"),
            5,
            0
        ),
        "music:stats:listeners:siren:1:month:listens:5:0"
    );
}

#[tokio::test]
async fn the_library_needs_a_signed_in_caller() {
    let (ctx, db) = common::context().await;
    assert!(matches!(
        library::get(&ctx, &Subject::Anonymous).await,
        Err(AppError::Unauthorized)
    ));
    assert!(matches!(
        library::like(&ctx, &Subject::Anonymous, "siren:1").await,
        Err(AppError::Unauthorized)
    ));
    db.close().await;
}

#[tokio::test]
async fn likes_and_plays_round_trip_and_stay_per_account() {
    let (ctx, db) = common::context().await;
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;

    library::like(&ctx, &me, "siren:1").await.unwrap();
    library::like(&ctx, &me, "siren:1").await.unwrap();
    assert!(matches!(
        library::like(&ctx, &me, "bad").await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        library::unlike(&ctx, &other, "siren:1").await,
        Err(AppError::NotFound(None))
    ));

    let play = library::log_play(
        &ctx,
        &me,
        MusicPlayInput {
            track_id: "game:unknown".into(),
            listened_seconds: 10.0,
        },
    )
    .await
    .unwrap();
    let raise = |s: f32| MusicPlayUpdate {
        listened_seconds: s,
    };
    library::update_play(&ctx, &me, play.id, raise(40.0))
        .await
        .unwrap();
    library::update_play(&ctx, &me, play.id, raise(5.0))
        .await
        .unwrap();
    assert!(matches!(
        library::update_play(&ctx, &other, play.id, raise(50.0)).await,
        Err(AppError::NotFound(None))
    ));

    let lib = library::get(&ctx, &me).await.unwrap();
    assert_eq!(lib.likes.len(), 1);
    assert_eq!(lib.history.len(), 1);
    assert!((lib.history[0].listened_seconds - 40.0).abs() < f32::EPSILON);
    assert_eq!(lib.totals[0].plays, 1);
    assert!(lib.first_played_at.is_some());
    assert_eq!((lib.playlist_limit, lib.history_limit), (100, 200));
    let json = serde_json::to_value(&lib).unwrap();
    assert!(json.get("playlistTracksMax").is_some() && json["likes"][0].get("likedAt").is_some());

    library::unlike(&ctx, &me, "siren:1").await.unwrap();
    assert!(library::get(&ctx, &other).await.unwrap().history.is_empty());
    assert!(library::get(&ctx, &me).await.unwrap().likes.is_empty());
    db.close().await;
}

#[tokio::test]
async fn playlists_dedupe_reorder_cap_and_stay_private() {
    let (ctx, db) = common::context().await;
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    let other = common::user(&ctx, db.user(2, Server::EN).await, Role::User).await;
    let create = |name: &str, tracks: &[&str]| MusicPlaylistCreate {
        name: name.into(),
        track_ids: ids(tracks),
    };

    assert!(matches!(
        library::create_playlist(&ctx, &me, create("  ", &[])).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(matches!(
        library::create_playlist(&ctx, &me, create(&"n".repeat(61), &[])).await,
        Err(AppError::BadRequest(_))
    ));
    let p = library::create_playlist(
        &ctx,
        &me,
        create(" Mix ", &["siren:1", "game:a", "siren:1"]),
    )
    .await
    .unwrap();
    assert_eq!(p.name, "Mix");
    assert_eq!(p.track_ids, ids(&["siren:1", "game:a"]));

    let patched = library::patch_playlist(
        &ctx,
        &me,
        p.id,
        MusicPlaylistPatch {
            name: Some("Renamed".into()),
            track_ids: Some(ids(&["game:a", "siren:2"])),
        },
    )
    .await
    .unwrap();
    assert_eq!(patched.name, "Renamed");
    assert_eq!(patched.track_ids, ids(&["game:a", "siren:2"]));

    let add = |tracks: &[&str]| MusicPlaylistAdd {
        track_ids: ids(tracks),
    };
    let added = library::add_tracks(&ctx, &me, p.id, add(&["siren:2", "siren:3"]))
        .await
        .unwrap();
    assert_eq!(added.track_ids, ids(&["game:a", "siren:2", "siren:3"]));
    let many: Vec<String> = (0..PLAYLIST_TRACKS_MAX - 2)
        .map(|i| format!("siren:{}", i + 10))
        .collect();
    assert!(matches!(
        library::add_tracks(&ctx, &me, p.id, MusicPlaylistAdd { track_ids: many }).await,
        Err(AppError::BadRequest(_))
    ));

    let removed = library::remove_track(&ctx, &me, p.id, "siren:2")
        .await
        .unwrap();
    assert_eq!(removed.track_ids, ids(&["game:a", "siren:3"]));
    assert!(matches!(
        library::remove_track(&ctx, &me, p.id, "siren:2").await,
        Err(AppError::NotFound(None))
    ));

    assert!(matches!(
        library::add_tracks(&ctx, &other, p.id, add(&["siren:9"])).await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        library::delete_playlist(&ctx, &other, p.id).await,
        Err(AppError::NotFound(None))
    ));
    library::delete_playlist(&ctx, &me, p.id).await.unwrap();
    assert!(library::get(&ctx, &me).await.unwrap().playlists.is_empty());
    db.close().await;
}

#[tokio::test]
async fn an_account_keeps_at_most_the_playlist_limit() {
    let (ctx, db) = common::context().await;
    let me = common::user(&ctx, db.user(1, Server::EN).await, Role::User).await;
    for i in 0..PLAYLIST_LIMIT {
        let input = MusicPlaylistCreate {
            name: format!("p{i}"),
            track_ids: vec![],
        };
        library::create_playlist(&ctx, &me, input).await.unwrap();
    }
    let input = MusicPlaylistCreate {
        name: "one more".into(),
        track_ids: vec![],
    };
    assert!(matches!(
        library::create_playlist(&ctx, &me, input).await,
        Err(AppError::Conflict(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_listen_counts_once_per_window_and_a_play_shares_the_account_slot() {
    let (ctx, db) = common::context().await;
    seed_siren(&ctx, &["1", "2"]).await;
    let user = db.user(1, Server::EN).await;
    let me = common::user(&ctx, user, Role::User).await;
    let anon = anonymous_client(common::JWT_SECRET, "203.0.113.7".parse().unwrap());

    assert!(matches!(
        stats::record(&ctx, StatKind::Listen, "siren:404", Some(&anon)).await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        stats::record(&ctx, StatKind::Listen, "nope", None).await,
        Err(AppError::BadRequest(_))
    ));
    assert!(
        !stats::record(&ctx, StatKind::Listen, "siren:1", None)
            .await
            .unwrap()
            .counted
    );
    assert!(
        stats::record(&ctx, StatKind::Listen, "siren:1", Some(&anon))
            .await
            .unwrap()
            .counted
    );
    assert!(
        !stats::record(&ctx, StatKind::Listen, "siren:1", Some(&anon))
            .await
            .unwrap()
            .counted
    );
    assert!(
        stats::record(&ctx, StatKind::Download, "siren:1", Some(&anon))
            .await
            .unwrap()
            .counted
    );

    let play = MusicPlayInput {
        track_id: "siren:1".into(),
        listened_seconds: 30.0,
    };
    library::log_play(&ctx, &me, play.clone()).await.unwrap();
    library::log_play(&ctx, &me, play).await.unwrap();
    let client = StatClient::of(&ctx, &me, None).unwrap();
    assert_eq!(client, StatClient::User(user));
    assert!(
        !stats::record(&ctx, StatKind::Listen, "siren:1", Some(&client))
            .await
            .unwrap()
            .counted
    );

    let mut conn = ctx.db.conn().await.unwrap();
    let past = Utc::now() - Duration::minutes(31);
    assert!(
        claim_window(&mut conn, StatKind::Listen, "late", "siren:2", past, 1800.0)
            .await
            .unwrap()
    );
    assert!(
        claim_window(
            &mut conn,
            StatKind::Listen,
            "late",
            "siren:2",
            Utc::now(),
            1800.0
        )
        .await
        .unwrap()
    );
    drop(conn);

    let catalog = stats::catalog(&ctx).await.unwrap();
    let one = catalog.tracks["siren:1"];
    assert_eq!((one.listens, one.downloads), (2, 1));
    assert!(ctx.cache.get(STATS_KEY).await.is_some());

    let top = stats::top_tracks(&ctx, MusicStatWindow::All, Some(0))
        .await
        .unwrap();
    assert_eq!(top.entries.len(), 1, "a limit is at least one");
    assert_eq!(top.entries[0].track_id, "siren:1");
    assert!(
        ctx.cache
            .get(&top_tracks_key(MusicStatWindow::All, 1))
            .await
            .is_some()
    );

    let swept = stats::sweep_dedupe(&ctx).await.unwrap();
    assert!(swept.starts_with("0 "), "{swept}");
    db.close().await;
}

#[tokio::test]
async fn listeners_list_only_public_profiles() {
    let (ctx, db) = common::context().await;
    seed_siren(&ctx, &["1"]).await;
    let shown = db.user(1, Server::EN).await;
    let hidden = db.user(2, Server::EN).await;
    public(&ctx, shown, true).await;
    public(&ctx, hidden, false).await;
    for user in [shown, hidden] {
        let play = MusicPlayInput {
            track_id: "siren:1".into(),
            listened_seconds: 60.0,
        };
        library::log_play(&ctx, &common::user(&ctx, user, Role::User).await, play)
            .await
            .unwrap();
    }
    let page = stats::listeners(
        &ctx,
        MusicStatWindow::Week,
        MusicListenerSort::Time,
        Some("siren:1"),
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.entries[0].uid, "1");
    assert_eq!(page.entries[0].listens, 1);
    let key = listeners_key(
        MusicStatWindow::Week,
        MusicListenerSort::Time,
        Some("siren:1"),
        20,
        0,
    );
    assert!(ctx.cache.get(&key).await.is_some());
    assert!(matches!(
        stats::listeners(
            &ctx,
            MusicStatWindow::Week,
            MusicListenerSort::Listens,
            Some("x"),
            None,
            None
        )
        .await,
        Err(AppError::BadRequest(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_en_catalogue_builds_and_its_ids_are_known_tracks() {
    let Some((mut ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let root = std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../assets/output"
    ));
    let mut config = (*ctx.config).clone();
    config.assets.dir =
        std::env::var("MYRTLE_TEST_ASSETS").unwrap_or_else(|_| root.display().to_string());
    ctx.config = std::sync::Arc::new(config);
    ctx.snapshots = std::sync::Arc::new(myrtle_app::snapshots::SnapshotRegistry::new(&ctx.config));
    ctx.snapshots.install_shared(Server::EN, snapshot);

    let index = myrtle_app::services::music::game_index(&ctx, Server::EN)
        .await
        .unwrap();
    let parsed: myrtle_engine_music::MusicIndex = serde_json::from_str(&index.body).unwrap();
    if parsed.tracks.is_empty() {
        db.close().await;
        return;
    }
    assert_eq!(parsed.totals.tracks as usize, parsed.tracks.len());
    let again = myrtle_app::services::music::game_index(&ctx, Server::EN)
        .await
        .unwrap();
    assert_eq!(again.etag, index.etag, "served from the cache");
    let id = format!("game:{}", parsed.tracks[0].id);
    assert!(stats::is_known_track(&ctx, &id).await.unwrap());
    assert!(
        !stats::is_known_track(&ctx, "game:no_such_track")
            .await
            .unwrap()
    );

    let version = &ctx
        .snapshots
        .get(Server::EN)
        .unwrap()
        .model
        .meta
        .game_version;
    let ids_key = format!(
        "static:en:music_ids:{}",
        myrtle_app::cache::key_hash(format!("en={}/{};", version.client, version.res).as_bytes())
    );
    assert!(ctx.cache.get(&ids_key).await.is_some());
    ctx.cache.invalidate(&ids_key).await;
    let derived = myrtle_app::services::music::game_track_ids(&ctx, Server::EN)
        .await
        .unwrap();
    assert!(derived.contains(&parsed.tracks[0].id));
    assert!(
        ctx.cache.get(&ids_key).await.is_some(),
        "ids derived from the index are stored again"
    );
    db.close().await;
}

#[test]
fn the_index_keys_name_every_loaded_version() {
    use myrtle_app::services::music::index_keys;
    use myrtle_app::snapshots::Snapshot;
    use myrtle_gamedata::model::GameModel;
    use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
    let at = |res: &str| {
        let mut model = GameModel::default();
        model.meta.game_version.res = res.to_owned();
        std::sync::Arc::new(Snapshot::from_model(model, SandboxUniverse::default()))
    };
    let en = (Server::EN, at("1-a"));
    let (key, ids) = index_keys(Server::EN, &[en.clone(), (Server::CN, at("1-b"))]);
    assert!(key.starts_with("static:en:music:"));
    assert_eq!(ids, key.replace(":music:", ":music_ids:"));
    let (moved, _) = index_keys(Server::EN, &[en.clone(), (Server::CN, at("2-b"))]);
    assert_ne!(moved, key, "another server's reload moves the key");
    assert_ne!(index_keys(Server::EN, &[en]).0, key);
}
