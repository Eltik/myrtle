//! The OpenAI-compatible chat backend (2026-10-16, design/trevor-integration-plan-v3.md phase B step 5): Cloudflare
//! Workers AI's `/ai/v1/chat/completions`, or any server with that wire shape (Homebrew `llama-server`'s own
//! `/v1/chat/completions`, which is how it is tested).
//!
//! - A bearer token and an ordered list of model ids. A call tries the preferred id first (the last one that answered,
//!   or the canary's first passing one) and fails over to the next on a missing model (404, or a 400/422 whose body says
//!   the model is not found) or on a server or transport error after one retry. Any other 4xx (a bad token, a rate
//!   limit, a spent budget) fails the call at once: another model id would not help.
//! - Thinking off per request, by default with `chat_template_kwargs: {"enable_thinking": false}` (what llama-server and
//!   vLLM-style servers read); `TREVOR_CF_THINKING=effort` sends `reasoning_effort: "none"` instead and `=omit` sends
//!   neither. Cloudflare's own page for gemma-4-26b-a4b-it was not read (no network in this build), so the default is
//!   llama-server's and the switch is the fallback.
//! - `stream: true` (with `stream_options.include_usage`) when a sink is given; `finish_reason` becomes `stop_type`
//!   (`stop` -> `eos`, `length` -> `limit`) and `usage` the token counts.
//!
//! Configuration (`OpenAiConfig::cloudflare_from_env`): `TREVOR_CF_MODELS` (comma-separated ids, required),
//! `TREVOR_CF_ACCOUNT_ID` and `TREVOR_CF_API_TOKEN`, or `TREVOR_CF_BASE_URL` (a full chat-completions URL, for tests).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;

use super::{Completion, Request, StreamEvent};

/// How a request turns the model's thinking off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thinking {
    /// `chat_template_kwargs: {"enable_thinking": false}`.
    TemplateKwargs,
    /// `reasoning_effort: "none"`.
    ReasoningEffort,
    /// Neither field.
    Omit,
}

