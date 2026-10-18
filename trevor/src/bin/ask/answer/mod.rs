//! Answering one question: the table answer or the retrieval answer, the follow-up calls (answer check, operator
//! list check) and the answer record.

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use trevor::llm::{Llm, is_json_invalid};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::tools::{AnswerConstraint, Route, Tools, TopicPassage, norm};

use crate::chrono::Chrono;
use crate::cli::{Args, RouterKind, deep_for, dossier_allowed, form_on, retrieval_plan};
use crate::detect::{
    asks_admire, asks_identity, asks_relation, declines,
    is_opinion_question, term_of,
};
use crate::form::{Form, question_form};
use crate::prompts::CHAR_LABEL;
use crate::kinds::{FALLBACK_KINDS, RECORD_KINDS};
use crate::labels::{Names, file_owner, story_label, typed_owner};
use crate::people::{
    expand_names, named_characters, names_proper_word, one_operator, one_operator_wide, opinion_needs_candidates,
    people_named,
};
use crate::retrieval::Pre;
use crate::runtimes::Runtimes;
use crate::routing::{Routed, is_ending_passages, route_question};
use crate::scenes::{SCOPED_REST, character_scenes, excerpt_window, scoped_excerpts, scoped_scenes, scoped_summaries};
use crate::timegap::{GAP_WINDOW, time_gap_events, time_gap_lines};
use crate::terms::{term_glossary, term_lookup};

mod generate;
pub(crate) mod identity;
pub(crate) mod referent;
mod cite_words;

/// The passage-label fix (`cite_words.rs`), for `--identity-scan`.
pub(crate) fn cite_words_fix(answer: &str) -> Option<String> {
    cite_words::fix(answer)
}

pub(crate) use generate::passage_label;
use generate::answer_from;

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
    /// The question form and evidence query, when the form call ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) form: Option<String>,
    /// The text of each generated passage only `--lore v2` adds (new dossier, game data), by passage id, for the judges.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub(crate) generated: std::collections::BTreeMap<String, String>,
    /// Each passage's label and text as the prompt showed it, in passage order (the identity-claim check reads them).
    #[serde(skip)]
    pub(crate) texts: Vec<String>,
    /// The retrieved passages' token budget the answer call had (the labelled theory's retry bound, `theory.rs`).
    #[serde(skip)]
    pub(crate) budget: u32,
    /// Whether the prompt carried a HINTS block (the labelled theory's second ask).
    #[serde(skip)]
    pub(crate) hinted: bool,
}

pub(crate) fn table_only(q: &str, text: String) -> Answer {
    Answer { qid: None, question: q.to_owned(), answer: text, cited: Vec::new(), passages: Vec::new(),
             invalid_citations: 0, prompt_tokens: 0, ms: 0.0, form: None, generated: Default::default(),
             texts: Vec::new(), budget: 0, hinted: false }
}

/// The table tools, and what `main` routed for the single question before the index loaded.
pub(crate) struct Tables {
    pub(crate) tools: Tools,
    /// The topic summary the router chose for the single question, routed before the index loads.
    pub(crate) topic: Option<TopicPassage>,
    /// The route of the single question, routed before the index loads.
    pub(crate) route: Option<Route>,
}

