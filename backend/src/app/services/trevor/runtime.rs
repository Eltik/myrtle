//! Trevor's in-process state: its configuration, the wake-up a long-polling
//! worker waits on, the per-job broadcast channels SSE readers subscribe to,
//! and the Turnstile verifier.
//!
//! One backend process serves the queue, so all of this is in memory; a
//! restart loses only live streams (readers fall back to polling) and the
//! twin-join list.
use std::collections::HashSet;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::sync::{Notify, broadcast};
use uuid::Uuid;

use super::JobView;
use super::turnstile::{SiteVerify, TurnstileVerifier};
use crate::utils::env::switched_off;

/// Trevor's environment knobs and kill switches (read once at boot).
#[derive(Debug, Clone)]
pub struct TrevorConfig {
    /// `TREVOR_WORKER_KEY`: the only credential the worker routes accept
    /// (header `x-trevor-worker-key`). Empty disables them with a 503.
    pub worker_key: String,
    /// `TREVOR_SCOPED_WORKER_KEY=0` restores the old gate: the global service
    /// key (`x-service-key`) on the worker routes and the worker key nowhere.
    pub scoped_worker_key: bool,
    /// `TREVOR_NOTIFY_POLL=0` restores the old long-poll: one claim query a
    /// second and a stale sweep on every poll, no background sweep.
    pub notify_poll: bool,
    /// `TREVOR_ERA_BUCKETS=0` keys jobs and answers by the exact horizon
    /// gatePos instead of its era boundary.
    pub era_buckets: bool,
    /// `TREVOR_PANELS_IMMUTABLE=0` serves every panel response with the old
    /// `public, max-age=300`, `?v=` or not.
    pub panels_immutable: bool,
    /// `TREVOR_CACHE_LIVE_ONLY=0` caches passages-only answers again.
    pub cache_live_only: bool,
    /// `TREVOR_CACHE_VERSION_CHECK=0` caches an answer whatever corpus
    /// version it came from, not only the job's.
    pub version_check: bool,
    /// `TREVOR_TTL_FROM_QUEUED=0` expires queued jobs by `created_at`, so a
    /// requeued job keeps its first clock.
    pub ttl_from_queued: bool,
    /// `TREVOR_BUSY_PROTECTS=0` requeues a stale running job even while an
    /// online worker's heartbeat names it as its busy job.
    pub busy_protects: bool,
    /// `TREVOR_JOB_MAX_RUN_SECS`: a running job this old is requeued (once)
    /// or failed even while a live worker names it as busy; `0` lets the
    /// busy heartbeat protect it forever (the old behaviour).
    pub job_max_run_secs: i64,
    /// `TREVOR_JOB_OWNER_ONLY=0` lets any signed-in user poll any job id.
    pub job_owner_only: bool,
    /// `TREVOR_REVERDICT_ON_ASK=0` resolves an ask's horizon from the stored
    /// read rows only, as before: an account with stage progress but no game
    /// read rows and no site marks then asks at gatePos 0.
    pub reverdict_on_ask: bool,
    /// `TREVOR_QUEUE_TTL_SECS`: a queued job nobody claimed in this long fails.
    pub queue_ttl_secs: i64,
    /// `TREVOR_CF_DAILY_NEURONS`: the Cloudflare free allowance per UTC day.
    pub cf_daily_neurons: i64,
    /// `TREVOR_CF_SUMMARY_RESERVE`: Neurons kept back for the daily summaries.
    pub cf_summary_reserve: i64,
    /// `TREVOR_CPU_QUEUE_MAX`: jobs queued or running on the CPU rung before
    /// new ones fall to passages.
    pub cpu_queue_max: i64,
    /// `TREVOR_RESULT_MAX_BYTES`: a worker result body over this is a 413;
    /// 0 lifts the cap (the old behaviour, bounded only by axum's 2 MB).
    pub result_max_bytes: usize,
    /// `TURNSTILE_SECRET`: when set, ask verifies the request's Turnstile
    /// token and fails closed (503) when Cloudflare cannot be reached.
    pub turnstile_secret: Option<String>,
}

