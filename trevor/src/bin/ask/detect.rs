//! Keyword tests on the question text that decide which answer rules and passage sources apply. None of
//! them calls a model.

use trevor::tools::{contains_words, norm};

pub(crate) fn has(q: &str, words: &[&str]) -> bool {
    let q = q.to_lowercase();
    words.iter().any(|w| q.contains(w))
}

pub(crate) fn is_time_question(q: &str) -> bool {
    has(q, &["chronolog", "what year", "which year", " year", "when did", "when was", "when is", "how long ago",
             "before or after", "in order", "order the", "timeline", "date"])
}

pub(crate) fn is_canon_question(q: &str) -> bool {
    has(q, &["canon", "official", "retcon", "mistranslat", "translation", "translated", "cn version", "en version", "localiz",
             "developer", "the devs", "hypergryph said", "writer"])
}

pub(crate) fn is_opinion_question(q: &str) -> bool {
    has(q, &["do you think", "what do you think", "who would win", "would win", "strongest", "most powerful", "stronger than",
             "predict", "prediction", "will happen", "theory", "theories", "speculat", "your opinion", "overrated",
             "underrated", "favorite", "favourite"])
}

/// The P4 source kind a question names, if any.
pub(crate) fn source_kind(q: &str) -> Option<&'static str> {
    [("module", &["module"][..]),
     ("voice", &["voice line", "voiceline", "voice lines", "poke", "trust line", "interact line", "dorm line", "says when"][..]),
     ("skin", &["skin", "outfit"][..]),
     ("is", &["integrated strategies", "roguelike", " is1", " is2", " is3", " is4", " is5", " is6", "is#"][..]),
     ("enemy", &["enemy file", "enemy description", "enemy entry", "enemy handbook"][..]),
     ("item", &["item description", "item text", "flavor text", "flavour text"][..])]
        .into_iter().find(|(_, ws)| has(&format!(" {q}"), ws)).map(|(k, _)| k)
}

pub(crate) fn is_source_question(q: &str) -> bool {
    has(q, &["module", "voice line", "voiceline", "voice lines", "skin", "outfit", "integrated strategies", "roguelike",
             " is1", " is2", " is3", " is4", " is5", " is6", "is#", "enemy file", "enemy description", "enemy entry",
             "operator record", "trust line", "interact line", "poke", "dorm line", "furniture"])
}

pub(crate) fn is_synthesis_question(q: &str) -> bool {
    has(q, &["rank", "strongest", "most powerful", "weakest", "compare", "comparison", "stronger", "overview",
             "in general", "summari"])
}

/// A question asking who someone is or which person did something, or naming a real name, codename or identity: the
/// questions identity notes are for (on every question they fired on 61 of 336 earlier answers, 41 of them trivia
/// questions about one operator's file; with this test, 5).
pub(crate) fn asks_identity(q: &str) -> bool {
    let n = norm(q);
    ["who", "whom", "whose", "which", "real name", "codename", "identity", "same person", "known as"].iter()
        .any(|w| contains_words(&n, w))
}

/// Words that make a named character the subject of a said or done act ("Mon3tr suggested", "did Necrass say").
pub(crate) const SAID_VERBS: [&str; 39] = ["say", "said", "says", "tell", "told", "tells", "suggest", "suggested", "suggests", "declare",
    "declared", "declares", "joke", "joked", "jokes", "admit", "admitted", "admits", "threaten", "threatened", "threatens",
    "propose", "proposed", "proposes", "offer", "offered", "offers", "mention", "mentioned", "mentions", "claim", "claimed",
    "claims", "promise", "promised", "promises", "confess", "confessed", "confesses"];

/// Whether the question makes the character (any of `forms`, folded) the subject of an act it asks about: a form followed,
/// with at most one word between, by a said verb (`SAID_VERBS`), or, when the question asks in which event, story,
/// chapter, episode or scene, by a said verb or a past-tense word ("During which event Mon3tr suggested ...").
pub(crate) fn asks_scene(q: &str, forms: &[String]) -> bool {
    let n = norm(&q.replace('\u{2019}', "'"));
    let words: Vec<&str> = n.split_whitespace().collect();
    let l = format!(" {} ", words.join(" "));
    let place = ["event", "story", "chapter", "episode", "scene", "stage"];
    let asks_place = ["which", "what"].iter().any(|w| place.iter().any(|p| l.contains(&format!(" {w} {p} "))));
    forms.iter().any(|f| {
        let fw: Vec<&str> = f.split_whitespace().collect();
        (0..words.len()).filter(|&i| !fw.is_empty() && words[i..].starts_with(&fw)).any(|i| {
            words.iter().skip(i + fw.len()).take(2).any(|w| SAID_VERBS.contains(w) || (asks_place && w.len() > 4 && w.ends_with("ed")))
        })
    })
}

/// Whether a relation question asks whom someone admires (one person named, the other asked for).
pub(crate) fn asks_admire(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" fan of ", " idolise ", " idolises ", " idolize ", " idolizes ", " idolised ", " idolized ", " admire ", " admires ",
     " look up to ", " looks up to "].iter().any(|w| ql.contains(w))
}