/// One answer with its checks, then the identity-claim check (`--no-identity-check`, `identity.rs`): a sentence
/// saying that two different names are one person stays only when an identity link or a passage joins them.
pub(crate) async fn answer_one(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono,
                    q: &str, a: &Args, tables: &mut Tables) -> Result<Answer> {
    let mut ans = answer_checked(rts, llm, names, chrono, q, a, tables).await?;
    if !a.no_identity_check && !ans.texts.is_empty() {
        if let Some((text, claims)) = identity::check(&ans.answer, q, &ans.texts, &tables.tools) {
            if trevor::env::debug_on("TREVOR_IDENTITY_DEBUG") {
                eprintln!("identity check: dropped {claims:?}");
            }
            ans.cited = identity::cited_ids(&text, &ans.passages);
            ans.answer = text;
        }
    }
    // The referent checks (`--no-referent-check`, `--no-quote-check`, `referent.rs`): a fate claim whose cited
    // passages do not tie it to its character, and a quote the passages give to another speaker.
    if !ans.texts.is_empty() {
        if let Some((text, found)) = referent::check(&ans.answer, q, &ans.texts, &tables.tools, !a.no_referent_check,
                                                     !a.no_quote_check) {
            if trevor::env::debug_on("TREVOR_IDENTITY_DEBUG") {
                eprintln!("referent check: dropped {found:?}");
            }
            ans.cited = identity::cited_ids(&text, &ans.passages);
            ans.answer = text;
        }
    }
    // Passage labels out of the text (`--no-passage-label-fix`, `cite_words.rs`): "in passage [4]" is "in the text [4]".
    if !a.no_passage_label_fix && !ans.passages.is_empty() {
        if let Some(text) = cite_words::fix(&ans.answer) {
            ans.cited = identity::cited_ids(&text, &ans.passages);
            // A bare number that became a citation ("passage 12" of 9 passages) counts as invalid when it points at
            // no passage.
            let n = ans.passages.len();
            ans.invalid_citations = (ans.invalid_citations + identity::invalid_count(&text, n))
                .saturating_sub(identity::invalid_count(&ans.answer, n));
            ans.answer = text;
        }
    }
    Ok(ans)
}

/// One answer, with the answer category check (`--no-answer-check`): a who/which question that constrains its answer's
/// category ("Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?") and whose
/// answer's first named character fails it by the game data (Sakiko Togawa is a playable operator) is asked once more
/// with a passage saying so; when the second answer's first name fails too, the data note is appended to it.
async fn answer_checked(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono,
                        q: &str, a: &Args, tables: &mut Tables) -> Result<Answer> {
    let first = answer_one_inner(rts, llm, names, chrono, q, a, tables).await?;
    let first = theory_retry(rts, llm, names, chrono, q, a, tables, first).await?;
    let first = operator_list_check(rts, llm, names, chrono, q, a, tables, first).await?;
    if a.no_answer_check || a.no_route || first.passages.is_empty() || !asks_identity(q) {
        return Ok(first);
    }
    let Some(c) = tables.tools.answer_constraint(q) else { return Ok(first) };
    let Some((name, reason)) = tables.tools.failing_answer_name(q, &first.answer, &c) else { return Ok(first) };
    let wants = match &c {
        AnswerConstraint::NonPlayable => "someone who is not a playable operator".to_owned(),
        AnswerConstraint::Playable => "a playable operator".to_owned(),
        AnswerConstraint::Nation(x) => format!("an operator from {x}"),
    };
    let note = format!("An earlier answer to this question named {name}. By the game data, {reason}, \
        and the question asks for {wants}. Do not give {name} as the answer. \
        Name only someone the passages show fits the question, or say that the passages do not identify one.");
    let again = Args { check_note: Some(note), routed: false, ..a.clone() };
    let mut second = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    if let Some((n2, r2)) = tables.tools.failing_answer_name(q, &second.answer, &c) {
        second.answer = format!("{}\n\nTrevor's data check: {r2}, and the question asks for {wants}, \
            so {n2} does not fit.", second.answer);
    }
    Ok(second)
}

