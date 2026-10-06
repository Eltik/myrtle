//! Scoring a retriever over a gold set, the run record, and the paired
//! comparison that gates a change.
//!
//! The gate follows the eval spec: one pooled primary metric (recall@10),
//! judged on the PAIRED difference against a baseline run over the same
//! frozen questions. FAIL only when the whole 95% bootstrap CI of the
//! difference is below zero; WARN when the point estimate is negative but the
//! CI straddles zero. A single absolute floor (hit@10 > 0.40) catches the
//! catastrophic case noise cannot reach: an index that did not build or a
//! model that changed silently.

use std::collections::BTreeMap;
use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use super::goldset::{GoldSet, Review, Stratum, resolve};
use super::metrics;
use super::stats::{Z95, mcnemar_exact, paired_bootstrap_ci, paired_t, wilson};
use crate::search::store::ChunkStore;

pub const PRIMARY: &str = "recall@10";
pub const ABSOLUTE_FLOOR_METRIC: &str = "hit@10";
pub const ABSOLUTE_FLOOR: f64 = 0.40;
pub const BOOTSTRAP_ITERS: usize = 10_000;
pub const BOOTSTRAP_SEED: u64 = 0x7472_6576_6f72; // "trevor" in ASCII

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOutcome {
    pub qid: String,
    pub stratum: Stratum,
    pub question: String,
    pub gold: Vec<String>,
    pub retrieved: Vec<String>,
    pub first_gold_rank: Option<usize>,
    pub metrics: BTreeMap<String, f64>,
    pub ms: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StratumSummary {
    pub n: usize,
    pub metrics: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub baseline_run_id: String,
    pub n_paired: usize,
    pub primary: String,
    pub delta: f64,
    pub ci95: (f64, f64),
    pub p_paired_t: f64,
    /// Discordant pairs on hit@10: only the baseline hit / only this run hit.
    pub hit10_only_baseline: usize,
    pub hit10_only_current: usize,
    pub hit10_mcnemar_p: f64,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub run_id: String,
    pub label: Option<String>,
    pub commit: Option<String>,
    pub run_at_unix: u64,
    pub goldset_path: String,
    pub goldset_sha: String,
    pub n_items: usize,
    pub n_scored: usize,
    pub n_unanswerable: usize,
    pub n_not_accepted: usize,
    pub config: serde_json::Value,
    pub fingerprint: serde_json::Value,
    pub metrics: BTreeMap<String, f64>,
    /// 95% Wilson interval on hit@10 (exact for a binary metric).
    pub hit10_wilson: (f64, f64),
    /// 95% bootstrap interval on mean recall@10.
    pub recall10_ci: (f64, f64),
    pub absolute_floor_pass: bool,
    pub per_stratum: BTreeMap<String, StratumSummary>,
    pub ms_p50: f64,
    pub ms_p95: f64,
    pub vs_baseline: Option<Comparison>,
}

/// Which items a run scores.
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    /// Score items not yet reviewed as accepted. Off for any gated run.
    pub include_unreviewed: bool,
}

/// Run `retrieve` over every scorable item.
///
/// Unanswerable items have no anchors and are skipped here; abstention is
/// measured with generation, not retrieval. Rejected items are never scored.
///
/// # Errors
/// A retrieval failure, reported with the question that caused it.
pub fn score(
    set: &GoldSet,
    store: &ChunkStore,
    sel: Selection,
    retrieve: &mut dyn FnMut(&str) -> Result<Vec<usize>>,
) -> Result<Vec<QueryOutcome>> {
    let mut out = Vec::new();
    for it in &set.items {
        if it.stratum == Stratum::Unanswerable || it.review == Review::Rejected {
            continue;
        }
        if it.review != Review::Accepted && !sel.include_unreviewed {
            continue;
        }
        let gold = resolve(it, store);
        let started = std::time::Instant::now();
        let ranked =
            retrieve(&it.question).with_context(|| format!("{}: {}", it.qid, it.question))?;
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        out.push(QueryOutcome {
            qid: it.qid.clone(),
            stratum: it.stratum,
            question: it.question.clone(),
            gold: gold
                .iter()
                .map(|&r| store.chunks[r].chunk_id.clone())
                .collect(),
            retrieved: ranked
                .iter()
                .take(20)
                .map(|&r| store.chunks[r].chunk_id.clone())
                .collect(),
            first_gold_rank: metrics::first_gold_rank(&ranked, &gold),
            metrics: metrics::all(&ranked, &gold)
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            ms,
        });
    }
    Ok(out)
}

fn mean_of(outcomes: &[&QueryOutcome], metric: &str) -> f64 {
    super::stats::mean(
        &outcomes
            .iter()
            .map(|o| o.metrics[metric])
            .collect::<Vec<_>>(),
    )
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let i = ((q * (sorted.len() - 1) as f64).round() as usize).min(sorted.len() - 1);
    sorted[i]
}

