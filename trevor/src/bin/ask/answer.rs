//! Answering one question: the table answer or the retrieval answer, the follow-up calls (retry, answer
//! check, composed evidence) and the answer record.

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use trevor::corpus::chunk::Chunk;
use trevor::goldgen::llm::{Llm, Request};
use trevor::router::Knn;
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;
use trevor::tools::{AnswerConstraint, Route, Tools, TopicPassage, norm};

use crate::chrono::{
    Chrono, chronology_for, identity_expand, identity_label, identity_notes, label_links, time_evidence_block,
    timeline_notes,
};
use crate::cli::{Args, RouterKind, deep_for, dossier_allowed, form_on, retrieval_plan};
use crate::detect::{
    asks_admire, asks_age, asks_identity, asks_origin, asks_relation, asks_term, asks_when, declines, is_open_listing,
    is_opinion_question, leaves_open, term_of,
};
use crate::form::{Flags, Form, question_form, story_words};
use crate::prompts::{
    AGE_RULE, ART_RULE, CANON_RULE, CHAR_LABEL, CHAR_RULE_TAIL, COMPOSE_RULE, DATE_RULE, INFER_RULE, LORE_ONLY_RULE,
    OPINION_FORM_RULE, OPINION_RULE, ORIGIN_RULE, OVERVIEW_RULE, PARTIAL, RELATION_RULE, REWRITE, SCOPED_RULE_HEAD,
    SCOPED_RULE_TAIL, SOURCE_RULE, SYNTH_RULE, SYSTEM, TERM_RULE, TIME_EVIDENCE_TAIL, TIME_RULE, YES_NO_RULE,
    YES_NO_VERDICT_RULE,
};
use crate::retrieval::{
    FALLBACK_KINDS, RECORD_KINDS, Names, Pre, Runtimes, expand_names, file_owner, named_characters, names_proper_word,
    one_operator, one_operator_wide, opinion_needs_candidates, passages, people_named, story_label, typed_owner,
};
use crate::routing::{is_ending_passages, route_question};
use crate::scenes::{SCOPED_REST, character_scenes, compose_evidence, line_pool, scoped_scenes};
use crate::terms::{term_glossary, term_lookup};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Answer {
    pub(crate) qid: Option<String>,
    pub(crate) question: String,
    pub(crate) answer: String,
    /// Chunk ids of the passages the answer cites, in citation order.
    pub(crate) cited: Vec<String>,
    /// Chunk ids given to the model, in passage order.
    pub(crate) passages: Vec<String>,
    pub(crate) invalid_citations: usize,
    pub(crate) prompt_tokens: u64,
    pub(crate) ms: f64,
    /// The search query of the retry, when the first answer declined and the retry was used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) retry_query: Option<String>,
    /// The question form and evidence query, when the form call ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) form: Option<String>,
    /// The text of each generated passage only `--lore v2` adds (new dossier, game data), by passage id, for the judges.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub(crate) generated: std::collections::BTreeMap<String, String>,
}

pub(crate) fn table_only(q: &str, text: String) -> Answer {
    Answer { qid: None, question: q.to_owned(), answer: text, cited: Vec::new(), passages: Vec::new(),
             invalid_citations: 0, prompt_tokens: 0, ms: 0.0, retry_query: None, form: None, generated: Default::default() }
}

/// The table tools and the kNN router when `--router knn` loaded it.
pub(crate) struct Tables {
    pub(crate) tools: Tools,
    pub(crate) knn: Option<Knn>,
    /// The topic summary the router chose for the single question, routed before the index loads.
    pub(crate) topic: Option<TopicPassage>,
    /// The route of the single question, routed before the index loads.
    pub(crate) route: Option<Route>,
}

