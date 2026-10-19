//! The one-process batch modes over a JSONL question file (`--batch` with `--show-expansion`, `--form-only`,
//! `--detect-only`, `--route-only`, `--scoped-only`, or alone to answer every question). Each
//! prints one line per question, or for answers appends one record per question to `--out` (resumable).

use std::collections::{BTreeSet, HashMap};
use std::io::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use trevor::llm::Llm;
use trevor::search::runtime::Runtime;
use trevor::tools::Tools;

use crate::answer::{Answer, Tables, answer_one};
use crate::chrono::{
    Chrono, chronology_for, load_chrono_cached, proper_name,
};
use crate::cli::{Args, RouterKind, deep_for};
use crate::detect::{asks_admire, asks_identity, asks_relation, asks_when, is_opinion_question, term_of};
use crate::form::{Flags, question_form};
use crate::labels::Names;
use crate::people::{
    expand_names, named_characters, names_proper_word, one_operator, one_operator_wide, opinion_needs_candidates,
    people_named,
};
use crate::runtimes::Runtimes;
use crate::routing::{is_ending_route, route_question};
use crate::scenes::{character_scenes, scoped_excerpts, scoped_scenes, scoped_summaries};
use crate::llama_server::connect_or_spawn;

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

/// `--form-only`: the question form call alone, "qid<TAB>form<TAB>evidence".
pub(crate) async fn form_only(a: &Args, tools: &Tools) -> Result<()> {
    let rt = Runtime::load(&a.runtime, false, false, false)?;
    let (llm, _server) = connect_or_spawn(a).await?;
    let qs: Vec<(String, String)> = match &a.batch {
        Some(b) => std::fs::read_to_string(b)?.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter_map(|v| Some((v["qid"].as_str().unwrap_or_default().to_owned(), v["question"].as_str()?.to_owned())))
            .collect(),
        None => a.question.clone().map(|q| (String::new(), q)).into_iter().collect(),
    };
    for (id, q) in qs {
        let e = if a.no_name_expansion { q.clone() } else { expand_names(&rt, tools, &q) };
        let f = question_form(&llm, &e, a.form_v1).await?;
        println!("{id}\t{}\t{}", f.form, f.evidence.unwrap_or_default());
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
        if let Some(term) = term_of(q) { f.push(format!("term={term}")); }
        if asks_relation(q) {
            let people = store_rt.as_ref().map_or(0, |rt| people_named(rt, t, q).len());
            f.push(if t.names_group_topic(q) { "relation-group".into() } else { format!("relation(people={people}{})",
                if asks_admire(q) { ",admire" } else { "" }) });
        }
        if Flags::keywords(q).asks_time && asks_when(q) && !chronology_for(q, load_chrono_cached()).is_empty() {
            f.push("chrono".into());
        }
        if let Some((e, l)) = crate::timegap::time_gap_events(q, t, load_chrono_cached()) {
            let n = store_rt.as_ref().map_or(0, |rt| crate::timegap::time_gap_lines(rt, t, e, l).len());
            f.push(format!("timegap={}>{}:{n}", e["groupId"].as_str().unwrap_or_default(),
                           l["groupId"].as_str().unwrap_or_default()));
        }
        if asks_identity(q) { if let Some(c) = t.answer_constraint(q) { f.push(format!("constraint={c:?}")); } }
        if t.broad_topic_question(q) { f.push("broad_topic".into()); }
        if let Some(g) = t.event_in_question(q) {
            let gid = g["groupId"].as_str().unwrap_or_default();
            let who: Vec<String> = t.event_characters(q, gid, g["name"].as_str().unwrap_or_default()).into_iter()
                .filter(|(n, _)| store_rt.as_ref().is_none_or(|rt| proper_name(rt, n))).map(|(n, _)| n).collect();
            f.push(if who.is_empty() { format!("event={gid}") } else { format!("scoped={gid}:{}", who.join("|")) });
        }
        if t.asks_operators(q) {
            f.push("operators".into());
        }
        let groups = t.named_group_topics(q);
        if groups.len() >= 2 {
            f.push(format!("groups={}", groups.iter().map(|g| g.0.as_str()).collect::<Vec<_>>().join("|")));
        }
        if let Some(rt) = store_rt.as_ref() {
            let e = expand_names(rt, t, q);
            if !opinion_needs_candidates(rt, t, &e, !named_characters(rt, t, &e).is_empty()) && is_opinion_question(&e)
                && !names_proper_word(rt, &e) {
                f.push("opinion-gate".into());
            }
        }
        if let Some(rt) = store_rt
            .as_ref()
            .filter(|_| t.event_in_question(q).is_none() && t.appearance_question(q).is_none())
        {
            if let Some((k, _)) = crate::scenes::scene_subject(t, &people_named(rt, t, q), q, a.scene_verbs(),
                                                                !a.scene_subject) {
                f.push(format!("charscope={k}"));
            }
        }
        if let Some(w) = crate::fate::alive_dead_subject(q, t).filter(|_| !a.no_death_evidence) {
            f.push(format!("death_evidence={w}"));
        }
        if let Some(op) = one_operator(q, t) { f.push(format!("one_operator={op}")); }
        else if let Some(op) = one_operator_wide(q, t) { f.push(format!("one_operator_wide={op}")); }
        println!("{id}\t{}", f.join(" "));
    }
    Ok(())
}

