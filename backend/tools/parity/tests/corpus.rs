#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use myrtle_parity::corpus::{
    ExpandOptions, Expansion, RouteMatcher, encode_query, encode_segment, expand_openapi,
    read_corpus, skipped_by_reason, write_corpus,
};
use myrtle_parity::ids::IdSet;
use regex::Regex;
use serde_json::{Value, json};

fn doc() -> Value {
    json!({
        "openapi": "3.1.0",
        "paths": {
            "/api/health": {"get": {"responses": {}}},
            "/api/operators/{id}": {"get": {"parameters": [
                {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
            ]}},
            "/api/{server}/operators/{id}": {"get": {"parameters": [
                {"name": "server", "in": "path", "required": true, "schema": {"type": "string"}},
                {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}
            ]}},
            "/api/story/art-gallery/{kind}": {"get": {"parameters": [
                {"name": "kind", "in": "path", "required": true, "schema": {"$ref": "#/components/schemas/StoryArtKind"}}
            ]}},
            "/api/get-user": {"get": {
                "security": [{"bearer_auth": []}, {}],
                "parameters": [{"name": "uid", "in": "query", "required": true, "schema": {"type": "string"}},
                               {"name": "server", "in": "query", "required": false, "schema": {"type": "string"}}]
            }},
            "/api/gacha/stats": {"get": {"security": [{"bearer_auth": []}]}},
            "/api/admin/stats": {"get": {"security": [{"bearer_auth": []}, {"service_key": []}]}},
            "/api/assets/{*path}": {"get": {"parameters": [
                {"name": "path", "in": "path", "required": true, "schema": {"type": "string"}}
            ]}},
            "/api/mystery/{thing}": {"get": {"parameters": [
                {"name": "thing", "in": "path", "required": true, "schema": {"type": "string"}}
            ]}},
            "/api/write": {"post": {}}
        },
        "components": {"schemas": {"StoryArtKind": {"type": "string", "enum": ["cg", "scene"]}}}
    })
}

fn ids() -> IdSet {
    let mut ids = IdSet::default();
    let put = |ids: &mut IdSet, k: &str, v: &[&str]| {
        ids.kinds
            .insert(k.to_owned(), v.iter().map(|s| (*s).to_owned()).collect());
    };
    put(
        &mut ids,
        "operator",
        &[
            "char_002_amiya",
            "char_003_kalts",
            "char_1012_skadi2",
            "char_002_amiya@winter#1",
        ],
    );
    put(&mut ids, "server", &["en", "jp", "kr"]);
    put(&mut ids, "uid", &["123"]);
    put(&mut ids, "asset_path", &["/textures/a b/c#d.png"]);
    ids.normalize();
    ids
}

fn paths(e: &Expansion, route: &str) -> Vec<String> {
    e.entries
        .iter()
        .filter(|x| x.route == route)
        .map(|x| x.path.clone())
        .collect()
}

#[test]
fn expands_every_id_without_sample() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    assert_eq!(paths(&e, "/api/health"), ["/api/health"]);
    assert_eq!(paths(&e, "/api/operators/{id}").len(), 4);
    assert!(
        paths(&e, "/api/operators/{id}")
            .contains(&"/api/operators/char_002_amiya@winter%231".to_owned())
    );
    assert_eq!(paths(&e, "/api/{server}/operators/{id}").len(), 12);
    assert!(e.entries.iter().all(|x| x.method == "GET"));
    assert!(!e.entries.iter().any(|x| x.route == "/api/write"));
}

#[test]
fn enum_schemas_expand_through_ref() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    assert_eq!(
        paths(&e, "/api/story/art-gallery/{kind}"),
        ["/api/story/art-gallery/cg", "/api/story/art-gallery/scene"]
    );
}

#[test]
fn wildcard_keeps_slashes_and_encodes_segments() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    assert_eq!(
        paths(&e, "/api/assets/{*path}"),
        ["/api/assets/textures/a%20b/c%23d.png"]
    );
}

#[test]
fn required_query_filled_and_optional_left_out() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    assert_eq!(paths(&e, "/api/get-user"), ["/api/get-user?uid=123"]);
}

#[test]
fn auth_routes_need_roles_and_optional_routes_get_both() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    let plan = e
        .plans
        .iter()
        .find(|p| p.route == "/api/gacha/stats")
        .unwrap();
    assert!(plan.skipped.as_deref().unwrap().contains("--role"));

    let opts = ExpandOptions {
        roles: vec!["user".to_owned(), "admin".to_owned()],
        ..ExpandOptions::default()
    };
    let e = expand_openapi(&doc(), &ids(), &opts);
    let roles = |route: &str| -> Vec<Option<String>> {
        e.entries
            .iter()
            .filter(|x| x.route == route)
            .map(|x| x.role.clone())
            .collect()
    };
    assert_eq!(
        roles("/api/gacha/stats"),
        [Some("admin".to_owned()), Some("user".to_owned())]
    );
    assert_eq!(
        roles("/api/admin/stats"),
        [Some("admin".to_owned()), Some("user".to_owned())]
    );
    assert_eq!(
        roles("/api/get-user"),
        [None, Some("admin".to_owned()), Some("user".to_owned())]
    );
    assert_eq!(roles("/api/health"), [None]);
    let ids_unique: HashSet<_> = e.entries.iter().map(|x| &x.id).collect();
    assert_eq!(ids_unique.len(), e.entries.len());
}

