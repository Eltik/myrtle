//! Content types and ETag unquoting.

use myrtle_storage::store::{content_type_for, unquote_etag};

#[test]
fn content_types_match_the_backend() {
    assert_eq!(content_type_for("a/b.m4a"), "audio/mp4");
    assert_eq!(content_type_for("a/b.M4A"), "audio/mp4");
    assert_eq!(content_type_for("a/b.png"), "image/png");
    assert_eq!(content_type_for("a/[scene].json"), "application/json");
    assert_eq!(content_type_for("a/b.skel"), "application/octet-stream");
    assert_eq!(content_type_for("a/b.ogg"), "audio/ogg");
}

#[test]
fn etag_quotes_are_stripped() {
    assert_eq!(unquote_etag("\"abc\""), "abc");
    assert_eq!(unquote_etag("W/\"abc-2\""), "abc-2");
    assert_eq!(unquote_etag("abc"), "abc");
}
