//! Build the retrieval gold set with local models and no manual review.
//!
//! Stages, each resumable and each writing one file under `--work`:
//!
//! ```text
//! sample        backend index + chunks        -> sample.jsonl, names.json
//! generate      generator server (Gemma)      -> generated.jsonl
//! filter        BM25 + embedder, no LLM       -> filtered.jsonl
//! closed-book   generator server              -> closed_book.jsonl
//! judge         judge server (Qwen)           -> judged.jsonl
//! finish        judge server + indexes        -> eval/goldset_v1.jsonl, .meta.json, .spotcheck.md
//! ```

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use futures::StreamExt as _;
use serde::{Serialize, de::DeserializeOwned};
use trevor::eval::goldset::{Anchor, ClosedBook, GoldItem, Review, Stratum};
use trevor::goldgen::filters::{deictic, fnv1a, quote_in_passage, trigram_jaccard, verbatim_prefix};
use trevor::goldgen::llm::{Llm, Request};
use trevor::goldgen::sample::{self, SampleItem};
use trevor::goldgen::{
    self as gg, ClosedBookAnswer, Filtered, Generated, Judged, containment, sha16,
};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::search::store::ChunkStore;

#[derive(Parser)]
#[command(about = "Generate the Trevor retrieval gold set with local models")]
struct Args {
    /// Directory for the intermediate files.
    #[arg(long, default_value = "artifacts/goldgen", global = true)]
    work: PathBuf,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Draw source passages. Appends a new round if sample.jsonl exists.
    Sample {
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        /// Backend, for story and group names.
        #[arg(long, default_value = "http://127.0.0.1:3060")]
        base: String,
        #[arg(long, default_value_t = 1.0)]
        scale: f64,
        #[arg(long, default_value_t = 20_260_925)]
        seed: u64,
    },
    /// Render one prompt and run it, to check a server and its template.
    Probe {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
    },
    Generate {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        #[command(flatten)]
        knobs: GenArgs,
    },
    Filter {
        #[command(flatten)]
        runtime: RuntimeArgs,
        /// Accept an evidence quote that is not verbatim in its passage when
        /// its verbatim prefix has at least this many characters, and carry
        /// the prefix forward as the evidence. Absent means v1: whole quote
        /// or reject.
        #[arg(long)]
        evidence_min_prefix: Option<usize>,
    },
    ClosedBook {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
    },
    Judge {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
    },
    /// Measure the judge itself: its verdicts on passages known NOT to
    /// answer a question, and on questions known to be bad or good.
    Calibrate {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[arg(long, default_value = "artifacts")]
        corpus: PathBuf,
        #[arg(long, default_value_t = 60)]
        n: usize,
    },
    Finish {
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        server: String,
        #[command(flatten)]
        runtime: RuntimeArgs,
        #[arg(long, default_value = "eval/goldset_v1")]
        out: PathBuf,
        /// Passages taken from each retriever for anchor completion.
        #[arg(long, default_value_t = 5)]
        pool_depth: usize,
    },
}

/// Generation knobs. Every one is off by default, and off reproduces gold
/// set v1 exactly: the same grammar bytes (so the same `grammarSha`), the same
/// temperature, and rows without a `temperature` key.
#[derive(clap::Args, Clone, Debug, Default)]
struct GenArgs {
    /// Sampling temperature; absent means v1's 0.7.
    #[arg(long)]
    temperature: Option<f32>,
    /// Cap on evidence-quote length in characters; absent means v1's 300.
    #[arg(long)]
    evidence_max: Option<usize>,
    /// Forbid line breaks inside an evidence quote.
    #[arg(long)]
    evidence_one_line: bool,
}

/// Final set size per stratum. Ambiguous-entity (10) and spoiler-boundary
/// (5) items need hand construction and are deferred; unanswerable items
/// come from prompts/unanswerable.txt.
const QUOTAS: [(Stratum, usize); 6] = [
    (Stratum::SingleFact, 35),
    (Stratum::Entity, 25),
    (Stratum::Causal, 20),
    (Stratum::Temporal, 15),
    (Stratum::Aggregation, 10),
    (Stratum::MultiHop, 30),
];
const UNANSWERABLE_QUOTA: usize = 15;
/// Generation requests per stratum: roughly 1.7 to 2.5 times the quota,
/// more where the passage is less likely to support the type.
const GENERATE: [(Stratum, usize); 5] = [
    (Stratum::SingleFact, 60),
    (Stratum::Entity, 45),
    (Stratum::Causal, 40),
    (Stratum::Temporal, 35),
    (Stratum::Aggregation, 25),
];
const GENERATE_MULTI: usize = 75;
/// Above this the eval measures pretraining, not retrieval (eval spec 2.4).
const CLOSED_BOOK_GATE: f64 = 0.30;
/// Word-trigram containment above which a question copies its passage.
const CONTAINMENT_MAX: f64 = 0.30;
const DEDUP_COSINE: f32 = 0.95;

