//! Arguments by dictionary match: the name tables a kNN-chosen tool's arguments are read from.

use std::collections::BTreeMap;

use crate::tools::{CLASSES, Route, Tools, contains_words, norm};


/// Name dictionaries for argument extraction, normalized once.
pub struct Dict {
    events: Vec<(String, String)>,
    speakers: Vec<(String, String)>,
    dead: Vec<(String, String)>,
    operators: Vec<(String, String)>,
    places: Vec<(String, String)>,
    races: Vec<(String, String)>,
    branches: Vec<(String, String)>,
    topics: Vec<(String, String)>,
    runs: Vec<(String, String)>,
}

pub(crate) fn normed<I: IntoIterator<Item = String>>(it: I, min_len: usize) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = it.into_iter().map(|x| (norm(&x), x)).filter(|(n, _)| n.chars().count() >= min_len).collect();
    v.sort();
    v.dedup();
    v
}

impl Dict {
    #[must_use]
    pub fn new(t: &Tools) -> Self {
        let idents: Vec<String> = t.identities.iter().flatten().cloned().collect();
        let classes: Vec<String> = CLASSES.iter().map(|c| c.to_lowercase()).collect();
        Self {
            events: normed(t.groups.values().filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
                .filter_map(|g| g["name"].as_str().map(str::to_owned)), 4),
            // Speakers as the keyword route took them: capitalized, 3 characters or more ("Medic" and "Guard"
            // are speakers too, so a class word never counts as a name here).
            speakers: normed(t.speaker_names().into_iter().filter(|n| n.chars().next().is_some_and(char::is_uppercase))
                .map(str::to_owned).chain(idents.iter().cloned()), 3)
                .into_iter().filter(|(n, _)| !classes.iter().any(|c| n == c || *n == format!("{c}s"))).collect(),
            dead: normed(t.death_names().into_iter().chain(idents.iter().cloned()), 1),
            operators: normed(t.operator_names().into_iter().map(str::to_owned)
                .chain(t.real_names.iter().flatten().filter_map(|r| r["name"].as_str().map(str::to_owned)))
                .chain(idents.iter().cloned()), 1),
            places: normed(t.places(), 3),
            races: normed(t.attr_values("race"), 3),
            branches: normed(t.attr_values("branch").into_iter()
                .filter(|b| !classes.contains(&b.to_lowercase())), 3),
            topics: normed(t.topics.iter().flatten().flat_map(|r| std::iter::once(r["topic"].as_str().unwrap_or_default().to_owned())
                .chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned)))), 3),
            runs: normed(t.canon.as_ref().and_then(|c| c["runs"].as_array()).into_iter().flatten()
                .filter_map(|r| r["runName"].as_str().map(str::to_owned)), 3),
        }
    }
}

/// The longest dictionary entry the normalized question holds as whole words (a plural "s" allowed).
pub(crate) fn longest<'a>(nq: &str, dict: &'a [(String, String)]) -> Option<&'a str> {
    dict.iter().filter(|(n, _)| contains_words(nq, n) || contains_words(nq, &format!("{n}s")))
        .max_by_key(|(n, _)| n.len()).map(|(_, orig)| orig.as_str())
}

/// "episode 7", "chapter 14", "ep 7", "ch14" in a normalized question.
pub(crate) fn episode_in(nq: &str) -> Option<String> {
    let t: Vec<&str> = nq.split(' ').collect();
    for (i, w) in t.iter().enumerate() {
        if matches!(*w, "episode" | "chapter" | "ep" | "ch") {
            if let Some(n) = t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) {
                return Some(format!("Episode {n}"));
            }
        }
        for p in ["episode", "chapter", "ep", "ch"] {
            if let Some(n) = w.strip_prefix(p).and_then(|x| x.parse::<u64>().ok()) {
                return Some(format!("Episode {n}"));
            }
        }
    }
    None
}

pub(crate) fn any_word(nq: &str, words: &[&str]) -> bool {
    words.iter().any(|w| contains_words(nq, w))
}

/// Operator attribute fields by the words that ask for them, as the keyword route has them.
pub(crate) const FIELD_WORDS: &[(&str, &[&str])] = &[
    ("race", &["race", "species"]), ("birthplace", &["born", "birthplace", "place of birth", "hometown"]),
    ("birthday", &["birthday", "date of birth"]), ("height", &["height", "how tall", "tall"]),
    ("infection", &["infected", "infection", "oripathy"]), ("class", &["class", "archetype"]),
    ("branch", &["branch", "subclass"]), ("rarity", &["rarity", "how many stars", "star", "stars"]),
    ("gender", &["gender"]), ("affiliation", &["affiliation", "faction", "belong", "nation"]),
];

