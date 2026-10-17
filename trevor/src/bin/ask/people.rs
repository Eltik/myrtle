//! The characters and operators a question names: by file headings, real names, identity links and short names.

use std::collections::HashMap;

use trevor::search::runtime::Runtime;
use trevor::tools::{Tools, contains_words, norm};

use crate::chrono::proper_name;
use crate::kinds::CHARACTER_GROUPS;

/// The characters whose operator file or dossier the question names, as (story id, the name the question used, the
/// file's own name), longest name first. A file's names are its heading's ("Operator archive: Midnight", "Character
/// profile: Kal'tsit (also known as AMa-10, ...)"), plus, for each of them, the operator's real name from
/// `real_names.jsonl` (and its first word when it has two or more: "Margaret Nearl" gives "Margaret") and the names of its
/// identity links. Matched as whole words, case folded, 3 letters or more. The real names matter because files carry
/// only codenames ("Margaret vs Degenbrecher" finds Nearl's files through "Margaret Nearl").
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

/// Whether an opinion question gets candidates from operator files and dossiers: it names a character (`people_named`,
/// or a file `named_characters` finds) or a nation, place, race or organization (`Tools::names_group_topic`), or asks
/// who or which. When it does not, the opinion gate (the default, `--no-opinion-gate`) gives a question the keyword
/// test also calls an opinion, naming no proper word, OPINION_RULE ("the story does not settle it", then the evidence)
/// without candidates, since candidates for "In lore how strong are my favourite characters?" would be presented as
/// the asker's favourites.
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
        let cap: String = cs.next().map(char::to_uppercase).into_iter().flatten()
            .chain(cs.flat_map(char::to_lowercase))
            .collect();
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
    let mut names: Vec<&str> = tools.operator_names().into_iter()
        .chain(tools.speaker_names())
        .filter(|x| hit(x))
        .collect();
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
            .chain(tools.identities.iter().flatten().map(String::as_str))
            .find(|x| norm(x) == k)
            .unwrap_or_default()
            .to_owned();
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
/// operator's name and is no operator's whole name ("Mutsumi" is Mutsumi Wakaba); for the own-file fallback.
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
