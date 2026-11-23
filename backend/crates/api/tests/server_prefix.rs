#![allow(clippy::unwrap_used)]

use axum::body::Body;
use axum::extract::Request;
use myrtle_api::server_prefix::{PrefixServer, split_server, strip_server};
use myrtle_types::Server;

#[test]
fn a_server_segment_after_api_is_split_off() {
    let cases = [
        ("/api/en/operators", Some(Server::EN), "/api/operators"),
        (
            "/api/CN/static/skills",
            Some(Server::CN),
            "/api/static/skills",
        ),
        ("/api/bili/health", Some(Server::Bilibili), "/api/health"),
        ("/api/tw", Some(Server::TW), "/api/"),
        ("/api/operators/en", None, "/api/operators/en"),
        ("/api/enemies/x", None, "/api/enemies/x"),
        ("/en/operators", None, "/en/operators"),
        ("/api/", None, "/api/"),
        ("/", None, "/"),
    ];
    for (path, server, rest) in cases {
        let (got_server, got_rest) = split_server(path);
        assert_eq!((got_server, &*got_rest), (server, rest), "{path}");
    }
}

#[test]
fn the_rewrite_keeps_the_query_and_records_the_server() {
    let req = Request::get("/api/jp/operators/index?limit=5&q=a%20b")
        .body(Body::empty())
        .unwrap();
    let req = strip_server(req);
    assert_eq!(req.uri().path(), "/api/operators/index");
    assert_eq!(req.uri().query(), Some("limit=5&q=a%20b"));
    assert_eq!(
        req.extensions().get::<PrefixServer>(),
        Some(&PrefixServer(Server::JP))
    );
}

#[test]
fn a_path_with_no_server_is_untouched() {
    let req = Request::get("/api/operators?server=jp")
        .body(Body::empty())
        .unwrap();
    let req = strip_server(req);
    assert_eq!(req.uri(), "/api/operators?server=jp");
    assert!(req.extensions().get::<PrefixServer>().is_none());
}
