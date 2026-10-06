//! The one-process batch modes over a JSONL question file (`--batch` with `--show-expansion`, `--form-only`,
//! `--detect-only`, `--route-only`, `--scoped-only`, `--detect-identity`, or alone to answer every question). Each
//! prints one line per question, or for answers appends one record per question to `--out` (resumable).

use std::collections::{BTreeSet, HashMap};
use std::io::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use trevor::goldgen::llm::Llm;
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;
use trevor::tools::{Tools, norm};

use crate::answer::{Answer, Tables, answer_one};
use crate::chrono::{
    Chrono, chronology_for, identity_expand, identity_notes, label_links, load_chrono, load_chrono_cached, proper_name,
};
use crate::cli::{Args, RouterKind, deep_for};
use crate::detect::{asks_admire, asks_identity, asks_relation, asks_scene, asks_when, is_opinion_question, term_of};
use crate::form::{Flags, question_form};
use crate::retrieval::{
    Names, Runtimes, expand_names, load_names, named_characters, names_proper_word, one_operator, one_operator_wide,
    opinion_needs_candidates, people_named,
};
use crate::routing::{is_ending_route, route_question};
use crate::scenes::{character_scenes, compose_evidence, person_forms, scoped_scenes};
use crate::server::connect_or_spawn;

/// The non-blank lines of a batch file, each parsed as JSON. The file is read up front; a line that does not parse
/// fails when the loop reaches it, after the lines before it have printed.
pub(crate) fn batch_items(path: &Path) -> Result<Vec<Result<serde_json::Value>>> {
    Ok(std::fs::read_to_string(path)?.lines().filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).map_err(anyhow::Error::from)).collect())
}

/// A batch item's `qid` and `question`, empty when absent.
pub(crate) fn qid_question(v: &serde_json::Value) -> (&str, &str) {
    (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default())
}

/// `--show-expansion`: each question whose name expansion changes it, as "question<TAB>expanded".
pub(crate) fn show_expansion(a: &Args, tools: &Tools) -> Result<()> {
    let rt = Runtime::load(&a.runtime, false, false, false)?;
    let qs: Vec<String> = match &a.batch {
        Some(b) => std::fs::read_to_string(b)?.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter_map(|v| v["question"].as_str().map(str::to_owned)).collect(),
        None => a.question.clone().into_iter().collect(),
    };
    for q in qs {
        let e = expand_names(&rt, tools, &q);
        if e != q {
            println!("{q}\t{e}");
        }
    }
    Ok(())
}

/// `--form-only`: the question form call alone, "qid<TAB>form<TAB>evidence" (and the flags with `--model-flags`).
pub(crate) async fn form_only(a: &Args, tools: &Tools) -> Result<()> {
    let rt = Runtime::load(&a.runtime, false, false, false)?;
    let (llm, _server) = connect_or_spawn(a).await?;
    let qs: Vec<(String, String)> = match &a.batch {
        Some(b) => std::fs::read_to_string(b)?.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter_map(|v| Some((v["qid"].as_str().unwrap_or_default().to_owned(), v["question"].as_str()?.to_owned()))).collect(),
        None => a.question.clone().map(|q| (String::new(), q)).into_iter().collect(),
    };
    for (id, q) in qs {
        let e = if a.no_name_expansion { q.clone() } else { expand_names(&rt, tools, &q) };
        let f = question_form(&llm, &e, a.form_v1, a.model_flags).await?;
        match f.flags {
            // With the flags: the model's flags, then the keyword tests' (to compare).
            Some(fl) => println!("{id}\t{}\t{}\t{}\t{}", f.form, f.evidence.unwrap_or_default(), fl.show(),
                                 Flags::keywords(&e).show()),
            None => println!("{id}\t{}\t{}", f.form, f.evidence.unwrap_or_default()),
        }
    }
    Ok(())
}

