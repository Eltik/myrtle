use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use arc_swap::ArcSwap;
use reqwest::Client;
use sqlx::PgPool;

use crate::app::cache::Inflight;
use crate::app::cache::store::CacheStore;
use crate::core::auth::credentials::CredentialKey;
use crate::core::gamedata::{assets::AssetIndex, types::GameData};
use crate::core::hypergryph::constants::Server;
use crate::core::service_account::ServiceAccounts;
use tracing::warn;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Hot-reloadable game data and asset index for a single server.
///
/// Stored behind `Arc` in [`AppStateInner::servers`] so multiple server keys can
/// point at one cell (for example `Bilibili` shares `CN`'s Hypergryph data). Each
/// field hot-reloads in place via [`ArcSwap`], so a swap made through any aliasing
/// key is visible through all of them.
pub struct ServerData {
    pub game_data: ArcSwap<GameData>,
    pub asset_index: ArcSwap<AssetIndex>,
    pub game_data_dir: String,
    pub assets_dir: String,
    /// The tree the art (asset index, stage art, chibis) is read from. Equal to
    /// `assets_dir` unless this server ships no art; see [`art_dir_for`].
    pub art_dir: String,
    /// False while this entry is a placeholder: the server was configured but its game data
    /// failed to load at boot, so `game_data`/`asset_index` hold the DEFAULT server's Arcs
    /// until a hot reload (`asset_watcher::perform_reload`) succeeds and flips this. Explicit
    /// per-server lookups treat an unloaded entry as absent; internal fallbacks keep working.
    pub loaded: AtomicBool,
    /// Lazy-load bookkeeping (see `app::residency`). Inert for an eager server.
    pub residency: crate::app::residency::Residency,
}

impl ServerData {
    /// An entry with nothing loaded yet: a server that failed at boot, or a
    /// lazy server before its first request.
    ///
    /// It holds an EMPTY `GameData`, not the default server's, and
    /// [`AppState::server_data`] falls back past it to a loaded server. Holding
    /// the default's `Arc` (the old placeholder) pinned the boot-time default
    /// `GameData` for the life of the process once the default hot-reloaded: the
    /// 6-hourly pool-detail reload alone made that a second resident copy of
    /// EN. `PLACEHOLDER_EMPTY=0` restores the old placeholder (the default's
    /// `Arc`s) and the old `server_data` lookup exactly; `fallback` supplies those
    /// `Arc`s and is ignored otherwise.
    pub fn placeholder(
        game_data_dir: String,
        assets_dir: String,
        art_dir: String,
        lazy: bool,
        fallback: Option<(Arc<GameData>, Arc<AssetIndex>)>,
    ) -> Self {
        let (game_data, asset_index) = match fallback {
            Some((gd, ai)) if !placeholder_empty() => (gd, ai),
            _ => (
                Arc::new(GameData::default()),
                Arc::new(AssetIndex::default()),
            ),
        };
        Self {
            game_data: ArcSwap::new(game_data),
            asset_index: ArcSwap::new(asset_index),
            game_data_dir,
            assets_dir,
            art_dir,
            loaded: AtomicBool::new(false),
            residency: crate::app::residency::Residency::new(lazy),
        }
    }

    /// Drop this server's data, leaving an unloaded entry. The memory goes when
    /// the last request holding the old `Arc` finishes.
    pub(crate) fn unload(&self) {
        self.loaded.store(false, Ordering::Release);
        self.game_data.store(Arc::new(GameData::default()));
        self.asset_index.store(Arc::new(AssetIndex::default()));
    }
}

/// `PLACEHOLDER_EMPTY=0` is the kill switch for the empty placeholder and the
/// lookup that falls back past it; see [`ServerData::placeholder`].
fn placeholder_empty() -> bool {
    !crate::utils::env::switched_off("PLACEHOLDER_EMPTY")
}

