//! Retrieval: the passages a question gets from the corpus (hybrid search, typed sources, opinion
//! candidates), the corpora it can switch between, and name expansion.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use trevor::corpus::chunk::Chunk;
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::{Runtime, RuntimeArgs};
use trevor::tools::{Tools, contains_words, norm};

use crate::chrono::proper_name;
use crate::cli::{Args, lore_on};
use crate::detect::asks_origin;
use crate::form::Form;

/// Hits of `extra` and `hits` alternately, first seen wins.
pub(crate) fn interleave(extra: Vec<trevor::search::pipeline::Hit>, hits: Vec<trevor::search::pipeline::Hit>) -> Vec<trevor::search::pipeline::Hit> {
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

/// Character sources: operator files and Trevor's character dossiers.
pub(crate) const CHARACTER_GROUPS: [&str; 2] = ["archive", "profile"];
/// Candidates an opinion question gathers from character sources, and at most this many per character.
pub(crate) const CANDIDATES: usize = 6;
pub(crate) const CANDIDATE_PER_CHARACTER: usize = 2;

#[derive(Deserialize)]
pub(crate) struct Names {
    pub(crate) story: String,
    pub(crate) group: String,
}

/// The gold-set pipeline's names table (`artifacts/goldgen/names.json`), empty when missing or unparsable.
pub(crate) fn load_names() -> HashMap<String, Names> {
    std::fs::read("artifacts/goldgen/names.json").ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

/// A story passage's label: "story: <group>, <story name>" from the names table, or "story: <id>" for a story it lacks.
pub(crate) fn story_label(names: &HashMap<String, Names>, story_id: &str) -> String {
    names.get(story_id).map_or_else(|| format!("story: {story_id}"), |n| format!("story: {}, {}", n.group, n.story))
}

/// The label of an operator-file or story passage: "operator file: <char id>", else [`story_label`].
pub(crate) fn file_or_story_label(names: &HashMap<String, Names>, c: &Chunk) -> String {
    match c.group_id.as_str() {
        "archive" => format!("operator file: {}", c.story_id.trim_start_matches("archive_")),
        _ => story_label(names, &c.story_id),
    }
}

/// P4 unit kinds: the typed sources and Trevor's topic summaries, which a lore query leaves out.
pub(crate) const TYPED: [&str; 7] = ["module", "voice", "skin", "is", "enemy", "item", "topic"];

/// The operator a typed unit belongs to, from its heading line: "Outfit: Newsgirl (Amiya)", "Voice lines: Texas",
/// "Module story: Drone Control Module.P (Magallan's module, SUM-X)"; None for IS, enemy and item units.
pub(crate) fn unit_owner(heading: &str) -> Option<String> {
    if let Some(n) = heading.strip_prefix("Voice lines: ") {
        return Some(n.trim().to_owned());
    }
    let open = heading.rfind('(')?;
    let inner = heading[open + 1..].trim_end_matches(')');
    let name = inner.split(['\'', '\u{2019}', ',']).next()?.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// What `passages` put first for a typed source: how many source units, then how many lore passages after them.
#[derive(Default, Clone, Copy)]
pub(crate) struct Lore {
    pub(crate) units: usize,
    pub(crate) lore: usize,
}

/// Passages for `q`, and for a typed source with its lore (see below) how many units and lore hits lead them.
pub(crate) fn passages(rt: &mut Runtime, q: &str, a: &Args, kind: Option<&str>, form: Option<&Form>) -> Result<(Vec<usize>, Option<Lore>)> {
    let cfg = RetrievalConfig { mode: Mode::Hybrid, k: a.k, ..RetrievalConfig::default() };
    let mut hits = rt.retrieve(q, &cfg)?;
    if a.rerank_add > 0 {
        let keep = a.k.saturating_sub(a.rerank_add).min(hits.len());
        let rr = rt.retrieve(q, &RetrievalConfig { rerank_top: 40, k: 40, ..cfg })?;
        let mut head: Vec<_> = hits[..keep].to_vec();
        for h in rr {
            if head.len() >= a.k {
                break;
            }
            if !head.iter().any(|x| x.row == h.row) {
                head.push(h);
            }
        }
        let rest: Vec<_> = hits.into_iter().filter(|h| !head.iter().any(|x| x.row == h.row)).collect();
        hits = head.into_iter().chain(rest).collect();
    }
    let mut lore_added: Option<Lore> = None;
    let evidence = form.and_then(|f| f.evidence.as_deref()).filter(|e| !e.trim().is_empty());
    // A yes/no question interleaves the hits of its evidence query with its own: "Has Harold married twice? Does he have
    // children?" ranked the lines naming his wife and daughter 29th to 44th, and "Harold wife daughter children married"
    // ranks them 1st to 8th (2026-09-30).
    if let (Some(ev), Some("yes_no")) = (evidence, form.map(|f| f.form.as_str())) {
        let extra = rt.retrieve(ev, &cfg)?;
        hits = interleave(extra, hits);
    }
    // `--story-query`: the story-words query's hits interleaved with the question's (and the yes/no evidence's).
    if let Some(sq) = form.and_then(|f| f.story.as_deref()) {
        let extra = rt.retrieve(sq, &cfg)?;
        hits = interleave(extra, hits);
    }
    let evidence = evidence.filter(|_| form.is_some_and(|f| f.form == "opinion"));
    // A judgment question gathers candidates first: the evidence query ("skilled cook, culinary talent") over a wide list,
    // kept to operator files and dossiers, at most two chunks per character. "Who is the best cook?" retrieved the Dungeon
    // Meshi collaboration and no operator file in its top 100; the evidence query puts Matterhorn's, Hung's, Lee's and
    // Gummy's files in the top 52 (2026-09-30). They take no neighbours, so six candidates cost six chunks. Characters the
    // question names come first, each from its own files: "Midnight or Matsukiri?" otherwise filled four of six places
    // with other operators' files and dropped Midnight's.
    let mut bare: BTreeSet<usize> = BTreeSet::new();
    if let Some(ev) = evidence {
        let mut per: HashMap<String, usize> = HashMap::new();
        let mut cands = Vec::new();
        // At most two files per name the question uses ("Margaret" is Nearl's and Nearl the Radiant Knight's real name
        // and a dossier's title), six in all.
        let mut per_name: HashMap<String, usize> = HashMap::new();
        let named: Vec<_> = form.map(|f| f.named.clone()).unwrap_or_default().into_iter()
            .filter(|(_, asked, _)| { let n = per_name.entry(asked.to_lowercase()).or_default(); *n += 1; *n <= 2 }).take(CANDIDATES).collect();
        for (sid, asked, file) in named {
            let own = rt.retrieve(&format!("{file} {asked} {ev}"), &RetrievalConfig { k: 60, ..cfg })?;
            let mine: Vec<_> = own.into_iter().filter(|h| rt.store.chunks[h.row].story_id == sid).take(CANDIDATE_PER_CHARACTER).collect();
            // A named character always gets its file: the first chunk when the query ranks none of it in the top 60.
            if mine.is_empty() {
                if let Some(&row) = rt.store.story_rows(&sid).first() {
                    cands.push(trevor::search::pipeline::Hit { row, score: 0.0, dense_rank: None, bm25_rank: None, fused_rank: None });
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
        // Combat outfit say?" put Swire's Bloodline of Combat outfit third, 2026-09-30).
        if lore_here {
            let owner_of = |row: usize| unit_owner(rt.store.chunks[row].text.lines().next().unwrap_or_default());
            if let Some(o) = pref.first().and_then(|h| owner_of(h.row)) {
                pref.retain(|h| owner_of(h.row).is_none_or(|x| x == o));
            }
            pref.truncate(3);
        }
        // Lore for a typed source (Ian, 2026-09-30: "What does Angelina's Bloodline of Combat outfit say?" answered with
        // the outfit's two sentences and no lore): the best unit's own text is a second query, and its best story and
        // operator-file passages follow the units, so the answer can explain who and what the unit names.
        let mut lore = Vec::new();
        if lore_here {
            if let Some(top) = pref.first() {
                let text: String = rt.store.chunks[top.row].text.chars().take(600).collect();
                let lq = rt.retrieve(&text, &RetrievalConfig { k: 40, ..cfg })?;
                lore = lq.into_iter().filter(|h| !TYPED.contains(&rt.store.chunks[h.row].group_id.as_str())).take(4).collect();
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
    // "Why did swire join the LGD?" ranks her story's L.G.D. years (#0009) 6th and its opening chunks 7th and 8th, and
    // the 9,000-token budget keeps only the first 6 hits' clusters, so the kidnapping (#0003, 20th) and the badge (#0005)
    // never reach the answer. The story with the most chunks in the top 20 (at least 3) goes first: its chunks there that
    // come before its best-ranked hit, without neighbours, in story order (at most 5), then that hit with its neighbours,
    // then the other hits. Two earlier versions (2026-10-03) added nothing to this question: the earliest top-60 scene of
    // the top 3 stories (her story is the 6th distinct one), and the dominant story's earlier chunks placed after the
    // first 4 hits while "earlier" was measured from the lowest position among the hits (#0000, so nothing came first).
    if !a.no_origin_chain && !a.no_route && asks_origin(q) {
        let wide = rt.retrieve(q, &RetrievalConfig { k: 20, ..cfg })?;
        let mut count: HashMap<&str, usize> = HashMap::new();
        for h in &wide {
            *count.entry(rt.store.chunks[h.row].story_id.as_str()).or_default() += 1;
        }
        let top = count.iter().filter(|(_, n)| **n >= 3).max_by_key(|(sid, n)| (**n, std::cmp::Reverse(**sid))).map(|(s, _)| (*s).to_owned());
        if let Some(sid) = top {
            let rows = rt.store.story_rows(&sid);
            let pos = |row: usize| rows.iter().position(|&r| r == row).unwrap_or(usize::MAX);
            let lead = hits.iter().chain(wide.iter()).find(|h| rt.store.chunks[h.row].story_id == sid).cloned();
            if let Some(lead) = lead {
                let mut add: Vec<_> = wide.iter().filter(|h| rt.store.chunks[h.row].story_id == sid && pos(h.row) < pos(lead.row))
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

/// The characters whose operator file or dossier the question names, as (story id, the name the question used, the
/// file's own name), longest name first. A file's names are its heading's ("Operator archive: Midnight", "Character
/// profile: Kal'tsit (also known as AMa-10, ...)"), plus, for each of them, the operator's real name from
/// `real_names.jsonl` (and its first word when it has two or more: "Margaret Nearl" gives "Margaret") and the names of its
/// identity links. Matched as whole words, case folded, 3 letters or more. "Margaret vs Degenbrecher" found no file
/// for Margaret before the real names were added: Nearl's files carry only codenames (2026-09-30).
pub(crate) fn named_characters(rt: &Runtime, tools: &Tools, q: &str) -> Vec<(String, String, String)> {
    let ql = q.to_lowercase();
    let whole = |n: &str| {
        let n = n.to_lowercase();
        n.chars().count() >= 3 && ql.match_indices(&n).any(|(i, _)| {
            ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                && ql[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
        })
    };
    let mut real: HashMap<String, Vec<String>> = HashMap::new();
    for r in tools.real_names.iter().flatten() {
        if let (Some(n), Some(rn)) = (r["name"].as_str(), r["realName"].as_str()) {
            let e = real.entry(norm(n)).or_default();
            e.push(rn.to_owned());
            let words: Vec<&str> = rn.split_whitespace().collect();
            if words.len() >= 2 {
                e.push(words[0].to_owned());
            }
        }
    }
    let mut out: Vec<(String, String, String)> = Vec::new();
    for c in &rt.store.chunks {
        if !CHARACTER_GROUPS.contains(&c.group_id.as_str()) || out.iter().any(|(s, _, _)| *s == c.story_id) {
            continue;
        }
        let head = c.text.lines().next().unwrap_or_default();
        let own: Vec<String> = if let Some(n) = head.strip_prefix("Operator archive: ") { vec![n.trim().to_owned()] }
            else if let Some(r) = head.strip_prefix("Character profile: ") {
                let (main, aka) = r.split_once(" (also known as ").unwrap_or((r, ""));
                std::iter::once(main.trim()).chain(aka.trim_end_matches(')').split(", ").map(str::trim))
                    .filter(|x| !x.is_empty()).map(str::to_owned).collect()
            } else { Vec::new() };
        let Some(file) = own.first().cloned() else { continue };
        let mut names = own.clone();
        for n in &own {
            let k = norm(n);
            names.extend(real.get(&k).into_iter().flatten().cloned());
            for ident in tools.identities.iter().filter(|i| i.iter().any(|x| norm(x) == k)) {
                names.extend(ident.iter().cloned());
            }
        }
        if let Some(n) = names.into_iter().filter(|n| whole(n)).max_by_key(String::len) {
            out.push((c.story_id.clone(), n, file));
        }
    }
    out.sort_by_key(|(_, n, _)| std::cmp::Reverse(n.len()));
    out
}

/// The opinion gate (`--opinion-gate`, opt-in since 2026-10-05 night): the question form's opinion branch gathers
/// candidates from operator files and dossiers only when the question names a character (`people_named`, or a file
/// `named_characters` finds) or a nation, place, race or organization (`Tools::names_group_topic`), or asks who or which;
/// otherwise OPINION_RULE ("the story does not settle it", then the evidence) without candidates. "In lore how strong are
/// my favourite characters?" (real r030) listed Degenbrecher's strength as if she were the asker's favourite (2026-10-05
/// regression run). Measured: responds 10 to 13 and handles 8 to 10 of the 16 real questions it touches by the
/// real-probe judge, but Qwen pairwise 2 wins and 3 losses (r039, r067, r068) over 19; two other answer rules for the
/// gated questions lost more (the keyword rule alone: 2 wins, 5 losses; OPINION_FORM_RULE without candidates: 3 and 3).
pub(crate) fn opinion_needs_candidates(rt: &Runtime, tools: &Tools, q: &str, named: bool) -> bool {
    let n = norm(q);
    named || tools.names_group_topic(q) || ["who", "which", "whom", "whose"].iter().any(|w| contains_words(&n, w))
        || !people_named(rt, tools, q).is_empty()
}

/// Whether a word of the question (3 letters or more), capitalized, is a proper name by the corpus (`proper_name`): "How
/// big do you think Fort barron is?" names Barron (13 capitalized, 0 lowercase), a place no topic or character list holds.
pub(crate) fn names_proper_word(rt: &Runtime, q: &str) -> bool {
    q.split(|c: char| !c.is_alphanumeric()).filter(|w| w.chars().count() >= 3).any(|w| {
        let mut cs = w.chars();
        let cap: String = cs.next().map(char::to_uppercase).into_iter().flatten().chain(cs.flat_map(char::to_lowercase)).collect();
        proper_name(rt, &cap)
    })
}

/// The characters a question names: operator names, story speakers and identity-link names it holds as whole words (case
/// and punctuation folded, a possessive allowed, 3 characters or more) that the corpus writes capitalized (`proper_name`),
/// one per person: a name inside a longer matched name, or linked to an earlier one by an identity link, is not counted
/// again ("Margaret" and "Margaret Nearl" are one).
pub(crate) fn people_named(rt: &Runtime, tools: &Tools, q: &str) -> Vec<String> {
    let n = norm(q);
    let np = norm(&q.replace('\u{2019}', "'").replace("'s", " "));
    let hit = |x: &str| { let k = norm(x); k.chars().count() >= 3
        && (contains_words(&n, &k) || contains_words(&np, &k)) };
    let mut names: Vec<&str> = tools.operator_names().into_iter().chain(tools.speaker_names()).filter(|x| hit(x)).collect();
    names.extend(tools.identities.iter().flatten().map(String::as_str).filter(|x| hit(x)));
    let mut keys: Vec<String> = names.into_iter().map(trevor::tools::norm).collect();
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));
    keys.dedup();
    let mut out: Vec<String> = Vec::new();
    for k in keys {
        let linked = |a: &str, b: &str| tools.identities.iter().any(|i| i.iter().any(|x| norm(x) == a)
            && i.iter().any(|x| norm(x) == b));
        if out.iter().any(|o| contains_words(o, &k) || linked(o, &k)) {
            continue;
        }
        let shown = tools.operator_names().into_iter().chain(tools.speaker_names())
            .chain(tools.identities.iter().flatten().map(String::as_str)).find(|x| norm(x) == k).unwrap_or_default().to_owned();
        if proper_name(rt, &shown) || capitalized_in(q, &shown) {
            out.push(k);
        }
    }
    out
}

/// Whether the question itself writes `name` capitalized after its first word ("Is Magallan and Emperor related?"): a
/// name that is also a common word ("Emperor", "Doctor", "Red", "Platinum") is not capitalized in the corpus often enough
/// for `proper_name`, but the asker marks it as a name.
pub(crate) fn capitalized_in(q: &str, name: &str) -> bool {
    let first = name.split_whitespace().next().unwrap_or_default();
    if first.is_empty() || !first.chars().next().is_some_and(char::is_uppercase) {
        return false;
    }
    q.match_indices(first).any(|(i, _)| i > 0
        && !q[..i].chars().next_back().is_some_and(char::is_alphanumeric)
        && !q[i + first.len()..].chars().next().is_some_and(char::is_alphanumeric))
}

/// Chunk kinds Trevor wrote itself, never a term's evidence.
pub(crate) const GENERATED_KINDS: [&str; 3] = ["profile", "summary", "topic"];
/// Chunk kinds that hold records and files, which define a term more often than a scene does.
pub(crate) const RECORD_KINDS: [&str; 7] = ["archive", "gametext", "is", "module", "item", "enemy", "skin"];

/// The operator a cited operator-file chunk belongs to ("archive_char_338_iris#0001" -> Iris).
pub(crate) fn file_owner(tools: &Tools, id: &str) -> Option<String> {
    id.starts_with("archive_").then(|| tools.source_owner(id.split('#').next().unwrap_or(id))).flatten()
}

/// The kinds whose text describes an object (outfit, item, module, enemy) and so names lore it does not explain.
/// Operator files and voice lines keep plain retrieval: all 5 answers the first two lore versions lost were those kinds
/// (2026-09-30: Kal'tsit's voice, Mon3tr's and two gold operator files, a gold voice question).
pub(crate) const LORE_KINDS: [&str; 4] = ["skin", "item", "module", "enemy"];

/// Chunks whose text holds `w` as a whole word, case folded (apostrophes count as letters, so "kal" is not in
/// "Kal'tsit").
pub(crate) fn word_chunks(rt: &Runtime, w: &str) -> usize {
    let w = w.to_lowercase();
    let edge = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '\''));
    rt.store.chunks.iter().filter(|c| {
        let t = c.text.to_lowercase();
        t.match_indices(&w).any(|(i, _)| edge(t[..i].chars().last()) && edge(t[i + w.len()..].chars().next()))
    }).count()
}

/// The question with the full name after each short name it uses ("What did bibeak design for kal" gets
/// "kal (Kal'tsit)"), for retrieval and the answer. A short name counts only when the corpus uses it as a word, but
/// rarely: in 1 to 49 chunks and at most a twentieth as often as the full name (kal: 6 chunks against Kal'tsit's
/// hundreds; "can" 9,817 and "ceo" 10 are left alone, and the typo "adn" (0 chunks) does not become Adnachiel).
pub(crate) fn expand_names(rt: &Runtime, tools: &Tools, q: &str) -> String {
    let mut out = q.to_owned();
    for (w, full) in tools.short_names(q) {
        let lead = full.split_whitespace().next().unwrap_or(&full).to_lowercase();
        let (dw, dn) = (word_chunks(rt, &w), word_chunks(rt, &lead));
        if (1..50).contains(&dw) && dw * 20 <= dn {
            if let Some(i) = out.find(&w) {
                out.insert_str(i + w.len(), &format!(" ({full})"));
            }
        }
    }
    out
}

/// The loaded corpus, and P4 loaded on first need when `--corpus` was left at its default (a batch can mix both).
pub(crate) struct Runtimes {
    pub(crate) main: Runtime,
    pub(crate) p4: Option<Runtime>,
    /// P4 plus the operator art units (`--art`), loaded on the first question routed to the "art" source.
    pub(crate) art: Option<Runtime>,
    /// The P4 directory of the lore served (`--lore`).
    pub(crate) p4_dir: &'static str,
    pub(crate) switch: bool,
    pub(crate) args: RuntimeArgs,
    /// Load the reranker (`--rerank-add`).
    pub(crate) rerank: bool,
    /// The spoiler horizon of the current `--serve` job, given to every corpus `pick` returns. None outside serve.
    pub(crate) horizon: Option<std::sync::Arc<trevor::search::horizon::Horizon>>,
}

impl Runtimes {
    pub(crate) fn pick(&mut self, p4: Option<bool>) -> Result<&mut Runtime> {
        if self.switch && p4 == Some(true) && self.args.corpus != std::path::Path::new(self.p4_dir) {
            if self.p4.is_none() {
                let mut a = self.args.clone();
                a.corpus = PathBuf::from(self.p4_dir);
                self.p4 = Some(Runtime::load(&a, true, true, self.rerank)?);
            }
            let rt = self.p4.as_mut().expect("loaded above");
            rt.horizon.clone_from(&self.horizon);
            return Ok(rt);
        }
        self.main.horizon.clone_from(&self.horizon);
        Ok(&mut self.main)
    }

    /// Whether a spoiler horizon is set (`--serve` jobs only): table answers, generated passages and the chronology
    /// notes are then left out (they are built from every story, later ones included).
    pub(crate) fn horizon_set(&self) -> bool {
        self.horizon.is_some()
    }

    /// Set the horizon on every loaded corpus (and on the ones `pick` loads later).
    pub(crate) fn set_horizon(&mut self, h: Option<std::sync::Arc<trevor::search::horizon::Horizon>>) {
        for rt in std::iter::once(&mut self.main).chain(self.p4.as_mut()).chain(self.art.as_mut()) {
            rt.horizon.clone_from(&h);
        }
        self.horizon = h;
    }

    /// The corpus for the "art" source: `ART_DIR` (P4 plus the art units), P4 when it has not been built.
    pub(crate) fn pick_art(&mut self) -> Result<&mut Runtime> {
        if !self.switch || !std::path::Path::new(ART_DIR).join("chunks.jsonl").exists() {
            return self.pick(Some(true));
        }
        if self.art.is_none() {
            let mut a = self.args.clone();
            a.corpus = PathBuf::from(ART_DIR);
            self.art = Some(Runtime::load(&a, true, true, self.rerank)?);
        }
        let rt = self.art.as_mut().expect("loaded above");
        rt.horizon.clone_from(&self.horizon);
        Ok(rt)
    }
}

/// P4 plus one unit per operator art image (`scripts/art_captions.py units`, then `build-units --units
/// artifacts/art/p4_art_units.jsonl --out artifacts/p4-art`). Read only for the router's "art" source (`--art`), so P4
/// and every other answer stay as they were.
pub(crate) const ART_DIR: &str = "artifacts/p4-art";

/// A generated passage placed before the retrieved ones: a topic summary or a new dossier (`id` is what answers cite).
#[derive(Clone)]
pub(crate) struct Pre {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) text: String,
    /// After the retrieved passages instead of before them (a new dossier: placed first it lost g0107, 2026-10-02).
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

/// The unit kinds the typed fallback tries, in no order: the best-ranked unit of the named operator decides the kind.
pub(crate) const FALLBACK_KINDS: [&str; 3] = ["voice", "module", "art"];

/// The operator a typed unit belongs to, from its first line ("Voice lines: X", "Module story: ... (X's module, ...)",
/// "Operator art: X, E2 ...").
pub(crate) fn typed_owner(heading: &str) -> Option<String> {
    if let Some(r) = heading.strip_prefix("Operator art: ") {
        return r.split(',').next().map(|n| n.trim().to_owned());
    }
    unit_owner(heading)
}

/// The one operator a question names (codenames of the attribute table, whole words after `norm`, the longest match
/// kept when one name holds another); None for none or several.
pub(crate) fn one_operator(q: &str, tools: &Tools) -> Option<String> {
    let nq = norm(q);
    let mut found: Vec<&str> = tools.operator_names().into_iter()
        .filter(|n| { let x = norm(n); x.chars().count() >= 3 && contains_words(&nq, &x) }).collect();
    found.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let mut keep: Vec<&str> = Vec::new();
    for n in found {
        if !keep.iter().any(|k| contains_words(&norm(k), &norm(n))) {
            keep.push(n);
        }
    }
    (keep.len() == 1).then(|| keep[0].to_owned())
}

/// [`one_operator`], or else the operator whose multi-word name starts with a word of the question that starts no other
/// operator's name and is no operator's whole name ("Mutsumi" is Mutsumi Wakaba, 2026-10-03); for the own-file fallback.
pub(crate) fn one_operator_wide(q: &str, tools: &Tools) -> Option<String> {
    if let Some(op) = one_operator(q, tools) {
        return Some(op);
    }
    let nq = norm(q);
    let names = tools.operator_names();
    let firsts: Vec<(String, &str)> = names.iter().filter_map(|n| {
        let x = norm(n);
        let first = x.split(' ').next()?.to_owned();
        (x.contains(' ') && first.chars().count() >= 4).then_some((first, *n))
    }).collect();
    let mut hit: Vec<&str> = firsts.iter().filter(|(f, _)| contains_words(&nq, f)
        && firsts.iter().filter(|(g, _)| g == f).count() == 1 && !names.iter().any(|n| norm(n) == *f))
        .map(|(_, n)| *n).collect();
    hit.dedup();
    (hit.len() == 1).then(|| hit[0].to_owned())
}
