//! Answer a lore question from retrieved passages with a local model.
//!
//! Hybrid retrieval over the default corpus (stories plus operator archives), each hit widened by
//! its neighbouring chunks in the same story (`--neighbors`, 0 turns it off), the passages numbered
//! and handed to the model, which must answer only from them, cite them as [n], and say so when
//! they do not hold the answer. Needs a llama-server (`--server`). `--batch <goldset.jsonl>`
//! answers every item of a gold set into `--out` (resumable), for the answer eval.

mod cli;
mod prompts;
mod form;
mod detect;
mod chrono;
mod scenes;
mod terms;
mod retrieval;
mod routing;
mod deduce;
mod answer;
mod batch;
mod server;
mod intents;

use std::path::PathBuf;

use anyhow::{Context, Result};
use trevor::goldgen::llm::Llm;
use trevor::search::runtime::Runtime;
use trevor::tools::{ToolOptions, Tools};

use crate::answer::{Tables, answer_one};
use crate::chrono::load_chrono;
use crate::cli::{Args, LoreSet, RouterKind, deep_for, p4_dir, retrieval_plan, router_source};
use crate::detect::source_kind;
use crate::intents::{knn_eval, label_intents, load_knn, paraphrase_intents};
use crate::retrieval::{Runtimes, load_names};
use crate::routing::{print_route_only, route_question};
use crate::scenes::line_study;
use crate::server::{Spawned, connect_or_spawn};

#[tokio::main]
async fn main() -> Result<()> {
    // `ask` answers from P3b (P3a plus the 200 dossiers and 88 event summaries) unless --corpus is given: on 120
    // real Reddit questions it responds 0.575 against 0.550 (4 gained, 1 lost) and on gold set v2 0.641 against
    // 0.618 (Ian, 2026-09-28). Search and eval keep P3a, where P3b costs 0.015 retrieval recall.
    let m = <Args as clap::CommandFactory>::command().get_matches();
    let mut a = <Args as clap::FromArgMatches>::from_arg_matches(&m)?;
    let corpus_default = m.value_source("corpus") == Some(clap::parser::ValueSource::DefaultValue);
    // RELATION_RULE is the default since 2026-10-05 night (`--no-relation-rule`).
    a.relation_rule = !a.no_relation_rule;
    if corpus_default {
        // P4 (P3b plus 6,347 chunks of module stories, voice lines, IS, enemies, outfits and items) only for a question
        // that names one of those sources: on gold set v2 it cost 0.008 correct and 0.025 faithfulness against P3b,
        // while source-specific real questions went from 1 to 3 of 7 (2026-09-29).
        // With the model router naming the source, the corpus follows its route instead (set after routing).
        let source_q = !router_source(&a) && a.batch.is_none() && a.question.as_deref().is_some_and(|q| source_kind(q).is_some());
        let (p3b, p4) = (a.lore.dirs().0, p4_dir(&a));
        a.runtime.corpus = PathBuf::from(if source_q { p4 } else { p3b });
    }
    let chrono = load_chrono();
    let tools = load_tools(&a);
    if a.knn_eval {
        return knn_eval(&a, &tools);
    }
    if a.show_expansion {
        return batch::show_expansion(&a, &tools);
    }
    if a.form_only {
        return batch::form_only(&a, &tools).await;
    }
    if let Some(out) = a.paraphrase_intents.clone() {
        let (llm, _server) = connect_or_spawn(&a).await?;
        return paraphrase_intents(&a, &tools, &llm, &out).await;
    }
    if let Some(out) = a.label_intents.clone() {
        let (llm, _server) = connect_or_spawn(&a).await?;
        return label_intents(&tools, &llm, &out).await;
    }
    let knn = if matches!(a.router, RouterKind::Knn | RouterKind::Hybrid) && !a.no_route { Some(load_knn(&a, &tools)?) } else { None };
    let mut tables = Tables { tools, knn, topic: None, route: None };
    // Table-routed questions need no index: answer them before loading it. The keyword, kNN and off routers need
    // no model either, so they answer before starting or connecting to a server.
    let mut early: Option<(Llm, Option<Spawned>)> = None;
    if let (Some(b), true) = (&a.batch, a.detect_only) {
        return batch::detect_only(&a, &tables.tools, b);
    }
    if let (Some(b), true, false) = (&a.batch, a.route_only, a.no_route) {
        return batch::route_only(&a, &mut tables, b).await;
    }
    if let (None, Some(q), false) = (&a.batch, &a.question, a.no_route) {
        // The hybrid router connects only when kNN is unsure, which it learns after the vote; it connects up front
        // like the model router for simplicity, since a question it routes to retrieval needs the server anyway.
        let llm = if matches!(a.router, RouterKind::Model | RouterKind::Hybrid) { Some(connect_or_spawn(&a).await?) } else { None };
        tables.tools.topics_deep = deep_for(&a, &tables.tools, q);
        let r = route_question(q, &a, &tables.tools, tables.knn.as_mut(), llm.as_ref().map(|x| &x.0)).await?;
        if a.route_only {
            if a.router == RouterKind::Model {
                eprintln!("model router: {:.0} ms", r.ms);
            }
            return print_route_only(q, &a, &r);
        }
        if let Some(text) = r.text {
            println!("{text}");
            return Ok(());
        }
        tables.topic = r.topic;
        if corpus_default && retrieval_plan(q, &a, Some(&r.route)).0 == Some(true) {
            a.runtime.corpus = PathBuf::from(p4_dir(&a));
        }
        tables.route = Some(r.route);
        early = llm;
    }
    // Loaded after the table routes, which need no index (a table answer no longer waits on the ONNX load).
    let mut rts = Runtimes { main: Runtime::load(&a.runtime, true, true, a.rerank_add > 0)?, p4: None, art: None, p4_dir: p4_dir(&a),
                             switch: corpus_default && router_source(&a), args: a.runtime.clone(), rerank: a.rerank_add > 0 };
    if let (Some(b), true) = (&a.batch, a.line_study) {
        return line_study(&mut rts.main, &rts.args, b);
    }
    if let (Some(b), true) = (&a.batch, a.scoped_only) {
        return batch::scoped_only(&mut rts, &tables.tools, b);
    }
    if let (Some(b), true) = (&a.batch, a.detect_identity) {
        return batch::detect_identity(&a, &mut rts, b);
    }
    let (llm, _server) = match early { Some(x) => x, None => connect_or_spawn(&a).await? };
    let names = load_names();
    if let Some(b) = &a.batch {
        return batch::answer_batch(&a, &mut rts, &llm, &names, &chrono, &mut tables, b).await;
    }
    let q = a.question.clone().context("give a question, or --batch")?;
    // The single question was routed above; answer it from retrieval.
    let mut once = a.clone();
    once.routed = true;
    let ans = answer_one(&mut rts, &llm, &names, &chrono, &q, &once, &mut tables).await?;
    println!("{}\n", ans.answer);
    // Sources under the same [n] the answer uses.
    for (i, id) in ans.passages.iter().enumerate() {
        if ans.cited.contains(id) {
            println!("  [{}] {id}", i + 1);
        }
    }
    if ans.invalid_citations > 0 {
        println!("  ({} citation(s) pointed at no passage)", ans.invalid_citations);
    }
    Ok(())
}

