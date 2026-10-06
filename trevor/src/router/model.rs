//! The model router: one grammar-bound call to the answer model, and the prompt variants each later part
//! (sources, reading compare, overview, wider topics, voice rule, art, cross-references) adds or drops.

use anyhow::Result;

use crate::goldgen::llm::{Llm, Request};
use crate::tools::Route;

pub const ROUTER_SYSTEM: &str = include_str!("../../prompts/router.system.txt");
pub const ROUTER_GRAMMAR: &str = include_str!("../../prompts/router.gbnf");

/// One grammar-bound call to the answer model. Output that does not parse (it cannot under the grammar unless
/// cut off at `n_predict`) is retrieval.
///
/// # Errors
/// Server errors after the client's retries.
pub async fn route_model(llm: &Llm, q: &str) -> Result<Route> {
    route_model_with(llm, q, &RouterPrompt { compare: true, ..RouterPrompt::default() }).await
}

/// Which parts the model router's prompt and grammar carry. Each `false` drops a part added later, so the prompt
/// with every later part off is the one before it, line for line.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct RouterPrompt {
    /// A retrieval route names its source (2026-09-30).
    pub source: bool,
    /// The `reading_compare` tool (2026-09-30).
    pub compare: bool,
    /// The `overview` tool (2026-10-01).
    pub overview: bool,
    /// The wider topic line of `--lore v2` (2026-10-01).
    pub wide_topics: bool,
    /// With `source`: everyday-life questions about one operator go to their voice lines (2026-10-03).
    pub voice_rule: bool,
    /// With `source`: the "art" source, model-written descriptions of operator art (2026-10-03).
    pub art: bool,
    /// The `cross_ref` tool: cast-wide cross-references from game data (2026-10-03, `ask --cross-ref`).
    pub cross_ref: bool,
}

/// The source line's voice clause, and the clause with the everyday-life rule (`RouterPrompt::voice_rule`). Six
/// trivia misses were voice-line questions ("what does broca do for a hobby?") the router sent to "story", whose
/// corpus (P3b) has no voice lines (2026-10-01).
pub(crate) const VOICE_CLAUSE: &str = "\"voice\" for an operator's voice lines;";
pub(crate) const VOICE_CLAUSE_EVERYDAY: &str = "\"voice\" for an operator's voice lines, and for a question about one operator's \
everyday life at Rhodes Island that their own lines would mention (what they like, do for fun or in their downtime, think of \
the base or of other operators, how they feel about their own past, or how they act and talk day to day) when it names no \
event or story;";
pub(crate) const VOICE_EXAMPLE: &str = "Q: what does Hoshiguma get up to on her days off?\n{\"tool\": \"retrieve\", \"args\": {\"source\": \"voice\"}}\n";
/// The "art" source (`RouterPrompt::art`): its clause after the item clause, an example, and its grammar alternative.
pub(crate) const ITEM_CLAUSE: &str = "\"item\" for an item or its description;";
pub(crate) const ART_CLAUSE: &str = " \"art\" for what an operator's illustration, elite (E2) art or outfit art shows: what is drawn, \
worn or written in the picture;";
pub(crate) const ART_EXAMPLE: &str = "Q: What is Exusiai holding in her E2 art?\n{\"tool\": \"retrieve\", \"args\": {\"source\": \"art\"}}\n";

/// The `cross_ref` tool (`RouterPrompt::cross_ref`): its line after `dead_operators`, an example, and its grammar
/// alternative. Ian's "What is every boss we fight that joins our side" (2026-10-03) went to retrieval, which can never
/// list the whole cast.
pub(crate) const DEAD_LINE: &str = "- dead_operators {}: which playable operators are dead.";
pub(crate) const CROSS_REF_LINE: &str = "\n- cross_ref {table}: a question asking for every character across the whole cast who fits one relation \
the game data holds; table \"bosses_playable\" lists the boss enemies the player fights who are, or later become, playable \
operators (join Rhodes Island, join our side, become recruitable).";
pub(crate) const CROSS_REF_EXAMPLE: &str = "Q: which bosses can I later recruit as operators?\n{\"tool\": \"cross_ref\", \"args\": {\"table\": \"bosses_playable\"}}\n";
pub(crate) const CROSS_REF_GRAMMAR: &str = "  \"cross_ref\\\", \\\"args\\\": {\\\"table\\\": \\\"bosses_playable\\\"}\" |\n";