pub struct AppStateInner {
    pub db: PgPool,
    pub cache: CacheStore,
    pub servers: HashMap<Server, Arc<ServerData>>,
    pub default_server: Server,
    pub config: Arc<AppConfig>,
    pub http_client: Client,
    pub service_accounts: ServiceAccounts,
    /// Detached cache builds in flight (see `cache::cached_json_detached`).
    pub inflight: Inflight,
    /// Set when a roster sync changes rows the operator ownership aggregate is
    /// computed from; cleared by `core::operator_ownership_job` once it has
    /// recomputed. See [`AppState::mark_ownership_dirty`].
    ownership_dirty: AtomicBool,
}

impl AppState {
    pub fn new(
        db: PgPool,
        cache: CacheStore,
        servers: HashMap<Server, Arc<ServerData>>,
        default_server: Server,
        config: AppConfig,
        client: Client,
        service_accounts: ServiceAccounts,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                cache,
                servers,
                default_server,
                config: Arc::new(config),
                http_client: client,
                service_accounts,
                inflight: Inflight::default(),
                ownership_dirty: AtomicBool::new(false),
            }),
        }
    }

    /// Record that a roster sync changed rows the operator ownership aggregate
    /// reads, so the job recomputes on its next tick.
    ///
    /// This is the whole cost the refresh request pays: recomputing inline is
    /// 62.531 ms of work whose size grows with the roster, and because refresh
    /// frequency also grows with the user count the total goes as users
    /// squared. Coalescing breaks that term, since one recompute answers every
    /// sync that arrived inside the window.
    pub fn mark_ownership_dirty(&self) {
        self.ownership_dirty.store(true, Ordering::Release);
    }

    /// Claim a pending recompute, clearing the flag. Returns false when nothing
    /// has changed since the last pass.
    ///
    /// The flag is cleared BEFORE the recompute rather than after, so a sync
    /// landing mid-recompute re-arms it and gets its own pass. Clearing
    /// afterwards would swallow that sync until the next one arrived.
    pub fn take_ownership_dirty(&self) -> bool {
        self.ownership_dirty.swap(false, Ordering::AcqRel)
    }

    /// Resolve a server's data, falling back to the default server when the
    /// requested one is not loaded. Use this for internal access where a server
    /// is always available (the default and configured servers).
    ///
    /// An entry that is configured but not loaded (failed at boot, or a lazy
    /// server not resident) is skipped too: the default when it is loaded, else
    /// the first loaded server in config order, else the unloaded entry itself
    /// (empty data). `PLACEHOLDER_EMPTY=0` restores the old lookup, which
    /// returned the unloaded entry and so served whatever its placeholder held.
    ///
    /// Code that writes to a server's entry (a reload, a swap, a sidecar path)
    /// must use [`AppState::entry`], never this.
    pub fn server_data(&self, server: Server) -> Arc<ServerData> {
        let raw = self
            .servers
            .get(&server)
            .or_else(|| self.servers.get(&self.default_server))
            .expect("default server data must be present");
        if !placeholder_empty() || raw.loaded.load(Ordering::Acquire) {
            return raw.clone();
        }
        std::iter::once(self.default_server)
            .chain(self.config.servers.iter().copied())
            .filter_map(|s| self.servers.get(&s))
            .find(|sd| sd.loaded.load(Ordering::Acquire))
            .unwrap_or(raw)
            .clone()
    }

    /// The server's own entry, loaded or not, with no fallback. `None` when the
    /// server is not configured.
    pub fn entry(&self, server: Server) -> Option<Arc<ServerData>> {
        self.servers.get(&server).cloned()
    }

    /// The loaded data for an explicit per-server request, or the error that
    /// request should answer with.
    ///
    /// A server that is not configured is a 404. One that IS configured but has
    /// no data loaded (its load failed, or a lazy server could not be brought
    /// in) is a 503 that names the server: the resource exists, the process
    /// cannot serve it right now, and a retry may succeed.
    /// `UNLOADED_SERVER_503=0` restores the old plain 404 for both.
    pub fn require_server_data(
        &self,
        server: Server,
    ) -> Result<Arc<ServerData>, crate::app::error::ApiError> {
        if let Some(sd) = self.try_server_data(server) {
            return Ok(sd);
        }
        if self.servers.contains_key(&server)
            && !crate::utils::env::switched_off("UNLOADED_SERVER_503")
        {
            return Err(crate::app::error::ApiError::ServiceUnavailableMessage(
                format!(
                    "game data for server '{}' is not loaded right now; retry shortly",
                    server.as_str()
                ),
            ));
        }
        Err(crate::app::error::ApiError::NotFound)
    }

    /// Resolve a server's data only when it is actually loaded, without the
    /// default fallback. Returns `None` for valid but unconfigured servers so
    /// explicit per-server requests can answer 404 instead of serving default
    /// data. Aliases (such as `Bilibili`) are real map entries and resolve here.
    pub fn try_server_data(&self, server: Server) -> Option<Arc<ServerData>> {
        self.servers
            .get(&server)
            .filter(|server_data| server_data.loaded.load(Ordering::Acquire))
            .cloned()
    }

    /// The servers a visitor can pick game data from: every loaded server, the
    /// default first and the rest in [`Server::all`] order.
    ///
    /// `Bilibili` is left out even when loaded. It is an alias of CN's game data,
    /// so offering it would put two entries with identical text in the picker.
    ///
    /// A lazy server counts while it is unloaded: a request for it loads it.
    pub fn pickable_servers(&self) -> Vec<Server> {
        std::iter::once(self.default_server)
            .chain(
                Server::all()
                    .iter()
                    .copied()
                    .filter(|s| *s != self.default_server),
            )
            .filter(|s| {
                *s != Server::Bilibili
                    && (self.try_server_data(*s).is_some()
                        || self.servers.get(s).is_some_and(|sd| sd.residency.lazy))
            })
            .collect()
    }

    /// The game data a reload of `server` reads its reference facts from:
    /// every other loaded server, the default first. See
    /// `enrich::reference::align_reference_facts`.
    pub fn reference_servers(&self, server: Server) -> Vec<Arc<GameData>> {
        // Loaded servers only: a lazy server that is not resident would resolve
        // through `game_data`'s fallback to the default a second time.
        self.pickable_servers()
            .into_iter()
            .filter(|s| *s != server)
            .filter_map(|s| self.try_server_data(s))
            .map(|sd| sd.game_data.load_full())
            .collect()
    }

    /// Current game data for a server as a single atomic Arc clone. Callers that
    /// need a borrow `Guard` should `load()` on a held [`ServerData`] instead.
    pub fn game_data(&self, server: Server) -> Arc<GameData> {
        self.server_data(server).game_data.load_full()
    }

    pub fn asset_index(&self, server: Server) -> Arc<AssetIndex> {
        self.server_data(server).asset_index.load_full()
    }

    pub fn default_game_data(&self) -> Arc<GameData> {
        self.game_data(self.default_server)
    }

    pub fn default_asset_index(&self) -> Arc<AssetIndex> {
        self.asset_index(self.default_server)
    }

    /// Publish `new` into `server`'s OWN entry (never a fallback's).
    pub fn swap_game_data(&self, server: Server, new: GameData) {
        self.write_entry(server).game_data.store(Arc::new(new));
    }

    pub fn swap_asset_index(&self, server: Server, new: AssetIndex) {
        self.write_entry(server).asset_index.store(Arc::new(new));
    }

    /// The entry a write for `server` lands in: its own, or the default's for
    /// an unconfigured server (the old `server_data` resolution, which writes
    /// used before the lookup learned to skip unloaded entries).
    fn write_entry(&self, server: Server) -> Arc<ServerData> {
        self.servers
            .get(&server)
            .or_else(|| self.servers.get(&self.default_server))
            .expect("default server data must be present")
            .clone()
    }
}

