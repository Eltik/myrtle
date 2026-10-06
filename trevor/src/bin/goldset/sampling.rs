//! `sample` (draw source passages) and `probe` (render one prompt and run it).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use anyhow::{Context, Result};
use trevor::eval::goldset::Stratum;
use trevor::goldgen::llm::{Llm, Request};
use trevor::goldgen::sample::{self, SampleItem};
use trevor::goldgen::{self as gg};
use trevor::search::store::ChunkStore;

use crate::files::{append, read_jsonl};
use crate::prompt::{fetch_names, load_names, user_prompt};

/// Generation requests per stratum: roughly 1.7 to 2.5 times the quota,
/// more where the passage is less likely to support the type.
pub(crate) const GENERATE: [(Stratum, usize); 5] = [
    (Stratum::SingleFact, 60),
    (Stratum::Entity, 45),
    (Stratum::Causal, 40),
    (Stratum::Temporal, 35),
    (Stratum::Aggregation, 25),
];
pub(crate) const GENERATE_MULTI: usize = 75;
pub(crate) async fn cmd_sample(work: &Path, corpus: &Path, base: &str, scale: f64, seed: u64) -> Result<()> {
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let spoiler: Vec<serde_json::Value> = read_jsonl(&corpus.join("spoiler.jsonl"))?;
    let category: HashMap<String, String> = spoiler
        .iter()
        .filter_map(|s| {
            Some((
                s["storyId"].as_str()?.to_owned(),
                s["category"].as_str().unwrap_or("unknown").to_owned(),
            ))
        })
        .collect();
    let names_path = work.join("names.json");
    if !names_path.exists() {
        match fetch_names(base).await {
            Ok((n, ops)) => {
                std::fs::write(&names_path, serde_json::to_vec(&n)?)?;
                std::fs::write(work.join("operators.json"), serde_json::to_vec(&ops)?)?;
                eprintln!(
                    "names: {} stories, {} operators from {base}",
                    n.len(),
                    ops.len()
                );
            }
            Err(e) => eprintln!("names: backend unavailable ({e}); prompts will use story ids"),
        }
    }
    let path = work.join("sample.jsonl");
    let existing: Vec<SampleItem> = read_jsonl(&path)?;
    let round = existing
        .iter()
        .filter_map(|s| {
            s.id.strip_prefix('r')?
                .split('-')
                .next()?
                .parse::<u32>()
                .ok()
        })
        .max()
        .map_or(1, |r| r + 1);
    let exclude: BTreeSet<String> = existing
        .iter()
        .flat_map(|s| s.chunk_ids.iter().cloned())
        .collect();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let sc = |n: usize| ((n as f64) * scale).round() as usize;
    let counts: Vec<(Stratum, usize)> = GENERATE.iter().map(|&(s, n)| (s, sc(n))).collect();
    let mut items = sample::single(&store, &category, &counts, round, seed, &exclude);
    let operators: BTreeSet<String> = std::fs::read(work.join("operators.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    items.extend(sample::multi_hop(
        &store,
        &operators,
        sc(GENERATE_MULTI),
        round,
        seed,
        &exclude,
    ));
    append(&path, &items)?;
    let mut by: BTreeMap<&str, usize> = BTreeMap::new();
    for i in &items {
        *by.entry(i.stratum.name()).or_default() += 1;
    }
    println!("round {round}: {} items {by:?}", items.len());
    Ok(())
}

pub(crate) async fn cmd_probe(work: &Path, server: &str, corpus: &Path) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let names = load_names(work);
    let items: Vec<SampleItem> = read_jsonl(&work.join("sample.jsonl"))?;
    let item = items.first().context("run sample first")?;
    let user = user_prompt(item, &store, &names)?;
    let rendered = llm.render(gg::SYSTEM_SINGLE, &user).await?;
    println!(
        "model {} ({} slots)\n--- rendered prompt tail ---\n{}",
        llm.model,
        llm.slots,
        &rendered[rendered.len().saturating_sub(400)..]
    );
    let c = llm
        .complete(&Request {
            system: gg::SYSTEM_SINGLE,
            user: &user,
            grammar: Some(gg::GRAMMAR_SINGLE),
            seed: item.seed,
            temperature: 0.7,
            n_predict: 400,
            stop: &[],
        })
        .await?;
    println!(
        "--- output ({}, {} prompt tokens, {} predicted, {:.0} ms) ---\n{}",
        c.stop_type, c.prompt_tokens, c.predicted_tokens, c.ms, c.content
    );
    let v = llm
        .verdict(
            gg::JUDGE_SELF_CONTAINED,
            "QUESTION: Why does Ch'en leave the Lungmen Guard Department?",
            gg::GRAMMAR_VERDICT,
        )
        .await?;
    println!("--- verdict probe (expect true): {v}");
    let v = llm
        .verdict(
            gg::JUDGE_SELF_CONTAINED,
            "QUESTION: What does the speaker say about it here?",
            gg::GRAMMAR_VERDICT,
        )
        .await?;
    println!("--- verdict probe (expect false): {v}");
    Ok(())
}
