//! Canon questions about Integrated Strategies endings: the per-run verdicts, the reference counts and
//! the ending a question names.

use std::collections::{BTreeSet, HashMap, HashSet};

use serde_json::Value;

use super::text::{contains_words, fuzzy_unique, norm, strip_citations};
use super::{IsEnding, ToolError, ToolResult, Tools, s};

impl Tools {
    /// Which IS endings later stories continue from, per run, with one quote each. The game never labels an ending
    /// canon; this is inferred from later stories only (`scripts/canon_refs.py`).
    ///
    /// # Errors
    /// Missing table or a run that does not resolve.
    pub fn canon(&self, run: Option<&str>) -> ToolResult {
        let runs = self.canon.as_ref().and_then(|v| v["runs"].as_array()).ok_or(ToolError::Missing("is_endings.json"))?;
        if self.opts.is_offset && run.is_some_and(is_fungimist) {
            return Ok(FUNGIMIST.to_owned());
        }
        let chosen: Vec<&Value> = match run {
            Some(r) => vec![self.resolve_run(r, runs)?],
            None => runs.iter().collect(),
        };
        let offset = usize::from(self.opts.is_offset);
        // The per-run verdicts (scripts/canon_refs.py, 2026-09-30) when every chosen run has one; else the reference
        // counts, as before.
        if !self.opts.canon_v1 && chosen.iter().all(|r| r["verdict"].is_object()) {
            let mut out = String::from("The game never labels an ending canon; Trevor's reading of the stories:\n");
            for r in chosen {
                out.push('\n');
                out.push_str(&canon_verdict(r, offset));
            }
            return Ok(out);
        }
        let mut out = String::from("The game never labels an Integrated Strategies ending as canon. What follows is inferred from later stories that continue from or refer to an ending, with one quote each.\n");
        for r in chosen {
            out.push('\n');
            out.push_str(&canon_run(r, offset));
        }
        Ok(out)
    }

