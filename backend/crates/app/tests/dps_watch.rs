#![allow(clippy::unwrap_used)]

mod common;

use std::sync::Arc;
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::services::catalog::dps::watch::{
    Head, MAX_BACKOFF_SECS, WatchMemory, backing_off, backoff_secs, check, interval_secs, observe,
    watch_upstream,
};
use myrtle_jobs::{JobKind, schedule};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[test]
fn backoff_doubles_then_caps() {
    assert_eq!(backoff_secs(3600, 0), 3600);
    assert_eq!(backoff_secs(3600, 1), 7200);
    assert_eq!(backoff_secs(3600, 2), 14_400);
    assert_eq!(backoff_secs(3600, 3), MAX_BACKOFF_SECS);
    assert_eq!(backoff_secs(3600, 99), MAX_BACKOFF_SECS);
    assert_eq!(backoff_secs(60, 99), 3840, "doubling stops at 64x");
    assert_eq!(backoff_secs(u64::MAX, 3), MAX_BACKOFF_SECS);
}

#[test]
fn the_backoff_base_is_the_schedule_row_not_every_secs() {
    assert_eq!(interval_secs(), 3600);
}

#[test]
fn only_failures_hold_a_run_back() {
    let clean = WatchMemory {
        last_checked_at: 1_000,
        ..WatchMemory::default()
    };
    assert!(!backing_off(&clean, 3600, 1_001));
    let failed = WatchMemory {
        consecutive_failures: 2,
        ..clean
    };
    assert!(backing_off(&failed, 3600, 1_000 + 12_600));
    assert!(
        !backing_off(&failed, 3600, 1_000 + 14_400),
        "missed runs never stretch the wait"
    );
    let waited = WatchMemory {
        skipped_runs: 3,
        ..failed
    };
    assert!(!backing_off(&waited, 3600, 1_000 + 3_600));
}

fn checks(slots: i64, late: impl Fn(i64) -> i64) -> Vec<i64> {
    let base = interval_secs();
    let mut memory = WatchMemory::default();
    let mut ran = Vec::new();
    for n in 1..=slots {
        let now = n * 3600 + late(n);
        if backing_off(&memory, base, now) {
            memory.skipped_runs += 1;
        } else {
            ran.push(n);
            memory = observe(memory, None, now).0;
        }
    }
    ran
}

#[test]
fn on_an_hourly_schedule_each_failure_skips_more_slots() {
    assert_eq!(checks(16, |_| 3), [1, 3, 7, 13], "2h, 4h, then the 6h cap");
}

#[test]
fn a_late_run_does_not_push_the_next_check_back() {
    assert_eq!(
        checks(16, |n| if n == 1 { 31 * 60 } else { 3 }),
        [1, 3, 7, 13]
    );
    assert_eq!(
        checks(16, |n| if n % 2 == 0 { 50 * 60 } else { 0 }),
        [1, 3, 7, 13],
        "uneven lateness either way"
    );
}

#[test]
fn a_commit_is_reported_once() {
    let head = |sha: &str| Head {
        sha: Some(sha.to_owned()),
        etag: Some("\"e1\"".to_owned()),
    };
    let (memory, fresh) = observe(WatchMemory::default(), Some(head("abc")), 10);
    assert_eq!(fresh.as_deref(), Some("abc"));
    assert_eq!(memory.etag.as_deref(), Some("\"e1\""));
    let (memory, fresh) = observe(memory, Some(head("abc")), 20);
    assert_eq!(fresh, None);
    let (memory, fresh) = observe(memory, Some(head("def")), 30);
    assert_eq!(fresh.as_deref(), Some("def"));
    assert_eq!(memory.last_commit_sha.as_deref(), Some("def"));
}

#[test]
fn a_failure_counts_and_keeps_what_was_known() {
    let known = WatchMemory {
        last_commit_sha: Some("abc".into()),
        etag: Some("\"e1\"".into()),
        ..WatchMemory::default()
    };
    let (memory, fresh) = observe(known, None, 50);
    assert_eq!(fresh, None);
    assert_eq!(memory.consecutive_failures, 1);
    assert_eq!(memory.last_checked_at, 50);
    assert_eq!(memory.etag.as_deref(), Some("\"e1\""));
    let (memory, _) = observe(
        memory,
        Some(Head {
            sha: None,
            etag: Some("\"e1\"".into()),
        }),
        60,
    );
    assert_eq!(memory.consecutive_failures, 0);
    assert_eq!(memory.last_commit_sha.as_deref(), Some("abc"));
}

