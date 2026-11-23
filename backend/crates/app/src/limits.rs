use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::AppError;
use crate::config::CpuConfig;

#[derive(Debug)]
struct Pools {
    heavy: Arc<Semaphore>,
    light: Arc<Semaphore>,
    permits: usize,
    wait: Duration,
    queue_depth: usize,
    waiting: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct CpuLimiter {
    pools: Arc<Pools>,
}

#[derive(Debug)]
pub struct Admission {
    _permit: OwnedSemaphorePermit,
}

struct Waiter<'a>(&'a AtomicUsize);

impl Drop for Waiter<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for CpuLimiter {
    fn default() -> Self {
        Self::new(&CpuConfig {
            task_permits: 0,
            light_permits: 0,
            task_wait_ms: 8_000,
            task_queue: 0,
        })
    }
}

impl CpuLimiter {
    #[must_use]
    pub fn new(config: &CpuConfig) -> Self {
        let or = |n: usize, fallback: usize| if n == 0 { fallback } else { n };
        let cores = std::thread::available_parallelism().map_or(2, NonZero::get);
        let permits = or(config.task_permits, (cores / 2).max(1));
        Self {
            pools: Arc::new(Pools {
                heavy: Arc::new(Semaphore::new(permits)),
                light: Arc::new(Semaphore::new(or(config.light_permits, permits))),
                permits,
                wait: Duration::from_millis(config.task_wait_ms),
                queue_depth: or(config.task_queue, permits * 8),
                waiting: AtomicUsize::new(0),
            }),
        }
    }

    #[must_use]
    pub fn permits(&self) -> usize {
        self.pools.permits
    }

    #[must_use]
    pub fn waiting(&self) -> usize {
        self.pools.waiting.load(Ordering::Relaxed)
    }

    pub async fn run<T, F>(&self, label: &'static str, work: F) -> Result<T, AppError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let admission = self.admit(label).await?;
        let held = move || {
            let _admission = admission;
            work()
        };
        self.offload(label, held).await
    }

    pub async fn offload<T, F>(&self, label: &'static str, work: F) -> Result<T, AppError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        tokio::task::spawn_blocking(work).await.map_err(|e| {
            tracing::error!(label, error = %e, "CPU task panicked");
            AppError::Internal(format!("{label} task failed: {e}"))
        })
    }

    pub async fn admit(&self, label: &'static str) -> Result<Admission, AppError> {
        self.acquire(&self.pools.heavy, label).await
    }

    pub async fn admit_light(&self, label: &'static str) -> Result<Admission, AppError> {
        self.acquire(&self.pools.light, label).await
    }

    async fn acquire(
        &self,
        pool: &Arc<Semaphore>,
        label: &'static str,
    ) -> Result<Admission, AppError> {
        if let Ok(permit) = Arc::clone(pool).try_acquire_owned() {
            return Ok(Admission { _permit: permit });
        }
        let pools = &self.pools;
        if pools.wait.is_zero() {
            return Err(self.refuse(label, "no permit free and waiting is disabled"));
        }
        let depth = pools.waiting.fetch_add(1, Ordering::Relaxed) + 1;
        let _waiter = Waiter(&pools.waiting);
        if depth > pools.queue_depth {
            return Err(self.refuse(label, "wait queue is full"));
        }
        match tokio::time::timeout(pools.wait, Arc::clone(pool).acquire_owned()).await {
            Ok(Ok(permit)) => Ok(Admission { _permit: permit }),
            Ok(Err(_)) => Err(self.refuse(label, "permit pool closed")),
            Err(_) => Err(self.refuse(label, "timed out waiting for a permit")),
        }
    }

    fn refuse(&self, label: &'static str, why: &'static str) -> AppError {
        tracing::warn!(
            label,
            why,
            permits = self.pools.permits,
            waiting = self.waiting(),
            "CPU admission refused; shedding request"
        );
        AppError::Unavailable
    }
}