impl Thinking {
    /// `TREVOR_CF_THINKING`: `effort`, `omit`, anything else the template kwargs.
    #[must_use]
    pub fn from_env() -> Self {
        match crate::env::var("TREVOR_CF_THINKING").as_deref() {
            Some("effort") => Self::ReasoningEffort,
            Some("omit") => Self::Omit,
            _ => Self::TemplateKwargs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    /// The full chat-completions URL.
    pub url: String,
    pub token: Option<String>,
    /// Model ids in failover order.
    pub models: Vec<String>,
    pub thinking: Thinking,
    pub timeout_secs: u64,
    /// Requests in flight at once.
    pub concurrency: usize,
}

/// Cloudflare's OpenAI-compatible endpoint for an account.
#[must_use]
pub fn cloudflare_url(account: &str) -> String {
    format!("https://api.cloudflare.com/client/v4/accounts/{account}/ai/v1/chat/completions")
}

impl OpenAiConfig {
    /// The Cloudflare backend from the environment, or None when `TREVOR_CF_MODELS` is unset or empty, or neither
    /// `TREVOR_CF_BASE_URL` nor `TREVOR_CF_ACCOUNT_ID` is set.
    #[must_use]
    pub fn cloudflare_from_env() -> Option<Self> {
        let models: Vec<String> = crate::env::var("TREVOR_CF_MODELS")?.split(',').map(str::trim)
            .filter(|m| !m.is_empty()).map(str::to_owned).collect();
        if models.is_empty() {
            return None;
        }
        let url = match crate::env::non_empty("TREVOR_CF_BASE_URL") {
            Some(u) => u,
            None => cloudflare_url(&crate::env::non_empty("TREVOR_CF_ACCOUNT_ID")?),
        };
        let token = crate::env::non_empty("TREVOR_CF_API_TOKEN");
        Some(Self { url, token, models, thinking: Thinking::from_env(), timeout_secs: 120, concurrency: 1 })
    }

    /// A local llama-server's own chat endpoint (`ask --backend openai` without the Cloudflare variables).
    #[must_use]
    pub fn local(server_base: &str, models: Vec<String>) -> Self {
        Self { url: format!("{}/v1/chat/completions", server_base.trim_end_matches('/')), token: None, models,
               thinking: Thinking::from_env(), timeout_secs: 600, concurrency: 1 }
    }
}

pub(crate) struct Client {
    pub(crate) cfg: OpenAiConfig,
    preferred: AtomicUsize,
}

/// Whether a 4xx response says the model id does not exist.
fn model_missing(status: u16, body: &str) -> bool {
    let b = body.to_lowercase();
    status == 404 || (matches!(status, 400 | 422) && b.contains("model")
        && ["not found", "no such", "does not exist", "unknown", "invalid model", "not supported"].iter().any(|w| b.contains(w)))
}

/// `finish_reason` as llama.cpp's `stop_type`.
fn stop_type(finish: &str) -> String {
    match finish {
        "stop" => "eos".into(),
        "length" => "limit".into(),
        other => other.into(),
    }
}

/// The token counts of an OpenAI `usage` object: (prompt tokens not cached, cached, completion).
fn usage_of(u: &serde_json::Value) -> (u64, u64, u64) {
    let prompt = u["prompt_tokens"].as_u64().unwrap_or(0);
    let cached = u["prompt_tokens_details"]["cached_tokens"].as_u64().unwrap_or(0).min(prompt);
    (prompt - cached, cached, u["completion_tokens"].as_u64().unwrap_or(0))
}

enum Attempt {
    Done(Completion),
    /// Try the same model once more, then the next.
    Retry(anyhow::Error),
    /// Go to the next model now.
    Next(anyhow::Error),
}

impl Client {
    pub(crate) fn new(cfg: OpenAiConfig) -> Self {
        Self { cfg, preferred: AtomicUsize::new(0) }
    }

    pub(crate) fn preferred(&self) -> String {
        self.cfg.models[self.preferred.load(Ordering::Relaxed) % self.cfg.models.len()].clone()
    }

    pub(crate) fn prefer(&self, id: &str) -> bool {
        match self.cfg.models.iter().position(|m| m == id) {
            Some(i) => {
                self.preferred.store(i, Ordering::Relaxed);
                true
            }
            None => false,
        }
    }

    fn body(&self, model: &str, req: &Request<'_>, system: &str, user: &str, no_cache: bool, stream: bool) -> serde_json::Value {
        let mut body = json!({
            "model": model,
            "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
            "temperature": req.temperature,
            "top_p": 0.95,
            "seed": req.seed,
            "max_tokens": req.n_predict,
            "stream": stream,
        });
        if !req.stop.is_empty() {
            body["stop"] = json!(req.stop);
        }
        if stream {
            body["stream_options"] = json!({"include_usage": true});
        }
        match self.cfg.thinking {
            Thinking::TemplateKwargs => body["chat_template_kwargs"] = json!({"enable_thinking": false}),
            Thinking::ReasoningEffort => body["reasoning_effort"] = json!("none"),
            Thinking::Omit => {}
        }
        if no_cache {
            body["cache_prompt"] = json!(false);
        }
        body
    }

    /// One chat call (`system` and `user` replace the request's own, for the JSON prompts), with failover.
    pub(crate) async fn chat(&self, http: &reqwest::Client, req: &Request<'_>, system: &str, user: &str, no_cache: bool,
                             sink: Option<&UnboundedSender<StreamEvent>>) -> Result<Completion> {
        let n = self.cfg.models.len();
        let first = self.preferred.load(Ordering::Relaxed) % n;
        let mut errors = Vec::new();
        for idx in (0..n).map(|i| (first + i) % n) {
            let model = &self.cfg.models[idx];
            let body = self.body(model, req, system, user, no_cache, sink.is_some());
            for attempt in 0..2 {
                if attempt > 0 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                match self.attempt(http, &body, sink).await? {
                    Attempt::Done(c) => {
                        self.preferred.store(idx, Ordering::Relaxed);
                        return Ok(c);
                    }
                    Attempt::Retry(e) => errors.push(format!("{model}: {e}")),
                    Attempt::Next(e) => {
                        errors.push(format!("{model}: {e}"));
                        break;
                    }
                }
            }
        }
        bail!("every model id failed: {}", errors.join("; "))
    }

    async fn attempt(&self, http: &reqwest::Client, body: &serde_json::Value, sink: Option<&UnboundedSender<StreamEvent>>) -> Result<Attempt> {
        let mut rb = http.post(&self.cfg.url).json(body);
        if let Some(t) = &self.cfg.token {
            rb = rb.bearer_auth(t);
        }
        let r = match rb.send().await {
            Ok(r) => r,
            Err(e) => return Ok(Attempt::Retry(anyhow!("{e}"))),
        };
        let status = r.status().as_u16();
        if r.status().is_server_error() {
            return Ok(Attempt::Retry(anyhow!("HTTP {status}: {}", r.text().await.unwrap_or_default())));
        }
        if !r.status().is_success() {
            let text = r.text().await.unwrap_or_default();
            if model_missing(status, &text) {
                return Ok(Attempt::Next(anyhow!("HTTP {status}: {text}")));
            }
            bail!("chat completions: HTTP {status}: {text}");
        }
        let Some(sink) = sink else {
            let v: serde_json::Value = match r.json().await {
                Ok(v) => v,
                Err(e) => return Ok(Attempt::Retry(anyhow!("unreadable response: {e}"))),
            };
            let ch = &v["choices"][0];
            let (prompt, cached, out) = usage_of(&v["usage"]);
            return Ok(Attempt::Done(Completion {
                content: ch["message"]["content"].as_str().unwrap_or_default().to_owned(),
                stop_type: stop_type(ch["finish_reason"].as_str().unwrap_or_default()),
                prompt_tokens: prompt, cached_tokens: cached, predicted_tokens: out, ms: 0.0,
            }));
        };
        let _ = sink.send(StreamEvent::Start);
        let mut lines = SseLines::new(r);
        let (mut content, mut finish, mut usage) = (String::new(), None::<String>, serde_json::Value::Null);
        loop {
            match lines.next_data().await {
                Ok(Some(data)) => {
                    let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) else { continue };
                    if v.get("error").is_some_and(|e| !e.is_null()) {
                        return Ok(Attempt::Retry(anyhow!("stream error: {}", v["error"])));
                    }
                    let ch = &v["choices"][0];
                    if let Some(p) = ch["delta"]["content"].as_str().filter(|p| !p.is_empty()) {
                        content.push_str(p);
                        let _ = sink.send(StreamEvent::Piece(p.to_owned()));
                    }
                    if let Some(f) = ch["finish_reason"].as_str() {
                        finish = Some(f.to_owned());
                    }
                    if v["usage"].is_object() {
                        usage = v["usage"].clone();
                    }
                }
                Ok(None) => break,
                Err(e) => return Ok(Attempt::Retry(anyhow!("stream cut: {e}"))),
            }
        }
        let Some(finish) = finish else { return Ok(Attempt::Retry(anyhow!("the stream ended without a finish_reason"))) };
        let (prompt, cached, out) = usage_of(&usage);
        Ok(Attempt::Done(Completion { content, stop_type: stop_type(&finish), prompt_tokens: prompt, cached_tokens: cached,
                                      predicted_tokens: out, ms: 0.0 }))
    }
}

/// Server-sent events of one response: the `data:` payloads, None at `[DONE]` or the end of the body.
pub(crate) struct SseLines {
    resp: reqwest::Response,
    buf: Vec<u8>,
    ended: bool,
}

impl SseLines {
    pub(crate) fn new(resp: reqwest::Response) -> Self {
        Self { resp, buf: Vec::new(), ended: false }
    }

