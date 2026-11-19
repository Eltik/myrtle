#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use myrtle_app::services::catalog::reload::{
    Announcer, DEBOUNCE, PipelineMessage, parse_message, pipeline_urls, retry_delay,
};
use myrtle_types::Server;

#[test]
fn the_pipeline_frames_parse_by_type() {
    assert_eq!(
        parse_message(r#"{"type":"update_complete","version":"24-11-01-10-00-00"}"#),
        Some(PipelineMessage::UpdateComplete {
            version: "24-11-01-10-00-00".into()
        })
    );
    assert_eq!(
        parse_message(r#"{"type":"update_complete"}"#),
        Some(PipelineMessage::UpdateComplete {
            version: "unknown".into()
        })
    );
    assert_eq!(
        parse_message(r#"{"type":"error","message":"disk full"}"#),
        Some(PipelineMessage::Error {
            message: "disk full".into()
        })
    );
    assert_eq!(
        parse_message(r#"{"type":"download_progress","done":3}"#),
        Some(PipelineMessage::Other("download_progress".into()))
    );
    for junk in ["not json", "[]", r#"{"version":"1"}"#, r#"{"type":7}"#] {
        assert_eq!(parse_message(junk), None, "{junk}");
    }
}

#[test]
fn configured_urls_map_to_data_servers() {
    let configured: BTreeMap<String, String> = [
        ("en", "ws://127.0.0.1:9160"),
        ("bilibili", " ws://127.0.0.1:9161 "),
        ("mars", "ws://127.0.0.1:9162"),
        ("jp", ""),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
    assert_eq!(
        pipeline_urls(&configured),
        [
            (Server::CN, "ws://127.0.0.1:9161".to_owned()),
            (Server::EN, "ws://127.0.0.1:9160".to_owned()),
        ]
    );
}

#[test]
fn the_debounce_runs_from_the_end_of_the_last_announcement() {
    let start = Instant::now();
    let mut announcer = Announcer::default();
    assert!(announcer.receive("a".into(), start));
    assert_eq!(announcer.due(start), Some("a"));
    let done = start + Duration::from_secs(20); // a slow reload
    announcer.finished(done);
    assert_eq!(announcer.due(done), None);
    assert!(
        !announcer.receive("a".into(), done + Duration::from_secs(1)),
        "a frame read right after a long reload repeats it"
    );
    assert!(announcer.receive("b".into(), done + DEBOUNCE));
    assert_eq!(announcer.due(done + DEBOUNCE), Some("b"));
}

#[test]
fn a_failed_announcement_is_kept_retried_and_absorbs_later_frames() {
    let start = Instant::now();
    let mut announcer = Announcer::default();
    announcer.receive("a".into(), start);
    announcer.finished(start);
    assert!(announcer.receive("b".into(), start + DEBOUNCE));
    let mut now = start + DEBOUNCE;
    for failures in 1..=40 {
        announcer.failed(now);
        assert_eq!(announcer.due(now), None, "inside the wait");
        assert_eq!(announcer.wait(now), Some(retry_delay(failures)));
        now += retry_delay(failures);
    }
    assert!(
        announcer.receive("c".into(), now),
        "a frame during an outage is never debounced"
    );
    assert_eq!(
        announcer.due(now),
        Some("c"),
        "the latest version is announced"
    );
    announcer.finished(now);
    assert_eq!(announcer.wait(now), None);
}

#[test]
fn the_retry_wait_doubles_to_a_cap_and_never_gives_up() {
    let waits: Vec<u64> = [1, 2, 3, 6, 7, 1000]
        .into_iter()
        .map(|n| retry_delay(n).as_secs())
        .collect();
    assert_eq!(waits, [1, 2, 4, 30, 30, 30]);
}
