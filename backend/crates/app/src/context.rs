use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::data_generations::{self, Bump, Generation};
use myrtle_db::{Db, PoolOptions};
use myrtle_hypergryph::{
    AccountOptions, DeviceIds, GameAccounts, NetworkConfig, SessionStore, Upstream,
};
use myrtle_jobs::{JobKind, JobQueue, JobSpec, PgQueue, enqueue_in};
use myrtle_storage::ObjectStore;
use myrtle_types::Server;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppError;
use crate::alerts;
use crate::audit::Auditor;
use crate::cache::Cache;
use crate::config::{Config, ConfigError, SecretRef};
use crate::limits::CpuLimiter;
use crate::policy::Policy;
use crate::services::accounts::credentials::CredentialKey;
use crate::services::accounts::sessions::Sessions;
use crate::services::catalog::release;
use crate::services::gacha::sidecar;
use crate::snapshots::{Snapshot, SnapshotRegistry};

pub struct Secrets {
    pub jwt: String,
    pub credentials: Arc<CredentialKey>,
    pub service_key: Option<String>,
    pub discord_alert_webhook: Option<String>,
}

impl Secrets {
    pub fn resolve(config: &Config) -> Result<Self, ConfigError> {
        let secrets = &config.secrets;
        let key = secrets.game_credential_key.resolve()?;
        let credentials = CredentialKey::parse(&key).map_err(|e| ConfigError::Invalid {
            key: "secrets.game_credential_key".into(),
            message: e.to_string(),
        })?;
        Ok(Self {
            jwt: secrets.jwt_secret.resolve()?,
            credentials: Arc::new(credentials),
            service_key: SecretRef::optional(secrets.service_key.as_ref()),
            discord_alert_webhook: SecretRef::optional(secrets.discord_alert_webhook.as_ref()),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Announced {
    #[serde(default)]
    pub generation: i64,
}

impl Announced {
    #[must_use]
    pub fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BootStep {
    Database,
    GameData(Server),
    Upstream,
    Accounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameData {
    All,
    Default,
    None,
}

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<Config>,
    pub secrets: Arc<Secrets>,
    pub db: Db,
    pub queue: Arc<dyn JobQueue>,
    pub cache: Cache,
    pub snapshots: Arc<SnapshotRegistry>,
    pub upstream: Arc<Upstream>,
    pub sessions: Arc<dyn SessionStore>,
    pub accounts: Arc<GameAccounts>,
    pub store: Option<Arc<dyn ObjectStore>>,
    pub policy: Policy,
    pub audit: Auditor,
    pub cpu: CpuLimiter,
}

impl AppContext {
    pub async fn build(config: Config) -> Result<Self, AppError> {
        Self::build_with(config, |_| {}).await
    }

    pub async fn build_with(
        config: Config,
        step: impl Fn(BootStep) + Send + Sync,
    ) -> Result<Self, AppError> {
        Self::open(config, GameData::All, step).await
    }

    pub async fn open(
        config: Config,
        game_data: GameData,
        step: impl Fn(BootStep) + Send + Sync,
    ) -> Result<Self, AppError> {
        step(BootStep::Database);
        let secrets = Secrets::resolve(&config).map_err(AppError::internal)?;
        let database_url = config.database.url.resolve().map_err(AppError::internal)?;
        let db = Db::connect(&PoolOptions {
            max_connections: config.database.max_connections,
            acquire_timeout: Duration::from_millis(config.database.acquire_timeout_ms),
            ..PoolOptions::new(database_url)
        })
        .await?;

        let redis_url = SecretRef::optional(config.redis.url.as_ref());
        let cache = Cache::connect(redis_url.as_deref()).await?;

        let snapshots = SnapshotRegistry::new(&config);
        let generations: HashMap<Server, Generation> =
            data_generations::all(&mut *db.conn().await?)
                .await?
                .into_iter()
                .collect();
        let wanted = match game_data {
            GameData::All => snapshots.servers(),
            GameData::Default => vec![snapshots.default_server()],
            GameData::None => Vec::new(),
        };
        for server in wanted {
            step(BootStep::GameData(server));
            let stored = generations.get(&server).cloned().unwrap_or_default();
            let activated = snapshots.activate_at(server, &stored).await;
            match activated {
                Ok(()) => {}
                Err(e) if server == snapshots.default_server() => return Err(e),
                Err(e) => {
                    tracing::warn!(server = server.as_str(), error = %e, "game data not loaded");
                }
            }
        }

        step(BootStep::Upstream);
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(AppError::internal)?;
        let upstream = Arc::new(Upstream::new(client, network(&config)));
        upstream.reload().await;
        upstream.spawn_network_retry(config.server.servers.clone());

        let sessions: Arc<dyn SessionStore> = Arc::new(Sessions::new(
            db.clone(),
            cache.clone(),
            Arc::clone(&secrets.credentials),
        ));
        step(BootStep::Accounts);
        let options = AccountOptions {
            secret_max_age: Duration::from_secs(config.hypergryph.service_session_max_age_secs),
        };
        let accounts = GameAccounts::load(&config.server.servers, &sessions, options).await;

        let store = config
            .object_storage()
            .map_err(AppError::internal)?
            .map(|storage| myrtle_storage::open_store(&storage))
            .transpose()
            .map_err(AppError::internal)?;

        let ctx = Self {
            policy: Policy {
                role_recheck: config.switches.role_recheck.enabled,
            },
            cpu: CpuLimiter::new(&config.cpu),
            audit: Auditor { db: db.clone() },
            queue: Arc::new(PgQueue::new(db.clone())),
            config: Arc::new(config),
            secrets: Arc::new(secrets),
            db,
            cache,
            snapshots: Arc::new(snapshots),
            upstream,
            sessions,
            accounts: Arc::new(accounts),
            store,
        };
        for server in ctx.snapshots.loaded() {
            if let Some(snapshot) = ctx.snapshots.get(server) {
                ctx.alert_degraded(&snapshot);
            }
            let ctx = ctx.clone();
            tokio::spawn(async move { crate::services::story::warm(&ctx, server).await });
        }
        Ok(ctx)
    }
}

impl AppContext {
    pub async fn reload_game_data(&self, server: Server) -> Result<(), AppError> {
        let generation = self.announce_update(server, None).await?;
        self.reload_if_changed().await?;
        match self.snapshots.get(server) {
            Some(live) if live.generation >= generation => Ok(()),
            _ => Err(AppError::Internal(format!(
                "game data for {} did not reload",
                server.as_str()
            ))),
        }
    }

    pub async fn announce_update(
        &self,
        server: Server,
        res_version: Option<&str>,
    ) -> Result<i64, AppError> {
        let server = server.data_server();
        let mut tx = self.db.begin().await?;
        let generation = match data_generations::bump(&mut tx, server, res_version).await? {
            Bump::Published(bumped) => bumped.generation,
            Bump::AlreadyPublished(held) => {
                tracing::info!(
                    server = server.as_str(),
                    generation = held.generation,
                    res_version,
                    "game data version already published, nothing announced"
                );
                return Ok(held.generation);
            }
        };
        let announced = Announced { generation };
        let mut jobs = release::jobs::follow_ups(self, server, &announced);
        if server == self.snapshots.default_server() {
            jobs.push(JobSpec {
                payload: announced.payload(),
                ..JobSpec::new(JobKind::RarityReconcile)
            });
        }
        for job in &jobs {
            enqueue_in(&mut tx, job).await?;
        }
        tx.commit().await?;
        tracing::info!(
            server = server.as_str(),
            generation,
            res_version,
            jobs = jobs.len(),
            "game data update announced"
        );
        Ok(generation)
    }

    pub async fn announce_art(&self, server: Server) -> Result<i64, AppError> {
        let server = server.data_server();
        let generation = data_generations::bump(&mut *self.db.conn().await?, server, None)
            .await?
            .current()
            .generation;
        tracing::info!(
            server = server.as_str(),
            generation,
            "release art announced"
        );
        Ok(generation)
    }

    pub async fn reload_if_changed(&self) -> Result<Vec<Server>, AppError> {
        let reloaded = {
            let _reloading = self.snapshots.reloading().await;
            let stored: HashMap<Server, Generation> =
                data_generations::all(&mut *self.db.conn().await?)
                    .await?
                    .into_iter()
                    .collect();
            let mut reloaded = Vec::new();
            for (server, stored) in self.snapshots.due(&stored) {
                let generation = stored.generation;
                if self.snapshots.get(server).map(|s| s.generation) != Some(generation) {
                    sidecar::take_pending(server); // before the load reads, so a write during it stays pending
                    let loaded = self.snapshots.load_at(server, &stored).await;
                    let snapshot = match loaded {
                        Ok(snapshot) => snapshot,
                        Err(e) => {
                            tracing::error!(server = server.as_str(), generation, error = %e, "game data reload failed, keeping the old data");
                            sidecar::mark_pending(server);
                            self.snapshots.failed(server, generation);
                            continue;
                        }
                    };
                    self.cache
                        .drop_shared(&format!("static:{}:", server.as_str())); // every other game-data key carries the generation
                    self.alert_degraded(&snapshot);
                    let ctx = self.clone();
                    tokio::spawn(async move { crate::services::story::warm(&ctx, server).await });
                    tracing::info!(server = server.as_str(), generation, "game data reloaded");
                    reloaded.push(server);
                }
                self.snapshots.done(server, generation);
            }
            reloaded
        };
        if reloaded.contains(&self.snapshots.default_server()) {
            self.upstream.reload().await;
        }
        Ok(reloaded)
    }

    #[must_use]
    pub fn table_warnings(&self, snapshot: &Snapshot) -> Vec<String> {
        let meta = &snapshot.model.meta;
        let has_account = self
            .accounts
            .servers()
            .iter()
            .any(|account| account.data_server() == meta.server);
        alerts::reportable_warnings(&meta.table_warnings, has_account)
    }

    fn alert_degraded(&self, snapshot: &Snapshot) {
        let meta = &snapshot.model.meta;
        let warnings = self.table_warnings(snapshot);
        if warnings.is_empty() {
            return;
        }
        let (ctx, server) = (self.clone(), meta.server);
        let sent = format!(
            "alert:degraded:{}:{}",
            server.as_str(),
            meta.game_version.res
        );
        tokio::spawn(async move {
            if ctx.cache.get(&sent).await.is_some() {
                return;
            }
            ctx.cache.set(&sent, "1", Duration::from_hours(24)).await;
            alerts::report_degraded_tables(
                ctx.upstream.client(),
                ctx.secrets.discord_alert_webhook.as_deref(),
                server.as_str(),
                &warnings,
            )
            .await;
        });
    }

    pub async fn snapshot_at(
        &self,
        server: Server,
        generation: i64,
    ) -> Result<Arc<Snapshot>, AppError> {
        if self
            .snapshots
            .get(server)
            .is_none_or(|s| s.generation < generation)
        {
            self.reload_if_changed().await?;
        }
        let snapshot = self.snapshots.require(server)?;
        if snapshot.generation < generation {
            return Err(AppError::Internal(format!(
                "game data for {} is at generation {}, not yet {generation}",
                server.as_str(),
                snapshot.generation
            )));
        }
        Ok(snapshot)
    }
}

fn network(config: &Config) -> NetworkConfig {
    let hypergryph = &config.hypergryph;
    let configured = DeviceIds {
        device_id: hypergryph.device_id.clone(),
        device_id2: hypergryph.device_id2.clone(),
        device_id3: hypergryph.device_id3.clone(),
    };
    let device_ids = if configured.is_complete() {
        configured
    } else {
        tracing::warn!("hypergryph.device_id* incomplete; using generated device ids for this run");
        DeviceIds::generate()
    };
    let mut network = NetworkConfig::new(device_ids);
    if !hypergryph.network_version_override.is_empty() {
        network.network_version_override = Some(hypergryph.network_version_override.clone());
    }
    if !hypergryph.cn_app_code.is_empty() {
        network.cn_app_code.clone_from(&hypergryph.cn_app_code);
    }
    network
}
