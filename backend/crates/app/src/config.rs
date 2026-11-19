//! One typed configuration, loaded once at boot from `config/myrtle.toml`;
//! secrets are references, resolved when asked for.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use myrtle_types::Server;
use serde::Deserialize;

/// `"env:NAME"` or `"file:/path"`; the value itself never appears in the file
#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct SecretRef(pub String);

impl std::fmt::Debug for SecretRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecretRef({})", self.0)
    }
}

impl SecretRef {
    /// An unset optional secret reads as absent
    #[must_use]
    pub fn optional(secret: Option<&Self>) -> Option<String> {
        secret?.resolve().ok()
    }

    /// An empty value is an error
    pub fn resolve(&self) -> Result<String, ConfigError> {
        let unresolved = || ConfigError::Secret {
            reference: self.0.clone(),
        };
        let value = if let Some(name) = self.0.strip_prefix("env:") {
            std::env::var(name).map_err(|_| unresolved())?
        } else if let Some(path) = self.0.strip_prefix("file:") {
            let text = std::fs::read_to_string(path).map_err(|_| unresolved())?;
            text.trim_end_matches(['\r', '\n']).to_owned()
        } else {
            return Err(unresolved());
        };
        if value.trim().is_empty() {
            return Err(unresolved());
        }
        Ok(value)
    }
}