/// The labelled theory (`--no-theory-rule`, `theory.rs`): a declined yes/no question naming one character whose
/// passages hold lines about that character touching what it asks is asked once more with those lines as HINTS and
/// THEORY_RULE; the second answer is kept when it has its "Theory:" line.
#[allow(clippy::too_many_arguments)]
async fn theory_retry(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str,
                      a: &Args, tables: &mut Tables, first: Answer) -> Result<Answer> {
    // An alive-or-dead question has the deaths evidence instead (`fate.rs`).
    if a.no_theory_rule || a.no_route || first.passages.is_empty() || !crate::theory::asks_whether(q)
        || !crate::theory::declined(&first.answer) || crate::detect::is_death_question(q) || crate::detect::has(q, &["alive"]) {
        return Ok(first);
    }
    let people = people_named(&rts.main, &tables.tools, q);
    let [key] = people.as_slice() else { return Ok(first) };
    // The name as the tables write it ("Fyodor", not the folded key "fyodor").
    let who = tables.tools.operator_names().into_iter().chain(tables.tools.speaker_names())
        .find(|x| norm(x) == *key).map_or_else(|| key.clone(), str::to_owned);
    let who = who.as_str();
    let Some(mut spec) = crate::theory::spec(q, &first.answer, &tables.tools, who, &first.passages)
    else { return Ok(first) };
    spec.pinned = crate::theory::hinted(&first.texts, &spec).into_iter()
        .filter_map(|n| first.passages.get(n - 1).cloned()).collect();
    let block = crate::theory::block(&first.texts, &spec);
    let extra = crate::theory::extra_hints(&rts.main, &spec);
    if block.is_none() && extra.is_empty() {
        return Ok(first);
    }
    // The second ask must fit where the first did: the pinned chunks, the extra excerpts and the HINTS block with
    // THEORY_RULE come out of the first ask's retrieved budget, so when they exceed it the first answer stays (with them
    // inside it, the retrieved passages get the rest and even the first hit keeps to it, `retrieval::passages`).
    let main = &rts.main;
    let cost = spec.pinned.iter().filter_map(|id| main.store.row(id)).map(|r| main.store.chunks[r].token_count)
        .chain(extra.iter().map(|&(r, i)| excerpt_window(main, r, &[i], 1).1))
        .fold(crate::theory::THEORY_TOKENS, u32::saturating_add);
    if cost > first.budget {
        if trevor::env::debug_on("TREVOR_THEORY_DEBUG") {
            eprintln!("theory: {who} skipped, {cost} tokens over a budget of {}", first.budget);
        }
        return Ok(first);
    }
    if trevor::env::debug_on("TREVOR_THEORY_DEBUG") {
        eprintln!("theory: {who}{} extra {:?}", block.unwrap_or_default(),
                  extra.iter().map(|&(r, i)| format!("{}:{i}", rts.main.store.chunks[r].chunk_id)).collect::<Vec<_>>());
    }
    // Retrieved hits without their neighbouring chunks: a hit's neighbours would overrun the reduced budget.
    let again = Args { theory: Some(spec), routed: false, neighbors: 0, ..a.clone() };
    let second = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    // Kept only when its prompt carried the HINTS block (and so THEORY_RULE) and it has the "Theory:" line.
    Ok(if second.hinted && crate::theory::has_theory(&second.answer) { second } else { first })
}

/// The operator list check (`--no-operator-list-check`): an answer to a question
/// asking for operators (`Tools::asks_operators`) whose list items name someone who is not a playable operator (Aurela,
/// a Sankta mercenary named in Gracebearer's file 4, in a ranking of operators) is asked once more with a note naming
/// each such person and the operator file the cited passage
/// belongs to; when the second answer still lists one, the data note is appended.
#[allow(clippy::too_many_arguments)]
async fn operator_list_check(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono, q: &str,
                             a: &Args, tables: &mut Tables, first: Answer) -> Result<Answer> {
    if a.no_operator_list_check || a.no_route || first.passages.is_empty() || !tables.tools.asks_operators(q) {
        return Ok(first);
    }
    let reasons = |ans: &Answer, tools: &Tools| -> Vec<String> {
        tools.non_operator_items(&ans.answer).into_iter().map(|(name, cites)| {
            let file = cites.iter().filter_map(|n| ans.passages.get(n.wrapping_sub(1)))
                .find_map(|id| file_owner(tools, id));
            match file {
                Some(o) => format!("{name} is not a playable operator; {name} is a person named in {o}'s operator \
                    file, and what that file says {name} did is {name}'s, not {o}'s"),
                None => format!("{name} is not a playable operator by the game data"),
            }
        }).collect()
    };
    let found = reasons(&first, &tables.tools);
    if found.is_empty() {
        return Ok(first);
    }
    let note = format!("An earlier answer to this question listed people as operators who are not. \
        By the game data: {}. The question asks about operators: list only playable operators, \
        and give each act to the person the passage says did it.", found.join("; "));
    let again = Args { check_note: Some(note), routed: false, ..a.clone() };
    let mut second = answer_one_inner(rts, llm, names, chrono, q, &again, tables).await?;
    let still = reasons(&second, &tables.tools);
    if !still.is_empty() {
        second.answer = format!("{}\n\nTrevor's data check: {}.", second.answer, still.join("; "));
    }
    Ok(second)
}