/// One answer, with the answer category check (default since 2026-10-05, `--no-answer-check`): a who/which question that constrains its
/// answer's category ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") and whose
/// answer's first named character fails it by the game data (Sakiko Togawa is a playable operator) is asked once more
/// with a passage saying so; when the second answer's first name fails too, the data note is appended to it.
pub(crate) async fn answer_one(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                    tables: &mut Tables) -> Result<Answer> {
    let first = answer_one_inner(rts, llm, names, chrono, q, a, tables).await?;
    if a.no_answer_check || a.no_route || first.passages.is_empty() || !asks_identity(q) {
        return Ok(first);
    }
    let Some(c) = tables.tools.answer_constraint(q) else { return Ok(first) };
    let Some((name, reason)) = tables.tools.failing_answer_name(q, &first.answer, &c) else {
        return compose_after(rts, llm, names, chrono, q, a, tables, first).await;
    };
    let wants = match &c {
        AnswerConstraint::NonPlayable => "someone who is not a playable operator".to_owned(),
        AnswerConstraint::Playable => "a playable operator".to_owned(),
        AnswerConstraint::Nation(x) => format!("an operator from {x}"),
    };
    let note = format!("An earlier answer to this question named {name}. By the game data, {reason}, and the question asks for {wants}. Do not give {name} as the answer. Name only someone the passages show fits the question, or say that the passages do not identify one.");
    let again = Args { check_note: Some(note), routed: false, ..a.clone() };
    let mut second = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    if let Some((n2, r2)) = tables.tools.failing_answer_name(q, &second.answer, &c) {
        second.answer = format!("{}\n\nTrevor's data check: {r2}, and the question asks for {wants}, so {n2} does not fit.", second.answer);
    }
    compose_after(rts, llm, names, chrono, q, &again, tables, second).await
}

/// The evidence composition after the answer check (`compose_evidence`): only for a declined answer to a question whose
/// answer category the check reads; the composed answer is kept when it names someone and that name passes the check.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn compose_after(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                       tables: &mut Tables, done: Answer) -> Result<Answer> {
    let Some(c) = tables.tools.answer_constraint(q) else { return Ok(done) };
    if !a.compose_evidence || a.no_route || !(declines(&done.answer) || leaves_open(&done.answer)) {
        return Ok(done);
    }
    // P4x: voice lines and records name people too (Iris's "I'm looking for a girl named Mabel ... twenty years ago").
    let compose = compose_evidence(rts.pick(Some(true))?, chrono, names, q, &done.passages);
    if compose.is_empty() {
        return Ok(done);
    }
    let again = Args { compose, routed: false, ..a.clone() };
    let composed = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    if std::env::var("TREVOR_SCOPED_DEBUG").is_ok() {
        eprintln!("compose: {:?}; composed answer: {}", again.compose.iter().map(|x| &x.0).collect::<Vec<_>>(), composed.answer);
    }
    if declines(&composed.answer) || leaves_open(&composed.answer) || tables.tools.failing_answer_name(q, &composed.answer, &c).is_some() {
        return Ok(done);
    }
    Ok(composed)
}