async fn stub(response: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://{}/repos/o/r/commits",
        listener.local_addr().unwrap()
    );
    let served = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buf = [0u8; 1024];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = socket.read(&mut buf).await.unwrap();
            request.extend_from_slice(&buf[..n]);
        }
        socket.write_all(response.as_bytes()).await.unwrap();
        socket.shutdown().await.unwrap();
        String::from_utf8(request).unwrap().to_ascii_lowercase()
    });
    (url, served)
}

fn client() -> reqwest::Client {
    let _ = rustls::crypto::ring::default_provider().install_default();
    reqwest::Client::builder().no_proxy().build().unwrap()
}

#[tokio::test]
async fn a_new_head_comes_back_with_its_etag() {
    let body = r#"[{"sha":"abc123"}]"#;
    let response = Box::leak(
        format!(
            "HTTP/1.1 200 OK\r\nETag: \"e2\"\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .into_boxed_str(),
    );
    let (url, served) = stub(response).await;
    let head = check(&client(), &url, Some("\"e1\""), Some("tok"))
        .await
        .unwrap();
    assert_eq!(
        head,
        Head {
            sha: Some("abc123".into()),
            etag: Some("\"e2\"".into())
        }
    );
    let request = served.await.unwrap();
    assert!(request.contains("if-none-match: \"e1\""));
    assert!(request.contains("authorization: bearer tok"));
    assert!(request.contains("user-agent: myrtle-dps-watcher"));
}

#[tokio::test]
async fn not_modified_keeps_the_old_etag() {
    let (url, served) = stub("HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n").await;
    let head = check(&client(), &url, Some("\"e1\""), None).await.unwrap();
    assert_eq!(
        head,
        Head {
            sha: None,
            etag: Some("\"e1\"".into())
        }
    );
    assert!(!served.await.unwrap().contains("authorization"));
}

#[tokio::test]
async fn a_refusal_or_rate_limit_counts_as_a_failure_and_keeps_the_etag() {
    for response in [
        "HTTP/1.1 403 Forbidden\r\nRetry-After: 60\r\nContent-Length: 0\r\n\r\n",
        "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 60\r\nContent-Length: 0\r\n\r\n",
    ] {
        let (url, served) = stub(response).await;
        let checked = check(&client(), &url, Some("\"e1\""), None).await;
        assert_eq!(checked, None);
        served.await.unwrap();
        let known = WatchMemory {
            consecutive_failures: 1,
            etag: Some("\"e1\"".into()),
            ..WatchMemory::default()
        };
        let (memory, fresh) = observe(known, checked, 70);
        assert_eq!((memory.consecutive_failures, fresh), (2, None));
        assert_eq!(memory.etag.as_deref(), Some("\"e1\""));
    }
}

fn watching(base: &AppContext, every_secs: u64) -> AppContext {
    let mut config = (*base.config).clone();
    config.jobs.dps_watch.every_secs = every_secs;
    AppContext {
        config: Arc::new(config),
        ..base.clone()
    }
}

#[tokio::test]
async fn the_job_is_off_at_zero_and_reads_its_backoff_from_the_schedule_row() {
    let (base, db) = common::context().await;
    assert_eq!(
        watch_upstream(&watching(&base, 0)).await.unwrap(),
        "DPS upstream watch is off"
    );

    let mut conn = base.db.conn().await.unwrap();
    schedule::sync(&mut conn, &schedule::DEFAULTS, Duration::ZERO)
        .await
        .unwrap();
    let memory = WatchMemory {
        consecutive_failures: 3,
        last_checked_at: chrono::Utc::now().timestamp(),
        ..WatchMemory::default()
    };
    schedule::remember(
        &mut conn,
        JobKind::DpsUpstreamWatch,
        None,
        &serde_json::to_value(&memory).unwrap(),
    )
    .await
    .unwrap();
    drop(conn);
    assert_eq!(
        watch_upstream(&watching(&base, 60)).await.unwrap(),
        "backing off after 3 failures"
    );
    db.close().await;
}
