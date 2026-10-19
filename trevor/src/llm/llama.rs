//! The llama.cpp backend: connecting to a llama-server, rendering a prompt with its chat template, and
//! `/completion` calls, streamed when a sink is set.

use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use super::{Backend, Completion, Llm, Request, StreamEvent, openai};

impl Llm {
    /// A llama-server at `base`.
    ///
    /// # Errors
    /// The server is unreachable or `/props` lacks the fields used here.
    pub async fn connect(base: &str) -> Result<Self> {
        let http = reqwest::Client::builder().timeout(Duration::from_secs(600)).build()?;
        let props: serde_json::Value = http
            .get(format!("{base}/props"))
            .send()
            .await
            .with_context(|| format!("no llama-server at {base}"))?
            .error_for_status()?
            .json()
            .await?;
        let path = props["model_path"].as_str().context("/props has no model_path")?;
        let model = std::path::Path::new(path)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(path)
            .to_owned();
        let slots = usize::try_from(props["total_slots"].as_u64().unwrap_or(1))
            .unwrap_or(1)
            .max(1);
        Ok(Self::with_backend(
            http,
            Backend::LlamaCpp {
                base: base.trim_end_matches('/').to_owned(),
            },
            model,
            slots,
        ))
    }

    async fn post(&self, base: &str, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let mut last = None;
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << attempt)).await;
            }
            match self.http.post(format!("{base}{path}")).json(body).send().await {
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
    async fn post_stream(
        &self,
        base: &str,
        body: &serde_json::Value,
        sink: &UnboundedSender<StreamEvent>,
    ) -> Result<serde_json::Value> {
        let mut last = None;
        for attempt in 0..4u32 {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(500 << attempt)).await;
            }
            let r = match self.http.post(format!("{base}/completion")).json(body).send().await {
                Ok(r) => r,
                Err(e) => {
                    last = Some(anyhow!("/completion: {e}"));
                    continue;
                }
            };
            if r.status().is_server_error() {
                last = Some(anyhow!(
                    "/completion: HTTP {}: {}",
                    r.status(),
                    r.text().await.unwrap_or_default()
                ));
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
                        let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) else {
                            continue;
                        };
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
        // `--reasoning on` renders Trevor's prompts with the thinking turn open, and `ask` can hang on it.
        // On a `--reasoning off` server the rendered prompt is byte-identical with or without this (verified for Gemma
        // 4 12B and Qwen3.5 9B). `TREVOR_TEMPLATE_THINK=1` sends the request without it.
        if !crate::env::is("TREVOR_TEMPLATE_THINK", "1") {
            body["chat_template_kwargs"] = json!({"enable_thinking": false});
        }
        let v = self.post(base, "/apply-template", &body).await?;
        Ok(v["prompt"]
            .as_str()
            .context("/apply-template returned no prompt")?
            .to_owned())
    }

    pub(super) async fn complete_llama(
        &self,
        base: &str,
        req: &Request<'_>,
        sink: Option<&UnboundedSender<StreamEvent>>,
    ) -> Result<Completion> {
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
}
