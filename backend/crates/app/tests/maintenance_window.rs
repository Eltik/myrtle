#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_app::services::catalog::announcements::parse_meta;
use myrtle_app::services::catalog::maintenance::{
    MaintenancePhase, MaintenanceState, Notice, NoticeWindow, OPEN_ENDED_LIMIT_SECS,
    OVERRUN_LIMIT_SECS, Signals, blocking_window, carry_over, may_be_maintenance, poll,
    prean_is_active, prean_url, resolve, state_key, status, status_at, wants_probe,
};
use myrtle_types::Server;

const H: i64 = 3600;
const T0: i64 = 1_800_000_000;

fn notice(id: u64, start: i64, end: Option<i64>, extended: bool) -> Notice {
    Notice {
        id,
        url: format!("https://example.invalid/{id}.html"),
        window: NoticeWindow {
            starts_at: start,
            ends_at: end,
            extended,
            title: format!("notice {id}"),
        },
    }
}

const DOWN: Signals = Signals {
    prean_active: false,
    game_down: Some(true),
};
const UP: Signals = Signals {
    prean_active: false,
    game_down: Some(false),
};

#[test]
fn real_meta_group_filter_keeps_the_statements() {
    let listed = parse_meta(include_str!("fixtures/maintenance/meta_android.json")).unwrap();
    let kept: Vec<&str> = listed
        .iter()
        .filter(|i| may_be_maintenance(i))
        .map(|i| i.announce_id.as_str())
        .collect();
    assert_eq!(kept, ["1107", "1090"]);
}

