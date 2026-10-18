//! The identity-claim check (`--no-identity-check`): an answer sentence asserting that two different names are one
//! person ("Later, she is referred to as Silence [3]", "X is also known as Y", "X is actually Y") stays only when an
//! identity link (`Tools::identities`, `Tools::identity_links`) joins the two names or a passage the model was given
//! holds both; otherwise the sentence is dropped. No model call: the prompts are those of the answer without the check,
//! byte for byte.
//!
//! Why: "Tell me more about the Quiet Woman" (Guide Ahead's opening story) retrieved four Mansfield Break passages by
//! dense similarity ("Quiet" sits next to "Silence"), and E2B joined the two with "Later, she is referred to as
//! Silence". The Quiet Woman is a wheelchair user in Laterano; Silence never appears in Guide Ahead.

use trevor::tools::{Tools, norm};

/// Phrases that assert an identity whatever precedes them.
const CLAIMS: &[&str] = &[
    "referred to as", "known as", "nicknamed", "revealed to be", "revealed as", "identified as",
    "the same person as", "the same character as", "the same individual as", "another name for", "an alias of",
    "an alias for", "a pseudonym of", "a pseudonym for", "goes by the name of", "goes by the name", "goes by",
    "also called", "also named", "is actually", "was actually", "is in fact", "was in fact", "is secretly",
    "was secretly", "is really", "was really", "is called", "was called", "are called", "were called", "be called",
    "been called", "is named", "was named",
];

/// Capitalized words that start a sentence or a clause, not a name.
pub(super) const NOT_NAMES: &[&str] = &[
    "The", "A", "An", "She", "He", "They", "It", "Her", "His", "Their", "Its", "Him", "Them", "Later", "In", "This",
    "That", "These", "Those", "However", "Also", "Both", "When", "After", "Before", "At", "On", "By", "As", "But",
    "And", "While", "During", "Then", "Here", "There", "Each", "Some", "Many", "Trevor", "I", "We", "You", "Tell",
    "What", "Who", "Whom", "Which", "Where", "Why", "How", "Is", "Are", "Was", "Were", "Do", "Does", "Did", "Can",
    "Could", "Give", "Explain", "Describe", "Summarize", "Summarise", "List", "Name", "More", "About", "Me", "Please",
    "Yes", "No", "Not", "One", "Another", "According", "Passage", "Passages", "Ms", "Mr", "Mrs", "Dr", "Regarding",
    "Based", "Overall", "Like", "Unlike", "Although", "Though", "Because", "Since", "If", "So", "Yet", "For", "From", "With", "To",
];

/// Lowercase words that may sit inside a name ("Knight of the Snow", "Feoria La Porta").
pub(super) const NAME_JOINERS: &[&str] = &["of", "the", "de", "von", "van", "du", "di", "la", "le", "del", "da"];

/// Pronouns that stand for the answer's subject. "They" and "it" are left out: "they are called Originium slugs"
/// names a kind of thing, not a person.
const PRONOUNS: &[&str] = &["she", "he"];

const ABBREVIATIONS: &[&str] = &["Mr", "Mrs", "Ms", "Dr", "St", "Jr", "Sr", "vs", "Mt", "No"];

/// One sentence of an answer and the whitespace after it.
pub(super) struct Piece<'a> {
    pub(super) text: &'a str,
    pub(super) sep: &'a str,
}

