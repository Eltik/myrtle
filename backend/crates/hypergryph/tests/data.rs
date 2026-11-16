//! `syncData`, replays, the network config reload, and Monster Siren against a scripted server.

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};

use common::{Recorded, Reply, serve, upstream};
use myrtle_hypergryph::login::yostar;
use myrtle_hypergryph::replay::{
    ReplayHarvestOptions, ReplayOutcome, collect_player_replays, get_battle_replay,
};
use myrtle_hypergryph::siren::SirenClient;
use myrtle_hypergryph::{
    AuthSession, Domain, NetworkConfig, Upstream, UpstreamError, envelope_code,
};
use myrtle_types::Server;
use serde_json::json;

const REPLAY_ZIP_B64: &str = "UEsDBBQAAAAIAKqxblfS/TKfrQEAAN4CAAANAAAAZGVmYXVsdF9lbnRyeW1SwW7bMAz9F+7qtJaTIoBvW7BDsQwNumKHFYXB2YzDRZYcSfYWBP73UXK7tdh8sh7f43skeIEaux65NXdGn7+S82wNlHkGgTvyQYpQglrnzx9k8MMOzqCG8gIdBWwwYPwXrmnQNTuN5wfRQqmK/OomgxY7uic/6CBQBh5HmutQ5MVyodRCrR6KolTLssi/iYGjDtmwaTfWR03+CtrynnaWjeDLDI6sNTUfDXVMfhPBlZA79v4tKqCmkfRtI65338P1Z+l2naAqNq5ytcjXYi1DtJRYb+ARNTef/mM2ieQ0oAgeL1Af0N0aH6K+kMKRTWoV8SrPiwo7PuM7JQ1D16cwZtA6MfWczR/rg5t5VQEvFdPQr3l38bkdZffr54mgvJHh+gN6Sqa9DWQCo75Hc5RiBnscrfuAIWja/aGxf+89+7CRaFDuUXvKYDBMp4H7lOTlIcHVHAj+MraztZqeJIZtfRr/1cGoq0KcvVwVh8FRPA+Rnoa0WomeNvXvakDWafsUsGFHdUi3uIxT+djD2Z/pWVsxX03R3cnR2e4LkXRbic4NhrYcz+bxaZp+A1BLAQIUAxQAAAAIAKqxblfS/TKfrQEAAN4CAAANAAAAAAAAAAAAAACAAQAAAABkZWZhdWx0X2VudHJ5UEsFBgAAAAABAAEAOwAAANgBAAAAAA==";

fn logged_in() -> AuthSession {
    AuthSession::new(Some("u"), Some("s"), Some(1), None)
}

fn replay_server() -> impl Fn(&Recorded) -> Reply + Send + Sync {
    move |req: &Recorded| match req.path.as_str() {
        "/gs/account/syncData" => Reply::json(&json!({ "user": {
            "dungeon": { "stages": { "main_01-07": { "hasBattleReplay": 1 }, "main_00-01": { "hasBattleReplay": 0 } } },
            "campaignsV2": { "instances": { "camp_r_01": { "hasBattleReplay": 1 } } },
        }})),
        "/gs/quest/getBattleReplay" => Reply::json(&json!({ "battleReplay": REPLAY_ZIP_B64 })),
        "/gs/campaignV2/getBattleReplay" => Reply::status(
            400,
            &json!({ "code": envelope_code::STAGE_NO_REPLAY, "msg": "no replay" }),
        ),
        _ => Reply::status(404, &json!({})),
    }
}

#[tokio::test]
async fn sync_data_raw_returns_the_body_and_bumps_seqnum() {
    let server = serve(replay_server()).await;
    let up = upstream(&server.base);
    let mut session = logged_in();
    let raw = yostar::sync_data_raw(&up, &mut session, Server::EN)
        .await
        .unwrap();
    assert!(raw.contains("hasBattleReplay"));
    assert_eq!(session.seqnum, 2);
    let req = &server.requests()[0];
    assert_eq!(req.json(), json!({ "platform": 1 }));
    assert_eq!(req.seqnum(), 2);
}

#[tokio::test]
async fn a_replay_is_fetched_and_decoded() {
    let server = serve(replay_server()).await;
    let up = upstream(&server.base);
    let replay = get_battle_replay(&up, &mut logged_in(), Server::EN, "quest", "main_01-07")
        .await
        .unwrap();
    assert_eq!(replay.journal.metadata.stage_id, "main_01-07");
    assert_eq!(
        server.requests()[0].json(),
        json!({ "stageId": "main_01-07" })
    );
}