    /// "IS2", "IS #2", "Integrated Strategies 2", "2" (the game's numbers: IS #2 is rogue_1), "rogue_1" (the data's
    /// id), a run name ("Mizuki & Caerula Arbor") or a unique part of one ("Mizuki"), within the same misspelling
    /// budget as names.
    pub(super) fn resolve_run<'a>(&self, r: &str, runs: &'a [Value]) -> Result<&'a Value, ToolError> {
        let nf = || ToolError::NotFound { what: "Integrated Strategies run", name: r.to_owned() };
        let n = norm(r);
        let digits: String = n.chars().filter(char::is_ascii_digit).collect();
        let numbered = n.split(' ').all(|w| matches!(w, "is" | "integrated" | "strategies" | "strategy" | "rogue" | "roguelike")
            || w.chars().all(|c| c.is_ascii_digit()) || w.strip_prefix("is").is_some_and(|x| x.chars().all(|c| c.is_ascii_digit())));
        if numbered && !digits.is_empty() {
            let id = match digits.parse::<usize>() {
                Ok(d) if self.opts.is_offset && !n.contains("rogue") => format!("rogue_{}", d.saturating_sub(1)),
                _ => format!("rogue_{digits}"),
            };
            return runs.iter().find(|x| s(x, "run") == id).ok_or_else(nf);
        }
        let names: Vec<(&Value, String)> = runs.iter().map(|x| (x, norm(&s(x, "runName")))).collect();
        if let Some((x, _)) = names.iter().find(|(_, rn)| *rn == n) {
            return Ok(x);
        }
        let part: Vec<&(&Value, String)> = names.iter().filter(|(_, rn)| n.len() >= 3 && (contains_words(rn, &n) || contains_words(&n, rn))).collect();
        if part.len() == 1 {
            return Ok(part[0].0);
        }
        fuzzy_unique(&n, names.iter().map(|(x, rn)| (*x, rn.as_str()))).ok_or_else(nf)
    }

    // ---------------------------------------------------------------- topics

    /// An Integrated Strategies ending a question asks about (2026-10-03, Ian's item 2): the run (an "IS" number,
    /// "Integrated Strategies" with a number, or a run's full name) and the ending (its number, "first" to "fifth", or its
    /// name), from `is_endings.json`. Returns (data run id, the run's numbered endings in the game's order as (number, name),
    /// the asked ending's number when the question names one). Endings without a number in their id are not numbered
    /// endings ("ro3_ending_c", The Fleetest of Light, has no script).
    #[must_use]
    pub fn is_ending_question(&self, q: &str) -> Option<IsEnding> {
        let n = norm(q);
        if !["ending", "endings", "endbook", "endbooks"].iter().any(|w| contains_words(&n, w)) {
            return None;
        }
        let runs = self.canon.as_ref()?["runs"].as_array()?;
        let offset = usize::from(self.opts.is_offset);
        let words: Vec<&str> = n.split(' ').collect();
        let mut run: Option<&Value> = None;
        for (i, w) in words.iter().enumerate() {
            let num = w.strip_prefix("is").filter(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit())).map(str::to_owned)
                .or_else(|| (matches!(*w, "is" | "strategies") && i + 1 < words.len()).then(|| words[i + 1].trim_start_matches('#').to_owned())
                    .filter(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit()) && (*w == "strategies" || words[..i].iter().all(|p| *p != "what"))));
            if let Some(d) = num.and_then(|d| d.parse::<usize>().ok()) {
                let id = format!("rogue_{}", d.saturating_sub(offset));
                run = runs.iter().find(|r| s(r, "run") == id);
                if run.is_some() {
                    break;
                }
            }
        }
        let run = run.or_else(|| runs.iter().find(|r| { let rn = norm(&s(r, "runName")); rn.len() >= 5 && contains_words(&n, &rn) }))?;
        let mut endings: Vec<(usize, String)> = run["endings"].as_array().into_iter().flatten().filter_map(|e| {
            let k = s(e, "endingId").rsplit('_').next()?.parse::<usize>().ok()?;
            Some((k, s(e, "name")))
        }).collect();
        endings.sort();
        let ords = ["first", "second", "third", "fourth", "fifth", "sixth"];
        let asked = endings.iter().find(|(k, name)| {
            let x = norm(name);
            (x.len() >= 4 && contains_words(&n, &x))
                || [format!("ending {k}"), format!("ending #{k}"), format!("ending no {k}"), format!("{} ending", ords.get(k - 1).copied().unwrap_or("-")),
                    format!("e{k}"), format!("end {k}"), format!("ending{k}")].iter().any(|p| contains_words(&n, &norm(p)))
        }).map(|(k, _)| *k);
        Some((s(run, "run"), s(run, "runName"), endings, asked))
    }
}

/// IS #1, which the EN game data has only as the untranslated CN `roguelike_table.json` (no roguelike_topic_table
/// entry), so no ending text or later reference can be read.
pub(super) const FUNGIMIST: &str = "Integrated Strategies #1, Ceobe's Fungimist, is not in the game data Trevor has: the EN data \
holds only an untranslated Chinese table for it, with no ending text, so Trevor cannot say which of its endings later \
stories continue from. The game never labels an Integrated Strategies ending as canon in any case. Trevor's IS data \
starts at #2, Phantom & Crimson Solitaire.";

/// "IS1", "IS #1", "Integrated Strategies 1", "Fungimist", "Ceobe's Fungimist".
pub(super) fn is_fungimist(r: &str) -> bool {
    let n = norm(r);
    if n.contains("fungimist") {
        return true;
    }
    let digits: String = n.chars().filter(char::is_ascii_digit).collect();
    digits == "1" && !n.contains("rogue")
        && n.split(' ').all(|w| matches!(w, "is" | "integrated" | "strategies" | "strategy" | "roguelike" | "1" | "is1"))
}

