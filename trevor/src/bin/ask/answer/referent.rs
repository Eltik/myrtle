//! The referent checks (no model; the prompts are those of the answer without them, byte for byte):
//!
//! - the fate check (`--no-referent-check`): a sentence saying that a named character is dead or alive, died, was
//!   killed or survived ("Mandragora is dead [5]") stays only when a passage it cites ties that state to that character:
//!   a line naming them (or a name an identity link joins to theirs) beside a line with a death or survival word, or a
//!   line of theirs that speaks of their own death in the first person. A line spoken by them about someone else ("Hah,
//!   she's dead") does not count.
//! - the quote check (`--no-quote-check`): a sentence saying that a named character said a quoted line ("Harmonie states
//!   that 'she's dead'") is dropped when the passages hold the quote only in lines another named speaker says.
//!
//! Why: "Is Mandragora alive or dead?" (E2B, mid) was answered "Mandragora is dead [5]. Harmonie states that 'she's
//! dead' in reference to The Leader." Passage [5] was Stormwatch's "Verse in the Ashes" (`main_9_level_st_09-02#0002`),
//! where Mandragora herself says "Hah, she's dead" of Saoirse Kelly, the traitor The Leader has just executed. The
//! verdict happens to be right (Mandragora dies in Shatterpoint's "Crossroads", and The Brigadier says "Mandragora is
//! dead." in What the Firelight Casts), but neither sentence is what its passage says.

use trevor::tools::Tools;

use super::identity::{NAME_JOINERS, NOT_NAMES, clean, holds, is_cap, kept_sentences, name_runs, same, sentences};

/// Words that state a fate in an answer sentence (whole words, lowercase). "passed away" and "lost her/his/their life"
/// are matched as phrases.
const FATE: &[&str] = &[
    "dead", "alive", "died", "dies", "killed", "executed", "murdered", "perished", "perishes", "survived", "survives",
    "deceased", "slain",
];

/// Words that may sit between a fate claim's subject and its fate word ("Mandragora is eventually killed", "she is no
/// longer alive", "Mandragora was confirmed dead").
const BETWEEN: &[&str] = &[
    "is",
    "was",
    "are",
    "were",
    "has",
    "had",
    "have",
    "been",
    "be",
    "being",
    "will",
    "would",
    "did",
    "does",
    "not",
    "no",
    "never",
    "longer",
    "still",
    "eventually",
    "ultimately",
    "finally",
    "later",
    "already",
    "apparently",
    "presumably",
    "likely",
    "also",
    "reportedly",
    "tragically",
    "confirmed",
    "seemingly",
    "indeed",
    "actually",
    "certainly",
    "probably",
    "definitely",
    "now",
    "then",
    "both",
];

/// Stems of the words that show a death or survival in a passage line.
const FATE_STEMS: &[&str] = &[
    "dead",
    "death",
    "die",
    "died",
    "dies",
    "dying",
    "kill",
    "alive",
    "live",
    "living",
    "survive",
    "surviving",
    "execut",
    "perish",
    "murder",
    "corpse",
    "grave",
    "funeral",
    "buried",
    "bury",
    "slain",
    "passed away",
    "lost her life",
    "lost his life",
    "fallen",
];

/// Prepositions: a name right after one is not a sentence's subject.
const PREPOSITIONS: &[&str] = &[
    "in", "of", "from", "at", "to", "with", "by", "for", "on", "about", "during", "after", "before", "near", "inside",
    "within", "against", "toward", "towards", "like",
];

/// First-person phrases of a character's own line about their own fate ("Mandragora: I just couldn't hold on! ...
/// Dead." is about a traitor, so a first-person word beside a death word is not enough).
const FIRST_PERSON: &[&str] = &[
    "i'm dying",
    "i am dying",
    "i'll die",
    "i will die",
    "i'm going to die",
    "i'm dead",
    "i am dead",
    "kill me",
    "my death",
    "i died",
    "i won't live",
    "i'm alive",
    "i am alive",
    "i'm still alive",
    "i survived",
    "i'm not dead",
    "i didn't die",
    "my life is",
    "my last breath",
    "i'm done for",
];

