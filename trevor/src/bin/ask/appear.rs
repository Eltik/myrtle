//! First and last appearances (`--no-appearance-order`): "When was Talulah last mentioned (chronologically)?", "What is
//! X's latest appearance?", "When was X first mentioned?" are answered from Trevor's appearance index
//! (`artifacts/appearances/appearances.json`, `scripts/appearances.py`): the story groups where the character speaks or
//! is named, ordered by EN release (the reading guide's position), and by Trevor's in-world estimate (the chronology's
//! year, ties in release order) when the question says "chronologically". Under a spoiler horizon only groups released
//! at or before it count; a group the horizon cuts counts when a visible chunk of it names the character.
//!
//! Why: retrieval returns a few similar chunks and cannot know which is the last. "When was Talulah last mentioned
//! (chronologically)?" (E2B, mid) answered with Episode 8's "Behold, the Twilight Comes" (and R8-5 in another run),
//! while she speaks in Episodes 13, 14 and 16 and later events.

use serde_json::Value;
use trevor::search::horizon::Horizon;
use trevor::search::store::ChunkStore;
use trevor::tools::{Route, Tools, contains_words, norm};

/// What a first-or-last appearance question asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OrderQ {
    /// The character's name in the appearance index.
    pub(crate) who: String,
    /// The last appearance (else the first).
    pub(crate) last: bool,
    /// The in-world order ("chronologically"), else the release order.
    pub(crate) chrono: bool,
}

const LAST: &[&str] = &["last", "latest", "most recent", "most recently", "final", "newest"];
const FIRST: &[&str] = &["first", "earliest"];
const SEEN: &[&str] = &["mentioned", "mention", "mentions", "named", "referenced", "brought up", "seen", "appear",
                        "appears", "appeared", "appearance", "show up", "shows up", "showed up", "featured", "speak",
                        "speaks", "spoke", "spoken", "talked about", "saw", "see"];
const MENTION: &[&str] = &["mentioned", "mention", "mentions", "named", "referenced", "brought up", "talked about"];
const CHRONO: &[&str] = &["chronologically", "chronological", "in world", "in universe", "in lore", "timeline",
                          "in story time"];

/// The question's first-or-last appearance ask: a last word with an appearance word, or a first word with a mention
/// word or a chronology word ("first appear" alone stays with the `first_appearance` tool), one character of the
/// appearance index (or a name an identity link joins to one) as whole words, and no event named (a question about one
/// event asks within it).
pub(crate) fn order_question(q: &str, tools: &Tools) -> Option<OrderQ> {
    let n = norm(q);
    let any = |ws: &[&str]| ws.iter().any(|w| contains_words(&n, w));
    // A first or last word within 4 words of an appearance word ("last mentioned", "the last time ... was seen").
    let toks: Vec<&str> = n.split(' ').collect();
    let seen = positions(&toks, SEEN);
    let near = |ws: &[&str]| positions(&toks, ws).iter().any(|&i| seen.iter().any(|&j| i.abs_diff(j) <= 4));
    let chrono = any(CHRONO);
    let last = near(LAST);
    let first = near(FIRST) && (any(MENTION) || chrono);
    if !(last || first) || (last && any(FIRST)) || tools.event_in_question(q).is_some() {
        return None;
    }
    let chars = tools.appearances.as_ref()?["characters"].as_object()?;
    let lower = !q.chars().any(char::is_uppercase);
    // A possessive ("Talulah's") is the name and "s": `norm` drops the apostrophe.
    let np = norm(&q.replace('\u{2019}', "'").replace("'s", " "));
    let skip: Vec<&str> = LAST.iter().chain(FIRST).chain(SEEN).chain(CHRONO).copied().collect();
    let mut best: Option<&str> = None;
    for name in chars.keys().map(String::as_str).chain(tools.identities.iter().flatten().map(String::as_str)) {
        let x = norm(name);
        if x.chars().count() < 3 || skip.contains(&x.as_str()) || !(contains_words(&n, &x) || contains_words(&np, &x))
            || !(lower || written(q, name)) {
            continue;
        }
        if best.is_none_or(|b| norm(b).len() < x.len()) {
            best = Some(name);
        }
    }
    let who = index_name(tools, best?)?;
    Some(OrderQ { who, last, chrono })
}

/// The word positions in `toks` where a phrase of `ws` starts.
fn positions(toks: &[&str], ws: &[&str]) -> Vec<usize> {
    ws.iter().flat_map(|w| {
        let p: Vec<&str> = w.split(' ').collect();
        (0..toks.len()).filter(|&i| toks.get(i..i + p.len()) == Some(&p[..])).collect::<Vec<_>>()
    }).collect()
}

