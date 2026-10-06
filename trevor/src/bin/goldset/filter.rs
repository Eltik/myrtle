//! `filter`: the model-free filters (deictic questions, copied wording, verbatim prefixes, duplicates,
//! retrievability).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;
use trevor::eval::goldset::Stratum;
use trevor::goldgen::filters::{deictic, quote_in_passage, trigram_jaccard, verbatim_prefix};
use trevor::goldgen::{Filtered, Generated, containment};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{Runtime, RuntimeArgs};

use crate::files::{read_jsonl, write_jsonl};

/// Word-trigram containment above which a question copies its passage.
pub(crate) const CONTAINMENT_MAX: f64 = 0.30;
pub(crate) const DEDUP_COSINE: f32 = 0.95;

pub(crate) fn cmd_filter(work: &Path, runtime: &RuntimeArgs, min_prefix: Option<usize>) -> Result<()> {
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
