//! The kNN router's labelled intents: labelling, paraphrasing and the held-out evaluation.

use std::collections::BTreeSet;
use std::io::Write as _;

use anyhow::{Context, Result};
use trevor::goldgen::llm::{Llm, Request};
use trevor::router::{self, Intent, Knn, KnnParams};
use trevor::tools::{Route, Tools, norm};

use crate::cli::Args;
use crate::routing::keyword_route;

pub(crate) fn load_knn(a: &Args, tools: &Tools) -> Result<Knn> {
    let p = KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share };
    Knn::load(&a.knn_model_dir, &a.knn_onnx, &a.intents, a.knn_synth.as_deref(), p, tools)
}

/// `--label-intents`: the model router's choice for every question of the three real-question sets, resumable.
/// Route-check expectations are hand-verified, so they win: a question expected to go to retrieval is labelled
/// `retrieve`, and one expected at a table takes the keyword router's tool and arguments (61 of 61 pass).
pub(crate) async fn label_intents(tools: &Tools, llm: &Llm, out_path: &std::path::Path) -> Result<()> {
    let done: BTreeSet<String> = std::fs::read_to_string(out_path).unwrap_or_default().lines()
        .filter_map(|l| serde_json::from_str::<Intent>(l).ok().map(|i| i.q)).collect();
    let mut todo: Vec<(String, String, Option<serde_json::Value>)> = Vec::new();
    let mut seen = done.clone();
    for (file, source) in [("eval/routes.jsonl", "routes"), ("eval/reddit-lore-questions.all.jsonl", "reddit"),
                           ("eval/discord-lore-questions.jsonl", "discord")] {
        for l in std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?.lines() {
            let v: serde_json::Value = serde_json::from_str(l)?;
            // X is noise and P out of scope (gameplay), design/trevor-questions.md section 2; Discord has neither.
            if matches!(v["category"].as_str(), Some("X" | "P")) {
                continue;
            }
            let q = v["q"].as_str().context("item without q")?.to_owned();
            if seen.insert(q.clone()) {
                // A case naming its tool (canon, topic, Storylines cases) is checked on the tool, not table or retrieval.
                let expect = (source == "routes").then(|| if v["tool"].is_string() { v["tool"].clone() } else { v["expect"].clone() });
                todo.push((q, source.to_owned(), expect));
            }
        }
    }
    eprintln!("label-intents: {} to do, {} done, model {}", todo.len(), done.len(), llm.model);
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_path)?;
    let started = std::time::Instant::now();
    let mut ms = Vec::new();
    for (n, (q, source, expect)) in todo.iter().enumerate() {
        let t0 = std::time::Instant::now();
        let m = router::route_model(llm, q).await?;
        ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        let mut it = Intent { q: q.clone(), tool: m.tool.clone(), args: m.args.clone(), source: source.clone(), model_tool: None,
                               source_q: None, verified: None };
        match expect.as_ref().and_then(|e| e.as_str()) {
            Some("retrieval") if !m.is_retrieve() => {
                it.model_tool = Some(m.tool.clone());
                it.tool = trevor::tools::RETRIEVE.into();
                it.args.clear();
            }
            Some(tool) if tool != "table" && tool != "retrieval" => {
                if m.tool != tool {
                    it.model_tool = Some(m.tool.clone());
                    it.tool = tool.to_owned();
                    it.args.clear();
                }
            }
            Some("table") => {
                if let Some((kr, _)) = keyword_route(q, tools) {
                    if kr != m {
                        it.model_tool = Some(m.tool.clone());
                        it.tool = kr.tool;
                        it.args = kr.args;
                    }
                }
            }
            _ => {}
        }
        serde_json::to_writer(&mut out, &it)?;
        out.write_all(b"\n")?;
        if (n + 1) % 50 == 0 {
            #[allow(clippy::cast_precision_loss)]
            let each = started.elapsed().as_secs_f64() / (n + 1) as f64;
            eprintln!("label-intents {}/{}, {each:.2} s each", n + 1, todo.len());
        }
    }
    if !ms.is_empty() {
        ms.sort_by(f64::total_cmp);
        #[allow(clippy::cast_precision_loss)]
        let mean = ms.iter().sum::<f64>() / ms.len() as f64;
        eprintln!("model router latency over {} questions: mean {mean:.0} ms, median {:.0} ms, p90 {:.0} ms", ms.len(),
                  ms[ms.len() / 2], ms[ms.len() * 9 / 10]);
    }
    Ok(())
}

