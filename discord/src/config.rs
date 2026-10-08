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
    #[serde(default)]
    pub tts: TtsConfig,
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

/// Text-to-speech in voice channels: where the Piper voice lives and the limits it speaks under.
///
/// Every field is optional, so a config without a `tts` section still parses. With no section the
/// bot looks for the voice under [`DEFAULT_TTS_MODEL_DIR`]; when the files aren't there, TTS stays
/// off with one warning and nothing else changes. `enabled: false` turns it off without the
/// warning.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// A sherpa-onnx `vits-piper-*` directory: one `.onnx` model, `tokens.txt` and
    /// `espeak-ng-data/`, as the release archive unpacks.
    #[serde(default = "default_tts_model_dir")]
    pub model_dir: String,
    /// The `.onnx` model. Defaults to the only `.onnx` file in `model_dir`.
    #[serde(default)]
    pub model: Option<String>,
    /// Defaults to `<model_dir>/tokens.txt`.
    #[serde(default)]
    pub tokens: Option<String>,
    /// Defaults to `<model_dir>/espeak-ng-data`.
    #[serde(default)]
    pub data_dir: Option<String>,
    /// Characters spoken per message after cleanup; longer text is cut at a word boundary.
    #[serde(default = "default_tts_max_chars")]
    pub max_chars: usize,
    /// Seconds without speech before the bot leaves the channel.
    #[serde(default = "default_tts_idle_secs")]
    pub idle_secs: u64,
    /// Messages waiting per guild; anything past this is dropped without a reply.
    #[serde(default = "default_tts_queue_max")]
    pub queue_max: usize,
    /// Least time between two spoken messages from one member, in milliseconds.
    #[serde(default = "default_tts_user_cooldown_ms")]
    pub user_cooldown_ms: u64,
    /// Wait after the voice connection is up before the first playback, in milliseconds. Covers
    /// the DAVE handshake, which songbird does not signal (see `tts::session`).
    #[serde(default = "default_tts_settle_ms")]
    pub settle_ms: u64,
}

/// Where the voice is looked for when the config names no `model_dir`, relative to the working
/// directory.
pub const DEFAULT_TTS_MODEL_DIR: &str = "tts/vits-piper-en_US-ljspeech-medium";

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            model_dir: default_tts_model_dir(),
            model: None,
            tokens: None,
            data_dir: None,
            max_chars: default_tts_max_chars(),
            idle_secs: default_tts_idle_secs(),
            queue_max: default_tts_queue_max(),
            user_cooldown_ms: default_tts_user_cooldown_ms(),
            settle_ms: default_tts_settle_ms(),
        }
    }
}

const fn default_true() -> bool {
    true
}

fn default_tts_model_dir() -> String {
    DEFAULT_TTS_MODEL_DIR.to_string()
}

const fn default_tts_max_chars() -> usize {
    300
}

const fn default_tts_idle_secs() -> u64 {
    300
}

const fn default_tts_queue_max() -> usize {
    10
}

const fn default_tts_user_cooldown_ms() -> u64 {
    2000
}

const fn default_tts_settle_ms() -> u64 {
    1500
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configs_without_tts_still_parse() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert!(config.tts.enabled);
        assert_eq!(config.tts.model_dir, DEFAULT_TTS_MODEL_DIR);
        assert_eq!(config.tts.max_chars, 300);
        let off: Config = serde_json::from_str(r#"{"tts": {"enabled": false}}"#).unwrap();
        assert!(!off.tts.enabled);
        assert_eq!(off.tts.queue_max, 10);
        assert!(serde_json::from_str::<Config>(r#"{"tts": {"voice": "x"}}"#).is_err());
    }

    #[test]
    fn example_config_parses() {
        let config: Config = serde_json::from_str(include_str!("../config.example.json")).unwrap();
        assert_eq!(config.tts.settle_ms, 1500);
        assert_eq!(config.assets.resolved_servers().len(), 4);
    }
}
