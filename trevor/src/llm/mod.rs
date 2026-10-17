//! The model client: a local `llama-server` (the default backend) or an OpenAI-compatible chat endpoint
//! (Cloudflare Workers AI; [`openai`]).
//!
//! llama.cpp backend: chat formatting goes through the server's own `/apply-template`, so the prompt is rendered by
//! the model's Jinja template and nothing here knows Gemma's or Qwen's turn markers. Generation goes through
//! `/completion`, the only endpoint that reports `stop_type`, which is how a grammar-bound output cut off at `n_predict`
//! is told apart from a finished one.
//!
//! Server behaviours this client is written around (llama.cpp server README): requests queue rather than fail when
//! every slot is busy, so concurrency is bounded here to the server's slot count; and one failed decode answers 500 to
//! every in-flight request, so 5xx and transport errors are retried.
//!
//! Streaming: when a stream sink is set ([`Llm::set_stream`], the `ask --worker` job loop), a call with no grammar and
//! no stop strings (an answer call) streams (`stream: true`) and sends its text pieces to the sink as they arrive; the
//! returned [`Completion`] is the same. Without a sink no request streams.
//!
//! The OpenAI-compatible backend has no grammar: a grammar-bound call (router, question form, flags) is sent as a plain
//! prompt asking for one JSON object, validated (it must parse, and its keys must be keys the grammar names), asked once
//! more on failure, and then fails with [`JsonInvalid`], on which `ask` falls back to the keyword router or the fact
//! form.

mod json;
mod llama;
pub mod openai;
mod usage;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedSender;

pub use json::{JsonInvalid, grammar_keys, is_json_invalid, valid_json};
pub use openai::{OpenAiConfig, Thinking};
pub use usage::UsageSnapshot;
use usage::Usage;

#[derive(Clone)]
pub struct Llm {
    http: reqwest::Client,
    backend: Backend,
    sem: Arc<Semaphore>,
    /// File name of the loaded model, from `/props` (llama.cpp), or the first model id (OpenAI-compatible).
    pub model: String,
    pub slots: usize,
    /// `with_dump`: every rendered prompt, its token counts and the output, one JSON line per request.
    dump: Option<Arc<Mutex<std::fs::File>>>,
    /// `without_prompt_cache`: send `cache_prompt: false`, so no request reuses another's KV cache.
    no_cache: bool,
    usage: Arc<Usage>,
    sink: Arc<Mutex<Option<UnboundedSender<StreamEvent>>>>,
}

#[derive(Clone)]
enum Backend {
    LlamaCpp { base: String },
    OpenAi(Arc<openai::Client>),
}

/// What a streaming call sends to the sink: `Start` before each attempt's first piece, then its text pieces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamEvent {
    Start,
    Piece(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub content: String,
    /// `eos`, `word` or `limit` (an OpenAI `finish_reason` of `stop` is `eos`, `length` is `limit`).
    pub stop_type: String,
    /// Prompt tokens evaluated (llama.cpp `prompt_n`; OpenAI `prompt_tokens` minus the cached ones).
    pub prompt_tokens: u64,
    pub cached_tokens: u64,
    pub predicted_tokens: u64,
    pub ms: f64,
}

pub struct Request<'a> {
    pub system: &'a str,
    pub user: &'a str,
    pub grammar: Option<&'a str>,
    pub seed: u64,
    pub temperature: f32,
    pub n_predict: u32,
    /// Stop strings; empty for grammar-bound output.
    pub stop: &'a [&'a str],
}