/// The router's system prompt and grammar for `o`.
#[must_use]
pub fn router_prompt(o: &RouterPrompt) -> (String, String) {
    let (system, grammar) = if o.source { (source_variant().0.as_str(), source_variant().1.as_str()) } else { (ROUTER_SYSTEM, ROUTER_GRAMMAR) };
    let mut system = system.to_owned();
    let mut grammar = grammar.to_owned();
    if !o.compare {
        (system, grammar) = without_compare(&system, &grammar);
    }
    if !o.overview {
        (system, grammar) = without_overview(&system, &grammar);
    }
    if o.wide_topics {
        system = with_wide_topics(&system);
    }
    let examples_end = "\nAnswer with the JSON object only.";
    if o.source && o.voice_rule {
        assert!(system.contains(VOICE_CLAUSE), "router source line lost its voice clause");
        system = system.replacen(VOICE_CLAUSE, VOICE_CLAUSE_EVERYDAY, 1).replacen(examples_end, &format!("{VOICE_EXAMPLE}{examples_end}"), 1);
    }
    if o.source && o.art {
        assert!(system.contains(ITEM_CLAUSE), "router source line lost its item clause");
        system = system.replacen(ITEM_CLAUSE, &format!("{ITEM_CLAUSE}{ART_CLAUSE}"), 1)
            .replacen(examples_end, &format!("{ART_EXAMPLE}{examples_end}"), 1);
        grammar = grammar.replacen("| \"any\")", "| \"any\" | \"art\")", 1);
        assert!(grammar.contains("\"art\")"), "router grammar lost its source rule");
    }
    if o.cross_ref {
        assert!(system.contains(DEAD_LINE), "router prompt lost its dead_operators line");
        system = system.replacen(DEAD_LINE, &format!("{DEAD_LINE}{CROSS_REF_LINE}"), 1)
            .replacen(examples_end, &format!("{CROSS_REF_EXAMPLE}{examples_end}"), 1);
        let anchor = "  \"dead_operators\\\", \\\"args\\\": {}\" |\n";
        assert!(grammar.contains(anchor), "router grammar lost its dead_operators rule");
        grammar = grammar.replacen(anchor, &format!("{anchor}{CROSS_REF_GRAMMAR}"), 1);
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
    let g: Vec<&str> = grammar.split('\n').filter(|l| !l.contains(tool) && !rules.iter().any(|r| l.starts_with(r))).collect();
    (keep.join("\n"), g.join("\n"))
}

/// The prompt and grammar without the `reading_compare` tool (added 2026-09-30).
pub(crate) fn without_compare(system: &str, grammar: &str) -> (String, String) {
    without_tool(system, grammar, "reading_compare", &["other ::="])
}

/// The prompt and grammar without the `overview` tool (added 2026-10-01), as before it.
pub(crate) fn without_overview(system: &str, grammar: &str) -> (String, String) {
    without_tool(system, grammar, "overview", &[])
}

/// The topic tool's line as written before the topics from data (2026-10-01), and its wider form for `ask --lore v2`,
/// where topics also cover places, organizations, factions and events.
pub(crate) const TOPIC_LINE: &str = "a question about what one race, nation or world concept is or is like in general (Sankta, Sarkaz, Laterano, Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";
pub(crate) const TOPIC_LINE_WIDE: &str = "a question about what one race, nation, city or place, organization or faction, historical event or world concept is or is like in general (Sankta, Sarkaz, Laterano, Londinium, Chernobog, Reunion, Rhodes Island, Rhine Lab, Babel, Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";

/// The prompt with the wider topic line (`ask --lore v2`); without it the prompt is the one before, byte for byte.
pub(crate) fn with_wide_topics(system: &str) -> String {
    assert!(system.contains(TOPIC_LINE), "router prompt lost its topic line");
    system.replace(TOPIC_LINE, TOPIC_LINE_WIDE)
}

/// The source kinds a retrieval route may name (`route_model_with(.., true)`): each maps to one kind of P4 unit, or
/// "story" (P3b, no preference) and "any" (P4, no preference).
pub const SOURCES: &[&str] = &["story", "operator_file", "module", "voice", "skin", "is", "enemy", "item", "any"];

/// The router prompt and grammar with a required `source` on retrieval routes, built from the plain ones so that
/// `source: false` sends exactly the prompt and grammar of before (2026-09-30).
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

/// As [`route_model`]; with `source`, a retrieval route also names the kind of text that holds the answer; without
/// `compare`, the router has no `reading_compare` tool (as before 2026-09-30); without `overview`, no `overview` tool
/// (as before 2026-10-01).
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