/// Whether `q` holds `name` as written, as whole words.
fn written(q: &str, name: &str) -> bool {
    q.match_indices(name).any(|(i, _)| q[..i].chars().last().is_none_or(|c| !c.is_alphanumeric())
        && q[i + name.len()..].chars().next().is_none_or(|c| !c.is_alphanumeric()))
}

/// The appearance index's name for `name`: itself, or the member of its identity group with the most scenes.
fn index_name(tools: &Tools, name: &str) -> Option<String> {
    let chars = tools.appearances.as_ref()?["characters"].as_object()?;
    if chars.contains_key(name) {
        return Some(name.to_owned());
    }
    let scenes = |c: &Value| ["speaks", "mentioned"].iter()
        .flat_map(|k| c[*k].as_object().into_iter().flatten().filter_map(|(_, v)| v.as_u64())).sum::<u64>();
    tools.identities.iter().filter(|g| g.iter().any(|x| norm(x) == norm(name))).flatten()
        .filter(|x| chars.contains_key(x.as_str()))
        .max_by_key(|x| scenes(&chars[x.as_str()])).cloned()
}

/// The route for `oq`: tool "appearance_order" with its arguments.
pub(crate) fn route(oq: &OrderQ) -> Route {
    Route::new("appearance_order", &[("character", oq.who.as_str()), ("which", if oq.last { "last" } else { "first" }),
                                     ("order", if oq.chrono { "chrono" } else { "release" })])
}

/// The ask a route of tool "appearance_order" carries.
pub(crate) fn from_route(r: &Route) -> Option<OrderQ> {
    (r.tool == "appearance_order").then(|| OrderQ {
        who: r.args.get("character").cloned().unwrap_or_default(),
        last: r.args.get("which").is_none_or(|w| w == "last"),
        chrono: r.args.get("order").is_some_and(|o| o == "chrono"),
    })
}

/// One group where the character appears.
struct Seen {
    label: String,
    release: u64,
    /// Trevor's in-world year estimate, when the chronology has one.
    year: Option<f64>,
    speaks: u64,
    named: u64,
}

impl Seen {
    fn how(&self, who: &str) -> String {
        let scenes = |n: u64| if n == 1 { "1 scene".to_owned() } else { format!("{n} scenes") };
        if self.speaks > 0 {
            format!("{who} speaks in {}", scenes(self.speaks))
        } else {
            format!("{who} is named without speaking in {}", scenes(self.named))
        }
    }
}

/// The visible chunks of group `gid` in `store` where a form of `forms` speaks, and those that only name it: the
/// counts of a group the horizon cuts, so its later stories are not counted.
fn seen_within(store: &ChunkStore, h: &Horizon, gid: &str, forms: &[String]) -> (u64, u64) {
    let (mut speaks, mut named) = (0u64, 0u64);
    for c in store.chunks.iter().filter(|c| c.group_id == gid && h.allows(&c.story_id, &c.group_id)) {
        if c.text.lines().any(|l| forms.iter().any(|f| l.strip_prefix(f.as_str()).is_some_and(|r| r.starts_with(':')))) {
            speaks += 1;
        } else if forms.iter().any(|f| written(&c.text, f)) {
            named += 1;
        }
    }
    (speaks, named)
}

