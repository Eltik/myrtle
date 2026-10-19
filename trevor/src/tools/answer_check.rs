//! The answer check of who/which questions: the category a question asks its answer to belong to, and
//! the answer names that fall outside it.

use std::collections::BTreeSet;

use super::text::{contains_words, norm, whole_word_matches};
use super::{AnswerConstraint, Tools, s};

impl Tools {
    /// The category a who/which question asks its answer to belong to (`ask --answer-check`), read from the
    /// question's words alone: not playable ("not playable", "non-playable", "unplayable", "NPC"), playable, or from a
    /// nation of the attribute table ("from Victoria", "Victoria operator").
    #[must_use]
    pub fn answer_constraint(&self, q: &str) -> Option<AnswerConstraint> {
        let n = norm(q);
        // "NPC" alone constrains only when the question does not offer an operator as the alternative: "Which
        // operator/Rhodes Island NPC comes closest to being a Psychologist?" accepts both.
        let npc = ["npc", "npcs"].iter().any(|w| contains_words(&n, w)) && !contains_words(&n, "operator");
        if npc
            || [
                "not playable",
                "non playable",
                "nonplayable",
                "unplayable",
                "not a playable",
            ]
            .iter()
            .any(|w| contains_words(&n, w))
        {
            return Some(AnswerConstraint::NonPlayable);
        }
        if contains_words(&n, "playable") {
            return Some(AnswerConstraint::Playable);
        }
        // A nation value that is an organization topic is an employer, not a homeland: "Which Rhodes Island operator
        // won the landship's annual chess tournament?" (gold u002) names the company every operator works for.
        let orgs: BTreeSet<String> = self
            .topics
            .iter()
            .flatten()
            .filter(|r| s(r, "kind") == "organization")
            .map(|r| norm(&s(r, "topic")))
            .collect();
        let nations: BTreeSet<String> = self
            .attributes
            .iter()
            .flatten()
            .filter_map(|r| r["nation"].as_str())
            .filter(|x| !orgs.contains(&norm(x)))
            .map(str::to_owned)
            .collect();
        nations
            .into_iter()
            .filter(|x| x.chars().count() >= 3)
            .find(|x| {
                let nx = norm(x);
                contains_words(&n, &format!("from {nx}"))
                    || contains_words(&n, &format!("{nx} operator"))
                    || contains_words(&n, &format!("{nx} operators"))
            })
            .map(AnswerConstraint::Nation)
    }

    /// The playable operator a character name belongs to: an operator name, an operator's real name, or a name one
    /// identity link away from either.
    pub(super) fn playable_of(&self, name: &str) -> Option<String> {
        let n = norm(name);
        let ops = self.operator_names();
        let direct = |x: &str| -> Option<String> {
            let nx = norm(x);
            ops.iter().find(|o| norm(o) == nx).map(|o| (*o).to_owned()).or_else(|| {
                self.real_names
                    .iter()
                    .flatten()
                    .find(|r| norm(&s(r, "realName")) == nx)
                    .map(|r| s(r, "name"))
            })
        };
        direct(name).or_else(|| {
            self.identity_links
                .iter()
                .filter(|l| norm(&l.0) == n)
                .find_map(|l| direct(&l.1))
        })
    }

    /// Whether a question asks for operators as its answer ("Give a ranking of operators who break the most Geneva
    /// convention rules"): it says "operators", or "operator" in a who/which question, and not that the answer is not
    /// playable (that is `answer_constraint`'s NonPlayable), and not when it offers NPCs or characters as the
    /// alternative ("Which operator/Rhodes Island NPC comes closest to being a Psychologist?").
    #[must_use]
    pub fn asks_operators(&self, q: &str) -> bool {
        let n = norm(q);
        if ["npc", "npcs", "character", "characters"]
            .iter()
            .any(|w| contains_words(&n, w))
        {
            return false;
        }
        let who = ["who", "whom", "whose", "which", "rank", "ranking", "list"]
            .iter()
            .any(|w| contains_words(&n, w));
        (contains_words(&n, "operators") || (contains_words(&n, "operator") && who))
            && !matches!(self.answer_constraint(q), Some(AnswerConstraint::NonPlayable))
    }

