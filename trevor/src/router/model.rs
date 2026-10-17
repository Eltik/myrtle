//! The model router: one grammar-bound call to the answer model, and the prompt variants each later part
//! (sources, reading compare, wider topics) adds or drops.

use anyhow::Result;

use crate::llm::{Llm, Request};
use crate::tools::Route;

pub const ROUTER_SYSTEM: &str = include_str!("../../prompts/router.system.txt");
pub const ROUTER_GRAMMAR: &str = include_str!("../../prompts/router.gbnf");

/// Which parts the model router's prompt and grammar carry. Each `false` drops a part added later, so the prompt
/// with every later part off is the one before it, line for line.
#[derive(Debug, Clone, Copy, Default)]
pub struct RouterPrompt {
    /// A retrieval route names its source.
    pub source: bool,
    /// The `reading_compare` tool.
    pub compare: bool,
    /// The wider topic line of `--lore v2`.
    pub wide_topics: bool,
}

/// The router's system prompt and grammar for `o`.
#[must_use]
pub fn router_prompt(o: &RouterPrompt) -> (String, String) {
    let (system, grammar) = if o.source {
        (source_variant().0.as_str(), source_variant().1.as_str())
    } else {
        (ROUTER_SYSTEM, ROUTER_GRAMMAR)
    };
    let mut system = system.to_owned();
    let mut grammar = grammar.to_owned();
    if !o.compare {
        (system, grammar) = without_compare(&system, &grammar);
    }
    if o.wide_topics {
        system = with_wide_topics(&system);
    }
    (system, grammar)
}

/// The prompt and grammar without one tool: its description, rules, examples and grammar alternative are the only
/// lines that name it, so dropping them (and the grammar rules in `rules`) gives the prompt and grammar before it.
pub(crate) fn without_tool(system: &str, grammar: &str, tool: &str, rules: &[&str]) -> (String, String) {
    let lines: Vec<&str> = system.split('\n').collect();
    let mut keep = Vec::with_capacity(lines.len());
    for (i, l) in lines.iter().enumerate() {
        let next_is = lines.get(i + 1).is_some_and(|n| n.contains(tool));
        if l.contains(tool) || (l.starts_with("Q: ") && next_is) {
            continue;
        }
        keep.push(*l);
    }
    let g: Vec<&str> = grammar
        .split('\n')
        .filter(|l| !l.contains(tool) && !rules.iter().any(|r| l.starts_with(r)))
        .collect();
    (keep.join("\n"), g.join("\n"))
}

/// The prompt and grammar without the `reading_compare` tool.
pub(crate) fn without_compare(system: &str, grammar: &str) -> (String, String) {
    without_tool(system, grammar, "reading_compare", &["other ::="])
}

/// The topic tool's line as written before the topics from data, and its wider form for `ask --lore v2`,
/// where topics also cover places, organizations, factions and events.
pub(crate) const TOPIC_LINE: &str = "a question about what one race, nation or world concept is or is like in general \
    (Sankta, Sarkaz, Laterano, Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";
pub(crate) const TOPIC_LINE_WIDE: &str = "a question about what one race, nation, city or place, \
    organization or faction, historical event or world concept is or is like in general \
    (Sankta, Sarkaz, Laterano, Londinium, Chernobog, Reunion, Rhodes Island, Rhine Lab, Babel, \
    Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";

/// The prompt with the wider topic line (`ask --lore v2`); without it the prompt is the one before, byte for byte.
pub(crate) fn with_wide_topics(system: &str) -> String {
    assert!(system.contains(TOPIC_LINE), "router prompt lost its topic line");
    system.replace(TOPIC_LINE, TOPIC_LINE_WIDE)
}

/// The source kinds a retrieval route may name (`route_model_with(.., true)`): each maps to one kind of P4 unit, or
/// "story" (P3b, no preference) and "any" (P4, no preference).
pub const SOURCES: &[&str] = &["story", "operator_file", "module", "voice", "skin", "is", "enemy", "item", "any"];

/// The router prompt and grammar with a required `source` on retrieval routes, built from the plain ones so that
/// `source: false` sends exactly the plain prompt and grammar.
pub(crate) fn source_variant() -> &'static (String, String) {
    static V: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        let line = include_str!("../../prompts/router.source.txt").trim_end();
        let examples = include_str!("../../prompts/router.source.examples.txt");
        let system = ROUTER_SYSTEM.replace("- retrieve {}: everything else.", line)
            .replace(r#"{"tool": "retrieve", "args": {}}"#, r#"{"tool": "retrieve", "args": {"source": "story"}}"#)
            .replace("\nAnswer with the JSON object only.", &format!("{examples}\nAnswer with the JSON object only."));
        let kinds = SOURCES.iter().map(|k| format!("\"{k}\"")).collect::<Vec<_>>().join(" | ");
        let grammar = ROUTER_GRAMMAR.replace(r#""retrieve\", \"args\": {}""#, r#""retrieve\", \"args\": {" source "}""#)
            + &format!("source ::= \"\\\"source\\\": \\\"\" ({kinds}) \"\\\"\"\n");
        (system, grammar)
    })
}

/// One grammar-bound call to the answer model with the prompt and grammar `o` picks. Output that does not parse (it
/// cannot under the grammar unless cut off at `n_predict`) is retrieval. With `source`, a retrieval route also names
/// the kind of text that holds the answer; without `compare`, the router has no `reading_compare` tool.
///
/// # Errors
/// Server errors after the client's retries.
pub async fn route_model_with(llm: &Llm, q: &str, o: &RouterPrompt) -> Result<Route> {
    let (system, grammar) = router_prompt(o);
    // n_predict 160: the longest legal output (operator_filter with 8 fields of 60 characters) is about 700 tokens; real
    // outputs are 15 to 50.
    let c = llm.complete(&Request::greedy(&system, &format!("Q: {q}"), Some(&grammar), 160, &[])).await?;
    Ok(serde_json::from_str::<Route>(c.content.trim()).unwrap_or_else(|_| Route::retrieve()))
}
