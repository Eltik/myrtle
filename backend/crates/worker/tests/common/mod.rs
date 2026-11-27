#![allow(dead_code, clippy::unwrap_used)]

use std::sync::Arc;
use std::time::Duration;

use myrtle_app::AppContext;
use myrtle_app::audit::Auditor;
use myrtle_app::cache::Cache;
use myrtle_app::config::{Config, render_example};
use myrtle_app::context::Secrets;
use myrtle_app::limits::CpuLimiter;
use myrtle_app::policy::Policy;
use myrtle_app::services::accounts::credentials::CredentialKey;
use myrtle_app::services::accounts::sessions::Sessions;
use myrtle_app::snapshots::SnapshotRegistry;
use myrtle_hypergryph::{DeviceIds, GameAccounts, NetworkConfig, Upstream};
use myrtle_jobs::{Job, JobFilter, JobKind, JobState, PgQueue};
use myrtle_testkit::db::{TestDb, database};
use myrtle_worker::Runtime;

pub fn config() -> Config {
    toml::from_str(&render_example()).unwrap()
}

pub async fn context(config: Config) -> (AppContext, TestDb) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let test_db = database().await;
    let db = test_db.db.clone();
    let cache = Cache::memory();
    let secrets = Secrets {
        jwt: "test-jwt-secret-0123456789abcdef".into(),
        credentials: Arc::new(CredentialKey::parse(&"ab".repeat(32)).unwrap()),
        service_key: None,
        discord_alert_webhook: None,
    };
    let sessions = Sessions::new(db.clone(), cache.clone(), Arc::clone(&secrets.credentials));
    let ctx = AppContext {
        policy: Policy::default(),
        cpu: CpuLimiter::new(&config.cpu),
        audit: Auditor { db: db.clone() },
        queue: Arc::new(PgQueue::new(db.clone())),
        snapshots: Arc::new(SnapshotRegistry::new(&config)),
        upstream: Arc::new(Upstream::new(
            reqwest::Client::new(),
            NetworkConfig::new(DeviceIds::default()),
        )),
        sessions: Arc::new(sessions),
        accounts: Arc::new(GameAccounts::default()),
        store: None,
        config: Arc::new(config),
        secrets: Arc::new(secrets),
        db,
        cache,
    };
    (ctx, test_db)
}

pub async fn jobs(queue: &PgQueue, kind: JobKind) -> Vec<Job> {
    let filter = JobFilter {
        state: None,
        kind: Some(kind),
        limit: 100,
    };
    queue.recent(filter).await.unwrap()
}

/// Runs the worker until no `kind` job is still on its first attempt
pub async fn run_until_attempted(ctx: &AppContext, kind: JobKind) -> Vec<Job> {
    let queue = PgQueue::new(ctx.db.clone());
    let attempted = async {
        loop {
            let jobs = jobs(&queue, kind).await;
            if jobs
                .iter()
                .all(|j| j.attempts > 0 && j.state != JobState::Running)
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    let run = Runtime::new(ctx.clone(), queue.clone()).run(attempted);
    tokio::time::timeout(Duration::from_secs(20), run)
        .await
        .expect("the worker attempts every job")
        .unwrap();
    jobs(&queue, kind).await
}