fn read_jsonl<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let Ok(f) = std::fs::File::open(path) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (i, line) in std::io::BufReader::new(f).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(
            serde_json::from_str(&line)
                .with_context(|| format!("{} line {}", path.display(), i + 1))?,
        );
    }
    Ok(out)
}

fn append<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    for r in rows {
        writeln!(f, "{}", serde_json::to_string(r)?)?;
    }
    Ok(())
}

fn write_jsonl<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp)?);
    for r in rows {
        writeln!(f, "{}", serde_json::to_string(r)?)?;
    }
    f.flush()?;
    drop(f);
    std::fs::rename(tmp, path)?;
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Names {
    story: String,
    group: String,
}

fn load_names(work: &Path) -> HashMap<String, Names> {
    std::fs::read(work.join("names.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn label(names: &HashMap<String, Names>, story_id: &str) -> String {
    match names.get(story_id) {
        Some(n) if !n.story.is_empty() => format!("{} ({})", n.story, n.group),
        _ => story_id.to_owned(),
    }
}

fn user_prompt(
    item: &SampleItem,
    store: &ChunkStore,
    names: &HashMap<String, Names>,
) -> Result<String> {
    let chunk = |id: &str| -> Result<&trevor::corpus::chunk::Chunk> {
        Ok(&store.chunks[store
            .row(id)
            .with_context(|| format!("{id} not in chunks.jsonl"))?])
    };
    if item.stratum == Stratum::MultiHop {
        let (a, b) = (chunk(&item.chunk_ids[0])?, chunk(&item.chunk_ids[1])?);
        Ok(format!(
            "CHARACTER IN BOTH: {}\n\nPASSAGE A (from {}):\n{}\n\nPASSAGE B (from {}):\n{}",
            item.entity.as_deref().unwrap_or(""),
            label(names, &a.story_id),
            a.text,
            label(names, &b.story_id),
            b.text
        ))
    } else {
        let c = chunk(&item.chunk_ids[0])?;
        Ok(format!(
            "STORY: {}\nQUESTION TYPE: {}\n\nPASSAGE:\n{}",
            label(names, &c.story_id),
            item.stratum.name(),
            c.text
        ))
    }
}

async fn fetch_names(base: &str) -> Result<(HashMap<String, Names>, BTreeSet<String>)> {
    let v: serde_json::Value = reqwest::get(format!("{base}/api/story/index"))
        .await?
        .error_for_status()?
        .json()
        .await?;
    let mut out = HashMap::new();
    let operators: BTreeSet<String> = v["records"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["name"].as_str().map(str::to_owned))
        .collect();
    for g in v["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(v["records"].as_array().into_iter().flatten())
    {
        let group = g["name"]
            .as_str()
            .or_else(|| g["charName"].as_str())
            .unwrap_or("")
            .to_owned();
        for s in g["stories"].as_array().into_iter().flatten() {
            if let Some(id) = s["id"].as_str() {
                out.entry(id.to_owned()).or_insert_with(|| Names {
                    story: s["name"].as_str().unwrap_or("").to_owned(),
                    group: group.clone(),
                });
            }
        }
    }
    Ok((out, operators))
}

#[tokio::main]
async fn main() -> Result<()> {
    let a = Args::parse();
    std::fs::create_dir_all(&a.work)?;
    let work = a.work.clone();
    match a.cmd {
        Cmd::Sample {
            corpus,
            base,
            scale,
            seed,
        } => cmd_sample(&work, &corpus, &base, scale, seed).await,
        Cmd::Probe { server, corpus } => cmd_probe(&work, &server, &corpus).await,
        Cmd::Generate {
            server,
            corpus,
            knobs,
        } => cmd_generate(&work, &server, &corpus, &knobs).await,
        Cmd::Filter {
            runtime,
            evidence_min_prefix,
        } => cmd_filter(&work, &runtime, evidence_min_prefix),
        Cmd::ClosedBook { server } => cmd_closed_book(&work, &server).await,
        Cmd::Judge { server, corpus } => cmd_judge(&work, &server, &corpus).await,
        Cmd::Calibrate { server, corpus, n } => cmd_calibrate(&work, &server, &corpus, n).await,
        Cmd::Finish {
            server,
            runtime,
            out,
            pool_depth,
        } => cmd_finish(&work, &server, &runtime, &out, pool_depth).await,
    }
}

async fn cmd_sample(work: &Path, corpus: &Path, base: &str, scale: f64, seed: u64) -> Result<()> {
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

async fn cmd_probe(work: &Path, server: &str, corpus: &Path) -> Result<()> {
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

async fn cmd_generate(work: &Path, server: &str, corpus: &Path, knobs: &GenArgs) -> Result<()> {
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

fn cmd_filter(work: &Path, runtime: &RuntimeArgs, min_prefix: Option<usize>) -> Result<()> {
    let mut rt = Runtime::load(runtime, true, true, false)?;
    let gens: Vec<Generated> = read_jsonl(&work.join("generated.jsonl"))?;
    let bm25_cfg = RetrievalConfig {
        mode: Mode::Bm25,
        k: 100,
        ..RetrievalConfig::default()
    };
    let mut out: Vec<Filtered> = Vec::new();
    let mut kept_vecs: Vec<(String, Vec<f32>)> = Vec::new();
    let mut sorted: Vec<&Generated> = gens.iter().collect();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    for g in sorted {
        let mut reasons = Vec::new();
        let passages: Vec<String> = g
            .chunk_ids
            .iter()
            .map(|id| {
                rt.store
                    .row(id)
                    .map(|r| rt.store.chunks[r].text.clone())
                    .unwrap_or_default()
            })
            .collect();
        let q = g.question.clone().unwrap_or_default();
        if let Some(e) = &g.parse_error {
            reasons.push(format!("unparsed: {e}"));
        }
        if !g.answerable {
            reasons.push("generator abstained".into());
        }
        let mut grounded = g.evidence.clone();
        let mut rescued = false;
        for (i, (quote, passage)) in g.evidence.iter().zip(&passages).enumerate() {
            if quote_in_passage(quote, passage) {
                continue;
            }
            let prefix = min_prefix
                .map(|_| verbatim_prefix(quote, passage))
                .filter(|p| min_prefix.is_some_and(|n| p.chars().count() >= n));
            if let Some(p) = prefix {
                grounded[i] = p;
                rescued = true;
            } else {
                reasons.push(format!("evidence {i} not in passage"));
            }
        }
        if let Some(d) = deictic(&q) {
            reasons.push(format!("deictic: {d}"));
        }
        let joined = passages.join("\n");
        let cont = containment(&q, &joined);
        if cont > CONTAINMENT_MAX {
            reasons.push(format!("copies passage wording ({cont:.2})"));
        }
        let mut bm25_rank = None;
        if !q.is_empty() {
            let hits = rt.retrieve(&q, &bm25_cfg)?;
            let gold: BTreeSet<usize> = g
                .chunk_ids
                .iter()
                .filter_map(|id| rt.store.row(id))
                .collect();
            bm25_rank = hits
                .iter()
                .position(|h| gold.contains(&h.row))
                .map(|p| p + 1);
            if g.stratum != Stratum::MultiHop && bm25_rank == Some(1) {
                reasons.push("BM25 ranks the source first".into());
            }
        }
        let mut dup_of = None;
        if reasons.is_empty() {
            let v = rt.embed_query(&q)?;
            if let Some((other, _)) = kept_vecs
                .iter()
                .find(|(_, u)| u.iter().zip(&v).map(|(a, b)| a * b).sum::<f32>() > DEDUP_COSINE)
            {
                dup_of = Some(other.clone());
                reasons.push(format!("near-duplicate of {other}"));
            } else {
                kept_vecs.push((g.id.clone(), v));
            }
        }
        out.push(Filtered {
            id: g.id.clone(),
            pass: reasons.is_empty(),
            reasons,
            word_trigram_containment: cont,
            char_trigram_jaccard: trigram_jaccard(&q, &joined),
            bm25_rank,
            dup_of,
            grounded_evidence: rescued.then_some(grounded),
        });
    }
    write_jsonl(&work.join("filtered.jsonl"), &out)?;
    let mut why: BTreeMap<String, usize> = BTreeMap::new();
    for f in &out {
        if let Some(r) = f.reasons.first() {
            let key = r.split([':', '(']).next().unwrap_or(r).trim().to_owned();
            *why.entry(key).or_default() += 1;
        }
    }
    let pass = out.iter().filter(|f| f.pass).count();
    println!(
        "filter: {pass} of {} pass; first failing reason: {why:?}",
        out.len()
    );
    let single: Vec<&Filtered> = out
        .iter()
        .filter(|f| {
            !f.id.contains("multi_hop") && gens.iter().any(|g| g.id == f.id && g.question.is_some())
        })
        .collect();
    let rank1 = single.iter().filter(|f| f.bm25_rank == Some(1)).count();
    println!(
        "  BM25 ranked the source first for {rank1} of {} single-passage questions before filtering",
        single.len()
    );
    Ok(())
}

async fn cmd_closed_book(work: &Path, server: &str) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let gens: HashMap<String, Generated> = read_jsonl::<Generated>(&work.join("generated.jsonl"))?
        .into_iter()
        .map(|g| (g.id.clone(), g))
        .collect();
    let passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass)
        .collect();
    let out = work.join("closed_book.jsonl");
    let done: BTreeSet<String> = read_jsonl::<ClosedBookAnswer>(&out)?
        .into_iter()
        .map(|c| c.id)
        .collect();
    let todo: Vec<&Generated> = passing
        .iter()
        .filter(|f| !done.contains(&f.id))
        .filter_map(|f| gens.get(&f.id))
        .collect();
    eprintln!("closed-book: {} to do with {}", todo.len(), llm.model);
    let mut stream = futures::stream::iter(todo.into_iter().map(|g| {
        let llm = llm.clone();
        async move {
            let q = g.question.clone().unwrap_or_default();
            let c = llm
                .complete(&Request {
                    system: gg::CLOSED_BOOK,
                    user: &format!("QUESTION: {q}"),
                    grammar: None,
                    seed: 0,
                    temperature: 0.0,
                    n_predict: 96,
                    stop: &[],
                })
                .await?;
            anyhow::Ok(ClosedBookAnswer {
                id: g.id.clone(),
                // The first non-empty line: some templates open with a newline.
                answer: c
                    .content
                    .lines()
                    .map(str::trim)
                    .find(|l| !l.is_empty())
                    .unwrap_or("")
                    .to_owned(),
                model: llm.model.clone(),
            })
        }
    }))
    .buffer_unordered(llm.slots);
    let mut n = 0;
    while let Some(r) = stream.next().await {
        append(&out, &[r?])?;
        n += 1;
    }
    eprintln!("closed-book: {n} answered");
    Ok(())
}

fn passage_prompt(passage: &str, question: &str) -> String {
    format!("PASSAGE:\n{passage}\n\nQUESTION: {question}")
}

async fn cmd_judge(work: &Path, server: &str, corpus: &Path) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let gens: HashMap<String, Generated> = read_jsonl::<Generated>(&work.join("generated.jsonl"))?
        .into_iter()
        .map(|g| (g.id.clone(), g))
        .collect();
    let cb: HashMap<String, String> =
        read_jsonl::<ClosedBookAnswer>(&work.join("closed_book.jsonl"))?
            .into_iter()
            .map(|c| (c.id, c.answer))
            .collect();
    let passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass)
        .collect();
    let out = work.join("judged.jsonl");
    let done: BTreeSet<String> = read_jsonl::<Judged>(&out)?
        .into_iter()
        .map(|j| j.id)
        .collect();
    let todo: Vec<(&Generated, Vec<String>)> = passing
        .iter()
        .filter(|f| !done.contains(&f.id))
        .filter_map(|f| gens.get(&f.id).map(|g| (g, f.evidence(g))))
        .collect();
    if gens.values().any(|g| g.model == llm.model) {
        bail!(
            "the judge server runs {}, the model that generated the questions; start the judge model instead",
            llm.model
        );
    }
    eprintln!("judge: {} to do with {}", todo.len(), llm.model);
    let started = std::time::Instant::now();
    let store = &store;
    let cb = &cb;
    let mut stream = futures::stream::iter(todo.into_iter().map(|(g, evidence)| {
        let llm = llm.clone();
        async move {
            let q = g.question.clone().unwrap_or_default();
            let text = |i: usize| store.row(&g.chunk_ids[i]).map(|r| store.chunks[r].text.clone()).unwrap_or_default();
            let mut j = Judged { id: g.id.clone(), model: llm.model.clone(), ..Judged::default() };
            j.self_contained = llm.verdict(gg::JUDGE_SELF_CONTAINED, &format!("QUESTION: {q}"), gg::GRAMMAR_VERDICT).await?;
            if g.chunk_ids.len() == 2 {
                let (a, b) = (text(0), text(1));
                j.answerable = llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&format!("{a}\n\n{b}"), &q), gg::GRAMMAR_VERDICT).await?;
                j.a_alone = Some(llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&a, &q), gg::GRAMMAR_VERDICT).await?);
                j.b_alone = Some(llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&b, &q), gg::GRAMMAR_VERDICT).await?);
            } else {
                j.answerable = llm.verdict(gg::JUDGE_ANSWERABLE, &passage_prompt(&text(0), &q), gg::GRAMMAR_VERDICT).await?;
            }
            if let Some(ans) = cb.get(&g.id) {
                j.closed_book_correct = Some(if ans.to_lowercase().contains("don't know") || ans.is_empty() {
                    false
                } else {
                    llm.verdict(
                        gg::JUDGE_CLOSED_BOOK,
                        &format!("QUESTION: {q}\n\nREFERENCE EVIDENCE:\n{}\n\nCANDIDATE ANSWER: {ans}", evidence.join("\n")),
                        gg::GRAMMAR_VERDICT,
                    )
                    .await?
                });
            }
            anyhow::Ok(j)
        }
    }))
    .buffer_unordered(llm.slots);
    let mut n = 0usize;
    while let Some(r) = stream.next().await {
        append(&out, &[r?])?;
        n += 1;
        if n % 25 == 0 {
            eprintln!(
                "  {n} judged, {:.1}/min",
                n as f64 / started.elapsed().as_secs_f64() * 60.0
            );
        }
    }
    eprintln!(
        "judge: {n} judged in {:.0}s",
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

/// Questions whose self-containedness is known, for calibrating the judge.
const BAD_QUESTIONS: [&str; 10] = [
    "What does he want from her in this scene?",
    "Why does the speaker refuse the offer?",
    "What happens right after that?",
    "Who is being talked about in the passage?",
    "What did they decide to do about it?",
    "Why is she so upset here?",
    "What does the character reveal at the end of the conversation?",
    "What is the reason for his anger in the text above?",
    "Where are they going next?",
    "What does it do when the others arrive?",
];
const GOOD_QUESTIONS: [&str; 10] = [
    "Why does Ch'en leave the Lungmen Guard Department?",
    "Who does the Duke of Caster think might be Victoria's chosen one?",
    "What final wish does Faust leave Mephisto with?",
    "Where does the Doctor decide to take young Amiya for her Oripathy treatment?",
    "What was forced into Specter's spinal cord?",
    "Who serves as Siesta's Catastrophe Messenger?",
    "Why does Kal'tsit warn Nightingale to stop while she is suppressing the Confessarius?",
    "What names is Sciurus considering for her baby?",
    "Which Columbian law about the Infected is used to pressure the mayor of Siesta?",
    "What did Hoshiguma decide about the former Reunion member who now sells soda in Lungmen?",
];

#[derive(serde::Serialize, serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Calibration {
    judge: String,
    random_passage_trials: usize,
    random_passage_false_positives: usize,
    random_passage_fp_wilson95: (f64, f64),
    source_passage_trials: usize,
    source_passage_true: usize,
    bad_questions_passed: usize,
    good_questions_failed: usize,
}