/// Passages before the retrieved ones: the topic summary the router chose, then (`--lore v2`) the new dossier of a
/// character the question names. Under v1 the prompt is the one before, byte for byte. In order after `pre` (the
/// IS-ending passages): the topic, its event summaries (`--topic-events`), its nation (`--place-nation`), the new
/// dossier, the composed evidence (which replaces everything before it), the answer-check note and the game-data passage.
fn lead_passages(a: &Args, tools: &Tools, q: &str, topic: &Option<TopicPassage>, route: &Option<Route>, mut pre: Vec<Pre>) -> Vec<Pre> {
    let topic_label = |t: &TopicPassage| if tools.deep_entry(&t.topic).is_some() {
        format!("Trevor's topic entry (generated from the story passages, operator files and records that name it; prefer the story passages): {}", t.topic)
    } else {
        format!("Trevor's topic summary (generated; prefer the story passages): {}", t.topic)
    };
    pre.extend(topic.iter().map(|t| Pre::new(format!("topic:{}", t.topic), topic_label(t), t.text.clone())));
    // A topic summary comes with the event summaries of the 3 groups that name the topic most (items 3 and 4).
    if a.topic_events {
        if let Some(t) = &topic {
            for (gid, name, text) in tools.topic_events(&t.topic, 3) {
                pre.push(Pre::new(format!("event:{gid}"), format!("Trevor's event summary (generated; prefer the story passages): {name}"), text));
            }
        }
    }
    // A place's nation topic after it (item 5): "factions in dossoles" are Bolívar's.
    if a.place_nation {
        if let Some(n) = topic.as_ref().and_then(|t| tools.place_nation(&t.topic, !a.place_nation_summary)) {
            pre.push(Pre::new(format!("topic:{}", n.topic), format!("Trevor's topic summary (generated; prefer the story passages): {}, \
the nation of {}", n.topic, topic.as_ref().map_or("", |t| t.topic.as_str())), n.text));
        }
    }
    if let Some(d) = tools.new_dossier(q).filter(|_| dossier_allowed(q, a, route.as_ref())) {
        let label = format!("Trevor's character dossier (generated; prefer the story passages): {}", d.topic);
        pre.push(Pre::new(format!("dossier:{}", d.topic), label, d.text).after());
    }
    if !a.compose.is_empty() {
        pre.clear();
    }
    for (i, (id, label, text)) in a.compose.iter().enumerate() {
        pre.insert(i, Pre::new(id.clone(), label.clone(), text.clone()));
    }
    if let Some(note) = &a.check_note {
        pre.push(Pre::new("check".into(), "Trevor's check of an earlier answer against the game data".into(), note.clone()).after());
    }
    if let Some(g) = tools.game_data(q) {
        pre.push(Pre::new(format!("gamedata:{}", g.topic), format!("game data: {}", g.topic), g.text).after().with_rule());
    }
    pre
}