/// The answer for `oq` from the appearance index, under horizon `h` (with `store` for a group the horizon cuts); None
/// when the character has no placed group within it.
pub(crate) fn order_text(tools: &Tools, oq: &OrderQ, h: Option<&Horizon>, store: Option<&ChunkStore>) -> Option<String> {
    let v = tools.appearances.as_ref()?;
    let c = v["characters"].get(oq.who.as_str())?;
    let groups = &v["groups"];
    let mut forms: Vec<String> = c["forms"].as_array().into_iter().flatten().filter_map(Value::as_str)
        .map(str::to_owned).collect();
    if !forms.contains(&oq.who) {
        forms.push(oq.who.clone());
    }
    let count = |k: &str, g: &str| c[k].get(g).and_then(Value::as_u64).unwrap_or(0);
    let mut ids: Vec<&String> = c["speaks"].as_object().into_iter().flatten().map(|(g, _)| g)
        .chain(c["mentioned"].as_object().into_iter().flatten().map(|(g, _)| g)).collect();
    ids.sort();
    ids.dedup();
    let mut seen: Vec<Seen> = Vec::new();
    for g in ids {
        let gr = &groups[g.as_str()];
        // Operator records the reading guide does not place (order 10000) have no release position.
        let Some(release) = gr["order"].as_u64().filter(|&o| o < 10_000) else { continue };
        let (mut speaks, mut named) = (count("speaks", g), count("mentioned", g));
        if let Some(h) = h {
            match h.order().group_span(g) {
                Some((_, last)) if last <= h.pos => {}
                Some((first, _)) if first <= h.pos => {
                    (speaks, named) = store.map_or((0, 0), |s| seen_within(s, h, g, &forms));
                    if speaks + named == 0 {
                        continue;
                    }
                }
                _ => continue,
            }
        }
        let year = tools.groups.get(g.as_str()).and_then(|x| x["storylineYear"].as_f64());
        seen.push(Seen { label: label(gr, g), release, year, speaks, named });
    }
    if seen.is_empty() {
        return None;
    }
    seen.sort_by_key(|s| s.release);
    let who = &oq.who;
    let within = h.map_or(String::new(), |h| {
        h.order().era_at(h.pos).filter(|e| !e.name.is_empty()).map_or_else(|| " up to where you have read".to_owned(),
                                             |e| format!(" up to where you have read (you are up to {})", e.name))
    });
    let (word, pick): (&str, &Seen) = if oq.last { ("latest", seen.last()?) } else { ("first", seen.first()?) };
    let mut out = String::new();
    // Groups without a year estimate are left out of the in-world order.
    let mut by_year: Vec<(&Seen, f64)> = seen.iter().filter_map(|s| s.year.map(|y| (s, y))).collect();
    by_year.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.release.cmp(&b.0.release)));
    let in_world = if oq.last { by_year.last() } else { by_year.first() };
    if let (true, Some(&(w, y))) = (oq.chrono, in_world) {
        out.push_str(&format!("In Trevor's in-world order (an estimate from the years the script states; most stories \
            state none, so stories with the same estimate follow release order), the {word} story group where {who} \
            appears{within} is {}, placed at about {y:.0}: {}.\n\n", w.label, w.how(who)));
        out.push_str(&format!("In EN release order (the order the stories came out), the {word} is {}: {}.\n",
                              pick.label, pick.how(who)));
    } else {
        out.push_str(&format!("In EN release order (the order the stories came out), the {word} story group where \
            {who} appears{within} is {}: {}.\n", pick.label, pick.how(who)));
    }
    let around: Vec<&Seen> = if oq.last { seen.iter().rev().skip(1).take(3).collect() }
        else { seen.iter().skip(1).take(3).collect() };
    if !around.is_empty() {
        let lead = if oq.last { "Before that" } else { "After that" };
        out.push_str(&format!("{lead}: {}.\n", around.iter().map(|s| format!("{} ({})", s.label,
            if s.speaks > 0 { "speaks" } else { "named" })).collect::<Vec<_>>().join("; ")));
    }
    out.push_str("\nFrom Trevor's appearance index (the scripts' speaker labels and name matches; a scene is a passage of \
        about 400 tokens). Not counted: stories without speaker labels (Integrated Strategies, paradox simulations), \
        operator records the reading guide does not place, and appearances under a name the identity links do not \
        connect.");
    Some(out)
}