    pub(crate) async fn next_data(&mut self) -> Result<Option<String>> {
        loop {
            while let Some(i) = self.buf.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = self.buf.drain(..=i).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim_end_matches(['\n', '\r']);
                if let Some(d) = line.strip_prefix("data:") {
                    let d = d.strip_prefix(' ').unwrap_or(d);
                    if d == "[DONE]" {
                        return Ok(None);
                    }
                    return Ok(Some(d.to_owned()));
                }
            }
            if self.ended {
                if self.buf.is_empty() {
                    return Ok(None);
                }
                self.buf.push(b'\n');
                continue;
            }
            match self.resp.chunk().await? {
                Some(c) => self.buf.extend_from_slice(&c),
                None => self.ended = true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{Llm, Request};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// A one-route HTTP server: each request's model id picks a canned (status, content type, body).
    async fn mock(routes: Vec<(&'static str, u16, &'static str, String)>) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        let h = tokio::spawn(async move {
            let mut seen = Vec::new();
            for route in &routes {
                let (mut s, _) = l.accept().await.unwrap();
                let mut buf = Vec::new();
                let mut tmp = [0u8; 8192];
                loop {
                    let n = s.read(&mut tmp).await.unwrap();
                    buf.extend_from_slice(&tmp[..n]);
                    let text = String::from_utf8_lossy(&buf).to_string();
                    if let Some(i) = text.find("\r\n\r\n") {
                        let len: usize = text[..i].lines().find_map(|l| l.to_lowercase().strip_prefix("content-length:").map(|v| v.trim().parse().unwrap())).unwrap_or(0);
                        if buf.len() >= i + 4 + len {
                            break;
                        }
                    }
                }
                let text = String::from_utf8_lossy(&buf).to_string();
                let body = &text[text.find("\r\n\r\n").unwrap() + 4..];
                let v: serde_json::Value = serde_json::from_str(body).unwrap();
                let model = v["model"].as_str().unwrap().to_owned();
                let auth = text.lines().find(|l| l.to_lowercase().starts_with("authorization")).unwrap_or("").to_owned();
                seen.push(format!("{model} {} {auth}", v["chat_template_kwargs"]));
                let (want, status, ctype, resp) = route;
                assert_eq!(*want, model);
                let head = format!("HTTP/1.1 {status} X\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", resp.len());
                s.write_all(head.as_bytes()).await.unwrap();
                s.write_all(resp.as_bytes()).await.unwrap();
                s.shutdown().await.unwrap();
            }
            seen
        });
        (format!("http://{addr}/v1/chat/completions"), h)
    }

    #[tokio::test]
    async fn failover_streaming_finish_reason_and_usage() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"Tal\"},\"finish_reason\":null}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"content\":\"ulah\"},\"finish_reason\":\"length\"}]}\n\n\
                   data: {\"choices\":[],\"usage\":{\"prompt_tokens\":12,\"completion_tokens\":2}}\n\ndata: [DONE]\n\n".to_owned();
        let (url, h) = mock(vec![
            ("gone", 404, "application/json", "{\"errors\":[{\"message\":\"No such model\"}]}".into()),
            ("ok", 200, "text/event-stream", sse),
            ("ok", 200, "application/json", "{\"choices\":[{\"message\":{\"content\":\"Talulah.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":10,\"completion_tokens\":3,\"prompt_tokens_details\":{\"cached_tokens\":4}}}".into()),
        ]).await;
        let cfg = OpenAiConfig { url, token: Some("tok".into()), models: vec!["gone".into(), "ok".into()],
                                 thinking: Thinking::TemplateKwargs, timeout_secs: 10, concurrency: 1 };
        let llm = Llm::openai(cfg).unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        llm.set_stream(Some(tx));
        let c = llm.complete(&Request::greedy("sys", "Who leads Reunion?", None, 8, &[])).await.unwrap();
        assert_eq!((c.content.as_str(), c.stop_type.as_str(), c.prompt_tokens, c.predicted_tokens), ("Talulah", "limit", 12, 2));
        let mut got = Vec::new();
        while let Ok(e) = rx.try_recv() {
            got.push(e);
        }
        assert_eq!(got, [StreamEvent::Start, StreamEvent::Piece("Tal".into()), StreamEvent::Piece("ulah".into())]);
        // The failed id is skipped from now on; a non-streaming call reads finish_reason and the cached tokens.
        assert_eq!(llm.preferred_model().as_deref(), Some("ok"));
        llm.set_stream(None);
        let c = llm.complete(&Request::greedy("sys", "q", None, 8, &[])).await.unwrap();
        assert_eq!((c.content.as_str(), c.stop_type.as_str(), c.prompt_tokens, c.cached_tokens), ("Talulah.", "eos", 6, 4));
        let u = llm.usage();
        assert_eq!((u.calls, u.tokens_in, u.tokens_out), (2, 22, 5));
        let seen = h.await.unwrap();
        assert_eq!(seen[0], "gone {\"enable_thinking\":false} authorization: Bearer tok");
        assert!(seen[1].starts_with("ok "));
    }