/// A run's heading with the game's number: "Mizuki & Caerula Arbor (Integrated Strategies #3)".
pub(super) fn run_heading(r: &Value, offset: usize) -> String {
    let no = s(r, "run").strip_prefix("rogue_").and_then(|d| d.parse::<usize>().ok()).map_or_else(|| s(r, "run"), |d| (d + offset).to_string());
    format!("{} (Integrated Strategies #{no}):\n", s(r, "runName"))
}

/// The judge's reasoning for a reader: its passage numbers mean nothing outside the prompt, so "Passages [1], [2]
/// and [4]" becomes "The stories" (one: "One story"), other bracket numbers go, ending ids become names, and the
/// text is cut to its first 3 sentences.
pub(super) fn reader_reasoning(text: &str, names: &HashMap<String, String>) -> String {
    let mut t = text.to_owned();
    for (id, name) in names {
        t = t.replace(&format!("{id} ({name})"), name).replace(id.as_str(), name);
    }
    let mut out = String::with_capacity(t.len());
    let mut rest = t.as_str();
    while let Some(i) = rest.find("assage") {
        let start = i.saturating_sub(1);
        let word_start = &rest[start..];
        let plural = word_start[1..].starts_with("assages");
        let after = &word_start[if plural { 8 } else { 7 }..];
        // A list of "[n]" joined by ", ", " and ", ", and ".
        let mut j = 0;
        let mut count = 0;
        loop {
            let tail = &after[j..];
            let sep = [", and ", " and ", ", ", " "].iter().find(|p| tail.starts_with(**p)).map_or(0, |p| p.len());
            let t2 = &tail[sep..];
            let digits = t2.chars().take_while(char::is_ascii_digit).count();
            if digits > 0 && (count == 0 || sep > 0) {
                j += sep + digits;
                count += 1;
                continue;
            }
            if t2.starts_with('[') {
                if let Some(close) = t2.find(']') {
                    if t2[1..close].chars().all(|c| c.is_ascii_digit() || c == ',' || c == ' ') && close > 1 {
                        j += sep + close + 1;
                        count += 1;
                        continue;
                    }
                }
            }
            break;
        }
        if count == 0 || !matches!(&rest[start..=start], "P" | "p") {
            out.push_str(&rest[..i + 6]);
            rest = &rest[i + 6..];
            continue;
        }
        let capital = &rest[start..=start] == "P";
        let phrase = match (count > 1 || plural, capital) {
            (true, true) => "The stories",
            (true, false) => "the stories",
            (false, true) => "One story",
            (false, false) => "one story",
        };
        out.push_str(&rest[..start]);
        out.push_str(phrase);
        rest = &after[j..];
    }
    out.push_str(rest);
    let clean = strip_citations(&out);
    // A sentence still about passage numbers ("while 3 and 4 describe local events") or about passages it set aside
    // ("Passages [3] and [8] were discounted", which reads as if every story were) is prompt bookkeeping; drop it.
    let sentences: Vec<&str> = clean.split_inclusive(". ").filter(|x| {
        let l = x.to_lowercase();
        !l.contains("passage") && !l.contains("discount")
    }).collect();
    sentences.iter().take(3).copied().collect::<String>().trim().to_owned()
}

