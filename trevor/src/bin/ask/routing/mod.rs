//! Routing a question to a table tool or to retrieval: the keyword rules (the kill switch), the model router, and
//! the `--route-only` output.

use anyhow::{Context, Result};
use trevor::llm::Llm;
use trevor::router;
use trevor::tools::{Route, ToolError, Tools, TopicPassage, norm};

use crate::cli::{Args, RouterKind, dossier_allowed, router_prompt};
use crate::detect::is_open_listing;

mod is_ending;
mod part_recap;
mod rules;

pub(crate) use is_ending::{is_ending_passages, is_ending_route};
use rules::keyword_route;

/// What a router chose, the table's text when a tool answered, and why a chosen tool did not answer.
pub(crate) struct Routed {
    pub(crate) route: Route,
    pub(crate) text: Option<String>,
    pub(crate) error: Option<ToolError>,
    /// A topic summary to add to retrieval (the topic tool never answers on its own).
    pub(crate) topic: Option<TopicPassage>,
    /// The new dossier `ask --lore v2` adds, by name (shown by `--route-only`).
    pub(crate) dossier: Option<String>,
    /// The characters of the game-data passage `ask --lore v2` adds (shown by `--route-only`).
    pub(crate) game: Option<String>,
    pub(crate) ms: f64,
}

impl Routed {
    /// What a retrieval route adds before the retrieved passages, as `--route-only` shows it: " (topic passage: X)",
    /// " (dossier passage: X)", " (game data: X)", each when present.
    pub(crate) fn passage_notes(&self) -> String {
        let topic = self.topic.as_ref().map_or(String::new(), |t| format!(" (topic passage: {})", t.topic));
        let dossier = self.dossier.as_ref().map_or(String::new(), |d| format!(" (dossier passage: {d})"));
        let game = self.game.as_ref().map_or(String::new(), |g| format!(" (game data: {g})"));
        format!("{topic}{dossier}{game}")
    }
}

