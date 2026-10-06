//! Timeline and identity notes: the in-world chronology lines a time question gets, and the identity
//! links (one person under several names) found in the retrieved passages.

use std::collections::{BTreeSet, HashMap};

use trevor::search::runtime::Runtime;
use trevor::tools::{Tools, contains_words, norm};

use crate::retrieval::GENERATED_KINDS;
use crate::terms::count_word;

/// Trevor's chronology, for timeline notes: storyline position per event, dated and derived lines per story.
#[derive(Default)]
pub(crate) struct Chrono {
    pub(crate) groups: std::collections::BTreeMap<String, serde_json::Value>,
    pub(crate) lines: HashMap<String, Vec<String>>,
    /// The dated lines of artifacts/chrono/terra_history.md as (year, line), for a when-question's subject (2026-10-03).
    pub(crate) history: Vec<(f64, String)>,
    /// Identity links of identities.v2.json: (name, other, relation, quote, where story id, stated), for identity notes.
    pub(crate) links: Vec<(String, String, String, String, String, bool)>,
}

pub(crate) fn load_chrono() -> Chrono {
    let mut c = Chrono::default();
    if let Ok(b) = std::fs::read("artifacts/chrono/timeline_v1.json") {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&b) {
            for g in v["groups"].as_array().into_iter().flatten() {
                if let Some(id) = g["groupId"].as_str() {
                    c.groups.insert(id.to_owned(), g.clone());
                }
            }
        }
    }
    let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
    for l in read("artifacts/chrono/events.jsonl").lines() {
        let Ok(e) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        if e["is_year"].as_bool() == Some(true) && e["kind"].as_str() == Some("story") {
            let tag = if e["present"].as_bool() == Some(true) { "scene dated" } else { "stated" };
            c.lines.entry(e["source"].as_str().unwrap_or_default().to_owned()).or_default().push(format!(
                "{} ({tag}): {}", e["year"], e["line"].as_str().unwrap_or_default().chars().take(140).collect::<String>()));
        }
    }
    for l in read("artifacts/chrono/terra_history.md").lines() {
        // "- **1086**: text" or "- **ca. 600** (derived: ...): text"
        let Some(rest) = l.strip_prefix("- **") else { continue };
        let Some((year, _)) = rest.split_once("**") else { continue };
        if let Ok(y) = year.trim_start_matches("ca. ").trim().parse::<f64>() {
            c.history.push((y, rest.replacen("**", "", 1).trim().to_owned()));
        }
    }
    c.links = Tools::load(std::path::Path::new(".")).identity_links;
    for l in read("artifacts/chrono/derived.jsonl").lines() {
        let Ok(d) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        let approx = if d["approximate"].as_bool() == Some(true) { "about " } else { "" };
        c.lines.entry(d["storyId"].as_str().unwrap_or_default().to_owned()).or_default().push(format!(
            "{approx}{} (derived, counted from {} via {}): \"{}\"",
            d["year"], d["from"].as_f64().map_or(String::new(), |f| format!("{f:.0}")),
            d["basis"].as_str().unwrap_or_default(), d["quote"].as_str().unwrap_or_default().chars().take(120).collect::<String>()));
    }
    c
}

/// The chronology, loaded once per process (the detector listing calls it per question).
pub(crate) fn load_chrono_cached() -> &'static Chrono {
    static C: std::sync::OnceLock<Chrono> = std::sync::OnceLock::new();
    C.get_or_init(load_chrono)
}