#[tokio::test]
async fn collect_harvests_only_flagged_stages_and_reads_5516_as_none_saved() {
    let server = serve(replay_server()).await;
    let up = upstream(&server.base);
    let mut session = logged_in();
    let opts = ReplayHarvestOptions {
        min_interval: std::time::Duration::ZERO,
        max_interval: std::time::Duration::ZERO,
        max_consecutive_errors: 3,
    };
    let mut seen = Vec::new();
    collect_player_replays(
        &up,
        &mut session,
        Server::EN,
        &opts,
        |kind, stage, outcome| {
            let label = match outcome {
                Ok(ReplayOutcome::Replay(_)) => "replay",
                Ok(ReplayOutcome::NoSaved) => "none",
                Err(_) => "error",
            };
            seen.push(format!("{kind}/{stage}:{label}"));
        },
    )
    .await
    .unwrap();
    seen.sort();
    assert_eq!(
        seen,
        ["campaignV2/camp_r_01:none", "quest/main_01-07:replay"]
    );
    assert_eq!(server.requests().len(), 3);
    assert_eq!(session.seqnum, 4);
}

#[tokio::test]
async fn harvest_stops_after_consecutive_errors() {
    let server = serve(|_: &Recorded| Reply::text(500, "nope")).await;
    let up = upstream(&server.base);
    let opts = ReplayHarvestOptions {
        min_interval: std::time::Duration::ZERO,
        max_interval: std::time::Duration::ZERO,
        max_consecutive_errors: 2,
    };
    let targets = (0..5).map(|i| ("quest".to_owned(), format!("s{i}")));
    let calls = AtomicUsize::new(0);
    let err = myrtle_hypergryph::replay::harvest_replays(
        &up,
        &mut logged_in(),
        Server::EN,
        targets,
        &opts,
        |_, _, _| {
            calls.fetch_add(1, Ordering::SeqCst);
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(err, UpstreamError::Parse { .. }), "{err:?}");
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(server.requests().len(), 2);
}

#[tokio::test]
async fn reload_reads_routes_then_versions_from_the_hv_route() {
    let server = serve(|req: &Recorded| {
        let Some(code) = req.path.strip_prefix("/conf/") else {
            return match req.path.as_str() {
                "/v/en/Android/version" => Reply::json(&json!({ "resVersion": "r-en", "clientVersion": "c-en" })),
                _ => Reply::status(404, &json!({})),
            };
        };
        if code != "en" {
            return Reply::text(500, "down");
        }
        let base = req.header("host").map(|h| format!("http://{h}")).unwrap();
        let content = json!({
            "funcVer": "V1",
            "configs": { "V1": { "network": { "gs": format!("{base}/gs"), "hv": format!("{base}/v/en/{{0}}/version") } } },
        });
        Reply::json(&json!({ "content": content.to_string() }))
    })
    .await;
    let mut cfg = NetworkConfig::new(common::device_ids());
    for (i, s) in Server::ALL.into_iter().enumerate() {
        cfg.hosts.network_routes[i] = format!("{}/conf/{}", server.base, s.as_str());
    }
    let up = Upstream::new(common::client(), cfg);

    assert_eq!(up.reload_network().await, 1);
    let snapshot = up.config();
    assert_eq!(
        snapshot.domain(Server::EN, Domain::GS),
        Some(format!("{}/gs", server.base).as_str())
    );
    assert_eq!(snapshot.domain(Server::JP, Domain::GS), None);

    assert_eq!(up.reload_versions().await, 1);
    assert_eq!(up.config().version(Server::EN).client_version, "c-en");
    assert_eq!(server.to("/v/en/Android/version").len(), 1);
    assert_eq!(
        server
            .requests()
            .iter()
            .filter(|r| r.path.starts_with("/conf/"))
            .count(),
        6
    );
}

fn siren_server() -> impl Fn(&Recorded) -> Reply + Send + Sync {
    |req: &Recorded| match req.path.as_str() {
        "/api/albums" => Reply::json(&json!({ "code": 0, "msg": "", "data": [
            { "cid": "1", "name": "A", "coverUrl": "https://c/1.jpg", "artistes": ["x"] }
        ]})),
        "/api/songs" => Reply::json(&json!({ "code": 0, "data": { "list": [
            { "cid": "9", "name": "S", "albumCid": "1", "artists": [] }
        ], "autoplay": null }})),
        "/api/song/a%2Fb" => Reply::json(&json!({ "code": 0, "data": {
            "cid": "a/b", "name": "S", "albumCid": "1", "sourceUrl": "https://s", "lyricUrl": null, "mvUrl": null, "mvCoverUrl": null
        }})),
        "/api/album/2/detail" => {
            Reply::json(&json!({ "code": 1, "msg": "not found", "data": null }))
        }
        "/lyrics.lrc" => Reply::text(200, "[00:00.00]la"),
        _ => Reply::status(404, &json!({})),
    }
}

#[tokio::test]
async fn siren_reads_the_envelope_and_sends_its_user_agent() {
    let server = serve(siren_server()).await;
    let http = common::client();
    let siren = SirenClient::new(&http, &format!("{}/api/", server.base));

    let albums = siren.albums().await.unwrap();
    assert_eq!(
        (albums[0].cid.as_str(), albums[0].cover_url.as_str()),
        ("1", "https://c/1.jpg")
    );
    assert_eq!(siren.songs().await.unwrap()[0].album_cid, "1");
    assert_eq!(
        siren
            .song_detail("a/b")
            .await
            .unwrap()
            .source_url
            .as_deref(),
        Some("https://s")
    );
    assert_eq!(
        siren
            .text(&format!("{}/lyrics.lrc", server.base))
            .await
            .unwrap(),
        "[00:00.00]la"
    );

    let err = siren.album_detail("2").await.unwrap_err();
    assert!(
        matches!(&err, UpstreamError::ApiCode { code: 1, message, url } if message == "not found" && url.ends_with("/api/album/2/detail")),
        "{err:?}"
    );
    let err = siren
        .text(&format!("{}/missing", server.base))
        .await
        .unwrap_err();
    assert!(
        matches!(&err, UpstreamError::HttpStatus { status: 404, url } if url.ends_with("/missing")),
        "{err:?}"
    );

    let req = &server.requests()[0];
    assert_eq!(req.path, "/api/albums");
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.header("user-agent"),
        Some("myrtle-music-sync (+https://myrtle.moe)")
    );
    assert_eq!(req.header("accept"), Some("application/json"));
}