impl<'a> Request<'a> {
    /// Greedy decoding, as `ask` and the model router make every deterministic call: seed 1, temperature 0.
    #[must_use]
    pub fn greedy(system: &'a str, user: &'a str, grammar: Option<&'a str>, n_predict: u32, stop: &'a [&'a str])
        -> Self {
        Self { system, user, grammar, seed: 1, temperature: 0.0, n_predict, stop }
    }
}

impl Llm {
    fn with_backend(http: reqwest::Client, backend: Backend, model: String, slots: usize) -> Self {
        Self { http, backend, sem: Arc::new(Semaphore::new(slots)), model, slots, dump: None, no_cache: false,
               usage: Arc::default(), sink: Arc::default() }
    }

    /// An OpenAI-compatible chat endpoint (no request is made here).
    ///
    /// # Errors
    /// An empty model list, or the HTTP client failing to build.
    pub fn openai(cfg: OpenAiConfig) -> Result<Self> {
        if cfg.models.is_empty() {
            bail!("the OpenAI-compatible backend needs at least one model id");
        }
        let http = reqwest::Client::builder().timeout(Duration::from_secs(cfg.timeout_secs)).build()?;
        let (model, slots) = (cfg.models[0].clone(), cfg.concurrency.max(1));
        Ok(Self::with_backend(http, Backend::OpenAi(Arc::new(openai::Client::new(cfg))), model, slots))
    }

    /// The model ids of the OpenAI-compatible backend, in failover order (empty for llama.cpp).
    #[must_use]
    pub fn model_ids(&self) -> Vec<String> {
        match &self.backend {
            Backend::OpenAi(c) => c.cfg.models.clone(),
            Backend::LlamaCpp { .. } => Vec::new(),
        }
    }

    /// The model id the OpenAI-compatible backend tries first (the last one that answered); None for llama.cpp.
    #[must_use]
    pub fn preferred_model(&self) -> Option<String> {
        match &self.backend {
            Backend::OpenAi(c) => Some(c.preferred()),
            Backend::LlamaCpp { .. } => None,
        }
    }

    /// Try `id` first (the canary's first passing model). False when `id` is not in the list.
    pub fn prefer_model(&self, id: &str) -> bool {
        match &self.backend {
            Backend::OpenAi(c) => c.prefer(id),
            Backend::LlamaCpp { .. } => false,
        }
    }

    /// The llama.cpp server's base URL (None for the OpenAI-compatible backend).
    #[must_use]
    pub fn llama_base(&self) -> Option<&str> {
        match &self.backend {
            Backend::LlamaCpp { base } => Some(base),
            Backend::OpenAi(_) => None,
        }
    }

    /// Append every request's rendered prompt, `prompt_n`, `cache_n` and output to `path` as JSON lines
    /// (`ask --dump-prompt`). The requests themselves are unchanged.
    ///
    /// # Errors
    /// The file cannot be opened.
    pub fn with_dump(mut self, path: &std::path::Path) -> Result<Self> {
        let f = std::fs::OpenOptions::new().create(true).append(true).open(path)
            .with_context(|| format!("opening {}", path.display()))?;
        self.dump = Some(Arc::new(Mutex::new(f)));
        Ok(self)
    }

    /// Send `cache_prompt: false` (`ask --no-cache-prompt`). llama-server's output depends on how much of a prompt it
    /// reuses from the KV cache or the `-cram` host cache: a reused prefix changes the batch split of the rest, and
    /// with it the logits in the last bits, so the same prompt can decode to other words.
    #[must_use]
    pub fn without_prompt_cache(mut self) -> Self {
        self.no_cache = true;
        self
    }

    /// Stream every later answer call's text to `sink` (None stops streaming). Shared by every clone of this client.
    pub fn set_stream(&self, sink: Option<UnboundedSender<StreamEvent>>) {
        if let Ok(mut s) = self.sink.lock() {
            *s = sink;
        }
    }

    /// The running totals of calls, tokens and JSON validations.
    #[must_use]
    pub fn usage(&self) -> UsageSnapshot {
        let u = &self.usage;
        UsageSnapshot { calls: u.calls.load(Ordering::Relaxed), tokens_in: u.tokens_in.load(Ordering::Relaxed),
                        tokens_out: u.tokens_out.load(Ordering::Relaxed),
                        json_calls: u.json_calls.load(Ordering::Relaxed),
                        json_retries: u.json_retries.load(Ordering::Relaxed),
                        json_failures: u.json_failures.load(Ordering::Relaxed) }
    }

    /// # Errors
    /// Server errors after retries; on the OpenAI-compatible backend also [`JsonInvalid`] for a grammar-bound call.
    pub async fn complete(&self, req: &Request<'_>) -> Result<Completion> {
        let _permit = self.sem.acquire().await?;
        let started = std::time::Instant::now();
        let sink = if req.grammar.is_none() && req.stop.is_empty() {
            self.sink.lock().ok().and_then(|s| s.clone())
        } else {
            None
        };
        let mut c = match &self.backend {
            Backend::LlamaCpp { base } => self.complete_llama(base, req, sink.as_ref()).await?,
            Backend::OpenAi(client) => match req.grammar {
                Some(g) => self.complete_json(client, req, g).await?,
                None => {
                    let c = client.chat(&self.http, req, req.system, req.user, self.no_cache, sink.as_ref()).await?;
                    self.count(&c);
                    c
                }
            },
        };
        c.ms = started.elapsed().as_secs_f64() * 1000.0;
        Ok(c)
    }

    fn count(&self, c: &Completion) {
        self.usage.calls.fetch_add(1, Ordering::Relaxed);
        self.usage.tokens_in.fetch_add(c.prompt_tokens + c.cached_tokens, Ordering::Relaxed);
        self.usage.tokens_out.fetch_add(c.predicted_tokens, Ordering::Relaxed);
    }

    /// A binary judgment under `verdict.gbnf`, at temperature 0.
    ///
    /// # Errors
    /// Server errors, or output that is not the grammar's shape.
    pub async fn verdict(&self, system: &str, user: &str, grammar: &str) -> Result<bool> {
        let c = self
            .complete(&Request {
                system,
                user,
                grammar: Some(grammar),
                seed: 0,
                temperature: 0.0,
                n_predict: 48,
                stop: &[],
            })
            .await?;
        #[derive(Deserialize)]
        struct V {
            verdict: bool,
        }
        let v: V = serde_json::from_str(c.content.trim())
            .with_context(|| format!("verdict did not parse: {:?}", c.content))?;
        Ok(v.verdict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_keys_and_json_validation() {
        let form = include_str!("../../prompts/form.gbnf");
        assert_eq!(grammar_keys(form), ["form", "evidence"]);
        let router = include_str!("../../prompts/router.gbnf");
        let rk = grammar_keys(router);
        assert!(rk.contains(&"tool".to_owned()) && rk.contains(&"args".to_owned()) && rk.contains(&"event".to_owned()));
        assert!(!rk.contains(&"reading_guide".to_owned()));
        let fk = grammar_keys(&form.replace("evidence", "x"));
        assert_eq!(valid_json("```json\n{\"form\": \"fact\"}\n```", &fk).as_deref(), Some("{\"form\": \"fact\"}"));
        assert_eq!(valid_json("{\"form\": \"yes_no\", \"other\": 1}", &fk), None);
        assert_eq!(valid_json("fact", &fk), None);
        assert_eq!(valid_json("{\"tool\": \"retrieve\", \"args\": {\"event\": \"x\"}}", &rk).as_deref(),
                   Some("{\"tool\": \"retrieve\", \"args\": {\"event\": \"x\"}}"));
        assert_eq!(valid_json("{\"tool\": \"retrieve\", \"args\": {\"nope\": 1}}", &rk), None);
    }
}
