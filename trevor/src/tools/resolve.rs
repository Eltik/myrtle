//! Resolving the names a route or a question gives: events by name, number or misspelling, characters
//! through the tables and the identity links.

use std::collections::{BTreeSet, HashSet};

use serde_json::Value;

use super::text::{contains_words, episode_number, fuzzy_unique, norm};
use super::{ToolError, Tools};

impl Tools {
    /// A main episode or event: a group id, "Episode 7" / "chapter 14" / "EP07" / "7", its name in any case,
    /// a text that contains one name (the longest wins), or a name within 1 edit (2 from 9 characters).
    ///
    /// # Errors
    /// [`ToolError::NotFound`] when nothing or more than one group matches.
    pub fn resolve_event(&self, name: &str) -> Result<&Value, ToolError> {
        let nf = || ToolError::NotFound { what: "event", name: name.to_owned() };
        if let Some(g) = self.groups.get(name.trim()) {
            return Ok(g);
        }
        let n = norm(name);
        if let Some(ch) = episode_number(&n) {
            return self.groups.values().find(|g| g["chapter"].as_u64() == Some(ch)).ok_or_else(nf);
        }
        // Operator records (story_*) are not events a player names.
        let events: Vec<(&Value, String)> = self.groups.values()
            .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
            .filter_map(|g| g["name"].as_str().map(|x| (g, norm(x)))).collect();
        let strip = |x: &str| x.strip_prefix("the ").unwrap_or(x).to_owned();
        if let Some((g, _)) = events.iter().find(|(_, gn)| *gn == n || strip(gn) == strip(&n)) {
            return Ok(g);
        }
        // Without a leading "the" and each word's trailing "s": "Masses Travel" is The Masses' Travels (the model
        // router wrote it so for Ian's question, 2026-09-30).
        let key = |x: &str| strip(x).split(' ').map(|w| if w.len() > 3 { w.trim_end_matches('s') } else { w }).collect::<Vec<_>>().join(" ");
        let nk = key(&n);
        let keyed: Vec<(&Value, String)> = events.iter().map(|(g, gn)| (*g, key(gn))).collect();
        if let Some((g, _)) = keyed.iter().find(|(_, k)| *k == nk) {
            return Ok(g);
        }
        // A longer text naming one event ("the Babel rerun").
        if let Some((g, _)) = events.iter().filter(|(_, gn)| gn.len() >= 4 && contains_words(&n, gn)).max_by_key(|(_, gn)| gn.len()) {
            return Ok(g);
        }
        // Part of one name ("Siracusano"), when only one event holds it.
        let part: Vec<&(&Value, String)> = events.iter().filter(|_| n.len() >= 5).filter(|(_, gn)| contains_words(gn, &n)).collect();
        if part.len() == 1 {
            return Ok(part[0].0);
        }
        fuzzy_unique(&n, events.iter().map(|(g, gn)| (*g, gn.as_str())))
            .or_else(|| fuzzy_unique(&nk, keyed.iter().map(|(g, k)| (*g, k.as_str())))).ok_or_else(nf)
    }

