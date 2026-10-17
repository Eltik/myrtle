//! The keyword rules: whole-corpus questions answered from Trevor's tables (reading order, recaps, listings, deaths,
//! real names, operator attributes), each rule a parser that returns a route or None.

use trevor::tools::{Route, Tools};

use crate::detect::{
    has, is_death_question, is_listing_question, is_open_listing, is_operator_status_question, is_time_question,
};

/// Whole-corpus questions answered from Trevor's tables instead of a few passages, which cannot list the whole corpus
/// ("every story in chronological order").
pub(crate) fn keyword_route(q: &str, t: &Tools) -> Option<(Route, String)> {
    let parsers: [fn(&str, &Tools) -> Option<Route>; 6] =
        [reading_route, recap_route, listing_route, death_route, real_name_route, attribute_route];
    parsers.iter().filter_map(|p| p(q, t)).find_map(|r| t.run(&r).ok().map(|text| (r, text)))
}

pub(crate) fn listing_route(q: &str, _: &Tools) -> Option<Route> {
    is_listing_question(q).then(|| Route::new("timeline_all", &[]))
}

pub(crate) fn gid(g: &serde_json::Value) -> &str {
    g["groupId"].as_str().unwrap_or_default()
}

/// The place a question names (an affiliation or birthplace in the attribute table).
pub(crate) fn place_filter(q: &str, t: &Tools) -> Option<String> {
    let rows = t.attributes.as_ref()?;
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\''], " "));
    let keys = ["nation", "group", "team", "birthplace"];
    rows.iter().flat_map(|r| keys.iter().filter_map(|k| r[*k].as_str().map(str::to_owned)).collect::<Vec<_>>())
        .filter(|p| p.len() >= 3 && !matches!(p.as_str(), "Unknown" | "Undisclosed")
            && ql.contains(&format!(" {} ", p.to_lowercase())))
        .max_by_key(String::len)
}

