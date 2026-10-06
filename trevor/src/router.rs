//! Routers that choose one of Trevor's table tools (`crate::tools`) for a question, or retrieval.
//!
//! Keyword routing misfired 13 times on real player phrasing (design/trevor-questions.md sections 8 to 11,
//! design/trevor-retrieval-baseline.md sections 16 to 20), so the choice is now a model's: [`route_model`] asks the
//! answer model (Gemma 4 12B) for `{"tool", "args"}` under a grammar that fixes the tool names and the enumerated
//! argument values. [`Knn`] is the serving router with no LLM: the nearest labelled example questions by embedding
//! vote on the tool, and the arguments come from a dictionary match against the same name tables
//! ([`extract_args`]). Both return [`Route::retrieve`] when unsure, and a tool that cannot resolve its arguments
//! sends the question to retrieval too.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::goldgen::llm::{Llm, Request};
use crate::tools::{self, CLASSES, Route, Tools, contains_words, norm};

pub const ROUTER_SYSTEM: &str = include_str!("../prompts/router.system.txt");
pub const ROUTER_GRAMMAR: &str = include_str!("../prompts/router.gbnf");

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
const VOICE_CLAUSE: &str = "\"voice\" for an operator's voice lines;";
const VOICE_CLAUSE_EVERYDAY: &str = "\"voice\" for an operator's voice lines, and for a question about one operator's \
everyday life at Rhodes Island that their own lines would mention (what they like, do for fun or in their downtime, think of \
the base or of other operators, how they feel about their own past, or how they act and talk day to day) when it names no \
event or story;";
const VOICE_EXAMPLE: &str = "Q: what does Hoshiguma get up to on her days off?\n{\"tool\": \"retrieve\", \"args\": {\"source\": \"voice\"}}\n";
/// The "art" source (`RouterPrompt::art`): its clause after the item clause, an example, and its grammar alternative.
const ITEM_CLAUSE: &str = "\"item\" for an item or its description;";
const ART_CLAUSE: &str = " \"art\" for what an operator's illustration, elite (E2) art or outfit art shows: what is drawn, \
worn or written in the picture;";
const ART_EXAMPLE: &str = "Q: What is Exusiai holding in her E2 art?\n{\"tool\": \"retrieve\", \"args\": {\"source\": \"art\"}}\n";

