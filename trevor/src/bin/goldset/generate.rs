//! `generate`: one question per sampled passage from the generator model, under its grammar.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use futures::StreamExt as _;
use trevor::eval::goldset::Stratum;
use trevor::goldgen::llm::{Llm, Request};
use trevor::goldgen::sample::SampleItem;
use trevor::goldgen::{Generated, self as gg, sha16};
use trevor::search::store::ChunkStore;

use crate::files::{append, read_jsonl};
use crate::prompt::{load_names, user_prompt};

/// Generation knobs. Every one is off by default, and off reproduces gold
/// set v1 exactly: the same grammar bytes (so the same `grammarSha`), the same
/// temperature, and rows without a `temperature` key.
#[derive(clap::Args, Clone, Debug, Default)]
pub(crate) struct GenArgs {
    /// Sampling temperature; absent means v1's 0.7.
    #[arg(long)]
    pub(crate) temperature: Option<f32>,
    /// Cap on evidence-quote length in characters; absent means v1's 300.
    #[arg(long)]
    pub(crate) evidence_max: Option<usize>,
    /// Forbid line breaks inside an evidence quote.
    #[arg(long)]
    pub(crate) evidence_one_line: bool,
}

pub(crate) async fn cmd_generate(work: &Path, server: &str, corpus: &Path, knobs: &GenArgs) -> Result<()> {
    let grammar_single = gg::evidence_grammar(gg::GRAMMAR_SINGLE, knobs.evidence_max, knobs.evidence_one_line)?;
    let grammar_multi = gg::evidence_grammar(gg::GRAMMAR_MULTI, knobs.evidence_max, knobs.evidence_one_line)?;
    let temperature = knobs.temperature.unwrap_or(gg::V1_TEMPERATURE);
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let names = load_names(work);
    let items: Vec<SampleItem> = read_jsonl(&work.join("sample.jsonl"))?;
    let out = work.join("generated.jsonl");
    let done: BTreeSet<String> = read_jsonl::<Generated>(&out)?
        .into_iter()
        .map(|g| g.id)
        .collect();
    let todo: Vec<&SampleItem> = items.iter().filter(|i| !done.contains(&i.id)).collect();
    eprintln!(
        "generate: {} to do, {} done, model {} ({} slots)",
        todo.len(),
        done.len(),
        llm.model,
        llm.slots
    );
    let started = std::time::Instant::now();
    let mut stream = futures::stream::iter(todo.into_iter().map(|item| {
        let llm = llm.clone();
        let user = user_prompt(item, &store, &names);
        let (grammar_single, grammar_multi) = (&grammar_single, &grammar_multi);
        async move {
            let user = user?;
            let multi = item.stratum == Stratum::MultiHop;
            let (system, grammar) = if multi {
                (gg::SYSTEM_MULTI, grammar_multi.as_str())
            } else {
                (gg::SYSTEM_SINGLE, grammar_single.as_str())
            };
            let c = llm
                .complete(&Request {
                    system,
                    user: &user,
                    grammar: Some(grammar),
                    seed: item.seed,
                    temperature,
                    n_predict: if multi { 700 } else { 400 },
                    stop: &[],
                })
                .await?;
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(c.content.trim());
            let (mut question, mut answerable, mut evidence, mut parse_error) =
                (None, false, Vec::new(), None);
            match parsed {
                Ok(v) => {
                    question = v["question"].as_str().map(str::to_owned);
                    answerable = v[if multi {
                        "answerable_from_both"
                    } else {
                        "answerable_from_passage"
                    }]
                    .as_bool()
                    .unwrap_or(false);
                    for k in if multi {
                        &["evidence_quote_a", "evidence_quote_b"][..]
                    } else {
                        &["evidence_quote"][..]
                    } {
                        evidence.push(v[*k].as_str().unwrap_or("").to_owned());
                    }
                }
                Err(e) => parse_error = Some(format!("{e} (stop_type {})", c.stop_type)),
            }
            anyhow::Ok(Generated {
                id: item.id.clone(),
                stratum: item.stratum,
                chunk_ids: item.chunk_ids.clone(),
                entity: item.entity.clone(),
                question,
                answerable,
                evidence,
                stop_type: c.stop_type,
                parse_error,
                model: llm.model.clone(),
                prompt_sha: sha16(system),
                grammar_sha: sha16(grammar),
                seed: item.seed,
                ms: c.ms,
                temperature: knobs.temperature,
            })
        }
    }))
    .buffer_unordered(llm.slots);
    let mut n = 0usize;
    while let Some(r) = stream.next().await {
        let g = r?;
        append(&out, std::slice::from_ref(&g))?;
        n += 1;
        if n % 20 == 0 {
            eprintln!(
                "  {n} generated, {:.1}/min",
                n as f64 / started.elapsed().as_secs_f64() * 60.0
            );
        }
    }
    eprintln!(
        "generate: {n} new in {:.0}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
