use std::{fs, path::Path};

use serde::Deserialize;

use crate::types::Error;

pub const DEFAULT_CONFIG_PATH: &str = "config.json";

/// Top-level config loaded once at startup.
///
/// All sections default to empty so partial configs still parse - validate per-section
/// where the values are actually used.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    #[serde(default)]
    pub assets: AssetsConfig,
}

/// HTTP endpoints the status check pings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointsConfig {
    #[serde(default)]
    pub local_backend: String,
    #[serde(default)]
    pub local_frontend: String,
    #[serde(default)]
    pub public_backend: String,
    #[serde(default)]
    pub public_frontend: String,
}

/// One Arknights asset-pipeline server the bot watches. The bot opens a WS per
/// server and labels every announcement with `label` (e.g. "EN", "CN").
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetServerConfig {
    pub label: String,
    pub ws_url: String,
}

/// Connection settings for the Arknights asset pipeline `WebSockets` (`run.mjs ws`).
///
/// Prefer `servers` (one entry per region). The legacy single `ws_url` is still
/// honored as one server labeled "EN". With neither set, the watcher is disabled.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetsConfig {
    #[serde(default)]
    pub ws_url: String,
    #[serde(default)]
    pub servers: Vec<AssetServerConfig>,
    #[serde(default = "default_reconnect_secs")]
    pub reconnect_secs: u64,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            ws_url: String::new(),
            servers: Vec::new(),
            reconnect_secs: default_reconnect_secs(),
        }
    }
}

impl AssetsConfig {
    /// The servers to watch: `servers` if non-empty, else the legacy `ws_url` as a
    /// single "EN" server, else empty (watcher disabled).
    #[must_use]
    pub fn resolved_servers(&self) -> Vec<AssetServerConfig> {
        if !self.servers.is_empty() {
            self.servers.clone()
        } else if self.ws_url.is_empty() {
            Vec::new()
        } else {
            vec![AssetServerConfig {
                label: "EN".to_string(),
                ws_url: self.ws_url.clone(),
            }]
        }
    }
}

const fn default_reconnect_secs() -> u64 {
    5
}

impl Config {
    /// Load and parse `path`.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let bytes = fs::read(path)
            .map_err(|e| format!("Failed to read config at {}: {e}", path.display()))?;
        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Failed to parse config at {}: {e}", path.display()))?;
        Ok(config)
    }

    /// Load from `$DISCORD_CONFIG_PATH` if set, else `config.json` in the working dir.
    pub fn load_default() -> Result<Self, Error> {
        let path = std::env::var("DISCORD_CONFIG_PATH")
            .unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
        Self::load(path)
    }
}