    /// A name among `cands`: exact, then case and punctuation folded, then through an identity link (Louisa is
    /// Kal'tsit), then within 1 edit (2 from 9 characters) when one candidate is closest.
    ///
    /// # Errors
    /// [`ToolError::NotFound`].
    pub fn resolve_name<'a>(&self, what: &'static str, name: &str, cands: &[&'a str]) -> Result<&'a str, ToolError> {
        let name = name.trim();
        if let Some(c) = cands.iter().find(|c| **c == name) {
            return Ok(c);
        }
        let n = norm(name);
        let by_norm = |x: &str| cands.iter().find(|c| norm(c) == x).copied();
        if let Some(c) = by_norm(&n) {
            return Ok(c);
        }
        for ident in self.identities.iter().filter(|i| i.iter().any(|x| norm(x) == n)) {
            if let Some(c) = ident.iter().find_map(|x| by_norm(&norm(x))) {
                return Ok(c);
            }
        }
        let normed: Vec<(&'a str, String)> = cands.iter().map(|c| (*c, norm(c))).collect();
        fuzzy_unique(&n, normed.iter().map(|(c, cn)| (*c, cn.as_str())))
            .ok_or_else(|| ToolError::NotFound { what, name: name.to_owned() })
    }

    /// Operator codenames of the attribute table.
    #[must_use]
    pub fn operator_names(&self) -> Vec<&str> {
        self.attributes.iter().flatten().filter_map(|r| r["name"].as_str()).filter(|n| !n.is_empty()).collect()
    }

    /// Whether the question names anything Trevor's tables know: an operator, a speaker, an event or episode, a topic
    /// (race, nation, concept, with its merged names), a place, or an identity-link name, as whole words after `norm`,
    /// three letters or more; or the word "arknights". `ask --lore-only` adds its off-topic sentence only when this is
    /// false (2026-10-03), so a lore question that names its subject keeps its prompt byte for byte.
    #[must_use]
    pub fn names_anything(&self, q: &str) -> bool {
        // Possessives too: `norm` drops the apostrophe, so "siege's" would read "sieges".
        let nq = norm(q);
        let np = norm(&q.replace("'s", " ").replace("\u{2019}s", " "));
        let hit = |n: &str| { let n = norm(n); n.chars().count() >= 3 && (contains_words(&nq, &n) || contains_words(&np, &n)) };
        if hit("arknights") || self.operator_names().into_iter().any(hit) || self.speaker_names().into_iter().any(hit)
            || self.places().iter().any(|p| hit(p)) || self.identities.iter().flatten().any(|n| hit(n)) {
            return true;
        }
        if self.groups.values().any(|g| g["name"].as_str().is_some_and(hit)) {
            return true;
        }
        self.topics.iter().flatten().any(|t| t["topic"].as_str().is_some_and(hit)
            || t["names"].as_array().into_iter().flatten().filter_map(Value::as_str).any(hit))
    }

    /// Short names in a question that are the start of exactly one operator or identity-link name ("kal" for
    /// Kal'tsit): (the word as written, the full name). Words that are a whole name, or shorter than 3 letters, are
    /// left alone; the caller also checks that the word is rare in the corpus (a real word like "can" is not a name).
    #[must_use]
    pub fn short_names(&self, q: &str) -> Vec<(String, String)> {
        let mut names: Vec<String> = self.operator_names().into_iter().map(str::to_owned).collect();
        names.extend(self.identities.iter().flatten().cloned());
        let first = |n: &str| norm(n).split(' ').next().unwrap_or_default().to_owned();
        let firsts: HashSet<String> = names.iter().map(|n| first(n)).collect();
        let mut out = Vec::new();
        for w in q.split(|c: char| !(c.is_alphanumeric() || matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`' | '\u{b4}'))).filter(|w| !w.is_empty()) {
            let f = norm(w);
            if f.chars().count() < 3 || firsts.contains(&f) || f.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }
            let hits: BTreeSet<&str> = names.iter().filter(|n| first(n).starts_with(&f)).map(String::as_str).collect();
            // One target, or several spellings of one person (an identity's names all start alike, as Kal'tsit's do).
            let targets: BTreeSet<String> = hits.iter().map(|n| first(n)).collect();
            if targets.len() == 1 {
                if let Some(full) = hits.iter().min_by_key(|n| n.len()) {
                    out.push((w.to_owned(), (*full).to_owned()));
                }
            }
        }
        out
    }

    /// Speakers with a first appearance.
    #[must_use]
    pub fn speaker_names(&self) -> Vec<&str> {
        self.first.as_ref().and_then(Value::as_object).map(|m| m.keys().map(String::as_str).collect()).unwrap_or_default()
    }

    // ---------------------------------------------------------------- listing
}