pub(crate) fn timeline_notes(rt: &Runtime, chrono: &Chrono, rows: &[usize], off: usize) -> String {
    // One note per event, listed in in-world order (storyline year, then main-story episode), so the
    // model reads the order instead of having to infer it from bounds: Episodes 6 and 7 share the bound
    // "at most 1098", and without the order the model called their order undeterminable.
    let mut events: Vec<(f64, u64, i64, String, Vec<usize>)> = Vec::new();
    let mut idx: HashMap<String, usize> = HashMap::new();
    for (i, &r) in rows.iter().enumerate() {
        let c = &rt.store.chunks[r];
        let Some(g) = chrono.groups.get(&c.group_id) else { continue };
        let k = *idx.entry(c.group_id.clone()).or_insert_with(|| {
            events.push((g["storylineYear"].as_f64().unwrap_or(9999.0), g["chapter"].as_u64().unwrap_or(0),
                         g["releaseTime"].as_i64().unwrap_or(0), c.group_id.clone(), Vec::new()));
            events.len() - 1
        });
        events[k].4.push(i + 1 + off);
    }
    events.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut out = String::from("\nTIMELINE NOTES (Trevor's chronology, listed in in-world order; years are estimates unless a line states them)\n");
    for (n, (_, chapter, _, gid, ps)) in events.iter().enumerate() {
        let g = &chrono.groups[gid];
        let bound = g["yearBound"].as_str().map_or(String::new(), |b| format!("{b} "));
        let ep = if *chapter > 0 || gid == "main_0" { format!(", main story Episode {chapter}") } else { String::new() };
        let list = ps.iter().map(|p| format!("[{p}]")).collect::<Vec<_>>().join(" ");
        out.push_str(&format!("{}. {}{ep}, passages {list}: storyline {bound}about {:.0} ({})\n", n + 1,
            g["name"].as_str().unwrap_or(gid), g["storylineYear"].as_f64().unwrap_or(0.0), g["basis"].as_str().unwrap_or_default()));
        let mut seen = BTreeSet::new();
        for &p in ps {
            let sid = &rt.store.chunks[rows[p - 1 - off]].story_id;
            if seen.insert(sid.clone()) {
                for l in chrono.lines.get(sid).into_iter().flatten().take(3) {
                    out.push_str(&format!("   - [{p}] {l}\n"));
                }
            }
        }
    }
    out
}

/// The chronology lines of `chronology_for` as the numbered TIME EVIDENCE block of `--time-evidence` (2026-10-03 night 8:
/// ian17 had the 1086 and 1090 to 1094 lines in its prompt and still declined without an estimate).
pub(crate) fn time_evidence_block(lines: &str) -> String {
    let items: Vec<&str> = lines.lines().filter_map(|l| l.strip_prefix("- ")).collect();
    let mut out = String::from("\nTIME EVIDENCE (dated anchors from Trevor's chronology of the whole story, oldest first; use them for the estimate)\n");
    for (i, l) in items.iter().enumerate() {
        out.push_str(&format!("T{}. {l}\n", i + 1));
    }
    out
}

/// Question words that never name a when-question's subject.
pub(crate) const QUESTION_WORDS: &[&str] = &["When", "What", "Which", "Who", "Why", "How", "Where", "Did", "Does", "Do", "Is", "Was", "Were", "Are", "In", "The"];

/// The dated lines of Trevor's chronology that name a when-question's subject (2026-10-03, leftover c: "When was Rhodes
/// Island founded?" had timeline notes only for the events its passages came from, none about the founding). Subjects
/// are the question's runs of capitalized words (question words left out); a line counts when it holds one as whole
/// words, case folded. Lines that share more of the question's other words (first 4 letters) come first, then the
/// earliest; at most 8, listed by year. Empty when no line names a subject.
pub(crate) fn chronology_for(q: &str, chrono: &Chrono) -> String {
    let words: Vec<&str> = q.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-')).filter(|w| !w.is_empty()).collect();
    let mut subjects: Vec<String> = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for w in words.iter().chain(std::iter::once(&"")) {
        if w.chars().next().is_some_and(char::is_uppercase) && !QUESTION_WORDS.contains(w) {
            run.push(w);
        } else if !run.is_empty() {
            subjects.push(run.join(" "));
            run.clear();
        }
    }
    let subjects: Vec<String> = subjects.into_iter().map(|x| norm(&x)).filter(|x| x.chars().count() >= 3).collect();
    if subjects.is_empty() {
        return String::new();
    }
    let stems: Vec<String> = words.iter().filter(|w| w.len() >= 4 && !w.chars().next().is_some_and(char::is_uppercase))
        .map(|w| w.to_lowercase().chars().take(4).collect()).collect();
    let mut hits: Vec<(usize, f64, &String)> = chrono.history.iter().filter_map(|(y, l)| {
        let nl = norm(l);
        subjects.iter().any(|x| contains_words(&nl, x)).then(|| {
            let shared = stems.iter().filter(|st| nl.split(' ').any(|w| w.starts_with(st.as_str()))).count();
            (shared, *y, l)
        })
    }).collect();
    if hits.is_empty() {
        return String::new();
    }
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.total_cmp(&b.1)));
    hits.truncate(8);
    hits.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut out = format!("\nCHRONOLOGY LINES THAT NAME {} (Trevor's dated lines from the whole story, not only the passages; a derived \
year says how it was counted)\n", subjects.join(", ").to_uppercase());
    for (_, _, l) in hits {
        out.push_str(&format!("- {l}\n"));
    }
    out
}

