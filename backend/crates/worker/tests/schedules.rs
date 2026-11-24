#![allow(clippy::unwrap_used)]

mod common;

use std::time::Duration;

use myrtle_jobs::JobKind;
use myrtle_jobs::schedule::{DEFAULTS, Schedule};

fn of(schedules: &[Schedule], kind: JobKind) -> Schedule {
    *schedules.iter().find(|s| s.kind == kind).unwrap()
}

#[tokio::test]
async fn config_periods_override_the_defaults_and_zero_disables() {
    let mut config = common::config();
    config.jobs.trending.every_secs = 77;
    config.jobs.siren.every_secs = 0;
    config.jobs.dps_watch.every_secs = 0;
    let (ctx, db) = common::context(config).await;

    let schedules = myrtle_worker::schedules(&ctx);
    assert_eq!(
        schedules.len(),
        DEFAULTS.len(),
        "no game data, no per-server schedule"
    );
    let trending = of(&schedules, JobKind::TrendingRecompute);
    assert_eq!(
        (trending.every, trending.enabled),
        (Duration::from_secs(77), true)
    );
    assert!(!of(&schedules, JobKind::SirenSync).enabled);
    assert!(!of(&schedules, JobKind::DpsUpstreamWatch).enabled);
    let sweep = of(&schedules, JobKind::MusicStatDedupeSweep);
    assert_eq!(
        sweep.every,
        of(&DEFAULTS, JobKind::MusicStatDedupeSweep).every
    );
    assert!(sweep.enabled);
    db.close().await;
}

#[tokio::test]
async fn a_configured_dps_watch_period_enables_its_schedule() {
    let mut config = common::config();
    config.jobs.dps_watch.every_secs = 30 * 60;
    let (ctx, db) = common::context(config).await;

    let watch = of(&myrtle_worker::schedules(&ctx), JobKind::DpsUpstreamWatch);
    assert_eq!(
        (watch.every, watch.enabled),
        (Duration::from_mins(30), true)
    );
    db.close().await;
}