/// One run from its verdict: the ending the stories treat as canon (or none, or undetermined) with the judge's
/// confidence and reasoning, whether the second model agrees, the stories relied on with a quote each, then every
/// other story linked to an ending. Hand grades (`grade`, `gradeNote`) are evaluation data and never printed.
pub(super) fn canon_verdict(r: &Value, offset: usize) -> String {
    let v = &r["verdict"];
    let names: HashMap<String, String> = r["endings"].as_array().into_iter().flatten()
        .map(|e| (s(e, "endingId"), s(e, "name"))).filter(|(i, _)| !i.is_empty()).collect();
    let name_of = |id: &str| names.get(id).cloned().unwrap_or_else(|| id.to_owned());
    let mut out = run_heading(r, offset);
    let id = s(v, "endingId");
    let conf = s(v, "confidence");
    match id.as_str() {
        "none" => out.push_str("No ending is treated as canon by any story.\n"),
        "undetermined" | "" => out.push_str("Undetermined: the stories Trevor has do not settle which ending is canon.\n"),
        _ => out.push_str(&format!("Treated as canon: {} ({conf} confidence).\n", if s(v, "name").is_empty() { name_of(&id) } else { s(v, "name") })),
    }
    let why = reader_reasoning(&s(v, "reasoning"), &names);
    if !why.is_empty() {
        out.push_str(&format!("Why: {why}\n"));
    }
    let second = &v["second"];
    if second.is_object() {
        let sid = s(second, "endingId");
        let reading = match sid.as_str() {
            "none" => "no ending".to_owned(),
            "undetermined" | "" => "undetermined".to_owned(),
            x => format!("{} ({} confidence)", name_of(x), s(second, "confidence")),
        };
        if second["agrees"].as_bool() == Some(true) {
            out.push_str(&format!("A second model agrees: {reading}.\n"));
        } else {
            out.push_str(&format!("A second model disagrees: it reads {reading}.\n"));
        }
    } else {
        out.push_str("No second model has read this run.\n");
    }
    let evidence: Vec<&Value> = r["evidence"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    let date = |e: &Value| e["released"].as_str().unwrap_or("date unknown").to_owned();
    // A quote credited to several endings (Highmore's "Gone will be the fins" to Precious Days and Stella Caerula) is
    // printed once, under its own heading, and counts for neither ending's story lines.
    let mut endings_of: HashMap<(String, String), BTreeSet<String>> = HashMap::new();
    for e in &evidence {
        endings_of.entry((s(e, "story"), s(e, "quote"))).or_default().insert(s(e, "endingId"));
    }
    let shared = |e: &Value| endings_of.get(&(s(e, "story"), s(e, "quote"))).is_some_and(|x| x.len() > 1);
    // One line per (ending, story): the longest quote, cut at 200 characters, and how many other passages it has.
    let story_line = |items: &[&Value]| -> String {
        let quotes: BTreeSet<String> = items.iter().map(|e| s(e, "quote")).collect();
        let best = quotes.iter().max_by_key(|q| q.chars().count()).cloned().unwrap_or_default();
        let best = if best.chars().count() > 200 { format!("{}...", best.chars().take(200).collect::<String>().trim_end()) } else { best };
        let best = best.split_whitespace().collect::<Vec<_>>().join(" ");
        let more = quotes.len().saturating_sub(1);
        let more = if more > 0 { format!(" (+{more} more passage{})", if more == 1 { "" } else { "s" }) } else { String::new() };
        format!("\"{best}\"{more}")
    };
    let group = |ending: &str, story: &str| -> Vec<&Value> {
        evidence.iter().copied().filter(|e| s(e, "endingId") == ending && s(e, "story") == story && !shared(e)).collect()
    };
    let mut done: HashSet<(String, String)> = HashSet::new();
    let cites: Vec<String> = v["cites"].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_owned)).collect();
    if !cites.is_empty() {
        out.push_str("Stories that treat it as canon:\n");
        for c in &cites {
            let items = group(&id, c);
            let when = items.first().map(|e| date(e))
                .or_else(|| evidence.iter().find(|e| s(e, "story") == *c).map(|e| date(e)));
            let when = when.map_or(String::new(), |d| format!(" ({d})"));
            if items.is_empty() {
                out.push_str(&format!("  - {c}{when}\n"));
            } else {
                out.push_str(&format!("  - {c}{when}: {}\n", story_line(&items)));
            }
            done.insert((id.clone(), c.clone()));
        }
    }
    // Every other (ending, story) pair, in the order the evidence lists them.
    let mut order: Vec<(String, String)> = Vec::new();
    for e in &evidence {
        let key = (s(e, "endingId"), s(e, "story"));
        if !shared(e) && !done.contains(&key) && !order.contains(&key) {
            order.push(key);
        }
    }
    if !order.is_empty() {
        out.push_str("Other stories linked to an ending (not relied on):\n");
        let mut endings: Vec<String> = Vec::new();
        for (e, _) in &order {
            if !endings.contains(e) {
                endings.push(e.clone());
            }
        }
        for ending in &endings {
            out.push_str(&format!("  {}:\n", name_of(ending)));
            for (_, story) in order.iter().filter(|(e, _)| e == ending) {
                let items = group(ending, story);
                out.push_str(&format!("    - {story} ({}): {}\n", date(items[0]), story_line(&items)));
            }
        }
    }
    let mut several: Vec<(&Value, String)> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for e in evidence.iter().filter(|e| shared(e)) {
        if seen.insert((s(e, "story"), s(e, "quote"))) {
            let names: Vec<String> = endings_of[&(s(e, "story"), s(e, "quote"))].iter().map(|x| name_of(x)).collect();
            several.push((e, names.join(", ")));
        }
    }
    if !several.is_empty() {
        out.push_str("Linked to several endings:\n");
        for (e, names) in &several {
            out.push_str(&format!("  - {} ({}), {names}: {}\n", s(e, "story"), date(e), story_line(&[*e])));
        }
    }
    out
}

