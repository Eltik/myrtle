#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use chrono::NaiveDate;
use myrtle_app::AppError;
use myrtle_app::services::catalog::announcements::{
    AnnouncementSummary, Board, banner_url, board_key, board_server, list, parse_meta,
    published_on, servers, sort_newest_first, summary,
};
use myrtle_types::Server;

const EN_URL: &str =
    "https://ark-us-static-online.yo-star.com/announce/Android/announcement/1627.html?t=1791543600";

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
}

#[test]
fn real_metas_parse_with_groups() {
    for (raw, n, first) in [
        (
            include_str!("fixtures/maintenance/meta_android.json"),
            7,
            "1608",
        ),
        (include_str!("fixtures/maintenance/meta_jp.json"), 6, "1428"),
        (include_str!("fixtures/maintenance/meta_kr.json"), 5, "1391"),
    ] {
        let items = parse_meta(raw).unwrap();
        assert_eq!(items.len(), n);
        assert_eq!(items[0].announce_id, first);
        assert!(items.iter().all(|i| !i.group.is_empty() && i.day > 0));
    }
    let en = parse_meta(include_str!("fixtures/maintenance/meta_android.json")).unwrap();
    let groups: Vec<&str> = en.iter().map(|i| i.group.as_str()).collect();
    assert_eq!(
        groups,
        [
            "ACTIVITY", "ACTIVITY", "ACTIVITY", "ACTIVITY", "ACTIVITY", "SYSTEM", "SYSTEM"
        ]
    );
    assert!(
        en[0]
            .web_url
            .ends_with("/announcement/1608.html?t=1791543600")
    );
    assert!(
        parse_meta(r#"{"announceList":[{"announceId":"1"}]}"#)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn year_is_the_latest_not_in_the_future() {
    let today = day("2026-10-11");
    assert_eq!(published_on(9, 10, today), Some(day("2026-10-09")));
    assert_eq!(
        published_on(12, 10, today),
        Some(day("2026-10-12")),
        "a zone ahead"
    );
    assert_eq!(published_on(13, 10, today), Some(day("2025-10-13")));
    assert_eq!(published_on(31, 7, today), Some(day("2026-07-31")));
    assert_eq!(
        published_on(31, 12, day("2027-01-02")),
        Some(day("2026-12-31"))
    );
    assert_eq!(published_on(0, 0, today), None);
    assert_eq!(
        published_on(29, 2, day("2029-03-01")),
        Some(day("2028-02-29"))
    );
}

#[test]
fn real_board_sorts_newest_first() {
    let today = day("2026-10-11");
    let items = parse_meta(include_str!("fixtures/maintenance/meta_android.json")).unwrap();
    let mut built: Vec<AnnouncementSummary> =
        items.iter().map(|i| summary(i, None, today)).collect();
    sort_newest_first(&mut built);
    let ids: Vec<&str> = built.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(
        ids,
        ["1608", "1629", "1627", "1599", "1624", "1107", "1090"]
    );
    assert_eq!(built[0].title, "Displayed Operators Rate Up !!!");
    assert_eq!(built[0].published_on.as_deref(), Some("2026-10-09"));
    assert_eq!(built[6].published_on.as_deref(), Some("2026-07-31"));
}

#[test]
fn banners_are_absolute_and_web_only() {
    assert_eq!(
        banner_url(include_str!("fixtures/maintenance/1627.html"), EN_URL),
        None
    );
    assert_eq!(
        banner_url(include_str!("fixtures/maintenance/1608.html"), EN_URL).as_deref(),
        Some(
            "https://ark-us-static-online.yo-star.com/announce/images/20260804/2712dbbcbd6206c2f3cc6ad94de47668.jpg"
        )
    );
    let jp = "https://ark-jp-static-online.yo-star.com/announce/Android/announcement/1427.html";
    assert!(
        banner_url(include_str!("fixtures/maintenance/jp_1427.html"), jp)
            .is_some_and(|u| u.ends_with("/20260923/0d4ade920ccb4638dd7a4db8debb365b.png"))
    );
    let kr = "https://ark-kr-static-online-1300509597.yo-star.com/announce/Android/announcement/1391.html";
    assert!(banner_url(include_str!("fixtures/maintenance/kr_1391.html"), kr).is_some());
    let relative = r#"<img class="banner-image" src="../../images/a.png?x=1&amp;y=2">"#;
    assert_eq!(
        banner_url(relative, EN_URL).as_deref(),
        Some("https://ark-us-static-online.yo-star.com/announce/images/a.png?x=1&y=2")
    );
    assert_eq!(
        banner_url(r#"<img class="banner-image" src="uniwebview://x">"#, EN_URL),
        None
    );
}

#[tokio::test]
async fn list_serves_the_stored_board() {
    let (ctx, db) = common::context().await;
    assert_eq!(servers(&ctx), [Server::EN, Server::JP, Server::KR]);
    assert_eq!(board_server(&ctx, None).unwrap(), Server::EN);
    assert_eq!(board_server(&ctx, Some(Server::KR)).unwrap(), Server::KR);
    assert!(matches!(
        board_server(&ctx, Some(Server::CN)),
        Err(AppError::NotFound(Some(_)))
    ));

    let empty = list(&ctx, Some(Server::KR)).await.unwrap();
    assert!(empty.announcements.is_empty());
    assert_eq!(empty.checked_at, None);
    assert_eq!(empty.servers, ["en", "jp", "kr"]);

    let today = day("2026-10-11");
    let items = parse_meta(include_str!("fixtures/maintenance/meta_kr.json")).unwrap();
    let board = Board {
        announcements: items.iter().map(|i| summary(i, None, today)).collect(),
        checked_at: 42,
    };
    ctx.cache
        .set_json(&board_key(Server::KR), &board, Duration::from_mins(5))
        .await;
    let served = list(&ctx, Some(Server::KR)).await.unwrap();
    assert_eq!(served.server, "kr");
    assert_eq!(served.announcements.len(), 5);
    assert_eq!(served.checked_at, Some(42));
    assert!(
        list(&ctx, None).await.unwrap().announcements.is_empty(),
        "EN untouched"
    );

    let json = serde_json::to_value(&served).unwrap();
    assert_eq!(json["announcements"][0]["id"], "1391");
    assert!(json["announcements"][0].get("published_on").is_some());
    db.close().await;
}