#[tokio::test]
async fn failed_fetches_keep_the_previous_routes_and_versions() {
    let server = serve(|_: &Recorded| Reply::text(500, "down")).await;
    let mut cfg = common::network_config(&server.base);
    cfg.set_domain(Server::EN, Domain::GS, "https://kept");
    let up = Upstream::new(common::client(), cfg);

    assert_eq!(up.reload_network().await, 0);
    assert_eq!(up.reload_versions().await, 0);
    let snapshot = up.config();
    assert_eq!(
        snapshot.domain(Server::EN, Domain::GS),
        Some("https://kept")
    );
    assert_eq!(snapshot.version(Server::EN), &common::version());
    assert_eq!(
        server.requests().len(),
        12,
        "6 routes and 6 versions were asked for"
    );
}

#[tokio::test]
async fn siren_decode_failures_are_parse() {
    let server = serve(|_: &Recorded| Reply::text(200, "not json")).await;
    let http = common::client();
    let err = SirenClient::new(&http, &server.base)
        .albums()
        .await
        .unwrap_err();
    assert!(
        matches!(&err, UpstreamError::Parse { context, message } if context == "siren::albums" && message.contains("/albums")),
        "{err:?}"
    );
}

#[tokio::test]
async fn a_5516_worded_as_maintenance_is_still_none_saved() {
    let server = serve(|_: &Recorded| {
        Reply::status(
            400,
            &json!({ "code": envelope_code::STAGE_NO_REPLAY, "msg": "replay unavailable during maintenance" }),
        )
    })
    .await;
    let up = upstream(&server.base);
    let opts = ReplayHarvestOptions {
        min_interval: std::time::Duration::ZERO,
        max_interval: std::time::Duration::ZERO,
        max_consecutive_errors: 1,
    };
    let targets = (0..3).map(|i| ("quest".to_owned(), format!("s{i}")));
    let mut none_saved = 0;
    myrtle_hypergryph::replay::harvest_replays(
        &up,
        &mut logged_in(),
        Server::EN,
        targets,
        &opts,
        |_, _, outcome| {
            if matches!(outcome, Ok(ReplayOutcome::NoSaved)) {
                none_saved += 1;
            }
        },
    )
    .await
    .unwrap();
    assert_eq!(none_saved, 3);
}

#[tokio::test]
async fn siren_decodes_raw_bytes_so_a_bom_is_a_decode_error() {
    let server =
        serve(|_: &Recorded| Reply::text(200, "\u{feff}{\"code\":0,\"msg\":\"\",\"data\":[]}"))
            .await;
    let http = common::client();
    let err = SirenClient::new(&http, &server.base)
        .albums()
        .await
        .unwrap_err();
    assert!(matches!(err, UpstreamError::Parse { .. }), "{err:?}");
}