pub struct AppConfig {
    pub jwt_secret: String,
    /// Seals the durable Yostar credentials in `user_game_credentials`. Held
    /// separately from `jwt_secret` on purpose: a signing key and an encryption
    /// key should not be the same bytes.
    pub game_credential_key: CredentialKey,
    pub rate_limit_rpm: u32,
    pub service_key: String,
    /// Base output directory; per-server dirs derive as `{base}/{server}`.
    pub assets_base_dir: String,
    /// Servers to load at startup (first is the default).
    pub servers: Vec<Server>,
    pub default_server: Server,
    /// Per-server asset-pipeline WebSocket URLs for hot-reload. Each server's
    /// pipeline runs its own WS (typically a distinct port), so reloads are
    /// routed to the matching [`ServerData`].
    pub asset_ws_urls: HashMap<Server, String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let servers: Vec<Server> = std::env::var("SERVERS")
            .ok()
            .map(|v| v.split(',').filter_map(Server::parse).collect::<Vec<_>>())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec![Server::EN]);
        let default_server = servers.first().copied().unwrap_or(Server::EN);

        Self {
            jwt_secret: require_secret("JWT_SECRET", 32),
            // Fails the boot rather than degrading: without this key the
            // durable credential store silently stops persisting, and resync
            // goes back to breaking an hour after login with nothing in the
            // logs to say why. Generate with `openssl rand -hex 32`.
            game_credential_key: CredentialKey::parse(
                &std::env::var("GAME_CREDENTIAL_KEY").expect("GAME_CREDENTIAL_KEY must be set"),
            )
            .expect("GAME_CREDENTIAL_KEY must be 32 bytes, as hex or base64"),
            rate_limit_rpm: std::env::var("RATE_LIMIT_RPM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            service_key: require_secret("SERVICE_KEY", 16),
            assets_base_dir: std::env::var("ASSETS_DIR")
                .unwrap_or_else(|_| "../assets/output".into()),
            servers,
            default_server,
            asset_ws_urls: parse_asset_ws_urls(default_server),
        }
    }
}