/// The table tools as the flags configure them: the lore set's topics and dossiers, and the behaviors each router
/// keeps or drops.
fn load_tools(a: &Args) -> Tools {
    let mut tools = Tools::load(std::path::Path::new("."));
    if a.lore == LoreSet::V1 {
        tools.topics_v1();
    } else {
        tools.load_new_dossiers(std::path::Path::new("."));
        tools.game_data = !a.no_game_data;
        tools.topics_v2_text = true;
        tools.topics_speaker_check = a.speaker_trait_check;
        tools.topics_race_rule = !a.no_race_trait_rule;
        // Deep entries are switched on per question (`deep_for`, the broad-question gate).
        tools.topics_deep = false;
    }
    // The keyword router is the kill switch for all routing work, so it keeps the tools as they were.
    // The exact recap match changes the keyword router's "Explain the ending of Episode 1" (and Episode 0), so it
    // applies to the model and kNN routers only.
    tools.opts = if a.router == RouterKind::Keywords { ToolOptions { canon_v1: a.canon_v1, ..ToolOptions::KEYWORDS } }
        else { ToolOptions { storylines: !a.no_storylines, exact_recap: !a.recap_substring, is_offset: !a.is_numbering_v1,
                             canon_v1: a.canon_v1, wiki_legacy: a.wiki_legacy } };
    if a.wiki_legacy {
        tools.opts.wiki_legacy = true;
        tools.design_basis_wiki = trevor::reference::wiki_design_basis(std::path::Path::new("."));
    }
    tools
}

#[cfg(test)]
mod tests;
