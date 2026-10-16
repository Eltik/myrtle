//! The model client: a local `llama-server` (the default backend, unchanged) or an OpenAI-compatible chat endpoint
//! (Cloudflare Workers AI, 2026-10-16; [`openai`]).
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
//! Streaming (2026-10-16): when a stream sink is set ([`Llm::set_stream`], the `ask --worker` job loop), a call with no
//! grammar and no stop strings (an answer call) streams (`stream: true`) and sends its text pieces to the sink as they
//! arrive; the returned [`Completion`] is the same. Without a sink every request is sent exactly as before.
//!
//! The OpenAI-compatible backend has no grammar: a grammar-bound call (router, question form, flags) is sent as a plain
//! prompt asking for one JSON object, validated (it must parse, and its keys must be keys the grammar names), asked once
//! more on failure, and then fails with [`JsonInvalid`], on which `ask` falls back to the keyword router or the fact
//! form.

pub mod openai;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Semaphore;
use tokio::sync::mpsc::UnboundedSender;

pub use openai::{OpenAiConfig, Thinking};

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
    pub fn greedy(system: &'a str, user: &'a str, grammar: Option<&'a str>, n_predict: u32, stop: &'a [&'a str]) -> Self {
        Self { system, user, grammar, seed: 1, temperature: 0.0, n_predict, stop }
    }
}

/// A grammar-bound call on a backend without grammars returned no valid JSON object twice.
#[derive(Debug)]
pub struct JsonInvalid(pub String);

impl std::fmt::Display for JsonInvalid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "the model returned no valid JSON object twice: {:?}", self.0)
    }
}

impl std::error::Error for JsonInvalid {}

/// Whether `e` is (or wraps) a [`JsonInvalid`].
#[must_use]
pub fn is_json_invalid(e: &anyhow::Error) -> bool {
    e.chain().any(|c| c.downcast_ref::<JsonInvalid>().is_some())
}

/// Running totals over every call of this client (and its clones).
#[derive(Debug, Default)]
struct Usage {
    calls: AtomicU64,
    tokens_in: AtomicU64,
    tokens_out: AtomicU64,
    json_calls: AtomicU64,
    json_retries: AtomicU64,
    json_failures: AtomicU64,
}

/// A copy of the totals; subtract two to get one job's share.
#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub calls: u64,
    /// Every prompt token, cached or not.
    pub tokens_in: u64,
    pub tokens_out: u64,
    /// Grammar-bound calls answered as plain JSON prompts (OpenAI-compatible backend only).
    pub json_calls: u64,
    /// Of those, the ones whose first reply did not validate.
    pub json_retries: u64,
    /// And the ones whose second reply did not validate either.
    pub json_failures: u64,
}

impl std::ops::Sub for UsageSnapshot {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self { calls: self.calls - o.calls, tokens_in: self.tokens_in - o.tokens_in, tokens_out: self.tokens_out - o.tokens_out,
               json_calls: self.json_calls - o.json_calls, json_retries: self.json_retries - o.json_retries,
               json_failures: self.json_failures - o.json_failures }
    }
}

impl Llm {
    fn with_backend(http: reqwest::Client, backend: Backend, model: String, slots: usize) -> Self {
        Self { http, backend, sem: Arc::new(Semaphore::new(slots)), model, slots, dump: None, no_cache: false,
               usage: Arc::default(), sink: Arc::default() }
    }

    /// A llama-server at `base`.
    ///
    /// # Errors
    /// The server is unreachable or `/props` lacks the fields used here.
    pub async fn connect(base: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()?;
        let props: serde_json::Value = http
            .get(format!("{base}/props"))
            .send()
            .await
            .with_context(|| format!("no llama-server at {base}"))?
            .error_for_status()?
            .json()
            .await?;
        let path = props["model_path"]
            .as_str()
            .context("/props has no model_path")?;
        let model = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path)
            .to_owned();
        let slots = usize::try_from(props["total_slots"].as_u64().unwrap_or(1))
            .unwrap_or(1)
            .max(1);
        Ok(Self::with_backend(http, Backend::LlamaCpp { base: base.trim_end_matches('/').to_owned() }, model, slots))
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

