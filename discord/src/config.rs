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

/// Which sherpa-onnx voice family speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TtsEngine {
    /// Kokoro-82M (a `StyleTTS` 2 model): more natural prosody, far more CPU per second of audio.
    Kokoro,
    /// A Piper VITS voice, the default: flatter, and cheap enough for one shared core.
    Piper,
}

impl TtsEngine {
    /// Where this engine's voice is looked for when the config names no `model_dir`, relative to
    /// the working directory.
    #[must_use]
    pub const fn default_model_dir(self) -> &'static str {
        match self {
            // The fp32 package: the int8 one produced silence at one thread (see `tts::engine`).
            Self::Kokoro => "tts/kokoro-multi-lang-v1_0",
            Self::Piper => "tts/vits-piper-en_US-ljspeech-medium",
        }
    }
}

/// The engine used when the config names neither an engine nor a model directory.
pub const DEFAULT_TTS_ENGINE: TtsEngine = TtsEngine::Piper;

/// Kokoro v1.0's speaker id for `af_heart`, the voice its author grades highest (A).
pub const DEFAULT_KOKORO_SPEAKER: i32 = 3;

/// Text-to-speech in voice channels: which voice speaks, where it lives, and the limits it speaks
/// under.
///
/// Every field is optional, so a config without a `tts` section still parses. With no section the
/// bot looks for [`DEFAULT_TTS_ENGINE`]'s voice under its default directory; when the files aren't
/// there, TTS stays off with one warning and nothing else changes. `enabled: false` turns it off
/// without the warning.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TtsConfig {
    /// `false` turns TTS off without the missing-voice warning. Defaults to on.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// `"kokoro"` or `"piper"`. When unset, a `model_dir` holding `voices.bin` is Kokoro and any
    /// other is Piper, so a config written for the Piper release keeps working; with neither set
    /// it is [`DEFAULT_TTS_ENGINE`].
    #[serde(default)]
    pub engine: Option<TtsEngine>,
    /// The voice's directory as its sherpa-onnx release archive unpacks: `kokoro-*` (model,
    /// `voices.bin`, `tokens.txt`, lexicons, `espeak-ng-data/`) or `vits-piper-*` (one `.onnx`,
    /// `tokens.txt`, `espeak-ng-data/`). Defaults to the engine's directory under `tts/`.
    #[serde(default)]
    pub model_dir: Option<String>,
    /// The `.onnx` model. Defaults to the only `.onnx` file in `model_dir`.
    #[serde(default)]
    pub model: Option<String>,
    /// Defaults to `<model_dir>/tokens.txt`.
    #[serde(default)]
    pub tokens: Option<String>,
    /// Defaults to `<model_dir>/espeak-ng-data`.
    #[serde(default)]
    pub data_dir: Option<String>,
    /// Kokoro only: the speaker table. Defaults to `<model_dir>/voices.bin`.
    #[serde(default)]
    pub voices: Option<String>,
    /// Kokoro only: comma-separated lexicons. Defaults to `<model_dir>/lexicon-us-en.txt`.
    #[serde(default)]
    pub lexicon: Option<String>,
    /// Kokoro only: the speaker id. Defaults to [`DEFAULT_KOKORO_SPEAKER`] (`af_heart`); the ids
    /// are listed in the model's `speaker_names` metadata (`af_bella` 2, `bf_emma` 21, ...).
    #[serde(default)]
    pub speaker: Option<i32>,
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
    /// Deprecated and ignored: it dropped a member's second message inside the window, which is
    /// the bug `user_queue_max` replaced. Still accepted so configs that set it keep parsing.
    #[serde(default)]
    pub user_cooldown_ms: Option<u64>,
    /// Wait after the voice connection is up before the first playback, in milliseconds. Covers
    /// the DAVE handshake, which songbird does not signal (see `tts::session`).
    #[serde(default)]
    pub settle_ms: Option<u64>,
}

impl TtsConfig {
    /// The engine that speaks, explicit or inferred from `model_dir` (see [`Self::engine`]).
    #[must_use]
    pub fn resolved_engine(&self) -> TtsEngine {
        if let Some(engine) = self.engine {
            return engine;
        }
        match &self.model_dir {
            Some(dir) if std::path::Path::new(dir).join("voices.bin").is_file() => {
                TtsEngine::Kokoro
            }
            Some(_) => TtsEngine::Piper,
            None => DEFAULT_TTS_ENGINE,
        }
    }

    /// `model_dir`, or the resolved engine's default directory.
    #[must_use]
    pub fn resolved_model_dir(&self) -> String {
        self.model_dir
            .clone()
            .unwrap_or_else(|| self.resolved_engine().default_model_dir().to_string())
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
        assert_eq!(config.tts.resolved_engine(), DEFAULT_TTS_ENGINE);
        assert_eq!(
            config.tts.resolved_model_dir(),
            DEFAULT_TTS_ENGINE.default_model_dir()
        );
        assert_eq!(config.tts.max_chars(), 300);
        let off: Config = serde_json::from_str(r#"{"tts": {"enabled": false}}"#).unwrap();
        assert!(!off.tts.enabled());
        assert_eq!(off.tts.queue_max(), 10);
        assert_eq!(off.tts.user_queue_max(), 3);
        assert!(serde_json::from_str::<Config>(r#"{"tts": {"voice": "x"}}"#).is_err());
        // A config from before `user_queue_max` still parses; the old field is ignored.
        let old: Config = serde_json::from_str(r#"{"tts": {"user_cooldown_ms": 2000}}"#).unwrap();
        assert_eq!(old.tts.user_cooldown_ms, Some(2000));
        assert!(serde_json::from_str::<Config>(r#"{"tts": {"engine": "espeak"}}"#).is_err());
    }

    #[test]
    fn engine_is_explicit_or_inferred_from_the_directory() {
        let piper: Config = serde_json::from_str(r#"{"tts": {"engine": "piper"}}"#).unwrap();
        assert_eq!(piper.tts.resolved_engine(), TtsEngine::Piper);
        assert_eq!(
            piper.tts.resolved_model_dir(),
            "tts/vits-piper-en_US-ljspeech-medium"
        );
        // A Piper-era config names only its directory, which has no voices.bin: still Piper.
        let legacy: Config = serde_json::from_str(
            r#"{"tts": {"model_dir": "tts/vits-piper-en_US-ljspeech-medium"}}"#,
        )
        .unwrap();
        assert_eq!(legacy.tts.resolved_engine(), TtsEngine::Piper);
        let dir = std::env::temp_dir().join(format!("tts-kokoro-probe-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("voices.bin"), b"").unwrap();
        let kokoro = TtsConfig {
            model_dir: Some(dir.display().to_string()),
            ..TtsConfig::default()
        };
        assert_eq!(kokoro.resolved_engine(), TtsEngine::Kokoro);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn example_config_parses() {
        let config: Config = serde_json::from_str(include_str!("../config.example.json")).unwrap();
        assert_eq!(config.tts.settle_ms(), 1500);
        assert_eq!(config.assets.resolved_servers().len(), 4);
    }
}
