//! Time-gap evidence (the default; `--no-time-gap`): a question that asks how much in-world time passes between two
//! stories it names ("What is the time skip from chapter 8 to chapter 9?") gets, before the retrieved passages, the
//! lines of the later story that state elapsed time and name something of the earlier one. No year is stated for
//! Episodes 6 to 9 (the timeline puts all four "at most 1098"), so the timeline notes cannot answer it, and the
//! whole-corpus search does not rank Kal'tsit's "eleven months and twenty-seven days, since the end of the Chernobog
//! incident" (Episode 9) for a question that says neither "Chernobog" nor "months".

use std::collections::BTreeSet;

use serde_json::Value;
use trevor::search::runtime::Runtime;
use trevor::tools::Tools;

use crate::chrono::Chrono;
use crate::detect::has;
use crate::kinds::is_generated;

/// An elapsed-time question: "time skip", "time gap", "how long", "how many years", "years pass", ...
pub(crate) fn asks_time_gap(q: &str) -> bool {
    has(
        q,
        &[
            "time skip",
            "timeskip",
            "time-skip",
            "time gap",
            "time jump",
            "how long",
            "how much time",
            "how many years",
            "how many months",
            "how many weeks",
            "how many days",
            "years pass",
            "months pass",
            "time pass",
            "time has passed",
            "time elapse",
            "years later",
            "months later",
            "years after",
            "months after",
        ],
    )
}

/// The two stories of a time-gap question, earlier first by Trevor's chronology (storyline year, then episode, then
/// release): None unless the question asks for elapsed time and names at least two main episodes or events.
pub(crate) fn time_gap_events<'a>(q: &str, tools: &'a Tools, chrono: &Chrono) -> Option<(&'a Value, &'a Value)> {
    if !asks_time_gap(q) {
        return None;
    }
    let ev = tools.events_in_question(q);
    let [a, b, ..] = ev.as_slice() else { return None };
    let key = |g: &Value| {
        let id = g["groupId"].as_str().unwrap_or_default();
        let c = chrono.groups.get(id);
        (
            c.and_then(|c| c["storylineYear"].as_f64()).unwrap_or(9999.0),
            g["chapter"]
                .as_u64()
                .or_else(|| c.and_then(|c| c["chapter"].as_u64()))
                .unwrap_or(0),
            g["releaseTime"]
                .as_i64()
                .or_else(|| c.and_then(|c| c["releaseTime"].as_i64()))
                .unwrap_or(0),
        )
    };
    let (ka, kb) = (key(a), key(b));
    let later =
        kb.0.total_cmp(&ka.0)
            .then(kb.1.cmp(&ka.1))
            .then(kb.2.cmp(&ka.2))
            .is_ge();
    Some(if later { (*a, *b) } else { (*b, *a) })
}

/// Words that count a span of time ("eleven months", "a few years", "half a year").
const COUNT_WORDS: &[&str] = &[
    "a", "an", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve",
    "fifteen", "twenty", "thirty", "forty", "fifty", "hundred", "several", "few", "half", "dozen", "many", "some",
    "couple",
];
const UNIT_WORDS: &[&str] = &[
    "year", "years", "month", "months", "week", "weeks", "day", "days", "decade", "decades",
];
/// Words that make a span elapsed time rather than a duration of something else ("since", "ago", "passed").
const ELAPSED_WORDS: &[&str] = &["since", "ago", "passed", "later", "been", "past", "after", "gone"];

/// Whether a line states elapsed time: a count word (or a number) at most 3 words before a unit word, and an elapsed
/// word anywhere in the line.
pub(crate) fn states_elapsed(line: &str) -> bool {
    let w: Vec<String> = line
        .split(|c: char| !c.is_alphanumeric())
        .filter(|x| !x.is_empty())
        .map(str::to_lowercase)
        .collect();
    let span = w.iter().enumerate().any(|(i, x)| {
        UNIT_WORDS.contains(&x.as_str())
            && w[i.saturating_sub(3)..i]
                .iter()
                .any(|y| COUNT_WORDS.contains(&y.as_str()) || y.chars().all(|c| c.is_ascii_digit()))
    });
    span && w.iter().any(|x| ELAPSED_WORDS.contains(&x.as_str()))
}

/// Capitalized words of 4 letters or more, as written; with `inside`, only those that do not begin a sentence or a
/// speaker's line ("After every offence", "Kal'tsit: Ashes"), so a summary's marks are names, not sentence openers.
fn capitalized(text: &str, inside: bool) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut opener = true;
    for tok in text.split_whitespace() {
        let x = tok
            .trim_matches(|c: char| !c.is_alphanumeric())
            .trim_end_matches("'s")
            .trim_end_matches('\'');
        let name = x.chars().count() >= 4
            && x.chars().next().is_some_and(char::is_uppercase)
            && x.chars().skip(1).all(|c| c.is_lowercase() || c == '\'');
        if name && !(inside && opener) {
            out.insert(x.to_owned());
        }
        opener = tok.ends_with(['.', '!', '?', ':', '\u{2014}']) || tok.ends_with("...");
    }
    out
}

