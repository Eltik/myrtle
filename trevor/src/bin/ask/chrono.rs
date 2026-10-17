//! Timeline notes: the in-world chronology lines a time question gets, and the corpus test of a proper name.

use std::collections::{BTreeSet, HashMap};

use trevor::search::runtime::Runtime;
use trevor::tools::{contains_words, norm};

/// Trevor's chronology, for timeline notes: storyline position per event, dated and derived lines per story.
#[derive(Default)]
pub(crate) struct Chrono {
    pub(crate) groups: std::collections::BTreeMap<String, serde_json::Value>,
    pub(crate) lines: HashMap<String, Vec<String>>,
    /// The dated lines of artifacts/chrono/terra_history.md as (year, line), for a when-question's subject (2026-10-03).
    pub(crate) history: Vec<(f64, String)>,
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

/// Whether a name is a proper name by the corpus (2026-10-03 night 8): its lowercase form occurs
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

