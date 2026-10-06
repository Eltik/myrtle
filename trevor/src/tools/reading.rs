//! Reading order: the reading guide, chronological lists, first appearances, read-before comparisons,
//! recaps and the timeline.

use serde_json::Value;

use super::{Edge, ToolError, ToolResult, Tools, s};

impl Tools {
    pub(super) fn guide_items(&self) -> Result<&Vec<Value>, ToolError> {
        self.guide.as_ref().and_then(|g| g["items"].as_array()).ok_or(ToolError::Missing("reading_guide.json"))
    }

    /// The reading-guide entry of an event or episode.
    pub(super) fn guide_item(&self, event: &str) -> Result<&Value, ToolError> {
        let items = self.guide_items()?;
        let gid = self.resolve_event(event)?["groupId"].as_str().unwrap_or_default().to_owned();
        items.iter().find(|i| i["groupId"].as_str() == Some(gid.as_str()))
            .ok_or_else(|| ToolError::NotFound { what: "reading-guide entry", name: event.to_owned() })
    }

    /// Every main episode and event in Trevor's in-world order (chronology v1).
    #[must_use]
    pub fn timeline_all(&self) -> String {
        // The main story is its own spine (episode order is in-world order). Episodes 0 to 9 carry only an
        // upper bound, so sorting them among events by year would put the Chernobog arc after 1097 side
        // events; how they interleave with early events is not stated, so the two lists are kept apart.
        let key = |g: &&Value| (g["storylineYear"].as_f64().unwrap_or(9999.0), g["chapter"].as_u64().unwrap_or(0),
                                g["releaseTime"].as_i64().unwrap_or(0));
        let mut mains: Vec<&Value> = self.groups.values().filter(|g| g["chapter"].is_u64()).collect();
        mains.sort_by_key(|g| g["chapter"].as_u64().unwrap_or(0));
        let mut events: Vec<&Value> = self.groups.values()
            .filter(|g| !g["chapter"].is_u64() && !g["groupId"].as_str().unwrap_or_default().starts_with("story_")).collect();
        events.sort_by(|a, b| { let (x, y) = (key(a), key(b)); x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)) });
        let records = self.groups.values().filter(|g| g["groupId"].as_str().unwrap_or_default().starts_with("story_")).count();
        let line = |g: &Value| {
            let bound = g["yearBound"].as_str().map_or(String::new(), |b| format!("{b} "));
            let exc = g["candidateException"]["year"].as_i64().map_or(String::new(), |y| format!("; its own dated scenes suggest {y}"));
            format!("{}: {bound}about {:.0} ({}{exc})", g["name"].as_str().unwrap_or_default(),
                    g["storylineYear"].as_f64().unwrap_or(0.0), g["basis"].as_str().unwrap_or_default())
        };
        let mut out = format!("Trevor's in-world order. The main story is listed first as its own sequence, because the script does not \
say how Episodes 0 to 9 interleave with the early events. Years are estimates unless a basis says the script states them; \
{records} operator records are not listed.\n\nMain story ({} episodes, in-world order = episode order):\n", mains.len());
        for g in &mains {
            out.push_str(&format!("  Episode {}. {}\n", g["chapter"].as_u64().unwrap_or(0), line(g)));
        }
        out.push_str(&format!("\nEvents ({}), estimated in-world order:\n", events.len()));
        for (i, g) in events.iter().enumerate() {
            out.push_str(&format!("  {}. {}\n", i + 1, line(g)));
        }
        out
    }

    // ---------------------------------------------------------------- reading guide
    // Reading-order questions, 5 to 9% of real ones, answered 0.29 by retrieval (design/trevor-questions.md): the EN
    // release order (`scripts/reading_guide.py`; main episode and event dates from the game data since 2026-10-04,
    // the wiki's episode dates before, `opts.wiki_legacy`), what an event builds on (its primer's prior context), reading time, and where a character first speaks.

    /// Reading in chronological order: players ask it far more often than they recommend it (3 recommend, 6 advise
    /// against; design/trevor-reading-order-research.md), so the in-world order comes with the reason most don't.
    ///
    /// # Errors
    /// The reading guide is missing (the keyword route needed it too).
    pub fn reading_chronological(&self) -> ToolResult {
        self.guide_items()?;
        let why = self.community.as_ref().map_or(String::new(), |c| c["chronological"]["why"].as_str().unwrap_or_default().to_owned());
        // The prequel placements and years below were taken from the community's guides and the wiki (2026-09-28);
        // the wiki is a reference only since 2026-10-04, so the default keeps the community's reason and Trevor's
        // own estimate from the game text (`opts.wiki_legacy` restores the paragraph).
        if !self.opts.wiki_legacy {
            return Ok(format!("Most players recommend reading in release order, not chronological order: {why}. If you want \
the in-world order anyway, Trevor's own estimate from the dates the script states follows; most stories state no year, so \
their placement follows release order.\n\n{}", self.timeline_all()));
        }
        Ok(format!("Most players recommend reading in release order, not chronological order: {why}. If you want the \
in-world order anyway, the stories set before the Prologue come first: A Walk in the Dust (1070s to 1080s), Babel (the Kazdel \
civil war, 1090 to 1094) and Darknights Memoir (about 1094 to 1096), then the main story, with Children of Ursus during the \
Chernobog crisis (Episodes 0 to 3). Several side stories are set later than they came out: Guide Ahead, IL Siracusano, \
Dorothy's Vision and Lone Trail after Episode 14 (1099), and the Yan events (1101 to 1102) near the end. Placements of these \
prequels are from the community's reading guides and the Arknights wiki; Trevor's own estimate for the rest follows.\n\n{}",
            self.timeline_all()))
    }

    /// Where a character first speaks.
    ///
    /// # Errors
    /// Missing table or an unknown speaker.
    pub fn first_appearance(&self, character: &str) -> ToolResult {
        self.guide_items()?;
        let first = self.first.as_ref().ok_or(ToolError::Missing("first_appearance.json"))?;
        let names = self.speaker_names();
        let who = self.resolve_name("speaker", character, &names)?;
        let f = &first[who];
        Ok(format!("{who} first speaks in {} (EN release {}), story {}. That is the first line Trevor finds in \
the release-ordered main story and events; operator records and earlier mentions by name without a line are not counted.",
            s(f, "group"), s(f, "released"), s(f, "storyId")))
    }

    /// Reading time of one event or episode.
    ///
    /// # Errors
    /// Missing guide or an event it does not list.
    pub fn reading_time(&self, event: &str) -> ToolResult {
        let i = self.guide_item(event)?;
        let w = i["words"].as_u64().unwrap_or(0);
        #[allow(clippy::cast_precision_loss)]
        let hours = w as f64 / 250.0 / 60.0;
        Ok(format!("{} has about {} words of script, roughly {hours:.1} hours at 250 words per minute, without \
voice lines or battles.", guide_label(i), w))
    }

    /// Two events or episodes against each other, for "can I read X after Y", "does X spoil Y": their EN release
    /// order, the game's own Storylines links and shared storylines, what each builds on (shared recurring cast),
    /// the community's placement rule, and what that means for spoilers. Each fact is labelled by its source.
    ///
    /// # Errors
    /// Missing guide, an event it does not list, or the same event twice.
    pub fn reading_compare(&self, event: &str, other: &str) -> ToolResult {
        let (x, y) = (self.guide_item(event)?, self.guide_item(other)?);
        let (xid, yid) = (s(x, "groupId"), s(y, "groupId"));
        if xid == yid {
            return Err(ToolError::BadArg { arg: "other", value: other.to_owned() });
        }
        let (xl, yl) = (guide_label(x), guide_label(y));
        let pos = |i: &Value| i["position"].as_u64().unwrap_or(0);
        let (first, then) = if pos(x) <= pos(y) { (x, y) } else { (y, x) };
        let (fl, tl) = (guide_label(first), guide_label(then));
        let src = if self.opts.wiki_legacy { "from the game data and the Arknights wiki" } else { "from the game data" };
        let mut out = format!("Release order (EN, {src}): {fl} came out on {}, {tl} on {}.",
                              s(first, "released"), s(then, "released"));
        // The game's own links, direct or through a storyline that holds both.
        let edges = self.story_edges();
        let direct = edges.iter().find(|((a, _), (b, _))| (a == &xid && b == &yid) || (a == &yid && b == &xid));
        let mut game_order: Option<(String, String)> = None;
        if let Some(((_, al), (_, bl))) = direct {
            out.push_str(&format!(" The game's Storylines view places {al} before {bl}."));
            game_order = Some((al.clone(), bl.clone()));
        } else if let Some(line) = self.storylines.as_ref().and_then(|v| v["storylines"].as_array()).into_iter().flatten().find(|l| {
            let g: Vec<&str> = l["groups"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
            g.contains(&xid.as_str()) && g.contains(&yid.as_str())
        }) {
            let g: Vec<&str> = line["groups"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
            let (a, b) = if g.iter().position(|v| *v == xid) < g.iter().position(|v| *v == yid) { (&xl, &yl) } else { (&yl, &xl) };
            out.push_str(&format!(" The game's Storylines view lists both in its \"{}\" storyline, {a} before {b}.", s(line, "name")));
            game_order = Some((a.clone(), b.clone()));
        } else {
            out.push_str(" The game's Storylines view has no link between them.");
        }
        // Trevor's measure: what an event builds on shares the most of its recurring cast.
        let builds = |a: &Value, b: &Value| a["buildsOn"].as_array().into_iter().flatten().any(|n| n.as_str() == Some(s(b, "name").as_str()));
        let later_builds = builds(then, first);
        if later_builds {
            out.push_str(&format!(" Trevor's reading guide: {tl} builds on {fl} (it shares the most of its recurring cast)."));
        } else if builds(first, then) {
            out.push_str(&format!(" Trevor's reading guide: {fl} builds on {tl} (it shares the most of its recurring cast)."));
        }
        if let Some(c) = &self.community {
            for r in c["rules"].as_array().into_iter().flatten().filter(|r| [xid.as_str(), yid.as_str()].contains(&r["groupId"].as_str().unwrap_or_default())) {
                out.push_str(&format!(" Players (r/arknights reading-order threads, {} commenters) place {} {}.",
                                      r["commenters"].as_u64().unwrap_or(0), s(r, "name"), s(r, "rule")));
            }
        }
        // What it means: the later story can refer back to the earlier one, never the other way.
        let linked = game_order.is_some();
        let (a, b) = game_order.unwrap_or_else(|| (fl.clone(), tl.clone()));
        if game_order_agrees(&a, &fl) {
            out.push_str(&format!("\n\nSpoilers: {b} came out after {a}{}, so it can spoil {a}; {a} cannot spoil {b}. Reading {a} first \
is the safe order, and once you have read {a} you can read {b}.",
                if linked { " and the game's Storylines view puts it after" } else if later_builds { " and builds on it" } else { "" }));
        } else {
            out.push_str(&format!("\n\nSpoilers: {tl} came out after {fl}, so it can spoil {fl}, but the game's Storylines view places \
{a} before {b}; it treats {a} as the one to read first although it came out later."));
        }
        Ok(out)
    }

    /// What an event builds on and where it sits in the EN release order.
    ///
    /// # Errors
    /// Missing guide or an event it does not list.
    pub fn reading_event(&self, event: &str) -> ToolResult {
        let items = self.guide_items()?;
        let i = self.guide_item(event)?;
        let n = usize::try_from(i["position"].as_u64().unwrap_or(1)).unwrap_or(1);
        let prev = items[..n - 1].iter().rev().find(|x| s(x, "kind") == "main").map(guide_label);
        let next = items[n..].iter().find(|x| s(x, "kind") == "main").map(guide_label);
        let list = |k: &str| i[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
        let mut out = format!("{} came out on EN on {}", guide_label(i), s(i, "released"));
        match (prev, next) {
            (Some(p), Some(nx)) => out.push_str(&format!(", after {p} and before {nx}. In release order, read it after {p}.")),
            (Some(p), None) => out.push_str(&format!(", after {p}, the latest main episode on EN.")),
            _ => out.push('.'),
        }
        if let Some(c) = &self.community {
            let gid = s(i, "groupId");
            for r in c["rules"].as_array().into_iter().flatten().filter(|r| r["groupId"].as_str() == Some(gid.as_str())) {
                out.push_str(&format!(" Players recommend reading it {} ({} commenters in r/arknights reading-order threads).",
                                      s(r, "rule"), r["commenters"].as_u64().unwrap_or(0)));
            }
            if let Some(e) = c["essential"].as_array().into_iter().flatten().find(|e| e["groupId"].as_str() == Some(gid.as_str())) {
                out.push_str(&format!(" It is one of the side stories players call essential for the main story ({} commenters: {}).",
                                      e["commenters"].as_u64().unwrap_or(0), s(e, "why")));
            }
        }
        let b = list("buildsOn");
        if !b.is_empty() {
            out.push_str(&format!(" Earlier stories it builds on (sharing the most of its recurring cast): {b}."));
        }
        let l = list("builtOnBy");
        if !l.is_empty() {
            out.push_str(&format!(" Later stories that build on it: {l}."));
        }
        if let Some(p) = self.bank.iter().find(|r| r["id"].as_str() == Some(&format!("primer:{}", s(i, "groupId")))) {
            out.push_str(&format!("\n\nWhat to know before starting it:\n{}", s(p, "answer")));
        }
        Ok(out)
    }

    /// The whole guide.
    ///
    /// # Errors
    /// Missing guide.
    pub fn reading_guide(&self) -> ToolResult {
        let items = self.guide_items()?;
        let mut key: Vec<&Value> = items.iter().filter(|i| s(i, "kind") == "event").collect();
        key.sort_by_key(|i| std::cmp::Reverse(i["prereqOf"].as_u64().unwrap_or(0)));
        let keyl = key.iter().take(10).map(|i| format!("{} ({})", s(i, "name"), i["prereqOf"].as_u64().unwrap_or(0))).collect::<Vec<_>>().join(", ");
        let mut out = String::from("Trevor's reading guide. Most players recommend EN release order: the main story in episode \
order with the events in between as they came out; operator records are optional side stories. (Other common approaches: \
main story first with a few must-read side stories slotted in, or one storyline at a time, which the game's Movements view \
has sorted by since September 2025. Chronological order is rarely recommended because the prequels spoil the Doctor's amnesia.)\n");
        let edges = if self.opts.storylines { self.story_edges() } else { Vec::new() };
        if !edges.is_empty() {
            out.push_str("\nThe game's own reading links (its Storylines view, from the game data):\n");
            let rules: Vec<&Value> = self.community.as_ref().and_then(|c| c["rules"].as_array()).into_iter().flatten().collect();
            for e @ (a, b) in &edges {
                let players = rules.iter().find(|r| rule_matches_edge(r, std::slice::from_ref(e)))
                    .map_or(String::new(), |r| format!("; players also recommend this ({} commenters)", r["commenters"].as_u64().unwrap_or(0)));
                out.push_str(&format!("- The game's Storylines view places {} before {}{players}.\n", a.1, b.1));
            }
            let skipped = self.storylines.as_ref().and_then(|v| v["edges"].as_array()).map_or(0, Vec::len) - edges.len();
            if skipped > 0 {
                out.push_str(&format!("({skipped} more links involve a main-story section the game data names only by an id.)\n"));
            }
        }
        if let Some(c) = &self.community {
            out.push_str("\nSide stories players call essential for the main story (commenters in r/arknights reading-order threads):\n");
            for e in c["essential"].as_array().into_iter().flatten() {
                out.push_str(&format!("- {} ({}): {}\n", s(e, "name"), e["commenters"].as_u64().unwrap_or(0), s(e, "why")));
            }
            out.push_str("\nWhere players place them:\n");
            for r in c["rules"].as_array().into_iter().flatten().filter(|r| r["commenters"].as_u64().unwrap_or(0) >= 5) {
                let tag = if edges.is_empty() { String::new() } else if rule_matches_edge(r, &edges) {
                    "; the game's Storylines view agrees".into()
                } else { "; player advice".into() };
                out.push_str(&format!("- {}: {} ({}{tag})\n", s(r, "name"), s(r, "rule"), r["commenters"].as_u64().unwrap_or(0)));
            }
        }
        let dates = if self.opts.wiki_legacy { "main episode dates from the Arknights wiki, event dates from the game data" }
            else { "dates from the game data; \"about\" marks an estimate, \"by\" a latest date" };
        out.push_str(&format!("\nTrevor's own measure, the events later stories share the most cast with: {keyl}.\n\nEN release order \
({} entries; {dates}):\n", items.len()));
        for i in items {
            out.push_str(&format!("{}. {} ({})\n", i["position"].as_u64().unwrap_or(0), guide_label(i), s(i, "released")));
        }
        Ok(out)
    }

    /// The game's read-before links as ((id, label), (id, label)), main episodes labelled "Episode N"; links to a
    /// section the data names only by its id (act2mainss) are left out.
    pub(super) fn story_edges(&self) -> Vec<Edge> {
        let label = |id: &str, name: &str| -> Option<(String, String)> {
            if let Some(n) = self.groups.get(id).and_then(|g| g["chapter"].as_u64()) {
                return Some((id.to_owned(), format!("Episode {n} {name}")));
            }
            (name != id && !name.is_empty()).then(|| (id.to_owned(), name.to_owned()))
        };
        self.storylines.as_ref().and_then(|v| v["edges"].as_array()).into_iter().flatten()
            .filter_map(|e| Some((label(&s(e, "first"), &s(e, "firstName"))?, label(&s(e, "then"), &s(e, "thenName"))?)))
            .collect()
    }

    // ---------------------------------------------------------------- Integrated Strategies endings

    /// The event summary (P2 method C, or the story summary of a single-story group), and for an ending the
    /// summary of its last story. Plot recaps were 0.33 to 0.50 by retrieval.
    ///
    /// # Errors
    /// An unknown event, or no summary for it.
    pub fn recap(&self, event: &str, ending: bool) -> ToolResult {
        let g = self.resolve_event(event)?;
        let gid = g["groupId"].as_str().unwrap_or_default().to_owned();
        let name = g["name"].as_str().unwrap_or(&gid).to_owned();
        let event = self.bank.iter().find(|r| r["id"].as_str() == Some(&format!("event:{gid}")));
        // The keyword route's substring test let "main_1" match main_10 to main_16 story ids and "main_0" match
        // "main_01-01"; only the "ending" part and a group without an event summary read these stories.
        let prefix = format!("{gid}_");
        let stories: Vec<&Value> = self.bank.iter().filter(|r| s(r, "kind") == "story"
            && r["sources"][0].as_str().is_some_and(|x| self.groups.contains_key(&gid)
                && if self.opts.exact_recap { x.starts_with(&prefix) } else { x.contains(&gid) })).collect();
        let mut out = String::new();
        if let Some(e) = event {
            out.push_str(&format!("{name}, summarized by Trevor from its script (its stories in order):\n\n{}", s(e, "answer")));
        } else {
            let st = stories.first().ok_or_else(|| ToolError::NoData(format!("no summary of {name}")))?;
            out.push_str(&format!("{name}, summarized by Trevor from its script:\n\n{}", s(st, "answer")));
        }
        if ending {
            if let Some(last) = stories.last() {
                out.push_str(&format!("\n\nThe last story, {}:\n{}", s(last, "question").trim_start_matches("What happens in ")
                    .trim_end_matches('?'), s(last, "answer")));
            }
        }
        Ok(out)
    }

    // ---------------------------------------------------------------- deaths
    // Death events (`scripts/deaths.py`): each is a death the script shows or reports, tied to its story and quoted
    // verbatim, so status follows the stories a reader has seen and a new story only adds events (Ian, 2026-09-28:
    // Arknights is live service).

    pub(super) fn latest(&self) -> String {
        self.groups.values().max_by_key(|g| g["releaseTime"].as_i64().unwrap_or(0))
            .and_then(|g| g["name"].as_str()).unwrap_or("?").to_owned()
    }
}

pub(super) fn game_order_agrees(game_first: &str, release_first: &str) -> bool {
    game_first == release_first
}

/// A community placement rule agrees with a game link when it names the other story on the same side ("before
/// Episode 14" for Babel -> Episode 14).
pub(super) fn rule_matches_edge(r: &Value, edges: &[Edge]) -> bool {
    let g = s(r, "groupId");
    let rule = s(r, "rule").to_lowercase();
    let short = |label: &str| label.split_whitespace().take(2).collect::<Vec<_>>().join(" ").to_lowercase();
    edges.iter().any(|((a, al), (b, bl))| {
        let side = |label: &str, dir: &str| {
            let l = if label.starts_with("Episode ") { short(label) } else { label.to_lowercase() };
            rule.contains(&format!("{dir} {l}"))
        };
        (a == &g && side(bl, "before")) || (b == &g && side(al, "after"))
    })
}

pub(super) fn guide_label(i: &Value) -> String {
    if s(i, "kind") == "main" {
        format!("Episode {} {}", i["episode"].as_u64().unwrap_or(0), s(i, "name"))
    } else { s(i, "name") }
}