/// Identity links whose source line is among the passages (2026-10-03, leftover a): Iris's file states that Mabel was
/// the deceased Operator Bluishsilver, but no answer took the step. Stated links quote their line; inferred links that
/// share a name with a stated one follow, labelled as inferences. At most 4 lines; empty when no passage is a source.
pub(crate) fn identity_notes(rt: &Runtime, rows: &[usize], chrono: &Chrono) -> String {
    let stories: BTreeSet<&str> = rows.iter().map(|&r| rt.store.chunks[r].story_id.as_str()).collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut lines: Vec<String> = Vec::new();
    let mut names: BTreeSet<String> = BTreeSet::new();
    for (a, b, rel, quote, at, stated) in &chrono.links {
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if !*stated || rel == "title" || !stories.contains(at.as_str()) || lines.len() >= 4 || !seen.insert(key) {
            continue;
        }
        names.insert(a.clone());
        names.insert(b.clone());
        lines.push(format!("- {a} and {b} are the same person ({}, stated: \"{quote}\")", rel.replace('_', " ")));
    }
    for (a, b, _, _, _, stated) in &chrono.links {
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if *stated || lines.len() >= 4 || !(names.contains(a) || names.contains(b)) || !seen.insert(key) {
            continue;
        }
        lines.push(format!("- {a} may be {b} (an inference from the names, not stated)"));
    }
    if lines.is_empty() {
        return String::new();
    }
    format!("\nIDENTITY LINKS (from Trevor's identity table, for people the passages name)\n{}\n", lines.join("\n"))
}