/// Read a secret from the environment, refusing one that is present but empty.
///
/// Presence alone is not enough: `SERVICE_KEY=` sets the variable to the empty
/// string, and an empty secret compares equal to an empty header, which would
/// make the credential free to present.
///
/// A short-but-present secret is a weakness, not a hole, so it warns instead of
/// panicking: a deploy shouldn't start failing on a value that worked yesterday.
fn require_secret(name: &str, recommended_len: usize) -> String {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} must be set"));

    assert!(
        !value.trim().is_empty(),
        "{name} is set but empty; refusing to start"
    );

    if value.trim().len() != value.len() {
        warn!(
            secret = name,
            "value has leading or trailing whitespace; it is used verbatim, which is \
             usually not what a stray newline in .env intended"
        );
    }

    if value.len() < recommended_len {
        warn!(
            secret = name,
            length = value.len(),
            recommended = recommended_len,
            "secret is shorter than recommended"
        );
    }

    value
}

/// Parse the per-server hot-reload WebSocket spec, for example
/// `"en=ws://localhost:9160,cn=ws://localhost:9161"`. Entries with an unknown
/// server code or a `disabled`/empty URL are skipped.
fn parse_ws_url_spec(spec: &str) -> HashMap<Server, String> {
    let mut map = HashMap::new();
    for entry in spec.split(',') {
        if let Some((srv, url)) = entry.split_once('=') {
            let url = url.trim();
            if let Some(server) = Server::parse(srv.trim())
                && !url.is_empty()
                && url != "disabled"
            {
                map.insert(server, url.to_string());
            }
        }
    }
    map
}