async fn cmd_calibrate(work: &Path, server: &str, corpus: &Path, n: usize) -> Result<()> {
    let llm = Llm::connect(server).await?;
    let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
    let gens: HashMap<String, Generated> = read_jsonl::<Generated>(&work.join("generated.jsonl"))?
        .into_iter()
        .map(|g| (g.id.clone(), g))
        .collect();
    let mut passing: Vec<Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .filter(|f| f.pass && !f.id.contains("multi_hop"))
        .collect();
    passing.sort_by_key(|f| fnv1a(&format!("cal{}", f.id)));
    let items: Vec<&Generated> = passing
        .iter()
        .take(n)
        .filter_map(|f| gens.get(&f.id))
        .collect();
    let (mut fp, mut src_true) = (0, 0);
    for g in &items {
        let q = g.question.clone().unwrap_or_default();
        let src_row = store.row(&g.chunk_ids[0]).context("source chunk missing")?;
        let src_group = store.chunks[src_row].group_id.clone();
        // A random passage from another group cannot answer a question
        // written from this one, so every "true" here is a false positive.
        let mut rng = sample::Rng(fnv1a(&g.id));
        let other = loop {
            let r = rng.below(store.len());
            if store.chunks[r].group_id != src_group
                && store.chunks[r].token_count >= sample::MIN_TOKENS
            {
                break r;
            }
        };
        if llm
            .verdict(
                gg::JUDGE_ANSWERABLE,
                &passage_prompt(&store.chunks[other].text, &q),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            fp += 1;
        }
        if llm
            .verdict(
                gg::JUDGE_ANSWERABLE,
                &passage_prompt(&store.chunks[src_row].text, &q),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            src_true += 1;
        }
    }
    let mut bad_passed = 0;
    for q in BAD_QUESTIONS {
        if llm
            .verdict(
                gg::JUDGE_SELF_CONTAINED,
                &format!("QUESTION: {q}"),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            bad_passed += 1;
        }
    }
    let mut good_failed = 0;
    for q in GOOD_QUESTIONS {
        if !llm
            .verdict(
                gg::JUDGE_SELF_CONTAINED,
                &format!("QUESTION: {q}"),
                gg::GRAMMAR_VERDICT,
            )
            .await?
        {
            good_failed += 1;
        }
    }
    let cal = Calibration {
        judge: llm.model.clone(),
        random_passage_trials: items.len(),
        random_passage_false_positives: fp,
        random_passage_fp_wilson95: trevor::eval::stats::wilson(
            fp,
            items.len(),
            trevor::eval::stats::Z95,
        ),
        source_passage_trials: items.len(),
        source_passage_true: src_true,
        bad_questions_passed: bad_passed,
        good_questions_failed: good_failed,
    };
    std::fs::write(
        work.join("calibration.json"),
        serde_json::to_vec_pretty(&cal)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&cal)?);
    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    judge_calibration: Option<Calibration>,
    goldset_version: &'static str,
    built_at_unix: u64,
    n: usize,
    strata: BTreeMap<String, usize>,
    shortfall: BTreeMap<String, usize>,
    deferred: Vec<&'static str>,
    funnel: BTreeMap<String, usize>,
    closed_book_rate_candidates: f64,
    closed_book_rate_selected: f64,
    closed_book_remediated: bool,
    bm25_rank1_share_selected_single: f64,
    anchors_added: usize,
    items_with_added_anchors: usize,
    unanswerable_dropped: Vec<String>,
    generator: String,
    judge: String,
    prompt_shas: BTreeMap<&'static str, String>,
    /// Rows per `grammarSha@temperature`, present only when some row was
    /// generated with a knob that differs from v1.
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_variants: Option<BTreeMap<String, usize>>,
    review: &'static str,
}