/// Whether a name is a proper name by the corpus (`--identity-labels`, 2026-10-03 night 8): its lowercase form occurs
/// as a whole word under 1% as often as the name itself across the loaded chunks ("Operator" 1,168 of 6,260 and "Doctor"
/// 676 of 4,317 are not; "Iris" 3 of 363, "Bluishsilver" 0 of 12 are). A name with no lowercase letters to fold, or
/// never written, is not.
pub(crate) fn proper_name(rt: &Runtime, name: &str) -> bool {
    let lower = name.to_lowercase();
    if lower == name {
        return false;
    }
    let count = |needle: &str| -> usize {
        rt.store.chunks.iter().map(|c| c.text.match_indices(needle).filter(|(i, _)| {
            let before = c.text[..*i].chars().next_back();
            let after = c.text[i + needle.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        }).count()).sum()
    };
    let exact = count(name);
    exact > 0 && count(&lower) * 100 < exact
}

/// Stated identity links (not titles) whose source story is among the passages and whose two names are proper names,
/// as (story id, name, other, relation, quote); one per pair, at most 4 (`--identity-labels`).
pub(crate) fn label_links(rt: &Runtime, rows: &[usize], chrono: &Chrono) -> Vec<(String, String, String, String, String)> {
    let stories: BTreeSet<&str> = rows.iter().map(|&r| rt.store.chunks[r].story_id.as_str()).collect();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut proper: HashMap<String, bool> = HashMap::new();
    let mut out = Vec::new();
    for (a, b, rel, quote, at, stated) in &chrono.links {
        if !*stated || rel == "title" || !stories.contains(at.as_str()) || out.len() >= 4 {
            continue;
        }
        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
        if seen.contains(&key) {
            continue;
        }
        let ok = [a, b].iter().all(|n| *proper.entry((*n).clone()).or_insert_with(|| proper_name(rt, n)));
        if ok {
            seen.insert(key);
            out.push((at.clone(), a.clone(), b.clone(), rel.replace('_', " "), quote.clone()));
        }
    }
    out
}

/// A passage label with the identity links its story states, resolved: "...; this passage states that Bluishsilver's
/// former name is Mabel (\"Mabel, the owner of the radio, was ...\")".
pub(crate) fn identity_label(label: &str, story: &str, links: &[(String, String, String, String, String)]) -> String {
    let mut out = label.to_owned();
    for (_, a, b, rel, quote) in links.iter().filter(|l| l.0 == story) {
        out.push_str(&format!("; this passage states that {a} and {b} are the same person, {b} being the {rel} of {a} (\"{quote}\")"));
    }
    out
}

/// Identity expansion (`--identity-expand`, 2026-10-03 night 9): for each stated identity link whose source passage is
/// among `rows` (as the identity labels pick them) and names a capitalized, proper word of the question (Iris), the
/// chunks naming a linked name and that word together (the interlude where Iris looks for Mabel Grimm), by the
/// smaller of the two counts, at most `k` in all, inserted right after the link's source passage.
pub(crate) fn identity_expand(rt: &Runtime, chrono: &Chrono, q: &str, mut rows: Vec<usize>, k: usize) -> (Vec<usize>, HashMap<usize, String>) {
    let mut notes: HashMap<usize, String> = HashMap::new();
    let links = label_links(rt, &rows, chrono);
    if links.is_empty() {
        return (rows, notes);
    }
    let mut qwords: Vec<&str> = q.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().next().is_some_and(char::is_uppercase) && w.chars().count() >= 3).collect();
    qwords.dedup();
    let mut proper: HashMap<String, bool> = HashMap::new();
    let mut added = 0;
    for (at, x, y, rel, _) in &links {
        let Some(pos) = rows.iter().rposition(|&r| rt.store.chunks[r].story_id == *at) else { continue };
        let src: Vec<&str> = rows.iter().filter(|&&r| rt.store.chunks[r].story_id == *at).map(|&r| rt.store.chunks[r].text.as_str()).collect();
        let named: Vec<&str> = qwords.iter().copied()
            .filter(|w| w != x && w != y && src.iter().any(|t| count_word(t, w) > 0))
            .filter(|w| *proper.entry((*w).to_owned()).or_insert_with(|| proper_name(rt, w))).collect();
        if named.is_empty() {
            continue;
        }
        let mut cand: Vec<(usize, usize, usize)> = Vec::new();
        for (row, c) in rt.store.chunks.iter().enumerate() {
            if c.story_id == *at || rows.contains(&row) || GENERATED_KINDS.contains(&c.group_id.as_str()) {
                continue;
            }
            let n = count_word(&c.text, x).max(count_word(&c.text, y));
            let m = if n == 0 { 0 } else { named.iter().map(|w| count_word(&c.text, w)).max().unwrap_or(0) };
            if m > 0 {
                cand.push((n.min(m), n + m, row));
            }
        }
        cand.sort_by(|p, r| r.0.cmp(&p.0).then(r.1.cmp(&p.1)).then(p.2.cmp(&r.2)));
        let take: Vec<usize> = cand.iter().take(k - added).map(|c| c.2).collect();
        added += take.len();
        for (i, r) in take.into_iter().enumerate() {
            // The resolved link on the added passage itself: the answer model does not join two passages on its own
            // (ian24 declined with both in its prompt, 2026-10-03 night 9).
            let text = &rt.store.chunks[r].text;
            let (named, other) = if count_word(text, x) > 0 { (x, y) } else { (y, x) };
            notes.insert(r, format!("; this passage names {named}, who is the same person as {other} ({y} being the {rel} of {x}, as \
{at} states)"));
            rows.insert(pos + 1 + i, r);
        }
        if added >= k {
            break;
        }
    }
    (rows, notes)
}