/// The `cross_ref` tool (`RouterPrompt::cross_ref`): its line after `dead_operators`, an example, and its grammar
/// alternative. Ian's "What is every boss we fight that joins our side" (2026-10-03) went to retrieval, which can never
/// list the whole cast.
const DEAD_LINE: &str = "- dead_operators {}: which playable operators are dead.";
const CROSS_REF_LINE: &str = "\n- cross_ref {table}: a question asking for every character across the whole cast who fits one relation \
the game data holds; table \"bosses_playable\" lists the boss enemies the player fights who are, or later become, playable \
operators (join Rhodes Island, join our side, become recruitable).";
const CROSS_REF_EXAMPLE: &str = "Q: which bosses can I later recruit as operators?\n{\"tool\": \"cross_ref\", \"args\": {\"table\": \"bosses_playable\"}}\n";
const CROSS_REF_GRAMMAR: &str = "  \"cross_ref\\\", \\\"args\\\": {\\\"table\\\": \\\"bosses_playable\\\"}\" |\n";

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
fn without_tool(system: &str, grammar: &str, tool: &str, rules: &[&str]) -> (String, String) {
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
fn without_compare(system: &str, grammar: &str) -> (String, String) {
    without_tool(system, grammar, "reading_compare", &["other ::="])
}

/// The prompt and grammar without the `overview` tool (added 2026-10-01), as before it.
fn without_overview(system: &str, grammar: &str) -> (String, String) {
    without_tool(system, grammar, "overview", &[])
}

/// The topic tool's line as written before the topics from data (2026-10-01), and its wider form for `ask --lore v2`,
/// where topics also cover places, organizations, factions and events.
const TOPIC_LINE: &str = "a question about what one race, nation or world concept is or is like in general (Sankta, Sarkaz, Laterano, Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";
const TOPIC_LINE_WIDE: &str = "a question about what one race, nation, city or place, organization or faction, historical event or world concept is or is like in general (Sankta, Sarkaz, Laterano, Londinium, Chernobog, Reunion, Rhodes Island, Rhine Lab, Babel, Originium, Oripathy, Arts, Seaborn, Catastrophes, the Precursors...)";

/// The prompt with the wider topic line (`ask --lore v2`); without it the prompt is the one before, byte for byte.
fn with_wide_topics(system: &str) -> String {
    assert!(system.contains(TOPIC_LINE), "router prompt lost its topic line");
    system.replace(TOPIC_LINE, TOPIC_LINE_WIDE)
}

/// The source kinds a retrieval route may name (`route_model_with(.., true)`): each maps to one kind of P4 unit, or
/// "story" (P3b, no preference) and "any" (P4, no preference).
pub const SOURCES: &[&str] = &["story", "operator_file", "module", "voice", "skin", "is", "enemy", "item", "any"];

/// The router prompt and grammar with a required `source` on retrieval routes, built from the plain ones so that
/// `source: false` sends exactly the prompt and grammar of before (2026-09-30).
fn source_variant() -> &'static (String, String) {
    static V: std::sync::OnceLock<(String, String)> = std::sync::OnceLock::new();
    V.get_or_init(|| {
        let line = include_str!("../prompts/router.source.txt").trim_end();
        let examples = include_str!("../prompts/router.source.examples.txt");
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
    let c = llm
        .complete(&Request {
            system: &system,
            user: &format!("Q: {q}"),
            grammar: Some(&grammar),
            seed: 1,
            temperature: 0.0,
            // The longest legal output (operator_filter with 8 fields of 60 characters) is about 700 tokens; real
            // outputs are 15 to 50.
            n_predict: 160,
            stop: &[],
        })
        .await?;
    Ok(serde_json::from_str::<Route>(c.content.trim()).unwrap_or_else(|_| Route::retrieve()))
}

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

/// Name dictionaries for argument extraction, normalized once.
pub struct Dict {
    events: Vec<(String, String)>,
    speakers: Vec<(String, String)>,
    dead: Vec<(String, String)>,
    operators: Vec<(String, String)>,
    places: Vec<(String, String)>,
    races: Vec<(String, String)>,
    branches: Vec<(String, String)>,
    topics: Vec<(String, String)>,
    runs: Vec<(String, String)>,
}

fn normed<I: IntoIterator<Item = String>>(it: I, min_len: usize) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = it.into_iter().map(|x| (norm(&x), x)).filter(|(n, _)| n.chars().count() >= min_len).collect();
    v.sort();
    v.dedup();
    v
}

impl Dict {
    #[must_use]
    pub fn new(t: &Tools) -> Self {
        let idents: Vec<String> = t.identities.iter().flatten().cloned().collect();
        let classes: Vec<String> = CLASSES.iter().map(|c| c.to_lowercase()).collect();
        Self {
            events: normed(t.groups.values().filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
                .filter_map(|g| g["name"].as_str().map(str::to_owned)), 4),
            // Speakers as the keyword route took them: capitalized, 3 characters or more ("Medic" and "Guard"
            // are speakers too, so a class word never counts as a name here).
            speakers: normed(t.speaker_names().into_iter().filter(|n| n.chars().next().is_some_and(char::is_uppercase))
                .map(str::to_owned).chain(idents.iter().cloned()), 3)
                .into_iter().filter(|(n, _)| !classes.iter().any(|c| n == c || *n == format!("{c}s"))).collect(),
            dead: normed(t.death_names().into_iter().chain(idents.iter().cloned()), 1),
            operators: normed(t.operator_names().into_iter().map(str::to_owned)
                .chain(t.real_names.iter().flatten().filter_map(|r| r["name"].as_str().map(str::to_owned)))
                .chain(idents.iter().cloned()), 1),
            places: normed(t.places(), 3),
            races: normed(t.attr_values("race"), 3),
            branches: normed(t.attr_values("branch").into_iter()
                .filter(|b| !classes.contains(&b.to_lowercase())), 3),
            topics: normed(t.topics.iter().flatten().flat_map(|r| std::iter::once(r["topic"].as_str().unwrap_or_default().to_owned())
                .chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned)))), 3),
            runs: normed(t.canon.as_ref().and_then(|c| c["runs"].as_array()).into_iter().flatten()
                .filter_map(|r| r["runName"].as_str().map(str::to_owned)), 3),
        }
    }
}

