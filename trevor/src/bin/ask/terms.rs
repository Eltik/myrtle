//! "What is X?" questions: finding the term, its glossary lines and the passages that define it.

use std::collections::{BTreeSet, HashMap};

use trevor::search::runtime::Runtime;
use trevor::tools::Tools;

use crate::retrieval::{GENERATED_KINDS, RECORD_KINDS};

/// Whole-word occurrences of `needle` in `text`, case-sensitive.
pub(crate) fn count_word(text: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }
    text.match_indices(needle).filter(|(i, _)| {
        let before = text[..*i].chars().next_back();
        let after = text[i + needle.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    }).count()
}

/// Lowercase letters and digits only: "Khagan quest" and "Khaganquest" are both "khaganquest".
pub(crate) fn compact(x: &str) -> String {
    x.chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Term lookup (`--term-lookup`, 2026-10-03 night 9): the corpus's spelling of a term and the chunks naming it most.
/// The term matches one to three consecutive words of a chunk written together or apart ("khagan quest" finds
/// "Khaganquest"); the spelling is the commonest surface form; the chunks are at most `k` of distinct stories, records
/// and files first, then by how often they name it.
pub(crate) fn term_lookup(rt: &Runtime, term: &str, k: usize) -> Option<(String, Vec<usize>)> {
    let key = compact(term);
    if key.chars().count() < 4 {
        return None;
    }
    let mut forms: HashMap<String, usize> = HashMap::new();
    let mut hits: Vec<(bool, usize, usize)> = Vec::new();
    for (row, c) in rt.store.chunks.iter().enumerate() {
        if GENERATED_KINDS.contains(&c.group_id.as_str()) || !compact(&c.text).contains(&key) {
            continue;
        }
        let spans: Vec<(usize, usize)> = {
            let mut v = Vec::new();
            let mut start = None;
            for (i, ch) in c.text.char_indices() {
                match (ch.is_alphanumeric(), start) {
                    (true, None) => start = Some(i),
                    (false, Some(s0)) => { v.push((s0, i)); start = None; }
                    _ => {}
                }
            }
            if let Some(s0) = start { v.push((s0, c.text.len())); }
            v
        };
        let mut n = 0;
        for i in 0..spans.len() {
            let mut acc = String::new();
            for j in i..(i + 3).min(spans.len()) {
                acc.push_str(&compact(&c.text[spans[j].0..spans[j].1]));
                if acc.len() > key.len() || !key.starts_with(acc.as_str()) {
                    break;
                }
                if acc == key {
                    *forms.entry(c.text[spans[i].0..spans[j].1].to_owned()).or_default() += 1;
                    n += 1;
                    break;
                }
            }
        }
        if n > 0 {
            hits.push((!RECORD_KINDS.contains(&c.group_id.as_str()), n, row));
        }
    }
    let spelling = forms.into_iter().max_by(|x, y| x.1.cmp(&y.1).then(y.0.cmp(&x.0)))?.0;
    hits.sort_by(|x, y| x.0.cmp(&y.0).then(y.1.cmp(&x.1)).then(x.2.cmp(&y.2)));
    let mut seen = BTreeSet::new();
    let rows = hits.into_iter().filter(|h| seen.insert(rt.store.chunks[h.2].story_id.clone())).take(k).map(|h| h.2).collect();
    Some((spelling, rows))
}

/// The first `n` sentences of a summary, its section headings skipped (a deep entry starts with "What it is").
pub(crate) fn first_sentences(text: &str, n: usize) -> String {
    let body: Vec<&str> = text.lines().map(str::trim).filter(|l| l.ends_with(['.', '!', '?', ')', '"']) || l.len() > 60).collect();
    let joined = body.join(" ");
    let mut out = String::new();
    let mut count = 0;
    for part in joined.split_inclusive(". ") {
        out.push_str(part);
        count += 1;
        if count >= n {
            break;
        }
    }
    out.trim().to_owned()
}

/// Whole-word, case-sensitive occurrences of `f` in `text`.
pub(crate) fn whole_word_at(text: &str, f: &str) -> Option<usize> {
    trevor::tools::whole_word_matches(text, f).next()
}

/// The glossary of a term lookup: the served topics the term's chunks name (case-sensitive whole words, by first
/// mention), not the term itself, at most `k`, each as the first two sentences of its summary. With `rarity` (default
/// since 2026-10-05), the rarest first: fewest chunks of `rarity` naming one of the topic's forms, first mention breaking
/// ties ("What is the khagan quest?" got Rhodes Island, Terra and Precursors, the generic topics, on night 9).
pub(crate) fn term_glossary(tools: &Tools, texts: &[&str], term: &str, k: usize, rarity: Option<&Runtime>) -> Vec<(String, String)> {
    let key = compact(term);
    let Some(rows) = tools.topics.as_ref() else { return Vec::new() };
    let all = texts.join("\n");
    let mut found: Vec<(usize, usize, String)> = Vec::new();
    for r in rows {
        let name = r["topic"].as_str().unwrap_or_default();
        let forms: Vec<&str> = std::iter::once(name).chain(r["aliases"].as_array().into_iter().flatten().filter_map(|x| x.as_str())).collect();
        if forms.iter().any(|f| compact(f) == key) {
            continue;
        }
        let forms: Vec<&str> = forms.into_iter().filter(|f| f.chars().next().is_some_and(char::is_uppercase) && f.chars().count() >= 4).collect();
        let first = forms.iter().filter_map(|f| whole_word_at(&all, f)).min();
        if let Some(i) = first {
            let df = rarity.map_or(0, |rt| rt.store.chunks.iter().filter(|c| !GENERATED_KINDS.contains(&c.group_id.as_str())
                && forms.iter().any(|f| whole_word_at(&c.text, f).is_some())).count());
            found.push((df, i, name.to_owned()));
        }
    }
    found.sort();
    found.into_iter().filter_map(|(_, _, name)| tools.topic(&name).ok().map(|t| (t.topic, first_sentences(&t.text, 2))))
        .filter(|(_, x)| !x.is_empty()).take(k).collect()
}
