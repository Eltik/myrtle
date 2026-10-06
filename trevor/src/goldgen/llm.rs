//! A minimal client for a local `llama-server`.
//!
//! Chat formatting goes through the server's own `/apply-template`, so the
//! prompt is rendered by the model's Jinja template and nothing here knows
//! Gemma's or Qwen's turn markers. Generation goes through `/completion`,
//! the only endpoint that reports `stop_type`, which is how a grammar-bound
//! output cut off at `n_predict` is told apart from a finished one.
//!
//! Server behaviours this client is written around (llama.cpp server README):
//! requests queue rather than fail when every slot is busy, so concurrency
//! is bounded here to the server's slot count; and one failed decode answers
//! 500 to every in-flight request, so 5xx and transport errors are retried.

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct Llm {
    http: reqwest::Client,
    base: String,
    sem: Arc<Semaphore>,
    /// File name of the loaded model, from `/props`.
    pub model: String,
    pub slots: usize,
    /// `with_dump`: every rendered prompt, its token counts and the output, one JSON line per request.
    dump: Option<Arc<std::sync::Mutex<std::fs::File>>>,
    /// `without_prompt_cache`: send `cache_prompt: false`, so no request reuses another's KV cache.
    no_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub content: String,
    /// `eos`, `word` or `limit`.
    pub stop_type: String,
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

impl Llm {
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
        Ok(Self {
            http,
            base: base.trim_end_matches('/').to_owned(),
            sem: Arc::new(Semaphore::new(slots)),
            model,
            slots,
            dump: None,
            no_cache: false,
        })
    }

    /// Append every request's rendered prompt, `prompt_n`, `cache_n` and output to `path` as JSON lines
    /// (`ask --dump-prompt`). The requests themselves are unchanged.
    ///
    /// # Errors
    /// The file cannot be opened.
    pub fn with_dump(mut self, path: &std::path::Path) -> Result<Self> {
        let f = std::fs::OpenOptions::new().create(true).append(true).open(path)
            .with_context(|| format!("opening {}", path.display()))?;
        self.dump = Some(Arc::new(std::sync::Mutex::new(f)));
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

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let mut last = None;
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << attempt)).await;
            }
            match self
                .http
                .post(format!("{}{path}", self.base))
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

    /// The prompt exactly as the model's chat template renders it.
    ///
    /// # Errors
    /// Server errors.
    pub async fn render(&self, system: &str, user: &str) -> Result<String> {
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
        let v = self.post("/apply-template", &body).await?;
        Ok(v["prompt"]
            .as_str()
            .context("/apply-template returned no prompt")?
            .to_owned())
    }

    /// # Errors
    /// Server errors after retries.
    pub async fn complete(&self, req: &Request<'_>) -> Result<Completion> {
        let _permit = self.sem.acquire().await?;
        let started = std::time::Instant::now();
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
        let v = self.post("/completion", &body).await?;
        let t = &v["timings"];
        if let Some(d) = &self.dump {
            use std::io::Write as _;
            let line = json!({"prompt": prompt, "promptN": t["prompt_n"], "cacheN": t["cache_n"],
                              "content": v["content"], "seed": req.seed, "temperature": req.temperature});
            if let Ok(mut f) = d.lock() {
                writeln!(f, "{line}")?;
            }
        }
        Ok(Completion {
            content: v["content"].as_str().unwrap_or_default().to_owned(),
            stop_type: v["stop_type"].as_str().unwrap_or_default().to_owned(),
            prompt_tokens: t["prompt_n"].as_u64().unwrap_or(0),
            cached_tokens: t["cache_n"]
                .as_u64()
                .or_else(|| v["tokens_cached"].as_u64())
                .unwrap_or(0),
            predicted_tokens: t["predicted_n"].as_u64().unwrap_or(0),
            ms: started.elapsed().as_secs_f64() * 1000.0,
        })
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
