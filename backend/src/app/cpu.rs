//! Admission control for CPU-bound endpoints.
//!
//! Long synchronous work on a tokio worker blocks it, accept loop included, so
//! search and simulation endpoints are bounded here.
//!
//! - [`run`] moves the work to the blocking pool as an owned closure, keeping the
//!   async workers free.
//! - [`admit`] only takes a permit, for services that interleave compute with
//!   awaited I/O; it bounds how many async workers they occupy. Splitting such a
//!   service into load-then-compute is what lets it move to [`run`].
//!
//! Over the limit, a request WAITS a bounded time for a permit, refused only if
//! none frees up or the queue is full. Refusing instantly was the old behaviour
//! and wrong for a user-facing page: on a 3-core box the formula below gives ONE
//! permit, so a second reader of `/api/user/improvements` got a 503 while the
//! first computed. The queue is bounded by time and depth, so this still sheds. A
//! waiter yields on the semaphore and holds no worker, only its connection and
//! request state, which the depth cap protects.

use std::num::NonZero;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

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

/// How long a request may wait for a permit before it is refused.
///
/// A small multiple of the mean hold time (`/admin/stats`: `sum_micros / started`
/// per kind), so a burst drains while a saturated box still sheds before the 30s
/// handler timeout in `middleware`.
///
/// `CPU_TASK_WAIT_MS=0` restores the old behaviour EXACTLY: refuse the moment no
/// permit is free.
static WAIT: LazyLock<Duration> = LazyLock::new(|| {
    let ms = std::env::var("CPU_TASK_WAIT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2_500);
    Duration::from_millis(ms)
});

/// How many requests may wait for a permit at once.
///
/// Uncapped, a slow spell becomes a queue that grows with traffic, each entry
/// holding a connection and request state, then times out all together. Eight per
/// permit is a TRADE, not derived: deep enough for the bursts this sees, shallow
/// enough that memory is bounded and the tail waiter can still be served in WAIT.
static QUEUE_DEPTH: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("CPU_TASK_QUEUE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| *PERMITS * 8)
});

static WAITING: AtomicUsize = AtomicUsize::new(0);

/// Keeps `WAITING` honest when a waiter goes away. Axum drops the handler future
/// on client disconnect or handler timeout, so a decrement after the await would
/// be skipped exactly then, and the counter would climb until the depth cap
/// refused everything forever.
struct Waiter;

impl Drop for Waiter {
    fn drop(&mut self) {
        WAITING.fetch_sub(1, Ordering::Relaxed);
    }
}

pub fn waiting() -> usize {
    WAITING.load(Ordering::Relaxed)
}

pub fn wait_ms() -> u64 {
    WAIT.as_millis() as u64
}

pub fn permits() -> usize {
    *PERMITS
}

fn refuse(kind: &'static str, why: &'static str) -> ApiError {
    METRICS.cpu_rejected(kind);
    tracing::warn!(
        kind,
        why,
        permits = *PERMITS,
        waiting = waiting(),
        wait_ms = wait_ms(),
        "CPU admission refused; shedding request"
    );
    ApiError::ServiceUnavailable
}