/// Passages before the retrieved ones: the topic summary the router chose, then (`--lore v2`) the new dossier of a
/// character the question names. Under v1 the prompt is the one before, byte for byte. In order after `pre` (the
/// IS-ending passages): the topic, the new dossier, the answer-check note and the game-data passage.
fn lead_passages(a: &Args, tools: &Tools, q: &str, topic: &Option<TopicPassage>, route: &Option<Route>,
                 mut pre: Vec<Pre>) -> Vec<Pre> {
    let topic_label = |t: &TopicPassage| if tools.deep_entry(&t.topic).is_some() {
        format!("Trevor's topic entry (generated from the story passages, operator files and records that name \
            it; prefer the story passages): {}", t.topic)
    } else {
        format!("Trevor's topic summary (generated; prefer the story passages): {}", t.topic)
    };
    pre.extend(topic.iter().map(|t| Pre::new(format!("topic:{}", t.topic), topic_label(t), t.text.clone())));
    if let Some(d) = tools.new_dossier(q).filter(|_| dossier_allowed(q, a, route.as_ref())) {
        let label = format!("Trevor's character dossier (generated; prefer the story passages): {}", d.topic);
        pre.push(Pre::new(format!("dossier:{}", d.topic), label, d.text).after());
    }
    if let Some(note) = &a.check_note {
        pre.push(Pre::new("check".into(), "Trevor's check of an earlier answer against the game data".into(),
                          note.clone()).after());
    }
    if let Some(g) = tools.game_data(q) {
        pre.push(Pre::new(format!("gamedata:{}", g.topic), format!("game data: {}", g.topic), g.text)
            .after().with_rule());
    }
    pre
}

