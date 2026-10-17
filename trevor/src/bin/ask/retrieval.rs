//! Retrieval: the passages a question gets from the corpus (hybrid search, typed sources with their lore, opinion
//! candidates, the origin chain), and the generated passages placed before them.

use std::collections::{BTreeSet, HashMap};

use anyhow::Result;
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;
use trevor::tools::{contains_words, norm};

use crate::cli::{Args, lore_on};
use crate::detect::asks_origin;
use crate::form::Form;
use crate::kinds::{CHARACTER_GROUPS, LORE_KINDS, TYPED};
use crate::labels::unit_owner;

/// Hits of `extra` and `hits` alternately, first seen wins.
pub(crate) fn interleave(extra: Vec<trevor::search::pipeline::Hit>, hits: Vec<trevor::search::pipeline::Hit>)
    -> Vec<trevor::search::pipeline::Hit> {
    let mut taken: BTreeSet<usize> = BTreeSet::new();
    let mut mixed = Vec::with_capacity(hits.len() + extra.len());
    let (mut x, mut y) = (extra.into_iter(), hits.into_iter());
    loop {
        let (p, q) = (x.next(), y.next());
        if p.is_none() && q.is_none() {
            break;
        }
        mixed.extend([p, q].into_iter().flatten().filter(|h| taken.insert(h.row)));
    }
    mixed
}

/// Candidates an opinion question gathers from character sources, and at most this many per character.
pub(crate) const CANDIDATES: usize = 6;
pub(crate) const CANDIDATE_PER_CHARACTER: usize = 2;
/// Chunks per group an opinion question compares (`Form::groups`), ahead of the candidates.
pub(crate) const GROUP_EVIDENCE: usize = 3;

/// What `passages` put first for a typed source: how many source units, then how many lore passages after them.
#[derive(Default, Clone, Copy)]
pub(crate) struct Lore {
    pub(crate) units: usize,
    pub(crate) lore: usize,
}

