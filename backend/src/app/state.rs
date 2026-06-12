use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use arc_swap::ArcSwap;
use reqwest::Client;
use sqlx::PgPool;

use crate::app::cache::store::CacheStore;
use crate::core::gamedata::{assets::AssetIndex, types::GameData};
use crate::core::hypergryph::constants::Server;

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
}

pub struct AppStateInner {
    pub db: PgPool,
    pub cache: CacheStore,
    pub servers: HashMap<Server, Arc<ServerData>>,
    pub default_server: Server,
    pub config: Arc<AppConfig>,
    pub http_client: Client,
}

impl AppState {
    pub fn new(
        db: PgPool,
        cache: CacheStore,
        servers: HashMap<Server, Arc<ServerData>>,
        default_server: Server,
        config: AppConfig,
        client: Client,
    ) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                db,
                cache,
                servers,
                default_server,
                config: Arc::new(config),
                http_client: client,
            }),
        }
    }

    /// Resolve a server's data, falling back to the default server when the
    /// requested one is not loaded. Use this for internal access where a server
    /// is always available (the default and configured servers).
    pub fn server_data(&self, server: Server) -> Arc<ServerData> {
        self.servers
            .get(&server)
            .or_else(|| self.servers.get(&self.default_server))
            .expect("default server data must be present")
            .clone()
    }

    /// Resolve a server's data only when it is actually loaded, without the
    /// default fallback. Returns `None` for valid but unconfigured servers so
    /// explicit per-server requests can answer 404 instead of serving default
    /// data. Aliases (such as `Bilibili`) are real map entries and resolve here.
    pub fn try_server_data(&self, server: Server) -> Option<Arc<ServerData>> {
        self.servers.get(&server).cloned()
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

    pub fn swap_game_data(&self, server: Server, new: GameData) {
        self.server_data(server).game_data.store(Arc::new(new));
    }

    pub fn swap_asset_index(&self, server: Server, new: AssetIndex) {
        self.server_data(server).asset_index.store(Arc::new(new));
    }
}

pub struct AppConfig {
    pub jwt_secret: String,
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
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            rate_limit_rpm: std::env::var("RATE_LIMIT_RPM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            service_key: std::env::var("SERVICE_KEY").expect("SERVICE_KEY must be set"),
            assets_base_dir: std::env::var("ASSETS_DIR")
                .unwrap_or_else(|_| "../assets/output".into()),
            servers,
            default_server,
            asset_ws_urls: parse_asset_ws_urls(default_server),
        }
    }
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