pub(crate) async fn answer_one_inner(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, a: &Args,
                          tables: &mut Tables) -> Result<Answer> {
    // The relation rule is about people (2026-10-05): not for a question naming a nation, place, race or organization topic.
    let people_only;
    let a = if a.relation_rule && !a.relation_rule_all && tables.tools.names_group_topic(q) {
        people_only = Args { relation_rule: false, ..a.clone() };
        &people_only
    } else { a };
    let mut topic = if a.routed { tables.topic.take() } else { None };
    let mut route = if a.routed { tables.route.take() } else { None };
    if !a.no_route && !a.routed {
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_question(q, a, &tables.tools, tables.knn.as_mut(), Some(llm)).await?;
        // Under a spoiler horizon a table answer (built from every story) and a topic summary are not served; the
        // question is answered from the passages the horizon lets through instead.
        if let Some(text) = r.text {
            if !rts.horizon_set() {
                return Ok(table_only(q, text));
            }
            route = Some(Route::retrieve());
        } else {
            topic = if rts.horizon_set() { None } else { r.topic };
            route = Some(r.route);
        }
    }
    // An IS-ending route: the ending's own passages first, then a short plain retrieval.
    let mut ending_pre: Vec<Pre> = Vec::new();
    if route.as_ref().is_some_and(|r| r.tool == "is_ending") {
        let r = route.take().unwrap_or_else(Route::retrieve);
        ending_pre = is_ending_passages(rts.pick(Some(true))?, &tables.tools, q, &r);
        route = Some(Route::retrieve());
    }
    let short;
    let a = if ending_pre.is_empty() { a } else {
        short = Args { max_context_tokens: 3000, ..a.clone() };
        &short
    };
    // The topic the question names, when the router chose none and retrieval is not a typed source (item 5).
    if topic.is_none() && a.named_topic && !a.no_topics && !a.no_route && a.router != RouterKind::Keywords
        && route.as_ref().is_some_and(|r| r.is_retrieve() && matches!(r.args.get("source").map(String::as_str), None | Some("story" | "any")))
        // Since 2026-10-05 only a question about the topic as a whole: the night 7 package lost held-out h004 ("What's
        // the difference between the Silverlance Pegasi and the rest of Campaign/gendarmerie knights?"), whose
        // Gendarmerie summary and event summaries displaced the story passage on the Pegasi's speed.
        && (a.named_topic_all || tables.tools.broad_topic_question(q)) {
        topic = tables.tools.named_topic(q);
    }
    if route.as_ref().is_some_and(|r| r.tool == "overview") {
        if let Some(passages) = tables.tools.overview_passages().ok().filter(|_| !rts.horizon_set()) {
            return answer_overview(llm, q, a, &passages).await;
        }
        route = Some(Route::retrieve());
    }
    let mut pre = lead_passages(a, &tables.tools, q, &topic, &route, ending_pre);
    // Term lookup (night 9): the term's spelling in P4x, its two best record chunks and a glossary, before the passages.
    let mut term_search: Option<String> = None;
    if !a.no_term_lookup && !a.no_route {
        if let Some(term) = term_of(q) {
            let rt4 = rts.pick(Some(true))?;
            if let Some((spelling, rows)) = term_lookup(rt4, &term, 2) {
                let texts: Vec<&str> = rows.iter().map(|&r| rt4.store.chunks[r].text.as_str()).collect();
                let glossary = term_glossary(&tables.tools, &texts, &term, 3, (!a.no_glossary_rarity).then_some(&*rt4));
                let mut at = 0;
                for &r in &rows {
                    let c = &rt4.store.chunks[r];
                    let label = match c.group_id.as_str() {
                        "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
                        g if RECORD_KINDS.contains(&g) || g == "voice" => c.text.lines().next().unwrap_or_default().to_owned(),
                        _ => story_label(names, &c.story_id),
                    };
                    pre.insert(at, Pre::new(c.chunk_id.clone(), format!("{label}; it names {spelling}"), c.text.clone()));
                    at += 1;
                }
                if !glossary.is_empty() {
                    let text = glossary.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join("\n");
                    pre.insert(at, Pre::new("glossary".into(), "Trevor's glossary of the terms these passages use (generated from its topic \
summaries; prefer the passages)".into(), text));
                }
                if !q.to_lowercase().contains(&spelling.to_lowercase()) {
                    term_search = Some(format!("{q} {spelling}"));
                }
            }
        }
    }
    let (p4, kind) = retrieval_plan(q, a, route.as_ref());
    let rt = if kind == Some("art") { rts.pick_art()? } else { rts.pick(p4)? };
    let two_people;
    let a = if a.relation_rule && !a.relation_rule_any && asks_relation(q) && !asks_admire(q) && people_named(rt, &tables.tools, q).len() < 2 {
        two_people = Args { relation_rule: false, ..a.clone() };
        &two_people
    } else { a };
    let expanded = if a.router == RouterKind::Keywords || a.no_name_expansion || a.no_route { q.to_owned() }
        else { expand_names(rt, &tables.tools, q) };
    let q = expanded.as_str();
    let story_q = if a.story_query && !a.no_route && a.router != RouterKind::Keywords { story_words(llm, q).await? } else { None };
    let mut form = if form_on(a) { Some(question_form(llm, q, a.form_v1, a.model_flags).await?) } else { None };
    if let Some(sq) = story_q {
        form.get_or_insert_with(|| Form { form: "fact".into(), ..Form::default() }).story = Some(sq);
    }
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion") {
        f.named = named_characters(rt, &tables.tools, q);
    }
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion" && (a.opinion_gate || (!a.no_opinion_gate && is_opinion_question(q)))) {
        if !opinion_needs_candidates(rt, &tables.tools, q, !f.named.is_empty()) && (a.opinion_gate || !names_proper_word(rt, q)) {
            f.gated = true;
            f.evidence = None;
        }
    }
    // The scoped scenes go first (before a term lookup's records), and are not repeated among the retrieved passages.
    let scoped = if !a.no_scoped_scenes && !a.no_route && kind.is_none() { scoped_scenes(rt, &tables.tools, q) } else { None };
    if let Some((ename, who, rows)) = &scoped {
        for (i, &r) in rows.iter().enumerate() {
            let c = &rt.store.chunks[r];
            let label = story_label(names, &c.story_id);
            pre.insert(i, Pre::new(c.chunk_id.clone(), format!("{label}; a scene of {ename} in which {} speaks or is named",
                who.join(" or ")), c.text.clone()).in_scope());
        }
    }
    // Character-scoped scenes (2026-10-06): no event named, one character named as the subject of a said or done act.
    // They are added before the retrieved passages, which keep their whole budget (with the scoped scenes' 3,000-token
    // rest, gold g0004 lost the retrieved line "I won't burn Rhodes Island, or my homework").
    if scoped.is_none() && !a.no_character_scenes && !a.no_route && kind.is_none() {
        if let Some((who, rows)) = character_scenes(rt, &a.runtime, &tables.tools, q) {
            for (i, (r, text, _)) in rows.iter().enumerate() {
                let c = &rt.store.chunks[*r];
                let label = story_label(names, &c.story_id);
                pre.insert(i, Pre::new(c.chunk_id.clone(), format!("{label}; {CHAR_LABEL} {who} speaks"), text.clone()).in_scope());
            }
        }
    }
    // With scoped scenes the retrieved passages get the rest of the budget, 3,000 tokens, as with IS-ending passages.
    let scoped_budget;
    let a = if scoped.is_some() {
        scoped_budget = Args { max_context_tokens: a.max_context_tokens.min(SCOPED_REST), ..a.clone() };
        &scoped_budget
    } else { a };
    let form = form.as_ref();
    let offtopic = a.lore_only && !tables.tools.names_anything(q);
    let first = answer_from(rt, llm, names, chrono, q, term_search.as_deref().unwrap_or(q), a, &pre, kind, form, offtopic).await?;
    // The typed fallback (2026-10-03, item 10; `--no-typed-fallback`): a declined answer to a question that names exactly one operator and was
    // not routed to a typed source tries that operator's own voice lines, module stories and (when artifacts/p4-art is
    // built) art units, without touching the router prompt: every added router line moved about 20 unrelated routes.
    // Since 2026-10-03 (leftover b) also an answer that cites only the operator's own file: "who helped Mutsumi care
    // for the plants?" cited her file, which says only that others admire her; the robots are in her module story.
    let own_file_only = |op: &str| !a.no_own_file_fallback && !first.cited.is_empty() && first.cited.iter()
        .all(|id| file_owner(&tables.tools, id).is_some_and(|o| norm(&o) == norm(op)));
    let retry_op = one_operator(q, &tables.tools).filter(|_| declines(&first.answer))
        .or_else(|| one_operator_wide(q, &tables.tools).filter(|op| own_file_only(op)));
    if !a.no_typed_fallback && !a.no_route && kind.is_none() && retry_op.is_some() {
        if let Some(op) = retry_op {
            let rt2 = rts.pick_art()?;
            let wide = rt2.retrieve(q, &RetrievalConfig { mode: Mode::Hybrid, k: 60, ..RetrievalConfig::default() })?;
            let own = wide.iter().map(|h| &rt2.store.chunks[h.row]).find(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                && typed_owner(c.text.lines().next().unwrap_or_default()).is_some_and(|o| norm(&o) == norm(&op)))
                .map(|c| c.group_id.clone());
            if let Some(k) = own {
                let k: &'static str = FALLBACK_KINDS.iter().find(|x| **x == k).copied().unwrap_or("voice");
                let second = answer_from(rt2, llm, names, chrono, q, q, a, &pre, Some(k), form, offtopic).await?;
                // Kept only when it cites a unit of the operator itself: "what operator is based off an animal with
                // phantom as part of its name" (ian11) named Phantom, got Melantha's voice lines first and answered
                // "Melantha" where the default declines (2026-10-03 07:50 run).
                let cites_own = second.cited.iter().any(|id| rt2.store.chunks.iter().find(|c| c.chunk_id == *id)
                    .is_some_and(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                        && typed_owner(c.text.lines().next().unwrap_or_default()).is_some_and(|o| norm(&o) == norm(&op))));
                if !declines(&second.answer) && cites_own {
                    return Ok(second);
                }
            }
        }
    }
    let rt = if kind == Some("art") { rts.pick_art()? } else { rts.pick(p4)? };
    if !a.retry || !declines(&first.answer) {
        return Ok(first);
    }
    // Retry once with the question rewritten into the story's own terms. Real questions miss passages that hold the
    // answer when the wording differs ("When did Ceobe get high on drugs?" never retrieves the mushroom scene of The
    // Great Chief Returns); the research doc ranks a decline-only retry among the cheapest fixes.
    let rw = llm.complete(&Request::greedy(REWRITE, q, None, 60, &["\n"])).await?;
    let query = rw.content.trim().trim_matches('"').to_owned();
    if query.is_empty() {
        return Ok(first);
    }
    // The rewrite alone found nothing on the first real run (0 of 36 declines answered: Gemma cannot put the question in
    // the story's terms without knowing the story), so the retry also widens retrieval: twice the passages, one more
    // neighbour each side, a 14,000-token budget (the research survey's passage-budget evidence on 9 to 12B readers).
    let mut wide = a.clone();
    wide.k = a.k * 2;
    wide.neighbors = a.neighbors + 1;
    wide.max_context_tokens = a.max_context_tokens.max(14_000);
    let mut second = answer_from(rt, llm, names, chrono, q, &format!("{q} {query}"), &wide, &pre, kind, form, offtopic).await?;
    if declines(&second.answer) {
        return Ok(first);
    }
    second.retry_query = Some(query);
    Ok(second)
}

