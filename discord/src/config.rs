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

/// Text-to-speech in voice channels, spoken by Google Translate: the default voice and the
/// limits it speaks under.
///
/// Every field is optional, so a config without a `tts` section still parses. The fields of the
/// sherpa-onnx releases (`engine`, `model_dir`, ...) and `user_cooldown_ms` are still accepted, so
/// those configs keep parsing, and ignored with one warning listing them (see
/// [`TtsConfig::deprecated_fields`]).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsConfig {
    /// `false` turns TTS off: `/tts join` says it is unavailable. Defaults to on.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// The voice for members who haven't picked one, as a `/tts voice` id. Defaults to `en-us`.
    #[serde(default)]
    pub default_voice: Option<String>,
    /// Characters spoken per message after cleanup; longer text is cut at a word boundary.
    #[serde(default)]
    pub max_chars: Option<usize>,
    /// Seconds without speech before the bot leaves the channel.
    #[serde(default)]
    pub idle_secs: Option<u64>,
    /// Messages waiting per guild; anything past this is dropped without a reply.
    #[serde(default)]
    pub queue_max: Option<usize>,
    /// Messages one member may have waiting; more from them are dropped without a reply until
    /// some are spoken.
    #[serde(default)]
    pub user_queue_max: Option<usize>,
    /// Wait after the voice connection is up before the first playback, in milliseconds. Covers
    /// the DAVE handshake, which songbird does not signal (see `tts::session`).
    #[serde(default)]
    pub settle_ms: Option<u64>,
    // Deprecated: read by earlier releases, ignored now. Any JSON value is accepted. Kept as
    // fields, not a flattened struct, because serde can't combine `flatten` with
    // `deny_unknown_fields`.
    #[serde(default)]
    engine: Option<serde_json::Value>,
    #[serde(default)]
    model_dir: Option<serde_json::Value>,
    #[serde(default)]
    model: Option<serde_json::Value>,
    #[serde(default)]
    tokens: Option<serde_json::Value>,
    #[serde(default)]
    data_dir: Option<serde_json::Value>,
    #[serde(default)]
    voices: Option<serde_json::Value>,
    #[serde(default)]
    lexicon: Option<serde_json::Value>,
    #[serde(default)]
    speaker: Option<serde_json::Value>,
    #[serde(default)]
    user_cooldown_ms: Option<serde_json::Value>,
}

impl TtsConfig {
    /// The deprecated fields this config sets, for the one startup warning.
    #[must_use]
    pub fn deprecated_fields(&self) -> Vec<&'static str> {
        let d = self;
        [
            ("engine", d.engine.is_some()),
            ("model_dir", d.model_dir.is_some()),
            ("model", d.model.is_some()),
            ("tokens", d.tokens.is_some()),
            ("data_dir", d.data_dir.is_some()),
            ("voices", d.voices.is_some()),
            ("lexicon", d.lexicon.is_some()),
            ("speaker", d.speaker.is_some()),
            ("user_cooldown_ms", d.user_cooldown_ms.is_some()),
        ]
        .into_iter()
        .filter_map(|(name, set)| set.then_some(name))
        .collect()
    }

    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    #[must_use]
    pub fn max_chars(&self) -> usize {
        self.max_chars.unwrap_or(300)
    }

    #[must_use]
    pub fn idle_secs(&self) -> u64 {
        self.idle_secs.unwrap_or(300)
    }

    #[must_use]
    pub fn queue_max(&self) -> usize {
        self.queue_max.unwrap_or(10)
    }

    #[must_use]
    pub fn user_queue_max(&self) -> usize {
        self.user_queue_max.unwrap_or(3)
    }

    #[must_use]
    pub fn settle_ms(&self) -> u64 {
        self.settle_ms.unwrap_or(1500)
    }
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
        assert!(config.tts.enabled());
        assert_eq!(config.tts.default_voice, None);
        assert_eq!(config.tts.max_chars(), 300);
        assert_eq!(config.tts.user_queue_max(), 3);
        assert!(config.tts.deprecated_fields().is_empty());
        let off: Config = serde_json::from_str(r#"{"tts": {"enabled": false}}"#).unwrap();
        assert!(!off.tts.enabled());
        assert!(serde_json::from_str::<Config>(r#"{"tts": {"voice": "x"}}"#).is_err());
    }

    #[test]
    fn sherpa_era_configs_still_parse_and_name_what_is_ignored() {
        // The Piper release's example section, and the Kokoro fields.
        let piper: Config = serde_json::from_str(
            r#"{"tts": {
                "model_dir": "tts/vits-piper-en_US-ljspeech-medium",
                "max_chars": 280, "idle_secs": 300, "queue_max": 10,
                "user_cooldown_ms": 2000, "settle_ms": 1500
            }}"#,
        )
        .unwrap();
        assert_eq!(piper.tts.max_chars(), 280);
        assert_eq!(piper.tts.settle_ms(), 1500);
        assert_eq!(
            piper.tts.deprecated_fields(),
            ["model_dir", "user_cooldown_ms"]
        );
        let kokoro: Config = serde_json::from_str(
            r#"{"tts": {"engine": "kokoro", "model": "m.onnx", "tokens": "t", "data_dir": "d",
                "voices": "v.bin", "lexicon": "l", "speaker": 3}}"#,
        )
        .unwrap();
        assert_eq!(kokoro.tts.deprecated_fields().len(), 7);
    }

    #[test]
    fn example_config_parses() {
        let config: Config = serde_json::from_str(include_str!("../config.example.json")).unwrap();
        assert_eq!(config.tts.settle_ms(), 1500);
        assert_eq!(config.tts.default_voice.as_deref(), Some("en-us"));
        assert!(config.tts.deprecated_fields().is_empty());
        assert_eq!(config.assets.resolved_servers().len(), 4);
    }
}