/// `--detect-only --batch`: which detectors fire on each question, "qid<TAB>detections", with no model and no index.
pub(crate) fn detect_only(a: &Args, tools: &Tools, b: &Path) -> Result<()> {
    // The chunk store alone, for the proper-name check of the scoped scenes and the opinion gate's named characters.
    let store_rt = Runtime::load(&a.runtime, false, false, false).ok();
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        let t = tools;
        let mut f: Vec<String> = Vec::new();
        if let Some(r) = is_ending_route(q, a, t) { f.push(format!("is_ending={}", serde_json::to_string(&r.args)?)); }
        if let Some(c) = t.appearance_question(q) { f.push(format!("appearances={c}")); }
        if let Some(x) = t.cross_ref_question(q) { f.push(format!("cross_ref={x}")); }
        if t.design_basis(q).is_some() { f.push("design_basis".into()); }
        if let Some(term) = term_of(q) { f.push(format!("term={term}")); }
        if asks_relation(q) {
            let people = store_rt.as_ref().map_or(0, |rt| people_named(rt, t, q).len());
            f.push(if t.names_group_topic(q) { "relation-group".into() } else { format!("relation(people={people}{})",
                if asks_admire(q) { ",admire" } else { "" }) });
        }
        if Flags::keywords(q).asks_time && asks_when(q) && !chronology_for(q, load_chrono_cached()).is_empty() { f.push("chrono".into()); }
        if let Some(tp) = t.named_topic(q) { f.push(format!("named_topic={}", tp.topic)); }
        if asks_identity(q) { if let Some(c) = t.answer_constraint(q) { f.push(format!("constraint={c:?}")); } }
        if t.broad_topic_question(q) { f.push("broad_topic".into()); }
        if let Some(g) = t.event_in_question(q) {
            let gid = g["groupId"].as_str().unwrap_or_default();
            let who: Vec<String> = t.event_characters(q, gid, g["name"].as_str().unwrap_or_default()).into_iter()
                .filter(|(n, _)| store_rt.as_ref().is_none_or(|rt| proper_name(rt, n))).map(|(n, _)| n).collect();
            f.push(if who.is_empty() { format!("event={gid}") } else { format!("scoped={gid}:{}", who.join("|")) });
        }
        if let Some(rt) = store_rt.as_ref() {
            let e = expand_names(rt, t, q);
            if !opinion_needs_candidates(rt, t, &e, !named_characters(rt, t, &e).is_empty()) {
                f.push(if is_opinion_question(&e) && !names_proper_word(rt, &e) { "opinion-gate".into() } else { "opinion-gate-wide".into() });
            }
        }
        if let Some(rt) = store_rt.as_ref().filter(|_| t.event_in_question(q).is_none() && t.appearance_question(q).is_none()) {
            if let [k] = people_named(rt, t, q).as_slice() {
                let forms: Vec<String> = person_forms(t, k).iter().map(|x| norm(x)).collect();
                if !t.names_group_topic(k) && asks_scene(q, &forms) { f.push(format!("charscope={k}")); }
            }
        }
        if let Some(op) = one_operator(q, t) { f.push(format!("one_operator={op}")); }
        else if let Some(op) = one_operator_wide(q, t) { f.push(format!("one_operator_wide={op}")); }
        println!("{id}\t{}", f.join(" "));
    }
    Ok(())
}

/// `--route-only --batch FILE`: one line per question, "qid<TAB>route<TAB>first line of the route-only output", in
/// one process (2026-10-03; the per-question process loads the tools each time).
pub(crate) async fn route_only(a: &Args, tables: &mut Tables, b: &Path) -> Result<()> {
    let llm = if matches!(a.router, RouterKind::Model | RouterKind::Hybrid) { Some(connect_or_spawn(a).await?) } else { None };
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_question(q, a, &tables.tools, tables.knn.as_mut(), llm.as_ref().map(|x| &x.0)).await?;
        let head = r.text.as_deref().map_or_else(|| {
            // `--lore-only`: whether the off-topic sentence would be added (the question names nothing Trevor knows).
            let off = if a.lore_only && !tables.tools.names_anything(q) { " (off-topic rule)" } else { "" };
            format!("RETRIEVAL{}{off}", r.passage_notes())
        }, |t| format!("TABLE {}", t.lines().next().unwrap_or_default()));
        println!("{id}\t{}\t{head}", serde_json::to_string(&r.route)?);
    }
    Ok(())
}