/// The chunk ids an answer cites, in citation order, and how many citations point at no passage. "[4]", "[4, 5]" and
/// "[4][5]" all cite; anything else in brackets is prose. Numbers 1 to `off` are the passages before the retrieved
/// rows, then the rows, then `post`.
fn citations(text: &str, off: usize, first: &[&Pre], rows: &[usize], post: &[&Pre], rt: &Runtime) -> (Vec<String>, usize) {
    let mut cited = Vec::new();
    let mut invalid = 0usize;
    let mut i = 0;
    while let Some(open) = text[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = text[start..].find(']') else { break };
        // "[4]", "[4, 5]" and "[4][5]" all cite; anything else in brackets is prose.
        let inner = &text[start..start + close];
        let nums: Vec<usize> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).filter_map(|t| t.parse().ok()).collect();
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
fn passage_label(names: &HashMap<String, Names>, c: &Chunk) -> String {
    match c.group_id.as_str() {
        "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
        "profile" => format!("Trevor's character dossier (a summary written by Trevor): {}", c.story_id),
        "summary" => format!("Trevor's event summary (written by Trevor): {}", c.story_id),
        "module" | "voice" | "is" | "enemy" | "skin" | "item" | "art" | "gametext" => c.text.lines().next().unwrap_or_default().to_owned(),
        _ => story_label(names, &c.story_id),
    }
}

/// One answer from the passages retrieved for `search`, answering `q`. A topic summary, when the router chose one,
/// is passage [1] and the retrieved passages follow.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn answer_from(rt: &mut Runtime, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str, search: &str,
                     a: &Args, pre: &[Pre], kind: Option<&str>, form: Option<&Form>, offtopic: bool) -> Result<Answer> {
    let opinion_form = form.is_some_and(|f| f.form == "opinion");
    let (mut rows, lore) = passages(rt, search, a, kind, form)?;
    // Under a spoiler horizon (`--serve` jobs): only chunks released at or before it, and of the generated passages
    // only those that are themselves such a chunk (a topic, dossier, game-data or overview passage is not).
    let kept: Vec<Pre>;
    let pre = match rt.horizon.clone() {
        Some(h) => {
            rows.retain(|&r| rt.allows_row(&h, r));
            kept = pre.iter().filter(|p| rt.store.row(&p.id).is_some_and(|r| rt.allows_row(&h, r))).cloned().collect();
            &kept[..]
        }
        None => pre,
    };
    // The evidence composition's ask reads the composed candidates alone: with the retrieved passages after them, ian24's
    // composed ask named the collaboration's Sakiko Togawa again (2026-10-05 night).
    if !a.compose.is_empty() {
        rows.clear();
    }
    let scoped: Vec<&str> = pre.iter().filter(|p| p.scoped).map(|p| p.id.as_str()).collect();
    if !scoped.is_empty() {
        rows.retain(|&r| !scoped.contains(&rt.store.chunks[r].chunk_id.as_str()));
    }
    let (rows, expand_notes) = if a.identity_expand && !a.no_route && asks_identity(q) { identity_expand(rt, chrono, q, rows, 2) } else { (rows, HashMap::new()) };
    let mut pool = if a.line_pool && !a.no_route && kind.is_none() && scoped.is_empty() { line_pool(rt, &a.runtime, names, q, search, &rows, a.line_pool_all, a.line_pool_min)? } else { Vec::new() };
    if let Some(h) = rt.horizon.clone() {
        pool.retain(|p| rt.store.row(&p.id).is_some_and(|r| rt.allows_row(&h, r)));
    }
    if a.line_pool && std::env::var("TREVOR_LINE_POOL_DETECT").is_ok() {
        return Ok(Answer { qid: None, question: q.to_owned(), answer: format!("line-pool detect: {}", pool.len()), cited: Vec::new(),
                           passages: Vec::new(), invalid_citations: 0, prompt_tokens: 0, ms: 0.0, retry_query: None, form: None,
                           generated: std::collections::BTreeMap::new() });
    }
    let (first, post): (Vec<&Pre>, Vec<&Pre>) = pre.iter().partition(|p| !p.last);
    let post: Vec<&Pre> = post.into_iter().chain(pool.iter()).collect();
    let off = first.len();
    let flags = form.and_then(|f| f.flags).unwrap_or_else(|| Flags::keywords(q));
    let time_q = !a.no_route && flags.asks_time;
    let synth_q = !a.no_route && flags.asks_synthesis;
    let mut user = String::from("PASSAGES\n");
    for (i, p) in first.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", i + 1, p.label, p.text));
    }
    let label_links = if a.identity_labels && !a.no_route && asks_identity(q) { label_links(rt, &rows, chrono) } else { Vec::new() };
    for (i, &r) in rows.iter().enumerate() {
        let c = &rt.store.chunks[r];
        let label = passage_label(names, c);
        // A file found through another name of its character says so: with "Margaret vs Degenbrecher" and Nearl the
        // Radiant Knight's file among the passages, the answer still said it found nothing about "a character named
        // Margaret" (2026-09-30).
        let label = match form.and_then(|f| f.named.iter().find(|(sid, asked, file)| *sid == c.story_id
            && norm(asked) != norm(file))) {
            Some((_, asked, file)) => format!("{label}; this is the file of {file}, whom the question calls {asked}"),
            None => label,
        };
        let label = if label_links.is_empty() { label } else { identity_label(&label, &c.story_id, &label_links) };
        let label = match expand_notes.get(&r) { Some(n) => format!("{label}{n}"), None => label };
        user.push_str(&format!("\n[{}] ({label})\n{}\n", i + 1 + off, c.text));
    }
    for (i, p) in post.iter().enumerate() {
        user.push_str(&format!("\n[{}] ({})\n{}\n", off + rows.len() + i + 1, p.label, p.text));
    }
    if time_q {
        user.push_str(&timeline_notes(rt, chrono, &rows, off));
    }
    let subject_lines = if time_q && !a.no_chrono_subject && asks_when(q) { chronology_for(q, chrono) } else { String::new() };
    let time_evidence = !a.no_time_evidence && !subject_lines.is_empty();
    if !time_evidence {
        user.push_str(&subject_lines);
    }
    if a.identity_notes && !a.no_route && asks_identity(q) {
        user.push_str(&identity_notes(rt, &rows, chrono));
    }
    if time_evidence {
        user.push_str(&time_evidence_block(&subject_lines));
    }
    user.push_str(&format!("\nQUESTION: {q}"));
    if time_evidence {
        user.push_str(TIME_EVIDENCE_TAIL);
    }
    let started = std::time::Instant::now();
    let mut system = if a.partial { format!("{SYSTEM}{PARTIAL}") } else { SYSTEM.to_owned() };
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
        if (a.date_estimate && asks_when(q)) || !subject_lines.is_empty() {
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
    if offtopic {
        system.push_str(LORE_ONLY_RULE);
    }
    if !scoped.is_empty() {
        let tail = if pre.iter().any(|p| p.scoped && p.label.contains(CHAR_LABEL)) { CHAR_RULE_TAIL } else { SCOPED_RULE_TAIL };
        system.push_str(&format!("{SCOPED_RULE_HEAD}{}{tail}", scoped.len()));
    }
    if !a.compose.is_empty() {
        system.push_str(COMPOSE_RULE);
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
        system.push_str(if a.verdict_first { YES_NO_VERDICT_RULE } else { YES_NO_RULE });
    }
    match lore {
        // The typed units lead the passages, so the source-kind rule's "say Trevor lacks that text" does not apply:
        // it made the Angelina answer open with a decline although passage [1] was her outfit (2026-09-30).
        // Two parts, spelled out: with the lore passages merely present, Gemma answered from [1] alone (Angelina,
        // 2026-09-30 08:40).
        Some(l) => system.push_str(&format!(" Passages [{}] to [{}] are the source text the question asks about (their label \
names the kind); never say Trevor lacks that text. Answer in two parts. First, what that source text says, citing [{}] to [{}]. \
Then a paragraph that begins \"Lore context:\" and explains the people, places, powers and events the source text mentions, \
from passages [{}] to [{}] (story and operator-file passages about them), with citations; if those passages say nothing about \
what it mentions, say so in one sentence.", 1 + off, l.units + off, 1 + off, l.units + off, l.units + 1 + off,
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
        answer = format!("Trevor has no complete table for this question, so this list comes only from the {} passages it \
read and is not complete.\n\n{answer}", rows.len());
    }
    Ok(Answer {
        qid: None,
        question: q.to_owned(),
        answer,
        cited,
        passages: first.iter().map(|p| p.id.clone())
            .chain(rows.iter().map(|&r| rt.store.chunks[r].chunk_id.clone())).chain(post.iter().map(|p| p.id.clone())).collect(),
        invalid_citations: invalid,
        prompt_tokens: c.prompt_tokens,
        ms: started.elapsed().as_secs_f64() * 1000.0,
        retry_query: None,
        generated: pre.iter().filter(|p| !p.id.starts_with("topic:")).map(|p| (p.id.clone(), p.text.clone())).collect(),
        form: form.map(|f| {
            let base = f.evidence.as_ref().map_or_else(|| f.form.clone(), |e| format!("{}: {e}", f.form));
            f.story.as_ref().map_or(base.clone(), |sq| format!("{base}; story words: {sq}"))
        }),
    })
}

/// A whole-story answer from the summary tree: `passages` as (id, label, text), cited as "overview:<id>".
pub(crate) async fn answer_overview(llm: &Llm, q: &str, a: &Args, passages: &[(String, String, String)]) -> Result<Answer> {
    let started = std::time::Instant::now();
    let mut user = String::from("PASSAGES\n");
    for (i, (_, label, text)) in passages.iter().enumerate() {
        user.push_str(&format!("\n[{}] (Trevor's summary (generated): {label})\n{text}\n", i + 1));
    }
    user.push_str(&format!("\nQUESTION: {q}"));
    let system = format!("{SYSTEM}{OVERVIEW_RULE}");
    let c = llm.complete(&Request::greedy(&system, &user, None, a.n_predict, &[])).await?;
    let ids: Vec<String> = passages.iter().map(|(id, _, _)| format!("overview:{id}")).collect();
    let (mut cited, mut invalid) = (Vec::new(), 0usize);
    for inner in c.content.split('[').skip(1).filter_map(|t| t.split_once(']').map(|(x, _)| x)) {
        let parts: Vec<&str> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).collect();
        let nums: Vec<usize> = parts.iter().filter_map(|t| t.parse().ok()).collect();
        if nums.is_empty() || nums.len() != parts.len() {
            continue;
        }
        for n in nums {
            match ids.get(n.wrapping_sub(1)) {
                Some(id) if !cited.contains(id) => cited.push(id.clone()),
                Some(_) => {}
                None => invalid += 1,
            }
        }
    }
    Ok(Answer { qid: None, question: q.to_owned(), answer: c.content.trim().to_owned(), cited, passages: ids,
                invalid_citations: invalid, prompt_tokens: c.prompt_tokens, ms: started.elapsed().as_secs_f64() * 1000.0,
                retry_query: None, form: None, generated: Default::default() })
}