/// The longest dictionary entry the normalized question holds as whole words (a plural "s" allowed).
fn longest<'a>(nq: &str, dict: &'a [(String, String)]) -> Option<&'a str> {
    dict.iter().filter(|(n, _)| contains_words(nq, n) || contains_words(nq, &format!("{n}s")))
        .max_by_key(|(n, _)| n.len()).map(|(_, orig)| orig.as_str())
}

/// "episode 7", "chapter 14", "ep 7", "ch14" in a normalized question.
fn episode_in(nq: &str) -> Option<String> {
    let t: Vec<&str> = nq.split(' ').collect();
    for (i, w) in t.iter().enumerate() {
        if matches!(*w, "episode" | "chapter" | "ep" | "ch") {
            if let Some(n) = t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) {
                return Some(format!("Episode {n}"));
            }
        }
        for p in ["episode", "chapter", "ep", "ch"] {
            if let Some(n) = w.strip_prefix(p).and_then(|x| x.parse::<u64>().ok()) {
                return Some(format!("Episode {n}"));
            }
        }
    }
    None
}

fn any_word(nq: &str, words: &[&str]) -> bool {
    words.iter().any(|w| contains_words(nq, w))
}

/// Operator attribute fields by the words that ask for them, as the keyword route has them.
const FIELD_WORDS: &[(&str, &[&str])] = &[
    ("race", &["race", "species"]), ("birthplace", &["born", "birthplace", "place of birth", "hometown"]),
    ("birthday", &["birthday", "date of birth"]), ("height", &["height", "how tall", "tall"]),
    ("infection", &["infected", "infection", "oripathy"]), ("class", &["class", "archetype"]),
    ("branch", &["branch", "subclass"]), ("rarity", &["rarity", "how many stars", "star", "stars"]),
    ("gender", &["gender"]), ("affiliation", &["affiliation", "faction", "belong", "nation"]),
];

/// The filters of an `operator_filter` question by dictionary match: the keyword route's detection without its
/// guards (the classifier has already decided the question is a roster question).
#[must_use]
pub fn filter_args(q: &str, d: &Dict) -> BTreeMap<String, String> {
    let nq = norm(q);
    let raw = format!(" {} ", q.to_lowercase());
    let mut a = BTreeMap::new();
    let place = longest(&nq, &d.places).map(str::to_owned);
    if let Some(p) = &place {
        a.insert("place".into(), p.clone());
    }
    // "Ursus" is a nation and a race; a word taken as the place is not also a race filter.
    if let Some(r) = longest(&nq, &d.races).filter(|r| place.as_deref() != Some(*r)) {
        a.insert("race".into(), r.to_owned());
    }
    let classes: Vec<&str> = CLASSES.iter().filter(|c| any_word(&nq, &[&c.to_lowercase(), &format!("{}s", c.to_lowercase())])).copied().collect();
    if !classes.is_empty() {
        a.insert("class".into(), classes.join(","));
    }
    if let Some(b) = longest(&nq, &d.branches) {
        a.insert("branch".into(), b.to_owned());
    }
    if let Some(n) = (1..=6).find(|n| raw.contains(&format!(" {n}-star")) || raw.contains(&format!(" {n} star")) || raw.contains(&format!(" {n}*"))) {
        a.insert("rarity".into(), n.to_string());
    }
    if any_word(&nq, &["female", "women", "woman", "girls"]) {
        a.insert("gender".into(), "female".into());
    } else if any_word(&nq, &["male", "men", "man", "boys"]) {
        a.insert("gender".into(), "male".into());
    }
    if any_word(&nq, &["uninfected", "non infected", "not infected"]) {
        a.insert("infected".into(), "false".into());
    } else if any_word(&nq, &["infected"]) {
        a.insert("infected".into(), "true".into());
    }
    if any_word(&nq, &["tallest"]) {
        a.insert("sort".into(), "tallest".into());
    } else if any_word(&nq, &["shortest"]) {
        a.insert("sort".into(), "shortest".into());
    }
    a
}