/// Passages for `q`, and for a typed source with its lore (see below) how many units and lore hits lead them.
pub(crate) fn passages(rt: &mut Runtime, q: &str, a: &Args, kind: Option<&str>, form: Option<&Form>)
    -> Result<(Vec<usize>, Option<Lore>)> {
    let cfg = RetrievalConfig { mode: Mode::Hybrid, k: a.k, ..RetrievalConfig::default() };
    let mut hits = rt.retrieve(q, &cfg)?;
    let mut lore_added: Option<Lore> = None;
    let evidence = form.and_then(|f| f.evidence.as_deref()).filter(|e| !e.trim().is_empty());
    // A yes/no question interleaves the hits of its evidence query with its own: the evidence query names the facts
    // to check ("Harold wife daughter children married") and ranks the lines that hold them far higher than the
    // question's own wording does.
    if let (Some(ev), Some("yes_no")) = (evidence, form.map(|f| f.form.as_str())) {
        let extra = rt.retrieve(ev, &cfg)?;
        hits = interleave(extra, hits);
    }
    let evidence = evidence.filter(|_| form.is_some_and(|f| f.form == "opinion"));
    // A judgment question gathers candidates first: the evidence query ("skilled cook, culinary talent") over a wide list,
    // kept to operator files and dossiers, at most two chunks per character, since the question's own wording ("Who is
    // the best cook?") rarely retrieves an operator file. They take no neighbours, so six candidates cost six chunks.
    // Characters the question names come first, each from its own files, so other operators' files cannot crowd out a
    // named one ("Midnight or Matsukiri?").
    let mut bare: BTreeSet<usize> = BTreeSet::new();
    if let Some(ev) = evidence {
        let mut per: HashMap<String, usize> = HashMap::new();
        let mut cands = Vec::new();
        // At most two files per name the question uses ("Margaret" is Nearl's and Nearl the Radiant Knight's real name
        // and a dossier's title), six in all.
        let mut per_name: HashMap<String, usize> = HashMap::new();
        let named: Vec<_> = form.map(|f| f.named.clone()).unwrap_or_default().into_iter()
            .filter(|(_, asked, _)| {
                let n = per_name.entry(asked.to_lowercase()).or_default();
                *n += 1;
                *n <= 2
            })
            .take(CANDIDATES)
            .collect();
        for (sid, asked, file) in named {
            let own = rt.retrieve(&format!("{file} {asked} {ev}"), &RetrievalConfig { k: 60, ..cfg })?;
            let mine: Vec<_> = own.into_iter()
                .filter(|h| rt.store.chunks[h.row].story_id == sid)
                .take(CANDIDATE_PER_CHARACTER)
                .collect();
            // A named character always gets its file: the first chunk when the query ranks none of it in the top 60.
            if mine.is_empty() {
                if let Some(&row) = rt.store.story_rows(&sid).first() {
                    cands.push(trevor::search::pipeline::Hit { row, score: 0.0, dense_rank: None, bm25_rank: None,
                                                               fused_rank: None });
                }
            }
            cands.extend(mine);
            per.insert(sid, CANDIDATE_PER_CHARACTER);
        }
        let room = CANDIDATES.saturating_sub(cands.len());
        let wide = rt.retrieve(ev, &RetrievalConfig { k: 60, ..cfg })?;
        cands.extend(wide.into_iter().filter(|h| {
            let c = &rt.store.chunks[h.row];
            CHARACTER_GROUPS.contains(&c.group_id.as_str()) && {
                let n = per.entry(c.story_id.clone()).or_default();
                *n += 1;
                *n <= CANDIDATE_PER_CHARACTER
            }
        }).take(room));
        // Each group an opinion question compares gets its own chunks first: with only the question's and the evidence
        // query's hits, one group can fill every passage ("Karlan Trade Group and Blacksteel" retrieves Karlan Trade and
        // Silverash) and the answer then says the other is not mentioned.
        let groups = form.map(|f| f.groups.clone()).unwrap_or_default();
        let mut own: Vec<trevor::search::pipeline::Hit> = Vec::new();
        for (topic, forms) in &groups {
            let found = rt.retrieve(&format!("{topic} {ev}"), &RetrievalConfig { k: 60, ..cfg })?;
            let names = |row: usize| {
                let t = norm(&rt.store.chunks[row].text);
                forms.iter().any(|f| contains_words(&t, &norm(f)))
            };
            let mine: Vec<_> = found.into_iter()
                .filter(|h| names(h.row) && !own.iter().chain(&cands).any(|x| x.row == h.row))
                .take(GROUP_EVIDENCE).collect();
            own.extend(mine);
        }
        let cands: Vec<_> = own.into_iter().chain(cands).collect();
        bare = cands.iter().map(|h| h.row).collect();
        hits = cands.into_iter().chain(hits.into_iter().filter(|h| !bare.contains(&h.row))).collect();
    }
    // A question about one kind of source (P4 units: module, voice, is, enemy, skin, item) gets the best 3 chunks of
    // that kind first from a wider candidate list, so story passages cannot crowd them out.
    if let Some(kind) = kind {
        let wide = rt.retrieve(q, &RetrievalConfig { k: 60, ..cfg })?;
        let lore_here = lore_on(a) && LORE_KINDS.contains(&kind);
        let mut pref: Vec<_> = wide.into_iter().filter(|h| rt.store.chunks[h.row].group_id == kind)
            .take(if lore_here { 12 } else { 3 }).collect();
        // With lore on, units of another operator do not follow the best unit ("What does Angelina's Bloodline of
        // Combat outfit say?" would otherwise also take Swire's Bloodline of Combat outfit).
        if lore_here {
            let owner_of = |row: usize| unit_owner(rt.store.chunks[row].text.lines().next().unwrap_or_default());
            if let Some(o) = pref.first().and_then(|h| owner_of(h.row)) {
                pref.retain(|h| owner_of(h.row).is_none_or(|x| x == o));
            }
            pref.truncate(3);
        }
        // Lore for a typed source: the best unit's own text is a second query, and its best story and operator-file
        // passages follow the units, so the answer can explain who and what the unit names (an outfit's description
        // alone can be two sentences).
        let mut lore = Vec::new();
        if lore_here {
            if let Some(top) = pref.first() {
                let text: String = rt.store.chunks[top.row].text.chars().take(600).collect();
                let lq = rt.retrieve(&text, &RetrievalConfig { k: 40, ..cfg })?;
                lore = lq.into_iter()
                    .filter(|h| !TYPED.contains(&rt.store.chunks[h.row].group_id.as_str()))
                    .take(4)
                    .collect();
            }
        }
        if lore_here && !pref.is_empty() {
            lore_added = Some(Lore { units: pref.len(), lore: lore.len() });
        }
        let mut taken: BTreeSet<usize> = BTreeSet::new();
        let head: Vec<_> = pref.into_iter().chain(lore).filter(|h| taken.insert(h.row)).collect();
        hits = head.into_iter().chain(hits.into_iter().filter(|h| !taken.contains(&h.row))).collect();
    }
    // The origin chain (`--no-origin-chain` turns it off): a why/how-it-began question also gets the earlier scenes of the story its search keeps hitting.
    // The earliest cause often ranks too low for the passage budget to keep it ("Why did swire join the LGD?": the
    // kidnapping that opens her story ranks below her L.G.D. years). The story with the most chunks in the top 20 (at
    // least 3) goes first: its chunks there that come before its best-ranked hit, without neighbours, in story order (at
    // most 5), then that hit with its neighbours, then the other hits.
    if !a.no_origin_chain && !a.no_route && asks_origin(q) {
        let wide = rt.retrieve(q, &RetrievalConfig { k: 20, ..cfg })?;
        let mut count: HashMap<&str, usize> = HashMap::new();
        for h in &wide {
            *count.entry(rt.store.chunks[h.row].story_id.as_str()).or_default() += 1;
        }
        let top = count.iter()
            .filter(|(_, n)| **n >= 3)
            .max_by_key(|(sid, n)| (**n, std::cmp::Reverse(**sid)))
            .map(|(s, _)| (*s).to_owned());
        if let Some(sid) = top {
            let rows = rt.store.story_rows(&sid);
            let pos = |row: usize| rows.iter().position(|&r| r == row).unwrap_or(usize::MAX);
            let lead = hits.iter().chain(wide.iter()).find(|h| rt.store.chunks[h.row].story_id == sid).cloned();
            if let Some(lead) = lead {
                let mut add: Vec<_> = wide.iter()
                    .filter(|h| rt.store.chunks[h.row].story_id == sid && pos(h.row) < pos(lead.row))
                    .cloned().collect();
                add.sort_by_key(|h| pos(h.row));
                add.truncate(5);
                bare.extend(add.iter().map(|h| h.row));
                let first: Vec<_> = add.into_iter().chain(std::iter::once(lead)).collect();
                let rest: Vec<_> = hits.into_iter().filter(|h| !first.iter().any(|x| x.row == h.row)).collect();
                hits = first.into_iter().chain(rest).collect();
            }
        }
    }
    let mut chosen: Vec<usize> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut budget = 0u32;
    for h in hits {
        let c = &rt.store.chunks[h.row];
        let rows = rt.store.story_rows(&c.story_id);
        let pos = rows.iter().position(|&r| r == h.row).unwrap_or(0);
        let nb = if bare.contains(&h.row) { 0 } else { a.neighbors };
        let lo = pos.saturating_sub(nb);
        let hi = (pos + nb).min(rows.len().saturating_sub(1));
        let cluster: Vec<usize> = rows[lo..=hi].iter().copied().filter(|r| !seen.contains(r)).collect();
        let cost: u32 = cluster.iter().map(|&r| rt.store.chunks[r].token_count).sum();
        if !chosen.is_empty() && budget + cost > a.max_context_tokens {
            continue;
        }
        budget += cost;
        for r in cluster {
            seen.insert(r);
            chosen.push(r);
        }
    }
    Ok((chosen, lore_added))
}

/// A generated passage placed before the retrieved ones: a topic summary or a new dossier (`id` is what answers cite).
#[derive(Clone)]
pub(crate) struct Pre {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) text: String,
    /// After the retrieved passages instead of before them (a new dossier).
    pub(crate) last: bool,
    /// Adds INFER_RULE to the system prompt (the game-data passage).
    pub(crate) rule: bool,
    /// A scene of the event the question names where its character speaks or is named (`scoped_scenes`): adds
    /// SCOPED_RULE, and the same chunk is not repeated among the retrieved passages.
    pub(crate) scoped: bool,
}

impl Pre {
    /// A passage before the retrieved ones that adds no rule.
    pub(crate) fn new(id: String, label: String, text: String) -> Self {
        Self { id, label, text, last: false, rule: false, scoped: false }
    }

    /// Placed after the retrieved passages (`last`).
    pub(crate) fn after(mut self) -> Self {
        self.last = true;
        self
    }

    /// Adds INFER_RULE (`rule`).
    pub(crate) fn with_rule(mut self) -> Self {
        self.rule = true;
        self
    }

    /// A scoped scene (`scoped`).
    pub(crate) fn in_scope(mut self) -> Self {
        self.scoped = true;
        self
    }
}