/// Resolve per-server hot-reload WebSocket URLs from the environment.
///
/// Reads `ASSET_WS_URLS` (preferred), falling back to a single `ASSET_WS_URL`
/// that maps to the default server. Servers without a URL have no hot-reload and
/// refresh on restart; aliases like Bilibili reload via their source server.
fn parse_asset_ws_urls(default_server: Server) -> HashMap<Server, String> {
    if let Ok(raw) = std::env::var("ASSET_WS_URLS") {
        return parse_ws_url_spec(&raw);
    }
    match std::env::var("ASSET_WS_URL") {
        Ok(url) if !url.trim().is_empty() && url.trim() != "disabled" => {
            HashMap::from([(default_server, url.trim().to_string())])
        }
        _ => HashMap::new(),
    }
}

/// Load every configured server's game data and asset index into the map
/// `AppState::new` expects, every server eagerly.
///
/// Lifted out of `main` so binaries build the same map the server does, with
/// the same fallbacks: a server that fails to load is inserted as an unloaded
/// placeholder, and Bilibili shares CN's cell so the two hot-reload together.
/// A tool that rebuilt this by hand would drift from the server the first time
/// either changed, and would drift silently.
///
/// `phase` wraps each server's load for startup instrumentation. It returns a
/// guard the caller drops when the phase ends; pass `|_| ()` from a context
/// with no boot timeline to report to.
///
/// # Panics
/// Only with `GAMEDATA_DEFAULT_DEGRADE=0`, when the DEFAULT server's game data
/// cannot be loaded. See [`load_server_map_with`].
pub fn load_server_map<G>(
    config: &AppConfig,
    phase: impl FnMut(&str) -> G,
) -> HashMap<Server, Arc<ServerData>> {
    load_server_map_with(config, &[], phase)
}