    #[tokio::test]
    async fn grammar_calls_become_validated_json_prompts() {
        let (url, h) = mock(vec![
            ("m", 200, "application/json", "{\"choices\":[{\"message\":{\"content\":\"It is a fact question.\"},\"finish_reason\":\"stop\"}]}".into()),
            ("m", 200, "application/json", "{\"choices\":[{\"message\":{\"content\":\"```json\\n{\\\"form\\\": \\\"fact\\\"}\\n```\"},\"finish_reason\":\"stop\"}]}".into()),
            ("m", 200, "application/json", "{\"choices\":[{\"message\":{\"content\":\"no\"},\"finish_reason\":\"stop\"}]}".into()),
            ("m", 200, "application/json", "{\"choices\":[{\"message\":{\"content\":\"{\\\"tool\\\": 1}\"},\"finish_reason\":\"stop\"}]}".into()),
        ]).await;
        let cfg = OpenAiConfig { url, token: None, models: vec!["m".into()], thinking: Thinking::Omit, timeout_secs: 10, concurrency: 1 };
        let llm = Llm::openai(cfg).unwrap();
        let g = include_str!("../../prompts/form.gbnf");
        let c = llm.complete(&Request::greedy("sys", "Q: x", Some(g), 80, &[])).await.unwrap();
        assert_eq!(c.content, "{\"form\": \"fact\"}");
        let e = llm.complete(&Request::greedy("sys", "Q: y", Some(g), 80, &[])).await.unwrap_err();
        assert!(crate::llm::is_json_invalid(&e));
        let u = llm.usage();
        assert_eq!((u.json_calls, u.json_retries, u.json_failures, u.calls), (2, 2, 1, 4));
        assert_eq!(h.await.unwrap()[0], "m null ");
    }

    #[test]
    fn missing_model_and_stop_type() {
        assert!(model_missing(404, ""));
        assert!(model_missing(400, "{\"errors\":[{\"message\":\"Model not found: @cf/x\"}]}"));
        assert!(!model_missing(400, "bad request"));
        assert!(!model_missing(429, "model rate limited"));
        assert_eq!((stop_type("stop"), stop_type("length")), ("eos".to_owned(), "limit".to_owned()));
    }
}
