//! Routers that choose one of Trevor's table tools (`crate::tools`) for a question, or retrieval.
//!
//! Keyword routing misfired 13 times on real player phrasing (design/trevor-questions.md sections 8 to 11,
//! design/trevor-retrieval-baseline.md sections 16 to 20), so the choice is now a model's: [`route_model`] asks the
//! answer model (Gemma 4 12B) for `{"tool", "args"}` under a grammar that fixes the tool names and the enumerated
//! argument values. [`Knn`] is the serving router with no LLM: the nearest labelled example questions by embedding
//! vote on the tool, and the arguments come from a dictionary match against the same name tables
//! ([`extract_args`]). Both return [`Route::retrieve`] when unsure, and a tool that cannot resolve its arguments
//! sends the question to retrieval too.

mod args;
#[cfg(feature = "embed-core")]
mod knn;
mod model;
#[cfg(test)]
mod tests;
mod vote;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub use self::args::{Dict, extract_args, filter_args};
#[cfg(feature = "embed-core")]
pub use self::knn::Knn;
pub use self::model::{ROUTER_GRAMMAR, ROUTER_SYSTEM, RouterPrompt, SOURCES, route_model, route_model_with, router_prompt};
pub use self::vote::{KnnParams, Vote, decide, neighbours, vote, vote_from};

/// A labelled example question (`eval/intents.jsonl`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub q: String,
    pub tool: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
    pub source: String,
    /// The model router's own choice, kept when a hand-verified route-check expectation overrode it.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "modelTool")]
    pub model_tool: Option<String>,
    /// A generated paraphrase: the real question it rewrites (`eval/intents.synth.jsonl`).
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "sourceQ")]
    pub source_q: Option<String>,
    /// For a paraphrase: whether the model router routed it to the source's tool and target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}
