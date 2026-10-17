//! The model router: it chooses one of Trevor's table tools (`crate::tools`) for a question, or retrieval.
//!
//! Keyword routing misfired 13 times on real player phrasing (design/trevor-questions.md sections 8 to 11,
//! design/trevor-retrieval-baseline.md sections 16 to 20), so the choice is a model's: [`route_model_with`] asks the
//! answer model (Gemma 4 12B) for `{"tool", "args"}` under a grammar that fixes the tool names and the enumerated
//! argument values. Output that does not parse is [`Route::retrieve`](crate::tools::Route::retrieve), and a tool that
//! cannot resolve its arguments sends the question to retrieval too.

mod model;
#[cfg(test)]
mod tests;

pub use self::model::{ROUTER_GRAMMAR, ROUTER_SYSTEM, RouterPrompt, SOURCES, route_model_with, router_prompt};
