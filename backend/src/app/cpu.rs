//! Admission control for CPU-bound endpoints.
//!
//! Long synchronous work on a tokio worker blocks that worker, including the
//! accept loop, so the endpoints that run a search or a simulation are bounded
//! here rather than left to take as many workers as they are given.
//!
//! Two entry points:
//!
//! - [`run`] moves the work to the blocking pool. Use it when the computation
//!   can be handed over as an owned closure, which keeps the async workers
//!   free entirely.
//! - [`admit`] only takes a permit, for services that interleave computation
//!   with `await`ed I/O and so cannot be handed over wholesale. It bounds how
//!   many async workers such a service can occupy at once. Splitting a service
//!   into load-then-compute is what lets it move to [`run`].
//!
//! Both shed rather than queue: over the limit is an immediate 503, not a
//! backlog of requests whose callers have already given up.

use std::num::NonZero;
use std::sync::LazyLock;
use std::time::Instant;

use tokio::sync::{Semaphore, SemaphorePermit};

use crate::app::error::ApiError;
use crate::app::metrics::METRICS;

/// Concurrent CPU-bound requests allowed across the process.
///
/// Half the available cores, floor of one: half rather than all so a burst of
/// compute still leaves workers to serve cached reads and health checks.
static PERMITS: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("CPU_TASK_PERMITS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            let cores = std::thread::available_parallelism().map_or(2, NonZero::get);
            (cores / 2).max(1)
        })
});

static CPU: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(*PERMITS));

/// How many concurrent CPU-bound requests this process admits.
pub fn permits() -> usize {
    *PERMITS
}

/// Take a permit or refuse the request outright.
fn acquire(kind: &'static str) -> Result<SemaphorePermit<'static>, ApiError> {
    CPU.try_acquire().map_or_else(
        |_| {
            METRICS.cpu_rejected(kind);
            tracing::warn!(
                kind,
                permits = *PERMITS,
                "CPU admission refused; shedding request"
            );
            Err(ApiError::ServiceUnavailable)
        },
        |permit| {
            METRICS.cpu_started(kind);
            Ok(permit)
        },
    )
}

/// Run synchronous CPU-bound work on the blocking pool, under admission
/// control. The closure must own what it needs: clone the `AppState` (it is an
/// `Arc` behind the scenes) and move the request body in.
pub async fn run<F, T>(kind: &'static str, work: F) -> Result<T, ApiError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let permit = acquire(kind)?;
    let started = Instant::now();

    let outcome = tokio::task::spawn_blocking(work).await;

    METRICS.cpu_finished(kind, started.elapsed().as_micros() as u64);
    drop(permit);

    outcome.map_err(|e| {
        tracing::error!(kind, error = %e, "CPU task panicked");
        ApiError::Internal(anyhow::anyhow!("{kind} task failed: {e}"))
    })
}

/// A permit held for the duration of an async service that computes inline.
///
/// Records elapsed time on drop, so the metric is correct whether the handler
/// returned, errored, or was cancelled.
pub struct Admission {
    kind: &'static str,
    started: Instant,
    _permit: SemaphorePermit<'static>,
}

impl Drop for Admission {
    fn drop(&mut self) {
        METRICS.cpu_finished(self.kind, self.started.elapsed().as_micros() as u64);
    }
}

/// Bound the concurrency of an async service that computes on the async worker.
/// Hold the returned guard for as long as the work runs.
pub fn admit(kind: &'static str) -> Result<Admission, ApiError> {
    Ok(Admission {
        kind,
        started: Instant::now(),
        _permit: acquire(kind)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{admit, permits, run};

    #[test]
    fn permits_are_at_least_one() {
        assert!(permits() >= 1, "a zero-permit pool would refuse everything");
    }

    #[tokio::test]
    async fn work_runs_and_the_permit_comes_back() {
        // More passes than there are permits, so a leaked permit shows up as a
        // refusal before the loop ends.
        for i in 0..(permits() * 4) {
            let got = run("test", move || i * 2).await.expect("admitted");
            assert_eq!(got, i * 2);
        }
    }

    #[tokio::test]
    async fn saturation_sheds_instead_of_queueing() {
        let held: Vec<_> = (0..permits())
            .map(|_| admit("test").expect("under the limit"))
            .collect();
        assert!(
            admit("test").is_err(),
            "over the limit must refuse, not block"
        );
        drop(held);
        assert!(admit("test").is_ok(), "permits return when guards drop");
    }
}
