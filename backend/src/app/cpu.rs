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
//! Over the limit, a request WAITS a bounded time for a permit and is refused
//! only if none frees up, or if too many are already waiting. Refusing instantly
//! was the previous behaviour and it is wrong for a user-facing page: on a 3 core
//! box the permit formula below yields ONE, so a second reader of
//! `/api/user/improvements` got a 503 while the first was still computing. A
//! reader will happily wait a few hundred milliseconds; they will not accept an
//! error. The queue is bounded in both directions, by time and by depth, so this
//! is still shedding rather than an unbounded backlog of callers who have gone
//! away. Waiting on a semaphore yields, so a waiter does not hold an async
//! worker; it holds its connection and its request state, which is what the
//! depth cap protects.

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
/// Sized against how long the work actually takes: `/admin/stats` reports
/// `sum_micros` and `started` per kind, and their quotient is the mean hold time.
/// The wait wants to be a small multiple of that, so a burst drains instead of
/// shedding, while a genuinely saturated box still sheds rather than queueing
/// past the 30s handler timeout in `middleware`.
///
/// `CPU_TASK_WAIT_MS=0` restores the previous behaviour EXACTLY: no wait, refuse
/// the moment no permit is free.
static WAIT: LazyLock<Duration> = LazyLock::new(|| {
    let ms = std::env::var("CPU_TASK_WAIT_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(2_500);
    Duration::from_millis(ms)
});

/// How many requests may be waiting for a permit at once.
///
/// Without a ceiling, a slow spell converts into a queue that grows for as long
/// as traffic arrives, every entry holding a connection and its request state,
/// and the whole queue then times out together. Eight per permit is a TRADE, not
/// a derived number: deep enough to absorb the bursts this endpoint actually
/// sees, shallow enough that the memory is bounded and the tail waiter still has
/// a realistic chance of being served inside WAIT.
static QUEUE_DEPTH: LazyLock<usize> = LazyLock::new(|| {
    std::env::var("CPU_TASK_QUEUE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| *PERMITS * 8)
});

static WAITING: AtomicUsize = AtomicUsize::new(0);

/// Keeps `WAITING` honest when a waiter goes away.
///
/// Axum drops the handler future when the client disconnects or the handler
/// timeout fires, so a plain decrement after the await would be skipped on
/// exactly the paths that matter, and the counter would climb until the depth
/// cap refused everything forever.
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

/// Take a permit, waiting a bounded time for one, or refuse.
///
/// Three outcomes, in order of how common they should be: a permit is free and
/// the caller proceeds immediately; none is free so the caller queues and is
/// served when one returns; or the box is saturated, by depth or by time, and
/// the caller is refused. Tokio's semaphore is FIFO, so waiters are served in
/// arrival order and a steady stream of new requests cannot starve one that has
/// been waiting.
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

/// Run synchronous CPU-bound work on the blocking pool, under admission
/// control. The closure must own what it needs: clone the `AppState` (it is an
/// `Arc` behind the scenes) and move the request body in.
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

/// Runs a CPU-bound section on the blocking pool for a service that ALREADY
/// holds an [`Admission`] and cannot hand its whole body to [`run`] because it
/// loads first and computes after.
///
/// This is the second half of the load-then-compute split the module doc asks
/// for. The point is not throughput but isolation: a search that runs inline
/// on an async worker blocks that worker for its whole duration, and every
/// future parked there - another request's database lookups included - waits
/// it out. Measured before this existed: a 0.2 s planner request took 33 s
/// while an improvements search ran on the worker it had landed on. No permit
/// is taken here; the caller's admission already bounds concurrency.
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

    /// The behaviour this module exists to provide now: over the limit, a caller
    /// WAITS and is served when a permit comes back, rather than taking a 503
    /// while the box still has work capacity a moment later.
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

    /// Axum drops the handler future on client disconnect and on the handler
    /// timeout, which is exactly when a decrement placed after the await would be
    /// skipped. A leaked count is permanent: it climbs until the depth cap
    /// refuses every request forever, so this guards the `Waiter` Drop impl.
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