    /// Whether this is the OpenAI-compatible backend.
    #[must_use]
    pub fn is_openai(&self) -> bool {
        matches!(self.backend, Backend::OpenAi(_))
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
    /// with it the logits in the last bits, so the same prompt can decode to other words (measured 2026-10-02).
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
                        tokens_out: u.tokens_out.load(Ordering::Relaxed), json_calls: u.json_calls.load(Ordering::Relaxed),
                        json_retries: u.json_retries.load(Ordering::Relaxed), json_failures: u.json_failures.load(Ordering::Relaxed) }
    }

    async fn post(&self, base: &str, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let mut last = None;
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << attempt)).await;
            }
            match self
                .http
                .post(format!("{base}{path}"))
                .json(body)
                .send()
                .await
            {
                Ok(r) if r.status().is_server_error() => {
                    last = Some(anyhow!(
                        "{path}: HTTP {}: {}",
                        r.status(),
                        r.text().await.unwrap_or_default()
                    ));
                }
                Ok(r) if !r.status().is_success() => {
                    let s = r.status();
                    bail!("{path}: HTTP {s}: {}", r.text().await.unwrap_or_default());
                }
                Ok(r) => return Ok(r.json().await?),
                Err(e) => last = Some(anyhow!("{path}: {e}")),
            }
        }
        Err(last.unwrap_or_else(|| anyhow!("{path}: failed")))
    }

    /// `/completion` with `stream: true`: the pieces go to `sink` as they arrive; the result has the shape of the
    /// non-streaming response (`content`, `stop_type`, `timings`, `tokens_cached`). Retried like [`Llm::post`], also
    /// after a cut stream (the sink then sees a new `Start`).
    async fn post_stream(&self, base: &str, body: &serde_json::Value, sink: &UnboundedSender<StreamEvent>) -> Result<serde_json::Value> {
        let mut last = None;
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << attempt)).await;
            }
            let r = match self.http.post(format!("{base}/completion")).json(body).send().await {
                Ok(r) => r,
                Err(e) => { last = Some(anyhow!("/completion: {e}")); continue; }
            };
            if r.status().is_server_error() {
                last = Some(anyhow!("/completion: HTTP {}: {}", r.status(), r.text().await.unwrap_or_default()));
                continue;
            }
            if !r.status().is_success() {
                let s = r.status();
                bail!("/completion: HTTP {s}: {}", r.text().await.unwrap_or_default());
            }
            let _ = sink.send(StreamEvent::Start);
            let mut content = String::new();
            let mut lines = openai::SseLines::new(r);
            let end = loop {
                match lines.next_data().await {
                    Ok(Some(data)) => {
                        let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) else { continue };
                        if v.get("error").is_some() {
                            break Err(anyhow!("/completion stream error: {}", v["error"]));
                        }
                        if let Some(piece) = v["content"].as_str().filter(|p| !p.is_empty()) {
                            content.push_str(piece);
                            let _ = sink.send(StreamEvent::Piece(piece.to_owned()));
                        }
                        if v["stop"].as_bool() == Some(true) {
                            break Ok(v);
                        }
                    }
                    Ok(None) => break Err(anyhow!("/completion: the stream ended before its last event")),
                    Err(e) => break Err(anyhow!("/completion stream: {e}")),
                }
            };
            match end {
                Ok(mut v) => {
                    v["content"] = json!(content);
                    return Ok(v);
                }
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| anyhow!("/completion: failed")))
    }

    /// The prompt exactly as the model's chat template renders it (llama.cpp backend only).
    ///
    /// # Errors
    /// Server errors, or the OpenAI-compatible backend (its endpoint renders the template itself).
    pub async fn render(&self, system: &str, user: &str) -> Result<String> {
        let Backend::LlamaCpp { base } = &self.backend else {
            bail!("the OpenAI-compatible backend renders its own chat template");
        };
        let mut body = json!({"messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ]});
        // Thinking off per request, whatever the server was started with: a server another job started with
        // `--reasoning on` rendered Trevor's prompts with the thinking turn open, and `ask` hung on it (2026-09-30).
        // On a `--reasoning off` server the rendered prompt is byte-identical with or without this (verified for Gemma
        // 4 12B and Qwen3.5 9B). `TREVOR_TEMPLATE_THINK=1` sends the request as before.
        if std::env::var("TREVOR_TEMPLATE_THINK").as_deref() != Ok("1") {
            body["chat_template_kwargs"] = json!({"enable_thinking": false});
        }
        let v = self.post(base, "/apply-template", &body).await?;
        Ok(v["prompt"]
            .as_str()
            .context("/apply-template returned no prompt")?
            .to_owned())
    }

    /// # Errors
    /// Server errors after retries; on the OpenAI-compatible backend also [`JsonInvalid`] for a grammar-bound call.
    pub async fn complete(&self, req: &Request<'_>) -> Result<Completion> {
        let _permit = self.sem.acquire().await?;
        let started = std::time::Instant::now();
        let sink = if req.grammar.is_none() && req.stop.is_empty() { self.sink.lock().ok().and_then(|s| s.clone()) } else { None };
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

    async fn complete_llama(&self, base: &str, req: &Request<'_>, sink: Option<&UnboundedSender<StreamEvent>>) -> Result<Completion> {
        let prompt = self.render(req.system, req.user).await?;
        let mut body = json!({
            "prompt": prompt,
            "seed": req.seed,
            "temperature": req.temperature,
            "top_p": 0.95,
            "n_predict": req.n_predict,
            "cache_prompt": !self.no_cache,
            "stop": req.stop,
        });
        if let Some(g) = req.grammar {
            body["grammar"] = json!(g);
        }
        let v = match sink {
            Some(s) => {
                body["stream"] = json!(true);
                self.post_stream(base, &body, s).await?
            }
            None => self.post(base, "/completion", &body).await?,
        };
        let t = &v["timings"];
        if let Some(d) = &self.dump {
            use std::io::Write as _;
            let line = json!({"prompt": prompt, "promptN": t["prompt_n"], "cacheN": t["cache_n"],
                              "content": v["content"], "seed": req.seed, "temperature": req.temperature});
            if let Ok(mut f) = d.lock() {
                writeln!(f, "{line}")?;
            }
        }
        let c = Completion {
            content: v["content"].as_str().unwrap_or_default().to_owned(),
            stop_type: v["stop_type"].as_str().unwrap_or_default().to_owned(),
            prompt_tokens: t["prompt_n"].as_u64().unwrap_or(0),
            cached_tokens: t["cache_n"]
                .as_u64()
                .or_else(|| v["tokens_cached"].as_u64())
                .unwrap_or(0),
            predicted_tokens: t["predicted_n"].as_u64().unwrap_or(0),
            ms: 0.0,
        };
        self.count(&c);
        Ok(c)
    }

    /// A grammar-bound call on the OpenAI-compatible backend: a plain prompt for one JSON object, validated against the
    /// keys the grammar names, asked once more when it does not validate.
    async fn complete_json(&self, client: &openai::Client, req: &Request<'_>, grammar: &str) -> Result<Completion> {
        let system = format!("{}{JSON_ONLY}", req.system);
        let keys = grammar_keys(grammar);
        let plain = Request { system: &system, user: req.user, grammar: None, seed: req.seed, temperature: req.temperature,
                              n_predict: req.n_predict.max(64) * 2, stop: &[] };
        self.usage.json_calls.fetch_add(1, Ordering::Relaxed);
        let mut c = client.chat(&self.http, &plain, &system, req.user, self.no_cache, None).await?;
        self.count(&c);
        if let Some(j) = valid_json(&c.content, &keys) {
            c.content = j;
            return Ok(c);
        }
        self.usage.json_retries.fetch_add(1, Ordering::Relaxed);
        let user = format!("{}{JSON_AGAIN}", req.user);
        let mut c2 = client.chat(&self.http, &plain, &system, &user, self.no_cache, None).await?;
        self.count(&c2);
        if let Some(j) = valid_json(&c2.content, &keys) {
            c2.content = j;
            return Ok(c2);
        }
        self.usage.json_failures.fetch_add(1, Ordering::Relaxed);
        Err(JsonInvalid(c2.content).into())
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

/// Appended to the system prompt of a grammar-bound call on a backend without grammars.
const JSON_ONLY: &str = "\n\nReply with one JSON object only: no other text, no code fence.";
/// Appended to the question when the first reply did not validate.
const JSON_AGAIN: &str = "\n\nYour previous reply was not one valid JSON object. Reply with the JSON object only.";

/// The object keys a GBNF grammar names: every `\"name\":` in its literals.
#[must_use]
pub fn grammar_keys(grammar: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut rest = grammar;
    while let Some(i) = rest.find("\\\"") {
        rest = &rest[i + 2..];
        let end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        if end > 0 && rest[end..].starts_with("\\\":") && !keys.iter().any(|k| k == &rest[..end]) {
            keys.push(rest[..end].to_owned());
        }
    }
    keys
}

/// The JSON object in `text` (a code fence or prose around it is dropped) when it parses and every key, at any depth,
/// is one of `keys` (any key when `keys` is empty).
#[must_use]
pub fn valid_json(text: &str, keys: &[String]) -> Option<String> {
    fn ok(v: &serde_json::Value, keys: &[String]) -> bool {
        match v {
            serde_json::Value::Object(m) => m.iter().all(|(k, x)| (keys.is_empty() || keys.iter().any(|y| y == k)) && ok(x, keys)),
            serde_json::Value::Array(a) => a.iter().all(|x| ok(x, keys)),
            _ => true,
        }
    }
    let (start, end) = (text.find('{')?, text.rfind('}')?);
    let body = text.get(start..=end)?;
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    (v.is_object() && ok(&v, keys)).then(|| body.to_owned())
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