/// A kill switch; `owner` and `remove_after` are mandatory so each has a person and a deadline
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Switch {
    pub enabled: bool,
    pub owner: String,
    /// `YYYY-MM-DD`, past which `/health` reports the switch as expired
    pub remove_after: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub secrets: SecretsConfig,
    pub rate_limit: RateLimitConfig,
    pub cpu: CpuConfig,
    pub assets: AssetsConfig,
    pub storage: StorageConfig,
    pub hypergryph: HypergryphConfig,
    pub jobs: JobsConfig,
    pub dps: DpsConfig,
    pub release: ReleaseConfig,
    pub story: StoryConfig,
    pub cli: CliConfig,
    pub tools: ToolsConfig,
    pub debug: DebugConfig,
    pub switches: Switches,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub bind_host: String,
    pub port: u16,
    pub metrics_bind: String,
    pub trusted_proxy_hops: u8,
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    pub url: SecretRef,
    pub max_connections: u32,
    pub acquire_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedisConfig {
    /// Empty: the in-memory cache
    pub url: Option<SecretRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretsConfig {
    pub jwt_secret: SecretRef,
    pub game_credential_key: SecretRef,
    /// Superseded by scoped `api_keys`
    pub service_key: Option<SecretRef>,
    pub github_token: Option<SecretRef>,
    pub discord_alert_webhook: Option<SecretRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    pub rpm: u32,
    pub exempt_ips: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpuConfig {
    pub task_permits: usize,
    pub light_permits: usize,
    pub task_wait_ms: u64,
    pub task_queue: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetsConfig {
    pub dir: String,
    pub allow_all_dirs: bool,
    pub stream_chunk_bytes: usize,
    pub ws_urls: BTreeMap<String, String>,
    pub delivery: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub kind: String,
    pub bucket: String,
    pub access_key_id: Option<SecretRef>,
    pub secret_access_key: Option<SecretRef>,
    pub account_id: String,
    pub endpoint: String,
    pub prefix: String,
    pub public_base_url: String,
    pub local_dir: String,
    pub roots: Vec<String>,
    pub cache_control: String,
    pub sync_concurrency: usize,
    pub sync_hash_threads: usize,
    pub multipart_threshold_bytes: u64,
    pub evict_local: bool,
    pub evict_keep: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HypergryphConfig {
    pub network_version_override: String,
    pub cn_app_code: String,
    pub device_id: String,
    pub device_id2: String,
    pub device_id3: String,
    pub service_session_max_age_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobsConfig {
    pub stagger_secs: u64,
    pub trending: TrendingJob,
    pub regrade: RegradeJob,
    pub gacha_detail: UpstreamPollJob,
    pub event_shop: UpstreamPollJob,
    pub maintenance: MaintenanceJob,
    pub siren: SirenJob,
    pub dps_watch: DpsWatchJob,
    pub sandbox_backfill: SandboxBackfillJob,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxBackfillJob {
    pub switch_wait_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DpsConfig {
    #[serde(deserialize_with = "off_grid_ttl")]
    pub rank_off_grid_ttl_secs: u64,
}

const OFF_GRID_TTL_DEFAULT: u64 = 3600;

/// Zero warns and reads as the default rather than refusing the boot
fn off_grid_ttl<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let secs = u64::deserialize(deserializer)?;
    if secs == 0 {
        tracing::warn!("dps.rank_off_grid_ttl_secs is 0; using {OFF_GRID_TTL_DEFAULT}");
        return Ok(OFF_GRID_TTL_DEFAULT);
    }
    Ok(secs)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrendingJob {
    pub every_secs: u64,
    pub top_n: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegradeJob {
    pub every_secs: u64,
    pub concurrency: usize,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamPollJob {
    pub every_secs: u64,
    pub call_delay_ms: u64,
    pub max_failures: u32,
    pub checkpoint_every: Option<usize>,
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaintenanceJob {
    pub poll_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SirenJob {
    pub every_secs: u64,
    pub api_base: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DpsWatchJob {
    pub every_secs: u64,
    pub upstream_repo: String,
    pub upstream_branch: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseConfig {
    pub art_mirrors: Vec<String>,
    pub lag_window: usize,
    pub lag_half_life: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoryConfig {
    pub art_min_bytes_per_px: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliConfig {
    pub default_server: String,
    pub admin_uid: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolsConfig {
    pub python_bin: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub base_pin_trace: bool,
    pub base_rot_timing: bool,
    pub sync_dump_dir: String,
    pub story_thumb_out: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Switches {
    pub background_jobs: Switch,
    pub role_recheck: Switch,
    pub release_ledger: Switch,
    pub release_trust_skipped_tags: Switch,
    pub release_pooled_cadence: Switch,
    pub release_keep_unsold_skins: Switch,
    pub release_reviews_all_groups: Switch,
    pub sidecar_reload_always: Switch,
    pub asset_cn_fallback: Switch,
    pub art_fallback: Switch,
    pub obtain_channel_align: Switch,
    pub profile_align: Switch,
    pub grade_invested_only: Switch,
    pub grade_legacy_rarity_weights: Switch,
    pub grade_module_per_slot: Switch,
    pub story_hub: Switch,
    pub story_hub_fallback: Switch,
    pub story_whole_body: Switch,
    pub sandbox_topic_backfill: Switch,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config file {path}: {message}")]
    File { path: String, message: String },
    #[error("secret {reference} could not be resolved")]
    Secret { reference: String },
    #[error("{key}: {message}")]
    Invalid { key: String, message: String },
}

#[derive(Debug, Clone)]
pub struct ExpiredSwitch {
    pub name: &'static str,
    pub owner: String,
    pub remove_after: String,
}

impl Switches {
    #[must_use]
    pub const fn all(&self) -> [(&'static str, &Switch); 19] {
        [
            ("background_jobs", &self.background_jobs),
            ("role_recheck", &self.role_recheck),
            ("release_ledger", &self.release_ledger),
            (
                "release_trust_skipped_tags",
                &self.release_trust_skipped_tags,
            ),
            ("release_pooled_cadence", &self.release_pooled_cadence),
            ("release_keep_unsold_skins", &self.release_keep_unsold_skins),
            (
                "release_reviews_all_groups",
                &self.release_reviews_all_groups,
            ),
            ("sidecar_reload_always", &self.sidecar_reload_always),
            ("asset_cn_fallback", &self.asset_cn_fallback),
            ("art_fallback", &self.art_fallback),
            ("obtain_channel_align", &self.obtain_channel_align),
            ("profile_align", &self.profile_align),
            ("grade_invested_only", &self.grade_invested_only),
            (
                "grade_legacy_rarity_weights",
                &self.grade_legacy_rarity_weights,
            ),
            ("grade_module_per_slot", &self.grade_module_per_slot),
            ("story_hub", &self.story_hub),
            ("story_hub_fallback", &self.story_hub_fallback),
            ("story_whole_body", &self.story_whole_body),
            ("sandbox_topic_backfill", &self.sandbox_topic_backfill),
        ]
    }
}

pub const DEFAULT_PATH: &str = "config/myrtle.toml";

impl Config {
    /// Refuses a required secret that does not resolve to a non-empty value
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let path = path.unwrap_or_else(|| Path::new(DEFAULT_PATH));
        let file = |message: String| ConfigError::File {
            path: path.display().to_string(),
            message,
        };
        let text = std::fs::read_to_string(path).map_err(|e| file(e.to_string()))?;
        let config: Self = toml::from_str(&text).map_err(|e| file(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |key: &str, message: &str| ConfigError::Invalid {
            key: key.to_owned(),
            message: message.to_owned(),
        };
        if self.server.servers.is_empty() {
            return Err(invalid("server.servers", "at least one server is required"));
        }
        if self.storage.sync_concurrency == 0 {
            return Err(invalid("storage.sync_concurrency", "must be above 0"));
        }
        for (name, switch) in self.switches.all() {
            if parse_date(&switch.remove_after).is_none() {
                return Err(invalid(
                    &format!("switches.{name}.remove_after"),
                    "must be a YYYY-MM-DD date",
                ));
            }
        }

        let secrets = &self.secrets;
        self.database.url.resolve()?;
        secrets.game_credential_key.resolve()?;
        warn_if_weak("secrets.jwt_secret", &secrets.jwt_secret.resolve()?, 32);
        if let Some(key) = SecretRef::optional(secrets.service_key.as_ref()) {
            warn_if_weak("secrets.service_key", &key, 16);
        }
        self.object_storage()?;
        Ok(())
    }

    /// Switches whose `remove_after` is before `today` (`YYYY-MM-DD`)
    #[must_use]
    pub fn expired_switches(&self, today: &str) -> Vec<ExpiredSwitch> {
        self.switches
            .all()
            .into_iter()
            .filter(|(_, switch)| parse_date(&switch.remove_after) < parse_date(today))
            .map(|(name, switch)| ExpiredSwitch {
                name,
                owner: switch.owner.clone(),
                remove_after: switch.remove_after.clone(),
            })
            .collect()
    }

    /// The object store `[storage]` describes, `None` when its type is `off` or empty
    pub fn object_storage(&self) -> Result<Option<myrtle_storage::StorageConfig>, ConfigError> {
        let storage = &self.storage;
        let secret = |secret: &Option<SecretRef>| secret.as_ref().and_then(|s| s.resolve().ok());
        let lookup = |var: &str| -> Option<String> {
            let value = match var {
                "STORAGE_TYPE" => storage.kind.clone(),
                "CLOUDFLARE_BUCKET" => storage.bucket.clone(),
                "CLOUDFLARE_ACCOUNT_ID" => storage.account_id.clone(),
                "CLOUDFLARE_S3_API" => storage.endpoint.clone(),
                "CLOUDFLARE_API_ACCESS_KEY" => secret(&storage.access_key_id)?,
                "CLOUDFLARE_API_SECRET_KEY" => secret(&storage.secret_access_key)?,
                "CLOUDFLARE_PREFIX" => storage.prefix.clone(),
                "CLOUDFLARE_PUBLIC_BASE_URL" => storage.public_base_url.clone(),
                "STORAGE_LOCAL_DIR" => storage.local_dir.clone(),
                "STORAGE_ROOTS" => storage.roots.join(","),
                "STORAGE_CACHE_CONTROL" => storage.cache_control.clone(),
                "STORAGE_SYNC_CONCURRENCY" => storage.sync_concurrency.to_string(),
                "STORAGE_SYNC_HASH_THREADS" => positive(storage.sync_hash_threads)?,
                "STORAGE_MULTIPART_THRESHOLD_BYTES" => {
                    storage.multipart_threshold_bytes.to_string()
                }
                "STORAGE_EVICT_LOCAL" => storage.evict_local.to_string(),
                "STORAGE_EVICT_KEEP" => storage.evict_keep.join(","),
                _ => return None,
            };
            Some(value).filter(|v| !v.is_empty())
        };
        myrtle_storage::StorageConfig::from_lookup(&lookup).map_err(|e| {
            let unresolved = match &e {
                myrtle_storage::ConfigError::Missing { var, .. } => match *var {
                    "CLOUDFLARE_API_ACCESS_KEY" => {
                        Some(("storage.access_key_id", &storage.access_key_id))
                    }
                    "CLOUDFLARE_API_SECRET_KEY" => {
                        Some(("storage.secret_access_key", &storage.secret_access_key))
                    }
                    _ => None,
                },
                _ => None,
            };
            let resolve_error =
                unresolved.and_then(|(key, secret)| Some((key, secret.as_ref()?.resolve().err()?)));
            match resolve_error {
                Some((key, resolve)) => ConfigError::Invalid {
                    key: key.into(),
                    message: resolve.to_string(),
                },
                None => ConfigError::Invalid {
                    key: "storage".into(),
                    message: e.to_string(),
                },
            }
        })
    }

    /// How a request for a mirrored asset is answered (`assets.delivery`)
    pub fn delivery_mode(
        &self,
        storage: Option<&myrtle_storage::StorageConfig>,
    ) -> Result<myrtle_storage::DeliveryMode, ConfigError> {
        let lookup = |var: &str| {
            (var == "ASSET_DELIVERY" && !self.assets.delivery.is_empty())
                .then(|| self.assets.delivery.clone())
        };
        myrtle_storage::DeliveryMode::from_lookup(&lookup, storage).map_err(|e| {
            ConfigError::Invalid {
                key: "assets.delivery".into(),
                message: e.to_string(),
            }
        })
    }
}

/// `None` for zero, which the storage crate reads as "use the default"
fn positive(n: usize) -> Option<String> {
    (n > 0).then(|| n.to_string())
}

fn parse_date(date: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

fn warn_if_weak(key: &str, value: &str, recommended_len: usize) {
    if value.trim().len() != value.len() {
        tracing::warn!(
            key,
            "secret has leading or trailing whitespace; it is used verbatim"
        );
    }
    if value.len() < recommended_len {
        tracing::warn!(
            key,
            length = value.len(),
            recommended = recommended_len,
            "secret is shorter than recommended"
        );
    }
}

/// How a field renders in the example
#[derive(Debug, Clone, Copy)]
pub enum Kind {
    /// `default` is a TOML literal
    Value,
    /// `default` is the `"env:NAME"` literal
    Secret,
    /// A `[switches.<name>]` table; `default` is `true` or `false`
    Switch {
        owner: &'static str,
        remove_after: &'static str,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Field {
    /// Dotted path: `section.key`, `section.sub.key` or `switches.<name>`
    pub key: &'static str,
    pub default: &'static str,
    pub kind: Kind,
    /// The env vars this key replaces; empty for a new key
    pub env: &'static [&'static str],
    pub source: &'static str,
    pub doc: &'static str,
}

/// An env var with no key, and why
#[derive(Debug, Clone, Copy)]
pub struct Retired {
    pub env: &'static str,
    pub source: &'static str,
    pub reason: &'static str,
}

const OWNER: &str = "eltik";
const REMOVE_AFTER: &str = "2027-01-31"; // placeholder until each switch gets a real deadline

const fn value(
    key: &'static str,
    default: &'static str,
    env: &'static [&'static str],
    source: &'static str,
    doc: &'static str,
) -> Field {
    Field {
        key,
        default,
        kind: Kind::Value,
        env,
        source,
        doc,
    }
}

const fn secret(
    key: &'static str,
    default: &'static str,
    env: &'static [&'static str],
    source: &'static str,
    doc: &'static str,
) -> Field {
    Field {
        key,
        default,
        kind: Kind::Secret,
        env,
        source,
        doc,
    }
}

const fn switch(
    key: &'static str,
    default: &'static str,
    env: &'static [&'static str],
    source: &'static str,
    doc: &'static str,
) -> Field {
    Field {
        key,
        default,
        kind: Kind::Switch {
            owner: OWNER,
            remove_after: REMOVE_AFTER,
        },
        env,
        source,
        doc,
    }
}

/// Every key, in file order
pub const FIELDS: &[Field] = &[
    value(
        "server.bind_host",
        "\"127.0.0.1\"",
        &["HOST"],
        "backend/src/app/server.rs:93",
        "Interface the API binds. Today the default is 0.0.0.0 (backend/src/app/server.rs:132); nginx is in front, so loopback.",
    ),
    value(
        "server.port",
        "3060",
        &["PORT"],
        "backend/src/app/server.rs:89",
        "API port.",
    ),
    value(
        "server.metrics_bind",
        "\"127.0.0.1:9464\"",
        &[],
        "new",
        "Separate listener for /metrics. Today /metrics is on the public router (backend/src/app/server.rs:71).",
    ),
    value(
        "server.trusted_proxy_hops",
        "0",
        &["TRUSTED_PROXY_HOPS"],
        "backend/src/app/middleware.rs:145",
        "X-Forwarded-For entries appended by trusted proxies, counted from the right. Production behind nginx needs 1; check before changing.",
    ),
    value(
        "server.servers",
        "[\"en\"]",
        &["SERVERS"],
        "backend/src/app/state.rs:244",
        "Servers served; the first is the default.",
    ),
    secret(
        "database.url",
        "\"env:DATABASE_URL\"",
        &["DATABASE_URL"],
        "backend/src/main.rs:166",
        "Postgres URL. Also read by 10 of the 13 bins today.",
    ),
    value(
        "database.max_connections",
        "40",
        &["DATABASE_MAX_CONNECTIONS"],
        "backend/src/database/pool.rs:23",
        "Pool size.",
    ),
    value(
        "database.acquire_timeout_ms",
        "5000",
        &[],
        "backend/src/database/pool.rs:18",
        "Pool acquire timeout; a timeout answers 503 SERVICE_UNAVAILABLE. A const today.",
    ),
    secret(
        "redis.url",
        "\"env:REDIS_URL\"",
        &["REDIS_URL"],
        "backend/src/main.rs:176",
        "Redis for the shared cache. Remove the key to use the in-memory cache.",
    ),
    secret(
        "secrets.jwt_secret",
        "\"env:JWT_SECRET\"",
        &["JWT_SECRET"],
        "backend/src/app/state.rs:258",
        "Session signing key, at least 32 bytes.",
    ),
    secret(
        "secrets.game_credential_key",
        "\"env:GAME_CREDENTIAL_KEY\"",
        &["GAME_CREDENTIAL_KEY"],
        "backend/src/app/state.rs:264",
        "Seals user_game_credentials; 32 bytes as hex or base64.",
    ),
    secret(
        "secrets.service_key",
        "\"env:SERVICE_KEY\"",
        &["SERVICE_KEY"],
        "backend/src/app/state.rs:271",
        "Legacy SuperAdmin key for frontend SSR. Replaced by a scoped api_keys row; removed in phase 7 (Retire).",
    ),
    secret(
        "secrets.github_token",
        "\"env:GITHUB_TOKEN\"",
        &["GITHUB_TOKEN"],
        "backend/src/core/dps_watcher.rs:93",
        "Raises the GitHub API limit for the DPS upstream watch.",
    ),
    secret(
        "secrets.discord_alert_webhook",
        "\"env:DISCORD_ALERT_WEBHOOK\"",
        &["DISCORD_ALERT_WEBHOOK"],
        "backend/src/core/alerts.rs:16",
        "Where service-account refusals are posted.",
    ),
    value(
        "rate_limit.rpm",
        "100",
        &["RATE_LIMIT_RPM"],
        "backend/src/app/state.rs:267",
        "Requests per minute per client.",
    ),
    value(
        "rate_limit.exempt_ips",
        "[]",
        &["RATE_LIMIT_EXEMPT_IPS"],
        "backend/src/app/middleware.rs:256",
        "CIDRs with no allowance at all. Loopback is NOT implicitly exempt in the rewrite.",
    ),
    value(
        "cpu.task_permits",
        "0",
        &["CPU_TASK_PERMITS"],
        "backend/src/app/cpu.rs:35",
        "Concurrent CPU-bound requests; 0 = half the cores. Production sets 2.",
    ),
    value(
        "cpu.light_permits",
        "0",
        &["CPU_LIGHT_PERMITS"],
        "backend/src/app/cpu.rs:53",
        "Light CPU pool; 0 = same as task_permits.",
    ),
    value(
        "cpu.task_wait_ms",
        "8000",
        &["CPU_TASK_WAIT_MS"],
        "backend/src/app/cpu.rs:73",
        "How long a request waits for a permit; 0 refuses at once (the old behaviour).",
    ),
    value(
        "cpu.task_queue",
        "0",
        &["CPU_TASK_QUEUE"],
        "backend/src/app/cpu.rs:87",
        "Waiters allowed; 0 = task_permits * 8.",
    ),
    value(
        "assets.dir",
        "\"../assets/output\"",
        &["ASSETS_DIR"],
        "backend/src/app/state.rs:272",
        "Extract root; per-server dirs are {dir}/{server}.",
    ),
    value(
        "assets.allow_all_dirs",
        "false",
        &["ASSETS_ALLOW_ALL_DIRS"],
        "backend/src/app/routes/assets.rs:36",
        "Serve any directory under the root, not only the public list.",
    ),
    value(
        "assets.stream_chunk_bytes",
        "65536",
        &["ASSET_STREAM_CHUNK_BYTES"],
        "backend/src/app/routes/assets.rs:73",
        "Chunk size when streaming a local file.",
    ),
    value(
        "assets.ws_urls",
        "{}",
        &["ASSET_WS_URLS", "ASSET_WS_URL"],
        "backend/src/app/state.rs:346",
        "Per-server asset-pipeline WebSocket, e.g. { en = \"ws://127.0.0.1:9160\" }. ASSET_WS_URL (:349) was the one-server form.",
    ),
    value(
        "assets.delivery",
        "\"\"",
        &["ASSET_DELIVERY"],
        "backend/crates/storage/README.md:59",
        "local | redirect | proxy; empty = derived from [storage] as today.",
    ),
    value(
        "storage.type",
        "\"\"",
        &["S3_TYPE"],
        "backend/crates/storage/README.md:42",
        "cloudflare (alias r2) | local (alias fs); empty or off = storage disabled. R2 only: no aws or generic s3.",
    ),
    value(
        "storage.bucket",
        "\"\"",
        &["S3_BUCKET"],
        "backend/crates/storage/README.md:43",
        "Bucket name.",
    ),
    secret(
        "storage.access_key_id",
        "\"env:S3_ACCESS_KEY_ID\"",
        &["S3_ACCESS_KEY_ID", "CLOUDFLARE_API_ACCESS_KEY"],
        "backend/crates/storage/README.md:44",
        "Static credential.",
    ),
    secret(
        "storage.secret_access_key",
        "\"env:S3_SECRET_ACCESS_KEY\"",
        &["S3_SECRET_ACCESS_KEY", "CLOUDFLARE_API_SECRET_KEY"],
        "backend/crates/storage/README.md:44",
        "Static credential.",
    ),
    value(
        "storage.account_id",
        "\"\"",
        &["S3_ACCOUNT_ID", "CLOUDFLARE_ACCOUNT_ID"],
        "backend/crates/storage/README.md:45",
        "Cloudflare account; derives the R2 endpoint.",
    ),
    value(
        "storage.endpoint",
        "\"\"",
        &["S3_ENDPOINT", "CLOUDFLARE_S3_API"],
        "backend/crates/storage/README.md:46",
        "R2 S3 API endpoint; empty = https://{account_id}.r2.cloudflarestorage.com.",
    ),
    value(
        "storage.prefix",
        "\"\"",
        &["S3_PREFIX"],
        "backend/crates/storage/README.md:49",
        "Key prefix, e.g. \"myrtle\".",
    ),
    value(
        "storage.public_base_url",
        "\"\"",
        &["S3_PUBLIC_BASE_URL", "S3_PUBLIC_URL"],
        "backend/crates/storage/README.md:50",
        "Public base for GETs. S3_PUBLIC_URL was an alias.",
    ),
    value(
        "storage.local_dir",
        "\"\"",
        &["S3_LOCAL_DIR"],
        "backend/crates/storage/README.md:51",
        "Directory standing in for the bucket (type = local).",
    ),
    value(
        "storage.roots",
        "[\"textures\", \"spine\", \"audio\", \"video\", \"portraits\"]",
        &["S3_ROOTS"],
        "backend/crates/storage/README.md:52",
        "Top-level dirs under each server that live in the store.",
    ),
    value(
        "storage.cache_control",
        "\"public, max-age=604800\"",
        &["S3_CACHE_CONTROL"],
        "backend/crates/storage/README.md:53",
        "Cache-Control on uploads.",
    ),
    value(
        "storage.sync_concurrency",
        "16",
        &["S3_SYNC_CONCURRENCY"],
        "backend/crates/storage/README.md:54",
        "Concurrent uploads and deletes.",
    ),
    value(
        "storage.sync_hash_threads",
        "0",
        &["S3_SYNC_HASH_THREADS"],
        "backend/crates/storage/README.md:55",
        "Hashing tasks; 0 = available CPUs.",
    ),
    value(
        "storage.multipart_threshold_bytes",
        "268435456",
        &["S3_MULTIPART_THRESHOLD_BYTES"],
        "backend/crates/storage/README.md:56",
        "Multipart above this size.",
    ),
    value(
        "storage.evict_local",
        "false",
        &["S3_EVICT_LOCAL"],
        "backend/crates/storage/README.md:57",
        "Truncate local copies the bucket holds.",
    ),
    value(
        "storage.evict_keep",
        "[]",
        &["S3_EVICT_KEEP"],
        "backend/crates/storage/README.md:58",
        "Extra path prefixes never evicted.",
    ),
    value(
        "hypergryph.network_version_override",
        "\"\"",
        &["AK_NETWORK_VERSION"],
        "backend/src/core/hypergryph/session.rs:45",
        "Probe another networkVersion without a rebuild; empty = per-server default.",
    ),
    value(
        "hypergryph.cn_app_code",
        "\"4ca99fa6b56cc2ba\"",
        &["HYPERGRYPH_CN_APP_CODE"],
        "backend/src/core/hypergryph/passport.rs:36",
        "CN passport app code (the public client value).",
    ),
    value(
        "hypergryph.device_id",
        "\"\"",
        &["DEVICE_ID"],
        "backend/src/core/hypergryph/loaders/device_ids.rs:17",
        "Pinned device id; empty = generated once by `cli hypergryph device-ids` and written here.",
    ),
    value(
        "hypergryph.device_id2",
        "\"\"",
        &["DEVICE_ID2"],
        "backend/src/core/hypergryph/loaders/device_ids.rs:18",
        "Second device id.",
    ),
    value(
        "hypergryph.device_id3",
        "\"\"",
        &["DEVICE_ID3"],
        "backend/src/core/hypergryph/loaders/device_ids.rs:19",
        "Third device id.",
    ),
    value(
        "hypergryph.service_session_max_age_secs",
        "1800",
        &["GAME_SESSION_MAX_AGE_SECS"],
        "backend/src/core/service_account.rs:53",
        "Re-mint a service-account secret older than this.",
    ),
    value(
        "jobs.stagger_secs",
        "180",
        &["JOB_STAGGER_SECS"],
        "backend/src/core/jobs.rs:12",
        "Window schedule offsets are spread across; 0 = none.",
    ),
    value(
        "jobs.trending.every_secs",
        "900",
        &["TRENDING_RECOMPUTE_SECS"],
        "backend/src/core/trending_job.rs:37",
        "Trending recompute period.",
    ),
    value(
        "jobs.trending.top_n",
        "10",
        &["TRENDING_TOP_N"],
        "backend/src/core/trending_job.rs:8",
        "Entries ranked.",
    ),
    value(
        "jobs.regrade.every_secs",
        "86400",
        &["REGRADE_INTERVAL_SECS"],
        "backend/src/core/regrade_job.rs:87",
        "Full regrade period.",
    ),
    value(
        "jobs.regrade.concurrency",
        "2",
        &["REGRADE_CONCURRENCY"],
        "backend/src/core/regrade_job.rs:91",
        "RegradeUser jobs one worker runs at once.",
    ),
    value(
        "jobs.regrade.page_size",
        "500",
        &["REGRADE_PAGE_SIZE"],
        "backend/src/core/regrade_job.rs:96",
        "Users enqueued per RegradeAll page.",
    ),
    value(
        "jobs.gacha_detail.every_secs",
        "21600",
        &["GACHA_DETAIL_REFRESH_SECS"],
        "backend/src/core/gacha_detail_job.rs:74",
        "Pool detail refresh period.",
    ),
    value(
        "jobs.gacha_detail.call_delay_ms",
        "120",
        &["GACHA_DETAIL_CALL_DELAY_MS"],
        "backend/src/core/gacha_detail_job.rs:47",
        "Delay between upstream calls.",
    ),
    value(
        "jobs.gacha_detail.max_failures",
        "5",
        &["GACHA_DETAIL_MAX_FAILURES"],
        "backend/src/core/gacha_detail_job.rs:51",
        "Failures before a pass gives up.",
    ),
    value(
        "jobs.gacha_detail.checkpoint_every",
        "50",
        &["GACHA_DETAIL_CHECKPOINT_EVERY"],
        "backend/src/core/gacha_detail_job.rs:59",
        "Pools between checkpoints.",
    ),
    value(
        "jobs.gacha_detail.dir",
        "\"\"",
        &["GACHA_DETAIL_DIR"],
        "backend/src/core/gamedata/types/gacha_detail.rs:58",
        "Output dir override; empty = {assets.dir}/{server}/derived.",
    ),
    value(
        "jobs.event_shop.every_secs",
        "21600",
        &["EVENT_SHOP_REFRESH_SECS"],
        "backend/src/core/event_shop_job.rs:54",
        "Event shop refresh period.",
    ),
    value(
        "jobs.event_shop.call_delay_ms",
        "120",
        &["EVENT_SHOP_CALL_DELAY_MS"],
        "backend/src/core/event_shop_job.rs:39",
        "Delay between upstream calls.",
    ),
    value(
        "jobs.event_shop.max_failures",
        "5",
        &["EVENT_SHOP_MAX_FAILURES"],
        "backend/src/core/event_shop_job.rs:40",
        "Failures before a pass gives up.",
    ),
    value(
        "jobs.event_shop.dir",
        "\"\"",
        &["EVENT_SHOP_DIR"],
        "backend/src/core/gamedata/types/event_shop.rs:26",
        "Output dir override; empty = {assets.dir}/{server}/derived.",
    ),
    value(
        "jobs.maintenance.poll_secs",
        "300",
        &["MAINTENANCE_POLL_SECS"],
        "backend/src/core/maintenance/mod.rs:297",
        "Announcement poll period (60 s inside a window, a const).",
    ),
    value(
        "jobs.siren.every_secs",
        "21600",
        &["SIREN_REFRESH_SECS"],
        "backend/src/app/services/music/siren/mod.rs:483",
        "Monster Siren catalogue sync period; 0 = sync only when empty.",
    ),
    value(
        "jobs.siren.api_base",
        "\"https://monster-siren.hypergryph.com/api\"",
        &["SIREN_API_BASE"],
        "backend/src/app/services/music/siren/client.rs:112",
        "Siren API base.",
    ),
    value(
        "jobs.dps_watch.every_secs",
        "0",
        &["DPS_POLL_INTERVAL"],
        "backend/src/core/dps_watcher.rs:23",
        "Poll ArknightsDpsCompare for new commits; 0 = off (unset is off today).",
    ),
    value(
        "jobs.dps_watch.upstream_repo",
        "\"WhoAteMyCQQkie/ArknightsDpsCompare\"",
        &["DPS_UPSTREAM_REPO"],
        "backend/src/core/dps_watcher.rs:29",
        "Upstream repo.",
    ),
    value(
        "jobs.dps_watch.upstream_branch",
        "\"main\"",
        &["DPS_UPSTREAM_BRANCH"],
        "backend/src/core/dps_watcher.rs:31",
        "Upstream branch.",
    ),
    value(
        "jobs.sandbox_backfill.switch_wait_secs",
        "15",
        &["SANDBOX_TOPIC_BACKFILL_WAIT_S"],
        "backend/src/app/services/sandbox_backfill.rs:69",
        "Wait before switching an account back to its loaded RA topic; the client's CD_REQUEST_SWITCH_TOPIC reads 10, unit unknown.",
    ),
    value(
        "dps.rank_off_grid_ttl_secs",
        "3600",
        &["DPS_RANK_OFF_GRID_TTL_SECS"],
        "backend/src/app/cache/keys.rs:16",
        "Lifetime of a DPS rank entry off the page grid; 86400 matches the grid's. Above 0.",
    ),
    value(
        "release.art_mirrors",
        "[]",
        &["RELEASE_ART_MIRRORS"],
        "backend/src/core/release/art.rs:32",
        "server=url pairs; empty = the built-in DEFAULT_MIRRORS (art.rs:7).",
    ),
    value(
        "release.lag_window",
        "10",
        &["RELEASE_LAG_WINDOW"],
        "backend/src/core/release/estimate.rs:13",
        "Events in the CN->EN lag median.",
    ),
    value(
        "release.lag_half_life",
        "3.0",
        &["RELEASE_LAG_HALF_LIFE"],
        "backend/src/core/release/estimate.rs:35",
        "Half-life (events) of the lag weighting.",
    ),
    value(
        "story.art_min_bytes_per_px",
        "0.2",
        &["STORY_ART_MIN_BPP"],
        "backend/src/app/services/story/story_art.rs:87",
        "Story art below this density is treated as a placeholder.",
    ),
    value(
        "cli.default_server",
        "\"\"",
        &["BIN_SERVER"],
        "backend/src/app/state.rs:573",
        "Server the CLI acts on when none is given; empty = first of server.servers.",
    ),
    value(
        "cli.admin_uid",
        "\"\"",
        &["ADMIN_UID"],
        "backend/src/bin/manage_permissions.rs:41",
        "Acting admin for `cli grants`.",
    ),
    value(
        "tools.python_bin",
        "\"python3\"",
        &["PYTHON_BIN"],
        "backend/src/bin/generate_dps.rs:193",
        "Python used by tools/dps-transpile.",
    ),
    value(
        "debug.base_pin_trace",
        "false",
        &["BASE_PIN_TRACE"],
        "backend/src/core/grade/base/shift_rotation.rs:785",
        "Log why each pinned base operator stayed.",
    ),
    value(
        "debug.base_rot_timing",
        "false",
        &["BASE_ROT_TIMING"],
        "backend/src/core/grade/base/shift_rotation.rs:654",
        "Time the rotation search (as a tracing span in the caller).",
    ),
    value(
        "debug.sync_dump_dir",
        "\"\"",
        &["MYRTLE_SYNC_DUMP_DIR"],
        "backend/src/app/services/roster.rs:870",
        "Write raw syncData responses here; empty = off.",
    ),
    value(
        "debug.story_thumb_out",
        "\"\"",
        &["STORY_THUMB_OUT"],
        "backend/src/app/services/story_sprite_thumb.rs:389",
        "Write story sprite thumbnails here for inspection; empty = off.",
    ),
    switch(
        "switches.background_jobs",
        "true",
        &["DISABLE_BACKGROUND_JOBS"],
        "backend/src/main.rs:253",
        "Scheduler enqueues recurring jobs. Today DISABLE_BACKGROUND_JOBS=1 turns off every loop at once.",
    ),
    switch(
        "switches.role_recheck",
        "true",
        &["DISABLE_ROLE_RECHECK"],
        "backend/src/app/state.rs:277",
        "Gates re-read the stored role instead of trusting the JWT claim.",
    ),
    switch(
        "switches.release_ledger",
        "true",
        &["RELEASE_LEDGER"],
        "backend/src/core/release/ledger.rs:21",
        "Record gamedata sightings for the release planner.",
    ),
    switch(
        "switches.release_trust_skipped_tags",
        "true",
        &["RELEASE_TRUST_SKIPPED_TAGS"],
        "backend/src/core/release/skipped.rs:39",
        "Treat skipped-tag CN listings as Unlisted.",
    ),
    switch(
        "switches.release_pooled_cadence",
        "false",
        &["RELEASE_POOLED_CADENCE"],
        "backend/src/app/services/release/skins.rs:226",
        "Restore the pre-fix skin cadence rule.",
    ),
    switch(
        "switches.release_keep_unsold_skins",
        "false",
        &["RELEASE_KEEP_UNSOLD_SKINS"],
        "backend/src/core/release/skins.rs:30",
        "Keep skins never sold in a shop.",
    ),
    switch(
        "switches.release_reviews_all_groups",
        "false",
        &["RELEASE_REVIEWS_ALL_GROUPS"],
        "backend/src/core/release/skins.rs:36",
        "Attach every outfit-review window to every earlier group.",
    ),
    switch(
        "switches.sidecar_reload_always",
        "false",
        &["SIDECAR_RELOAD_ALWAYS"],
        "backend/src/core/sidecar.rs:76",
        "Reload after every job run that fetched anything, not only on change.",
    ),
    switch(
        "switches.asset_cn_fallback",
        "true",
        &["ASSET_CN_FALLBACK"],
        "backend/src/app/state.rs:155",
        "Serve CN art when a server lacks its own.",
    ),
    switch(
        "switches.art_fallback",
        "true",
        &["ART_FALLBACK"],
        "backend/src/app/state.rs:518",
        "Fall back to the default server's art tree.",
    ),
    switch(
        "switches.obtain_channel_align",
        "true",
        &["OBTAIN_CHANNEL_ALIGN"],
        "backend/src/core/gamedata/enrich/reference.rs:42",
        "Align obtain channels across servers at compile.",
    ),
    switch(
        "switches.profile_align",
        "true",
        &["PROFILE_ALIGN"],
        "backend/src/core/gamedata/enrich/profile.rs:51",
        "Align profile text across servers at compile.",
    ),
    switch(
        "switches.grade_invested_only",
        "false",
        &["GRADE_INVESTED_ONLY"],
        "backend/src/core/grade/grade_operators.rs:142",
        "Grade only invested operators.",
    ),
    switch(
        "switches.grade_legacy_rarity_weights",
        "false",
        &["GRADE_RARITY_WEIGHTS"],
        "backend/src/core/grade/grade_operators.rs:143",
        "Old rarity weights (GRADE_RARITY_WEIGHTS=legacy).",
    ),
    switch(
        "switches.grade_module_per_slot",
        "true",
        &["GRADE_MODULE_PER_SLOT"],
        "backend/src/core/grade/grade_operators.rs:144",
        "Score modules per slot.",
    ),
    switch(
        "switches.story_hub",
        "true",
        &["STORY_NO_HUB"],
        "backend/src/core/story/assets.rs:525",
        "Use hub sprites in story art (STORY_NO_HUB turned it off).",
    ),
    switch(
        "switches.story_hub_fallback",
        "true",
        &["STORY_NO_HUB_FALLBACK"],
        "backend/src/core/story/assets.rs:1304",
        "Fall back to hub sprites.",
    ),
    switch(
        "switches.story_whole_body",
        "true",
        &["STORY_NO_WHOLE_BODY"],
        "backend/src/core/story/assets.rs:1181",
        "Use whole-body sprites.",
    ),
    switch(
        "switches.sandbox_topic_backfill",
        "true",
        &["SANDBOX_TOPIC_BACKFILL"],
        "backend/src/app/services/sandbox_backfill.rs:59",
        "Fetch RA topics a sync does not carry by switching the account to each.",
    ),
];

pub const RETIRED: &[Retired] = &[
    Retired {
        env: "DEVICE_IDS_FILE",
        source: "backend/src/core/hypergryph/loaders/device_ids.rs:9",
        reason: "device ids are config values ([hypergryph] device_id*), not a file on the VPS disk",
    },
    Retired {
        env: "GAME_SESSION_DIR",
        source: "backend/src/core/service_account.rs:63",
        reason: "service-account sessions move to Postgres behind hypergryph::SessionStore",
    },
    Retired {
        env: "REGRADE_STATE_FILE",
        source: "backend/src/core/regrade_job.rs:35",
        reason: "last-run state is job_schedules.last_enqueued_at",
    },
    Retired {
        env: "DPS_STATE_FILE",
        source: "backend/src/core/dps_watcher.rs:32",
        reason: "last-seen commit is stored on the DpsUpstreamWatch schedule row",
    },
    Retired {
        env: "STARTUP_TIMINGS_FILE",
        source: "backend/src/core/startup.rs:148",
        reason: "boot no longer compiles game data, so the boot phase plan goes",
    },
    Retired {
        env: "STEP_RSS",
        source: "backend/src/core/startup.rs:362",
        reason: "boot step RSS tracing goes with the boot phase plan",
    },
    Retired {
        env: "S3_REGION",
        source: "backend/crates/storage/README.md:47",
        reason: "the store is R2 only, whose region is always auto",
    },
    Retired {
        env: "S3_FORCE_PATH_STYLE",
        source: "backend/crates/storage/README.md:48",
        reason: "the store is R2 only, always addressed path-style",
    },
    Retired {
        env: "NO_PROGRESS",
        source: "backend/src/core/startup.rs:535",
        reason: "boot progress bars go with the boot phase plan; the CLI uses --quiet",
    },
];

/// `config/myrtle.example.toml`
pub fn render_example() -> String {
    let mut out = String::new();
    out.push_str(
        "# GENERATED from crates/app/src/config.rs (FIELDS, RETIRED). Do not edit by hand:\n\
         # change FIELDS, then run `MYRTLE_BLESS=1 cargo test -p myrtle-app example_is_current`.\n\
         #\n\
         # Copy to config/myrtle.toml. Secrets are references: \"env:NAME\" or \"file:/path\".\n\
         # Each key notes the env var it replaces and where today's code reads it.\n",
    );
    let mut section = String::new();
    for field in FIELDS {
        match field.kind {
            Kind::Switch {
                owner,
                remove_after,
            } => {
                out.push('\n');
                push_doc(&mut out, field);
                let _ = writeln!(out, "[{}]", field.key);
                let _ = writeln!(out, "enabled = {}", field.default);
                let _ = writeln!(out, "owner = \"{owner}\"");
                let _ = writeln!(
                    out,
                    "remove_after = \"{remove_after}\" # PLACEHOLDER date, set in phase 1"
                );
                section.clear();
            }
            Kind::Value | Kind::Secret => {
                let (table, key) = field.key.rsplit_once('.').unwrap_or(("", field.key));
                if table != section {
                    let _ = write!(out, "\n[{table}]\n");
                    section = table.to_owned();
                }
                push_doc(&mut out, field);
                let _ = writeln!(out, "{key} = {}", field.default);
            }
        }
    }
    out.push_str("\n# Retired: read today, no key here.\n");
    for r in RETIRED {
        let _ = writeln!(out, "#   {} ({}): {}", r.env, r.source, r.reason);
    }
    out
}

fn push_doc(out: &mut String, field: &Field) {
    let _ = writeln!(out, "# {}", field.doc);
    if field.env.is_empty() {
        out.push_str("# was: (new)\n");
    } else {
        let _ = writeln!(out, "# was: {} ({})", field.env.join(", "), field.source);
    }
}