/// [`load_server_map`] with some servers left LAZY: entered unloaded, brought
/// in by the first request that names them (see `app::residency`). The default
/// server is never lazy; naming it here is ignored.
///
/// A server whose data fails to load is logged and entered unloaded; the next
/// asset-watcher `update_complete` for it retries the load. That includes the
/// default server: the process still comes up, explicit requests for the
/// server answer 503, and internal lookups fall back to the first loaded
/// server (see [`AppState::server_data`]). Before, a failed default panicked
/// the boot, so one incomplete tree took every other server down with it.
/// `GAMEDATA_DEFAULT_DEGRADE=0` restores that panic exactly.
///
/// # Panics
/// With `GAMEDATA_DEFAULT_DEGRADE=0`, if the default server fails to load.
pub fn load_server_map_with<G>(
    config: &AppConfig,
    lazy: &[Server],
    mut phase: impl FnMut(&str) -> G,
) -> HashMap<Server, Arc<ServerData>> {
    let mut servers: HashMap<Server, Arc<ServerData>> = HashMap::new();
    // Non-default servers that loaded, held owned until every server is in so
    // the reference-facts pass can read any of them as a reference, in config
    // order.
    let mut pending: Vec<PendingServer> = Vec::new();
    // Servers entered unloaded, placed once the default's fate is known (the
    // old placeholder borrowed the default's `Arc`s).
    let mut unloaded: Vec<(Server, String, String, String, bool)> = Vec::new();
    let default_degrade = !crate::utils::env::switched_off("GAMEDATA_DEFAULT_DEGRADE");

    for &srv in &config.servers {
        let game_data_dir = derive_game_data_dir(&config.assets_base_dir, srv);
        let assets_dir = derive_assets_dir(&config.assets_base_dir, srv);
        let art_dir = art_dir_for(config, srv);
        let is_lazy = srv != config.default_server && lazy.contains(&srv);
        if is_lazy {
            tracing::info!(
                server = srv.as_str(),
                "game data is lazy: loads on the first request for it"
            );
            unloaded.push((srv, game_data_dir, assets_dir, art_dir, true));
            continue;
        }
        let load_result = {
            let _phase = phase(&format!("gamedata:{}", srv.as_str()));
            crate::core::gamedata::init_game_data_with_art(
                std::path::Path::new(&game_data_dir),
                std::path::Path::new(&assets_dir),
                std::path::Path::new(&art_dir),
            )
        };

        match load_result {
            Ok((game_data, asset_index)) => {
                tracing::info!(
                    server = srv.as_str(),
                    operators = game_data.operators.len(),
                    "game data loaded"
                );
                let entry = PendingServer {
                    server: srv,
                    game_data,
                    asset_index,
                    game_data_dir,
                    assets_dir,
                    art_dir,
                };
                if srv == config.default_server {
                    servers.insert(srv, entry.into_server_data());
                } else {
                    pending.push(entry);
                }
            }
            Err(e) if srv == config.default_server && !default_degrade => {
                panic!("failed to load game data for {}: {e}", srv.as_str());
            }
            Err(e) => {
                tracing::error!(
                    server = srv.as_str(),
                    error = %e,
                    default = srv == config.default_server,
                    "game data failed to load; server marked unavailable until a hot reload succeeds"
                );
                unloaded.push((srv, game_data_dir, assets_dir, art_dir, false));
            }
        }
    }

    // The data the old placeholder borrowed (kill-switch path only).
    let fallback = servers
        .get(&config.default_server)
        .map(|d| (d.game_data.load_full(), d.asset_index.load_full()));
    let default_game_data = fallback.as_ref().map(|(gd, _)| Arc::clone(gd));
    for i in 0..pending.len() {
        let (before, rest) = pending.split_at_mut(i);
        let Some((target, after)) = rest.split_first_mut() else {
            continue;
        };
        let references: Vec<&GameData> = default_game_data
            .iter()
            .map(|gd| &**gd)
            .chain(before.iter().chain(after.iter()).map(|p| &p.game_data))
            .collect();
        let facts = crate::core::gamedata::enrich::reference::align_reference_facts(
            &mut target.game_data,
            &references,
        );
        tracing::info!(
            server = target.server.as_str(),
            profiles = facts.profiles,
            operator_channels = facts.operator_channels,
            skin_channels = facts.skin_channels,
            "reference facts aligned"
        );
    }
    for entry in pending {
        servers.insert(entry.server, entry.into_server_data());
    }
    for (srv, game_data_dir, assets_dir, art_dir, is_lazy) in unloaded {
        servers.insert(
            srv,
            Arc::new(ServerData::placeholder(
                game_data_dir,
                assets_dir,
                art_dir,
                is_lazy,
                fallback.clone(),
            )),
        );
    }

    // Bilibili shares CN's Hypergryph data (same Arc cell, hot-reloads together).
    if let Some(cn) = servers.get(&Server::CN).cloned() {
        servers.entry(Server::Bilibili).or_insert(cn);
    }

    servers
}

/// One server's freshly loaded data, before it is published as [`ServerData`].
struct PendingServer {
    server: Server,
    game_data: GameData,
    asset_index: AssetIndex,
    game_data_dir: String,
    assets_dir: String,
    art_dir: String,
}

impl PendingServer {
    fn into_server_data(self) -> Arc<ServerData> {
        Arc::new(ServerData {
            game_data: ArcSwap::from_pointee(self.game_data),
            asset_index: ArcSwap::from_pointee(self.asset_index),
            game_data_dir: self.game_data_dir,
            assets_dir: self.assets_dir,
            art_dir: self.art_dir,
            loaded: AtomicBool::new(true),
            residency: crate::app::residency::Residency::new(false),
        })
    }
}

/// The tree a server's art is read from.
///
/// A server pulled with the asset pipeline's `gamedata` profile (JP and KR,
/// which exist for their text) has no `portraits/` directory. Its asset index
/// built from its own tree resolved every portrait, skin, skill and item icon
/// to `None` (KR `char_377_gdglow`: 4 of 4 image fields `None` against EN's
/// 4 paths, 2026-10-06). Such a server reads the default server's art instead;
/// the asset routes already fall back to the default server's files, so the
/// paths it carries are servable.
///
/// `ART_FALLBACK=0` restores the old behaviour exactly: every server reads its
/// own tree.
pub fn art_dir_for(config: &AppConfig, server: Server) -> String {
    let own = derive_assets_dir(&config.assets_base_dir, server);
    if crate::utils::env::switched_off("ART_FALLBACK")
        || server == config.default_server
        || std::path::Path::new(&own).join("portraits").is_dir()
    {
        return own;
    }
    derive_assets_dir(&config.assets_base_dir, config.default_server)
}

