use std::fmt::Write as _;

use myrtle_parity::corpus::{RouteMatcher, Source};
use myrtle_parity::nginx::{
    LogOptions, LogStats, import, is_secret_param, parse_combined, redact_target,
};

const LOG: &str = r#"203.0.113.9 - - [24/Oct/2026:10:00:00 +0000] "GET /api/operators/char_002_amiya HTTP/1.1" 200 5120 "https://myrtle.moe/" "Mozilla/5.0"
203.0.113.9 - - [24/Oct/2026:10:00:01 +0000] "GET /api/operators/char_002_amiya HTTP/1.1" 200 5120 "-" "Mozilla/5.0"
203.0.113.9 - - [24/Oct/2026:10:00:02 +0000] "POST /api/login HTTP/1.1" 200 90 "-" "curl/8"
203.0.113.9 - - [24/Oct/2026:10:00:03 +0000] "GET /api/operators/nope HTTP/1.1" 404 20 "-" "curl/8"
203.0.113.9 - - [24/Oct/2026:10:00:04 +0000] "GET /api/get-user?uid=123&token=abc.def&apiKey=zzz HTTP/1.1" 200 900 "-" "a \"quoted\" agent"
203.0.113.9 - - [24/Oct/2026:10:00:05 +0000] "GET /favicon.ico HTTP/1.1" 200 1 "-" "x"
203.0.113.9 - - [24/Oct/2026:10:00:06 +0000] "GET /api/x/eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl HTTP/1.1" 200 1 "-" "x"
garbage line
203.0.113.9 - - [24/Oct/2026:10:00:07 +0000] "GET https://api.myrtle.moe/api/static/operators?Session_Id=1#frag HTTP/1.1" 204 0 "-" "x"
203.0.113.9 - - [24/Oct/2026:10:00:08 +0000] "GET /api/grids?sort=new&q=a%20b HTTP/2.0" 200 9 "-" "x"
"#;

fn matcher() -> RouteMatcher {
    RouteMatcher::new(
        [
            "/api/operators/{id}",
            "/api/get-user",
            "/api/static/{resource}",
            "/api/grids",
        ]
        .map(str::to_owned),
    )
}

#[test]
fn imports_filters_dedupes_and_redacts() {
    let (entries, stats) =
        import(LOG.as_bytes(), &LogOptions::default(), Some(&matcher())).unwrap();
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "/api/get-user?uid=123",
            "/api/grids?sort=new&q=a%20b",
            "/api/operators/char_002_amiya",
            "/api/static/operators",
        ]
    );
    assert_eq!(entries[0].route, "/api/get-user");
    assert_eq!(entries[3].route, "/api/static/{resource}");
    assert!(
        entries
            .iter()
            .all(|e| e.source == Source::Nginx && e.role.is_none())
    );
    assert_eq!(
        stats,
        LogStats {
            lines: 10,
            unparsed: 1,
            not_get: 1,
            not_2xx: 1,
            outside_prefix: 1,
            secret_in_path: 1,
            redacted_params: 3,
            duplicates: 1,
            sampled_out: 0,
            kept: 4,
        }
    );
    let all = serde_json::to_string(&entries).unwrap();
    assert!(!all.contains("abc.def") && !all.contains("zzz") && !all.contains("eyJ"));
}

#[test]
fn secret_param_names() {
    for name in [
        "token",
        "access_token",
        "apiKey",
        "X-Session-Id",
        "sig",
        "password",
        "code",
        "a%74oken",
    ] {
        assert!(is_secret_param(name), "{name}");
    }
    for name in ["uid", "server", "q", "sort", "limit", "item"] {
        assert!(!is_secret_param(name), "{name}");
    }
}

#[test]
fn jwt_in_a_kept_value_drops_the_line() {
    assert_eq!(
        redact_target("/api/a?next=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln"),
        None
    );
    assert_eq!(redact_target("/api/a?"), Some(("/api/a".to_owned(), 0)));
}

#[test]
fn sampling_is_seeded() {
    let mut log = String::new();
    for i in 0..50 {
        let _ = writeln!(
            log,
            "1.2.3.4 - - [24/Oct/2026:10:00:00 +0000] \"GET /api/operators/char_{i:03} HTTP/1.1\" 200 1 \"-\" \"x\""
        );
    }
    let opts = |seed| LogOptions {
        sample: Some(10),
        seed,
        ..LogOptions::default()
    };
    let (a, sa) = import(log.as_bytes(), &opts(1), None).unwrap();
    let (b, _) = import(log.as_bytes(), &opts(1), None).unwrap();
    let (c, _) = import(log.as_bytes(), &opts(2), None).unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 10);
    assert_eq!(sa.sampled_out, 40);
    assert!(a.iter().all(|e| e.route == "(unmatched)"));
}

#[test]
fn parses_escaped_quotes_in_request() {
    let l = parse_combined(r#"1.1.1.1 - - [x] "GET /api/a\"b HTTP/1.1" 200 1 "-" "-""#).unwrap();
    assert_eq!(l.target, r#"/api/a\"b"#);
    assert_eq!(l.status, 200);
}