pub struct RunInfo {
    pub run_id: String,
    pub label: Option<String>,
    pub commit: Option<String>,
    pub goldset_path: String,
    pub config: serde_json::Value,
    pub fingerprint: serde_json::Value,
}

#[must_use]
pub fn summarize(info: RunInfo, set: &GoldSet, outcomes: &[QueryOutcome]) -> RunRecord {
    let all: Vec<&QueryOutcome> = outcomes.iter().collect();
    let metrics: BTreeMap<String, f64> = metrics::METRICS
        .iter()
        .map(|m| ((*m).to_owned(), mean_of(&all, m)))
        .collect();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let hits10 = outcomes
        .iter()
        .filter(|o| o.metrics["hit@10"] > 0.5)
        .count();
    let recall10: Vec<f64> = outcomes.iter().map(|o| o.metrics[PRIMARY]).collect();
    let mut per_stratum: BTreeMap<String, StratumSummary> = BTreeMap::new();
    let mut by: BTreeMap<Stratum, Vec<&QueryOutcome>> = BTreeMap::new();
    for o in outcomes {
        by.entry(o.stratum).or_default().push(o);
    }
    for (s, os) in &by {
        per_stratum.insert(
            s.name().to_owned(),
            StratumSummary {
                n: os.len(),
                metrics: metrics::METRICS
                    .iter()
                    .map(|m| ((*m).to_owned(), mean_of(os, m)))
                    .collect(),
            },
        );
    }
    let mut ms: Vec<f64> = outcomes.iter().map(|o| o.ms).collect();
    ms.sort_by(f64::total_cmp);
    let floor_value = metrics.get(ABSOLUTE_FLOOR_METRIC).copied().unwrap_or(0.0);
    RunRecord {
        run_id: info.run_id,
        label: info.label,
        commit: info.commit,
        run_at_unix: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
        goldset_path: info.goldset_path,
        goldset_sha: set.sha.clone(),
        n_items: set.items.len(),
        n_scored: outcomes.len(),
        n_unanswerable: set
            .items
            .iter()
            .filter(|i| i.stratum == Stratum::Unanswerable)
            .count(),
        n_not_accepted: set
            .items
            .iter()
            .filter(|i| i.review != Review::Accepted)
            .count(),
        config: info.config,
        fingerprint: info.fingerprint,
        metrics,
        hit10_wilson: wilson(hits10, outcomes.len(), Z95),
        recall10_ci: mean_ci(&recall10),
        absolute_floor_pass: floor_value > ABSOLUTE_FLOOR,
        per_stratum,
        ms_p50: pct(&ms, 0.5),
        ms_p95: pct(&ms, 0.95),
        vs_baseline: None,
    }
}

/// Percentile bootstrap on a single sample's mean (not paired).
fn mean_ci(xs: &[f64]) -> (f64, f64) {
    // The paired bootstrap resamples the values it is given; on raw values
    // that is exactly the one-sample bootstrap of the mean.
    paired_bootstrap_ci(xs, BOOTSTRAP_ITERS, BOOTSTRAP_SEED)
}

/// Compare `current` against `baseline` question by question.
///
/// # Errors
/// The two runs did not score the same questions.
pub fn compare(
    baseline_id: &str,
    baseline: &[QueryOutcome],
    current: &[QueryOutcome],
) -> Result<Comparison> {
    let base: BTreeMap<&str, &QueryOutcome> =
        baseline.iter().map(|o| (o.qid.as_str(), o)).collect();
    let cur: BTreeMap<&str, &QueryOutcome> = current.iter().map(|o| (o.qid.as_str(), o)).collect();
    if base.keys().ne(cur.keys()) {
        let only_b: Vec<_> = base.keys().filter(|k| !cur.contains_key(*k)).collect();
        let only_c: Vec<_> = cur.keys().filter(|k| !base.contains_key(*k)).collect();
        bail!(
            "runs scored different questions (only baseline: {only_b:?}, only current: {only_c:?}); \
             a paired comparison needs the same frozen set"
        );
    }
    let deltas: Vec<f64> = cur
        .iter()
        .map(|(q, c)| c.metrics[PRIMARY] - base[q].metrics[PRIMARY])
        .collect();
    let t = paired_t(&deltas);
    let ci95 = paired_bootstrap_ci(&deltas, BOOTSTRAP_ITERS, BOOTSTRAP_SEED);
    let (mut only_b, mut only_c) = (0, 0);
    for (q, c) in &cur {
        match (base[q].metrics["hit@10"] > 0.5, c.metrics["hit@10"] > 0.5) {
            (true, false) => only_b += 1,
            (false, true) => only_c += 1,
            _ => {}
        }
    }
    let verdict = if ci95.1 < 0.0 {
        Verdict::Fail
    } else if t.mean < 0.0 {
        Verdict::Warn
    } else {
        Verdict::Pass
    };
    Ok(Comparison {
        baseline_run_id: baseline_id.to_owned(),
        n_paired: deltas.len(),
        primary: PRIMARY.to_owned(),
        delta: t.mean,
        ci95,
        p_paired_t: t.p,
        hit10_only_baseline: only_b,
        hit10_only_current: only_c,
        hit10_mcnemar_p: mcnemar_exact(only_b, only_c),
        verdict,
    })
}

