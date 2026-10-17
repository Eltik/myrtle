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
mod kinds;
mod labels;
mod people;
mod retrieval;
mod runtimes;
mod routing;
mod answer;
mod batch;
mod llama_server;
mod serve;
mod guard;

use std::path::PathBuf;

use anyhow::{Context, Result};
use trevor::llm::Llm;
use trevor::tools::Tools;

use crate::answer::{Tables, answer_one};
use crate::chrono::load_chrono;
use crate::cli::{Args, LoreSet, P3B_DIR, RetrievalKind, RouterKind, apply_profile, deep_for, p4_dir, retrieval_plan,
                 router_source, tool_options};
use crate::detect::source_kind;
use crate::labels::load_names;
use crate::runtimes::{Runtimes, load_runtime};
use crate::routing::{print_route_only, route_question};
use crate::scenes::line_study;
use crate::llama_server::{Spawned, connect_or_spawn};

#[tokio::main]
async fn main() -> Result<()> {
    // `ask` answers from P3b (P3a plus the 200 dossiers and 88 event summaries) unless --corpus is given; search and
    // eval keep P3a (design/trevor-retrieval-baseline.md has the measurements).
    let m = <Args as clap::CommandFactory>::command().get_matches();
    let mut a = <Args as clap::FromArgMatches>::from_arg_matches(&m)?;
    let corpus_default = m.value_source("corpus") == Some(clap::parser::ValueSource::DefaultValue);
    // RELATION_RULE is the default (`--no-relation-rule`); `answer_one_inner` narrows it per question.
    a.relation_rule = !a.no_relation_rule;
    if m.value_source("retrieval") == Some(clap::parser::ValueSource::DefaultValue)
        && trevor::env::is("TREVOR_RETRIEVAL", "bm25") {
        a.retrieval = RetrievalKind::Bm25;
    }
    let profile = a.profile;
    apply_profile(&mut a, profile);
    if a.gate_order {
        let f = trevor::search::horizon::ReleaseOrder::write_file(std::path::Path::new("."))?;
        println!("{}: {} stories ({} never visible, {} placed by date), {} dated sources, {} eras",
                 trevor::search::horizon::GATE_ORDER_PATH, f.story_order.len(), f.never.len(), f.dated.len(),
                 f.source_pos.len(), f.eras.len());
        return Ok(());
    }
    if let (Some(b), true) = (&a.batch, a.horizon_check) {
        return serve::horizon_check(&a, b);
    }
    if corpus_default {
        // P4 (P3b plus the module stories, voice lines, IS, enemies, outfits and items) only for a question that names
        // one of those sources (`source_kind`); with the model router naming the source, the corpus follows its route
        // instead (set after routing).
        let source_q = !router_source(&a) && a.batch.is_none()
            && a.question.as_deref().is_some_and(|q| source_kind(q).is_some());
        let (p3b, p4) = (P3B_DIR, p4_dir(&a));
        a.runtime.corpus = PathBuf::from(if source_q { p4 } else { p3b });
    }
    let chrono = load_chrono();
    let tools = load_tools(&a);
    if a.show_expansion {
        return batch::show_expansion(&a, &tools);
    }
    if a.form_only {
        return batch::form_only(&a, &tools).await;
    }
    if let Some(d) = &a.identity_scan {
        return batch::identity_scan(&tools, d);
    }
    let mut tables = Tables { tools, topic: None, route: None };
    if let (Some(b), true) = (&a.batch, a.json_check) {
        return serve::json_check(&a, &mut tables, b).await;
    }
    if a.canary {
        return serve::run_canary(a, corpus_default, tables, chrono).await;
    }
    if a.serve || a.worker.is_some() {
        return serve::run(a, corpus_default, tables, chrono).await;
    }
    // Table-routed questions need no index: answer them before loading it. The keyword and off routers need no model
    // either, so they answer before starting or connecting to a server.
    let mut early: Option<(Llm, Option<Spawned>)> = None;
    if let (Some(b), true) = (&a.batch, a.detect_only) {
        return batch::detect_only(&a, &tables.tools, b);
    }
    if let (Some(b), true, false) = (&a.batch, a.route_only, a.no_route) {
        return batch::route_only(&a, &mut tables, b).await;
    }
    if let (None, Some(q), false) = (&a.batch, &a.question, a.no_route) {
        let llm = if a.router == RouterKind::Model { Some(connect_or_spawn(&a).await?) } else { None };
        tables.tools.topics_deep = deep_for(&a, &tables.tools, q);
        let r = route_question(q, &a, &tables.tools, llm.as_ref().map(|x| &x.0)).await?;
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
    let dense = a.retrieval == RetrievalKind::Hybrid;
    let mut rts = Runtimes { main: load_runtime(&a.runtime, dense, None)?, p4: None, art: None, p4_dir: p4_dir(&a),
                             switch: corpus_default && router_source(&a), args: a.runtime.clone(), horizon: None,
                             dense };
    if let (Some(b), true) = (&a.batch, a.line_study) {
        return line_study(&mut rts.main, &rts.args, b);
    }
    if let (Some(b), true) = (&a.batch, a.scoped_only) {
        return batch::scoped_only(&mut rts, &tables.tools, b);
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
        tools.topics_race_rule = !a.no_race_trait_rule;
        // Deep entries are switched on per question (`deep_for`, the broad-question gate).
        tools.topics_deep = false;
    }
    tools.opts = tool_options(a);
    tools
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_worker;
