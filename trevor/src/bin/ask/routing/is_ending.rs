//! The Integrated Strategies ending route and the passages it adds.

use trevor::search::runtime::Runtime;
use trevor::tools::{Route, Tools};

use crate::cli::Args;
use crate::detect::has;
use crate::retrieval::Pre;

/// The IS-ending route: a question naming an Integrated Strategies run and one numbered ending,
/// not asking which ending is canon (the `canon` tool's question).
pub(crate) fn is_ending_route(q: &str, a: &Args, tools: &Tools) -> Option<Route> {
    if a.no_is_ending || has(q, &["canon", "canonical", "official", "true ending"]) {
        return None;
    }
    let (run, _, _, k) = tools.is_ending_question(q)?;
    Some(Route::new("is_ending", &[("run", run.as_str()), ("ending", k?.to_string().as_str())]))
}

/// The passages of an IS-ending route: the run's ending order from game data, then the ending's scene and endbook
/// stories from P4 (`is_rogue_N_ending_K_script`, `is_endbook_rogue_N_K_J`), in order.
pub(crate) fn is_ending_passages(rt: &Runtime, tools: &Tools, q: &str, r: &Route) -> Vec<Pre> {
    let Some((run, run_name, endings, Some(k))) = tools.is_ending_question(q) else { return Vec::new() };
    if r.args.get("run") != Some(&run) {
        return Vec::new();
    }
    let n: usize = run.trim_start_matches("rogue_").parse::<usize>().unwrap_or(0) + 1;
    let name = endings.iter().find(|(e, _)| *e == k).map_or(String::new(), |(_, x)| x.clone());
    let order = endings.iter().map(|(e, x)| format!("Ending {e} '{x}'")).collect::<Vec<_>>().join(", ");
    let mut pre = vec![Pre::new(format!("gamedata:{run}_endings"),
                                format!("game data: Integrated Strategies {n}, {run_name}, endings"),
                                format!("Integrated Strategies {n} ({run_name}) has {} numbered endings, \
                                         in the game's order: {order}.",
                                        endings.len()))];
    let script = format!("is_{run}_ending_{k}_script#");
    let book = format!("is_endbook_{run}_{k}_");
    for c in rt.store.chunks.iter().filter(|c| c.chunk_id.starts_with(&script)) {
        pre.push(Pre::new(c.chunk_id.clone(),
                          format!("Integrated Strategies {n}, {run_name}: Ending {k} '{name}', ending scene"),
                          c.text.clone()));
    }
    for c in rt.store.chunks.iter().filter(|c| c.chunk_id.starts_with(&book)) {
        let j = c.chunk_id[book.len()..].split('#').next().unwrap_or_default();
        let label = format!("Integrated Strategies {n}, {run_name}: Ending {k} '{name}', endbook story {j}");
        pre.push(Pre::new(c.chunk_id.clone(), label, c.text.clone()));
    }
    pre
}