/// One run's endings with the later stories that continue from each, and a strength label from the counts.
pub(super) fn canon_run(r: &Value, offset: usize) -> String {
    let endings: Vec<&Value> = r["endings"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    let stories = |e: &Value| -> BTreeSet<String> {
        e["referencedBy"].as_array().into_iter().flatten().map(|x| s(x, "story")).filter(|x| !x.is_empty()).collect()
    };
    // A story credited to several endings of this run splits its evidence between them.
    let mut credited: HashMap<String, Vec<String>> = HashMap::new();
    for e in &endings {
        for st in stories(e) {
            credited.entry(st).or_default().push(s(e, "name"));
        }
    }
    let counts: Vec<(String, usize)> = endings.iter().map(|e| (s(e, "name"), stories(e).len())).collect();
    let max = counts.iter().map(|c| c.1).max().unwrap_or(0);
    let leaders: Vec<&(String, usize)> = counts.iter().filter(|c| c.1 == max).collect();
    let no = s(r, "run").strip_prefix("rogue_").and_then(|d| d.parse::<usize>().ok()).map_or_else(|| s(r, "run"), |d| (d + offset).to_string());
    let mut out = format!("{} (Integrated Strategies #{no}):\n", s(r, "runName"));
    if max > 0 && leaders.len() == 1 {
        out.push_str(&format!("Most continued: {} ({} later {}).\n", leaders[0].0, max, if max == 1 { "story" } else { "stories" }));
    } else if max > 0 {
        out.push_str("No ending is favored: the most-referenced endings are tied.\n");
    } else {
        out.push_str("No ending is favored: no later story refers to any of its endings.\n");
    }
    for e in &endings {
        let st = stories(e);
        let strength = match st.len() {
            0 => "no later story refers to it".to_owned(),
            1 => "one later story".to_owned(),
            k => format!("continued by {k} later stories"),
        };
        out.push_str(&format!("- {}: {strength}", s(e, "name")));
        let split: Vec<String> = st.iter().filter_map(|x| credited.get(x).filter(|v| v.len() > 1)
            .map(|v| format!("{x} is credited to {} as well, so its evidence is split between them",
                             v.iter().filter(|n| **n != s(e, "name")).cloned().collect::<Vec<_>>().join(" and ")))).collect();
        if !split.is_empty() {
            out.push_str(&format!("; {}", split.join("; ")));
        }
        out.push('\n');
        for st_name in &st {
            if let Some(q) = e["quotes"].as_array().into_iter().flatten().find(|q| s(q, "story") == *st_name) {
                out.push_str(&format!("  - {st_name} ({}): \"{}\"\n", s(q, "released"), s(q, "quote")));
            }
        }
    }
    out
}