async fn cmd_finish(
    work: &Path,
    server: &str,
    runtime: &RuntimeArgs,
    out: &Path,
    pool_depth: usize,
) -> Result<()> {
    let judge = Llm::connect(server).await?;
    let mut rt = Runtime::load(runtime, true, true, false)?;
    let gens: Vec<Generated> = read_jsonl(&work.join("generated.jsonl"))?;
    let filtered: HashMap<String, Filtered> = read_jsonl::<Filtered>(&work.join("filtered.jsonl"))?
        .into_iter()
        .map(|f| (f.id.clone(), f))
        .collect();
    let judged: HashMap<String, Judged> = read_jsonl::<Judged>(&work.join("judged.jsonl"))?
        .into_iter()
        .map(|j| (j.id.clone(), j))
        .collect();
    let cb: HashMap<String, String> =
        read_jsonl::<ClosedBookAnswer>(&work.join("closed_book.jsonl"))?
            .into_iter()
            .map(|c| (c.id, c.answer))
            .collect();
    let generator = gens.first().map(|g| g.model.clone()).unwrap_or_default();
    if judge.model == generator {
        bail!("finish needs the judge model, not the generator ({generator})");
    }

    let mut funnel: BTreeMap<String, usize> = BTreeMap::new();
    funnel.insert("1 generated".into(), gens.len());
    funnel.insert(
        "2 passed deterministic filters".into(),
        filtered.values().filter(|f| f.pass).count(),
    );
    let ok = |g: &Generated| -> bool {
        let Some(j) = judged.get(&g.id) else {
            return false;
        };
        let multi_ok = j.a_alone != Some(true) && j.b_alone != Some(true);
        filtered.get(&g.id).is_some_and(|f| f.pass) && j.self_contained && j.answerable && multi_ok
    };
    let mut candidates: Vec<&Generated> = gens.iter().filter(|g| ok(g)).collect();
    funnel.insert(
        "3 judged self-contained".into(),
        judged.values().filter(|j| j.self_contained).count(),
    );
    funnel.insert("4 passed every judge check".into(), candidates.len());
    candidates.sort_by_key(|g| fnv1a(&g.id));

    let cb_correct = |g: &Generated| {
        judged
            .get(&g.id)
            .and_then(|j| j.closed_book_correct)
            .unwrap_or(false)
    };
    #[allow(clippy::cast_precision_loss)]
    let rate = |xs: &[&Generated]| {
        if xs.is_empty() {
            0.0
        } else {
            xs.iter().filter(|g| cb_correct(g)).count() as f64 / xs.len() as f64
        }
    };
    let cand_rate = rate(&candidates);

    let pick = |prefer_unknown: bool| -> Vec<&Generated> {
        let mut sel = Vec::new();
        for (s, quota) in QUOTAS {
            let mut pool: Vec<&Generated> = candidates
                .iter()
                .copied()
                .filter(|g| g.stratum == s)
                .collect();
            if prefer_unknown {
                pool.sort_by_key(|g| (cb_correct(g), fnv1a(&g.id)));
            }
            sel.extend(pool.into_iter().take(quota));
        }
        sel
    };
    let mut selected = pick(false);
    let mut remediated = false;
    if rate(&selected) > CLOSED_BOOK_GATE {
        // Spec remediation 1: prefer questions the model could not answer
        // from memory, which drops the wiki-famous ones first.
        selected = pick(true);
        remediated = true;
    }
    let sel_rate = rate(&selected);
    funnel.insert("5 selected".into(), selected.len());

    // Anchor completion: any pooled passage the judge says answers the
    // question on its own becomes gold as well.
    let cfgs = [Mode::Bm25, Mode::Dense, Mode::Hybrid].map(|m| RetrievalConfig {
        mode: m,
        k: pool_depth,
        ..RetrievalConfig::default()
    });
    let mut items = Vec::new();
    let (mut added_total, mut items_added) = (0, 0);
    let started = std::time::Instant::now();
    for (n, g) in selected.iter().enumerate() {
        let q = g.question.clone().unwrap_or_default();
        let gold: BTreeSet<usize> = g
            .chunk_ids
            .iter()
            .filter_map(|id| rt.store.row(id))
            .collect();
        let mut pooled: BTreeSet<usize> = BTreeSet::new();
        for c in &cfgs {
            pooled.extend(
                rt.retrieve(&q, c)?
                    .into_iter()
                    .map(|h| h.row)
                    .filter(|r| !gold.contains(r)),
            );
        }
        let checks = futures::future::join_all(pooled.iter().map(|&row| {
            let judge = judge.clone();
            let prompt = passage_prompt(&rt.store.chunks[row].text, &q);
            async move {
                judge
                    .verdict(gg::JUDGE_ANSWERABLE, &prompt, gg::GRAMMAR_VERDICT)
                    .await
                    .map(|v| (row, v))
            }
        }))
        .await;
        let mut anchors: Vec<Anchor> = g
            .chunk_ids
            .iter()
            .filter_map(|id| rt.store.row(id))
            .map(|r| {
                let c = &rt.store.chunks[r];
                Anchor {
                    story_id: c.story_id.clone(),
                    line_start: c.line_start,
                    line_end: c.line_end,
                }
            })
            .collect();
        let mut added = Vec::new();
        for r in checks {
            let (row, yes) = r?;
            if yes {
                let c = &rt.store.chunks[row];
                anchors.push(Anchor {
                    story_id: c.story_id.clone(),
                    line_start: c.line_start,
                    line_end: c.line_end,
                });
                added.push(c.chunk_id.clone());
            }
        }
        if !added.is_empty() {
            items_added += 1;
            added_total += added.len();
        }
        let f = &filtered[&g.id];
        let j = &judged[&g.id];
        let item = GoldItem {
            qid: format!("g{:04}", n + 1),
            question: q,
            anchors,
            stratum: g.stratum,
            source_chunk_id: g.chunk_ids.first().cloned(),
            generated_by: Some(format!("{}@{}", g.model, g.prompt_sha)),
            reviewed_by: Some(format!("auto:{}", judge.model)),
            review: Review::Accepted,
            closed_book: j.closed_book_correct.map(|correct| ClosedBook {
                model: g.model.clone(),
                correct,
                checked_at: "2026-09-25".into(),
            }),
            notes: None,
        };
        let mut v = serde_json::to_value(&item)?;
        v["evidence"] = serde_json::json!(f.evidence(g));
        v["gen_id"] = serde_json::json!(g.id);
        v["anchors_added_by_judge"] = serde_json::json!(added);
        v["closed_book_answer"] = serde_json::json!(cb.get(&g.id));
        v["filters"] = serde_json::json!({"word_trigram_containment": f.word_trigram_containment, "bm25_rank": f.bm25_rank});
        items.push(v);
        if (n + 1) % 25 == 0 {
            eprintln!(
                "  anchors: {} of {} items, {:.0}s",
                n + 1,
                selected.len(),
                started.elapsed().as_secs_f64()
            );
        }
    }

    // Unanswerable: keep a hand-written question only if no pooled passage
    // answers it.
    let mut dropped = Vec::new();
    let mut kept_unanswerable = 0;
    for q in gg::UNANSWERABLE
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        if kept_unanswerable >= UNANSWERABLE_QUOTA {
            break;
        }
        let mut pooled: BTreeSet<usize> = BTreeSet::new();
        for c in &cfgs {
            pooled.extend(rt.retrieve(q, c)?.into_iter().map(|h| h.row));
        }
        let checks = futures::future::join_all(pooled.iter().map(|&row| {
            let judge = judge.clone();
            let prompt = passage_prompt(&rt.store.chunks[row].text, q);
            async move {
                judge
                    .verdict(gg::JUDGE_ANSWERABLE, &prompt, gg::GRAMMAR_VERDICT)
                    .await
            }
        }))
        .await;
        let mut answered = false;
        for c in checks {
            answered |= c?;
        }
        if answered {
            dropped.push(q.to_owned());
            continue;
        }
        kept_unanswerable += 1;
        let item = GoldItem {
            qid: format!("u{kept_unanswerable:03}"),
            question: q.to_owned(),
            anchors: vec![],
            stratum: Stratum::Unanswerable,
            source_chunk_id: None,
            generated_by: Some("hand-written (Claude)".into()),
            reviewed_by: Some(format!("auto:{}", judge.model)),
            review: Review::Accepted,
            closed_book: None,
            notes: Some("no pooled passage judged to answer it".into()),
        };
        items.push(serde_json::to_value(&item)?);
    }

    let mut strata: BTreeMap<String, usize> = BTreeMap::new();
    for v in &items {
        *strata
            .entry(v["stratum"].as_str().unwrap_or("").to_owned())
            .or_default() += 1;
    }
    let mut shortfall = BTreeMap::new();
    for (s, q) in QUOTAS
        .iter()
        .copied()
        .chain([(Stratum::Unanswerable, UNANSWERABLE_QUOTA)])
    {
        let have = strata.get(s.name()).copied().unwrap_or(0);
        if have < q {
            shortfall.insert(s.name().to_owned(), q - have);
        }
    }
    let single_sel: Vec<&&Generated> = selected
        .iter()
        .filter(|g| g.stratum != Stratum::MultiHop)
        .collect();
    #[allow(clippy::cast_precision_loss)]
    let rank1 = if single_sel.is_empty() {
        0.0
    } else {
        single_sel
            .iter()
            .filter(|g| filtered[&g.id].bm25_rank == Some(1))
            .count() as f64
            / single_sel.len() as f64
    };
    let meta = Meta {
        judge_calibration: std::fs::read(work.join("calibration.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok()),
        goldset_version: "v1",
        built_at_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        n: items.len(),
        strata,
        shortfall,
        deferred: vec![
            "ambiguous (10): needs shared-name and ??? masking cases built by hand",
            "spoiler (5): needs the progress-boundary design",
        ],
        funnel,
        closed_book_rate_candidates: cand_rate,
        closed_book_rate_selected: sel_rate,
        closed_book_remediated: remediated,
        bm25_rank1_share_selected_single: rank1,
        anchors_added: added_total,
        items_with_added_anchors: items_added,
        unanswerable_dropped: dropped,
        generator,
        judge: judge.model.clone(),
        prompt_shas: BTreeMap::from([
            ("goldset.system", sha16(gg::SYSTEM_SINGLE)),
            ("goldset.gbnf", sha16(gg::GRAMMAR_SINGLE)),
            ("multihop.system", sha16(gg::SYSTEM_MULTI)),
            ("multihop.gbnf", sha16(gg::GRAMMAR_MULTI)),
            ("judge.answerable", sha16(gg::JUDGE_ANSWERABLE)),
            ("judge.self_contained", sha16(gg::JUDGE_SELF_CONTAINED)),
            ("closed_book", sha16(gg::CLOSED_BOOK)),
            ("judge.closed_book", sha16(gg::JUDGE_CLOSED_BOOK)),
        ]),
        generation_variants: generation_variants(&gens),
        review: "automatic: deterministic filters plus binary judge verdicts; no human review",
    };
    write_jsonl(&out.with_extension("jsonl"), &items)?;
    std::fs::write(
        out.with_extension("meta.json"),
        serde_json::to_vec_pretty(&meta)?,
    )?;
    write_spotcheck(&out.with_extension("spotcheck.md"), &items, &rt.store)?;
    println!("{}", serde_json::to_string_pretty(&meta)?);
    Ok(())
}

/// `None` when every row used the v1 grammars and temperature.
fn generation_variants(gens: &[Generated]) -> Option<BTreeMap<String, usize>> {
    let v1 = [sha16(gg::GRAMMAR_SINGLE), sha16(gg::GRAMMAR_MULTI)];
    if gens
        .iter()
        .all(|g| g.temperature.is_none() && v1.contains(&g.grammar_sha))
    {
        return None;
    }
    let mut m = BTreeMap::new();
    for g in gens {
        let t = g.temperature.unwrap_or(gg::V1_TEMPERATURE);
        *m.entry(format!("{}@{t}", g.grammar_sha)).or_insert(0) += 1;
    }
    Some(m)
}

/// Twenty items, seeded, for an optional human spot-check that measures how
/// often the automatic review is wrong.
fn write_spotcheck(path: &Path, items: &[serde_json::Value], store: &ChunkStore) -> Result<()> {
    let mut idx: Vec<usize> = (0..items.len())
        .filter(|&i| items[i]["stratum"] != "unanswerable")
        .collect();
    idx.sort_by_key(|&i| fnv1a(&format!("spot{}", items[i]["qid"])));
    let mut md = String::from(
        "# Gold set v1: optional spot-check\n\nTwenty items drawn at random. For each, mark whether the question is fair and the gold passage answers it. The share you reject estimates the automatic review's error rate; with 20 items, 0 rejections bounds it at 16% (Wilson 95%).\n",
    );
    for &i in idx.iter().take(20) {
        let v = &items[i];
        md.push_str(&format!(
            "\n## {} ({})\n\n**Q:** {}\n\n**Evidence:** {}\n\n",
            v["qid"].as_str().unwrap_or(""),
            v["stratum"].as_str().unwrap_or(""),
            v["question"].as_str().unwrap_or(""),
            v["evidence"]
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(" / "))
                .unwrap_or_default()
        ));
        if let Some(src) = v["source_chunk_id"].as_str().and_then(|id| store.row(id)) {
            let t: String = store.chunks[src].text.chars().take(900).collect();
            md.push_str(&format!(
                "<details><summary>{}</summary>\n\n{}\n\n</details>\n\n",
                store.chunks[src].chunk_id,
                t.replace('\n', "  \n")
            ));
        }
        md.push_str("- [ ] fair question, passage answers it\n- [ ] reject: \n");
    }
    std::fs::write(path, md)?;
    Ok(())
}