/// Words that make a span elapsed time for certain ("since the end of", "ago", "passed"); a line with one ranks first.
const STRONG_ELAPSED: &[&str] = &["since", "ago", "passed", "later"];

/// At most this many lines of evidence, each kept with `GAP_WINDOW` lines either side. With 3, E2B (mid) quoted the
/// second and third lines ("Several months later...", "A few months? Half a year?") instead of the first.
pub(crate) const GAP_LINES: usize = 1;
pub(crate) const GAP_WINDOW: usize = 1;

/// The time-gap evidence for a question naming stories `earlier` and `later`: the lines of the later story's chunks that
/// state elapsed time (`states_elapsed`) and name a capitalized word of the earlier story's event summary that the later
/// one's summary does not hold and that does not speak in the later story ("Chernobog" for Episodes 8 and 9; "Rhodes
/// Island" is in both summaries, Amiya speaks in both), best first (most such
/// words, then story order; a line with a sure elapsed word, "since", "ago", "passed" or "later", before any other), at
/// most `GAP_LINES`. Returns (row, line index) pairs; empty when no line qualifies, so a
/// question whose stories state no gap is answered as before.
pub(crate) fn time_gap_lines(rt: &Runtime, tools: &Tools, earlier: &Value, later: &Value) -> Vec<(usize, usize)> {
    let summary = |g: &Value| {
        let id = format!("event:{}", g["groupId"].as_str().unwrap_or_default());
        tools
            .bank
            .iter()
            .find(|r| r["id"].as_str() == Some(id.as_str()))
            .and_then(|r| r["answer"].as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let (sa, sb) = (
        capitalized(&summary(earlier), true),
        capitalized(&summary(later), false),
    );
    let gid = later["groupId"].as_str().unwrap_or_default();
    // A character who speaks in the later story ("Amiya") is no mark of the earlier one.
    let speakers: BTreeSet<&str> = rt
        .store
        .chunks
        .iter()
        .filter(|c| c.group_id == gid)
        .flat_map(|c| c.speakers.iter().map(String::as_str))
        .collect();
    let marks: BTreeSet<String> = sa
        .difference(&sb)
        .filter(|m| !speakers.contains(m.as_str()))
        .cloned()
        .collect();
    if marks.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(bool, usize, usize, usize)> = Vec::new();
    for (r, c) in rt
        .store
        .chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| c.group_id == gid && !is_generated(&c.group_id))
    {
        for (i, line) in c.text.lines().enumerate().filter(|(_, l)| states_elapsed(l)) {
            let n = capitalized(line, false).intersection(&marks).count();
            if n > 0 {
                let strong = line
                    .split(|ch: char| !ch.is_alphanumeric())
                    .any(|w| STRONG_ELAPSED.contains(&w.to_lowercase().as_str()));
                hits.push((strong, n, r, i));
            }
        }
    }
    hits.sort_by(|x, y| {
        y.0.cmp(&x.0)
            .then(y.1.cmp(&x.1))
            .then(x.2.cmp(&y.2))
            .then(x.3.cmp(&y.3))
    });
    hits.into_iter().take(GAP_LINES).map(|(_, _, r, i)| (r, i)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elapsed_lines_and_gap_questions() {
        assert!(states_elapsed(
            "Kal'tsit: There hasn't been an emergency transmission to Rhodes Island's deck for eleven \
            months and twenty-seven days, since the end of the Chernobog incident."
        ));
        assert!(states_elapsed(
            "Siege: A few months ago, Dr. Kal'tsit asked me if I was ready."
        ));
        assert!(!states_elapsed("Horn: Five, four, three... Ready—"));
        assert!(
            !states_elapsed("It took us two days to cross the plains."),
            "a duration without an elapsed word"
        );
        assert!(asks_time_gap("What is the time skip from chapter 8 to chapter 9?"));
        assert!(asks_time_gap("How many years pass between chapter 8 and chapter 9"));
        assert!(!asks_time_gap("What happens in chapter 9?"));
        let t = Tools::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        let c = crate::chrono::load_chrono();
        if !t.groups.is_empty() && !c.groups.is_empty() {
            let ids = |q: &str| time_gap_events(q, &t, &c).map(|(a, b)| (a["groupId"].clone(), b["groupId"].clone()));
            let want = Some((Value::from("main_8"), Value::from("main_9")));
            assert_eq!(ids("What is the time skip from chapter 8 to chapter 9?"), want);
            assert_eq!(
                ids("How many years pass between chapter 9 and chapter 8"),
                want,
                "earlier first"
            );
            assert_eq!(
                ids("What happens between chapter 8 and chapter 9?"),
                None,
                "no elapsed-time words"
            );
        }
        let set = |v: &[&str]| v.iter().map(|x| (*x).to_owned()).collect::<BTreeSet<String>>();
        assert_eq!(
            capitalized("Ashes... the Chernobog incident at Rhodes Island's deck. Who's", false),
            set(&["Ashes", "Chernobog", "Island", "Rhodes"])
        );
        assert_eq!(
            capitalized("After the battle, Amiya and Talulah met. Later Chernobog fell.", true),
            set(&["Amiya", "Chernobog", "Talulah"])
        );
    }
}
