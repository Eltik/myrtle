//! The part-aware recap (`--no-event-part-recap`): a question naming an event and a part of it ("What's going on at
//! the beginning of Ch 10?", "How does Near Light conclude?") gets the summary of the event's first or last story.
//! Before it, such a question had no route (the recap rule's leftover guard counted the part word) and went to
//! retrieval, which declined on E2B mid.

use trevor::tools::{RecapPart, Route, Tools, norm};

use crate::cli::Args;
use super::rules::{leftover, named_group, recap_route};

/// Story-unit nouns that "first" or "last" turn into a part ("the first chapter of Near Light").
const UNITS: [&str; 8] = ["chapter", "chapters", "stage", "stages", "story", "stories", "part", "act"];

/// The part of an event a question names: beginning words (beginning, start, opening, prologue, first chapter or
/// stage) or end words (end, ending, finale, conclude, last chapter or stage), as whole words of the folded question;
/// None when it names neither, or both ("from the beginning to the end").
pub(crate) fn question_part(q: &str) -> Option<RecapPart> {
    let n = norm(q);
    let t: Vec<&str> = n.split(' ').collect();
    let unit_after = |w: &str| t.windows(2).any(|p| p[0] == w && UNITS.contains(&p[1]));
    let beg = t.iter().any(|w| matches!(*w, "beginning" | "start" | "opening" | "prologue")) || unit_after("first");
    let end = t.iter().any(|w| matches!(*w, "end" | "ending" | "finale" | "conclusion" | "conclude" | "concludes"
                                            | "concluded"))
        || unit_after("last");
    match (beg, end) {
        (true, false) => Some(RecapPart::Beginning),
        (false, true) => Some(RecapPart::End),
        _ => None,
    }
}

/// The main episode a question names by number ("Episode 10", "chapter 10", "Ch 10", "ep10"), with the words that
/// named it.
fn episode<'a>(q: &str, t: &'a Tools) -> Option<(&'a serde_json::Value, Vec<String>)> {
    let n = norm(q);
    let w: Vec<&str> = n.split(' ').collect();
    let num = w.iter().enumerate().find_map(|(i, x)| {
        if matches!(*x, "episode" | "chapter" | "ch" | "ep") {
            let d = w.get(i + 1)?;
            return d.parse::<u64>().ok().map(|k| (k, vec![(*x).to_owned(), (*d).to_owned()]));
        }
        ["episode", "chapter", "ch", "ep"].iter()
            .find_map(|p| x.strip_prefix(p).and_then(|d| d.parse::<u64>().ok()))
            .map(|k| (k, vec![(*x).to_owned()]))
    })?;
    t.groups.values().find(|g| g["chapter"].as_u64() == Some(num.0)).map(|g| (g, num.1))
}

/// The part-aware recap route, for every router: an event named by number or name, one part word, and nothing else
/// asked (the recap rule's leftover guard, with the part words and "going on" allowed). A question the recap rule
/// already takes ("Explain the ending of Invitation to Wine": the event summary and its last story) and an
/// Integrated Strategies ending question keep their routes.
pub(crate) fn part_recap_route(q: &str, a: &Args, t: &Tools) -> Option<Route> {
    if a.no_event_part_recap || t.is_ending_question(q).is_some() || recap_route(q, t).is_some() {
        return None;
    }
    let part = question_part(q)?;
    let (g, mut used) = match episode(q, t) {
        Some(e) => e,
        None => (named_group(q, t).or_else(|| t.event_in_question(q))?, Vec::new()),
    };
    let id = g["groupId"].as_str()?;
    let name = g["name"].as_str().unwrap_or(id);
    used.extend(name.replace(['?', ',', '.', '!', '\'', '"'], " ").split_whitespace().map(str::to_owned));
    if let Some(n) = g["chapter"].as_u64() {
        used.push(n.to_string());
    }
    used.extend(["going", "on", "happens", "happened", "happen", "goes", "go", "went", "beginning", "start",
                 "starts", "opening", "prologue", "first", "last", "chapter", "stage", "story", "part", "act",
                 "episode", "ending", "ends", "ended", "finale", "conclusion", "conclude", "concludes", "concluded",
                 "explain", "summary", "summarize", "summarise", "recap", "tell", "me", "about", "can", "someone",
                 "you", "please", "give", "event", "spoiler", "spoilers", "i", "for", "this", "that", "plot",
                 "during", "like"].iter().map(|w| (*w).to_owned()));
    if leftover(q, &used) > 0 {
        return None;
    }
    Some(Route::new("recap", &[("event", id), ("part", part.as_str())]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part_words_name_one_part() {
        let p = question_part;
        assert_eq!(p("What's going on at the beginning of Ch 10?"), Some(RecapPart::Beginning));
        assert_eq!(p("What happens at the start of Dorothy's Vision?"), Some(RecapPart::Beginning));
        assert_eq!(p("How does the first chapter of Near Light go?"), Some(RecapPart::Beginning));
        assert_eq!(p("How does Episode 10 end?"), Some(RecapPart::End));
        assert_eq!(p("How does Near Light conclude?"), Some(RecapPart::End));
        assert_eq!(p("What happens in the last stage of Lone Trail?"), Some(RecapPart::End));
        assert_eq!(p("What happens in Near Light?"), None);
        assert_eq!(p("Who is the first operator in Near Light?"), None, "first without a story unit");
        assert_eq!(p("Recap Near Light from the beginning to the end"), None, "both parts");
        assert!(p("Where does the Endless Winter start?").is_some(), "a part word alone; the event decides");
    }

    /// The shipped tables, when present (artifacts/ is git-ignored, so this passes vacuously without them).
    #[test]
    fn event_and_part_questions_reach_the_part_recap() {
        let t = Tools::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        if t.bank.is_empty() || t.groups.is_empty() {
            return;
        }
        let parse = |v: &[&str]| <Args as clap::Parser>::parse_from(std::iter::once("ask").chain(v.iter().copied()));
        let a = parse(&[]);
        let r = |q: &str| part_recap_route(q, &a, &t).map(|r| (r.args["event"].clone(), r.args["part"].clone()));
        let want = |e: &str, p: &str| Some((e.to_owned(), p.to_owned()));
        assert_eq!(r("What's going on at the beginning of Ch 10?"), want("main_10", "beginning"));
        assert_eq!(r("How does Episode 10 end?"), want("main_10", "end"));
        assert_eq!(r("What happens at the start of Dorothy's Vision?"), want("act19side", "beginning"));
        assert_eq!(r("How does Near Light conclude?"), want("act13side", "end"));
        assert_eq!(r("What happens at the end of the world in Arknights?"), None, "no event");
        assert_eq!(r("Did Lappland die at the end of Il Siracusano?"), None, "a question about one character");
        assert_eq!(r("Which operators have we met by the end of chapter 8?"), None);
        assert_eq!(r("Explain the ending of Invitation to Wine"), None, "the recap rule's ending route keeps it");
        let off = parse(&["--no-event-part-recap"]);
        assert_eq!(part_recap_route("How does Episode 10 end?", &off, &t), None);
    }
}