/// "What is the khagan quest?", "what are the X", "who are the Sarkaz?": a what/who is/are question about a thing of at
/// most 4 words, no possessive (a question about one character's trait is not a definition).
pub(crate) fn asks_term(q: &str) -> bool {
    let l = q.trim().to_lowercase().replace('’', "'");
    let rest = ["what is ", "what are ", "what was ", "what were ", "what's ", "whats ", "who are the ", "who were the "]
        .iter().find_map(|p| l.strip_prefix(p));
    rest.is_some_and(|r| {
        let r = r.trim_end_matches(['?', '.', '!', ' ']);
        !r.contains("'s") && !r.contains('?') && r.split_whitespace().count() <= 4
    })
}

/// The term of a what-is question (`asks_term`), without its article: "What is the khagan quest?" -> "khagan quest".
pub(crate) fn term_of(q: &str) -> Option<String> {
    if !asks_term(q) {
        return None;
    }
    let l = q.trim().to_lowercase().replace('’', "'");
    let rest = ["what is ", "what are ", "what was ", "what were ", "what's ", "whats ", "who are the ", "who were the "]
        .iter().find_map(|p| l.strip_prefix(p))?;
    let rest = rest.trim_end_matches(['?', '.', '!', ' ']);
    let rest = ["the ", "a ", "an "].iter().find_map(|x| rest.strip_prefix(x)).unwrap_or(rest);
    Some(rest.trim().to_owned()).filter(|t| !t.is_empty())
}

pub(crate) fn is_listing_question(q: &str) -> bool {
    has(q, &["every story", "every event", "all stories", "all the stories", "all events", "all the events",
             "full timeline", "complete timeline", "whole timeline", "entire timeline", "every chapter", "all chapters"])
}

pub(crate) fn is_operator_status_question(q: &str) -> bool {
    has(q, &["operator"]) && has(q, &["dead", "died", "die ", "dies", "death", "killed", "deceased"])
}

// ---------------------------------------------------------------------------------------------------------------
// The keyword router (`--router keywords`, the kill switch): today's parsing of each table route, unchanged, now
// returning a tool and its arguments; the text comes from `trevor::tools`. A parser that matches but whose tool
// fails falls through to the next parser, as each route's `None` did before.

/// "List all X of every operator" with no table behind it: retrieval sees a sample, and Ian's real-names
/// question (2026-09-28) got 7 names from 17 passages, presented as the answer.
pub(crate) fn is_open_listing(q: &str) -> bool {
    has(q, &["list all", "list every", "list the", "all the ", "every ", "all known", "complete list", "full list",
             "name all", "all of the", "which operators", "which characters"])
        && has(q, &["operator", "character", "people", "names", "members", "who "])
}

pub(crate) fn is_death_question(q: &str) -> bool {
    has(q, &["dead", "died", "die ", "dies", "die?", "death", "killed", "deceased", "passed away"])
}

/// Whether a question asks someone's age ("how old", "age", "aged", "years old").
pub(crate) fn asks_age(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" how old ", " age ", " aged ", " years old ", " ages "].iter().any(|w| ql.contains(w))
}

/// Whether a question asks why, when or how something began (a why/when/how word and a beginning word).
pub(crate) fn asks_origin(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    let wh = [" why ", " when ", " how "].iter().any(|w| ql.contains(w));
    wh && [" start ", " started ", " starts ", " begin ", " began ", " begun ", " beginning ", " origin ", " origins ",
           " originate ", " originated ", " founded ", " founding ", " join ", " joined ", " joins ", " become ", " became ",
           " decide ", " decided ", " first "].iter().any(|w| ql.contains(w))
}

/// Whether a question asks how people are related, or whom someone admires.
pub(crate) fn asks_relation(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" relationship ", " relationships ", " relation ", " related ", " relatives ", " fan of ", " idolise ", " idolises ",
     " idolize ", " idolizes ", " idolised ", " idolized ", " admire ", " admires ", " look up to ", " looks up to "]
        .iter().any(|w| ql.contains(w))
}

/// Whether a question asks when something happened ("when", "what year", "which year", "how long ago").
pub(crate) fn asks_when(q: &str) -> bool {
    let ql = format!(" {} ", q.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '\'', " "));
    [" when ", " what year ", " which year ", " how long ago "].iter().any(|w| ql.contains(w))
}

/// An answer whose first sentence says the passages do not specify or identify what was asked ("The story text does not
/// specify which non-playable Rhodes Island Operator visited Iris's castle of dreams"): a decline for the evidence
/// composition, which `declines` (the retry's and the typed fallback's test) does not count.
pub(crate) fn leaves_open(answer: &str) -> bool {
    let first = answer.trim().split(['.', '!', '?']).next().unwrap_or_default().to_lowercase();
    ["does not specify", "do not specify", "does not identify", "do not identify", "does not name", "do not name"].iter().any(|w| first.contains(w))
}

/// A decline opens the answer ("I could not find it in the story text"), the rule `answer-eval.py` uses.
pub(crate) fn declines(answer: &str) -> bool {
    let first = answer.trim().split(['.', '!', '?']).next().unwrap_or_default().to_lowercase();
    ["could not find", "couldn't find", "cannot find", "can't find", "no information", "not mentioned", "not stated",
     "does not say", "does not state", "does not mention", "do not contain", "does not contain"].iter().any(|w| first.contains(w))
}