#[test]
fn prean_meta_sits_beside_the_board() {
    assert_eq!(
        prean_url(
            "https://ark-us-static-online.yo-star.com/announce/Android/announcement.meta.json"
        )
        .as_deref(),
        Some("https://ark-us-static-online.yo-star.com/announce/Android/preannouncement.meta.json")
    );
    assert!(!prean_is_active(include_str!(
        "fixtures/maintenance/prean.json"
    )));
    assert!(prean_is_active(r#"{"actived":true}"#));
    assert!(!prean_is_active("not json"));
}

#[test]
fn phases_across_a_window() {
    let n = [notice(1, T0, Some(T0 + 4 * H), false)];
    let at = |t| resolve(&n, t, Signals::default()).map(|w| w.phase);
    assert_eq!(at(T0 - H), Some(MaintenancePhase::Upcoming));
    assert_eq!(at(T0), Some(MaintenancePhase::Ongoing));
    assert_eq!(
        at(T0 + 4 * H),
        None,
        "closes at the planned end with no overrun signal"
    );
    assert_eq!(at(T0 - 15 * 24 * H), None, "beyond the horizon");
}

#[test]
fn overrun_stays_up_while_the_game_is_down() {
    let n = [notice(1, T0, Some(T0 + H), false)];
    let w = resolve(&n, T0 + 2 * H, DOWN).unwrap();
    assert_eq!(w.phase, MaintenancePhase::Extended);
    assert_eq!(w.ends_at, Some(T0 + H));
    let prean = Signals {
        prean_active: true,
        game_down: None,
    };
    assert!(resolve(&n, T0 + 2 * H, prean).is_some());
    assert!(
        resolve(&n, T0 + H + OVERRUN_LIMIT_SECS, DOWN).is_none(),
        "gives up eventually"
    );
    assert!(resolve(&n, T0 + 2 * H, UP).is_none());
}

#[test]
fn a_login_screen_notice_does_not_outrank_a_login_that_works() {
    let ten_minutes = [notice(1663, T0, Some(T0 + 600), false)];
    let notice_up_game_up = Signals {
        prean_active: true,
        game_down: Some(false),
    };
    assert!(resolve(&ten_minutes, T0 + 7 * H, notice_up_game_up).is_none());

    let open_ended = [notice(1, T0, None, false)];
    assert!(resolve(&open_ended, T0 + H, notice_up_game_up).is_none());

    for prean_active in [true, false] {
        let down = Signals {
            prean_active,
            game_down: Some(true),
        };
        assert!(resolve(&ten_minutes, T0 + 7 * H, down).is_some());
    }
    let notice_only = Signals {
        prean_active: true,
        game_down: None,
    };
    assert!(resolve(&ten_minutes, T0 + 7 * H, notice_only).is_some());
}

#[test]
fn newer_notice_extends_the_same_window() {
    let n = [
        notice(1626, T0, Some(T0 + H), false),
        notice(1630, T0, Some(T0 + 3 * H), false),
    ];
    let w = resolve(&n, T0 + 2 * H, Signals::default()).unwrap();
    assert_eq!(w.phase, MaintenancePhase::Extended);
    assert_eq!(w.ends_at, Some(T0 + 3 * H));
    assert!(w.notice_url.ends_with("1630.html"));
    let early = resolve(&n, T0 - H, Signals::default()).unwrap();
    assert_eq!(early.phase, MaintenancePhase::Upcoming);
    assert_eq!(early.ends_at, Some(T0 + 3 * H));
}

#[test]
fn nearest_of_two_separate_windows() {
    let n = [
        notice(2, T0 + 7 * 24 * H, Some(T0 + 7 * 24 * H + H), false),
        notice(1, T0, Some(T0 + H), false),
    ];
    assert_eq!(
        resolve(&n, T0 - H, Signals::default()).unwrap().starts_at,
        T0
    );
    let later = resolve(&n, T0 + 2 * H, Signals::default()).unwrap();
    assert_eq!(later.starts_at, T0 + 7 * 24 * H);
}

#[test]
fn open_ended_window_closes_when_the_game_answers() {
    let n = [notice(1, T0, None, false)];
    assert_eq!(
        resolve(&n, T0 + H, Signals::default()).map(|w| w.phase),
        Some(MaintenancePhase::Ongoing)
    );
    assert!(resolve(&n, T0 + H, UP).is_none());
    assert!(resolve(&n, T0 + OPEN_ENDED_LIMIT_SECS, Signals::default()).is_none());
}

#[test]
fn a_running_window_outlives_its_notice_leaving_the_board() {
    let previous = [notice(1628, T0, Some(T0 + 4 * H), false)];
    let kept = carry_over(&previous, Vec::new(), T0 + H);
    assert_eq!(
        resolve(&kept, T0 + H, Signals::default()).map(|w| w.phase),
        Some(MaintenancePhase::Ongoing)
    );
    assert_eq!(
        carry_over(&previous, Vec::new(), T0 - H / 2).len(),
        1,
        "board swap"
    );
    assert!(
        carry_over(&previous, Vec::new(), T0 - 24 * H).is_empty(),
        "cancellation"
    );
    let after = carry_over(&previous, Vec::new(), T0 + 5 * H);
    assert!(resolve(&after, T0 + 5 * H, Signals::default()).is_none());
    assert!(carry_over(&previous, Vec::new(), T0 + 4 * H + OVERRUN_LIMIT_SECS).is_empty());
    let fresh = vec![notice(1628, T0, Some(T0 + 5 * H), true)];
    let both = carry_over(&previous, fresh, T0 + H);
    assert_eq!(both.len(), 1);
    assert_eq!(both[0].window.ends_at, Some(T0 + 5 * H));
}

#[test]
fn probe_only_after_the_planned_end() {
    let n = [notice(1, T0, Some(T0 + H), false)];
    assert!(!wants_probe(&n, T0 - H));
    assert!(!wants_probe(&n, T0 + H / 2));
    assert!(wants_probe(&n, T0 + H));
    assert!(!wants_probe(&n, T0 + H + OVERRUN_LIMIT_SECS));
}

#[test]
fn a_stale_probe_is_not_trusted() {
    let state = MaintenanceState {
        notices: vec![notice(1, T0, Some(T0 + H), false)],
        prean_active: false,
        probe: Some((true, T0 + H)),
        checked_at: Some(T0 + H),
    };
    let fresh = status_at(&state, T0 + H + 300);
    assert_eq!(
        fresh.window.map(|w| w.phase),
        Some(MaintenancePhase::Extended)
    );
    assert_eq!(fresh.checked_at, Some(T0 + H));
    assert!(status_at(&state, T0 + H + 601).window.is_none());
}

#[tokio::test]
async fn blocking_window_reads_the_shared_state() {
    let (ctx, db) = common::context().await;
    assert_eq!(status(&ctx).await.checked_at, None);
    assert!(blocking_window(&ctx, Server::EN).await.is_none());

    let now = chrono::Utc::now().timestamp();
    let mut state = MaintenanceState {
        notices: vec![notice(1, now + H, Some(now + 2 * H), false)],
        checked_at: Some(now),
        ..MaintenanceState::default()
    };
    let key = state_key(Server::EN);
    ctx.cache
        .set_json(&key, &state, Duration::from_mins(5))
        .await;
    let shown = status(&ctx).await;
    assert_eq!(shown.window.unwrap().phase, MaintenancePhase::Upcoming);
    assert_eq!(shown.checked_at, Some(now));
    assert!(
        blocking_window(&ctx, Server::EN).await.is_none(),
        "upcoming does not block"
    );

    state.notices = vec![notice(1, now - H, Some(now + H), false)];
    ctx.cache
        .set_json(&key, &state, Duration::from_mins(5))
        .await;
    let window = blocking_window(&ctx, Server::EN).await.unwrap();
    assert_eq!(window.phase, MaintenancePhase::Ongoing);
    assert_eq!(window.ends_at, Some(now + H));
    assert!(
        blocking_window(&ctx, Server::JP).await.is_none(),
        "only EN is read"
    );
    db.close().await;
}

#[tokio::test]
async fn poll_is_off_at_zero_secs() {
    let (mut ctx, db) = common::context().await;
    let mut config = (*ctx.config).clone();
    config.jobs.maintenance.poll_secs = 0;
    ctx.config = std::sync::Arc::new(config);
    assert_eq!(poll(&ctx).await.unwrap(), Duration::ZERO);
    assert_eq!(status(&ctx).await.checked_at, None);
    db.close().await;
}