/// The filters of an `operator_filter` question by dictionary match: the keyword route's detection without its
/// guards (the classifier has already decided the question is a roster question).
#[must_use]
pub fn filter_args(q: &str, d: &Dict) -> BTreeMap<String, String> {
    let nq = norm(q);
    let raw = format!(" {} ", q.to_lowercase());
    let mut a = BTreeMap::new();
    let place = longest(&nq, &d.places).map(str::to_owned);
    if let Some(p) = &place {
        a.insert("place".into(), p.clone());
    }
    // "Ursus" is a nation and a race; a word taken as the place is not also a race filter.
    if let Some(r) = longest(&nq, &d.races).filter(|r| place.as_deref() != Some(*r)) {
        a.insert("race".into(), r.to_owned());
    }
    let classes: Vec<&str> = CLASSES.iter().filter(|c| any_word(&nq, &[&c.to_lowercase(), &format!("{}s", c.to_lowercase())])).copied().collect();
    if !classes.is_empty() {
        a.insert("class".into(), classes.join(","));
    }
    if let Some(b) = longest(&nq, &d.branches) {
        a.insert("branch".into(), b.to_owned());
    }
    if let Some(n) = (1..=6).find(|n| raw.contains(&format!(" {n}-star")) || raw.contains(&format!(" {n} star")) || raw.contains(&format!(" {n}*"))) {
        a.insert("rarity".into(), n.to_string());
    }
    if any_word(&nq, &["female", "women", "woman", "girls"]) {
        a.insert("gender".into(), "female".into());
    } else if any_word(&nq, &["male", "men", "man", "boys"]) {
        a.insert("gender".into(), "male".into());
    }
    if any_word(&nq, &["uninfected", "non infected", "not infected"]) {
        a.insert("infected".into(), "false".into());
    } else if any_word(&nq, &["infected"]) {
        a.insert("infected".into(), "true".into());
    }
    if any_word(&nq, &["tallest"]) {
        a.insert("sort".into(), "tallest".into());
    } else if any_word(&nq, &["shortest"]) {
        a.insert("sort".into(), "shortest".into());
    }
    a
}

/// The arguments `tool` needs, by dictionary match against the tables (longest whole-word match), or None when a
/// required one is not in the question.
#[must_use]
pub fn extract_args(tool: &str, q: &str, d: &Dict) -> Option<Route> {
    let nq = norm(q);
    let event = || episode_in(&nq).or_else(|| longest(&nq, &d.events).map(str::to_owned));
    let r = |args: Vec<(&str, String)>| Some(Route { tool: tool.to_owned(), args: args.into_iter().map(|(k, v)| (k.to_owned(), v)).collect() });
    match tool {
        "reading_guide" | "reading_chronological" | "timeline_all" | "dead_operators" => r(vec![]),
        "reading_event" | "reading_time" => r(vec![("event", event()?)]),
        "reading_compare" => {
            // Every event the question names: episodes by number, events by their longest names, first two in order.
            let mut found: Vec<(usize, String)> = Vec::new();
            let t: Vec<&str> = nq.split(' ').collect();
            for (i, w) in t.iter().enumerate() {
                let n = if matches!(*w, "episode" | "chapter" | "ep" | "ch") { t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) }
                    else { ["episode", "chapter", "ep", "ch"].iter().find_map(|p| w.strip_prefix(p).and_then(|x| x.parse::<u64>().ok())) };
                if let Some(n) = n {
                    found.push((nq.split(' ').take(i).map(|x| x.len() + 1).sum(), format!("Episode {n}")));
                }
            }
            let hay = format!(" {nq} ");
            for (n, orig) in &d.events {
                if let Some(at) = hay.find(&format!(" {n} ")) {
                    if !found.iter().any(|(_, o)| o == orig) {
                        found.push((at, orig.clone()));
                    }
                }
            }
            found.sort();
            found.dedup_by(|a, b| a.1 == b.1);
            if found.len() < 2 {
                return None;
            }
            r(vec![("event", found[0].1.clone()), ("other", found[1].1.clone())])
        }
        "recap" => {
            let ending = any_word(&nq, &["ending", "end of", "how does it end", "how did it end", "end"]);
            r(vec![("event", event()?), ("ending", ending.to_string())])
        }
        "deaths_in_event" => {
            let who = if any_word(&nq, &["non playable", "nonplayable", "npc", "npcs", "not playable"]) { "npc" }
                else if any_word(&nq, &["operator", "operators", "playable"]) { "operators" } else { "all" };
            r(vec![("event", event()?), ("who", who.into())])
        }
        "first_appearance" => r(vec![("character", longest(&nq, &d.speakers)?.to_owned())]),
        "death_of" => r(vec![("character", longest(&nq, &d.dead)?.to_owned())]),
        "real_name" => r(vec![("operator", longest(&nq, &d.operators)?.to_owned())]),
        "real_names" => r(longest(&nq, &d.places).map(|p| vec![("place", p.to_owned())]).unwrap_or_default()),
        "operator_attribute" => {
            let field = FIELD_WORDS.iter().find(|(_, ws)| any_word(&nq, ws)).map(|(f, _)| *f)?;
            r(vec![("operator", longest(&nq, &d.operators)?.to_owned()), ("field", field.into())])
        }
        "canon" => {
            // "IS3", "IS #3", "IS 3", "Integrated Strategies 3"; else a run name; else every run.
            let t: Vec<&str> = nq.split(' ').collect();
            let num = t.iter().enumerate().find_map(|(i, w)| {
                let after = |j: usize| t.get(j).and_then(|x| x.parse::<u64>().ok());
                w.strip_prefix("is").and_then(|x| x.parse::<u64>().ok())
                    .or_else(|| (*w == "is" || *w == "strategies").then(|| after(i + 1)).flatten())
            });
            match num.map(|n| format!("IS{n}")).or_else(|| longest(&nq, &d.runs).map(str::to_owned)) {
                Some(run) => r(vec![("run", run)]),
                None => r(vec![]),
            }
        }
        "topic" => r(vec![("topic", longest(&nq, &d.topics)?.to_owned())]),
        "operator_filter" => {
            let a = filter_args(q, d);
            if a.is_empty() {
                return None;
            }
            Some(Route { tool: tool.to_owned(), args: a })
        }
        _ => None,
    }
}
