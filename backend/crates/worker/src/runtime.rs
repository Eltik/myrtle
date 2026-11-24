use std::time::Duration;

use myrtle_app::{AppContext, AppError};
use myrtle_jobs::{Job, JobKind, JobQueue, PgQueue, QueueError, schedule};
use myrtle_types::WorkerId;
use tokio::task::JoinSet;
use tokio::time::{Instant, MissedTickBehavior};
use uuid::Uuid;

use crate::handlers;

const POLL: Duration = Duration::from_secs(2);
const UPKEEP: Duration = Duration::from_secs(15);
const PRUNE_EVERY: Duration = Duration::from_hours(1);
const RETENTION: Duration = Duration::from_hours(24 * 7);
const DRAIN: Duration = Duration::from_secs(30);
const BACKFILL_SLOTS: usize = 2;
const LONG_SLOTS: usize = 2;
const SHORT_SLOTS: usize = 4;
const RECORD_TRIES: u32 = 3;

struct Lane {
    kinds: Vec<JobKind>,
    slots: usize,
    busy: usize,
}

pub struct Runtime {
    ctx: AppContext,
    queue: PgQueue,
    worker: WorkerId,
    lanes: Vec<Lane>,
}

impl Runtime {
    #[must_use]
    pub fn new(ctx: AppContext, queue: PgQueue) -> Self {
        let per_user = [JobKind::RegradeUser, JobKind::SandboxBackfill];
        let short_lease = JobKind::MaintenancePoll.lease_secs();
        let (short, long) = JobKind::ALL
            .into_iter()
            .filter(|kind| !per_user.contains(kind))
            .partition(|kind| kind.lease_secs() <= short_lease);
        let lane = |kinds, slots| Lane {
            kinds,
            slots,
            busy: 0,
        };
        let regrades = ctx.config.jobs.regrade.concurrency.max(1);
        let lanes = vec![
            lane(vec![JobKind::RegradeUser], regrades),
            lane(vec![JobKind::SandboxBackfill], BACKFILL_SLOTS),
            lane(long, LONG_SLOTS),
            lane(short, SHORT_SLOTS),
        ];
        Self {
            ctx,
            queue,
            worker: WorkerId(Uuid::new_v4()),
            lanes,
        }
    }

    pub async fn run(mut self, shutdown: impl Future<Output = ()>) -> Result<(), AppError> {
        let mut upkeep = tokio::spawn(upkeep(self.ctx.clone(), self.queue.clone()));
        let mut running: JoinSet<usize> = JoinSet::new();
        let mut poll = tokio::time::interval(POLL);
        poll.set_missed_tick_behavior(MissedTickBehavior::Delay);
        tokio::pin!(shutdown);
        tracing::info!(worker = %self.worker.0, "claiming jobs");

        let stopped = loop {
            tokio::select! {
                () = &mut shutdown => break Ok(()),
                ended = &mut upkeep => {
                    break Err(AppError::Internal(format!("upkeep stopped: {ended:?}")));
                }
                Some(finished) = running.join_next() => match finished {
                    Ok(lane) => {
                        self.lanes[lane].busy -= 1;
                        self.claim(lane, &mut running).await;
                    }
                    Err(e) => break Err(AppError::internal(e)),
                },
                _ = poll.tick() => {
                    for lane in 0..self.lanes.len() {
                        self.claim(lane, &mut running).await;
                    }
                }
            }
        };

        upkeep.abort();
        if !running.is_empty() {
            tracing::info!(jobs = running.len(), "draining running jobs");
            let drain = async { while running.join_next().await.is_some() {} };
            if tokio::time::timeout(DRAIN, drain).await.is_err() {
                tracing::warn!(
                    jobs = running.len(),
                    "drain timed out; the reaper requeues what is left"
                );
            }
        }
        stopped
    }