/// The arguments `tool` needs, by dictionary match against the tables (longest whole-word match), or None when a
/// required one is not in the question.
#[must_use]
pub fn extract_args(tool: &str, q: &str, d: &Dict) -> Option<Route> {
    let nq = norm(q);
    let event = || episode_in(&nq).or_else(|| longest(&nq, &d.events).map(str::to_owned));
    let r = |args: Vec<(&str, String)>| Some(Route { tool: tool.to_owned(), args: args.into_iter().map(|(k, v)| (k.to_owned(), v)).collect() });
    match tool {
        "reading_guide" | "reading_chronological" | "timeline_all" | "dead_operators" => r(vec![]),
        "reading_event" | "reading_time" => r(vec![("event", event()?)]),
        "reading_compare" => {
            // Every event the question names: episodes by number, events by their longest names, first two in order.
            let mut found: Vec<(usize, String)> = Vec::new();
            let t: Vec<&str> = nq.split(' ').collect();
            for (i, w) in t.iter().enumerate() {
                let n = if matches!(*w, "episode" | "chapter" | "ep" | "ch") { t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) }
                    else { ["episode", "chapter", "ep", "ch"].iter().find_map(|p| w.strip_prefix(p).and_then(|x| x.parse::<u64>().ok())) };
                if let Some(n) = n {
                    found.push((nq.split(' ').take(i).map(|x| x.len() + 1).sum(), format!("Episode {n}")));
                }
            }
            let hay = format!(" {nq} ");
            for (n, orig) in &d.events {
                if let Some(at) = hay.find(&format!(" {n} ")) {
                    if !found.iter().any(|(_, o)| o == orig) {
                        found.push((at, orig.clone()));
                    }
                }
            }
            found.sort();
            found.dedup_by(|a, b| a.1 == b.1);
            if found.len() < 2 {
                return None;
            }
            r(vec![("event", found[0].1.clone()), ("other", found[1].1.clone())])
        }
        "recap" => {
            let ending = any_word(&nq, &["ending", "end of", "how does it end", "how did it end", "end"]);
            r(vec![("event", event()?), ("ending", ending.to_string())])
        }
        "deaths_in_event" => {
            let who = if any_word(&nq, &["non playable", "nonplayable", "npc", "npcs", "not playable"]) { "npc" }
                else if any_word(&nq, &["operator", "operators", "playable"]) { "operators" } else { "all" };
            r(vec![("event", event()?), ("who", who.into())])
        }
        "first_appearance" => r(vec![("character", longest(&nq, &d.speakers)?.to_owned())]),
        "death_of" => r(vec![("character", longest(&nq, &d.dead)?.to_owned())]),
        "real_name" => r(vec![("operator", longest(&nq, &d.operators)?.to_owned())]),
        "real_names" => r(longest(&nq, &d.places).map(|p| vec![("place", p.to_owned())]).unwrap_or_default()),
        "operator_attribute" => {
            let field = FIELD_WORDS.iter().find(|(_, ws)| any_word(&nq, ws)).map(|(f, _)| *f)?;
            r(vec![("operator", longest(&nq, &d.operators)?.to_owned()), ("field", field.into())])
        }
        "canon" => {
            // "IS3", "IS #3", "IS 3", "Integrated Strategies 3"; else a run name; else every run.
            let t: Vec<&str> = nq.split(' ').collect();
            let num = t.iter().enumerate().find_map(|(i, w)| {
                let after = |j: usize| t.get(j).and_then(|x| x.parse::<u64>().ok());
                w.strip_prefix("is").and_then(|x| x.parse::<u64>().ok())
                    .or_else(|| (*w == "is" || *w == "strategies").then(|| after(i + 1)).flatten())
            });
            match num.map(|n| format!("IS{n}")).or_else(|| longest(&nq, &d.runs).map(str::to_owned)) {
                Some(run) => r(vec![("run", run)]),
                None => r(vec![]),
            }
        }
        "topic" => r(vec![("topic", longest(&nq, &d.topics)?.to_owned())]),
        "operator_filter" => {
            let a = filter_args(q, d);
            if a.is_empty() {
                return None;
            }
            Some(Route { tool: tool.to_owned(), args: a })
        }
        _ => None,
    }
}