/// `--scoped-only --batch`: the composed evidence, scoped scenes and character scenes each question would get.
pub(crate) fn scoped_only(rts: &mut Runtimes, tools: &Tools, b: &Path) -> Result<()> {
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        if let Some(ps) = v["passages"].as_array() {
            let ps: Vec<String> = ps.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect();
            let names = load_names();
            for (cid, label, _) in compose_evidence(&mut rts.main, &load_chrono(), &names, q, &ps) {
                println!("{id}\tcompose\t{cid}\t{label}");
            }
        }
        if let Some((e, who, rows)) = scoped_scenes(&mut rts.main, tools, q) {
            let ids: Vec<&str> = rows.iter().map(|&r| rts.main.store.chunks[r].chunk_id.as_str()).collect();
            println!("{id}\t{e}\t{}\t{}", who.join("|"), ids.join(" "));
        }
        if let Some((who, rows)) = character_scenes(&mut rts.main, &rts.args, tools, q) {
            let ids: Vec<String> = rows.iter().map(|(r, _, n)| format!("{}:{n}", rts.main.store.chunks[*r].chunk_id)).collect();
            println!("{id}\tcharacter\t{who}\t{}", ids.join(" "));
        }
    }
    Ok(())
}

/// `--detect-only --batch` with the index: which identity questions would get identity notes from their plain hybrid
/// top k over P3b or P4 (the prefilter of the 2026-10-03 A/B; the answer's own passages may differ by the form's query).
pub(crate) fn detect_identity(a: &Args, rts: &mut Runtimes, b: &Path) -> Result<()> {
    let chrono = load_chrono();
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        if !asks_identity(q) {
            continue;
        }
        let cfg = RetrievalConfig { mode: Mode::Hybrid, k: a.k, ..RetrievalConfig::default() };
        let mut fired = Vec::new();
        for p4 in [false, true] {
            let rt = if p4 { rts.pick(Some(true))? } else { &mut rts.main };
            let rows: Vec<usize> = rt.retrieve(q, &cfg)?.iter().map(|h| h.row).collect();
            if !identity_notes(rt, &rows, &chrono).is_empty() {
                fired.push(if p4 { "p4" } else { "p3b" });
            }
            if !label_links(rt, &rows, &chrono).is_empty() {
                fired.push(if p4 { "labels-p4" } else { "labels-p3b" });
            }
            if identity_expand(rt, &chrono, q, rows.clone(), 2).0.len() > rows.len() {
                fired.push(if p4 { "expand-p4" } else { "expand-p3b" });
            }
        }
        println!("{id}\t{}", fired.join(" "));
    }
    Ok(())
}

/// `--batch` alone: answer every question not yet in `--out`, appending one answer record per question.
pub(crate) async fn answer_batch(a: &Args, rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono,
                            tables: &mut Tables, batch: &Path) -> Result<()> {
    let done: BTreeSet<String> = std::fs::read_to_string(&a.out)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<Answer>(l).ok()?.qid)
        .collect();
    let items: Vec<serde_json::Value> = std::fs::read_to_string(batch)
        .with_context(|| format!("reading {}", batch.display()))?
        .lines()
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()?;
    let mut out = std::fs::OpenOptions::new().create(true).append(true).open(&a.out)?;
    let todo: Vec<&serde_json::Value> =
        items.iter().filter(|i| !done.contains(i["qid"].as_str().unwrap_or(""))).collect();
    eprintln!("ask: {} to do, {} done, model {}", todo.len(), done.len(), llm.model);
    let started = std::time::Instant::now();
    for (n, it) in todo.iter().enumerate() {
        let q = it["question"].as_str().context("item without question")?;
        let mut ans = answer_one(rts, llm, names, chrono, q, a, tables).await?;
        ans.qid = it["qid"].as_str().map(str::to_owned);
        serde_json::to_writer(&mut out, &ans)?;
        out.write_all(b"\n")?;
        if (n + 1) % 10 == 0 {
            #[allow(clippy::cast_precision_loss)]
            let each = started.elapsed().as_secs_f64() / (n + 1) as f64;
            eprintln!("ask {}/{}, {each:.1} s each", n + 1, todo.len());
        }
    }
    Ok(())
}
