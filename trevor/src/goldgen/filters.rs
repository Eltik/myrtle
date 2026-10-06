//! Deterministic candidate filters (gold-set generation spec, section 5).
//! No model: each is a pure function, so a filtered set can be rebuilt bit
//! for bit from the generated candidates.

use std::collections::HashSet;

/// Collapse whitespace and fold typographic quotes and dashes, so a quote the
/// model copied with straight apostrophes still matches a script that uses
/// curly ones. Case is kept: a verbatim quote should match verbatim.
#[must_use]
pub fn normalize_for_match(s: &str) -> String {
    let folded: String = s
        .chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{02BC}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2013}' | '\u{2014}' | '\u{2015}' => '-',
            '\u{2026}' => '.',
            c => c,
        })
        .collect();
    folded.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Is `quote` a substring of `passage` after normalization? The free
/// grounding check: it catches evidence the model invented.
#[must_use]
pub fn quote_in_passage(quote: &str, passage: &str) -> bool {
    let q = normalize_for_match(quote);
    // A trailing ellipsis or period the model added is not evidence of
    // fabrication; strip trailing punctuation before matching.
    let q = q.trim_end_matches(['.', ',', '!', '?', ' ']);
    !q.is_empty() && normalize_for_match(passage).contains(q)
}

/// The longest prefix of `quote` that occurs verbatim in `passage` (both
/// normalized as for [`quote_in_passage`]), cut back to the last whole word
/// and stripped of trailing punctuation. In gold set v1 the generator often
/// quoted a real stretch of the passage and then ran on into the next
/// speaker's line without its label, or glued fragments with "..."; 58 of the
/// 101 failed quotes have a verbatim prefix of 40 characters or more.
#[must_use]
pub fn verbatim_prefix(quote: &str, passage: &str) -> String {
    let q = normalize_for_match(quote);
    let p = normalize_for_match(passage);
    let bounds: Vec<usize> = q
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(q.len()))
        .collect();
    // Substring containment is monotone in prefix length: binary search.
    let (mut lo, mut hi) = (0, bounds.len() - 1);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if p.contains(&q[..bounds[mid]]) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    let mut prefix = &q[..bounds[lo]];
    if bounds[lo] < q.len() {
        // The match ended mid-quote: drop a trailing partial word.
        let next_is_word = q[bounds[lo]..].starts_with(|c: char| c.is_alphanumeric());
        let last_is_word = prefix.ends_with(|c: char| c.is_alphanumeric());
        if next_is_word && last_is_word {
            prefix = prefix.rfind(' ').map_or("", |i| &prefix[..i]);
        }
    }
    prefix
        .trim_end_matches(|c: char| c.is_whitespace() || (c.is_ascii_punctuation() && c != '\''))
        .to_owned()
}

/// The spec's trigram definition, so the threshold is reproducible:
/// lowercase, keep `[a-z0-9 ]`, collapse whitespace, character 3-grams,
/// Jaccard.
#[must_use]
pub fn trigram_jaccard(a: &str, b: &str) -> f64 {
    fn grams(s: &str) -> HashSet<[u8; 3]> {
        let cleaned: String = s
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() || c.is_ascii_digit() {
                    c
                } else {
                    ' '
                }
            })
            .collect();
        let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
        joined
            .as_bytes()
            .windows(3)
            .map(|w| [w[0], w[1], w[2]])
            .collect()
    }
    let (x, y) = (grams(a), grams(b));
    let union = x.union(&y).count();
    if union == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let j = x.intersection(&y).count() as f64 / union as f64;
    j
}

/// Deictic references that make a question unanswerable without the passage.
#[must_use]
pub fn deictic(question: &str) -> Option<&'static str> {
    let q = format!(" {} ", question.to_lowercase());
    const PHRASES: [&str; 14] = [
        " this passage",
        " the passage",
        " the above",
        " this scene",
        " the scene ",
        " the speaker",
        " this excerpt",
        " the excerpt",
        " this text",
        " the text ",
        " this dialogue",
        " the dialogue",
        " this conversation",
        " here?",
    ];
    for p in PHRASES {
        if q.contains(p) {
            return Some(p.trim());
        }
    }
    // A personal pronoun with no proper noun anywhere after the first word
    // has nothing to refer to. With a name present the reference may still
    // be unclear; that is the judge's call, not a string match's.
    let words: Vec<&str> = question.split_whitespace().collect();
    let bare = |w: &str| w.trim_matches(|c: char| !c.is_alphanumeric()).to_owned();
    let pronoun = words.iter().any(|w| {
        matches!(
            bare(w).to_lowercase().as_str(),
            "he" | "she" | "him" | "her" | "his" | "hers" | "they" | "them" | "their"
        )
    });
    let named = words
        .iter()
        .skip(1)
        .any(|w| bare(w).starts_with(char::is_uppercase));
    if pronoun && !named {
        return Some("pronoun without a named antecedent");
    }
    None
}

/// FNV-1a, for a per-item seed derived from its id, so a rerun reproduces
/// the same generations.
#[must_use]
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbatim_prefix_cuts_run_on_quotes() {
        let passage = "Hoshiguma: They're my brothers. Always have been.\nL.G.D. Officer: Huh? So you...";
        // Runs on into the next line without its speaker label.
        let quote = "They\u{2019}re my brothers. Always have been. Huh? So you";
        assert_eq!(verbatim_prefix(quote, passage), "They're my brothers. Always have been");
        // A partial word at the break is dropped.
        assert_eq!(verbatim_prefix("my brothers. Alwaysxx", passage), "my brothers");
        // A whole verbatim quote comes back whole, minus trailing punctuation.
        assert_eq!(verbatim_prefix("Always have been.", passage), "Always have been");
        // Digit run-off after a real stretch.
        assert_eq!(verbatim_prefix("Huh? So you...8294857721", passage), "Huh? So you");
        assert_eq!(verbatim_prefix("nothing here", passage), "");
    }

    #[test]
    fn quotes_survive_typography_and_whitespace_but_not_invention() {
        let passage = "Kal’tsit: You aren’t ready to hear it,\n and I am not ready — to say it.";
        assert!(quote_in_passage(
            "You aren't ready to hear it, and I am not ready - to say it.",
            passage
        ));
        assert!(!quote_in_passage("You are not ready to hear it", passage));
        assert!(!quote_in_passage("...", passage));
    }

    #[test]
    fn trigram_jaccard_matches_hand_computation() {
        // "abcd" -> {abc, bcd}; "abce" -> {abc, bce}: 1 shared of 3.
        assert!((trigram_jaccard("abcd", "abce") - 1.0 / 3.0).abs() < 1e-12);
        assert!(
            (trigram_jaccard("Kal'tsit!", "kal tsit") - 1.0).abs() < 1e-12,
            "punctuation becomes space"
        );
        assert!(trigram_jaccard("", "") < 1e-12);
    }

    #[test]
    fn deictic_catches_what_the_prompt_forbids() {
        assert_eq!(deictic("What does the speaker want?"), Some("the speaker"));
        assert_eq!(
            deictic("Why does she leave?"),
            Some("pronoun without a named antecedent")
        );
        assert_eq!(
            deictic("Why does she leave Lungmen?"),
            None,
            "a name is present; the judge decides"
        );
        assert_eq!(
            deictic("What happens in this scene after the fight?"),
            Some("this scene")
        );
        assert_eq!(
            deictic("Why does Ch'en leave the Lungmen Guard Department?"),
            None
        );
        assert_eq!(deictic("Who is there when Amiya arrives?"), None);
    }
}
