use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

/// Width of the window job starts are spread across, in seconds.
const DEFAULT_STAGGER_SECS: u64 = 180;

/// Delay a job's first run by a fixed offset, so a boot doesn't open with every
/// job querying a cold pool at once. Derived from the name, not random, so start
/// times are the same every boot. `JOB_STAGGER_SECS=0` disables it (tests,
/// local runs).
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
        // Not a guarantee for arbitrary names: catches a rename that collapses the
        // names in use onto one offset.
        assert!(
            offsets.len() >= jobs.len() - 1,
            "jobs collapsed onto {} distinct offsets: {offsets:?}",
            offsets.len()
        );
    }
}