/// A seeded shuffle (xorshift64), so the split is the same on every run.
pub(crate) fn shuffled(n: usize, seed: u64) -> Vec<usize> {
    let mut x = seed.max(1);
    let mut idx: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        let j = usize::try_from(x % (i as u64 + 1)).unwrap_or(0);
        idx.swap(i, j);
    }
    idx
}

/// The seeded 80/20 split of the real labelled intents: (train, test) indices. Paraphrases are generated from the
/// training part only, so the test questions and anything written from them stay unseen.
pub(crate) fn intent_split(n: usize) -> (Vec<usize>, Vec<usize>) {
    let order = shuffled(n, 20_260_929);
    let cut = n * 4 / 5;
    (order[..cut].to_vec(), order[cut..].to_vec())
}

pub(crate) fn read_intents(p: &std::path::Path) -> Result<Vec<Intent>> {
    trevor::util::read_jsonl_strict(p)
}

/// Two routes reach the same table target: the same tool and either the same arguments (folded) or the same answer.
pub(crate) fn same_target(tools: &Tools, a: &Route, b: &Route) -> bool {
    if a.tool != b.tool {
        return false;
    }
    let fold = |r: &Route| r.args.iter().map(|(k, v)| (k.clone(), norm(v))).collect::<Vec<_>>();
    if fold(a) == fold(b) || a.args.is_empty() {
        return true;
    }
    if a.tool == "topic" {
        let t = |r: &Route| r.args.get("topic").and_then(|x| tools.topic(x).ok()).map(|p| p.topic);
        return t(a).is_some() && t(a) == t(b);
    }
    matches!((tools.run(a), tools.run(b)), (Ok(x), Ok(y)) if x == y)
}

/// Asks for paraphrases of one question (`--paraphrase-intents`).
pub(crate) const PARAPHRASE: &str = "You rewrite one question about the Arknights story the way different players would ask it on \
Reddit or Discord. Vary the wording, length and register: casual, lowercase, slang, a typo, a short one, one with a line of \
context before the question. Every rewrite must ask for exactly the same thing about the same subject (the same event, \
character, operator, Integrated Strategies run or topic, by its name or a common nickname), so that the same answer fits \
it. Output a JSON array of strings only.";

/// `--paraphrase-intents OUT`: `--paraphrases` rewrites of every table-labelled question of the TRAINING part of the
/// split (canon and topic included), each routed by the model router and marked verified when it reaches the source's
/// tool and target. Resumable by source question.
pub(crate) async fn paraphrase_intents(a: &Args, tools: &Tools, llm: &Llm, out_path: &std::path::Path) -> Result<()> {
    let items = read_intents(&a.intents)?;
    let (train, test) = intent_split(items.len());
    let test_q: BTreeSet<&str> = test.iter().map(|&i| items[i].q.as_str()).collect();
    let done: BTreeSet<String> = std::fs::read_to_string(out_path).unwrap_or_default().lines()
        .filter_map(|l| serde_json::from_str::<Intent>(l).ok()?.source_q).collect();
    let todo: Vec<&Intent> = train.iter().map(|&i| &items[i])
        .filter(|i| i.tool != trevor::tools::RETRIEVE && !done.contains(&i.q) && !test_q.contains(i.q.as_str())).collect();
    let n = a.paraphrases.max(1);
    let grammar = format!("root ::= \"[\" item (\", \" item){{{}}} \"]\"\nitem ::= \"\\\"\" [^\"\\\\\\x00-\\x1F]{{8,220}} \"\\\"\"\n", n - 1);
    eprintln!("paraphrase-intents: {} table-labelled training questions to do, {} done, {n} each", todo.len(), done.len());
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(out_path)?;
    let (mut kept, mut all) = (0usize, 0usize);
    let started = std::time::Instant::now();
    for (k, it) in todo.iter().enumerate() {
        let c = llm.complete(&Request { system: PARAPHRASE, user: &format!("Write {n} rewrites of: {}", it.q), grammar: Some(&grammar),
                                        seed: 1, temperature: 0.8, n_predict: 3000, stop: &[] }).await?;
        let list: Vec<String> = serde_json::from_str(c.content.trim()).unwrap_or_default();
        let label = Route { tool: it.tool.clone(), args: it.args.clone() };
        let mut seen = BTreeSet::new();
        for p in list.into_iter().filter(|p| p.trim() != it.q.trim() && seen.insert(p.trim().to_lowercase())) {
            let m = router::route_model(llm, &p).await?;
            let ok = same_target(tools, &label, &m);
            all += 1;
            kept += usize::from(ok);
            let row = Intent { q: p, tool: it.tool.clone(), args: it.args.clone(), source: "synthetic".into(),
                               model_tool: (!ok).then(|| m.tool.clone()), source_q: Some(it.q.clone()), verified: Some(ok) };
            serde_json::to_writer(&mut out, &row)?;
            out.write_all(b"\n")?;
        }
        if (k + 1) % 10 == 0 {
            #[allow(clippy::cast_precision_loss)]
            let each = started.elapsed().as_secs_f64() / (k + 1) as f64;
            eprintln!("paraphrase-intents {}/{}, {kept}/{all} verified, {each:.1} s per source", k + 1, todo.len());
        }
    }
    eprintln!("paraphrase-intents done: {kept}/{all} verified");
    Ok(())
}