/// The nearest-neighbour vote for one question.
#[derive(Debug, Clone)]
pub struct Vote {
    /// The winning tool (possibly `retrieve`) before the thresholds.
    pub tool: String,
    /// Cosine of the nearest example.
    pub top: f32,
    /// The winner's share of the similarity-weighted vote.
    pub share: f32,
}

/// The `n` nearest examples to `v` by cosine (unit vectors), best first, leaving out those `skip` names.
#[must_use]
pub fn neighbours(examples: &[(Vec<f32>, String)], v: &[f32], n: usize, skip: impl Fn(usize) -> bool) -> Vec<(f32, usize)> {
    let mut sims: Vec<(f32, usize)> = examples.iter().enumerate().filter(|(i, _)| !skip(*i))
        .map(|(i, (e, _))| (e.iter().zip(v).map(|(a, b)| a * b).sum::<f32>(), i)).collect();
    let n = n.max(1).min(sims.len());
    if n > 0 && n < sims.len() {
        sims.select_nth_unstable_by(n - 1, |a, b| b.0.total_cmp(&a.0));
        sims.truncate(n);
    }
    sims.sort_by(|a, b| b.0.total_cmp(&a.0));
    sims
}

/// The similarity-weighted vote of the first `k` of `neigh`; ties go to the tool of the nearer example.
#[must_use]
pub fn vote_from(examples: &[(Vec<f32>, String)], neigh: &[(f32, usize)], k: usize) -> Vote {
    let sims = &neigh[..k.max(1).min(neigh.len())];
    let mut w: Vec<(String, f32, f32)> = Vec::new();
    for &(s, i) in sims {
        let t = &examples[i].1;
        match w.iter_mut().find(|x| &x.0 == t) {
            Some(x) => x.1 += s.max(0.0),
            None => w.push((t.clone(), s.max(0.0), s)),
        }
    }
    let total: f32 = w.iter().map(|x| x.1).sum();
    let best = w.iter().max_by(|a, b| a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2)));
    Vote {
        tool: best.map_or_else(|| tools::RETRIEVE.to_owned(), |b| b.0.clone()),
        top: sims.first().map_or(0.0, |s| s.0),
        share: best.map_or(0.0, |b| if total > 0.0 { b.1 / total } else { 0.0 }),
    }
}

/// k nearest neighbours over labelled examples; ties go to the tool of the nearer example.
#[must_use]
pub fn vote(examples: &[(Vec<f32>, String)], v: &[f32], k: usize, skip: Option<usize>) -> Vote {
    vote_from(examples, &neighbours(examples, v, k, |i| Some(i) == skip), k)
}

/// Thresholds of the kNN router: a table tool needs its nearest example at `min_sim` and `min_share` of the vote.
#[derive(Debug, Clone, Copy)]
pub struct KnnParams {
    pub k: usize,
    pub min_sim: f32,
    pub min_share: f32,
}

/// The routing decision from a vote: retrieval below either threshold, else the tool with extracted arguments
/// (retrieval when a required argument is not in the question).
#[must_use]
pub fn decide(v: &Vote, p: KnnParams, q: &str, d: &Dict) -> Route {
    if v.tool == tools::RETRIEVE || v.top < p.min_sim || v.share < p.min_share {
        return Route::retrieve();
    }
    extract_args(&v.tool, q, d).unwrap_or_else(Route::retrieve)
}

