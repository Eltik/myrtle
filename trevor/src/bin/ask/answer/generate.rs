//! One model call over a question's passages: the prompt, the answer (asked again with twice the token limit when it
//! is cut off), and the mapping of its [n] citations back to chunk ids.

use std::collections::HashMap;

use anyhow::Result;
use trevor::corpus::chunk::Chunk;
use trevor::llm::{Llm, Request};
use trevor::search::runtime::Runtime;
use trevor::tools::norm;

use crate::chrono::{Chrono, chronology_for, time_evidence_block, timeline_notes};
use crate::cli::{Args, form_on};
use crate::detect::{
    asks_age, asks_origin, asks_relation, asks_term, asks_when, is_open_listing,
    is_opinion_question,
};
use crate::form::{Flags, Form};
use crate::prompts::{
    AGE_RULE, ART_RULE, CANON_RULE, CHAR_LABEL, CHAR_RULE_TAIL, DATE_RULE, INFER_RULE, OPINION_FORM_RULE, OPINION_RULE,
    ORIGIN_RULE, RELATION_RULE, SCOPED_RULE_HEAD, SCOPED_RULE_TAIL, SOURCE_RULE, SYNTH_RULE, SYSTEM, TERM_RULE,
    TIME_EVIDENCE_TAIL, TIME_RULE, YES_NO_RULE,
};
use crate::labels::{Names, story_label};
use crate::retrieval::{Pre, passages};

use super::Answer;

/// The chunk ids an answer cites, in citation order, and how many citations point at no passage. "[4]", "[4, 5]" and
/// "[4][5]" all cite; anything else in brackets is prose. Numbers 1 to `off` are the passages before the retrieved
/// rows, then the rows, then `post`.
fn citations(text: &str, off: usize, first: &[&Pre], rows: &[usize], post: &[&Pre], rt: &Runtime)
    -> (Vec<String>, usize) {
    let mut cited = Vec::new();
    let mut invalid = 0usize;
    let mut i = 0;
    while let Some(open) = text[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = text[start..].find(']') else { break };
        // "[4]", "[4, 5]" and "[4][5]" all cite; anything else in brackets is prose.
        let inner = &text[start..start + close];
        let nums: Vec<usize> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty())
            .filter_map(|t| t.parse().ok()).collect();
        if !nums.is_empty() && nums.len() == inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).count() {
            for n in nums {
                if (1..=off).contains(&n) {
                    let id = first[n - 1].id.clone();
                    if !cited.contains(&id) {
                        cited.push(id);
                    }
                    continue;
                }
                match rows.get(n.wrapping_sub(1 + off)) {
                    Some(&r) => {
                        let id = rt.store.chunks[r].chunk_id.clone();
                        if !cited.contains(&id) {
                            cited.push(id);
                        }
                    }
                    None => match post.get(n.wrapping_sub(1 + off + rows.len())) {
                        Some(p) => {
                            if !cited.contains(&p.id) {
                                cited.push(p.id.clone());
                            }
                        }
                        None => invalid += 1,
                    },
                }
            }
        }
        i = start + close;
    }
    (cited, invalid)
}

/// A retrieved passage's label. The source kind leads it, so an answer can say where a claim comes from (SOURCE_RULE):
/// an operator file, a dossier or event summary Trevor wrote, a typed unit by its own first line, or a story.
pub(crate) fn passage_label(names: &HashMap<String, Names>, c: &Chunk) -> String {
    match c.group_id.as_str() {
        "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
        "profile" => format!("Trevor's character dossier (a summary written by Trevor): {}", c.story_id),
        "summary" => format!("Trevor's event summary (written by Trevor): {}", c.story_id),
        "module" | "voice" | "is" | "enemy" | "skin" | "item" | "art" | "gametext" =>
            c.text.lines().next().unwrap_or_default().to_owned(),
        _ => story_label(names, &c.story_id),
    }
}