/// Split `text` into sentences: ".", "!" or "?" before whitespace or the end, with any "[n]" groups right after the
/// mark kept on the sentence; "Ms." and the like and single capitals ("W.") do not end one.
pub(super) fn sentences(text: &str) -> Vec<Piece<'_>> {
    let b = text.as_bytes();
    let mut out = Vec::new();
    let (mut start, mut i) = (0usize, 0usize);
    while i < b.len() {
        let c = b[i];
        if matches!(c, b'.' | b'!' | b'?') && (i + 1 == b.len() || b[i + 1].is_ascii_whitespace()) {
            let word = text[start..i].rsplit(|ch: char| ch.is_whitespace()).next().unwrap_or_default();
            let abbrev = c == b'.' && (ABBREVIATIONS.contains(&word)
                || (word.len() == 1 && word.chars().all(|ch| ch.is_ascii_uppercase())));
            if !abbrev {
                let mut end = i + 1;
                // Citations after the mark: "... frame. [1]".
                loop {
                    let rest = &text[end..];
                    let ws = rest.len() - rest.trim_start_matches(' ').len();
                    let r = &rest[ws..];
                    match r.strip_prefix('[').and_then(|x| x.find(']').map(|j| &x[..j])) {
                        Some(inner) if !inner.is_empty()
                            && inner.chars().all(|ch| ch.is_ascii_digit() || matches!(ch, ',' | ' ' | ';')) => {
                            end += ws + inner.len() + 2;
                        }
                        _ => break,
                    }
                }
                let sep_end = end + (text[end..].len() - text[end..].trim_start().len());
                out.push(Piece { text: &text[start..end], sep: &text[end..sep_end] });
                start = sep_end;
                i = sep_end;
                continue;
            }
        }
        i += 1;
    }
    if start < text.len() {
        let t = text[start..].trim_end();
        out.push(Piece { text: t, sep: &text[start + t.len()..] });
    }
    out
}

pub(super) fn clean(tok: &str) -> &str {
    tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'' && c != '-')
        .trim_end_matches("'s").trim_matches('\'')
}

pub(super) fn is_cap(tok: &str) -> bool {
    clean(tok).chars().next().is_some_and(char::is_uppercase)
}

/// The capitalized-name runs of `text`, in order ("In Guide Ahead, the Quiet Woman" gives "Guide Ahead" and "Quiet
/// Woman"). A run ends at a comma, a citation or a lowercase word that does not join two capitalized ones.
pub(super) fn name_runs(text: &str) -> Vec<String> {
    let toks: Vec<&str> = text.split_whitespace().collect();
    let mut runs = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    let flush = |cur: &mut Vec<&str>, runs: &mut Vec<String>| {
        while cur.last().is_some_and(|t| NAME_JOINERS.contains(&clean(t))) {
            cur.pop();
        }
        if !cur.is_empty() {
            runs.push(cur.iter().map(|t| clean(t)).collect::<Vec<_>>().join(" "));
        }
        cur.clear();
    };
    for (k, tok) in toks.iter().enumerate() {
        if tok.starts_with('[') {
            flush(&mut cur, &mut runs);
            continue;
        }
        let w = clean(tok);
        let joiner = !cur.is_empty() && NAME_JOINERS.contains(&w) && toks.get(k + 1).is_some_and(|n| is_cap(n));
        if (is_cap(tok) && !(cur.is_empty() && NOT_NAMES.contains(&w))) || joiner {
            cur.push(tok);
        } else {
            flush(&mut cur, &mut runs);
        }
        if tok.ends_with([',', ';', ':', '.', '!', '?', ')']) || tok.ends_with("\u{201d}") {
            flush(&mut cur, &mut runs);
        }
    }
    flush(&mut cur, &mut runs);
    runs
}

/// The name right after a claim phrase ("Silence in the context of ..." gives "Silence"), a leading "the" or "a"
/// skipped; None when what follows is not capitalized ("is actually a Sarkaz").
fn target_after(rest: &str) -> Option<String> {
    let mut toks: Vec<&str> = rest.split_whitespace().collect();
    if toks.first().is_some_and(|t| matches!(clean(t), "the" | "The")) {
        toks.remove(0);
    }
    let mut name: Vec<&str> = Vec::new();
    for (k, tok) in toks.iter().enumerate() {
        if tok.starts_with('[') {
            break;
        }
        let w = clean(tok);
        let joiner = !name.is_empty() && NAME_JOINERS.contains(&w) && toks.get(k + 1).is_some_and(|n| is_cap(n));
        if !(is_cap(tok) || joiner) || w.is_empty() {
            break;
        }
        // "is actually Lemuen's sister" relates two people; it does not name one.
        if tok.trim_end_matches([',', ';', ':', '.', '!', '?', ')']).ends_with("'s")
            || tok.trim_end_matches([',', ';', ':', '.', '!', '?', ')']).ends_with("\u{2019}s") {
            return None;
        }
        name.push(w);
        if tok.ends_with([',', ';', ':', '.', '!', '?', ')', '"']) || tok.ends_with("\u{201d}") {
            break;
        }
    }
    (!name.is_empty()).then(|| name.join(" "))
}