#[cfg(feature = "embed-core")]
pub use knn::Knn;

#[cfg(feature = "embed-core")]
mod knn {
    use std::collections::HashMap;
    use std::io::Write as _;
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use sha2::{Digest, Sha256};

    use super::{Dict, Intent, KnnParams, Route, decide, neighbours, vote_from};
    use crate::corpus::embed::QueryEmbedder;
    use crate::tools::Tools;

    /// The kNN router: labelled examples embedded with the query embedder (vectors cached per model under
    /// `artifacts/router/`, since embedding 1,200 questions takes about half a minute).
    pub struct Knn {
        embedder: QueryEmbedder,
        pub examples: Vec<(Vec<f32>, String)>,
        /// The real question each example is or paraphrases, so a held-out question also drops its paraphrases.
        pub origins: Vec<String>,
        /// Examples before this index are the real labelled questions, in file order; the rest are paraphrases.
        pub n_real: usize,
        pub params: KnnParams,
        pub dict: Dict,
    }

    impl Knn {
        /// # Errors
        /// Model load failures, or an unreadable intents file.
        pub fn load(model_dir: &Path, onnx: &str, intents: &Path, synth: Option<&Path>, params: KnnParams, tools: &Tools) -> Result<Self> {
            let tok = std::fs::read(model_dir.join("tokenizer.json")).with_context(|| format!("reading {}", model_dir.display()))?;
            let tok_sha = format!("{:x}", Sha256::digest(&tok));
            let mut embedder = QueryEmbedder::load(model_dir, onnx, None, &tok_sha)?;
            let read = |p: &Path| -> Result<Vec<Intent>> {
                Ok(std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?
                    .lines().map(serde_json::from_str).collect::<std::result::Result<_, _>>()?)
            };
            let mut items = read(intents)?;
            let n_real = items.len();
            if let Some(p) = synth {
                // Only paraphrases the model router sent to their source's tool and target.
                items.extend(read(p)?.into_iter().filter(|i| i.verified == Some(true)));
            }
            let cache_path = PathBuf::from("artifacts/router").join(format!("{:.16}.jsonl", embedder.model_sha));
            let mut cache: HashMap<String, Vec<f32>> = std::fs::read_to_string(&cache_path).unwrap_or_default().lines()
                .filter_map(|l| serde_json::from_str::<(String, Vec<f32>)>(l).ok()).collect();
            let mut fresh = Vec::new();
            let mut examples = Vec::with_capacity(items.len());
            for it in &items {
                let v = if let Some(v) = cache.get(&it.q) { v.clone() } else {
                    let v = embedder.embed(&it.q)?;
                    cache.insert(it.q.clone(), v.clone());
                    fresh.push((it.q.clone(), v.clone()));
                    v
                };
                examples.push((v, it.tool.clone()));
            }
            if !fresh.is_empty() {
                std::fs::create_dir_all("artifacts/router")?;
                let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&cache_path)?;
                for x in &fresh {
                    serde_json::to_writer(&mut f, x)?;
                    f.write_all(b"\n")?;
                }
            }
            let origins = items.into_iter().map(|i| i.source_q.unwrap_or(i.q)).collect();
            Ok(Self { embedder, examples, origins, n_real, params, dict: Dict::new(tools) })
        }

        /// # Errors
        /// An embedding failure.
        pub fn embed(&mut self, q: &str) -> Result<Vec<f32>> {
            self.embedder.embed(q)
        }