    /// The names an answer lists as its items ("*   **Aurela:** ...", "1. **Texas** ..."), each with the 1-based passage
    /// numbers its item cites, that are not playable operators by the game data (`playable_of`) and not a topic's name or
    /// alias (an organization or nation item is not a claim that it is an operator). An item name is the bold text at
    /// the start of a list line, a parenthesis and a trailing colon dropped, of one to three capitalized words.
    #[must_use]
    pub fn non_operator_items(&self, answer: &str) -> Vec<(String, Vec<usize>)> {
        let topic_forms: BTreeSet<String> = self
            .topics
            .iter()
            .flatten()
            .flat_map(|r| {
                std::iter::once(s(r, "topic")).chain(
                    r["aliases"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|a| a.as_str().map(str::to_owned)),
                )
            })
            .map(|f| norm(&f))
            .collect();
        let mut out: Vec<(String, Vec<usize>)> = Vec::new();
        for line in answer.lines() {
            let t = line.trim_start();
            let t = t.strip_prefix(['*', '-']).map(str::trim_start).or_else(|| {
                t.split_once(". ")
                    .filter(|(d, _)| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit()))
                    .map(|(_, r)| r.trim_start())
            });
            let Some(rest) = t.and_then(|t| t.strip_prefix("**")) else {
                continue;
            };
            let Some((name, after)) = rest.split_once("**") else {
                continue;
            };
            // The item says so itself ("**Aurela**: Although not a playable operator, she ..."): no claim to correct.
            let low = after.to_lowercase();
            if [
                "not a playable operator",
                "not an operator",
                "not playable",
                "not an rhodes island operator",
            ]
            .iter()
            .any(|w| low.contains(w))
            {
                continue;
            }
            let name = name
                .split(" (")
                .next()
                .unwrap_or_default()
                .trim()
                .trim_end_matches(':')
                .trim();
            let words: Vec<&str> = name.split_whitespace().collect();
            if words.is_empty()
                || words.len() > 3
                || !words.iter().all(|w| w.chars().next().is_some_and(char::is_uppercase))
            {
                continue;
            }
            if self.playable_of(name).is_some() || topic_forms.contains(&norm(name)) || out.iter().any(|o| o.0 == name)
            {
                continue;
            }
            let cites: Vec<usize> = after
                .split('[')
                .skip(1)
                .filter_map(|x| x.split(']').next())
                .flat_map(|x| {
                    x.split(',')
                        .filter_map(|n| n.trim().parse::<usize>().ok())
                        .collect::<Vec<_>>()
                })
                .collect();
            out.push((name.to_owned(), cites));
        }
        out
    }

    /// The first character name the first sentence of an answer gives (operator names, real names and identity-link names, whole words,
    /// longest first, not a name the question itself uses) and, when that character fails the question's category by
    /// the game data, the reason; `None` when the first name passes, the answer names no known character, or the
    /// category cannot be checked (a character with no operator file has no nation in the data).
    #[must_use]
    pub fn failing_answer_name(&self, q: &str, answer: &str, c: &AnswerConstraint) -> Option<(String, String)> {
        let nq = norm(q);
        let mut names: Vec<String> = self
            .operator_names()
            .into_iter()
            .map(str::to_owned)
            .chain(self.real_names.iter().flatten().map(|r| s(r, "realName")))
            .chain(self.identity_links.iter().flat_map(|l| [l.0.clone(), l.1.clone()]))
            .filter(|x| {
                x.chars().count() >= 3
                    && x.chars().next().is_some_and(char::is_uppercase)
                    && !contains_words(&nq, &norm(x))
            })
            .collect();
        names.sort_by_key(|x| std::cmp::Reverse(x.len()));
        names.dedup();
        // The first sentence names the answer; a later one may name a character only as context ("the text mentions
        // that Iris met with the elite operator Rosmontis", in a decline).
        let text = answer.trim_start().split(['\n']).next().unwrap_or_default();
        let text = text.find(". ").map_or(text, |i| &text[..=i]);
        let at = |f: &str| whole_word_matches(text, f).next();
        // Earliest position wins; at one position the longest name ("Sakiko Togawa" over "Sakiko").
        let first = names
            .iter()
            .filter_map(|x| at(x).map(|i| (i, std::cmp::Reverse(x.len()), x)))
            .min()?
            .2
            .clone();
        let op = self.playable_of(&first);
        let reason = match (c, &op) {
            (AnswerConstraint::NonPlayable, Some(o)) if norm(o) == norm(&first) => {
                format!("{first} is a playable operator")
            }
            (AnswerConstraint::NonPlayable, Some(o)) => format!("{first} is the playable operator {o}"),
            (AnswerConstraint::Playable, None) => {
                format!("{first} has no operator file, so is not a playable operator")
            }
            (AnswerConstraint::Nation(x), Some(o)) => {
                let r = self.attributes.iter().flatten().find(|r| s(r, "name") == *o)?;
                let (nat, birth) = (s(r, "nation"), s(r, "birthplace"));
                if norm(&nat) == norm(x) || norm(&birth) == norm(x) {
                    return None;
                }
                format!(
                    "{first}'s operator file gives the nation {} and the birthplace {}",
                    if nat.is_empty() { "none" } else { &nat },
                    if birth.is_empty() { "none" } else { &birth }
                )
            }
            _ => return None,
        };
        Some((first, reason))
    }
}
