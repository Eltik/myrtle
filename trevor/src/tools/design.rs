//! Design-basis questions ("what animal is X based on"): the inferred design table, the evidence
//! shortlist of `ask --design-deduce` and the wiki table of `--wiki-legacy`.

use std::collections::{BTreeSet, HashMap};

use serde_json::Value;

use super::text::{contains_words, norm};
use super::{Tools, s};

/// The design-question detector of the `design_basis` tool (2026-10-04): the normalized question and its content words
/// (4 letters or more, not a question or design word), or `None` when it is not a design question. "based on/off" counts
/// only at the end ("what is Lucilla based on"), before "a", "an", "any" or "real", or with "animal" or "creature" in the
/// question; "based on his inspection" and "Based on the conversation between ..." (2 gold questions) are not.
pub(super) fn design_question(q: &str) -> Option<(String, Vec<String>)> {
    const CUES: &[&str] = &["inspired by", "inspiration", "inspirations", "motif", "motifs", "modeled after", "modelled after",
        "modeled on", "modelled on", "designed after", "design inspiration", "character design", "real life counterpart"];
    const STOP: &[&str] = &["what", "which", "whose", "where", "when", "operator", "operators", "animal", "animals", "based",
        "inspired", "inspiration", "inspirations", "motif", "motifs", "modeled", "modelled", "design", "designs", "designed",
        "name", "names", "named", "part", "with", "have", "their", "there", "they", "this", "that", "from", "about", "into",
        "upon", "does", "were", "would", "could", "should", "character", "characters", "arknights", "real", "world", "life",
        "creature", "creatures", "thing", "things", "kind", "type", "like", "looks", "also", "some", "other", "only", "most",
        "more", "many", "much", "know", "tell", "anyone", "someone", "something", "whom", "being", "been", "game", "games",
        "story", "lore", "skin", "outfit", "outfits", "plant", "object", "person", "people", "mythology", "myth"];
    let n = norm(q);
    let words: Vec<&str> = n.split(' ').collect();
    let based = words.windows(2).enumerate().any(|(i, w)| w[0] == "based" && matches!(w[1], "on" | "off" | "upon")
        && words.get(i + 2).is_none_or(|x| matches!(*x, "a" | "an" | "any" | "real")))
        || (contains_words(&n, "based") && ["animal", "animals", "creature", "creatures"].iter().any(|w| contains_words(&n, w)));
    if !based && !CUES.iter().any(|c| contains_words(&n, c)) {
        return None;
    }
    let keys = n.split(' ').filter(|w| w.chars().count() >= 4 && !STOP.contains(w)).map(str::to_owned).collect();
    Some((n, keys))
}

/// The words the design evidence is matched on: words of 4 letters or more after `norm`, a plural "s" folded.
fn design_words(t: &str) -> BTreeSet<String> {
    norm(t).split(' ').filter(|w| w.chars().count() >= 4)
        .map(|w| if w.len() > 4 && w.ends_with('s') && !w.ends_with("ss") { w[..w.len() - 1].to_owned() } else { w.to_owned() })
        .collect()
}