/// Whether the kNN vote is sure enough for the hybrid router to skip the model: the thresholds, and for a table
/// tool also arguments found in the question and a tool that answers.
pub(crate) fn knn_sure(v: &router::Vote, q: &str, d: &router::Dict, tools: &Tools, min_sim: f32, min_share: f32) -> Option<Route> {
    if v.top < min_sim || v.share < min_share {
        return None;
    }
    if v.tool == trevor::tools::RETRIEVE {
        return Some(Route::retrieve());
    }
    let r = router::extract_args(&v.tool, q, d)?;
    let works = if r.tool == "topic" { r.args.get("topic").is_some_and(|t| tools.topic(t).is_ok()) } else { tools.run(&r).is_ok() };
    works.then_some(r)
}

/// `--knn-eval`: the kNN and hybrid routers on the seeded 80/20 split of the REAL labelled intents; paraphrases
/// (`--knn-synth`) join the training part only. Thresholds are swept by leave-one-out on the training part (a
/// question's own paraphrases left out with it) and reported on the test part at the training pick.
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
pub(crate) fn knn_eval(a: &Args, tools: &Tools) -> Result<()> {
    let knn = load_knn(a, tools)?;
    let items = read_intents(&a.intents)?;
    let (train, test) = intent_split(items.len());
    let test_q: BTreeSet<&str> = test.iter().map(|&i| items[i].q.as_str()).collect();
    // Training examples: the real training questions, then paraphrases of training questions only.
    let mut tr: Vec<usize> = train.clone();
    let mut leaked = 0usize;
    for i in knn.n_real..knn.examples.len() {
        if test_q.contains(knn.origins[i].as_str()) { leaked += 1 } else { tr.push(i) }
    }
    let ex: Vec<(Vec<f32>, String)> = tr.iter().map(|&i| knn.examples[i].clone()).collect();
    let origin: Vec<&str> = tr.iter().map(|&i| knn.origins[i].as_str()).collect();
    eprintln!("knn-eval: {} real labelled (train {}, test {}), {} paraphrases in training ({leaked} of test questions refused), k {}, model {}",
              items.len(), train.len(), test.len(), tr.len() - train.len(), a.knn_k, a.knn_model_dir.display());
    let d = &knn.dict;
    let runs = |r: &Route| !r.is_retrieve() && (if r.tool == "topic" { r.args.get("topic").is_some_and(|t| tools.topic(t).is_ok()) } else { tools.run(r).is_ok() });
    let label = |i: usize| {
        let r = Route { tool: items[i].tool.clone(), args: items[i].args.clone() };
        if runs(&r) || (r.tool == "topic" && r.args.is_empty()) { r } else { Route::retrieve() }
    };
    // What the model router chose (the label unless a route-check expectation overrode it).
    let model_tool = |i: usize| items[i].model_tool.clone().unwrap_or_else(|| label(i).tool);
    let kmax = a.knn_k.max(1);
    // Neighbours once per question: training queries leave out themselves and their paraphrases.
    let train_neigh: Vec<Vec<(f32, usize)>> = (0..train.len()).map(|j| {
        let o = origin[j];
        router::neighbours(&ex, &knn.examples[train[j]].0, kmax, |x| origin[x] == o)
    }).collect();
    let test_neigh: Vec<Vec<(f32, usize)>> = test.iter().map(|&i| router::neighbours(&ex, &knn.examples[i].0, kmax, |_| false)).collect();
    let decide_knn = |v: &router::Vote, q: &str, p: KnnParams| {
        let r = router::decide(v, p, q, d);
        if runs(&r) { r } else { Route::retrieve() }
    };
    let score = |pairs: &[(Route, Route)]| {
        let acc = pairs.iter().filter(|(p, l)| p.tool == l.tool).count() as f64 / pairs.len().max(1) as f64;
        let tab = pairs.iter().filter(|(p, _)| !p.is_retrieve()).count();
        let ok = pairs.iter().filter(|(p, l)| !p.is_retrieve() && p.tool == l.tool).count();
        let lab = pairs.iter().filter(|(_, l)| !l.is_retrieve()).count();
        (acc, ok, tab, lab)
    };
    println!("kNN sweep (train, leave-one-out): min_sim min_share -> tool accuracy, table answers right/given, table labels");
    let mut rows: Vec<(KnnParams, f64, f64)> = Vec::new();
    for min_sim in [0.0f32, 0.80, 0.85, 0.88, 0.90, 0.92, 0.94] {
        for min_share in [0.5f32, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let p = KnnParams { k: a.knn_k, min_sim, min_share };
            let pairs: Vec<(Route, Route)> = train.iter().enumerate()
                .map(|(j, &i)| (decide_knn(&router::vote_from(&ex, &train_neigh[j], p.k), &items[i].q, p), label(i))).collect();
            let (acc, ok, tab, lab) = score(&pairs);
            let wrong = (tab - ok) as f64 / pairs.len().max(1) as f64;
            println!("  {min_sim:.2} {min_share:.1} -> {acc:.3} {ok}/{tab} of {lab}");
            rows.push((p, acc, wrong));
        }
    }
    // A trade, not a derivation: a wrong table answer counts twice a missed one (see round 1).
    let picked = rows.iter().max_by(|x, y| (x.1 - 2.0 * x.2).total_cmp(&(y.1 - 2.0 * y.2))).map_or(
        KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share }, |r| r.0);
    let flags = KnnParams { k: a.knn_k, min_sim: a.knn_min_sim, min_share: a.knn_min_share };
    for (what, p) in [("the training pick", picked), ("the flags", flags)] {
        let pairs: Vec<(Route, Route)> = test.iter().enumerate()
            .map(|(j, &i)| (decide_knn(&router::vote_from(&ex, &test_neigh[j], p.k), &items[i].q, p), label(i))).collect();
        let (acc, ok, tab, lab) = score(&pairs);
        let agree = pairs.iter().zip(&test).filter(|(pl, i)| pl.0.tool == model_tool(**i)).count();
        let right_lab = pairs.iter().filter(|(p, l)| !l.is_retrieve() && p.tool == l.tool).count();
        println!("\nkNN on test at {what}: min_sim {:.2}, min_share {:.1} (k {})", p.min_sim, p.min_share, p.k);
        println!("  tool accuracy {acc:.3}; agreement with the model router {:.3} ({agree}/{}); table answers {ok} right of {tab} given; \
table recall {right_lab}/{lab}", agree as f64 / test.len() as f64, test.len());
        let tools_seen: BTreeSet<String> = pairs.iter().flat_map(|(p, l)| [p.tool.clone(), l.tool.clone()]).collect();
        println!("  per tool: label n, kNN n, right, precision, recall");
        for t in &tools_seen {
            let ln = pairs.iter().filter(|(_, l)| &l.tool == t).count();
            let pn = pairs.iter().filter(|(p, _)| &p.tool == t).count();
            let r = pairs.iter().filter(|(p, l)| &p.tool == t && &l.tool == t).count();
            println!("    {t:22} {ln:4} {pn:4} {r:4} {:.3} {:.3}", r as f64 / pn.max(1) as f64, r as f64 / ln.max(1) as f64);
        }
        for ((pr, lab), &i) in pairs.iter().zip(&test) {
            if !pr.is_retrieve() && pr.tool != lab.tool {
                println!("  wrong: {} {} (label {}): {}", pr.tool, serde_json::to_string(&pr.args)?, lab.tool, items[i].q);
            }
        }
    }
    // Hybrid: kNN when sure, else the model router's own choice (its recorded label), so no model runs here.
    let hybrid = |neigh: &[(f32, usize)], i: usize, hs: f32, hsh: f32| -> (String, bool) {
        let v = router::vote_from(&ex, neigh, a.knn_k);
        match knn_sure(&v, &items[i].q, d, tools, hs, hsh) {
            Some(r) => (r.tool, true),
            None => (model_tool(i), false),
        }
    };
    println!("\nhybrid sweep (train, leave-one-out): min_sim min_share -> agreement with the model router, share with no LLM call");
    let mut hrows = Vec::new();
    for hs in [0.0f32, 0.60, 0.70, 0.75, 0.80, 0.85, 0.88, 0.90, 0.92, 0.94, 0.96] {
        for hsh in [0.6f32, 0.7, 0.8, 0.9, 1.0] {
            let r: Vec<(String, bool)> = train.iter().enumerate().map(|(j, &i)| hybrid(&train_neigh[j], i, hs, hsh)).collect();
            let agree = r.iter().zip(&train).filter(|(x, i)| x.0 == model_tool(**i)).count() as f64 / train.len() as f64;
            let free = r.iter().filter(|x| x.1).count() as f64 / train.len() as f64;
            println!("  {hs:.2} {hsh:.1} -> {agree:.3} {free:.3}");
            hrows.push((hs, hsh, agree, free));
        }
    }
    // A trade: the most questions without an LLM call while agreeing with the model router on 0.98 of them.
    let hp = hrows.iter().filter(|r| r.2 >= 0.98).max_by(|x, y| x.3.total_cmp(&y.3).then(x.2.total_cmp(&y.2)))
        .map_or((a.hybrid_min_sim, a.hybrid_min_share), |r| (r.0, r.1));
    for (what, (hs, hsh)) in [("the training pick", hp), ("the flags", (a.hybrid_min_sim, a.hybrid_min_share))] {
        let r: Vec<(String, bool)> = test.iter().enumerate().map(|(j, &i)| hybrid(&test_neigh[j], i, hs, hsh)).collect();
        let agree = r.iter().zip(&test).filter(|(x, i)| x.0 == model_tool(**i)).count();
        let free = r.iter().filter(|x| x.1).count();
        let labs: Vec<Route> = test.iter().map(|&i| label(i)).collect();
        let tab = r.iter().filter(|(t, _)| t != trevor::tools::RETRIEVE).count();
        let ok = r.iter().zip(&labs).filter(|((t, _), l)| t != trevor::tools::RETRIEVE && *t == l.tool).count();
        let nlab = labs.iter().filter(|l| !l.is_retrieve()).count();
        let knn_wrong = r.iter().zip(&test).filter(|(x, i)| x.1 && x.0 != model_tool(**i)).count();
        println!("\nhybrid on test at {what}: min_sim {hs:.2}, min_share {hsh:.1}");
        println!("  agreement with the model router {:.3} ({agree}/{}); no LLM call {:.3} ({free}/{}); kNN decisions that differ from \
the model {knn_wrong}; table answers {ok} right of {tab}; table recall {ok}/{nlab}", agree as f64 / test.len() as f64, test.len(),
                 free as f64 / test.len() as f64, test.len());
    }
    let model_ok = test.iter().filter(|&&i| model_tool(i) == label(i).tool).count();
    println!("\nmodel router alone on test: tool accuracy against the labels {:.3} ({model_ok}/{})", model_ok as f64 / test.len() as f64, test.len());
    Ok(())
}