/// Per-server asset directory, for example `../assets/output/cn`.
pub fn derive_assets_dir(base: &str, server: Server) -> String {
    format!("{base}/{}", server.as_str())
}

/// Per-server gamedata excel directory, for example
/// `../assets/output/cn/gamedata/excel`.
pub fn derive_game_data_dir(base: &str, server: Server) -> String {
    format!("{base}/{}/gamedata/excel", server.as_str())
}

/// The server a standalone bin should load game data for.
///
/// Standalone bins (e.g. `regrade-users`, `resync-gacha`, `generate-dps`) load
/// a single server's game data rather than the full per-server map the app
/// holds. They resolve the base dir from `ASSETS_DIR` and the server from this:
/// `BIN_SERVER` if set, else the first entry of `SERVERS` (mirroring
/// [`AppConfig::from_env`]'s default-server selection), else EN.
pub fn default_bin_server_from_env() -> Server {
    resolve_bin_server(
        std::env::var("BIN_SERVER").ok().as_deref(),
        std::env::var("SERVERS").ok().as_deref(),
    )
}

/// Pure core of [`default_bin_server_from_env`]: `bin_server` if it parses, else
/// the first parseable entry of the comma-separated `servers`, else EN.
fn resolve_bin_server(bin_server: Option<&str>, servers: Option<&str>) -> Server {
    bin_server
        .and_then(Server::parse)
        .or_else(|| servers.and_then(|v| v.split(',').find_map(Server::parse)))
        .unwrap_or(Server::EN)
}

#[cfg(test)]
mod tests {
    use super::{derive_assets_dir, derive_game_data_dir, parse_ws_url_spec};
    use crate::core::hypergryph::constants::Server;

    #[test]
    fn derives_per_server_dirs() {
        assert_eq!(
            derive_assets_dir("../assets/output", Server::CN),
            "../assets/output/cn"
        );
        assert_eq!(
            derive_game_data_dir("../assets/output", Server::EN),
            "../assets/output/en/gamedata/excel"
        );
    }

    #[test]
    fn bin_server_falls_back_to_first_of_servers_then_en() {
        use super::resolve_bin_server;
        // BIN_SERVER takes precedence.
        assert_eq!(resolve_bin_server(Some("cn"), Some("en,kr")), Server::CN);
        // Else the first parseable SERVERS entry (unknown codes skipped).
        assert_eq!(resolve_bin_server(None, Some("zz,kr,en")), Server::KR);
        // Else EN.
        assert_eq!(resolve_bin_server(None, None), Server::EN);
        assert_eq!(resolve_bin_server(Some("zz"), Some("nope")), Server::EN);
    }

    #[test]
    fn parses_multi_server_ws_spec() {
        let map = parse_ws_url_spec("en=ws://localhost:9160, cn=ws://localhost:9161");
        assert_eq!(
            map.get(&Server::EN).map(String::as_str),
            Some("ws://localhost:9160")
        );
        assert_eq!(
            map.get(&Server::CN).map(String::as_str),
            Some("ws://localhost:9161")
        );
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn ws_spec_accepts_bili_alias_and_skips_invalid() {
        let map = parse_ws_url_spec("bilibili=ws://h:1, zz=ws://h:2, cn=disabled, kr=");
        assert_eq!(
            map.get(&Server::Bilibili).map(String::as_str),
            Some("ws://h:1")
        );
        assert!(!map.contains_key(&Server::CN), "disabled URL skipped");
        assert!(!map.contains_key(&Server::KR), "empty URL skipped");
        assert_eq!(map.len(), 1, "unknown server code skipped");
    }
}