/// Speech verbs of an attributed quote ("Harmonie states that '...'").
const SAYS: &[&str] = &[
    "say",
    "says",
    "said",
    "state",
    "states",
    "stated",
    "tell",
    "tells",
    "told",
    "remark",
    "remarks",
    "remarked",
    "note",
    "notes",
    "noted",
    "exclaim",
    "exclaims",
    "exclaimed",
    "reply",
    "replies",
    "replied",
    "mention",
    "mentions",
    "mentioned",
    "declare",
    "declares",
    "declared",
    "claim",
    "claims",
    "claimed",
    "admit",
    "admits",
    "admitted",
    "ask",
    "asks",
    "asked",
    "shout",
    "shouts",
    "shouted",
    "explain",
    "explains",
    "explained",
    "comment",
    "comments",
    "commented",
    "add",
    "adds",
    "added",
    "respond",
    "responds",
    "responded",
    "yell",
    "yells",
    "yelled",
    "whisper",
    "whispers",
    "whispered",
    "muse",
    "muses",
    "mused",
    "insist",
    "insists",
    "insisted",
    "warn",
    "warns",
    "warned",
    "recall",
    "recalls",
    "recalled",
    "reveal",
    "reveals",
    "revealed",
    "write",
    "writes",
    "wrote",
    "cry",
    "cries",
    "cried",
    "scream",
    "screams",
    "screamed",
];

/// Words that may sit between a quote's speaker and the speech verb ("Harmonie later states").
const SAYS_BETWEEN: &[&str] = &[
    "also",
    "later",
    "then",
    "explicitly",
    "clearly",
    "even",
    "directly",
    "simply",
    "once",
    "openly",
    "bluntly",
    "calmly",
    "coldly",
    "angrily",
    "quietly",
    "finally",
    "herself",
    "himself",
    "further",
    "again",
    "reportedly",
];

/// A passage line split into its speaker label (a short "Name:" prefix of capitalized words) and its text.
fn speaker_and_text(line: &str) -> (Option<&str>, &str) {
    if let Some(i) = line.find(": ") {
        let head = &line[..i];
        let ok = !head.is_empty()
            && head.len() <= 40
            && !head.contains(['.', '!', '?', ','])
            && head.split_whitespace().count() <= 5
            && head.split_whitespace().all(|w| {
                NAME_JOINERS.contains(&w) || w.chars().find(|c| c.is_alphanumeric()).is_some_and(char::is_uppercase)
            });
        if ok {
            return (Some(head.trim_matches(|c: char| c == '\'' || c == '"')), &line[i + 2..]);
        }
    }
    (None, line)
}

fn lower_words(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.trim_matches('\'').to_lowercase().replace('\u{2019}', "'"))
        .collect()
}

/// Whether a line's text has a death or survival word.
fn fate_in(text: &str) -> bool {
    let low = text.to_lowercase().replace('\u{2019}', "'");
    let words = lower_words(&low);
    FATE_STEMS.iter().any(|s| {
        if s.contains(' ') {
            low.contains(s)
        } else {
            words.iter().any(|w| w.starts_with(s))
        }
    })
}

/// The forms of a character's name a passage may use: the name, the names an identity link joins to it, and the
/// first word of a two-word name ("Talulah" of "Talulah Artorius") when it is 4 letters or more.
fn forms(tools: &Tools, name: &str) -> Vec<String> {
    let mut out = vec![name.to_owned()];
    for g in tools.identities.iter().filter(|g| g.iter().any(|n| same(n, name))) {
        out.extend(g.iter().cloned());
    }
    for (a, b, ..) in &tools.identity_links {
        if same(a, name) {
            out.push(b.clone());
        } else if same(b, name) {
            out.push(a.clone());
        }
    }
    let words: Vec<&str> = name.split_whitespace().collect();
    if words.len() >= 2 && words[0].chars().count() >= 4 && !NOT_NAMES.contains(&words[0]) {
        out.push(words[0].to_owned());
    }
    out.sort();
    out.dedup();
    out
}

