#![allow(dead_code, clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use myrtle_app::AppContext;
use myrtle_app::audit::Auditor;
use myrtle_app::cache::Cache;
use myrtle_app::config::{Config, render_example};
use myrtle_app::context::Secrets;
use myrtle_app::limits::CpuLimiter;
use myrtle_app::policy::{Policy, Subject};
use myrtle_app::services::accounts::credentials::CredentialKey;
use myrtle_app::services::accounts::sessions::Sessions;
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_db::repos::users;
use myrtle_gamedata::compile::CompileOptions;
use myrtle_hypergryph::{DeviceIds, GameAccounts, NetworkConfig, Upstream};
use myrtle_jobs::PgQueue;
use myrtle_testkit::db::{TestDb, database};
use myrtle_types::{Role, Server, UserId};

pub const JWT_SECRET: &str = "test-jwt-secret-0123456789abcdef";

pub fn config() -> Config {
    toml::from_str(&render_example()).unwrap()
}

pub async fn context() -> (AppContext, TestDb) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let test_db = database().await;
    let db = test_db.db.clone();
    let config = config();
    let cache = Cache::memory();
    let secrets = Secrets {
        jwt: JWT_SECRET.into(),
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

pub async fn user(ctx: &AppContext, id: UserId, role: Role) -> Subject {
    let account = users::account(&mut ctx.db.conn().await.unwrap(), id)
        .await
        .unwrap();
    let (uid, server) = account.map_or_else(
        || (id.0.to_string(), Server::EN),
        |a| (a.uid, Server::parse(&a.server).unwrap()),
    );
    Subject::User {
        id,
        uid: uid.into(),
        server: Some(server),
        role,
        grants: [].into(),
    }
}

pub fn snapshot() -> Option<Arc<Snapshot>> {
    static SNAPSHOT: OnceLock<Option<Arc<Snapshot>>> = OnceLock::new();
    SNAPSHOT
        .get_or_init(|| {
            let root = std::env::var_os("MYRTLE_TEST_ASSETS").map_or_else(
                || {
                    PathBuf::from(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../../assets/output"
                    ))
                },
                PathBuf::from,
            );
            let root = root.join("en");
            if !root.join("gamedata/excel/character_table.json").is_file() {
                return None;
            }
            Snapshot::load(&root, &CompileOptions::new(Server::EN))
                .ok()
                .map(Arc::new)
        })
        .clone()
}

pub async fn context_with_data() -> Option<(AppContext, TestDb, Arc<Snapshot>)> {
    let snapshot = snapshot()?;
    let (ctx, db) = context().await;
    ctx.snapshots
        .install_shared(Server::EN, Arc::clone(&snapshot));
    Some((ctx, db, snapshot))
}