        /// Route one question. `holdout` drops examples with exactly this text and their paraphrases, so a question
        /// that is itself a labelled example is routed by its neighbours, not by its own label (the route check).
        ///
        /// # Errors
        /// An embedding failure.
        pub fn route(&mut self, q: &str, holdout: bool) -> Result<(Route, super::Vote)> {
            let v = self.embed(q)?;
            let origins = &self.origins;
            let neigh = neighbours(&self.examples, &v, self.params.k, |i| holdout && origins[i] == q);
            let vt = vote_from(&self.examples, &neigh, self.params.k);
            Ok((decide(&vt, self.params, q, &self.dict), vt))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wide_topic_line_changes_only_that_line() {
        for (sys, _) in [(ROUTER_SYSTEM.to_owned(), ""), (source_variant().0.clone(), "")] {
            let w = with_wide_topics(&sys);
            let (a, b): (Vec<&str>, Vec<&str>) = (sys.lines().collect(), w.lines().collect());
            assert_eq!(a.len(), b.len());
            assert_eq!(a.iter().zip(&b).filter(|(x, y)| x != y).count(), 1);
            assert!(w.contains("Reunion"));
        }
    }

    #[test]
    fn dropping_reading_compare_leaves_no_trace_of_it() {
        let (s, g) = without_compare(ROUTER_SYSTEM, ROUTER_GRAMMAR);
        assert!(!s.contains("reading_compare") && !g.contains("reading_compare") && !g.contains("other ::="));
        assert_eq!(ROUTER_SYSTEM.lines().count() - s.lines().count(), 7, "the tool line, the two rules, and 2 examples of 2 lines");
        assert_eq!(ROUTER_GRAMMAR.lines().count() - g.lines().count(), 2);
    }

    #[test]
    fn dropping_overview_leaves_no_trace_of_it() {
        let (s, g) = without_overview(ROUTER_SYSTEM, ROUTER_GRAMMAR);
        assert!(!s.contains("overview") && !g.contains("overview"));
        assert_eq!(ROUTER_SYSTEM.lines().count() - s.lines().count(), 6, "the tool line, the reading_compare rule, and 2 examples of 2 lines");
        assert_eq!(ROUTER_GRAMMAR.lines().count() - g.lines().count(), 1);
        let (s2, g2) = without_overview(&source_variant().0, &source_variant().1);
        assert!(!s2.contains("overview") && !g2.contains("overview"));
    }

    #[test]
    fn voice_rule_and_art_are_inert_when_off() {
        let base = RouterPrompt { source: true, compare: true, ..RouterPrompt::default() };
        let (s0, g0) = router_prompt(&base);
        // off: the prompt and grammar route_model_with sent before these options existed
        let (mut s_old, mut g_old) = (source_variant().0.clone(), source_variant().1.clone());
        (s_old, g_old) = without_overview(&s_old, &g_old);
        assert_eq!((s0.clone(), g0.clone()), (s_old, g_old));
        let (s1, g1) = router_prompt(&RouterPrompt { voice_rule: true, ..base });
        assert_eq!(g1, g0);
        assert!(s1.contains(VOICE_CLAUSE_EVERYDAY) && s1.contains("Hoshiguma") && !s0.contains("Hoshiguma"));
        assert_eq!(s1.replacen(VOICE_CLAUSE_EVERYDAY, VOICE_CLAUSE, 1).replacen(VOICE_EXAMPLE, "", 1), s0);
        assert!(s1.contains("}}\nQ: what does Hoshiguma") && s1.contains("\"voice\"}}\n\nAnswer with the JSON object only."));
        let (s2, g2) = router_prompt(&RouterPrompt { art: true, ..base });
        assert!(g2.contains("| \"any\" | \"art\")") && g2.replacen(" | \"art\"", "", 1) == g0);
        assert_eq!(s2.replacen(ART_CLAUSE, "", 1).replacen(ART_EXAMPLE, "", 1), s0);
        let (s3, g3) = router_prompt(&RouterPrompt { cross_ref: true, ..base });
        assert_eq!(s3.replacen(CROSS_REF_LINE, "", 1).replacen(CROSS_REF_EXAMPLE, "", 1), s0);
        assert_eq!(g3.replacen(CROSS_REF_GRAMMAR, "", 1), g0);
        assert!(g3.contains("cross_ref") && s3.contains("bosses_playable"));
        // without a source the options do nothing
        let plain = RouterPrompt { compare: true, ..RouterPrompt::default() };
        assert_eq!(router_prompt(&RouterPrompt { voice_rule: true, art: true, ..plain }), router_prompt(&plain));
    }

    #[test]
    fn the_source_variant_changes_only_retrieval() {
        let (system, grammar) = source_variant();
        assert!(system.contains("source names the kind of game text") && !system.contains("- retrieve {}: everything else."));
        assert!(!system.contains(r#""retrieve", "args": {}}"#) && system.contains("Answer with the JSON object only."));
        assert!(grammar.contains(r#""retrieve\", \"args\": {" source "}""#), "{grammar}");
        assert!(grammar.ends_with("source ::= \"\\\"source\\\": \\\"\" (\"story\" | \"operator_file\" | \"module\" | \"voice\" | \"skin\" | \"is\" | \"enemy\" | \"item\" | \"any\") \"\\\"\"\n"), "{grammar}");
        assert!(system.len() - ROUTER_SYSTEM.len() > 500);
    }

    fn dict() -> Dict {
        let mut t = Tools::default();
        for (id, name, ch) in [("act33side", "Babel", None), ("main_7", "The Birth of Tragedy", Some(7)), ("act25side", "Lone Trail", None)] {
            let mut g = json!({"groupId": id, "name": name});
            if let Some(c) = ch {
                g["chapter"] = json!(c);
            }
            t.groups.insert(id.to_owned(), g);
        }
        t.attributes = Some(vec![
            json!({"name": "Texas", "race": "Lupo", "nation": "Lungmen", "class": "Vanguard", "branch": "Pioneer"}),
            json!({"name": "Texas the Omertosa", "race": "Lupo", "nation": "Siracusa", "class": "Specialist", "branch": "Executor"}),
            json!({"name": "Gummy", "race": "Ursus", "nation": "Ursus", "class": "Defender", "branch": "Fortress"}),
            json!({"name": "Ceobe", "race": "Perro", "nation": "Rhodes Island", "class": "Caster", "branch": "Medic"}),
        ]);
        t.deaths = Some(vec![json!({"character": "W", "generic": false}), json!({"character": "Patriot", "generic": false})]);
        Dict::new(&t)
    }

    #[test]
    fn arguments_come_from_the_longest_whole_word_match() {
        let d = dict();
        let a = |tool: &str, q: &str| extract_args(tool, q, &d).map(|r| r.args.into_iter().collect::<Vec<_>>());
        assert_eq!(a("operator_attribute", "How tall is Texas the Omertosa?").unwrap(),
                   vec![("field".into(), "height".into()), ("operator".into(), "Texas the Omertosa".into())]);
        assert_eq!(a("reading_time", "How long is Lone Trail?").unwrap(), vec![("event".into(), "Lone Trail".into())]);
        assert_eq!(a("deaths_in_event", "Who died in episode 7?").unwrap(),
                   vec![("event".into(), "Episode 7".into()), ("who".into(), "all".into())]);
        assert_eq!(a("death_of", "Does W die?").unwrap(), vec![("character".into(), "W".into())]);
        assert!(a("death_of", "Does Wisadel die?").is_none(), "no name in the table: retrieval");
        assert!(a("reading_event", "When should I read the good one?").is_none());
        // Ursus as place, not also race; a branch named like its class (Medic) is left to the class.
        assert_eq!(a("operator_filter", "Which 6-star operators are Ursus?").unwrap(),
                   vec![("place".into(), "Ursus".into()), ("rarity".into(), "6".into())]);
        assert_eq!(a("operator_filter", "Which medics are infected?").unwrap(),
                   vec![("class".into(), "Medic".into()), ("infected".into(), "true".into())]);
    }

    #[test]
    fn the_vote_is_weighted_and_reports_its_share() {
        let ex = vec![(vec![1.0, 0.0], "death_of".to_owned()), (vec![0.9, 0.436], "death_of".to_owned()),
                      (vec![0.0, 1.0], "retrieve".to_owned())];
        let v = vote(&ex, &[1.0, 0.0], 3, None);
        assert_eq!(v.tool, "death_of");
        assert!((v.top - 1.0).abs() < 1e-6 && v.share > 0.99);
        let v = vote(&ex, &[1.0, 0.0], 3, Some(0));
        assert!((v.top - 0.9).abs() < 1e-6);
    }
}
