//! The labelled theory (`--no-theory-rule`): a yes/no question about one named character whose answer declines,
//! while lines of its passages that name that character also touch what the question asks ("Is Fyodor infected?":
//! Fyodor asks "What of the Infected situation?", Witte speaks to him of "Talulah the Infected"), is asked once more
//! with those lines as HINTS and THEORY_RULE: the answer says "The story does not say outright, but", gives the hints
//! with their citations, and ends with a "Theory:" sentence labelled as Trevor's reading. The second answer is kept
//! only when it has that "Theory:" line; otherwise the decline stays. The first prompt is unchanged, so every answer
//! that does not decline is byte-identical with the rule off.

use trevor::search::runtime::Runtime;
use trevor::tools::{Tools, norm};

use crate::detect::declines;
use crate::kinds::{RECORD_KINDS, is_generated};

/// The most hint lines given.
pub(crate) const HINT_LINES: usize = 4;

/// Words of a question that say nothing about what it asks.
const STOP: &[&str] = &["is", "was", "are", "were", "does", "did", "do", "has", "have", "had", "can", "could", "will",
                        "would", "should", "the", "a", "an", "of", "in", "on", "at", "to", "for", "from", "with", "by",
                        "and", "or", "but", "not", "no", "yes", "ever", "really", "actually", "still", "also", "be",
                        "been", "being", "he", "she", "they", "it", "his", "her", "their", "its", "him", "them", "any",
                        "some", "that", "this", "there", "what", "who", "which", "when", "where", "why", "how", "know",
                        "known", "story", "stories", "lore", "canon", "arknights", "game", "true", "confirmed", "about",
                        "someone", "something", "anyone", "anything", "like", "kind", "sort", "type", "one", "become",
                        "became", "get", "got", "go", "went", "say", "said", "make", "made", "time", "now", "then", "part",
                        "member", "side", "really"];

/// Whether a question asks whether (its first word is an auxiliary verb).
pub(crate) fn asks_whether(q: &str) -> bool {
    let first = q.split_whitespace().next().unwrap_or_default().to_lowercase();
    matches!(first.as_str(), "is" | "was" | "are" | "were" | "does" | "did" | "do" | "has" | "have" | "had" | "can"
        | "could" | "will" | "would" | "should" | "isn't" | "wasn't" | "doesn't" | "didn't")
}

