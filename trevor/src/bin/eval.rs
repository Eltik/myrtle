//! Retrieval evaluation: score a configuration on a gold set, gate it against
//! a baseline run, and check the tier-A golden files.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use trevor::eval::golden::{self, Golden, RankedSnapshot};
use trevor::eval::goldset;
use trevor::eval::run::{self, RunInfo, RunRecord, Selection, Verdict};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{ConfigArgs, Runtime, RuntimeArgs};
use trevor::search::store::ChunkStore;

#[derive(Parser)]
#[command(about = "Trevor retrieval evaluation")]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check a gold set against the current chunks without running anything.
    Validate {
        #[arg(long)]
        goldset: PathBuf,
        #[arg(long, default_value = "artifacts/p3a")]
        corpus: PathBuf,
    },
    /// Score one configuration and append it to <out>/runs.jsonl.
    Run {
        #[arg(long)]
        goldset: PathBuf,
        #[command(flatten)]
        runtime: RuntimeArgs,
        #[command(flatten)]
        config: ConfigArgs,
        /// Where runs.jsonl and runs/ live.
        #[arg(long, default_value = "eval")]
        out: PathBuf,
        /// A run id, or `last` for the latest run on this gold set.
        #[arg(long)]
        baseline: Option<String>,
        /// Free-text name for the run, e.g. `p0-hybrid-rerank40`.
        #[arg(long)]
        label: Option<String>,
        /// Also score items not yet reviewed as accepted. Never for a gate.
        #[arg(long)]
        include_unreviewed: bool,
        /// Exit nonzero on a FAIL verdict or a broken absolute floor.
        #[arg(long)]
        gate: bool,
    },
    /// Compare two stored runs.
    Compare {
        baseline: String,
        current: String,
        #[arg(long)]
        goldset: PathBuf,
        #[arg(long, default_value = "eval")]
        out: PathBuf,
    },
    /// Tier A: corpus facts and exact top-10 ids for a fixed query list.
    Golden {
        #[arg(long, default_value = "eval/golden-queries.txt")]
        queries: PathBuf,
        #[arg(long, default_value = "eval/golden-a.json")]
        file: PathBuf,
        #[command(flatten)]
        runtime: RuntimeArgs,
        /// Rerank depth for the `hybrid+rerank` snapshot.
        #[arg(long, default_value_t = 40)]
        rerank_top: usize,
        /// Skip the model-dependent snapshots (dense, hybrid, rerank).
        #[arg(long)]
        bm25_only: bool,
        /// Record instead of check.
        #[arg(long)]
        update: bool,
    },
}

fn git_commit() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8(out.stdout).ok()?.trim().to_owned();
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "."])
        .output()
        .ok()
        .is_some_and(|o| !o.stdout.is_empty());
    Some(if dirty { format!("{sha}-dirty") } else { sha })
}

fn print_record(r: &RunRecord) {
    println!(
        "run {}  ({} scored of {}; {} unanswerable skipped; {} not accepted)",
        r.run_id, r.n_scored, r.n_items, r.n_unanswerable, r.n_not_accepted
    );
    let m = |k: &str| r.metrics.get(k).copied().unwrap_or(0.0);
    println!(
        "  recall@10 {:.3} [{:.3}, {:.3}]   recall@20 {:.3}",
        m("recall@10"),
        r.recall10_ci.0,
        r.recall10_ci.1,
        m("recall@20")
    );
    println!(
        "  hit@1 {:.3}  hit@5 {:.3}  hit@10 {:.3} [{:.3}, {:.3}]  hit@20 {:.3}",
        m("hit@1"),
        m("hit@5"),
        m("hit@10"),
        r.hit10_wilson.0,
        r.hit10_wilson.1,
        m("hit@20")
    );
    println!(
        "  precision@5 {:.3}  ndcg@10 {:.3} (diagnostic)   latency p50 {:.0} ms, p95 {:.0} ms",
        m("precision@5"),
        m("ndcg@10"),
        r.ms_p50,
        r.ms_p95
    );
    for (s, v) in &r.per_stratum {
        println!(
            "    {s:<13} n={:<3} recall@10 {:.3}  hit@10 {:.3}",
            v.n,
            v.metrics.get("recall@10").copied().unwrap_or(0.0),
            v.metrics.get("hit@10").copied().unwrap_or(0.0)
        );
    }
    if !r.absolute_floor_pass {
        println!(
            "  ABSOLUTE FLOOR BROKEN: hit@10 {:.3} <= {}",
            m("hit@10"),
            run::ABSOLUTE_FLOOR
        );
    }
    if let Some(c) = &r.vs_baseline {
        println!(
            "  vs {}: recall@10 {:+.3} [{:+.3}, {:+.3}], paired t p={:.3}; hit@10 lost {} gained {} (McNemar p={:.3}) -> {:?}",
            c.baseline_run_id,
            c.delta,
            c.ci95.0,
            c.ci95.1,
            c.p_paired_t,
            c.hit10_only_baseline,
            c.hit10_only_current,
            c.hit10_mcnemar_p,
            c.verdict
        );
    }
}