/// Roster questions ("Which operators are Sarkaz?", "How many 6-star Casters are there?", "List all operators from
/// Kazimierz") and one operator's attribute ("What race is Texas?").
pub(crate) fn attribute_route(q: &str, t: &Tools) -> Option<Route> {
    let rows = t.attributes.as_ref()?;
    let s = |r: &serde_json::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\''], " "));
    let word = |w: &str| !w.is_empty() && ql.contains(&format!(" {} ", w.to_lowercase()));
    // Filters found in the question, as tool arguments.
    let mut args: Vec<(&str, String)> = Vec::new();
    let mut used: Vec<String> = Vec::new();
    // "From X": the game's affiliation (nation, group, team) or the handbook's place of birth.
    if let Some(p) = t.places().iter().filter(|p| word(p)).max_by_key(|p| p.len()).cloned() {
        args.push(("place", p.clone()));
        used.push(p);
    }
    // "Ursus" is a nation and a race; a word already taken as the place is not also a race filter.
    if let Some(race) = t.attr_values("race").iter()
        .filter(|v| (word(v) || word(&format!("{v}s"))) && !used.contains(v))
        .max_by_key(|v| v.len()).cloned() {
        args.push(("race", race.clone()));
        used.push(race);
    }
    let mut classes: Vec<&str> = Vec::new();
    for c in trevor::tools::CLASSES {
        if word(c) || word(&format!("{c}s")) {
            classes.push(c);
            used.push(c.to_owned());
        }
    }
    if !classes.is_empty() {
        args.push(("class", classes.join(",")));
    }
    // Branches ("Fortress", "Soloblade"); a branch named like its class ("Medic") is left to the class filter.
    let class_words = ["vanguard", "guard", "defender", "sniper", "caster", "medic", "supporter", "specialist"];
    if let Some(b) = t.attr_values("branch").iter()
        .filter(|b| !class_words.contains(&b.to_lowercase().as_str()) && (word(b) || word(&format!("{b}s"))))
        .max_by_key(|b| b.len()).cloned() {
        args.push(("branch", b.clone()));
        used.push(b);
    }
    if let Some(n) = (1..=6).find(|n| ql.contains(&format!(" {n}-star")) || ql.contains(&format!(" {n} star"))
        || ql.contains(&format!(" {n}*"))) {
        args.push(("rarity", n.to_string()));
        used.push(format!("{n}"));
    }
    if word("female") || word("women") {
        args.push(("gender", "female".into()));
        used.push("female".into());
    } else if word("male") || word("men") {
        args.push(("gender", "male".into()));
        used.push("male".into());
    }
    if word("uninfected") || word("non-infected") || ql.contains(" not infected ") {
        args.push(("infected", "false".into()));
        used.push("infected".into());
    } else if word("infected") {
        args.push(("infected", "true".into()));
        used.push("infected".into());
    }
    // The extra-word guard below keeps a broad trigger safe: "Which female Sankta operators are infected?" has
    // no fixed phrase, and a story question with the same opener keeps more than one content word.
    // Superlatives over height, the one ordered attribute the game data gives every operator (401 of 407).
    let sup = ["tallest", "shortest"].into_iter().find(|w| word(w));
    if let Some(w) = sup {
        used.push(w.to_owned());
    }
    let listing = is_open_listing(q) || ql.starts_with(" which ") || ql.starts_with(" what ") || ql.starts_with(" who ")
        || has(q, &["how many", "list ", "who are the", "name the", "all operators"]);
    if (!args.is_empty() || sup.is_some()) && listing {
        // Only a roster question: after removing the filters and question words no content word may remain,
        // so "Which operators from Rhodes Island went to Londinium?" stays with retrieval.
        let stop = ["which", "what", "who", "how", "many", "are", "is", "there", "the", "a", "an", "all", "list",
                    "of", "in", "from", "operator", "operators", "character", "characters", "every", "known", "with",
                    "and", "or", "star", "one", "person", "people", "branch", "have", "has", "that", "born",
                    "affiliated", "me", "give", "show", "name", "names", "playable", "do", "does", "total", "count",
                    "number", "any"];
        let mut rest = ql.clone();
        for u in &used {
            rest = rest.replace(&format!(" {} ", u.to_lowercase()), " ")
                .replace(&format!(" {}s ", u.to_lowercase()), " ");
        }
        let extra = rest.split_whitespace()
            .filter(|w| !stop.contains(w) && !w.chars().all(|c| c.is_ascii_digit() || c == '-' || c == '*')
                && !w.ends_with("-star"))
            .count();
        // Zero, not one: "Which operators betrayed Rhodes Island?" has one extra word and listed all 84.
        if extra == 0 {
            if let Some(w) = sup {
                args.push(("sort", w.to_owned()));
            }
            return Some(Route {
                tool: "operator_filter".into(),
                args: args.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
            });
        }
    }
    // One operator's attribute: "What race is Texas?", "When is Exusiai's birthday?".
    let asks = [("race", &["race", "species"][..]),
                ("birthplace", &["born", "birthplace", "place of birth", "where is", "hometown"][..]),
                ("birthday", &["birthday", "date of birth"][..]), ("height", &["height", "how tall"][..]),
                ("infection", &["infected", "infection", "oripathy"][..]), ("class", &["class", "archetype"][..]),
                ("branch", &["branch", "subclass"][..]), ("rarity", &["rarity", "how many stars", "star"][..]),
                ("gender", &["gender"][..]), ("affiliation", &["affiliation", "faction", "belong", "nation"][..])];
    // Whole words only: "explanation" contains "nation" and had answered with Mostima's affiliation.
    let (field, kws) = asks.iter().find(|(_, ws)| ws.iter().any(|w| ql.contains(&format!(" {w} "))))?;
    let r = rows.iter().filter(|r| {
        let n = s(r, "name").to_lowercase();
        n.len() >= 2 && ql.contains(&format!(" {n} "))
    }).max_by_key(|r| s(r, "name").len())?;
    let mut used: Vec<String> = kws.iter().map(|w| (*w).to_owned()).collect();
    used.push(s(r, "name"));
    used.extend(["born", "birthday", "tall", "many", "stars", "star", "race", "species", "infected", "height",
                 "gender", "class", "branch", "affiliation", "faction", "belong", "to",
                 "rarity"].iter().map(|w| (*w).to_owned()));
    if leftover(q, &used) > 0 {
        return None;
    }
    Some(Route::new("operator_attribute", &[("operator", &s(r, "name")), ("field", field)]))
}