impl Default for TrevorConfig {
    /// The contract defaults, with no worker key and no Turnstile.
    fn default() -> Self {
        Self {
            worker_key: String::new(),
            scoped_worker_key: true,
            notify_poll: true,
            era_buckets: true,
            panels_immutable: true,
            cache_live_only: true,
            version_check: true,
            ttl_from_queued: true,
            busy_protects: true,
            job_max_run_secs: 1800,
            job_owner_only: true,
            reverdict_on_ask: true,
            queue_ttl_secs: 600,
            cf_daily_neurons: 9000,
            cf_summary_reserve: 1000,
            cpu_queue_max: 6,
            result_max_bytes: 64 * 1024,
            turnstile_secret: None,
        }
    }
}

fn env_num<T: std::str::FromStr>(name: &str, default: T) -> T {
    match std::env::var(name) {
        Ok(v) if !v.trim().is_empty() => v.trim().parse().unwrap_or_else(|_| {
            tracing::warn!(var = name, value = %v, "trevor: unparsable value; using the default");
            default
        }),
        _ => default,
    }
}

impl TrevorConfig {
    pub fn from_env() -> Self {
        let d = Self::default();
        let worker_key = std::env::var("TREVOR_WORKER_KEY").unwrap_or_default();
        if worker_key.trim().len() != worker_key.len() {
            tracing::warn!(
                "TREVOR_WORKER_KEY has leading or trailing whitespace; it is used verbatim"
            );
        }
        Self {
            worker_key,
            scoped_worker_key: !switched_off("TREVOR_SCOPED_WORKER_KEY"),
            notify_poll: !switched_off("TREVOR_NOTIFY_POLL"),
            era_buckets: !switched_off("TREVOR_ERA_BUCKETS"),
            panels_immutable: !switched_off("TREVOR_PANELS_IMMUTABLE"),
            cache_live_only: !switched_off("TREVOR_CACHE_LIVE_ONLY"),
            version_check: !switched_off("TREVOR_CACHE_VERSION_CHECK"),
            ttl_from_queued: !switched_off("TREVOR_TTL_FROM_QUEUED"),
            busy_protects: !switched_off("TREVOR_BUSY_PROTECTS"),
            job_max_run_secs: env_num("TREVOR_JOB_MAX_RUN_SECS", d.job_max_run_secs).max(0),
            job_owner_only: !switched_off("TREVOR_JOB_OWNER_ONLY"),
            reverdict_on_ask: !switched_off("TREVOR_REVERDICT_ON_ASK"),
            queue_ttl_secs: env_num("TREVOR_QUEUE_TTL_SECS", d.queue_ttl_secs).max(1),
            cf_daily_neurons: env_num("TREVOR_CF_DAILY_NEURONS", d.cf_daily_neurons),
            cf_summary_reserve: env_num("TREVOR_CF_SUMMARY_RESERVE", d.cf_summary_reserve),
            cpu_queue_max: env_num("TREVOR_CPU_QUEUE_MAX", d.cpu_queue_max),
            result_max_bytes: env_num("TREVOR_RESULT_MAX_BYTES", d.result_max_bytes),
            turnstile_secret: std::env::var("TURNSTILE_SECRET")
                .ok()
                .filter(|s| !s.trim().is_empty()),
        }
    }
}

/// One message on a job's broadcast channel.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Streamed answer text, in the worker's `seq` order.
    Delta { seq: u64, text: String },
    /// The job finished; the view carries the answer.
    Done(Box<JobView>),
    /// The job failed.
    Error(String),
}

impl StreamEvent {
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Done(_) | Self::Error(_))
    }
}

/// Deltas a slow SSE reader may fall behind by before it skips ahead.
const CHANNEL_CAPACITY: usize = 512;
/// How long a twin joiner may open the job's stream.
const JOINER_TTL: Duration = Duration::from_secs(3600);

pub struct TrevorRuntime {
    config: RwLock<Arc<TrevorConfig>>,
    /// Signalled after every enqueue (and requeue): wakes `worker/next`.
    pub notify: Notify,
    streams: DashMap<Uuid, broadcast::Sender<StreamEvent>>,
    /// Askers who joined another user's identical job, so they may stream it.
    joiners: DashMap<Uuid, (Instant, HashSet<Uuid>)>,
    /// Jobs running here and their worker, so a delta costs no query.
    running: DashMap<Uuid, String>,
    verifier: RwLock<Arc<dyn TurnstileVerifier>>,
    /// Claim queries `worker/next` has issued: an idle poll costs one under
    /// the `Notify` wake-up and about 21 under `TREVOR_NOTIFY_POLL=0`.
    pub claim_queries: AtomicU64,
    /// Accounts whose read verdict an ask has already tried to re-derive, so
    /// one whose stage progress legitimately yields no reads costs one
    /// re-derivation per process, not one per ask. 16 bytes per asker.
    reverdict_tried: DashMap<Uuid, ()>,
    /// Re-derivations an ask has started (tests).
    pub reverdicts: AtomicU64,
}