/// One answer from the passages retrieved for `search`, answering `q`. A topic summary, when the router chose one,
/// is passage [1] and the retrieved passages follow.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn answer_from(rt: &mut Runtime, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str,
                     search: &str, a: &Args, pre: &[Pre], kind: Option<&str>, form: Option<&Form>) -> Result<Answer> {
    let opinion_form = form.is_some_and(|f| f.form == "opinion");
    let (mut rows, lore) = passages(rt, search, a, kind, form)?;
    // Under a spoiler horizon (`--serve` jobs): only chunks released at or before it, and of the generated passages
    // only those that are themselves such a chunk (a topic, dossier or game-data passage is not).
    let kept: Vec<Pre>;
    let pre = match rt.horizon.clone() {
        Some(h) => {
            rows.retain(|&r| rt.allows_row(&h, r));
            kept = pre.iter().filter(|p| rt.store.row(&p.id).is_some_and(|r| rt.allows_row(&h, r))).cloned().collect();
            &kept[..]
        }
        None => pre,
    };
    let scoped: Vec<&str> = pre.iter().filter(|p| p.scoped).map(|p| p.id.as_str()).collect();
    if !scoped.is_empty() {
        rows.retain(|&r| !scoped.contains(&rt.store.chunks[r].chunk_id.as_str()));
    }
    let (first, post): (Vec<&Pre>, Vec<&Pre>) = pre.iter().partition(|p| !p.last);
    let off = first.len();
    let flags = Flags::keywords(q);
    let time_q = !a.no_route && flags.asks_time;
    let synth_q = !a.no_route && flags.asks_synthesis;
    let mut user = String::from("PASSAGES\n");
    let mut texts: Vec<String> = Vec::with_capacity(off + rows.len() + post.len());
    for (i, p) in first.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", i + 1, p.label, p.text));
        texts.push(format!("({})\n{}", p.label, p.text));
    }
    for (i, &r) in rows.iter().enumerate() {
        let c = &rt.store.chunks[r];
        let label = passage_label(names, c);
        // A file found through another name of its character says so: otherwise, with "Margaret vs Degenbrecher" and
        // Nearl the Radiant Knight's file among the passages, the answer can say it found nothing about "a character
        // named Margaret".
        let label = match form.and_then(|f| f.named.iter().find(|(sid, asked, file)| *sid == c.story_id
            && norm(asked) != norm(file))) {
            Some((_, asked, file)) => format!("{label}; this is the file of {file}, whom the question calls {asked}"),
            None => label,
        };
        user.push_str(&format!("\n[{}] ({label})\n{}\n", i + 1 + off, c.text));
        texts.push(format!("({label})\n{}", c.text));
    }
    for (i, p) in post.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", off + rows.len() + i + 1, p.label, p.text));
        texts.push(format!("({})\n{}", p.label, p.text));
    }
    if time_q {
        user.push_str(&timeline_notes(rt, chrono, &rows, off));
    }
    let subject_lines = if time_q && !a.no_chrono_subject && asks_when(q) { chronology_for(q, chrono) }
        else { String::new() };
    let time_evidence = !a.no_time_evidence && !subject_lines.is_empty();
    if !time_evidence {
        user.push_str(&subject_lines);
    }
    if time_evidence {
        user.push_str(&time_evidence_block(&subject_lines));
    }
    user.push_str(&format!("\nQUESTION: {q}"));
    if time_evidence {
        user.push_str(TIME_EVIDENCE_TAIL);
    }
    let started = std::time::Instant::now();
    let mut system = SYSTEM.to_owned();
    if pre.iter().any(|p| p.rule) {
        system.push_str(INFER_RULE);
        if !a.no_age_estimate && asks_age(q) {
            system.push_str(AGE_RULE);
        }
    }
    if rows.iter().any(|&r| rt.store.chunks[r].group_id == "art") {
        system.push_str(ART_RULE);
    }
    if time_q {
        system.push_str(TIME_RULE);
        if !subject_lines.is_empty() {
            system.push_str(DATE_RULE);
        }
    }
    if !a.no_origin_chain && !a.no_route && asks_origin(q) {
        system.push_str(ORIGIN_RULE);
    }
    if !a.no_term_rule && !a.no_route && asks_term(q) {
        system.push_str(TERM_RULE);
    }
    if a.relation_rule && !a.no_route && asks_relation(q) {
        system.push_str(RELATION_RULE);
    }
    if !scoped.is_empty() {
        let tail = if pre.iter().any(|p| p.scoped && p.label.contains(CHAR_LABEL)) { CHAR_RULE_TAIL }
            else { SCOPED_RULE_TAIL };
        system.push_str(&format!("{SCOPED_RULE_HEAD}{}{tail}", scoped.len()));
    }
    if synth_q {
        system.push_str(SYNTH_RULE);
    }
    if !a.no_route && flags.asks_canon {
        system.push_str(CANON_RULE);
    }
    if form.is_some_and(|f| f.gated) {
        system.push_str(OPINION_RULE);
    } else if opinion_form {
        system.push_str(OPINION_FORM_RULE);
    } else if !a.no_route && is_opinion_question(q) && (!form_on(a) || a.keyword_opinion) {
        system.push_str(OPINION_RULE);
    }
    if form.is_some_and(|f| f.form == "yes_no") {
        system.push_str(YES_NO_RULE);
    }
    match lore {
        // The typed units lead the passages, so the source-kind rule's "say Trevor lacks that text" does not apply: it
        // made the Angelina answer open with a decline although passage [1] was her outfit. The two parts are spelled
        // out: with the lore passages merely present, Gemma answered from [1] alone.
        Some(l) => system.push_str(&format!(" Passages [{}] to [{}] are the source text the question asks about \
            (their label names the kind); never say Trevor lacks that text. Answer in two parts. \
            First, what that source text says, citing [{}] to [{}]. \
            Then a paragraph that begins \"Lore context:\" and explains the people, places, powers and events \
            the source text mentions, from passages [{}] to [{}] (story and operator-file passages about them), \
            with citations; if those passages say nothing about what it mentions, say so in one sentence.",
            1 + off, l.units + off, 1 + off, l.units + off, l.units + 1 + off,
            l.units + l.lore.max(1) + off)),
        None => {
            if !a.no_route && flags.asks_source_text {
                system.push_str(SOURCE_RULE);
            }
        }
    }
    let mut c = llm.complete(&Request::greedy(&system, &user, None, a.n_predict, &[])).await?;
    // An answer cut off at the limit is asked again with twice the limit (`--no-answer-extend`): greedy decoding of the
    // same prompt repeats the cut answer and goes on.
    if !a.no_answer_extend && c.stop_type == "limit" {
        c = llm.complete(&Request::greedy(&system, &user, None, a.n_predict * 2, &[])).await?;
    }
    let (cited, invalid) = citations(&c.content, off, &first, &rows, &post, rt);
    let mut answer = c.content.trim().to_owned();
    if !a.no_route && is_open_listing(q) {
        answer = format!("Trevor has no complete table for this question, so this list comes only from the {} \
            passages it read and is not complete.\n\n{answer}", rows.len());
    }
    Ok(Answer {
        qid: None,
        question: q.to_owned(),
        answer,
        cited,
        passages: first.iter().map(|p| p.id.clone())
            .chain(rows.iter().map(|&r| rt.store.chunks[r].chunk_id.clone()))
            .chain(post.iter().map(|p| p.id.clone()))
            .collect(),
        invalid_citations: invalid,
        prompt_tokens: c.prompt_tokens,
        ms: started.elapsed().as_secs_f64() * 1000.0,
        generated: pre.iter().filter(|p| !p.id.starts_with("topic:")).map(|p| (p.id.clone(), p.text.clone())).collect(),
        form: form.map(|f| f.evidence.as_ref().map_or_else(|| f.form.clone(), |e| format!("{}: {e}", f.form))),
        texts,
    })
}
