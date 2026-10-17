//! Offline checks of the service paths: `--horizon-check` and `--json-check`.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use trevor::search::horizon::{Horizon, ReleaseOrder};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;

use crate::answer::{Tables, route_checked};
use crate::cli::{Args, RetrievalKind, deep_for};
use crate::retrieval::load_runtime;
use crate::llama_server::connect_or_spawn;


/// `--horizon-check --batch FILE`: the gatePos horizon against the old story-id horizon. First every chunk of the main
/// corpus and of P4x at every placed story and group of the release order, then each question's 2,000 retrieved
/// candidates at the tie stories and 32 other horizons; every difference is printed.
pub(crate) fn horizon_check(a: &Args, batch: &Path) -> Result<()> {
    let order = Arc::new(ReleaseOrder::load(Path::new("."))?);
    let gate = order.gate();
    let horizons: Vec<String> = gate.story_order.iter().filter(|s| order.legacy_horizon(s).is_ok()).cloned()
        .chain(gate.eras.iter().map(|e| e.id.clone())).collect();
    let ties: Vec<String> = {
        let mut seen: HashMap<(String, i64), Vec<String>> = HashMap::new();
        for l in std::fs::read_to_string("artifacts/spoiler.jsonl")?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            if let (Some(s), Some(g), Some(k)) = (v["storyId"].as_str(), v["groupId"].as_str(), v["storySort"].as_i64()) {
                seen.entry((g.to_owned(), k)).or_default().push(s.to_owned());
            }
        }
        let mut t: Vec<String> = seen.into_values().filter(|v| v.len() > 1).flatten().filter(|s| order.legacy_horizon(s).is_ok()).collect();
        t.sort();
        t
    };
    println!("tie stories in placed groups: {} {:?}", ties.len(), ties);
    let mut corpora = vec![a.runtime.corpus.clone()];
    if Path::new(crate::cli::GAME_TEXT_DIR).join("chunks.jsonl").exists() && a.runtime.corpus != Path::new(crate::cli::GAME_TEXT_DIR) {
        corpora.push(PathBuf::from(crate::cli::GAME_TEXT_DIR));
    }
    for dir in &corpora {
        let mut ra = a.runtime.clone();
        ra.corpus.clone_from(dir);
        let rt = Runtime::load(&ra, false, false, false)?;
        let (mut checks, mut diffs) = (0u64, BTreeMap::<(String, String, String), usize>::new());
        for h in &horizons {
            let (old, new) = (order.legacy_horizon(h)?, order.horizon(h)?);
            for c in &rt.store.chunks {
                checks += 1;
                if old.allows(&c.story_id, &c.group_id) != new.allows(&c.story_id, &c.group_id) {
                    *diffs.entry((h.clone(), c.story_id.clone(), c.group_id.clone())).or_default() += 1;
                }
            }
        }
        println!("{}: {} chunks x {} horizons = {checks} checks, {} (horizon, chunk story) pairs differ:", dir.display(),
                 rt.store.len(), horizons.len(), diffs.len());
        for ((h, s, g), n) in &diffs {
            let tie = ties.contains(h) && ties.contains(s);
            println!("  horizon {h} chunk story {s}: {n} chunks, old {} new {}{}", order.legacy_horizon(h)?.allows(s, g),
                     order.horizon(h)?.allows(s, g), if tie { " (tie)" } else { " (NOT A TIE)" });
        }
    }
    // The questions: hybrid candidates (BM25 with --retrieval bm25) at the tie stories and 32 spread horizons.
    let dense = a.retrieval == RetrievalKind::Hybrid;
    let mut rt = load_runtime(&a.runtime, dense, None)?;
    let step = (horizons.len() / 32).max(1);
    let sample: Vec<String> = ties.iter().cloned().chain(horizons.iter().step_by(step).cloned()).collect();
    let (mut kept_same, mut diff_rows) = (0usize, 0usize);
    for line in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let (qid, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
        let hits = rt.retrieve(q, &RetrievalConfig { mode: Mode::Hybrid, candidates: 2000, k: 2000, ..RetrievalConfig::default() })?;
        for h in &sample {
            let (old, new) = (order.legacy_horizon(h)?, order.horizon(h)?);
            let keep = |hz: &Horizon| -> Vec<usize> { hits.iter().map(|x| x.row).filter(|&r| rt.allows_row(hz, r)).collect() };
            let (ko, kn) = (keep(&old), keep(&new));
            if ko == kn {
                kept_same += 1;
                continue;
            }
            let only_old: Vec<&str> = ko.iter().filter(|r| !kn.contains(r)).map(|&r| rt.store.chunks[r].chunk_id.as_str()).collect();
            let only_new: Vec<&str> = kn.iter().filter(|r| !ko.contains(r)).map(|&r| rt.store.chunks[r].chunk_id.as_str()).collect();
            diff_rows += 1;
            println!("  {qid} at {h}: old keeps {} new keeps {}; old only {only_old:?}; new only {only_new:?}", ko.len(), kn.len());
        }
    }
    println!("questions: {kept_same} (question, horizon) pairs keep the same candidates, {diff_rows} differ");
    Ok(())
}

/// `--json-check --batch FILE`: the router and question-form calls of every question (no answer), with the usage
/// totals; on `--backend openai` that is the JSON validation rate. Prints `qid<TAB>route<TAB>form`.
pub(crate) async fn json_check(a: &Args, tables: &mut Tables, batch: &Path) -> Result<()> {
    let (llm, _guard) = connect_or_spawn(a).await?;
    let mut n = 0;
    for line in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let (qid, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_checked(q, a, tables, Some(&llm)).await?;
        let form = match crate::form::question_form(&llm, q, a.form_v1).await {
            Ok(f) => format!("{}{}", f.form, f.evidence.map_or(String::new(), |e| format!(": {e}"))),
            Err(e) if trevor::llm::is_json_invalid(&e) => "INVALID".to_owned(),
            Err(e) => return Err(e),
        };
        println!("{qid}\t{}\t{form}", serde_json::to_string(&r.route)?);
        n += 1;
    }
    let u = llm.usage();
    eprintln!("json-check: {n} questions, {} calls, {} JSON calls, {} needed a second ask, {} failed twice; {} tokens in, {} out",
              u.calls, u.json_calls, u.json_retries, u.json_failures, u.tokens_in, u.tokens_out);
    Ok(())
}
