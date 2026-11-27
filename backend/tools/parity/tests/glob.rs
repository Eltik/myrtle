#![allow(clippy::unwrap_used)]

use myrtle_parity::glob::{PointerGlob, format_pointer};

fn g(s: &str) -> PointerGlob {
    PointerGlob::parse(s).unwrap()
}

#[test]
fn root_glob_matches_only_root() {
    let empty: [&str; 0] = [];
    assert!(g("").matches(&empty));
    assert!(!g("").matches(&["a"]));
}

#[test]
fn literal_and_single_wildcard() {
    assert!(g("/operators/*/tags").matches(&["operators", "0", "tags"]));
    assert!(g("/operators/*/tags").matches(&["operators", "17", "tags"]));
    assert!(!g("/operators/*/tags").matches(&["operators", "tags"]));
    assert!(!g("/operators/*/tags").matches(&["operators", "0", "1", "tags"]));
}

#[test]
fn double_wildcard_spans_zero_or_more() {
    let glob = g("/**/tags");
    assert!(glob.matches(&["tags"]));
    assert!(glob.matches(&["a", "b", "tags"]));
    assert!(!glob.matches(&["a", "tags", "b"]));
    let all = g("/**");
    let empty: [&str; 0] = [];
    assert!(all.matches(&empty));
    assert!(all.matches(&["x", "y"]));
}

#[test]
fn escapes_round_trip() {
    assert!(g("/a~1b/c~0d").matches(&["a/b", "c~d"]));
    assert_eq!(format_pointer(&["a/b", "c~d"]), "/a~1b/c~0d");
    assert!(g("/~01").matches(&["~1"]));
}

#[test]
fn rejects_relative_globs() {
    assert!(PointerGlob::parse("a/b").is_err());
}
