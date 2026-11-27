#![allow(clippy::unwrap_used)]

use chrono::DateTime;
use myrtle_app::services::catalog::maintenance::parse::html_to_text;
use myrtle_app::services::catalog::maintenance::{NoticeWindow, parse_notice};

const FETCHED: i64 = 1_791_676_800;

fn ts(s: &str) -> i64 {
    DateTime::parse_from_rfc3339(s).unwrap().timestamp()
}

fn parse(html: &str) -> NoticeWindow {
    parse_notice(html, FETCHED).unwrap()
}

fn page(title: &str, body: &str) -> String {
    format!(r#"<span class="head-title">{title}</span><div class="content"><p>{body}</p></div>"#)
}

#[test]
fn fixed_reference_is_the_fetch_day() {
    assert_eq!(FETCHED, ts("2026-10-11T00:00:00Z"));
}

#[test]
fn four_hour_update_with_a_yearless_headline() {
    let w = parse(include_str!("fixtures/maintenance/1567.html"));
    assert_eq!(w.title, "June 22, 04:00 (UTC-7) MAINTENANCE");
    assert_eq!(w.starts_at, ts("2026-06-22T04:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-06-22T08:00:00-07:00")));
    assert!(!w.extended);
}

#[test]
fn ten_minute_window() {
    let w = parse(include_str!("fixtures/maintenance/1543.html"));
    assert_eq!(w.starts_at, ts("2026-05-08T08:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-05-08T08:10:00-07:00")));
}

#[test]
fn zone_with_a_stray_space_inside_the_parenthesis() {
    let w = parse(include_str!("fixtures/maintenance/1585.html"));
    assert_eq!(w.starts_at, ts("2026-07-16T08:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-07-16T08:10:00-07:00")));
}

#[test]
fn headline_with_no_space_before_the_zone() {
    let w = parse(include_str!("fixtures/maintenance/1592.html"));
    assert_eq!(w.title, "July 30, 2026, 08:00(UTC-7) MAINTENANCE");
    assert_eq!(w.starts_at, ts("2026-07-30T08:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-07-30T08:10:00-07:00")));
}

#[test]
fn trailing_full_stop_after_the_zone() {
    let w = parse(include_str!("fixtures/maintenance/1616.html"));
    assert_eq!(w.starts_at, ts("2026-09-02T08:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-09-02T08:10:00-07:00")));
}

#[test]
fn event_notices_are_not_maintenance() {
    assert_eq!(
        parse_notice(include_str!("fixtures/maintenance/1544.html"), FETCHED),
        None
    );
    assert_eq!(
        parse_notice(include_str!("fixtures/maintenance/1627.html"), FETCHED),
        None
    );
}

#[test]
fn range_that_crosses_midnight_with_explicit_end_date() {
    let w = parse(&page(
        "MAINTENANCE",
        "We plan to perform server maintenance on May 8, 2026, 23:00 – May 9, 2026, 01:30 (UTC-7).",
    ));
    assert_eq!(w.starts_at, ts("2026-05-08T23:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-05-09T01:30:00-07:00")));
}

#[test]
fn range_that_crosses_midnight_without_an_end_date() {
    let w = parse(&page(
        "MAINTENANCE",
        "Server maintenance: Dec 31, 2026, 23:00-01:00 (UTC-8)",
    ));
    assert_eq!(w.starts_at, ts("2026-12-31T23:00:00-08:00"));
    assert_eq!(w.ends_at, Some(ts("2027-01-01T01:00:00-08:00")));
}

#[test]
fn named_zones_and_minute_offsets() {
    let pdt = parse(&page("Maintenance", "October 14, 2026, 04:00 to 08:00 PDT"));
    assert_eq!(pdt.starts_at, ts("2026-10-14T04:00:00-07:00"));
    let pst = parse(&page(
        "Maintenance",
        "November 4, 2026, 04:00 - 08:00 (PST)",
    ));
    assert_eq!(pst.ends_at, Some(ts("2026-11-04T08:00:00-08:00")));
    let plus = parse(&page(
        "Maintenance",
        "Nov 4th, 2026, 10:00-12:00 (UTC+05:30)",
    ));
    assert_eq!(plus.starts_at, ts("2026-11-04T10:00:00+05:30"));
    let utc = parse(&page("Maintenance", "Nov 4, 2026, 10:00-12:00 UTC"));
    assert_eq!(utc.starts_at, ts("2026-11-04T10:00:00Z"));
}

#[test]
fn twenty_four_hundred_is_the_next_midnight() {
    let w = parse(&page("Maintenance", "July 16, 2026, 20:00 - 24:00 (UTC-7)"));
    assert_eq!(w.ends_at, Some(ts("2026-07-17T00:00:00-07:00")));
}

#[test]
fn missing_year_resolves_to_the_nearest_one() {
    let jan = ts("2027-01-02T00:00:00Z");
    let w = parse_notice(
        &page("Maintenance", "December 30, 04:00-08:00 (UTC-7)"),
        jan,
    )
    .unwrap();
    assert_eq!(w.starts_at, ts("2026-12-30T04:00:00-07:00"));
}

#[test]
fn extension_notice_moves_the_end() {
    let w = parse(&page(
        "MAINTENANCE EXTENSION",
        "The server maintenance on October 14, 2026, 04:00-08:00 (UTC-7) has been extended to October 14, 2026, 10:30 (UTC-7).",
    ));
    assert_eq!(w.starts_at, ts("2026-10-14T04:00:00-07:00"));
    assert_eq!(w.ends_at, Some(ts("2026-10-14T10:30:00-07:00")));
    assert!(w.extended);
}

#[test]
fn start_with_no_end() {
    let w = parse(&page(
        "EMERGENCY MAINTENANCE",
        "Emergency maintenance started on October 14, 2026, 09:12 (UTC-7). We will announce the end time later.",
    ));
    assert_eq!(w.starts_at, ts("2026-10-14T09:12:00-07:00"));
    assert_eq!(w.ends_at, None);
}

#[test]
fn entities_are_decoded_next_to_multibyte_text() {
    assert_eq!(html_to_text("&#39;임계 상전이&#39;"), "'임계 상전이'");
    assert_eq!(html_to_text("A & Pozëmka"), "A & Pozëmka");
    assert_eq!(
        html_to_text("a&#x27;b &amp; c&nbsp;d<br/>e"),
        "a'b & c d\ne"
    );
}