/// Reading-order questions: the whole guide, one event's placement, reading time, first appearance, and reading
/// in chronological order.
pub(crate) fn reading_route(q: &str, t: &Tools) -> Option<Route> {
    let items = t.guide.as_ref()?["items"].as_array()?;
    let s = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_owned();
    // Reading in chronological order: players ask it far more often than they recommend it.
    // Reading words, not "order": "In chronological order, order the death of FrostNova..." asks for a timeline.
    if has(q, &["read", "start", "play"])
        && has(q, &["chronolog", "in-world", "in world", "in universe", "in-universe", "timeline order"])
        && !has(q, &["placed", "was before", "were before", "happened before",
                     "take place", "takes place", "took place"]) {
        return Some(Route::new("reading_chronological", &[]));
    }
    // In-world timing ("Is Il Siracusano placed before chapter 2?", "this was before Babel, right?") is chronology,
    // not reading order: those go to retrieval with the timeline notes.
    if has(q, &["placed", "set before", "set after", "take place", "takes place", "took place", "was before",
                "were before", "happened before", "happen before", "timeline", "chronolog", "in-world", "in world"]) {
        return None;
    }
    let group = named_group(q, t).and_then(|g| items.iter().find(|i| i["groupId"].as_str() == Some(gid(g))));
    // Where a character first speaks.
    if has(q, &["first appear", "first appearance", "first show up", "first shows up", "debut", "shows up",
                "introduced"]) {
        let first = t.first.as_ref()?;
        let ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '[', ']'], " "));
        let who = first.as_object()?.keys().filter(|n| n.len() >= 3 && n.chars().next().is_some_and(char::is_uppercase)
            && ql.contains(&format!(" {} ", n.to_lowercase()))).max_by_key(|n| n.len())?.clone();
        // "Did you know the Candle Knight shows up in Catapult's operator record?" is not a first-appearance question.
        let mut used: Vec<String> = vec![who.clone()];
        used.extend(["first", "appear", "appears", "appearance", "show", "shows", "up", "debut", "introduced", "get",
                     "part", "story", "i", "we", "see", "meet", "which", "event",
                     "chapter"].iter().map(|w| (*w).to_owned()));
        if leftover(q, &used) > 0 {
            return None;
        }
        return Some(Route::new("first_appearance", &[("character", &who)]));
    }
    // Reading time of one event or episode.
    if let Some(i) = group {
        // Reading words, not a bare "how long" ("How long has it been in The Masses' Travels?" asks in-world time).
        if has(q, &["how many hours", "hours of reading", "how many words", "reading time", "how long to read",
                    "how long is", "how long does it take", "how long will it take"]) {
            return Some(Route::new("reading_time", &[("event", &s(i, "groupId"))]));
        }
    }
    // What an event builds on and where it sits in the EN release order.
    let per_event = ["before", "when should i read", "when do i read", "when to read", "should i read",
                     "need to read", "read first", "prerequisite", "recommended reading", "stories to play",
                     "what to read", "have to read"];
    if let Some(i) = group {
        let name = s(i, "name");
        let mut used: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
        used.extend(["before", "when", "should", "read", "reading", "need", "needs", "first", "prerequisite",
                     "prerequisites", "recommended", "stories", "story", "play", "playing", "what", "to", "have", "i",
                     "know", "complete", "specific", "chapters", "chapter", "rerun", "event", "regarding", "main",
                     "start", "after", "or", "things", "anything", "about", "it", "you", "do", "that", "which",
                     "episode"].iter().map(|w| (*w).to_owned()));
        if let Some(n) = i["episode"].as_u64() {
            used.push(n.to_string());
        }
        // One stray word is tolerated ("Is it recommended to know about specific chapters before the Babel rerun?");
        // "I was reading some Mansfield Break before Lone Trail comes out. Muelsyse talking about..." has many.
        if has(q, &per_event) && leftover(q, &used) <= 1 {
            return Some(Route::new("reading_event", &[("event", &s(i, "groupId"))]));
        }
    }
    // The whole guide.
    let general = ["reading order", "read order", "story order", "release order", "order to read", "order of events",
                   "where do i start", "where to start", "where should i start", "how do i start",
                   "start with arknights", "best way to start", "catch up", "key to understand", "key events",
                   "major lore", "important events", "essential events", "new player", "recommended order",
                   "which events are key", "which events matter"];
    if group.is_none() && has(q, &general) {
        return Some(Route::new("reading_guide", &[]));
    }
    None
}

