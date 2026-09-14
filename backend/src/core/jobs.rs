//! Shared behaviour for the background jobs.

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

/// Width of the window job starts are spread across, in seconds.
const DEFAULT_STAGGER_SECS: u64 = 180;

/// Hold a job back by a fixed offset before its first run.
///
/// Without this every long-lived task begins work at once, so a boot opens with
/// a burst of queries from every job at the moment the pool is coldest.
///
/// The offset is derived from the job's name rather than randomised, so the
/// spread is identical on every boot and a job's start time stays predictable
/// when reading logs. `JOB_STAGGER_SECS=0` disables it, which is what tests and
/// local runs want.
pub async fn stagger(job: &'static str) {
    let window = std::env::var("JOB_STAGGER_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_STAGGER_SECS);

    if window == 0 {
        return;
    }

    let delay = offset_for(job, window);
    tracing::debug!(job, delay_secs = delay, "staggering job start");
    tokio::time::sleep(Duration::from_secs(delay)).await;
}

/// Deterministic offset in `0..window` for a job name.
fn offset_for(job: &str, window: u64) -> u64 {
    if window == 0 {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    job.hash(&mut hasher);
    hasher.finish() % window
}

#[cfg(test)]
mod tests {
    use super::offset_for;

    #[test]
    fn offsets_are_inside_the_window_and_stable() {
        for job in [
            "trending",
            "regrade",
            "gacha_detail",
            "leaderboard_snapshot",
        ] {
            let a = offset_for(job, 180);
            assert!(a < 180, "{job} offset {a} outside window");
            assert_eq!(a, offset_for(job, 180), "{job} offset is not deterministic");
        }
    }

    #[test]
    fn a_zero_window_is_no_delay() {
        assert_eq!(offset_for("anything", 0), 0);
    }

    #[test]
    fn different_jobs_do_not_all_land_together() {
        let jobs = [
            "trending",
            "leaderboard_snapshot",
            "operator_ownership",
            "medal_ownership",
            "regrade",
            "gacha_detail",
        ];
        let offsets: std::collections::HashSet<u64> =
            jobs.iter().map(|j| offset_for(j, 180)).collect();
        // Not a guarantee for arbitrary names, but a regression check on the
        // names actually in use: a rename that collapses them onto one offset
        // defeats the stagger, and this says so.
        assert!(
            offsets.len() >= jobs.len() - 1,
            "jobs collapsed onto {} distinct offsets: {offsets:?}",
            offsets.len()
        );
    }
}
