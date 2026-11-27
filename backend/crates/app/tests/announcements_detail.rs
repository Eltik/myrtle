#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::services::catalog::announcements::page::{extract, is_coloured};
use myrtle_app::services::catalog::announcements::{
    Listed, banner_url, detail, list, parse_meta, publish,
};
use myrtle_types::Server;
use serde_json::Value;

const EN_URL: &str =
    "https://ark-us-static-online.yo-star.com/announce/Android/announcement/1627.html?t=1791543600";

#[test]
fn en_event_page_keeps_text_images_and_accents() {
    let p = extract(include_str!("fixtures/maintenance/1627.html"), EN_URL);
    assert_eq!(
        p.headline.as_deref(),
        Some("RECOMMENDED MONTHLY CARD AVAILABLE")
    );
    assert_eq!(p.banner_url, None);
    assert_eq!(p.body_html.matches("<img ").count(), 2);
    assert!(p.body_html.contains(
        r#"<img src="https://ark-us-static-online.yo-star.com/announce/images/20260927/e0ae4f38c73fe4570bfab5b30224c0fa.jpg" loading="lazy">"#
    ));
    assert_eq!(p.body_html.matches(r#"class="accent""#).count(), 2);
    assert!(!p.body_html.contains("style="));
    assert!(!p.body_html.contains("<script"));
    assert!(!p.body_html.contains("media-wrap"));
    assert!(p.body_html.contains("<strong>DURATION: </strong>"));
}

#[test]
fn cover_only_page_is_a_banner_with_no_body() {
    let url = "https://ark-us-static-online.yo-star.com/announce/Android/announcement/1608.html";
    let p = extract(include_str!("fixtures/maintenance/1608.html"), url);
    assert_eq!(p.headline, None);
    assert_eq!(p.body_html, "");
    assert_eq!(
        p.banner_url.as_deref(),
        Some(
            "https://ark-us-static-online.yo-star.com/announce/images/20260804/2712dbbcbd6206c2f3cc6ad94de47668.jpg"
        )
    );
}

#[test]
fn jp_page_with_banner_and_rules() {
    let url = "https://ark-jp-static-online.yo-star.com/announce/Android/announcement/1427.html";
    let p = extract(include_str!("fixtures/maintenance/jp_1427.html"), url);
    assert_eq!(
        p.headline.as_deref(),
        Some("メインテーマ 第十七章「相変臨界」公開")
    );
    assert!(
        p.banner_url
            .as_deref()
            .is_some_and(|u| u.ends_with("/20260923/0d4ade920ccb4638dd7a4db8debb365b.png"))
    );
    assert!(p.body_html.contains("<hr>"));
    assert!(p.body_html.contains("2026年10月14日"));
    assert!(p.body_html.contains(r#"<span class="accent">"#));
}

#[test]
fn kr_page_is_one_image() {
    let url = "https://ark-kr-static-online-1300509597.yo-star.com/announce/Android/announcement/1391.html";
    let p = extract(include_str!("fixtures/maintenance/kr_1391.html"), url);
    assert_eq!(p.headline.as_deref(), Some("'임계 상전이' 이벤트 안내"));
    assert_eq!(p.body_html.matches("<img ").count(), 1);
    assert!(p.banner_url.is_some());
}

#[test]
fn hostile_markup_is_stripped_and_relative_urls_made_absolute() {
    let html = r#"<div class="content"><p onclick="x()">hi<img src="../../images/a.png" onerror="x()"></p>
        <a href="javascript:alert(1)">bad</a><a href="uniwebview://move?target=shop">jump</a>
        <a href="/news/1">rel</a><iframe src="https://evil.invalid"></iframe><style>p{}</style>
        <span style="color:#333333">grey</span><span style="font-size:1.25rem">big</span></div><script>evil()</script>"#;
    let b = extract(html, EN_URL).body_html;
    assert!(!b.contains("onclick") && !b.contains("onerror"), "{b}");
    assert!(
        !b.contains("javascript") && !b.contains("uniwebview"),
        "{b}"
    );
    assert!(
        !b.contains("iframe") && !b.contains("evil") && !b.contains("p{}"),
        "{b}"
    );
    assert!(
        b.contains(
            r#"<img src="https://ark-us-static-online.yo-star.com/announce/images/a.png" loading="lazy">"#
        ),
        "{b}"
    );
    assert!(
        b.contains(
            r#"<a href="https://ark-us-static-online.yo-star.com/news/1" rel="noopener noreferrer">rel</a>"#
        ),
        "{b}"
    );
    assert!(b.contains("<span>grey</span><span>big</span>"), "{b}");
}

#[test]
fn grey_and_colour() {
    assert!(!is_coloured("000000"));
    assert!(!is_coloured("7e8c8d"));
    assert!(!is_coloured("fff"));
    assert!(is_coloured("c0392b"));
    assert!(is_coloured("003ba5"));
    assert!(is_coloured("d35400"));
}

fn listed(raw: &str, html: Option<&str>) -> Vec<Listed> {
    parse_meta(raw)
        .unwrap()
        .into_iter()
        .map(|item| Listed {
            banner_url: html.and_then(|h| banner_url(h, &item.web_url)),
            html: html.map(str::to_owned),
            item,
        })
        .collect()
}

#[tokio::test]
async fn the_store_serves_list_and_detail_and_keeps_unfetched_pages() {
    let (ctx, db) = common::context().await;
    let ttl = Duration::from_mins(5);
    let kr = include_str!("fixtures/maintenance/meta_kr.json");
    let page = include_str!("fixtures/maintenance/kr_1391.html");
    publish(&ctx, Server::KR, &listed(kr, Some(page)), 42, ttl).await;

    let board = list(&ctx, Some(Server::KR)).await.unwrap();
    assert_eq!(board.announcements.len(), 5);
    assert_eq!(board.checked_at, Some(42));
    let first = detail(&ctx, Some(Server::KR), "1391").await.unwrap();
    let one: Value = serde_json::from_str(&first.body).unwrap();
    assert_eq!(one["id"], "1391");
    assert_eq!(one["server"], "kr");
    assert_eq!(one["headline"], "'임계 상전이' 이벤트 안내");
    assert!(one["body_html"].as_str().unwrap().contains("<img "));
    assert!(one.get("summary").is_none(), "the summary is flattened");

    publish(&ctx, Server::KR, &listed(kr, None), 43, ttl).await;
    let kept = detail(&ctx, Some(Server::KR), "1391").await.unwrap();
    assert_eq!(kept.etag, first.etag, "an unfetched page keeps its detail");

    assert!(matches!(
        detail(&ctx, Some(Server::KR), "nope").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        detail(&ctx, Some(Server::JP), "1391").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        detail(&ctx, Some(Server::CN), "1391").await,
        Err(AppError::NotFound(Some(_)))
    ));
    publish(&ctx, Server::KR, &[], 44, ttl).await;
    assert!(
        matches!(
            detail(&ctx, Some(Server::KR), "1391").await,
            Err(AppError::NotFound(None))
        ),
        "a notice off the board is gone"
    );
    db.close().await;
}