impl TrevorRuntime {
    pub fn new(config: TrevorConfig, verifier: Arc<dyn TurnstileVerifier>) -> Self {
        Self {
            config: RwLock::new(Arc::new(config)),
            notify: Notify::new(),
            streams: DashMap::new(),
            joiners: DashMap::new(),
            running: DashMap::new(),
            verifier: RwLock::new(verifier),
            claim_queries: AtomicU64::new(0),
            reverdict_tried: DashMap::new(),
            reverdicts: AtomicU64::new(0),
        }
    }

    pub fn from_env(client: reqwest::Client) -> Self {
        Self::new(TrevorConfig::from_env(), Arc::new(SiteVerify::new(client)))
    }

    pub fn config(&self) -> Arc<TrevorConfig> {
        self.config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Replace the configuration (tests; the server reads it once at boot).
    pub fn set_config(&self, config: TrevorConfig) {
        *self
            .config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::new(config);
    }

    pub fn verifier(&self) -> Arc<dyn TurnstileVerifier> {
        self.verifier
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Replace the Turnstile verifier (tests use a fake; no network).
    pub fn set_verifier(&self, verifier: Arc<dyn TurnstileVerifier>) {
        *self
            .verifier
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = verifier;
    }

    /// A job is claimable: wake every waiting `worker/next`.
    pub fn job_enqueued(&self) {
        self.notify.notify_waiters();
    }

    /// Subscribe to a job's stream, creating its channel.
    pub fn subscribe(&self, job_id: Uuid) -> broadcast::Receiver<StreamEvent> {
        self.streams
            .entry(job_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe()
    }

    /// Fan a delta out to the job's subscribers; nothing when there are none.
    pub fn publish(&self, job_id: Uuid, event: StreamEvent) {
        if let Some(tx) = self.streams.get(&job_id) {
            let _ = tx.send(event);
        }
    }

    /// The job ended: tell its subscribers and drop its channel and run mark.
    pub fn finish(&self, job_id: Uuid, event: StreamEvent) {
        self.running.remove(&job_id);
        if let Some((_, tx)) = self.streams.remove(&job_id) {
            let _ = tx.send(event);
        }
    }

    /// Subscribers on a job's channel (tests and the prune).
    pub fn subscribers(&self, job_id: Uuid) -> usize {
        self.streams
            .get(&job_id)
            .map_or(0, |tx| tx.receiver_count())
    }

    pub fn mark_running(&self, job_id: Uuid, worker_id: &str) {
        self.running.insert(job_id, worker_id.to_owned());
    }

    pub fn clear_running(&self, job_id: Uuid) {
        self.running.remove(&job_id);
    }

    /// The worker a job runs under here, if this process saw the claim.
    pub fn running_worker(&self, job_id: Uuid) -> Option<String> {
        self.running.get(&job_id).map(|w| w.clone())
    }

    pub fn add_joiner(&self, job_id: Uuid, user_id: Uuid) {
        self.joiners
            .entry(job_id)
            .or_insert_with(|| (Instant::now(), HashSet::new()))
            .1
            .insert(user_id);
    }

    pub fn is_joiner(&self, job_id: Uuid, user_id: Uuid) -> bool {
        self.joiners
            .get(&job_id)
            .is_some_and(|j| j.1.contains(&user_id))
    }

    /// True the first time it is called for an account in this process: the
    /// one re-derivation an ask may start for it.
    pub fn first_reverdict(&self, user_id: Uuid) -> bool {
        self.reverdict_tried.insert(user_id, ()).is_none()
    }

    /// Drop channels nobody listens to and joiner lists past their hour.
    pub fn prune(&self) {
        self.streams.retain(|_, tx| tx.receiver_count() > 0);
        self.joiners.retain(|_, (at, _)| at.elapsed() < JOINER_TTL);
    }
}