impl Tools {
    /// A design-inspiration answer (2026-10-04, Ian's ian11), or `None` when the question is not one. It fires on a
    /// question with a design word ("inspired by", "inspiration", "motif", "modeled after", "design inspiration", or
    /// "based on/off" at the end, before "a"/"an"/"real", or with "animal" in the question) and reads it both ways: each
    /// content word of the question (4 letters or more, not a question or design word) is matched as a whole word against
    /// the inferred subjects of `design_infer.jsonl` (operators whose design draws on it) and against operator names (what
    /// their design draws on, with their game alters); an operator the question names with nothing inferred gets a line
    /// saying the game data does not show a subject. Labelled as Trevor's inference from the game data, never as a fact.
    /// With `opts.wiki_legacy` it is the wiki-trivia answer of the morning of 2026-10-04 (measurement only).
    #[must_use]
    pub fn design_basis(&self, q: &str) -> Option<String> {
        if self.opts.wiki_legacy {
            return self.design_basis_wiki(q);
        }
        let rows = self.design_basis.as_ref()?;
        let (n, keys) = design_question(q)?;
        let attrs = self.attributes.as_deref().unwrap_or_default();
        let subj_words = |sub: &Value| norm(sub["subject"].as_str().unwrap_or_default());
        let line = |r: &Value, sub: &Value| {
            let ev = sub["evidence"].as_array().into_iter().flatten().take(3)
                .map(|e| format!("{} ({}): \"{}\"", s(e, "id"), s(e, "source"), s(e, "text"))).collect::<Vec<_>>().join("; ");
            format!("- {}: likely draws on {} ({}, {} confidence). Clues: {} Evidence: {ev}", s(r, "operator"), s(sub, "subject"),
                    s(sub, "category"), s(sub, "confidence"), s(sub, "why"))
        };
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut parts: Vec<String> = Vec::new();
        for k in &keys {
            let named: Vec<String> = attrs.iter().map(|a| s(a, "name")).filter(|nm| contains_words(&norm(nm), k)).collect();
            let mut t = String::new();
            if about_char {
                for r in rows.iter().filter(|r| !named.contains(&s(r, "operator"))) {
                    for sub in r["subjects"].as_array().into_iter().flatten().filter(|sub| contains_words(&subj_words(sub), k)) {
                        t.push_str(&line(r, sub));
                        t.push('\n');
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators whose design Trevor infers draws on something named \"{k}\":\n{t}"));
            } else if about_char {
                parts.push(format!("No operator's inferred design names \"{k}\" (the game data may not show it).\n"));
            }
            let mut t = String::new();
            for nm in &named {
                let own: Vec<&Value> = rows.iter().filter(|r| s(r, "operator") == *nm).collect();
                let subs = |r: &Value| r["subjects"].as_array().cloned().unwrap_or_default();
                for r in &own {
                    if subs(r).is_empty() {
                        t.push_str(&format!("- {nm}: the game data does not point to a specific real-world subject.\n"));
                    }
                    for sub in &subs(r) {
                        t.push_str(&line(r, sub));
                        t.push('\n');
                    }
                }
                let alters: Vec<String> = own.first().map(|r| r["alters"].as_array().into_iter().flatten()
                    .filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
                for al in &alters {
                    for r in rows.iter().filter(|r| s(r, "operator") == *al) {
                        for sub in &subs(r) {
                            t.push_str(&format!("{} (the alternate operator of {nm})\n", line(r, sub)));
                        }
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators with \"{k}\" in their name, and what Trevor infers their design draws on:\n{t}"));
            }
        }
        // Only "no operator names it" lines: nothing to answer from, so the question goes on to the router as before.
        if parts.iter().all(|p| p.starts_with("No operator's")) {
            return None;
        }
        Some(format!("Trevor's inference from the game data, not stated by the game: the game never says what a design is \
based on, so these are deduced from each operator's file, skills, modules, name and the model-written descriptions of their \
art, and each was checked against the evidence it cites. Say they are inferences.\n\n{}", parts.join("\n")))
    }

    /// For `ask --design-deduce` (2026-10-05): `None` when the question is not a design question (the `design_basis`
    /// detector); else whether it asks which operator or character (the forward reading: "which operator is inspired by
    /// a jellyfish") and the evidence rows of the operators it names (the reverse reading: "what animal is Kirara based
    /// on?"). A question that asks for an operator is forward even when a word of it is an operator's name (ian11's
    /// "phantom").
    #[must_use]
    pub fn design_deduce_question(&self, q: &str) -> Option<(bool, Vec<&Value>)> {
        let rows = self.design_evidence.as_ref()?;
        let (n, _) = design_question(q)?;
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut named: Vec<&Value> = rows.iter().filter(|r| { let nm = norm(&s(r, "operator")); nm.len() >= 3 && contains_words(&n, &nm) }).collect();
        // "Texas the Omertosa" names Texas too: keep the longest names only
        let longest: Vec<String> = named.iter().map(|r| norm(&s(r, "operator"))).collect();
        named.retain(|r| { let nm = norm(&s(r, "operator")); !longest.iter().any(|o| o.len() > nm.len() && contains_words(o, &nm)) });
        Some((about_char, named))
    }

    /// Whether `phrase` is grounded in an evidence row (`--design-deduce`'s check of the matching step): it shares a word
    /// of 4 letters or more (plural folded) with the row's text that is not one of the generic words of the shortlist
    /// (in more than a quarter of the rows: "blue", "large", "body"), so a feature the model invents does not count.
    #[must_use]
    pub fn design_grounded(&self, row: &Value, phrase: &str) -> bool {
        let Some(rows) = self.design_evidence.as_ref() else { return false };
        let words = design_words;
        let have = words(&s(row, "text"));
        words(phrase).iter().filter(|w| have.contains(*w))
            .any(|w| rows.iter().filter(|r| words(&s(r, "text")).contains(w)).count() * 4 <= rows.len())
    }

    /// The `k` operators whose design evidence best matches `terms` (each a phrase with a weight: candidate names and
    /// groups 2, features 1): a phrase scores its weight times the inverse document frequency (over the evidence rows)
    /// of its rarest word the row contains, so one phrase counts once and a long row of generic words ("large black and
    /// white body") does not outscore a row naming the animal ("bear"). Words of 4 letters or more, a plural "s"
    /// folded; a word in more than a quarter of the rows never counts.
    #[must_use]
    pub fn design_shortlist(&self, terms: &[(String, f64)], k: usize) -> Vec<&Value> {
        let Some(rows) = self.design_evidence.as_ref() else { return Vec::new() };
        let words = design_words;
        let docs: Vec<BTreeSet<String>> = rows.iter().map(|r| words(&s(r, "text"))).collect();
        let mut df: HashMap<&str, usize> = HashMap::new();
        for d in &docs {
            for w in d {
                *df.entry(w.as_str()).or_default() += 1;
            }
        }
        let n = docs.len() as f64;
        let idf = |w: &str| { let f = *df.get(w).unwrap_or(&0) as f64; if f > n / 4.0 { 0.0 } else { (n / (1.0 + f)).ln().max(0.0) } };
        let phrases: Vec<(BTreeSet<String>, f64)> = terms.iter().map(|(t, wt)| (words(t), *wt)).collect();
        let mut scored: Vec<(f64, usize)> = docs.iter().enumerate().map(|(i, d)| {
            let sc: f64 = phrases.iter().map(|(ws, wt)| wt * ws.iter().filter(|w| d.contains(w.as_str())).map(|w| idf(w)).fold(0.0, f64::max)).sum();
            (sc, i)
        }).filter(|(sc, _)| *sc > 0.0).collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.into_iter().take(k).map(|(_, i)| &rows[i]).collect()
    }

    /// The wiki-trivia design answer of the morning of 2026-10-04 (`ask --wiki-legacy`, measurement only), unchanged:
    /// reads `design_basis_wiki`, loaded from `crate::reference` only under that flag. It detects a design question as
    /// [`design_question`] does.
    ///
    /// A design-inspiration answer (2026-10-04, Ian's ian11), or `None` when the question is not one. It fires on a
    /// question with a design word ("inspired by", "inspiration", "motif", "modeled after", "design inspiration", or
    /// "based on/off" at the end, before "a"/"an"/"real", or with "animal" in the question) and reads it both ways: each content word of the question (4 letters or more, not a question or design word) is
    /// matched as a whole word against the basis subjects of the wiki rows (operators based on it) and against
    /// operator names (what they are based on, with their game alters), and every reading that has rows is given, always
    /// labelled as the wiki's trivia, not the game text.
    #[must_use]
    pub(super) fn design_basis_wiki(&self, q: &str) -> Option<String> {
        let rows = self.design_basis_wiki.as_ref()?;
        let (n, keys) = design_question(q)?;
        let attrs = self.attributes.as_deref().unwrap_or_default();
        let line = |r: &Value| format!("- {}: based on or referencing {}. Wiki: \"{}\" ({})", s(r, "operator"),
            r["basis"].as_array().into_iter().flatten().filter_map(Value::as_str).collect::<Vec<_>>().join(", "), s(r, "sentence"), s(r, "url"));
        // The subject reading needs a question about a character ("Could Babel be inspired by the Tower of Babel?" asks
        // about an organization; 1 of 1,228 Reddit and Discord questions).
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut parts: Vec<String> = Vec::new();
        for k in &keys {
            let named: Vec<String> = attrs.iter().map(|a| s(a, "name")).filter(|nm| contains_words(&norm(nm), k)).collect();
            // An operator the name reading lists is not repeated under the subject reading.
            let by_subject: Vec<&Value> = rows.iter().filter(|_| about_char).filter(|r| !named.contains(&s(r, "operator")))
                .filter(|r| r["basis"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|b| contains_words(&norm(b), k))).collect();
            if !by_subject.is_empty() {
                let mut t = format!("Operators based on something named \"{k}\":\n");
                for r in by_subject {
                    t.push_str(&line(r));
                    t.push('\n');
                }
                parts.push(t);
            }
            let mut t = String::new();
            for nm in &named {
                let own: Vec<&Value> = rows.iter().filter(|r| s(r, "operator") == *nm).collect();
                let alters: Vec<String> = own.first().map(|r| r["alters"].as_array().into_iter().flatten()
                    .filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
                for r in &own {
                    t.push_str(&line(r));
                    t.push('\n');
                }
                for al in &alters {
                    for r in rows.iter().filter(|r| s(r, "operator") == *al) {
                        t.push_str(&format!("{} (the alternate operator of {nm})\n", line(r)));
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators with \"{k}\" in their name, and what the wiki says they are based on:\n{t}"));
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some(format!("From the Arknights wiki's trivia pages (arknights.wiki.gg), not the game text: design inspirations are \
not stated in the story, so this is fan-written trivia, quoted as written.\n\n{}", parts.join("\n")))
    }
}