fn names_any(text: &str, forms: &[String]) -> bool {
    forms.iter().any(|f| holds(text, f))
}

/// Whether passage `text` (its "(label)" line first) ties a fate to the character of `forms`.
fn supports_fate(text: &str, forms: &[String]) -> bool {
    let mut lines: Vec<&str> = text.lines().collect();
    // The label names the passage's source; it counts only when it states the fate itself (the deaths evidence).
    if lines.first().is_some_and(|l| l.starts_with('(')) {
        let label = lines.remove(0);
        if fate_in(label) && names_any(label, forms) {
            return true;
        }
    }
    let parts: Vec<(Option<&str>, &str)> = lines.iter().map(|l| speaker_and_text(l)).collect();
    for (j, (who, t)) in parts.iter().enumerate() {
        if !fate_in(t) {
            continue;
        }
        let near = j.saturating_sub(1)..=(j + 1).min(parts.len().saturating_sub(1));
        if near.into_iter().any(|k| names_any(parts[k].1, forms)) {
            return true;
        }
        let own = who.is_some_and(|w| forms.iter().any(|f| same(w, f)));
        let low = t.to_lowercase().replace('\u{2019}', "'");
        if own && FIRST_PERSON.iter().any(|p| low.contains(p)) {
            return true;
        }
    }
    false
}