/// "What happens in X?", "Explain the ending of X": the event summary.
pub(crate) fn recap_route(q: &str, t: &Tools) -> Option<Route> {
    let words = ["what happens in", "what happened in", "what happens during", "explain the ending", "ending of",
                 "summary of", "summarize", "summarise", "recap", "plot of", "story of", "what is the story",
                 "what was the story", "tl dr", "tldr", "rundown of"];
    // `summary_route`: a bare "summary" ("Guide Ahead summary"), the event the event detector finds when the name is
    // not in the question as written ("The Masses Travels"), and the name's words folded as `leftover` folds the
    // question's ("Masses'" is "masses").
    let wide = t.opts.summary_route;
    if !has(q, &words) && !(wide && has(q, &["summary"])) {
        return None;
    }
    let g = named_group(q, t).or_else(|| wide.then(|| t.event_in_question(q)).flatten())?;
    let id = g["groupId"].as_str()?.to_owned();
    let name = g["name"].as_str().unwrap_or(&id).to_owned();
    let mut used: Vec<String> = if wide {
        name.replace(['?', ',', '.', '!', '\'', '"'], " ").split_whitespace().map(str::to_owned).collect()
    } else { name.split_whitespace().map(str::to_owned).collect() };
    used.extend(["happens", "happened", "during", "explain", "ending", "summary", "summarize", "summarise", "recap",
                 "plot", "story", "tl", "dr", "tldr", "rundown", "can", "someone", "me", "give", "please", "event",
                 "episode", "chapter", "spoiler", "spoilers", "i", "you", "for", "on", "this",
                 "that"].iter().map(|w| (*w).to_owned()));
    if let Some(n) = g["chapter"].as_u64() {
        used.push(n.to_string());
    }
    if leftover(q, &used) > 0 {
        return None;
    }
    let ending = has(q, &["ending", "end of", "how does it end", "how did it end"]);
    Some(Route::new("recap", &[("event", &id), ("ending", if ending { "true" } else { "false" })]))
}

/// Content words a question has beyond `used` and common question words. A single-subject lookup ("What race is
/// Texas?", "Does W die?") has none; real questions that only mention an operator ("Has there been any explanation
/// why Mostima treats Exusiai the way she does?") have several and must go to retrieval, not to a table.
pub(crate) fn leftover(q: &str, used: &[String]) -> usize {
    let stop = ["which", "what", "who", "whom", "how", "when", "where", "is", "are", "was", "were", "the", "a", "an",
                "of", "in", "from", "does", "do", "did", "s", "operator", "operators", "their", "her", "his", "they",
                "she", "he", "it", "its", "to", "and", "or", "there", "any", "ever", "really", "actually", "still",
                "at", "end", "by"];
    let mut ql = format!(" {} ", q.to_lowercase().replace(['?', ',', '.', '!', '\'', '"'], " "));
    for u in used {
        ql = ql.replace(&format!(" {} ", u.to_lowercase()), " ");
    }
    ql.split_whitespace().filter(|w| !stop.contains(w)).count()
}

/// Real names from the operator table (`scripts/real_names.py`).
pub(crate) fn real_name_route(q: &str, t: &Tools) -> Option<Route> {
    if !has(q, &["real name", "true name", "birth name", "full name", "actual name"]) {
        return None;
    }
    let rows = t.real_names.as_ref()?;
    let s = |r: &serde_json::Value, k: &str| r[k].as_str().unwrap_or_default().to_owned();
    if is_open_listing(q) || has(q, &["operators' real", "every operator", "all operators"]) {
        // "Real names of every operator from Kazimierz": the place filter of the attribute table narrows the list.
        let place = place_filter(q, t);
        return Some(Route::new("real_names", &place.as_deref().map(|p| vec![("place", p)]).unwrap_or_default()));
    }
    // One operator: the longest codename in the question, as a whole word. No name in the table falls back to
    // retrieval, which may find a story that states it.
    let ql = q.to_lowercase();
    let r = rows.iter().filter(|r| {
        let n = s(r, "name").to_lowercase();
        !n.is_empty() && ql.match_indices(&n).any(|(i, _)| {
            ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
                && ql[i + n.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric())
        })
    }).max_by_key(|r| s(r, "name").len())?;
    Some(Route::new("real_name", &[("operator", &s(r, "name"))]))
}