pub(crate) async fn answer_one_inner(rts: &mut Runtimes, llm: &Llm, names: &HashMap<String, Names>, chrono: &Chrono,
                          q: &str, a: &Args, tables: &mut Tables) -> Result<Answer> {
    // The relation rule is about people: not for a question naming a nation, place, race or organization topic.
    let people_only;
    let a = if a.relation_rule && tables.tools.names_group_topic(q) {
        people_only = Args { relation_rule: false, ..a.clone() };
        &people_only
    } else { a };
    let mut topic = if a.routed { tables.topic.take() } else { None };
    let mut route = if a.routed { tables.route.take() } else { None };
    if !a.no_route && !a.routed {
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_checked(q, a, tables, Some(llm)).await?;
        // Under a spoiler horizon a table answer (built from every story) and a topic summary are not served; the
        // question is answered from the passages the horizon lets through instead.
        if let Some(text) = r.text {
            let served = match rts.horizon.as_deref() {
                None => Some(text),
                Some(h) => crate::spoiler::table_within(&r.route, text, &tables.tools, h, a.no_horizon_recap,
                                                        Some(&rts.main.store)),
            };
            if let Some(text) = served {
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
    let mut pre = lead_passages(a, &tables.tools, q, &topic, &route, ending_pre);
    // Term lookup: the term's spelling in P4x, its two best record chunks and a glossary, before the passages.
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
                        g if RECORD_KINDS.contains(&g) || g == "voice" =>
                            c.text.lines().next().unwrap_or_default().to_owned(),
                        _ => story_label(names, &c.story_id),
                    };
                    pre.insert(at, Pre::new(c.chunk_id.clone(), format!("{label}; it names {spelling}"),
                                            c.text.clone()));
                    at += 1;
                }
                if !glossary.is_empty() {
                    let text = glossary.iter().map(|(n, t)| format!("{n}: {t}")).collect::<Vec<_>>().join("\n");
                    pre.insert(at, Pre::new("glossary".into(), "Trevor's glossary of the terms these passages use \
                        (generated from its topic summaries; prefer the passages)".into(), text));
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
    let a = if a.relation_rule && asks_relation(q) && !asks_admire(q) && people_named(rt, &tables.tools, q).len() < 2 {
        two_people = Args { relation_rule: false, ..a.clone() };
        &two_people
    } else { a };
    let expanded = if a.router == RouterKind::Keywords || a.no_name_expansion || a.no_route { q.to_owned() }
        else { expand_names(rt, &tables.tools, q) };
    let q = expanded.as_str();
    let mut form = if form_on(a) {
        // A backend without grammars (OpenAI-compatible) whose JSON did not validate twice: the fact form.
        Some(match question_form(llm, q, a.form_v1).await {
            Err(e) if is_json_invalid(&e) => Form { form: "fact".into(), ..Form::default() },
            r => r?,
        })
    } else { None };
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion") {
        f.named = named_characters(rt, &tables.tools, q);
        if !a.no_group_evidence {
            f.groups = tables.tools.named_group_topics(q);
            if f.groups.len() < 2 {
                f.groups.clear();
            }
        }
    }
    if let Some(f) = form.as_mut().filter(|f| f.form == "opinion" && !a.no_opinion_gate && is_opinion_question(q)) {
        if !opinion_needs_candidates(rt, &tables.tools, q, !f.named.is_empty()) && !names_proper_word(rt, q) {
            f.gated = true;
            f.evidence = None;
        }
    }
    // Time-gap evidence (`--no-time-gap`): a question asking how much time passes between two stories it names gets
    // the later story's lines that state elapsed time since something of the earlier one, first; it then takes the
    // place of the scoped and character scenes, whose rule numbers the first passages too.
    let gap = if !a.no_time_gap && !a.no_route && kind.is_none() {
        time_gap_events(q, &tables.tools, chrono).map(|(e, l)| (e, l, time_gap_lines(rt, &tables.tools, e, l)))
            .filter(|(_, _, lines)| !lines.is_empty())
    } else { None };
    if let Some((e, l, lines)) = &gap {
        let (en, ln) = (e["name"].as_str().unwrap_or_default(), l["name"].as_str().unwrap_or_default());
        for (i, &(r, li)) in lines.iter().enumerate() {
            let c = &rt.store.chunks[r];
            let (text, _) = excerpt_window(rt, r, &[li], GAP_WINDOW);
            let line = c.text.lines().nth(li).unwrap_or_default().to_owned();
            pre.insert(i, Pre::new(c.chunk_id.clone(),
                                   format!("{}; a line of {ln} that states time passed since something of {en}",
                                           story_label(names, &c.story_id)), text).in_gap(line));
        }
    }
    // The scoped scenes go first (before a term lookup's records), and are not repeated among the retrieved passages.
    // Scoped excerpts (`--no-scoped-line-rank`): a wh-question about what a character of the named event said there
    // gets excerpts of that character's best reranked lines, labelled as character scenes, instead of scoped scenes.
    let excerpts = if gap.is_none() && !a.no_scoped_scenes && !a.no_scoped_line_rank && !a.no_route && kind.is_none() {
        scoped_excerpts(rt, &a.runtime, &tables.tools, q, a.scene_tokens.unwrap_or(crate::scenes::CHAR_TOKENS),
                        a.scene_verbs())
    } else { None };
    if let Some((_, who, rows)) = &excerpts {
        for (i, (r, text, _)) in rows.iter().enumerate() {
            let c = &rt.store.chunks[*r];
            let label = story_label(names, &c.story_id);
            pre.insert(i, Pre::new(c.chunk_id.clone(), format!("{label}; {CHAR_LABEL} {who} speaks"), text.clone())
                .in_scope());
        }
    }
    let scoped = if excerpts.is_none() && gap.is_none() && !a.no_scoped_scenes && !a.no_route && kind.is_none() {
        scoped_scenes(rt, &tables.tools, q, a.scene_tokens.unwrap_or(crate::scenes::SCOPED_TOKENS))
    } else { None };
    if let Some((ename, who, rows)) = &scoped {
        for (i, &r) in rows.iter().enumerate() {
            let c = &rt.store.chunks[r];
            let label = story_label(names, &c.story_id);
            pre.insert(i, Pre::new(c.chunk_id.clone(),
                                   format!("{label}; a scene of {ename} in which {} speaks or is named",
                                           who.join(" or ")),
                                   c.text.clone()).in_scope());
        }
        // The summaries of the scenes' stories follow them (opt-in, `--scoped-summaries`): a scene shows a moment
        // without its setting.
        if a.scoped_summaries {
            for (k, (sid, text)) in scoped_summaries(rt, &tables.tools, rows).into_iter().enumerate() {
                pre.insert(rows.len() + k, Pre::new(format!("story:{sid}"),
                    format!("Trevor's summary of the story {} (written by Trevor)",
                            story_label(names, &sid).trim_start_matches("story: ")), text));
            }
        }
    }
    // Character-scoped scenes: no event named, one character named as the subject of a said or done act. They are added
    // before the retrieved passages, which keep their whole budget (the scoped scenes' 3,000-token rest would drop
    // retrieved lines other questions need).
    if scoped.is_none() && excerpts.is_none() && gap.is_none() && !a.no_character_scenes && !a.no_route
        && kind.is_none() {
        if let Some((who, rows)) = character_scenes(rt, &a.runtime, &tables.tools, q, a.scene_tokens.unwrap_or(crate::scenes::CHAR_TOKENS),
                                                  a.scene_verbs()) {
            for (i, (r, text, _)) in rows.iter().enumerate() {
                let c = &rt.store.chunks[*r];
                let label = story_label(names, &c.story_id);
                pre.insert(i, Pre::new(c.chunk_id.clone(), format!("{label}; {CHAR_LABEL} {who} speaks"), text.clone())
                    .in_scope());
            }
        }
    }
    // Deaths evidence (`--no-death-evidence`, `fate.rs`): an alive-or-dead question about one character of the deaths
    // table gets the lines its rows quote, after any scoped scenes or time-gap lines (whose rule numbers the first
    // passages), and the retrieved passages give up the tokens they take.
    let mut pre_tokens = 0u32;
    if !a.no_death_evidence && !a.no_route && kind.is_none() {
        if let Some(who) = crate::fate::alive_dead_subject(q, &tables.tools) {
            let at = after_lead(&pre);
            for (k, (id, tail, text, tok)) in crate::fate::death_evidence(rt, &tables.tools, &who).into_iter()
                .enumerate() {
                let sid = rt.store.row(&id).map(|r| rt.store.chunks[r].story_id.clone()).unwrap_or_default();
                pre.insert(at + k, Pre::new(id, format!("{}; {tail}", story_label(names, &sid)), text).of_death(&who));
                pre_tokens = pre_tokens.saturating_add(tok);
            }
        }
    }
    // The labelled theory's second ask (`theory.rs`): lines of the character's own scenes that bear on the question,
    // after the retrieved passages, which give up the tokens they take.
    // The first ask's passages with hint lines go first, then excerpts of the character's own lines, and the
    // retrieved passages give up the tokens they and the HINTS block take.
    if let Some(t) = a.theory.as_ref().filter(|_| kind.is_none()) {
        pre_tokens = pre_tokens.saturating_add(crate::theory::THEORY_TOKENS);
        // After any scoped scenes or time-gap lines, whose rule numbers the first passages.
        let mut at = after_lead(&pre);
        for id in &t.pinned {
            let Some(r) = rt.store.row(id).filter(|_| !pre.iter().any(|p| p.id == *id)) else { continue };
            let c = &rt.store.chunks[r];
            pre.insert(at, Pre::new(c.chunk_id.clone(), passage_label(names, c), c.text.clone()).in_theory());
            at += 1;
            pre_tokens = pre_tokens.saturating_add(c.token_count);
        }
        for (r, i) in crate::theory::extra_hints(rt, t) {
            let c = &rt.store.chunks[r];
            let (text, tok) = excerpt_window(rt, r, &[i], 1);
            pre.insert(at, Pre::new(c.chunk_id.clone(), format!("{}; a line of {} that bears on the question",
                                                                story_label(names, &c.story_id), t.who), text)
                .in_theory());
            at += 1;
            pre_tokens = pre_tokens.saturating_add(tok);
        }
    }
    // With scoped scenes the retrieved passages get the rest of the budget, 3,000 tokens, as with IS-ending passages.
    let scoped_budget;
    // Under a capped profile (`scene_tokens`, mid and mid6k) scene passages of either kind share the profile's budget,
    // so the whole prompt fits the slot the profile is served with (4,096 tokens for mid).
    let scenes_in = pre.iter().any(|p| p.scoped);
    let a = if let (Some(cap), true) = (a.scene_tokens, scenes_in) {
        scoped_budget = Args { max_context_tokens: a.max_context_tokens.saturating_sub(cap), ..a.clone() };
        &scoped_budget
    } else if scoped.is_some() {
        scoped_budget = Args { max_context_tokens: a.max_context_tokens.min(SCOPED_REST), ..a.clone() };
        &scoped_budget
    } else { a };
    let death_budget;
    let a = if pre_tokens > 0 {
        death_budget = Args { max_context_tokens: a.max_context_tokens.saturating_sub(pre_tokens), ..a.clone() };
        &death_budget
    } else { a };
    let form = form.as_ref();
    let first = answer_from(rt, llm, names, chrono, q, term_search.as_deref().unwrap_or(q), a, &pre, kind, form).await?;
    // The typed fallback (`--no-typed-fallback`): a declined answer to a question that names exactly one operator and was
    // not routed to a typed source tries that operator's own voice lines, module stories and (when artifacts/p4-art is
    // built) art units, without touching the router prompt. So does an answer that cites only the operator's own file
    // (`--no-own-file-fallback`): "who helped Mutsumi care for the plants?" cited her file, which says only that others
    // admire her; the robots are in her module story.
    let own_file_only = |op: &str| !a.no_own_file_fallback && !first.cited.is_empty() && first.cited.iter()
        .all(|id| file_owner(&tables.tools, id).is_some_and(|o| norm(&o) == norm(op)));
    let retry_op = one_operator(q, &tables.tools).filter(|_| declines(&first.answer))
        .or_else(|| one_operator_wide(q, &tables.tools).filter(|op| own_file_only(op)));
    if !a.no_typed_fallback && !a.no_route && kind.is_none() && retry_op.is_some() {
        if let Some(op) = retry_op {
            let rt2 = rts.pick_art()?;
            let wide = rt2.retrieve(q, &RetrievalConfig { mode: Mode::Hybrid, k: 60, ..RetrievalConfig::default() })?;
            let own = wide.iter().map(|h| &rt2.store.chunks[h.row])
                .find(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                    && typed_owner(c.text.lines().next().unwrap_or_default()).is_some_and(|o| norm(&o) == norm(&op)))
                .map(|c| c.group_id.clone());
            if let Some(k) = own {
                let k: &'static str = FALLBACK_KINDS.iter().find(|x| **x == k).copied().unwrap_or("voice");
                let second = answer_from(rt2, llm, names, chrono, q, q, a, &pre, Some(k), form).await?;
                // Kept only when it cites a unit of the operator itself: a question that names an operator only in
                // passing ("what operator is based off an animal with phantom as part of its name") can get another
                // operator's units and answer with that operator.
                let cites_own = second.cited.iter().any(|id| rt2.store.chunks.iter().find(|c| c.chunk_id == *id)
                    .is_some_and(|c| FALLBACK_KINDS.contains(&c.group_id.as_str())
                        && typed_owner(c.text.lines().next().unwrap_or_default())
                            .is_some_and(|o| norm(&o) == norm(&op))));
                if !declines(&second.answer) && cites_own {
                    return Ok(second);
                }
            }
        }
    }
    Ok(first)
}

/// Where passages added after the scoped scenes or time-gap lines go: the rule of those numbers the first passages.
fn after_lead(pre: &[Pre]) -> usize {
    pre.iter().take_while(|p| p.scoped || p.gap.is_some()).count()
}

/// The router, with the keyword router as the fallback when the model's JSON did not validate twice (a backend without
/// grammars, `--backend openai`; a llama.cpp router call is grammar-bound and never fails that way).
pub(crate) async fn route_checked(q: &str, a: &Args, tables: &mut Tables, llm: Option<&Llm>) -> Result<Routed> {
    match route_question(q, a, &tables.tools, llm).await {
        Err(e) if is_json_invalid(&e) => {
            let kw = Args { router: RouterKind::Keywords, ..a.clone() };
            route_question(q, &kw, &tables.tools, llm).await
        }
        r => r,
    }
}
