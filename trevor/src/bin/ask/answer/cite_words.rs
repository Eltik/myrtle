//! Passage labels out of the answer text (`--no-passage-label-fix`): the model sees its sources as "[n]" passages and
//! sometimes writes "in passage [4]" or "passages [2], [3]"; a reader sees only the bracketed citations, so "passage" is
//! Trevor's internal word. Each "passage"/"passages" right before a citation (or before a bare number, which becomes
//! the citation "[n]") is written as "the text"/"the texts" (or "text"/"texts" after an article or a possessive word),
//! so the citation stays the bracket alone. No model call.
//!
//! Why: "When was Talulah last mentioned (chronologically)?" (E2B, mid) answered "in passage [4] in the story Roaring
//! Flare, Behold, the Twilight Comes". On the recorded E2B mid answers of the four eval sets, 5 of 341 wrote a passage
//! label ("Elysium appears in passages [1], [2], ...", "In passage [3], Grace asks the Doctor", "based on T1, T2, and
//! passage [2]").

/// Words after which "passage" keeps no article of its own ("the passage [2]" is "the text [2]").
const DETERMINERS: &[&str] = &[
    "the", "this", "that", "these", "those", "each", "every", "a", "an", "one", "provided", "same", "other", "another",
    "first", "last", "both", "which", "its", "their", "his", "her",
];

/// The answer with each passage label rewritten, or None when it has none.
pub(crate) fn fix(answer: &str) -> Option<String> {
    let mut out = String::with_capacity(answer.len());
    let mut changed = false;
    let mut i = 0;
    let lower = answer.to_ascii_lowercase();
    while i < answer.len() {
        let at_word = i == 0 || !answer[..i].chars().next_back().is_some_and(char::is_alphanumeric);
        if at_word && lower[i..].starts_with("passage") {
            let mut j = i + "passage".len();
            let plural = lower[j..].starts_with('s');
            if plural {
                j += 1;
            }
            let rest = &answer[j..];
            let ws = rest.len() - rest.trim_start_matches(' ').len();
            let after = &rest[ws..];
            let digits = after.chars().take_while(char::is_ascii_digit).count();
            let cites = after.starts_with('[') && after[1..].starts_with(|c: char| c.is_ascii_digit());
            // A bare number is a citation unless a letter or a possessive follows it ("passage 2's speaker" stays).
            let bare = digits > 0
                && !after[digits..].starts_with(|c: char| c.is_alphanumeric() || c == '\'' || c == '\u{2019}');
            if (cites || bare) && answer[j..].chars().next().is_none_or(|c| !c.is_alphanumeric()) {
                let prev = out
                    .trim_end()
                    .rsplit(|c: char| c.is_whitespace())
                    .next()
                    .unwrap_or_default()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_ascii_lowercase();
                let starts = out.trim_end().is_empty() || out.trim_end().ends_with(['.', '!', '?', '\n', ':']);
                let noun = if plural { "texts" } else { "text" };
                let word = if DETERMINERS.contains(&prev.as_str()) {
                    noun.to_owned()
                } else if starts {
                    format!("The {noun}")
                } else {
                    format!("the {noun}")
                };
                out.push_str(&word);
                out.push(' ');
                if bare {
                    out.push('[');
                    out.push_str(&after[..digits]);
                    out.push(']');
                    i = j + ws + digits;
                    // "passages 3 and 4": every bare number of the list becomes a citation.
                    if plural {
                        while let Some((sep, n)) = list_number(&answer[i..]) {
                            out.push_str(&answer[i..i + sep]);
                            out.push('[');
                            out.push_str(&answer[i + sep..i + sep + n]);
                            out.push(']');
                            i += sep + n;
                        }
                    }
                } else {
                    i = j + ws;
                }
                changed = true;
                continue;
            }
        }
        let c = answer[i..].chars().next().unwrap_or(' ');
        out.push(c);
        i += c.len_utf8();
    }
    changed.then_some(out)
}

/// The next number of a bare list ("3, 4 and 5"): (separator length, digits), for ", ", " and ", ", and ", " or ",
/// ", or ", "-" or " to " then digits that no letter or possessive follows.
fn list_number(s: &str) -> Option<(usize, usize)> {
    let sep = [", and ", ", or ", " and ", " or ", ", ", " to ", "-"]
        .iter()
        .find(|p| s.starts_with(**p))?
        .len();
    let digits = s[sep..].chars().take_while(char::is_ascii_digit).count();
    let ok =
        digits > 0 && !s[sep + digits..].starts_with(|c: char| c.is_alphanumeric() || c == '\'' || c == '\u{2019}');
    ok.then_some((sep, digits))
}

#[cfg(test)]
mod tests {
    use super::fix;

    #[test]
    fn labels_become_the_text() {
        let cases = [
            (
                "Talulah was last mentioned in passage [4] in the story Roaring Flare.",
                "Talulah was last mentioned in the text [4] in the story Roaring Flare.",
            ),
            (
                "Elysium appears in passages [1], [2], [3].",
                "Elysium appears in the texts [1], [2], [3].",
            ),
            (
                "Hi [1]. In passage [3], Grace asks [3].",
                "Hi [1]. In the text [3], Grace asks [3].",
            ),
            (
                "Passage [5] is estimated to be around 1100.",
                "The text [5] is estimated to be around 1100.",
            ),
            (
                "Estimate: 1086, based on T1, T2, and passage [2].",
                "Estimate: 1086, based on T1, T2, and the text [2].",
            ),
            (
                "It is not in the provided passages [2], [3].",
                "It is not in the provided texts [2], [3].",
            ),
            ("As passage 4 says, she left.", "As the text [4] says, she left."),
            ("It is in passages 3 and 4.", "It is in the texts [3] and [4]."),
            (
                "See passages 1, 2, and 5 for it.",
                "See the texts [1], [2], and [5] for it.",
            ),
            ("Passages 2-3 say so.", "The texts [2]-[3] say so."),
            ("In passage 2 and 3 people.", "In the text [2] and 3 people."),
        ];
        for (a, want) in cases {
            assert_eq!(fix(a).as_deref(), Some(want), "{a}");
        }
        for a in [
            "The passages do not say [1].",
            "A passage of the story [2].",
            "Passageway 4 [1].",
            "the passage in Episode 4 [2].",
            "In passage 2's last line, she leaves [2].",
            "In passage 2\u{2019}s last line, she leaves [2].",
        ] {
            assert_eq!(fix(a), None, "{a}");
        }
    }
}