fn main() -> Result<()> {
    match Args::parse().cmd {
        Cmd::Validate { goldset: g, corpus } => {
            let store = ChunkStore::load(&corpus.join("chunks.jsonl"))?;
            let set = goldset::load(&g)?;
            goldset::validate(&set, &store)?;
            let gold_chunks: usize = set
                .items
                .iter()
                .map(|i| goldset::resolve(i, &store).len())
                .sum();
            println!(
                "{} items valid ({} gold chunks in total, sha {:.16})",
                set.items.len(),
                gold_chunks,
                set.sha
            );
        }
        Cmd::Run {
            goldset: g,
            runtime,
            config,
            out,
            baseline,
            label,
            include_unreviewed,
            gate,
        } => {
            let mut cfg = RetrievalConfig::from(&config);
            // Metrics look 20 deep; a shorter list would score as misses.
            cfg.k = cfg.k.max(20);
            let mut rt = Runtime::load(
                &runtime,
                matches!(cfg.mode, Mode::Dense | Mode::Hybrid),
                matches!(cfg.mode, Mode::Bm25 | Mode::Hybrid),
                cfg.rerank_top > 0,
            )?;
            let set = goldset::load(&g)?;
            goldset::validate(&set, &rt.store)?;
            let started = std::time::Instant::now();
            let mut retrieve = |q: &str| -> Result<Vec<usize>> {
                Ok(rt.retrieve(q, &cfg)?.into_iter().map(|h| h.row).collect())
            };
            // The retrieval closure holds the runtime mutably, so scoring
            // resolves anchors against a second copy of the store, checked to
            // be the same bytes.
            let store = ChunkStore::load(&runtime.corpus.join("chunks.jsonl"))?;
            let outcomes = run::score(
                &set,
                &store,
                Selection { include_unreviewed },
                &mut retrieve,
            )?;
            if store.chunks_sha != rt.store.chunks_sha {
                bail!("chunks.jsonl changed during the run");
            }
            if outcomes.is_empty() {
                bail!(
                    "no items scored: are any reviewed as accepted? (--include-unreviewed to score the rest)"
                );
            }
            let run_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let run_id = format!("{run_at}-{}", label.as_deref().unwrap_or("run"));
            let mut record = run::summarize(
                RunInfo {
                    run_id,
                    label,
                    commit: git_commit(),
                    goldset_path: g.display().to_string(),
                    config: serde_json::to_value(&cfg)?,
                    fingerprint: serde_json::to_value(rt.fingerprint())?,
                },
                &set,
                &outcomes,
            );
            if let Some(b) = baseline {
                let (brec, bout) = run::load(&out, &b, &set.sha)?;
                record.vs_baseline = Some(run::compare(&brec.run_id, &bout, &outcomes)?);
            }
            let path = run::save(&out, &record, &outcomes)?;
            print_record(&record);
            eprintln!(
                "  {:.1}s, saved {}",
                started.elapsed().as_secs_f64(),
                path.display()
            );
            if gate {
                if !record.absolute_floor_pass {
                    bail!("gate: absolute floor broken");
                }
                if record
                    .vs_baseline
                    .as_ref()
                    .is_some_and(|c| c.verdict == Verdict::Fail)
                {
                    bail!("gate: recall@10 regressed against the baseline");
                }
            }
        }
        Cmd::Compare {
            baseline,
            current,
            goldset: g,
            out,
        } => {
            let set = goldset::load(&g)?;
            let (brec, bout) = run::load(&out, &baseline, &set.sha)?;
            let (mut crec, cout) = run::load(&out, &current, &set.sha)?;
            crec.vs_baseline = Some(run::compare(&brec.run_id, &bout, &cout)?);
            print_record(&crec);
        }
        Cmd::Golden {
            queries,
            file,
            runtime,
            rerank_top,
            bm25_only,
            update,
        } => {
            let text = std::fs::read_to_string(&queries)
                .with_context(|| format!("reading {}", queries.display()))?;
            let qs: Vec<&str> = text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
            let mut rt = Runtime::load(&runtime, !bm25_only, true, !bm25_only && rerank_top > 0)?;
            let fp = rt.fingerprint();
            let corpus = golden::corpus_facts(&rt.store, &runtime.corpus)?;
            let mut configs: Vec<(&str, RetrievalConfig, BTreeMap<String, String>)> = vec![(
                "bm25",
                RetrievalConfig {
                    mode: Mode::Bm25,
                    k: 10,
                    ..RetrievalConfig::default()
                },
                BTreeMap::new(),
            )];
            if !bm25_only {
                let q = BTreeMap::from([(
                    "queryModel".to_owned(),
                    fp.query_model_sha.clone().unwrap_or_default(),
                )]);
                let mut qr = q.clone();
                qr.insert(
                    "reranker".to_owned(),
                    fp.reranker_sha.clone().unwrap_or_default(),
                );
                qr.insert(
                    "rerankMaxTokens".to_owned(),
                    fp.rerank_max_tokens.unwrap_or(0).to_string(),
                );
                configs.push((
                    "dense",
                    RetrievalConfig {
                        mode: Mode::Dense,
                        k: 10,
                        ..RetrievalConfig::default()
                    },
                    q.clone(),
                ));
                configs.push((
                    "hybrid",
                    RetrievalConfig {
                        mode: Mode::Hybrid,
                        k: 10,
                        ..RetrievalConfig::default()
                    },
                    q,
                ));
                if rerank_top > 0 {
                    configs.push((
                        "hybrid+rerank",
                        RetrievalConfig {
                            mode: Mode::Hybrid,
                            k: 10,
                            rerank_top,
                            ..RetrievalConfig::default()
                        },
                        qr,
                    ));
                }
            }
            let mut rankings = BTreeMap::new();
            for (name, cfg, deps) in configs {
                let mut results = Vec::new();
                for q in &qs {
                    let ids = rt
                        .retrieve(q, &cfg)?
                        .into_iter()
                        .map(|h| rt.store.chunks[h.row].chunk_id.clone())
                        .collect();
                    results.push(((*q).to_owned(), ids));
                }
                rankings.insert(
                    name.to_owned(),
                    RankedSnapshot {
                        depends_on: deps,
                        results,
                    },
                );
            }
            let actual = Golden { corpus, rankings };
            if update {
                std::fs::write(&file, serde_json::to_vec_pretty(&actual)?)?;
                println!(
                    "recorded {} ({} queries, {} rankings)",
                    file.display(),
                    qs.len(),
                    actual.rankings.len()
                );
                return Ok(());
            }
            let expected: Golden =
                serde_json::from_slice(&std::fs::read(&file).with_context(|| {
                    format!("reading {}; record it with --update", file.display())
                })?)?;
            let (diffs, skipped) = golden::diff(&expected, &actual);
            for s in &skipped {
                println!("skipped  {s}");
            }
            for d in &diffs {
                println!("DIFF     {d}");
            }
            if !diffs.is_empty() {
                bail!(
                    "{} golden difference(s); if intended, re-record with --update",
                    diffs.len()
                );
            }
            println!(
                "golden: identical ({} checked, {} skipped)",
                actual.rankings.len() - skipped.len(),
                skipped.len()
            );
        }
    }
    Ok(())
}