/// `--route-only --batch FILE`: one line per question, "qid<TAB>route<TAB>first line of the route-only output", in
/// one process (a process per question would load the tools each time).
pub(crate) async fn route_only(a: &Args, tables: &mut Tables, b: &Path) -> Result<()> {
    let llm = if a.router == RouterKind::Model { Some(connect_or_spawn(a).await?) } else { None };
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_question(q, a, &tables.tools, llm.as_ref().map(|x| &x.0)).await?;
        let head = r.text.as_deref().map_or_else(|| format!("RETRIEVAL{}", r.passage_notes()),
                                                 |t| format!("TABLE {}", t.lines().next().unwrap_or_default()));
        println!("{id}\t{}\t{head}", serde_json::to_string(&r.route)?);
    }
    Ok(())
}

/// `--scoped-only --batch`: the scoped scenes and character scenes each question would get.
pub(crate) fn scoped_only(a: &Args, rts: &mut Runtimes, tools: &Tools, b: &Path) -> Result<()> {
    for v in batch_items(b)? {
        let v = v?;
        let (id, q) = qid_question(&v);
        let excerpts = if a.no_scoped_line_rank { None } else {
            scoped_excerpts(&mut rts.main, &rts.args, tools, q, crate::scenes::CHAR_TOKENS, a.scene_verbs())
        };
        if let Some((e, who, rows)) = &excerpts {
            let ids: Vec<String> = rows.iter().map(|(r, _, n)| format!("{}:{n}", rts.main.store.chunks[*r].chunk_id))
                .collect();
            println!("{id}\t{e}\texcerpts:{who}\t{}", ids.join(" "));
        } else if let Some((e, who, rows)) = scoped_scenes(&mut rts.main, tools, q, crate::scenes::SCOPED_TOKENS) {
            let ids: Vec<&str> = rows.iter().map(|&r| rts.main.store.chunks[r].chunk_id.as_str()).collect();
            println!("{id}\t{e}\t{}\t{}", who.join("|"), ids.join(" "));
            if a.scoped_summaries {
                let s: Vec<String> = scoped_summaries(&rts.main, tools, &rows).into_iter().map(|(s, _)| s).collect();
                println!("{id}\tsummaries\t{}", s.join(" "));
            }
        }
        if let Some((e, l)) = crate::timegap::time_gap_events(q, tools, crate::chrono::load_chrono_cached()) {
            let ids: Vec<String> = crate::timegap::time_gap_lines(&rts.main, tools, e, l).into_iter()
                .map(|(r, i)| format!("{}:{i}", rts.main.store.chunks[r].chunk_id)).collect();
            println!("{id}\ttimegap\t{}>{}\t{}", e["groupId"].as_str().unwrap_or_default(),
                     l["groupId"].as_str().unwrap_or_default(), ids.join(" "));
        }
        if let Some((who, rows)) = character_scenes(&mut rts.main, &rts.args, tools, q, crate::scenes::CHAR_TOKENS,
                                                    a.scene_verbs(), !a.scene_subject) {
            let ids: Vec<String> =
                rows.iter().map(|(r, _, n)| format!("{}:{n}", rts.main.store.chunks[*r].chunk_id)).collect();
            println!("{id}\tcharacter\t{who}\t{}", ids.join(" "));
        }
    }
    Ok(())
}