/// The main chapter or event a question names: "Episode 7" by chapter, otherwise the longest event
/// name contained in the question.
pub(crate) fn named_group<'a>(q: &str, t: &'a Tools) -> Option<&'a serde_json::Value> {
    let ql = q.to_lowercase();
    if let Some(n) = ql.split("episode").nth(1).and_then(|r| r.trim_start().split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|d| d.parse::<u64>().ok()) {
        if let Some(g) = t.groups.values().find(|g| g["chapter"].as_u64() == Some(n)) {
            return Some(g);
        }
    }
    t.groups.values()
        .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
        .filter(|g| g["name"].as_str().is_some_and(|n| n.len() >= 4 && ql.contains(&n.to_lowercase())))
        .max_by_key(|g| g["name"].as_str().unwrap_or_default().len())
}

/// Death questions answered from the death events (`scripts/deaths.py`).
pub(crate) fn death_route(q: &str, t: &Tools) -> Option<Route> {
    // Ordering and dating questions about deaths ("In chronological order, order the death of FrostNova, death of
    // Patriot...") need the timeline notes that retrieval routing adds, not a list of death events.
    if !is_death_question(q) || is_time_question(q) || has(q, &["before", "after", "chronolog", " order"]) {
        return None;
    }
    t.deaths.as_ref()?;
    let ql = q.to_lowercase();
    let npc_only = has(q, &["non-playable", "nonplayable", "npc", "not playable"]);
    let ops_only = !npc_only && has(q, &["operator", "playable"]);

    if let Some(g) = named_group(q, t) {
        let name = g["name"].as_str().unwrap_or(gid(g));
        // Only "who died in X": "Did Lappland die at the end of Il Siracusano? Or what happened?" and "Was Tin Man's
        // death in Mansfield Break, or an earlier story?" ask something else and go to retrieval.
        let mut used: Vec<String> = name.split_whitespace().map(str::to_owned).collect();
        used.extend(["die", "died", "dies", "dead", "death", "deaths", "killed", "who", "which", "characters",
                     "character", "named", "non-playable", "npcs", "npc", "people", "playable", "episode", "chapter",
                     "list", "all", "everyone", "anyone", "happen", "there", "any"].iter().map(|w| (*w).to_owned()));
        if let Some(n) = g["chapter"].as_u64() {
            used.push(n.to_string());
        }
        if leftover(q, &used) > 0 {
            return None;
        }
        let who = if npc_only { "npc" } else if ops_only { "operators" } else { "all" };
        return Some(Route::new("deaths_in_event", &[("event", gid(g)), ("who", who)]));
    }

    if is_operator_status_question(q) || (ops_only && has(q, &["which", "what", "list", "who"])) {
        return Some(Route::new("dead_operators", &[]));
    }

    // One character: every death event for them in release order, so the answer follows the story.
    let mut names = t.death_names();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    let who = names.into_iter().find(|n| {
        let nl = n.to_lowercase();
        ql.match_indices(&nl).any(|(i, _)| {
            let before = ql[..i].chars().last().is_none_or(|c| !c.is_alphanumeric());
            let after = ql[i + nl.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric() && c != '\'');
            before && after
        })
    })?;
    // Only a question about whether or how this one character dies; "Didn't the war end because Theresa died? How
    // can there have been an ascension ceremony..." is about something else and goes to retrieval.
    let dw = ["die", "dies", "died", "dead", "death", "killed", "deceased", "passed", "away", "confirmed",
              "how", "when"];
    if leftover(q, &std::iter::once(who.clone()).chain(dw.iter().map(|w| (*w).to_owned())).collect::<Vec<_>>()) > 0 {
        return None;
    }
    Some(Route::new("death_of", &[("character", &who)]))
}