/// Take a permit, waiting a bounded time, or refuse when saturated by depth or
/// time. Tokio's semaphore is FIFO, so new arrivals can't starve a waiter.
async fn acquire(kind: &'static str) -> Result<SemaphorePermit<'static>, ApiError> {
    if let Ok(permit) = CPU.try_acquire() {
        METRICS.cpu_started(kind);
        return Ok(permit);
    }

    if WAIT.is_zero() {
        return Err(refuse(kind, "no permit free and waiting is disabled"));
    }

    // Counted BEFORE the check so two racing arrivals cannot both see room for
    // one slot, and dropped by the guard on every exit including cancellation.
    let depth = WAITING.fetch_add(1, Ordering::Relaxed) + 1;
    let _waiter = Waiter;
    if depth > *QUEUE_DEPTH {
        return Err(refuse(kind, "wait queue is full"));
    }

    match tokio::time::timeout(*WAIT, CPU.acquire()).await {
        Ok(Ok(permit)) => {
            METRICS.cpu_started(kind);
            Ok(permit)
        }
        Ok(Err(_)) => Err(refuse(kind, "permit pool closed")),
        Err(_) => Err(refuse(kind, "timed out waiting for a permit")),
    }
}

/// Runs synchronous CPU-bound work on the blocking pool under admission control.
/// The closure owns its inputs: clone the `AppState` (an `Arc`) and move the body in.
pub async fn run<F, T>(kind: &'static str, work: F) -> Result<T, ApiError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let permit = acquire(kind).await?;
    let started = Instant::now();

    let outcome = tokio::task::spawn_blocking(work).await;

    METRICS.cpu_finished(kind, started.elapsed().as_micros() as u64);
    drop(permit);

    outcome.map_err(|e| {
        tracing::error!(kind, error = %e, "CPU task panicked");
        ApiError::Internal(anyhow::anyhow!("{kind} task failed: {e}"))
    })
}

/// Runs a CPU-bound section on the blocking pool for a service that already holds
/// an [`Admission`] and loads before it computes, so can't hand it all to [`run`].
///
/// For isolation, not throughput: an inline search blocks its async worker, and
/// every future parked there (other requests' DB lookups included) waits it out.
/// Measured before this existed: a 0.2 s planner request took 33 s behind an
/// improvements search on its worker. No permit taken; the caller's admission
/// already bounds concurrency.
pub async fn offload<F, T>(kind: &'static str, work: F) -> Result<T, ApiError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(work).await.map_err(|e| {
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
pub async fn admit(kind: &'static str) -> Result<Admission, ApiError> {
    let permit = acquire(kind).await?;
    Ok(Admission {
        kind,
        started: Instant::now(),
        _permit: permit,
    })
}

#[cfg(test)]
mod tests {
    use super::{admit, permits, run, waiting};
    use std::time::Duration;

    /// The pool is one process-wide semaphore and the test harness runs tests
    /// on parallel threads, so two tests that take permits at once see each
    /// other's holdings: on a 4-core runner (2 permits) the saturation test
    /// failed its FIRST acquire while the round-trip test was mid-loop. Every
    /// test that touches the pool holds this for its duration.
    static POOL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[test]
    fn permits_are_at_least_one() {
        assert!(permits() >= 1, "a zero-permit pool would refuse everything");
    }

    #[tokio::test]
    async fn work_runs_and_the_permit_comes_back() {
        let _pool = POOL.lock().await;
        // More passes than there are permits, so a leaked permit shows up as a
        // refusal before the loop ends.
        for i in 0..(permits() * 4) {
            let got = run("test", move || i * 2).await.expect("admitted");
            assert_eq!(got, i * 2);
        }
    }

    /// Over the limit, a caller WAITS and is served when a permit comes back,
    /// rather than taking a 503 while the box still has capacity a moment later.
    #[tokio::test]
    async fn a_waiter_is_served_when_a_permit_returns() {
        let _pool = POOL.lock().await;
        let mut held = Vec::new();
        for _ in 0..permits() {
            held.push(admit("test").await.expect("under the limit"));
        }

        let queued = tokio::spawn(async { admit("test").await.map(drop) });
        // Let it reach the wait before anything is released, so this proves the
        // permit was handed over rather than taken on the fast path.
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(waiting(), 1, "the caller should be queued, not refused");

        drop(held);
        assert!(
            queued.await.expect("task joined").is_ok(),
            "a queued caller must be served once a permit frees"
        );
        assert_eq!(waiting(), 0, "the queue must drain");
    }

    /// The handler future is dropped on disconnect and on timeout, exactly when a
    /// post-await decrement would be skipped. A leaked count is permanent (the depth
    /// cap ends up refusing everything), so this guards the `Waiter` Drop impl.
    #[tokio::test]
    async fn the_waiting_count_survives_a_cancelled_waiter() {
        let _pool = POOL.lock().await;
        let mut held = Vec::new();
        for _ in 0..permits() {
            held.push(admit("test").await.expect("under the limit"));
        }

        let abandoned = tokio::spawn(async { admit("test").await.map(drop) });
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(waiting(), 1);

        abandoned.abort();
        let _ = abandoned.await;
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(
            waiting(),
            0,
            "a cancelled waiter must release its slot, or the cap wedges shut"
        );

        drop(held);
        assert!(
            admit("test").await.is_ok(),
            "permits return when guards drop"
        );
    }
}