#[test]
fn unknown_or_empty_kinds_skip_with_reason() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    let reason = |route: &str| {
        e.plans
            .iter()
            .find(|p| p.route == route)
            .unwrap()
            .skipped
            .clone()
    };
    assert!(
        reason("/api/mystery/{thing}")
            .unwrap()
            .contains("no id kind known")
    );
    let mut no_uid = ids();
    no_uid.kinds.remove("uid");
    let e2 = expand_openapi(&doc(), &no_uid, &ExpandOptions::default());
    let r = e2
        .plans
        .iter()
        .find(|p| p.route == "/api/get-user")
        .unwrap();
    assert!(r.skipped.as_deref().unwrap().contains("`uid`"));
    let by = skipped_by_reason(&e2.plans);
    assert!(by.values().flatten().any(|r| r == "/api/get-user"));
}

#[test]
fn sampling_is_capped_deterministic_and_seed_dependent() {
    let opts = |seed| ExpandOptions {
        sample: Some(5),
        seed,
        ..ExpandOptions::default()
    };
    let a = expand_openapi(&doc(), &ids(), &opts(1));
    let b = expand_openapi(&doc(), &ids(), &opts(1));
    let c = expand_openapi(&doc(), &ids(), &opts(2));
    assert_eq!(a.entries, b.entries);
    let route = "/api/{server}/operators/{id}";
    assert_eq!(paths(&a, route).len(), 5);
    assert_ne!(paths(&a, route), paths(&c, route));
    let plan = a.plans.iter().find(|p| p.route == route).unwrap();
    assert_eq!(plan.combinations, 12);
    assert_eq!(plan.emitted, 5);
    assert_eq!(
        paths(&a, "/api/operators/{id}"),
        paths(&c, "/api/operators/{id}")
    );
}

#[test]
fn sample_of_one_route_does_not_depend_on_other_routes() {
    let opts = ExpandOptions {
        sample: Some(3),
        ..ExpandOptions::default()
    };
    let full = expand_openapi(&doc(), &ids(), &opts);
    let mut smaller = doc();
    smaller["paths"]
        .as_object_mut()
        .unwrap()
        .remove("/api/operators/{id}");
    let part = expand_openapi(&smaller, &ids(), &opts);
    let route = "/api/{server}/operators/{id}";
    assert_eq!(paths(&full, route), paths(&part, route));
}

#[test]
fn include_and_exclude_filter_routes() {
    let opts = ExpandOptions {
        include: vec![Regex::new("^/api/operators/.+").unwrap()],
        exclude: vec![Regex::new("/(index|roster)$").unwrap()],
        ..ExpandOptions::default()
    };
    let e = expand_openapi(&doc(), &ids(), &opts);
    assert!(e.entries.iter().all(|x| x.route == "/api/operators/{id}"));
}

#[test]
fn corpus_round_trips_through_jsonl() {
    let e = expand_openapi(&doc(), &ids(), &ExpandOptions::default());
    let dir = std::env::temp_dir().join(format!("parity-corpus-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("c.jsonl");
    write_corpus(&file, &e.entries).unwrap();
    assert_eq!(read_corpus(&file).unwrap(), e.entries);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn route_matcher_prefers_literals() {
    let m = RouteMatcher::new(
        [
            "/api/operators/{id}",
            "/api/operators/index",
            "/api/{server}/operators/{id}",
            "/api/assets/{*path}",
        ]
        .map(str::to_owned),
    );
    assert_eq!(
        m.route_for("/api/operators/index"),
        Some("/api/operators/index")
    );
    assert_eq!(
        m.route_for("/api/operators/char_002_amiya?x=1"),
        Some("/api/operators/{id}")
    );
    assert_eq!(
        m.route_for("/api/en/operators/char_002_amiya"),
        Some("/api/{server}/operators/{id}")
    );
    assert_eq!(
        m.route_for("/api/assets/a/b/c.png"),
        Some("/api/assets/{*path}")
    );
    assert_eq!(m.route_for("/api/nope"), None);
}

#[test]
fn encoders() {
    assert_eq!(
        encode_segment("a b/c#d?e%f@g:h"),
        "a%20b%2Fc%23d%3Fe%25f@g:h"
    );
    assert_eq!(encode_segment("アミヤ"), "%E3%82%A2%E3%83%9F%E3%83%A4");
    assert_eq!(encode_query("a b&c=d+e"), "a%20b%26c%3Dd%2Be");
}
