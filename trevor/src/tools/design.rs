//! Design-basis questions ("what animal is X based on"): the inferred design table.

use serde_json::Value;

use super::text::{contains_words, norm};
use super::{Tools, s};

/// The design-question detector of the `design_basis` tool: the normalized question and its content words
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

impl Tools {
    /// A design-inspiration answer, or `None` when the question is not one. It fires on a
    /// question with a design word ("inspired by", "inspiration", "motif", "modeled after", "design inspiration", or
    /// "based on/off" at the end, before "a"/"an"/"real", or with "animal" in the question) and reads it both ways: each
    /// content word of the question (4 letters or more, not a question or design word) is matched as a whole word against
    /// the inferred subjects of `design_infer.jsonl` (operators whose design draws on it) and against operator names (what
    /// their design draws on, with their game alters); an operator the question names with nothing inferred gets a line
    /// saying the game data does not show a subject. Labelled as Trevor's inference from the game data, never as a fact.
    #[must_use]
    pub fn design_basis(&self, q: &str) -> Option<String> {
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
}