/// Choose a table tool with the router `a.router` and run it. `llm` is needed by the model router only.
pub(crate) async fn route_question(q: &str, a: &Args, tools: &Tools, llm: Option<&Llm>) -> Result<Routed> {
    let started = std::time::Instant::now();
    // First and last appearances (`--no-appearance-order`, `appear.rs`), for every router: the appearance index in
    // release (or in-world) order; retrieval cannot tell which of its passages is the last.
    if !a.no_route && !a.no_appearance_order {
        if let Some(oq) = crate::appear::order_question(q, tools) {
            if let Some(text) = crate::appear::order_text(tools, &oq, None, None) {
                return Ok(Routed { route: crate::appear::route(&oq), text: Some(text), error: None, topic: None,
                                   dossier: None, game: None, ms: started.elapsed().as_secs_f64() * 1000.0 });
            }
        }
    }
    // "Was Talulah mentioned in People?" (`--no-event-mention`, 2026-10-19), for every router: the appearance index
    // answers whether a character speaks or is named in an event the question names.
    if !a.no_route && !a.no_event_mention {
        if let Some(text) = tools.event_mention(q) {
            return Ok(Routed { route: Route::new("appearances", &[("question", q)]), text: Some(text), error: None,
                               topic: None, dossier: None, game: None, ms: started.elapsed().as_secs_f64() * 1000.0 });
        }
    }
    // Detectors that read the question only, before the router: every router line added for a tool moved
    // 20 to 34 unrelated routes, so these tools are reached without one. The keyword router gets them too since
    // 2026-10-19 (`--keyword-detectors-v1`: model router only).
    let detectors = a.router == RouterKind::Model || (a.router == RouterKind::Keywords && !a.keyword_detectors_v1);
    if detectors && !a.no_route {
        let ms = || started.elapsed().as_secs_f64() * 1000.0;
        if let Some(route) = is_ending_route(q, a, tools) {
            return Ok(Routed { route, text: None, error: None, topic: None, dossier: None, game: None, ms: ms() });
        }
        // "What's going on at the beginning of Ch 10?" (`--no-event-part-recap`, `part_recap.rs`): the summary of the
        // event's first or last story; a part the tables cannot serve goes on to the router as before.
        if let Some(route) = part_recap::part_recap_route(q, a, tools) {
            if let Ok(text) = tools.run(&route) {
                return Ok(Routed {
                    route, text: Some(text), error: None, topic: None, dossier: None, game: None, ms: ms()
                });
            }
        }
        // A time-gap question goes to retrieval, which adds the time-gap evidence (`timegap.rs`); the router read
        // "the time skip from chapter 8 to chapter 9" as a reading-order comparison.
        if !a.no_time_gap && crate::timegap::time_gap_events(q, tools, crate::chrono::load_chrono_cached()).is_some() {
            return Ok(Routed {
                route: Route::retrieve(), text: None, error: None, topic: None, dossier: None, game: None, ms: ms()
            });
        }
        // "Why did Priestess show up in chapter 15?" asks for a reason, not a list (`--keyword-detectors-v1` lets it
        // through, as before 2026-10-19).
        let reason = a.keyword_detectors_v1 || !matches!(norm(q).split(' ').next(), Some("why" | "how"));
        let pre = (!a.no_appearances && reason).then(|| tools.appearance_question(q)).flatten()
            .map(|c| Route::new("appearances", &[("character", c.as_str())]))
            .or_else(|| (!a.no_cross_ref_trigger).then(|| tools.cross_ref_question(q)).flatten()
                .map(|t| Route::new("cross_ref", &[("table", t)])));
        if let Some(route) = pre {
            if let Ok(text) = tools.run(&route) {
                return Ok(Routed {
                    route, text: Some(text), error: None, topic: None, dossier: None, game: None, ms: ms()
                });
            }
        }
    }
    let route = match a.router {
        RouterKind::Off => Route::retrieve(),
        RouterKind::Keywords => {
            let (route, text) = keyword_route(q, tools).map_or((Route::retrieve(), None), |(r, t)| (r, Some(t)));
            return Ok(Routed {
                route, text, error: None, topic: None, dossier: None, game: None,
                ms: started.elapsed().as_secs_f64() * 1000.0
            });
        }
        RouterKind::Model => router::route_model_with(
            llm.context("the model router needs a llama-server")?, q, &router_prompt(a)).await?,
    };
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    let dossier = tools.new_dossier(q).filter(|_| dossier_allowed(q, a, Some(&route))).map(|d| d.topic);
    let game = tools.game_data(q).map(|g| g.topic);
    if route.tool == "topic" {
        if a.no_topics {
            return Ok(Routed { route, text: None, error: None, topic: None, dossier: None, game: None, ms });
        }
        let r = route.args.get("topic").map_or(Err(ToolError::MissingArg("topic")), |t| tools.topic(t));
        let (topic, error) = match r { Ok(p) => (Some(p), None), Err(e) => (None, Some(e)) };
        return Ok(Routed { route, text: None, error, topic, dossier, game, ms });
    }
    let (text, error) = if route.is_retrieve() { (None, None) } else {
        match tools.run(&route) { Ok(t) => (Some(t), None), Err(e) => (None, Some(e)) }
    };
    let (dossier, game) = if text.is_none() { (dossier, game) } else { (None, None) };
    Ok(Routed { route, text, error, topic: None, dossier, game, ms })
}

/// `--route-only`: the keyword router prints as before (the table text, or RETRIEVAL); the model router
/// prints its choice first, as `ROUTE {"tool": ..., "args": {...}}`.
pub(crate) fn print_route_only(q: &str, a: &Args, r: &Routed) -> Result<()> {
    if a.router != RouterKind::Keywords {
        println!("ROUTE {}", serde_json::to_string(&r.route)?);
    }
    match &r.text {
        Some(t) => println!("{t}"),
        None => {
            let guard = if is_open_listing(q) { " (listing guard)" } else { "" };
            let why = r.error.as_ref().map_or(String::new(), |e| format!(" (tool failed: {e})"));
            println!("RETRIEVAL{guard}{why}{}", r.passage_notes());
        }
    }
    Ok(())
}
