//! JSON on a backend without grammars: a grammar-bound call becomes a prompt asking for one JSON object, checked
//! against the keys the grammar names, asked once more on failure, else [`JsonInvalid`].

use std::sync::atomic::Ordering;

use anyhow::Result;

use super::{Completion, Llm, Request, openai};

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

impl Llm {
    /// A grammar-bound call on the OpenAI-compatible backend: a plain prompt for one JSON object, validated against the
    /// keys the grammar names, asked once more when it does not validate.
    pub(super) async fn complete_json(
        &self,
        client: &openai::Client,
        req: &Request<'_>,
        grammar: &str,
    ) -> Result<Completion> {
        let system = format!("{}{JSON_ONLY}", req.system);
        let keys = grammar_keys(grammar);
        let plain = Request {
            system: &system,
            user: req.user,
            grammar: None,
            seed: req.seed,
            temperature: req.temperature,
            n_predict: req.n_predict.max(64) * 2,
            stop: &[],
        };
        self.usage.json_calls.fetch_add(1, Ordering::Relaxed);
        let mut c = client
            .chat(&self.http, &plain, &system, req.user, self.no_cache, None)
            .await?;
        self.count(&c);
        if let Some(j) = valid_json(&c.content, &keys) {
            c.content = j;
            return Ok(c);
        }
        self.usage.json_retries.fetch_add(1, Ordering::Relaxed);
        let user = format!("{}{JSON_AGAIN}", req.user);
        let mut c2 = client
            .chat(&self.http, &plain, &system, &user, self.no_cache, None)
            .await?;
        self.count(&c2);
        if let Some(j) = valid_json(&c2.content, &keys) {
            c2.content = j;
            return Ok(c2);
        }
        self.usage.json_failures.fetch_add(1, Ordering::Relaxed);
        Err(JsonInvalid(c2.content).into())
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
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
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
            serde_json::Value::Object(m) => m
                .iter()
                .all(|(k, x)| (keys.is_empty() || keys.iter().any(|y| y == k)) && ok(x, keys)),
            serde_json::Value::Array(a) => a.iter().all(|x| ok(x, keys)),
            _ => true,
        }
    }
    let (start, end) = (text.find('{')?, text.rfind('}')?);
    let body = text.get(start..=end)?;
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    (v.is_object() && ok(&v, keys)).then(|| body.to_owned())
}