/// An appearance-index group as a reader names it (`Tools::appearances`' labels).
fn label(g: &Value, id: &str) -> String {
    let name = g["name"].as_str().unwrap_or(id);
    let kind = g["kind"].as_str().unwrap_or_default();
    match (kind, g["episode"].as_u64()) {
        ("main", Some(e)) => format!("Episode {e}: {name}"),
        (k, _) if k.starts_with("operator record") => format!("{k}, '{name}'"),
        _ => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn tools() -> Tools {
        let mut t = Tools { appearances: Some(json!({
            "characters": {
                "Talulah": {"forms": ["Talulah"], "speaks": {"main_8": 91, "main_16": 4, "act49side": 1},
                            "mentioned": {"main_3": 2, "story_x_set_1": 1}},
                "Last": {"forms": ["Last"], "speaks": {"main_8": 1}, "mentioned": {}},
            },
            "groups": {
                "main_3": {"order": 3, "kind": "main", "episode": 3, "name": "Stinging Shock"},
                "main_8": {"order": 20, "kind": "main", "episode": 8, "name": "Roaring Flare"},
                "main_16": {"order": 79, "kind": "main", "episode": 16, "name": "Abnormal Spectrum"},
                "act49side": {"order": 83, "kind": "event", "name": "First of A Thousand Autumns"},
                "story_x_set_1": {"order": 10000, "kind": "operator record of X", "name": "Fate"},
            }
        })), ..Tools::default() };
        t.groups.insert("main_16".into(), json!({"groupId": "main_16", "name": "Abnormal Spectrum", "chapter": 16,
                                                 "storylineYear": 1100.5}));
        t.groups.insert("act49side".into(), json!({"groupId": "act49side", "name": "First of A Thousand Autumns",
                                                   "storylineYear": 1100.0}));
        t.groups.insert("main_8".into(), json!({"groupId": "main_8", "name": "Roaring Flare", "chapter": 8,
                                                "storylineYear": 1098.5}));
        t
    }

    #[test]
    fn last_and_first_questions() {
        let t = tools();
        let q = |s: &str| order_question(s, &t);
        assert_eq!(q("When was Talulah last mentioned (chronologically)?"),
                   Some(OrderQ { who: "Talulah".into(), last: true, chrono: true }));
        assert_eq!(q("What is Talulah's latest appearance?"),
                   Some(OrderQ { who: "Talulah".into(), last: true, chrono: false }));
        assert_eq!(q("When was the last time we saw Talulah?"),
                   Some(OrderQ { who: "Talulah".into(), last: true, chrono: false }));
        assert_eq!(q("when was talulah first mentioned"),
                   Some(OrderQ { who: "Talulah".into(), last: false, chrono: false }));
        // "first appear" stays with the first_appearance tool; no appearance word; an event named.
        for s in ["When does Talulah first appear?", "Is Talulah the last Kaltsit?", "Who is Talulah?",
                  "What recent event did Talulah mention?", "Talulah was last in Ursus, as Ch'en said she mentioned",
                  "When was Talulah last mentioned in Roaring Flare?", "Talulah first and last appearance"] {
            assert_eq!(q(s), None, "{s}");
        }
    }

    #[test]
    fn the_answer_follows_the_order_asked() {
        let t = tools();
        let last = order_text(&t, &OrderQ { who: "Talulah".into(), last: true, chrono: false }, None, None).unwrap();
        assert!(last.starts_with("In EN release order (the order the stories came out), the latest story group where \
            Talulah appears is First of A Thousand Autumns: Talulah speaks in 1 scene.\n"), "{last}");
        assert!(last.contains("Before that: Episode 16: Abnormal Spectrum (speaks); Episode 8: Roaring Flare (speaks); \
            Episode 3: Stinging Shock (named)."), "{last}");
        let chrono = order_text(&t, &OrderQ { who: "Talulah".into(), last: true, chrono: true }, None, None).unwrap();
        assert!(chrono.contains("the latest story group where Talulah appears is Episode 16: Abnormal Spectrum, \
            placed at about 1100: Talulah speaks in 4 scenes."), "{chrono}");
        assert!(chrono.contains("In EN release order (the order the stories came out), the latest is First of A \
            Thousand Autumns"), "{chrono}");
        let first = order_text(&t, &OrderQ { who: "Talulah".into(), last: false, chrono: false }, None, None).unwrap();
        assert!(first.contains("the first story group where Talulah appears is Episode 3: Stinging Shock: Talulah is \
            named without speaking in 2 scenes."), "{first}");
        // Under a horizon at the end of Episode 16, the event after it does not count.
        let guide = json!({"items": [
            {"groupId": "main_3", "position": 1}, {"groupId": "main_8", "position": 2},
            {"groupId": "main_16", "position": 3}, {"groupId": "act49side", "position": 4}]});
        let stories: Vec<Value> = ["main_3", "main_8", "main_16", "act49side"].iter()
            .map(|g| json!({"storyId": format!("{g}_a"), "groupId": g, "storySort": 1})).collect();
        let o = std::sync::Arc::new(trevor::search::horizon::ReleaseOrder::from_parts(&guide, &stories));
        let h = o.horizon_at(3).unwrap();
        let cut = order_text(&t, &OrderQ { who: "Talulah".into(), last: true, chrono: false }, Some(&h), None)
            .unwrap();
        assert!(cut.contains("the latest story group where Talulah appears up to where you have read is Episode 16: \
            Abnormal Spectrum"), "{cut}");
        assert!(order_text(&t, &OrderQ { who: "Talulah".into(), last: true, chrono: false },
                           Some(&o.horizon_at(0).unwrap()), None).is_none());
        assert_eq!(from_route(&route(&OrderQ { who: "Talulah".into(), last: false, chrono: true })),
                   Some(OrderQ { who: "Talulah".into(), last: false, chrono: true }));
    }
}