    async fn claim(&mut self, index: usize, running: &mut JoinSet<usize>) {
        let lane = &mut self.lanes[index];
        let free = u16::try_from(lane.slots - lane.busy).unwrap_or(u16::MAX);
        let jobs = match self.queue.claim(self.worker, &lane.kinds, free).await {
            Ok(jobs) => jobs,
            Err(e) => {
                tracing::warn!(error = %e, "job claim failed");
                return;
            }
        };
        lane.busy += jobs.len();
        for job in jobs {
            let (ctx, queue, worker) = (self.ctx.clone(), self.queue.clone(), self.worker);
            running.spawn(async move {
                execute(ctx, &queue, worker, job).await;
                index
            });
        }
    }
}

async fn execute(ctx: AppContext, queue: &PgQueue, worker: WorkerId, job: Job) {
    let (id, kind, attempt) = (job.id, job.kind.as_str(), job.attempts);
    let started = Instant::now();
    let beat_every = Duration::from_secs(u64::from(job.lease_secs / 3).max(1));
    let mut beat = tokio::time::interval_at(started + beat_every, beat_every);
    let mut handler = tokio::spawn(async move { handlers::run(&ctx, &job).await });
    let outcome = loop {
        tokio::select! {
            outcome = &mut handler => break outcome,
            _ = beat.tick() => {
                match queue.heartbeat(worker, id, attempt).await {
                    Ok(()) => {}
                    Err(QueueError::LeaseLost(_)) => {
                        handler.abort();
                        tracing::warn!(job = id.0, kind, "lease lost, job abandoned");
                        return;
                    }
                    Err(e) => tracing::warn!(job = id.0, kind, error = %e, "heartbeat failed"),
                }
            }
        }
    };

    let took_ms = started.elapsed().as_millis();
    let result = match outcome {
        Ok(Ok(summary)) => {
            tracing::info!(job = id.0, kind, took_ms, summary, "job done");
            Ok(summary)
        }
        Ok(Err(e)) => {
            tracing::warn!(job = id.0, kind, attempt, took_ms, error = %e, "job failed");
            Err(e.to_string())
        }
        Err(e) => {
            tracing::error!(job = id.0, kind, attempt, error = %e, "job panicked");
            Err(format!("handler panicked: {e}"))
        }
    };
    for tried in 1..=RECORD_TRIES {
        let recorded = match &result {
            Ok(summary) => {
                let output = Some(summary.as_str().into());
                queue.complete(worker, id, attempt, output).await
            }
            Err(error) => queue.fail(worker, id, attempt, error, None).await,
        };
        match recorded {
            Ok(()) => return,
            Err(QueueError::Db(e)) if tried < RECORD_TRIES => {
                tracing::warn!(job = id.0, kind, error = %e, "job outcome not recorded, retrying");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => {
                tracing::error!(job = id.0, kind, error = %e, "job outcome not recorded");
                return;
            }
        }
    }
}

async fn upkeep(ctx: AppContext, queue: PgQueue) {
    let mut tick = tokio::time::interval(UPKEEP);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut pruned: Option<Instant> = None;
    loop {
        tick.tick().await;
        if ctx.config.switches.background_jobs.enabled
            && let Err(e) = schedule::enqueue_due(&ctx.db).await
        {
            tracing::warn!(error = %e, "schedule tick failed");
        }
        match queue.reap_expired().await {
            Ok(0) => {}
            Ok(jobs) => tracing::warn!(jobs, "expired leases reaped"),
            Err(e) => tracing::warn!(error = %e, "lease reaper failed"),
        }
        if pruned.is_none_or(|at| at.elapsed() >= PRUNE_EVERY) {
            pruned = Some(Instant::now());
            match queue.prune(RETENTION).await {
                Ok(jobs) => tracing::debug!(jobs, "finished jobs pruned"),
                Err(e) => tracing::warn!(error = %e, "job prune failed"),
            }
        }
        let reloading = ctx.clone();
        let reloaded = tokio::spawn(async move { reloading.reload_if_changed().await }).await;
        match reloaded {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => tracing::warn!(error = %e, "game data generation check failed"),
            Err(e) => tracing::error!(error = %e, "game data reload panicked"),
        }
    }
}