/// The stems of the question's content words: lowercase, not a stop word, not a word of the character's names, with
/// "ed", "ing", "es" or "s" cut while 4 letters or more remain ("infected" is "infect").
pub(crate) fn stems(q: &str, forms: &[String]) -> Vec<String> {
    let name_words: Vec<String> = forms.iter().flat_map(|f| norm(f).split(' ').map(str::to_owned).collect::<Vec<_>>())
        .collect();
    let mut out: Vec<String> = Vec::new();
    for w in norm(q).split(' ') {
        if w.chars().count() < 4 || STOP.contains(&w) || name_words.iter().any(|n| n == w)
            || w.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let mut s = w.to_owned();
        for suf in ["ing", "ed", "es", "s"] {
            if let Some(x) = s.strip_suffix(suf) {
                if x.chars().count() >= 4 {
                    s = x.to_owned();
                    break;
                }
            }
        }
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

fn speaker_and_text(line: &str) -> (Option<&str>, &str) {
    match line.find(": ") {
        Some(i) if i <= 40 && !line[..i].contains(['.', '!', '?', ',']) => (Some(line[..i].trim_matches('\'')), &line[i + 2..]),
        _ => (None, line),
    }
}

fn names(text: &str, forms: &[String]) -> bool {
    let t = norm(text);
    forms.iter().any(|f| { let k = norm(f); !k.is_empty() && format!(" {t} ").contains(&format!(" {k} ")) })
}

/// Whether `text` has a word starting with a stem, the character's own names left out ("Deepcolor" is not "deep").
/// Words that show the one state the stems cannot reach by spelling: Oripathy ("Is Fyodor infected?" is hinted by a
/// cough or an Originium crystal, not by the word "infected").
const RELATED: &[(&str, &[&str])] = &[("infect", &["oripath", "originium", "cough", "crystal"])];

fn touches(text: &str, stems: &[String], forms: &[String]) -> bool {
    let own: Vec<String> = forms.iter().flat_map(|f| norm(f).split(' ').map(str::to_owned).collect::<Vec<_>>())
        .collect();
    // A possessive is the name and "s" once `norm` drops the apostrophe ("Deepcolor's" is "deepcolors").
    let related: Vec<&str> = RELATED.iter().filter(|(k, _)| stems.iter().any(|s| s == k))
        .flat_map(|(_, ws)| ws.iter().copied()).collect();
    norm(text).split(' ').any(|w| !own.iter().any(|o| w.starts_with(o.as_str()))
        && (stems.iter().any(|s| w.starts_with(s.as_str())) || related.iter().any(|r| w.starts_with(r))))
}

/// The hint lines of the passages `texts` (each "(label)" then its lines): (passage number, line), a line that touches
/// a stem and is spoken by the character, names them, or replies to a line of theirs; at most `HINT_LINES`, in passage
/// order.
pub(crate) fn hints(texts: &[String], forms: &[String], stems: &[String]) -> Vec<(usize, String)> {
    let mut out: Vec<(usize, String)> = Vec::new();
    if stems.is_empty() {
        return out;
    }
    for (n, t) in texts.iter().enumerate() {
        let lines: Vec<&str> = t.lines().skip(1).collect();
        let parts: Vec<(Option<&str>, &str)> = lines.iter().map(|l| speaker_and_text(l)).collect();
        for (j, (_, text)) in parts.iter().enumerate() {
            if !touches(text, stems, forms) {
                continue;
            }
            let spoken_by = |k: usize| parts[k].0.is_some_and(|w| forms.iter().any(|f| norm(f) == norm(w)));
            // Their own line, a line naming them, or the reply to a line of theirs.
            if spoken_by(j) || names(text, forms) || (j > 0 && spoken_by(j - 1)) {
                let line = lines[j].trim();
                let line: String = if line.chars().count() > 300 {
                    format!("{}...", line.chars().take(300).collect::<String>())
                } else { line.to_owned() };
                if !out.iter().any(|(_, l)| *l == line) {
                    out.push((n + 1, line));
                }
            }
            if out.len() >= HINT_LINES {
                return out;
            }
        }
    }
    out
}

/// Words of a question about the game rather than the story ("Does Fyodor deserve a sprite?").
const META: &[&str] = &["deserve", "sprite", "translat", "skin", "banner", "rerun", "voice actor", "artist", "nerf",
                        "buff", "tier", "worth", "pull", "playable", "release", "upcoming", "model", "design",
                        "based on", "inspired", "reference"];

/// The words of a whether-question after its subject, when the subject is `who` right after the first word (and an
/// optional "the"): "Is Fyodor infected?" gives "infected"; None when `who` is not the subject or the question is about
/// the game rather than the story.
pub(crate) fn predicate(q: &str, who: &str) -> Option<String> {
    if !asks_whether(q) || !who.starts_with(char::is_uppercase) {
        return None;
    }
    let n = norm(q);
    if META.iter().any(|m| n.contains(m)) {
        return None;
    }
    let rest = n.split_once(' ')?.1;
    let rest = rest.strip_prefix("the ").unwrap_or(rest);
    let w = norm(who);
    let after = rest.strip_prefix(w.as_str())?;
    (after.is_empty() || after.starts_with(' ')).then(|| after.trim().to_owned())
}

/// What the second ask needs: the character, the forms of their name, the stems of what the question asks, and the
/// chunks the first ask already had (the extra hint scenes leave them out).
#[derive(Clone, Debug, Default)]
pub(crate) struct Spec {
    pub(crate) who: String,
    pub(crate) forms: Vec<String>,
    pub(crate) stems: Vec<String>,
    pub(crate) seen: Vec<String>,
    /// The chunks of the first ask whose lines hint, kept first in the second ask so a smaller retrieval budget
    /// cannot drop them.
    pub(crate) pinned: Vec<String>,
}

/// The spec for a declined answer to `q` about the one character `who`, or None when the rule does not apply: the
/// answer does not decline, the question does not ask whether `who` (its subject) is something, or its words leave no
/// stem.
pub(crate) fn spec(q: &str, answer: &str, tools: &Tools, who: &str, seen: &[String]) -> Option<Spec> {
    if !declined(answer) {
        return None;
    }
    let pred = predicate(q, who)?;
    let mut forms = vec![who.to_owned()];
    for g in tools.identities.iter().filter(|g| g.iter().any(|x| norm(x) == norm(who))) {
        forms.extend(g.iter().cloned());
    }
    let stems = stems(&pred, &forms);
    (!stems.is_empty()).then(|| Spec { who: who.to_owned(), forms, stems, seen: seen.to_vec(), pinned: Vec::new() })
}

/// The passage numbers (1-based) of `texts` that hold a hint line.
pub(crate) fn hinted(texts: &[String], t: &Spec) -> Vec<usize> {
    let mut out: Vec<usize> = hints(texts, &t.forms, &t.stems).into_iter().map(|(n, _)| n).collect();
    out.dedup();
    out
}

/// The tokens the HINTS block and THEORY_RULE add, taken from the retrieved passages' budget so the prompt still fits
/// a capped profile's slot.
pub(crate) const THEORY_TOKENS: u32 = 800;

/// The HINTS block over the passages `texts` as the prompt numbers them; None when no line hints.
pub(crate) fn block(texts: &[String], t: &Spec) -> Option<String> {
    let h = hints(texts, &t.forms, &t.stems);
    if h.is_empty() {
        return None;
    }
    let mut out = format!("\nHINTS (lines of the passages above that bear on the question about {} but do not \
        settle it)\n", t.who);
    for (n, line) in &h {
        out.push_str(&format!("[{n}] {line}\n"));
    }
    Some(out)
}

/// Lines of the character's own story scenes, outside the chunks the first ask had, that touch what the question
/// asks: (row, line index), at most `EXTRA_SCENES`, one per chunk, in corpus order. "Is Fyodor infected?" retrieved
/// Episode 8 and People but not Episode 17's "...*cough* *cough*!".
pub(crate) fn extra_hints(rt: &Runtime, t: &Spec) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (r, c) in rt.store.chunks.iter().enumerate() {
        if out.len() >= EXTRA_SCENES {
            break;
        }
        if is_generated(&c.group_id) || RECORD_KINDS.contains(&c.group_id.as_str()) || c.group_id == "voice"
            || t.seen.contains(&c.chunk_id) {
            continue;
        }
        let hit = c.text.lines().position(|l| {
            let (who, text) = speaker_and_text(l);
            who.is_some_and(|w| t.forms.iter().any(|f| norm(f) == norm(w))) && touches(text, &t.stems, &t.forms)
        });
        if let Some(i) = hit {
            out.push((r, i));
        }
    }
    out
}

/// The most extra hint scenes the second ask adds.
pub(crate) const EXTRA_SCENES: usize = 3;

/// Whether an answer declines, by `detect::declines` or by the plural and hedged forms it misses ("The provided passages
/// do not state whether Fyodor is infected [1-7]." declined the Fyodor question on E2B and passed `declines`).
pub(crate) fn declined(answer: &str) -> bool {
    let first = answer.trim().split(['.', '!', '?']).next().unwrap_or_default().to_lowercase();
    declines(answer) || ["do not state", "do not say", "do not mention", "do not indicate", "does not indicate",
        "do not specify", "does not specify", "not explicitly", "don't say", "doesn't say", "do not explicitly",
        "does not explicitly", "no mention", "is not mentioned", "are not mentioned", "is not stated"]
        .iter().any(|w| first.contains(w))
}

/// Whether a second answer kept the labelled form (a "Theory:" sentence).
pub(crate) fn has_theory(answer: &str) -> bool {
    answer.contains("Theory:")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts() -> Vec<String> {
        vec![
            "(story: Roaring Flare, x)\nFyodor: I hope it all proceeds as you've said. What of the Infected \
             situation? Any developments?\nIslam Witte: Your Majesty is overhasty.".to_owned(),
            "(story: Stultifera Navis, y)\nWitte: Talulah the Infected... Should you break the curse, Fyodor.\n\
             Amiya: The infected suffer.".to_owned(),
        ]
    }

    #[test]
    fn a_declined_whether_question_gets_the_hint_lines() {
        let t = Tools::default();
        assert_eq!(stems("Is Fyodor infected?", &["Fyodor".to_owned()]), vec!["infect".to_owned()]);
        let hb = |q: &str, a: &str, x: &[String], who: &str| spec(q, a, &t, who, &[]).and_then(|s| block(x, &s));
        let b = hb("Is Fyodor infected?", "I could not find it in the story text.", &texts(), "Fyodor").unwrap();
        assert!(b.contains("[1] Fyodor: I hope it all proceeds"), "{b}");
        assert!(b.contains("[2] Witte: Talulah the Infected"), "{b}");
        assert!(!b.contains("Amiya"), "{b}");
        // An answer that does not decline, a question that does not ask whether, no line that hints.
        assert!(hb("Is Fyodor infected?", "Yes, he is [1].", &texts(), "Fyodor").is_none());
        assert!(hb("Why is Fyodor infected?", "I could not find it.", &texts(), "Fyodor").is_none());
        assert!(hb("Is Fyodor tall?", "I could not find it.", &texts(), "Fyodor").is_none());
        let cough = vec!["(story: x)\nFyodor: Absurd... Ridiculous! ...*cough* *cough*!".to_owned()];
        assert!(hb("Is Fyodor infected?", "I could not find it.", &cough, "Fyodor").is_some());
        let deep = vec!["(story: x)\nDeacon: Deepcolor steps into the ocean.".to_owned()];
        assert!(hb("Is Deepcolor part of the deep?", "I could not find it.", &deep, "Deepcolor").is_none());
        assert!(has_theory("The story does not say outright, but ... Theory: he may be."));
        assert!(declined("The provided passages do not state whether Fyodor is infected [1-7]."));
        assert!(!declined("Fyodor is the Emperor [1]. The text does not state his age."));
        // Not the subject, or a question about the game.
        assert_eq!(predicate("Is Fyodor infected?", "Fyodor").as_deref(), Some("infected"));
        assert_eq!(predicate("Does anyone know what crossbow Faust uses?", "Faust"), None);
        assert_eq!(predicate("Does Fyodor deserve a sprite?", "Fyodor"), None);
        assert_eq!(predicate("Is there a lore behind Ascalon's tattoo?", "Ascalon"), None);
    }
}