/// The spans of `s` inside quotation marks: "...", “...”, and '...' opened at a word start and closed at a word end.
fn quote_spans(s: &str) -> Vec<(usize, usize)> {
    let b: Vec<(usize, char)> = s.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let (at, c) = b[i];
        let close = match c {
            '"' => Some('"'),
            '\u{201c}' => Some('\u{201d}'),
            '\'' | '\u{2018}' => {
                let starts = i == 0 || b[i - 1].1.is_whitespace() || matches!(b[i - 1].1, '(' | '[');
                starts.then_some(if c == '\'' { '\'' } else { '\u{2019}' })
            }
            _ => None,
        };
        if let Some(cl) = close {
            let end = (i + 1..b.len()).find(|&k| {
                b[k].1 == cl && (cl == '"' || cl == '\u{201d}' || b.get(k + 1).is_none_or(|n| !n.1.is_alphanumeric()))
            });
            if let Some(k) = end {
                out.push((at + c.len_utf8(), b[k].0));
                i = k + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// The capitalized subject right before token `end` of `toks` (exclusive), over `between` words; "she"/"he" give
/// `lead`. Empty when there is none.
fn subject_at(toks: &[&str], mut end: usize, between: &[&str], lead: &[String]) -> Vec<String> {
    while end > 0 && between.contains(&clean(toks[end - 1]).to_lowercase().as_str()) && !is_cap(toks[end - 1]) {
        end -= 1;
    }
    if end == 0 {
        return Vec::new();
    }
    let last = clean(toks[end - 1]).to_lowercase();
    if matches!(last.as_str(), "she" | "he") {
        return lead.to_vec();
    }
    // An article right before the fate word ("A deceased Operator") or a name ending a clause is not a subject.
    if !is_cap(toks[end - 1])
        || NOT_NAMES.contains(&clean(toks[end - 1]))
        || toks[end - 1].ends_with([',', ';', ':', '.', '!', '?', ')'])
    {
        return Vec::new();
    }
    let mut start = end - 1;
    while start > 0 {
        let prev = toks[start - 1];
        let joiner = NAME_JOINERS.contains(&clean(prev)) && start >= 2 && is_cap(toks[start - 2]);
        if prev.ends_with([',', ';', ':', '.', '!', '?', ')']) || !(is_cap(prev) || joiner) {
            break;
        }
        start -= 1;
    }
    // A name after a preposition is not the subject ("The Young Chef in Baizao claims").
    if start > 0 && PREPOSITIONS.contains(&clean(toks[start - 1]).to_lowercase().as_str()) {
        return Vec::new();
    }
    while start < end && NOT_NAMES.contains(&clean(toks[start])) {
        start += 1;
    }
    if start == end {
        return Vec::new();
    }
    vec![toks[start..end].iter().map(|t| clean(t)).collect::<Vec<_>>().join(" ")]
}

/// The passage numbers a sentence cites ("[4]", "[4, 5]").
fn cites(sentence: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(open) = sentence[i..].find('[') {
        let start = i + open + 1;
        let Some(close) = sentence[start..].find(']') else {
            break;
        };
        let toks: Vec<&str> = sentence[start..start + close]
            .split([',', ';', ' '])
            .filter(|t| !t.is_empty())
            .collect();
        let nums: Vec<usize> = toks.iter().filter_map(|t| t.parse().ok()).collect();
        if !nums.is_empty() && nums.len() == toks.len() {
            out.extend(nums);
        }
        i = start + close;
    }
    out
}

/// The character a fate claim of `sentence` is about when none of its cited passages ties the fate to them.
fn unsupported_fate(sentence: &str, lead: &[String], tools: &Tools, texts: &[String]) -> Option<String> {
    let cited = cites(sentence);
    if cited.is_empty() || cited.iter().any(|&n| n == 0 || n > texts.len()) {
        return None;
    }
    // Fate words inside quotation marks are what someone said, not a claim of the answer.
    let quotes = quote_spans(sentence);
    let mut toks: Vec<(usize, &str)> = Vec::new();
    let mut at = 0;
    for t in sentence.split_whitespace() {
        let i = sentence[at..].find(t).map_or(at, |k| at + k);
        at = i + t.len();
        if !quotes.iter().any(|&(a, b)| i >= a && i < b) {
            toks.push((i, t));
        }
    }
    let words: Vec<&str> = toks.iter().map(|(_, t)| *t).collect();
    for (k, t) in words.iter().enumerate() {
        let w = clean(t).to_lowercase();
        let fate = FATE.contains(&w.as_str())
            || (w == "passed" && words.get(k + 1).is_some_and(|n| clean(n).eq_ignore_ascii_case("away")))
            || (w == "lost" && words.get(k + 2).is_some_and(|n| clean(n).eq_ignore_ascii_case("life")));
        if !fate {
            continue;
        }
        let subjects = subject_at(&words, k, BETWEEN, lead);
        if subjects.is_empty() {
            continue;
        }
        let ok = subjects.iter().any(|s| {
            let f = forms(tools, s);
            cited.iter().any(|&n| supports_fate(&texts[n - 1], &f))
        });
        if !ok {
            return Some(subjects[0].clone());
        }
    }
    None
}

fn norm_quote(s: &str) -> String {
    let s = s
        .replace(['\u{2019}', '\u{2018}'], "'")
        .replace(['\u{201c}', '\u{201d}'], "\"")
        .to_lowercase();
    let s = s.trim_matches(|c: char| !c.is_alphanumeric());
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// (said to say it, who says it in the passages) for a quote of `sentence` the passages hold only in another named
/// speaker's lines.
fn misattributed_quote(sentence: &str, tools: &Tools, texts: &[String]) -> Option<(String, String)> {
    for (a, b) in quote_spans(sentence) {
        let quote = norm_quote(&sentence[a..b]);
        if quote.split_whitespace().count() < 2 {
            continue;
        }
        // The speech verb within the 6 words before the quote, and the name right before it.
        let before: Vec<&str> = sentence[..a]
            .trim_end_matches(['"', '\'', '\u{201c}', '\u{2018}'])
            .split_whitespace()
            .collect();
        let from = before.len().saturating_sub(6);
        let Some(v) = (from..before.len())
            .rev()
            .find(|&k| SAYS.contains(&clean(before[k]).to_lowercase().as_str()))
        else {
            continue;
        };
        let said_by = subject_at(&before, v, SAYS_BETWEEN, &[]);
        let Some(said_by) = said_by.first() else { continue };
        let f = forms(tools, said_by);
        let mut speakers: Vec<String> = Vec::new();
        let mut narrated = false;
        for t in texts {
            for line in t.lines().skip(1) {
                let (who, text) = speaker_and_text(line);
                if !norm_quote(text).contains(&quote) {
                    continue;
                }
                match who {
                    Some(w) if f.iter().any(|x| same(x, w)) => return None,
                    Some(w) => speakers.push(w.to_owned()),
                    None => narrated = true,
                }
            }
        }
        if !narrated && !speakers.is_empty() {
            speakers.sort();
            speakers.dedup();
            return Some((said_by.clone(), speakers.join(" and ")));
        }
    }
    None
}

/// What the referent checks found in one answer.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Found {
    /// Characters whose fate a sentence states without a cited passage tying it to them.
    pub(crate) fates: Vec<String>,
    /// (said to say it, who says it) for each misattributed quote.
    pub(crate) quotes: Vec<(String, String)>,
}

/// The answer without its unsupported fate claims (`fate`) and misattributed quotes (`quote`), and what was found;
/// None when nothing was. When every sentence would go, the answer stays and a check note is appended.
pub(crate) fn check(
    answer: &str,
    q: &str,
    texts: &[String],
    tools: &Tools,
    fate: bool,
    quote: bool,
) -> Option<(String, Found)> {
    if !fate && !quote {
        return None;
    }
    let pieces = sentences(answer);
    let mut lead: Vec<String> = pieces
        .first()
        .map(|p| name_runs(p.text))
        .unwrap_or_default()
        .into_iter()
        .take(1)
        .collect();
    lead.extend(name_runs(q));
    let mut found = Found::default();
    let keep: Vec<bool> = pieces
        .iter()
        .map(|p| {
            let f = if fate {
                unsupported_fate(p.text, &lead, tools, texts)
            } else {
                None
            };
            let m = if quote {
                misattributed_quote(p.text, tools, texts)
            } else {
                None
            };
            let ok = f.is_none() && m.is_none();
            found.fates.extend(f);
            found.quotes.extend(m);
            ok
        })
        .collect();
    if keep.iter().all(|&k| k) {
        return None;
    }
    if keep.iter().all(|&k| !k) {
        let mut notes: Vec<String> = found
            .fates
            .iter()
            .map(|x| format!("the passage cited for what happened to {x} does not say it of {x}"))
            .collect();
        notes.extend(
            found
                .quotes
                .iter()
                .map(|(s, w)| format!("in the passages that line is said by {w}, not {s}")),
        );
        return Some((format!("{answer}\n\nTrevor's check: {}.", notes.join("; ")), found));
    }
    Some((kept_sentences(&pieces, &keep), found))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stormwatch() -> Vec<String> {
        vec![
            "(story: What the Firelight Casts, Unlucky Reunion; the line where Trevor's deaths table finds a report \
             that Mandragora is dead)\n'The Brigadier': Mandragora is dead. I can stop investigating your whim.\n\
             Harmonie: Oh, you mean that dear, pitiful Loughshinny."
                .to_owned(),
            "(story: Stormwatch, Verse in the Ashes)\n'The Leader': — For... Dublinn! I've said these words.\n\
             Mandragora: Hah, she's dead. That's decent enough. Give the order, the traitor's been executed.\n\
             Mandragora: I just couldn't hold on! All because of this shameless traitor. Dead. Dust to dust.\n\
             Dublinn Soldier: Yes, sir.\nHarmonie: Jealous, now?"
                .to_owned(),
            "(story: Shatterpoint, Crossroads)\nManfred: You won't live much longer either, Mandragora.\n\
             Mandragora: ......"
                .to_owned(),
        ]
    }

    #[test]
    fn the_mandragora_answer_loses_both_sentences_and_gets_a_note() {
        let a = "Mandragora is dead [2]. Harmonie states that 'she's dead' in reference to The Leader.";
        let (out, found) = check(
            a,
            "Is Mandragora alive or dead?",
            &stormwatch(),
            &Tools::default(),
            true,
            true,
        )
        .unwrap();
        assert_eq!(found.fates, vec!["Mandragora".to_owned()]);
        assert_eq!(found.quotes, vec![("Harmonie".to_owned(), "Mandragora".to_owned())]);
        assert!(out.starts_with(a), "{out}");
        assert!(
            out.contains(
                "Trevor's check: the passage cited for what happened to Mandragora does not say it of \
                              Mandragora; in the passages that line is said by Mandragora, not Harmonie."
            ),
            "{out}"
        );
    }

    #[test]
    fn a_fate_its_passage_ties_to_the_character_stays() {
        let t = Tools::default();
        // The deaths evidence's label, a line naming her beside a death word, and the pronoun standing for her.
        for a in [
            "Mandragora is dead [1].",
            "Mandragora is dead [3].",
            "She is dead [1]. She fought Manfred [3].",
            "Mandragora is no longer alive [1, 2].",
        ] {
            assert!(
                check(a, "Is Mandragora alive or dead?", &stormwatch(), &t, true, true).is_none(),
                "{a}"
            );
        }
        // One unsupported sentence of two is dropped.
        let (out, _) = check(
            "Mandragora fought Manfred [3]. Mandragora was executed [2].",
            "What happened to Mandragora?",
            &stormwatch(),
            &t,
            true,
            true,
        )
        .unwrap();
        assert_eq!(out, "Mandragora fought Manfred [3].");
    }

    #[test]
    fn non_claims_do_not_fire() {
        let t = Tools::default();
        for a in [
            "The traitor is dead [2].",
            "Mandragora says the traitor is dead [2].",
            "*   **Mandragora (Spy):** A deceased officer [2].",
            "The traitor in Stormwatch died [2].",
            "The officer in Harmonie says \"hah, she's dead\" [2].",
            "Mandragora says, \"Hah, she's dead\" [2].",
            "Many soldiers died in the fire [2].",
            "Mandragora is dead.",
            "Harmonie says \"jealous, now\" [2].",
            "Mandragora is alive [9].",
        ] {
            assert!(
                check(a, "Is Mandragora alive or dead?", &stormwatch(), &t, true, true).is_none(),
                "{a}"
            );
        }
        // The switches.
        let a = "Mandragora is dead [2]. Harmonie states that 'she's dead' in reference to The Leader.";
        assert!(check(a, "Is Mandragora alive or dead?", &stormwatch(), &t, false, false).is_none());
        let (_, f) = check(a, "q", &stormwatch(), &t, false, true).unwrap();
        assert!(f.fates.is_empty() && f.quotes.len() == 1);
    }

    #[test]
    fn speaker_labels_and_quotes() {
        assert_eq!(
            speaker_and_text("'The Leader': For Dublinn!"),
            (Some("The Leader"), "For Dublinn!")
        );
        assert_eq!(speaker_and_text("She says: no."), (None, "She says: no."));
        let s = "Harmonie states that 'she's dead' here, and \u{201c}go now\u{201d} too.";
        let q: Vec<&str> = quote_spans(s).into_iter().map(|(a, b)| &s[a..b]).collect();
        assert_eq!(q, vec!["she's dead", "go now"]);
        assert!(quote_spans("Mandragora's sister isn't here.").is_empty());
    }
}
