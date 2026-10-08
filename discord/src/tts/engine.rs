//! The Piper voice, run through sherpa-onnx.
//!
//! The model is loaded once, on the first message that needs it, and kept for the life of the
//! process. The VPS has three cores shared with the backend, so synthesis is held to one thread
//! and one job at a time across every guild: [`Synth`] owns the only permit, and both the load and
//! each synthesis run on the blocking pool under it.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig,
    OfflineTtsVitsModelConfig,
};
use tokio::sync::{OnceCell, Semaphore};

use crate::config::TtsConfig;
use crate::types::Error;

/// Synthesis threads. One: a second core would halve the wait at the backend's expense.
const NUM_THREADS: i32 = 1;

/// The three files a `vits-piper-*` voice needs.
#[derive(Debug, Clone)]
pub struct ModelPaths {
    pub model: PathBuf,
    pub tokens: PathBuf,
    pub data_dir: PathBuf,
}

impl ModelPaths {
    /// Resolve the voice's files from `config`, or say which one is missing.
    pub fn resolve(config: &TtsConfig) -> Result<Self, Error> {
        let dir = Path::new(&config.model_dir);
        let model = match &config.model {
            Some(model) => PathBuf::from(model),
            None => only_onnx_in(dir)?,
        };
        let tokens = config
            .tokens
            .as_ref()
            .map_or_else(|| dir.join("tokens.txt"), PathBuf::from);
        let data_dir = config
            .data_dir
            .as_ref()
            .map_or_else(|| dir.join("espeak-ng-data"), PathBuf::from);
        if !model.is_file() {
            return Err(format!("model {} not found", model.display()).into());
        }
        if !tokens.is_file() {
            return Err(format!("tokens {} not found", tokens.display()).into());
        }
        if !data_dir.is_dir() {
            return Err(format!("espeak-ng data {} not found", data_dir.display()).into());
        }
        Ok(Self {
            model,
            tokens,
            data_dir,
        })
    }
}

/// The one `.onnx` file in `dir`.
fn only_onnx_in(dir: &Path) -> Result<PathBuf, Error> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("model directory {}: {e}", dir.display()))?;
    let mut found: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "onnx"))
        .collect();
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!("no .onnx model in {}", dir.display()).into()),
        n => Err(format!("{n} .onnx models in {}; set tts.model", dir.display()).into()),
    }
}

/// Mono samples in `[-1, 1]` at `sample_rate`.
#[derive(Debug, Clone)]
pub struct Speech {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl Speech {
    /// Playing time in seconds.
    #[must_use]
    pub fn seconds(&self) -> f64 {
        if self.sample_rate == 0 {
            return 0.0;
        }
        f64::from(u32::try_from(self.samples.len()).unwrap_or(u32::MAX))
            / f64::from(self.sample_rate)
    }
}

/// A loaded voice.
pub struct Engine {
    tts: OfflineTts,
}

impl Engine {
    /// Load the model. Blocking: run it on the blocking pool.
    pub fn load(paths: &ModelPaths) -> Result<Self, Error> {
        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                vits: OfflineTtsVitsModelConfig {
                    model: Some(paths.model.display().to_string()),
                    tokens: Some(paths.tokens.display().to_string()),
                    data_dir: Some(paths.data_dir.display().to_string()),
                    ..Default::default()
                },
                num_threads: NUM_THREADS,
                provider: Some("cpu".into()),
                ..Default::default()
            },
            max_num_sentences: 1,
            ..Default::default()
        };
        let tts = OfflineTts::create(&config)
            .ok_or_else(|| format!("sherpa-onnx refused the model at {}", paths.model.display()))?;
        Ok(Self { tts })
    }

    /// Speak `text`. Blocking and CPU-bound: run it on the blocking pool, one at a time.
    ///
    /// `text` must hold no NUL byte (the C API takes a C string; the wrapper panics on one).
    /// [`super::text::clean`] drops every control character, NUL included.
    #[must_use]
    pub fn synthesize(&self, text: &str) -> Option<Speech> {
        if text.contains('\0') {
            return None;
        }
        let audio = self.tts.generate_with_config(
            text,
            &GenerationConfig::default(),
            None::<fn(&[f32], f32) -> bool>,
        )?;
        let sample_rate = u32::try_from(audio.sample_rate()).ok()?;
        let samples = audio.samples().to_vec();
        (!samples.is_empty()).then_some(Speech {
            samples,
            sample_rate,
        })
    }
}