/// Whether `text` holds `name` as whole words, case kept (proper names are capitalized; "silent" is not "Silence").
pub(super) fn holds(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(i, _)| {
        text[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
            && text[i + name.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
    })
}

pub(super) fn same(a: &str, b: &str) -> bool {
    let (a, b) = (norm(a), norm(b));
    let strip = |x: &str| x.strip_prefix("the ").unwrap_or(x).to_owned();
    strip(&a) == strip(&b)
}

/// An identity link or a passage joins `subject` and `target`.
fn linked(tools: &Tools, texts: &[String], subject: &str, target: &str) -> bool {
    let group = tools.identities.iter().any(|g| g.iter().any(|n| same(n, subject)) && g.iter().any(|n| same(n, target)));
    let link = tools.identity_links.iter().any(|(a, b, ..)| (same(a, subject) && same(b, target))
        || (same(a, target) && same(b, subject)));
    group || link || texts.iter().any(|t| holds(t, subject) && holds(t, target))
}

/// What a claim phrase is said of.
enum Subject {
    Name(String),
    Pronoun,
    Other,
}

/// Words that may sit between a claim's subject and its phrase ("Silence, also known as", "she is now called").
const BETWEEN: &[&str] = &["is", "was", "are", "were", "be", "been", "being", "also", "later", "actually", "often",
                           "sometimes", "now", "formerly", "previously", "then", "in", "fact", "once", "herself",
                           "himself", "themselves", "itself"];

/// The subject of a claim whose phrase follows `pre`: the name run or pronoun right before it, with only `BETWEEN`
/// words and commas in between. Anything else ("a former host who was known as") is not a claim about a name.
fn subject_before(pre: &str) -> Subject {
    let toks: Vec<&str> = pre.split_whitespace().filter(|t| !t.starts_with('[')).collect();
    let mut end = toks.len();
    while end > 0 && BETWEEN.contains(&clean(toks[end - 1]).to_lowercase().as_str()) && !is_cap(toks[end - 1]) {
        end -= 1;
    }
    if end == 0 {
        return Subject::Other;
    }
    let last = clean(toks[end - 1]).to_lowercase();
    if PRONOUNS.contains(&last.as_str()) {
        return Subject::Pronoun;
    }
    if !is_cap(toks[end - 1]) {
        return Subject::Other;
    }
    // The run of capitalized words (and joiners inside it) ending at `end`.
    let mut start = end - 1;
    while start > 0 {
        let prev = toks[start - 1];
        let joiner = NAME_JOINERS.contains(&clean(prev)) && start >= 2 && is_cap(toks[start - 2]);
        if prev.ends_with([',', ';', ':', '.', '!', '?', ')']) || !(is_cap(prev) || joiner) {
            break;
        }
        start -= 1;
    }
    while start < end && NOT_NAMES.contains(&clean(toks[start])) {
        start += 1;
    }
    if start == end {
        return Subject::Other;
    }
    Subject::Name(toks[start..end].iter().map(|t| clean(t)).collect::<Vec<_>>().join(" "))
}

/// The two names of each identity claim in `sentence` (subject, target) that nothing joins. `lead` is the answer's
/// subject (the first name of its first sentence), standing in for a pronoun subject ("Later, she is referred to as").
fn unsupported(sentence: &str, lead: &[String], tools: &Tools, texts: &[String]) -> Option<(String, String)> {
    let lower = sentence.to_ascii_lowercase();
    let mut hits: Vec<(usize, &str)> = CLAIMS.iter().filter_map(|p| {
        lower.match_indices(p).find(|(i, _)| {
            lower[..*i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                && lower[i + p.len()..].starts_with(' ')
        }).map(|(i, _)| (i, *p))
    }).collect();
    hits.sort_by_key(|&(i, p)| (i, std::cmp::Reverse(p.len())));
    let &(at, phrase) = hits.first()?;
    let target = target_after(&sentence[at + phrase.len()..])?;
    let subjects: Vec<String> = match subject_before(&sentence[..at]) {
        Subject::Name(s) => vec![s],
        Subject::Pronoun => lead.to_vec(),
        Subject::Other => return None,
    };
    let subjects: Vec<String> = subjects.into_iter().filter(|s| !same(s, &target)
        && !norm(s).contains(&norm(&target)) && !norm(&target).contains(&norm(s))).collect();
    if subjects.is_empty() || subjects.iter().any(|s| linked(tools, texts, s, &target)) {
        return None;
    }
    Some((subjects[0].clone(), target))
}

/// The answer with each unsupported identity claim's sentence dropped, and the claims as (subject, target); None when
/// there is none. When every sentence would go, the answer stays and a check note naming the claim is appended.
pub(crate) fn check(answer: &str, q: &str, texts: &[String], tools: &Tools) -> Option<(String, Vec<(String, String)>)> {
    let pieces = sentences(answer);
    let mut lead: Vec<String> = pieces.first().map(|p| name_runs(p.text)).unwrap_or_default().into_iter().take(1)
        .collect();
    lead.extend(name_runs(q));
    let flags: Vec<Option<(String, String)>> = pieces.iter().map(|p| unsupported(p.text, &lead, tools, texts))
        .collect();
    let claims: Vec<(String, String)> = flags.iter().flatten().cloned().collect();
    if claims.is_empty() {
        return None;
    }
    if flags.iter().all(Option::is_some) {
        let notes: Vec<String> = claims.iter()
            .map(|(s, t)| format!("no passage Trevor read says that {s} and {t} are the same person")).collect();
        return Some((format!("{answer}\n\nTrevor's check: {}.", notes.join("; ")), claims));
    }
    let keep: Vec<bool> = flags.iter().map(Option::is_none).collect();
    Some((kept_sentences(&pieces, &keep), claims))
}

/// The sentences of `pieces` whose `keep` is true, joined as they were; a dropped paragraph end keeps its break.
pub(super) fn kept_sentences(pieces: &[Piece<'_>], keep: &[bool]) -> String {
    let mut out = String::new();
    for (p, &k) in pieces.iter().zip(keep) {
        if !k {
            if p.sep.contains('\n') && !out.is_empty() {
                let trimmed = out.trim_end().len();
                out.truncate(trimmed);
                out.push_str(p.sep);
            }
            continue;
        }
        out.push_str(p.text);
        out.push_str(p.sep);
    }
    out.trim_end().to_owned()
}

/// The chunk ids an answer's "[n]" citations name, in order, by passage number (`passages[n - 1]`).
/// How many citations of `text` point at no passage (of `passages` passages), counted as `generate::citations` does.
pub(crate) fn invalid_count(text: &str, passages: usize) -> usize {
    let mut invalid = 0usize;
    let mut i = 0;
    while let Some(open) = text[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = text[start..].find(']') else { break };
        let inner = &text[start..start + close];
        let toks: Vec<&str> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).collect();
        let nums: Vec<usize> = toks.iter().filter_map(|t| t.parse().ok()).collect();
        if !nums.is_empty() && nums.len() == toks.len() {
            invalid += nums.iter().filter(|&&n| n == 0 || n > passages).count();
        }
        i = start + close;
    }
    invalid
}

pub(crate) fn cited_ids(text: &str, passages: &[String]) -> Vec<String> {
    let mut cited: Vec<String> = Vec::new();
    let mut i = 0;
    while let Some(open) = text[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = text[start..].find(']') else { break };
        let inner = &text[start..start + close];
        let toks: Vec<&str> = inner.split([',', ';', ' ']).filter(|t| !t.is_empty()).collect();
        let nums: Vec<usize> = toks.iter().filter_map(|t| t.parse().ok()).collect();
        if !nums.is_empty() && nums.len() == toks.len() {
            for n in nums {
                if let Some(id) = passages.get(n.wrapping_sub(1)) {
                    if !cited.contains(id) {
                        cited.push(id.clone());
                    }
                }
            }
        }
        i = start + close;
    }
    cited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_citations_are_numbers_past_the_passages() {
        assert_eq!(invalid_count("In the texts [3] and [12], and [0] [1, 4] [a 2].", 9), 2);
        assert_eq!(invalid_count("No citation here.", 9), 0);
    }

    fn texts() -> Vec<String> {
        vec![
            "(story: Guide Ahead, Right Place, Right time)\nThe woman in the wheelchair utters not a word. It is silent.\n\
             Wasting Man: She'll be back soon.\nQuiet Woman: Don't come back.".to_owned(),
            "(story: Mansfield Break, Secret Meeting)\nMayer: Silence, she really wanted to meet you here?\n\
             Silence: Mm.".to_owned(),
            "(story: x)\nAmiya: Kal'tsit, also known as the Doctor's old friend.\nKal'tsit: Yes.".to_owned(),
        ]
    }

    #[test]
    fn the_quiet_woman_answer_loses_its_silence_sentence() {
        let a = "The Quiet Woman is a woman who is in a wheelchair [1]. She is the one who speaks to the Wasting Man, \
                 asking how long he will stay [1]. She has watched the sun rise and set over Laterano countless times \
                 through a window frame [1]. Later, she is referred to as Silence in the context of a secret meeting [3].";
        let (out, claims) = check(a, "Tell me more about the Quiet Woman", &texts(), &Tools::default()).unwrap();
        assert_eq!(claims, vec![("Quiet Woman".to_owned(), "Silence".to_owned())]);
        assert!(out.ends_with("through a window frame [1]."), "{out}");
        assert!(!out.contains("Silence"));
        assert_eq!(cited_ids(&out, &["a".into(), "b".into(), "c".into()]), vec!["a".to_owned()]);
    }

    #[test]
    fn a_claim_a_passage_or_an_identity_link_supports_stays() {
        // Mayer and Silence share passage 2; Mayer and Kal'tsit share none.
        let a = "Mayer is also known as Kal'tsit [3]. Mayer is also known as Silence [2]. Mayer talks [2].";
        let r = check(a, "Who is Mayer?", &texts(), &Tools::default());
        assert_eq!(r.map(|x| x.1), Some(vec![("Mayer".to_owned(), "Kal'tsit".to_owned())]));
        let mut t = Tools::default();
        t.identities.push(vec!["Quiet Woman".into(), "Silence".into()]);
        assert!(check("Later, she is referred to as Silence [2]. She reads [1].", "Who is the Quiet Woman?",
                      &texts(), &t).is_none());
    }

    #[test]
    fn non_claims_do_not_fire() {
        let t = Tools::default();
        for a in ["She is actually a Sankta [1].", "A girl named Cecelia lives there [1].",
                  "The city is known as the holy city [1].", "He called her a fool [1].",
                  "Ms. Silence meets Muelsyse [2]. Silence is known as Silence [2].",
                  "The Quiet Woman is actually Lemuen's sister [1].",
                  "Vanilla says that they are called \"Originium slugs\" [2].",
                  "Regarding Midnight, the story describes him as a host who was known as the \"Devil\" [2]."] {
            assert!(check(a, "Who is she?", &texts(), &t).is_none(), "{a}");
        }
    }

    #[test]
    fn a_lone_claim_keeps_the_answer_and_gets_a_note() {
        let (out, _) = check("The Quiet Woman is actually Silence [2].", "Who is the Quiet Woman?", &texts(),
                             &Tools::default()).unwrap();
        assert!(out.starts_with("The Quiet Woman is actually Silence [2]."));
        assert!(out.contains("Trevor's check: no passage Trevor read says that Quiet Woman and Silence"));
    }

    #[test]
    fn sentences_keep_paragraph_breaks_and_citations() {
        let p = sentences("One [1].\n\nTwo. [2] Three Ms. X here.");
        let got: Vec<(&str, &str)> = p.iter().map(|x| (x.text, x.sep)).collect();
        assert_eq!(got, vec![("One [1].", "\n\n"), ("Two. [2]", " "), ("Three Ms. X here.", "")]);
    }
}