#[derive(Serialize, Deserialize)]
struct StoredRun {
    record: RunRecord,
    outcomes: Vec<QueryOutcome>,
}

/// Write `<dir>/runs/<run_id>.json` (record plus per-question outcomes) and
/// append the record alone to `<dir>/runs.jsonl`.
///
/// # Errors
/// I/O.
pub fn save(dir: &Path, record: &RunRecord, outcomes: &[QueryOutcome]) -> Result<PathBuf> {
    let runs = dir.join("runs");
    std::fs::create_dir_all(&runs)?;
    let path = runs.join(format!("{}.json", record.run_id));
    let stored = StoredRun {
        record: record.clone(),
        outcomes: outcomes.to_vec(),
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&stored)?)?;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("runs.jsonl"))?;
    writeln!(f, "{}", serde_json::to_string(record)?)?;
    Ok(path)
}

/// Load a stored run by id, or `last` for the most recent run in
/// `runs.jsonl` on the same gold set.
///
/// # Errors
/// No such run, or no earlier run on this gold set.
pub fn load(dir: &Path, id: &str, goldset_sha: &str) -> Result<(RunRecord, Vec<QueryOutcome>)> {
    let id = if id == "last" {
        let f = std::fs::File::open(dir.join("runs.jsonl"))
            .context("no runs.jsonl yet, so there is no last run")?;
        let mut last = None;
        for line in std::io::BufReader::new(f).lines() {
            let r: RunRecord = serde_json::from_str(&line?)?;
            if r.goldset_sha == goldset_sha {
                last = Some(r.run_id);
            }
        }
        last.context("no earlier run on this gold set")?
    } else {
        id.to_owned()
    };
    let path = dir.join("runs").join(format!("{id}.json"));
    let s: StoredRun = serde_json::from_slice(
        &std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?,
    )?;
    if s.record.goldset_sha != goldset_sha {
        bail!(
            "run {id} used a different gold set ({:.16}); runs are only comparable on the same set",
            s.record.goldset_sha
        );
    }
    Ok((s.record, s.outcomes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(qid: &str, r10: f64, h10: f64) -> QueryOutcome {
        QueryOutcome {
            qid: qid.into(),
            stratum: Stratum::SingleFact,
            question: String::new(),
            gold: vec![],
            retrieved: vec![],
            first_gold_rank: None,
            metrics: [("recall@10".to_owned(), r10), ("hit@10".to_owned(), h10)].into(),
            ms: 0.0,
        }
    }

    #[test]
    fn a_consistent_drop_fails_and_noise_does_not() {
        let base: Vec<_> = (0..40).map(|i| o(&format!("q{i}"), 1.0, 1.0)).collect();
        let worse: Vec<_> = (0..40)
            .map(|i| {
                o(
                    &format!("q{i}"),
                    if i < 20 { 0.0 } else { 1.0 },
                    if i < 20 { 0.0 } else { 1.0 },
                )
            })
            .collect();
        let c = compare("b", &base, &worse).expect("same qids");
        assert_eq!(c.verdict, Verdict::Fail);
        assert_eq!((c.hit10_only_baseline, c.hit10_only_current), (20, 0));

        // One question lost, one gained: mean 0, a pass.
        let mixed: Vec<_> = (0..40)
            .map(|i| o(&format!("q{i}"), if i == 0 { 0.0 } else { 1.0 }, 1.0))
            .collect();
        let base2: Vec<_> = (0..40)
            .map(|i| o(&format!("q{i}"), if i == 1 { 0.0 } else { 1.0 }, 1.0))
            .collect();
        assert_eq!(
            compare("b", &base2, &mixed).expect("same").verdict,
            Verdict::Pass
        );

        // One question lost only: negative mean, CI reaches 0 -> warn, not fail.
        let one: Vec<_> = (0..40)
            .map(|i| o(&format!("q{i}"), if i == 0 { 0.0 } else { 1.0 }, 1.0))
            .collect();
        assert_eq!(
            compare("b", &base, &one).expect("same").verdict,
            Verdict::Warn
        );
    }

    #[test]
    fn different_question_sets_are_refused() {
        let a = vec![o("q1", 1.0, 1.0)];
        let b = vec![o("q2", 1.0, 1.0)];
        assert!(compare("b", &a, &b).is_err());
    }
}