/// The voice as the rest of the bot sees it: lazily loaded, one job at a time.
pub struct Synth {
    paths: Option<ModelPaths>,
    engine: OnceCell<Option<Arc<Engine>>>,
    /// Set once the load has failed, so the message handler stops joining channels for it.
    failed: AtomicBool,
    permit: Semaphore,
}

impl Synth {
    /// `paths` is `None` when TTS is off or its files are missing; the caller has logged why.
    #[must_use]
    pub fn new(paths: Option<ModelPaths>) -> Self {
        Self {
            paths,
            engine: OnceCell::new(),
            failed: AtomicBool::new(false),
            permit: Semaphore::new(1),
        }
    }

    /// Whether speaking can work: the files were found and loading them has not failed.
    #[must_use]
    pub fn available(&self) -> bool {
        self.paths.is_some() && !self.failed.load(Ordering::Relaxed)
    }

    /// Speak `text`, loading the model first if this is the first time. `None` when the voice is
    /// unavailable or produced nothing.
    pub async fn speak(&self, text: String) -> Option<Speech> {
        let paths = self.paths.as_ref()?;
        let _permit = self.permit.acquire().await.ok()?;
        let engine = self
            .engine
            .get_or_init(|| async {
                let paths = paths.clone();
                let started = Instant::now();
                let loaded = tokio::task::spawn_blocking(move || Engine::load(&paths)).await;
                match loaded {
                    Ok(Ok(engine)) => {
                        tracing::info!("TTS voice loaded in {} ms", started.elapsed().as_millis());
                        Some(Arc::new(engine))
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("TTS disabled: the voice failed to load: {e}");
                        self.failed.store(true, Ordering::Relaxed);
                        None
                    }
                    Err(e) => {
                        tracing::warn!("TTS disabled: the voice load panicked: {e}");
                        self.failed.store(true, Ordering::Relaxed);
                        None
                    }
                }
            })
            .await
            .clone()?;
        let started = Instant::now();
        let speech = tokio::task::spawn_blocking(move || engine.synthesize(&text))
            .await
            .ok()
            .flatten();
        if let Some(speech) = &speech {
            tracing::debug!(
                "TTS synthesized {:.2} s of audio in {} ms",
                speech.seconds(),
                started.elapsed().as_millis()
            );
        }
        speech
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Times the real voice on three sentences and writes them as WAVs.
    ///
    /// Needs the model on disk, so it is ignored by default:
    /// `TTS_MODEL_DIR=<vits-piper dir> TTS_OUT_DIR=<dir> cargo test --release -- --ignored
    /// --nocapture synthesize_three_sentences`.
    #[test]
    #[ignore = "needs the Piper model on disk"]
    fn synthesize_three_sentences() {
        let dir = std::env::var("TTS_MODEL_DIR").expect("set TTS_MODEL_DIR");
        let out = PathBuf::from(std::env::var("TTS_OUT_DIR").expect("set TTS_OUT_DIR"));
        let config = TtsConfig {
            model_dir: dir,
            ..TtsConfig::default()
        };
        let paths = ModelPaths::resolve(&config).expect("model files");

        let started = Instant::now();
        let engine = Engine::load(&paths).expect("load");
        println!("load: {} ms", started.elapsed().as_millis());

        let sentences = [
            "Hello there, Doctor.",
            "The quick brown fox jumps over the lazy dog while the band plays a slow song \
             outside the old town hall.",
            "Text to speech in a voice channel has to be quick enough that a reply still \
             lands while the conversation is about it. This sentence is long on purpose, so \
             the timing shows what the longest allowed message costs on one thread, which is \
             the budget the bot runs with on a small shared server.",
        ];
        for (i, text) in sentences.iter().enumerate() {
            let started = Instant::now();
            let speech = engine.synthesize(text).expect("audio");
            let wall = started.elapsed().as_secs_f64();
            let audio = speech.seconds();
            println!(
                "sentence {}: {} chars, {} words, wall {wall:.3} s, audio {audio:.3} s, \
                 rtf {:.3}, sample rate {} Hz",
                i + 1,
                text.chars().count(),
                text.split_whitespace().count(),
                wall / audio,
                speech.sample_rate
            );
            let path = out.join(format!("tts-sentence-{}.wav", i + 1));
            assert!(sherpa_onnx::write(
                &path.display().to_string(),
                &speech.samples,
                i32::try_from(speech.sample_rate).unwrap()
            ));
        }
    }
}