/// `--batch` alone: answer every question not yet in `--out`, appending one answer record per question.
pub(crate) async fn answer_batch(a: &Args, rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>,
                            chrono: &Chrono, tables: &mut Tables, batch: &Path) -> Result<()> {
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

/// `--identity-scan DUMP`: the identity-claim check (`answer/identity.rs`) over the answers of a `--dump-prompt` file,
/// without a model: each answer call's passages and question are read back from its rendered prompt. Prints every
/// answer the check would change (question, claims, the answer after) and the counts. The referent checks and the
/// passage-label fix are scanned and counted only while their switches are on; with all three off the output is the
/// identity-claim scan alone.
pub(crate) fn identity_scan(a: &Args, tools: &Tools, dump: &Path) -> Result<()> {
    let referent_on = !a.no_referent_check || !a.no_quote_check;
    let text = std::fs::read_to_string(dump).with_context(|| format!("reading {}", dump.display()))?;
    let (mut answers, mut changed, mut referent, mut labels) = (0usize, 0usize, 0usize, 0usize);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let (Some(prompt), Some(content)) = (v["prompt"].as_str(), v["content"].as_str()) else { continue };
        let Some(user) = prompt.split("<|turn>user\n").nth(1).map(|u| u.split("<turn|>").next().unwrap_or(u)) else {
            continue;
        };
        let Some(body) = user.strip_prefix("PASSAGES\n") else { continue };
        let Some(qat) = body.rfind("\nQUESTION: ") else { continue };
        let q = body[qat + 11..].lines().next().unwrap_or_default();
        // Passage headers: "\n[n] (" at a line start, numbered 1, 2, ... in order.
        let mut starts: Vec<usize> = Vec::new();
        let mut n = 1;
        let mut from = 0;
        while let Some(i) = body[from..qat].find(&format!("\n[{n}] (")) {
            starts.push(from + i + 1);
            from += i + 1;
            n += 1;
        }
        let texts: Vec<String> = starts.iter().enumerate().map(|(k, &s)| {
            let end = starts.get(k + 1).copied().unwrap_or(qat);
            let p = &body[s..end];
            p[p.find(' ').map_or(0, |i| i + 1)..].trim_end().to_owned()
        }).collect();
        if texts.is_empty() {
            continue;
        }
        answers += 1;
        if let Some((after, claims)) = crate::answer::identity::check(content.trim(), q, &texts, tools) {
            changed += 1;
            println!("Q: {q}\nclaims: {claims:?}\nbefore: {}\nafter: {after}\n", content.trim());
        }
        if let Some((after, found)) = crate::answer::referent::check(content.trim(), q, &texts, tools,
                                                                       !a.no_referent_check, !a.no_quote_check)
            .filter(|_| referent_on) {
            referent += 1;
            println!("Q: {q}\nreferent: {found:?}\nbefore: {}\nafter: {after}\n", content.trim());
        }
        if !a.no_passage_label_fix && crate::answer::cite_words_fix(content.trim()).is_some() {
            labels += 1;
        }
    }
    let mut tail = String::new();
    if referent_on {
        tail.push_str(&format!(", {referent} by the referent checks"));
    }
    if !a.no_passage_label_fix {
        tail.push_str(&format!(", {labels} by the passage-label fix"));
    }
    println!("{answers} answer calls, {changed} changed by the identity-claim check{tail}");
    Ok(())
}
