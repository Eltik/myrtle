//! Trevor's table tools: answers produced from precomputed tables, with structured arguments.
//!
//! `ask` used to answer some questions from tables with one function per route that both parsed the
//! question with keyword lists and wrote the answer. The writing half lives here, unchanged in its
//! text; the parsing half is now a router's job (`ask --router model|keywords|knn`, `crate::router`).
//! Every tool resolves its own arguments: an event given by name ("Babel", "Episode 7", "chapter 14",
//! a misspelling within one or two letters) resolves to a group id, a character or operator name
//! through the tables and the identity links. A failed resolution is a [`ToolError`], which the
//! caller treats as "answer by retrieval"; a tool never answers for a name it did not resolve.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Every tool a router may choose, plus `retrieve` (answer by retrieval).
pub const TOOL_NAMES: &[&str] = &[
    "reading_guide", "reading_event", "reading_chronological", "reading_time", "first_appearance", "recap",
    "timeline_all", "deaths_in_event", "death_of", "dead_operators", "real_name", "real_names",
    "operator_attribute", "operator_filter", "canon", "topic", "reading_compare",
];

/// Behaviors added after the keyword router, each with its off value. `ask --router keywords` runs with
/// [`ToolOptions::KEYWORDS`], so its output stays byte-identical to the binary before them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolOptions {
    /// The reading guide leads with the game's own Storylines links (`ask --no-storylines` turns it off).
    pub storylines: bool,
    /// A recap's stories are those whose id starts with "<group>_"; off is the old substring test, under which
    /// Episode 0 took 182 story summaries instead of 14 and Episode 1 took 211 instead of 12 (`--recap-substring`).
    pub exact_recap: bool,
    /// Integrated Strategies numbers as the game counts them: IS #1 is Ceobe's Fungimist (CN only, not in the data
    /// Trevor has), so rogue_1 is IS #2 (Ian, 2026-09-30). Off (`ask --is-numbering-v1`) is the old rogue_N = IS N.
    pub is_offset: bool,
    /// The canon answer as before the per-run verdicts (2026-09-30): strength labels from reference counts only
    /// (`ask --canon-v1`). Off, a run whose data carries a `verdict` is answered from it.
    pub canon_v1: bool,
    /// The wiki-based answers of the morning of 2026-10-04, for measurement only (`ask --wiki-legacy`): the design tool
    /// reads the wiki trivia table (`crate::reference`) and the reading-guide and death texts credit the wiki. Off (the
    /// default since 2026-10-04, Ian: the wiki is a reference), Trevor serves only what he deduces from the game data.
    pub wiki_legacy: bool,
}

impl ToolOptions {
    pub const KEYWORDS: Self = Self { storylines: false, exact_recap: false, is_offset: true, canon_v1: false, wiki_legacy: false };
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self { storylines: true, exact_recap: true, is_offset: true, canon_v1: false, wiki_legacy: false }
    }
}
pub const RETRIEVE: &str = "retrieve";

/// A router's decision: a tool name (or `retrieve`) and its string arguments.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub tool: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
}

impl Route {
    #[must_use]
    pub fn new(tool: &str, args: &[(&str, &str)]) -> Self {
        Self { tool: tool.to_owned(), args: args.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect() }
    }

    #[must_use]
    pub fn retrieve() -> Self {
        Self::new(RETRIEVE, &[])
    }

    #[must_use]
    pub fn is_retrieve(&self) -> bool {
        self.tool == RETRIEVE
    }

    fn arg(&self, k: &'static str) -> Result<&str, ToolError> {
        self.args.get(k).map(String::as_str).filter(|v| !v.trim().is_empty()).ok_or(ToolError::MissingArg(k))
    }

    fn opt(&self, k: &str) -> Option<&str> {
        self.args.get(k).map(String::as_str).map(str::trim).filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none"))
    }
}

/// Why a tool gave no answer. Every variant means "answer by retrieval", never a guess.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToolError {
    #[error("unknown tool {0:?}")]
    UnknownTool(String),
    #[error("missing argument {0}")]
    MissingArg(&'static str),
    #[error("bad value {value:?} for {arg}")]
    BadArg { arg: &'static str, value: String },
    #[error("no {what} matches {name:?}")]
    NotFound { what: &'static str, name: String },
    #[error("{0}")]
    NoData(String),
    #[error("table {0} is missing")]
    Missing(&'static str),
}

pub type ToolResult = Result<String, ToolError>;

/// Which deaths `deaths_in_event` lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    All,
    Npc,
    Operators,
}

/// One operator's attribute, as `operator_attribute` names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Race,
    Birthplace,
    Birthday,
    Height,
    Infection,
    Class,
    Branch,
    Rarity,
    Gender,
    Affiliation,
}

impl Field {
    pub const NAMES: &[&str] =
        &["race", "birthplace", "birthday", "height", "infection", "class", "branch", "rarity", "gender", "affiliation"];

    /// # Errors
    /// A name outside [`Field::NAMES`].
    pub fn parse(s: &str) -> Result<Self, ToolError> {
        Ok(match s.trim().to_lowercase().as_str() {
            "race" | "species" => Self::Race,
            "birthplace" | "place of birth" | "hometown" => Self::Birthplace,
            "birthday" | "date of birth" => Self::Birthday,
            "height" | "heightcm" => Self::Height,
            "infection" | "infected" | "infectiontext" | "oripathy" => Self::Infection,
            "class" => Self::Class,
            "branch" | "subclass" => Self::Branch,
            "rarity" | "stars" => Self::Rarity,
            "gender" => Self::Gender,
            "affiliation" | "nation" | "faction" => Self::Affiliation,
            _ => return Err(ToolError::BadArg { arg: "field", value: s.to_owned() }),
        })
    }

    /// The attribute table's key.
    fn key(self) -> &'static str {
        match self {
            Self::Race => "race",
            Self::Birthplace => "birthplace",
            Self::Birthday => "birthday",
            Self::Height => "heightCm",
            Self::Infection => "infectionText",
            Self::Class => "class",
            Self::Branch => "branch",
            Self::Rarity => "rarity",
            Self::Gender => "gender",
            Self::Affiliation => "nation",
        }
    }
}

/// Filters of `operator_filter`; every field optional, at least one filter or a sort required.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    pub place: Option<String>,
    pub race: Option<String>,
    /// More than one only from the keyword router ("guard defenders"), which lists the intersection.
    pub classes: Vec<String>,
    pub branch: Option<String>,
    pub rarity: Option<i64>,
    /// Some(true) female, Some(false) male.
    pub female: Option<bool>,
    pub infected: Option<bool>,
    /// Some(true) tallest, Some(false) shortest.
    pub tallest: Option<bool>,
}

pub const CLASSES: [&str; 8] = ["Vanguard", "Guard", "Defender", "Sniper", "Caster", "Medic", "Supporter", "Specialist"];
const PLACE_KEYS: [&str; 4] = ["nation", "group", "team", "birthplace"];

/// The tables, loaded once.
#[derive(Default)]
pub struct Tools {
    /// Chronology v1 groups by id (`artifacts/chrono/timeline_v1.json`). Ordered: lookups by name and the latest
    /// release iterate it, and a HashMap made their ties depend on the run (2026-10-02).
    pub groups: BTreeMap<String, Value>,
    pub guide: Option<Value>,
    pub community: Option<Value>,
    pub first: Option<Value>,
    pub bank: Vec<Value>,
    pub deaths: Option<Vec<Value>>,
    pub real_names: Option<Vec<Value>>,
    pub attributes: Option<Vec<Value>>,
    /// Name lists of the identity links (`artifacts/entities/identities.json`), label first.
    pub identities: Vec<Vec<String>>,
    /// The game's Storylines view (`artifacts/chrono/storylines.json`, `scripts/reading_guide.py storylines`).
    pub storylines: Option<Value>,
    /// Integrated Strategies endings and the later stories that refer to them (`artifacts/canon/is_endings.json`).
    pub canon: Option<Value>,
    /// Topic summaries of races, nations and concepts (`artifacts/topics/topics.jsonl`).
    pub topics: Option<Vec<Value>>,
    /// The summary tree for whole-story questions (`artifacts/overview/overview.jsonl`, `scripts/overview.py`).
    pub overview: Option<Vec<Value>>,
    /// The dossiers chosen from data (2026-10-01), not in P3b: (name, text as P3b shows a dossier). Empty unless
    /// [`Tools::load_new_dossiers`] ran (`ask --lore v2`).
    pub new_dossiers: Vec<(String, String)>,
    /// `ask --lore v2`: answer a trait question with the named characters' game data (see [`Tools::game_data`]).
    pub game_data: bool,
    /// `ask --lore v2`: topic summaries use `summaryV2` where a trait audit regenerated one (2026-10-02).
    pub topics_v2_text: bool,
    /// With `topics_v2_text`: `summaryV2s` where the speaker-aware trait check (`topics.py traitcheck2`, 2026-10-03)
    /// wrote one (`ask --speaker-trait-check`).
    pub topics_speaker_check: bool,
    /// With `topics_v2_text`: `summaryV2r` where the race-trait rule (`topics.py traitrule`, 2026-10-03, no model) wrote
    /// one (`ask --race-trait-rule`).
    pub topics_race_rule: bool,
    /// Cast-wide cross-references from game data (`scripts/crossref.py`, 2026-10-03): table name to its JSON
    /// (`artifacts/crossref/<table>.json`). Read by the `cross_ref` tool, which only `ask --cross-ref` routes to.
    pub crossref: BTreeMap<String, Value>,
    /// Per character, the story groups where they speak and where they are only mentioned (`scripts/appearances.py`,
    /// 2026-10-03): `{"groups": {id: {name, kind, episode, order}}, "characters": {name: {forms, speaks, mentioned,
    /// namedIn}}}`. Read by the `appearances` tool.
    pub appearances: Option<Value>,
    /// Per topic, the story groups that name it most (`scripts/topic_groups.py`, 2026-10-03), and the P2 event summary
    /// of each group (`artifacts/p2/groups.jsonl`): the deeper topic answer serves the first 3 after the topic summary.
    pub topic_groups: Option<Value>,
    /// The nation of each place topic from data (`scripts/place_nation.py`, 2026-10-03 night 8).
    pub place_nations: Option<Value>,
    /// Operator design inspirations inferred from the game data (`scripts/design_infer.py`, 2026-10-04): one row per
    /// operator, `{operator, charId, alters, subjects: [{subject, category, why, confidence, evidence}]}`, each subject
    /// checked by a second model against the evidence it cites. Read by the `design_basis` tool, labelled as Trevor's
    /// inference.
    pub design_basis: Option<Vec<Value>>,
    /// The wiki's trivia rows (`crate::reference::wiki_design_basis`), loaded only for `ask --wiki-legacy`.
    pub design_basis_wiki: Option<Vec<Value>>,
    /// What the game shows about each operator's design (`scripts/design_infer.py evidence`, 2026-10-05): one row per
    /// operator, `{charId, operator, race, alters, text}`, the text being the race, recruitment text, body-feature
    /// sentences of the operator file, the model-written art description and the whole-image and tiled visual readings.
    /// Read by `ask --design-deduce` (shortlist by data, then Gemma matches features).
    pub design_evidence: Option<Vec<Value>>,
    /// Deep topic entries (`scripts/topic_deep.py`, 2026-10-03 night 9): topic name to the judged entry's served text
    /// (a map-reduce over the story groups, operator files, game records and dossiers naming the topic).
    pub deep_topics: HashMap<String, String>,
    /// Serve the deep entry instead of the summary where one exists (`ask --deep-topics`).
    pub topics_deep: bool,
    pub group_summaries: HashMap<String, String>,
    /// Identity links with their evidence (`artifacts/entities/identities.v2.json`, `ENTITY_V2=1 scripts/entities.py
    /// build`, 2026-10-03): (name, other name, relation, quote, where, stated). Read by `ask`'s identity notes.
    pub identity_links: Vec<(String, String, String, String, String, bool)>,
    pub opts: ToolOptions,
}

/// The category a who/which question asks its answer to belong to (`Tools::answer_constraint`, 2026-10-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerConstraint {
    /// "not playable", "NPC": the answer has no operator file.
    NonPlayable,
    /// "playable operator": the answer has one.
    Playable,
    /// "from Victoria", "Victoria operator": the operator file's nation or birthplace.
    Nation(String),
}

/// An Integrated Strategies ending a question asks about: (data run id, run name, numbered endings in the game's order as
/// (number, name), the asked ending's number).
pub type IsEnding = (String, String, Vec<(usize, String)>, Option<usize>);

/// The cross-reference tables `scripts/crossref.py` writes.
pub const CROSSREF_TABLES: &[&str] = &["bosses_playable"];

/// A read-before link of the game's Storylines view: ((id, label), (id, label)).
type Edge = ((String, String), (String, String));

/// A topic summary to add to retrieval as one more passage (the summaries are generated and not yet judged, so
/// they are never an answer on their own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicPassage {
    pub topic: String,
    pub text: String,
}

/// Words of a question about a body, a race feature, an ability or an age (`Tools::game_data`).
const TRAIT_WORDS: &[&str] = &["wing", "wings", "horn", "horns", "halo", "halos", "ear", "ears", "tail", "tails", "fur", "furry",
    "scales", "scale", "claws", "fangs", "antlers", "feathers", "hooves", "mane", "eyes", "pupils", "age", "aged", "old", "older",
    "oldest", "younger", "youngest", "years", "height", "tall", "taller", "race", "species", "fly", "flying", "swim", "lifespan",
    "immortal", "ageless", "ability", "abilities", "power", "powers", "body", "blood", "physiology", "anatomy", "biology"];

/// Body words of a trait question for which the game-data passage lists the races described with them.
const BODY_WORDS: &[&str] = &["wing", "wings", "horn", "horns", "halo", "halos", "ear", "ears", "tail", "tails", "fur", "scales",
    "claws", "fangs", "antlers", "feathers", "hooves", "mane"];

/// The topic rows the tool serves: every row not marked thin.
fn served_topics(rows: Vec<Value>) -> Vec<Value> {
    rows.into_iter().filter(|r| r.get("thin").is_none()).collect()
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

/// The design-question detector of the `design_basis` tool (2026-10-04): the normalized question and its content words
/// (4 letters or more, not a question or design word), or `None` when it is not a design question. "based on/off" counts
/// only at the end ("what is Lucilla based on"), before "a", "an", "any" or "real", or with "animal" or "creature" in the
/// question; "based on his inspection" and "Based on the conversation between ..." (2 gold questions) are not.
fn design_question(q: &str) -> Option<(String, Vec<String>)> {
    const CUES: &[&str] = &["inspired by", "inspiration", "inspirations", "motif", "motifs", "modeled after", "modelled after",
        "modeled on", "modelled on", "designed after", "design inspiration", "character design", "real life counterpart"];
    const STOP: &[&str] = &["what", "which", "whose", "where", "when", "operator", "operators", "animal", "animals", "based",
        "inspired", "inspiration", "inspirations", "motif", "motifs", "modeled", "modelled", "design", "designs", "designed",
        "name", "names", "named", "part", "with", "have", "their", "there", "they", "this", "that", "from", "about", "into",
        "upon", "does", "were", "would", "could", "should", "character", "characters", "arknights", "real", "world", "life",
        "creature", "creatures", "thing", "things", "kind", "type", "like", "looks", "also", "some", "other", "only", "most",
        "more", "many", "much", "know", "tell", "anyone", "someone", "something", "whom", "being", "been", "game", "games",
        "story", "lore", "skin", "outfit", "outfits", "plant", "object", "person", "people", "mythology", "myth"];
    let n = norm(q);
    let words: Vec<&str> = n.split(' ').collect();
    let based = words.windows(2).enumerate().any(|(i, w)| w[0] == "based" && matches!(w[1], "on" | "off" | "upon")
        && words.get(i + 2).is_none_or(|x| matches!(*x, "a" | "an" | "any" | "real")))
        || (contains_words(&n, "based") && ["animal", "animals", "creature", "creatures"].iter().any(|w| contains_words(&n, w)));
    if !based && !CUES.iter().any(|c| contains_words(&n, c)) {
        return None;
    }
    let keys = n.split(' ').filter(|w| w.chars().count() >= 4 && !STOP.contains(w)).map(str::to_owned).collect();
    Some((n, keys))
}

fn read_jsonl(path: &Path) -> Option<Vec<Value>> {
    Some(std::fs::read_to_string(path).ok()?.lines().filter_map(|l| serde_json::from_str(l).ok()).collect())
}

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().to_owned()
}

impl Tools {
    /// Load every table under `root` (the trevor directory). A missing table is not an error here: the tools
    /// that need it answer [`ToolError::Missing`].
    #[must_use]
    pub fn load(root: &Path) -> Self {
        let mut t = Self::default();
        if let Some(v) = read_json(&root.join("artifacts/chrono/timeline_v1.json")) {
            for g in v["groups"].as_array().into_iter().flatten() {
                if let Some(id) = g["groupId"].as_str() {
                    t.groups.insert(id.to_owned(), g.clone());
                }
            }
        }
        t.guide = read_json(&root.join("artifacts/chrono/reading_guide.json"));
        t.community = read_json(&root.join("data/reading_community.json"));
        t.first = read_json(&root.join("artifacts/chrono/first_appearance.json"));
        t.bank = read_jsonl(&root.join("artifacts/bank/bank.jsonl")).unwrap_or_default();
        t.deaths = read_jsonl(&root.join("artifacts/entities/deaths.jsonl"));
        t.real_names = read_jsonl(&root.join("artifacts/entities/real_names.jsonl"));
        t.attributes = read_jsonl(&root.join("artifacts/entities/operator_attributes.jsonl"));
        if let Some(v) = read_json(&root.join("artifacts/entities/identities.json")) {
            for i in v.as_array().into_iter().flatten() {
                let mut names = vec![s(i, "label")];
                names.extend(i["names"].as_array().into_iter().flatten().filter_map(|n| n.as_str().map(str::to_owned)));
                t.identities.push(names);
            }
        }
        t.storylines = read_json(&root.join("artifacts/chrono/storylines.json"));
        t.canon = read_json(&root.join("artifacts/canon/is_endings.json"));
        // A thin topic (too few passages name it, or a short or poorly supported summary; scripts/topics.py thin) is
        // never served; only topics from data (2026-10-01) are ever marked.
        t.topics = read_jsonl(&root.join("artifacts/topics/topics.jsonl")).map(served_topics);
        t.overview = read_jsonl(&root.join("artifacts/overview/overview.jsonl"));
        for table in CROSSREF_TABLES {
            if let Some(v) = read_json(&root.join(format!("artifacts/crossref/{table}.json"))) {
                t.crossref.insert((*table).to_owned(), v);
            }
        }
        t.appearances = read_json(&root.join("artifacts/appearances/appearances.json"));
        t.topic_groups = read_json(&root.join("artifacts/topics/topic_groups.json"));
        t.place_nations = read_json(&root.join("artifacts/topics/place_nation.json"));
        t.design_basis = read_jsonl(&root.join("artifacts/entities/design_infer.jsonl")).filter(|v| !v.is_empty());
        t.design_evidence = read_jsonl(&root.join("artifacts/entities/design_evidence.jsonl")).filter(|v| !v.is_empty());
        for d in read_jsonl(&root.join("artifacts/topics/deep.jsonl")).unwrap_or_default() {
            if let (Some(t_), Some(text)) = (d["topic"].as_str(), d["served"].as_str()) {
                t.deep_topics.insert(t_.to_owned(), text.to_owned());
            }
        }
        for g in read_jsonl(&root.join("artifacts/p2/groups.jsonl")).unwrap_or_default() {
            t.group_summaries.insert(s(&g, "groupId"), s(&g, "summary"));
        }
        if let Some(v) = read_json(&root.join("artifacts/entities/identities.v2.json")) {
            for i in v["identities"].as_array().into_iter().flatten() {
                for (name, ev) in i["evidence"].as_object().into_iter().flatten() {
                    for e in ev.as_array().into_iter().flatten() {
                        t.identity_links.push((name.clone(), s(e, "other"), s(e, "relation"), s(e, "quote"), s(e, "where"), true));
                    }
                }
            }
            for e in v["inferred"].as_array().into_iter().flatten() {
                t.identity_links.push((s(e, "name_a"), s(e, "name_b"), s(e, "relation"), s(e, "quote"), s(e, "where"), false));
            }
        }
        t
    }

    /// The topic a question names itself (2026-10-03, Ian's item 5: "Who are the three main factions in dossoles?" was
    /// routed to plain retrieval though Dossoles is a topic): a served topic's name or alias of 4 or more characters as
    /// whole words, case folded, when it is the only topic the question names (a name inside a longer one does not
    /// count), the topic is a nation, place, race or organization, and the question names no operator. None when the
    /// topic tool cannot serve it.
    #[must_use]
    pub fn named_topic(&self, q: &str) -> Option<TopicPassage> {
        let n = norm(q);
        let rows = self.topics.as_ref()?;
        let mut hits: Vec<(String, String, String)> = Vec::new();
        for r in rows {
            for f in std::iter::once(s(r, "topic")).chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned))) {
                let x = norm(&f);
                if x.chars().count() >= 4 && contains_words(&n, &x) {
                    hits.push((x, s(r, "topic"), s(r, "kind")));
                }
            }
        }
        let outer: Vec<&(String, String, String)> = hits.iter()
            .filter(|(x, t, _)| !hits.iter().any(|(y, u, _)| u != t && y.len() > x.len() && contains_words(y, x))).collect();
        let topics: BTreeSet<&str> = outer.iter().map(|h| h.1.as_str()).collect();
        if topics.len() != 1 || !matches!(outer[0].2.as_str(), "nation" | "place" | "race" | "organization") {
            return None;
        }
        if self.operator_names().iter().any(|o| { let x = norm(o); x.chars().count() >= 3 && contains_words(&n, &x) }) {
            return None;
        }
        self.topic(&outer[0].1).ok()
    }

    /// The nation topic of a place topic (2026-10-03, Ian's item 5: Dossoles's summary is about the city, and the three
    /// factions are in Bolívar's): the served nation topic whose name or aliases the place's summary writes most often
    /// (whole words, at least once); None for a topic that is not a place. A trade, not a derivation: no table gives a
    /// place's nation, and the summary's most-named nation is wrong for Chernobog (Yan, 2 mentions; it is Ursus).
    ///
    /// Since 2026-10-03 night 8 the nation comes from data when `from_data` is set and artifacts/topics/place_nation.json
    /// maps the place (scripts/place_nation.py: the birthplace table, else the nation story chunks naming the place name
    /// most by count x log lift; Chernobog: Ursus, 98 of 262 chunks); `from_data` false is the summary count above.
    #[must_use]
    pub fn place_nation(&self, topic: &str, from_data: bool) -> Option<TopicPassage> {
        let rows = self.topics.as_ref()?;
        if from_data {
            let place = rows.iter().find(|r| s(r, "topic") == topic && s(r, "kind") == "place")?;
            let nation = self.place_nations.as_ref()?[s(place, "topic")]["nation"].as_str()?.to_owned();
            return self.topic(&nation).ok();
        }
        let place = rows.iter().find(|r| s(r, "topic") == topic && s(r, "kind") == "place")?;
        let text = s(place, self.topic_text_key(place));
        let mut best: Option<(usize, String)> = None;
        for r in rows.iter().filter(|r| s(r, "kind") == "nation") {
            let n: usize = std::iter::once(s(r, "topic")).chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned)))
                .map(|f| text.match_indices(f.as_str()).filter(|(i, _)| {
                    let before = text[..*i].chars().next_back();
                    let after = text[i + f.len()..].chars().next();
                    !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
                }).count()).sum();
            if n > 0 && best.as_ref().is_none_or(|(m, _)| n > *m) {
                best = Some((n, s(r, "topic")));
            }
        }
        best.and_then(|(_, t)| self.topic(&t).ok())
    }

    /// The P2 event summaries of the story groups that name a topic most, at most `k` (`scripts/topic_groups.py`), as
    /// (group id, the group's name, summary).
    #[must_use]
    pub fn topic_events(&self, topic: &str, k: usize) -> Vec<(String, String, String)> {
        let Some(list) = self.topic_groups.as_ref().and_then(|v| v[topic].as_array()) else { return Vec::new() };
        list.iter().filter_map(|g| {
            let id = g["groupId"].as_str()?;
            let text = self.group_summaries.get(id)?;
            let name = self.groups.get(id).and_then(|x| x["name"].as_str()).map_or_else(|| id.to_owned(), str::to_owned);
            Some((id.to_owned(), name, text.clone()))
        }).take(k).collect()
    }

    /// The category a who/which question asks its answer to belong to (2026-10-05, `ask --answer-check`), read from the
    /// question's words alone: not playable ("not playable", "non-playable", "unplayable", "NPC"), playable, or from a
    /// nation of the attribute table ("from Victoria", "Victoria operator").
    #[must_use]
    pub fn answer_constraint(&self, q: &str) -> Option<AnswerConstraint> {
        let n = norm(q);
        // "NPC" alone constrains only when the question does not offer an operator as the alternative: "Which
        // operator/Rhodes Island NPC comes closest to being a Psychologist?" (real r055) accepts both.
        let npc = ["npc", "npcs"].iter().any(|w| contains_words(&n, w)) && !contains_words(&n, "operator");
        if npc || ["not playable", "non playable", "nonplayable", "unplayable", "not a playable"].iter().any(|w| contains_words(&n, w)) {
            return Some(AnswerConstraint::NonPlayable);
        }
        if contains_words(&n, "playable") {
            return Some(AnswerConstraint::Playable);
        }
        // A nation value that is an organization topic is an employer, not a homeland: "Which Rhodes Island operator
        // won the landship's annual chess tournament?" (gold u002) names the company every operator works for.
        let orgs: BTreeSet<String> = self.topics.iter().flatten().filter(|r| s(r, "kind") == "organization").map(|r| norm(&s(r, "topic"))).collect();
        let nations: BTreeSet<String> = self.attributes.iter().flatten().filter_map(|r| r["nation"].as_str())
            .filter(|x| !orgs.contains(&norm(x))).map(str::to_owned).collect();
        nations.into_iter().filter(|x| x.chars().count() >= 3).find(|x| {
            let nx = norm(x);
            contains_words(&n, &format!("from {nx}")) || contains_words(&n, &format!("{nx} operator")) || contains_words(&n, &format!("{nx} operators"))
        }).map(AnswerConstraint::Nation)
    }

    /// The playable operator a character name belongs to: an operator name, an operator's real name, or a name one
    /// identity link away from either.
    fn playable_of(&self, name: &str) -> Option<String> {
        let n = norm(name);
        let ops = self.operator_names();
        let direct = |x: &str| -> Option<String> {
            let nx = norm(x);
            ops.iter().find(|o| norm(o) == nx).map(|o| (*o).to_owned())
                .or_else(|| self.real_names.iter().flatten().find(|r| norm(&s(r, "realName")) == nx).map(|r| s(r, "name")))
        };
        direct(name).or_else(|| self.identity_links.iter().filter(|l| norm(&l.0) == n).find_map(|l| direct(&l.1)))
    }

    /// The first character name the first sentence of an answer gives (operator names, real names and identity-link names, whole words,
    /// longest first, not a name the question itself uses) and, when that character fails the question's category by
    /// the game data, the reason; `None` when the first name passes, the answer names no known character, or the
    /// category cannot be checked (a character with no operator file has no nation in the data).
    #[must_use]
    pub fn failing_answer_name(&self, q: &str, answer: &str, c: &AnswerConstraint) -> Option<(String, String)> {
        let nq = norm(q);
        let mut names: Vec<String> = self.operator_names().into_iter().map(str::to_owned)
            .chain(self.real_names.iter().flatten().map(|r| s(r, "realName")))
            .chain(self.identity_links.iter().flat_map(|l| [l.0.clone(), l.1.clone()]))
            .filter(|x| x.chars().count() >= 3 && x.chars().next().is_some_and(char::is_uppercase) && !contains_words(&nq, &norm(x)))
            .collect();
        names.sort_by_key(|x| std::cmp::Reverse(x.len()));
        names.dedup();
        // The first sentence names the answer; a later one may name a character only as context ("the text mentions
        // that Iris met with the elite operator Rosmontis", in a decline, 2026-10-05).
        let text = answer.trim_start().split(['\n']).next().unwrap_or_default();
        let text = text.find(". ").map_or(text, |i| &text[..=i]);
        let at = |f: &str| text.match_indices(f).find(|(i, _)| {
            let before = text[..*i].chars().next_back();
            let after = text[i + f.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        }).map(|(i, _)| i);
        // Earliest position wins; at one position the longest name ("Sakiko Togawa" over "Sakiko").
        let first = names.iter().filter_map(|x| at(x).map(|i| (i, std::cmp::Reverse(x.len()), x))).min()?.2.clone();
        let op = self.playable_of(&first);
        let reason = match (c, &op) {
            (AnswerConstraint::NonPlayable, Some(o)) if norm(o) == norm(&first) => format!("{first} is a playable operator"),
            (AnswerConstraint::NonPlayable, Some(o)) => format!("{first} is the playable operator {o}"),
            (AnswerConstraint::Playable, None) => format!("{first} has no operator file, so is not a playable operator"),
            (AnswerConstraint::Nation(x), Some(o)) => {
                let r = self.attributes.iter().flatten().find(|r| s(r, "name") == *o)?;
                let (nat, birth) = (s(r, "nation"), s(r, "birthplace"));
                if norm(&nat) == norm(x) || norm(&birth) == norm(x) {
                    return None;
                }
                format!("{first}'s operator file gives the nation {} and the birthplace {}",
                        if nat.is_empty() { "none" } else { &nat }, if birth.is_empty() { "none" } else { &birth })
            }
            _ => return None,
        };
        Some((first, reason))
    }

    /// Whether a question names a served nation, place, race or organization topic (a form of 4 letters or more as whole
    /// words): "Are Petrams related to Aegirs?" asks about peoples, not people (2026-10-05, the relation rule's gate).
    #[must_use]
    pub fn names_group_topic(&self, q: &str) -> bool {
        let n = norm(q);
        self.topics.iter().flatten().filter(|r| matches!(s(r, "kind").as_str(), "nation" | "place" | "race" | "organization"))
            .flat_map(|r| std::iter::once(s(r, "topic")).chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned))))
            .map(|f| norm(&f)).any(|f| f.chars().count() >= 4 && (contains_words(&n, &f) || contains_words(&n, &format!("{f}s"))))
    }

    /// Whether a question asks about a topic as a whole (2026-10-05, the deep-entry gate): it names a served topic and,
    /// besides that topic's forms and function words, has at most one content word of 3 letters or more. "Who are the
    /// sarkaz?" (0) and "What is sami guarding?" (1, guarding) are; "Do Liberi, as a culture, give their feathers as
    /// gifts?" (held-out h033) and "Do black Sankta halos adn energy wings mean something?" (real r079), the two answers
    /// the ungated deep entries lost on night 9, are not.
    #[must_use]
    pub fn broad_topic_question(&self, q: &str) -> bool {
        const STOP: &[&str] = &["what", "who", "whom", "which", "where", "when", "why", "how", "are", "is", "was", "were",
            "the", "do", "does", "did", "about", "tell", "know", "explain", "mean", "means", "they", "them", "their", "there",
            "this", "that", "these", "those", "and", "for", "with", "really", "exactly", "actually", "lore", "arknights",
            "can", "you", "someone", "anyone", "please", "all", "any", "much", "more", "exist"];
        let Some(rows) = self.topics.as_ref() else { return false };
        let n = norm(q);
        let mut rest = format!(" {n} ");
        let mut named = false;
        let mut forms: Vec<String> = rows.iter().flat_map(|r| std::iter::once(s(r, "topic"))
            .chain(r["aliases"].as_array().into_iter().flatten().filter_map(|a| a.as_str().map(str::to_owned))))
            .map(|f| norm(&f)).filter(|f| f.chars().count() >= 4).collect();
        forms.sort_by_key(|f| std::cmp::Reverse(f.len()));
        for f in &forms {
            let pad = format!(" {f} ");
            if rest.contains(&pad) {
                named = true;
                rest = rest.replace(&pad, " ");
            }
        }
        named && rest.split_whitespace().filter(|w| w.chars().count() >= 3 && !STOP.contains(w)).count() <= 1
    }

    /// The deep entry of a topic (`ask --deep-topics`), its citations stripped, when one of at least 100 words exists.
    #[must_use]
    pub fn deep_entry(&self, topic: &str) -> Option<String> {
        let t = self.deep_topics.get(topic).filter(|_| self.topics_deep)?;
        Some(strip_citations(t)).filter(|x| x.split_whitespace().count() >= 100)
    }

    /// Which summary field a topic row serves: `summary` (v1), `summaryV2` (`--lore v2`, where the trait audit
    /// regenerated one), `summaryV2s` (`--speaker-trait-check`, where the speaker-aware check wrote one).
    fn topic_text_key(&self, r: &Value) -> &'static str {
        if self.topics_v2_text && self.topics_race_rule && r.get("summaryV2r").is_some() {
            "summaryV2r"
        } else if self.topics_v2_text && self.topics_speaker_check && r.get("summaryV2s").is_some() {
            "summaryV2s"
        } else if self.topics_v2_text && r.get("summaryV2").is_some() {
            "summaryV2"
        } else {
            "summary"
        }
    }

    /// The 46 topics as before 2026-10-01 (`ask --lore v1`): the rows without a `source`, without the merged `names`
    /// the topics from data added to them.
    pub fn topics_v1(&mut self) {
        if let Some(rows) = self.topics.as_mut() {
            rows.retain(|r| r.get("source").is_none());
            for r in rows.iter_mut() {
                if let Some(o) = r.as_object_mut() {
                    // Only the fields a v1 row had: the trait audit added summaryV2, passagesV2 and their keys to old rows.
                    o.retain(|k, _| matches!(k.as_str(), "topic" | "kind" | "aliases" | "asked" | "passages" | "summary" | "inputSha" | "promptSha"));
                }
            }
        }
    }

    /// Load the dossiers of `artifacts/dossiers/dossiers.jsonl` that are not among the v1 ones
    /// (`dossiers.v1.jsonl`, which P3b holds), with their text as `build-profiles` writes a profile.
    pub fn load_new_dossiers(&mut self, root: &Path) {
        let v1: std::collections::HashSet<String> = read_jsonl(&root.join("artifacts/dossiers/dossiers.v1.jsonl"))
            .unwrap_or_default().iter().map(|r| s(r, "name")).collect();
        for d in read_jsonl(&root.join("artifacts/dossiers/dossiers.jsonl")).unwrap_or_default() {
            let name = s(&d, "name");
            if v1.contains(&name) {
                continue;
            }
            let aka: Vec<&str> = d["names"].as_array().into_iter().flatten().filter_map(Value::as_str).filter(|x| *x != name).collect();
            let head = if aka.is_empty() { format!("Character profile: {name}") }
                else { format!("Character profile: {name} (also known as {})", aka.join(", ")) };
            let text = format!("{head}\n{}", s(&d, "dossier"));
            self.new_dossiers.push((name, text));
        }
    }

    /// A game-data passage for a question about a named character's (or family's) body, race features, abilities or
    /// age: each named operator's race, birthplace, faction and height from the operator files, then the topic summary
    /// of each race (2026-10-02: "Do the Nearls have wings?" was declined by both lores, though the files list every
    /// Nearl as Kuranta and the Kuranta are horse-like). Names match as written (4 letters or more), folded from 6
    /// letters, and a family plural matches each operator whose first word it is ("Nearls": Nearl, Nearl the Radiant
    /// Knight). The trait test is a word list, a trade: no model call is spent on it.
    #[must_use]
    pub fn game_data(&self, q: &str) -> Option<TopicPassage> {
        if !self.game_data {
            return None;
        }
        let nq = norm(q);
        let words: Vec<&str> = nq.split(' ').collect();
        if !words.iter().any(|w| TRAIT_WORDS.contains(w)) && !nq.contains("how old") {
            return None;
        }
        let attrs = self.attributes.as_ref()?;
        let written = |name: &str| q.match_indices(name).any(|(i, _)| {
            !q[..i].chars().next_back().is_some_and(char::is_alphanumeric)
                && !q[i + name.len()..].chars().next().is_some_and(char::is_alphanumeric)
        });
        let mut hits: Vec<&Value> = attrs.iter().filter(|r| {
            let name = s(r, "name");
            let n = norm(&name);
            let first = n.split(' ').next().unwrap_or_default().to_owned();
            (name.chars().count() >= 4 && written(&name)) || (n.chars().count() >= 6 && contains_words(&nq, &n))
                || (first.chars().count() >= 4 && words.contains(&format!("{first}s").as_str()))
        }).collect();
        // A name inside a longer matched name ("Nearl" in "Nearl the Radiant Knight") is kept: both are the family.
        hits.truncate(6);
        if hits.is_empty() {
            return None;
        }
        let field = |r: &Value, k: &str| r[k].as_str().filter(|v| !v.is_empty()).map(str::to_owned)
            .or_else(|| r[k].as_u64().map(|v| v.to_string()));
        let mut text = String::from("Game data (the operator files' fields):");
        let mut races: Vec<String> = Vec::new();
        for r in &hits {
            let mut parts = Vec::new();
            for (k, label) in [("race", "race"), ("birthplace", "birthplace"), ("nation", "nation"), ("group", "faction"), ("heightCm", "height cm")] {
                if let Some(v) = field(r, k) {
                    parts.push(format!("{label} {v}"));
                }
            }
            text.push_str(&format!("\n- {}: {}", s(r, "name"), parts.join("; ")));
            if let Some(race) = field(r, "race").filter(|v| !matches!(v.as_str(), "Unknown" | "Undisclosed")) {
                if !races.contains(&race) {
                    races.push(race);
                }
            }
        }
        for race in &races {
            if let Ok(t) = self.topic(race) {
                let short: Vec<&str> = t.text.split_whitespace().take(120).collect();
                text.push_str(&format!("\nRace {race} (Trevor's topic summary, start): {}", short.join(" ")));
            }
        }
        // For each body word the question asks about, the races whose summaries describe it, with that sentence: "Do the
        // Nearls have wings?" needs to know that wings belong to the Sankta, not only what the Kuranta summary says.
        for w in words.iter().filter(|w| BODY_WORDS.contains(w)) {
            let stem = w.trim_end_matches('s');
            let mut with: Vec<String> = Vec::new();
            for r in self.topics.iter().flatten().filter(|r| s(r, "kind") == "race") {
                let body = s(r, self.topic_text_key(r));
                let sent = strip_citations(&body).split_inclusive(['.', '!', '?'])
                    .find(|x| norm(x).split(' ').any(|t| t.trim_end_matches('s') == stem)).map(|x| x.trim().to_owned());
                if let Some(x) = sent {
                    with.push(format!("{} (\"{x}\")", s(r, "topic")));
                }
            }
            text.push_str(&format!("\nRaces whose topic summaries mention {w}: {}", if with.is_empty() { "none".to_owned() } else { with.join("; ") }));
        }
        let names: Vec<String> = hits.iter().map(|r| s(r, "name")).collect();
        Some(TopicPassage { topic: names.join(", "), text })
    }

    /// The new dossier of the character the question names, if one does (the longest name wins). A dossier is matched
    /// by its own name only, never by identity-link names (those joined Estelle to Perfumer, 2026-10-02), and as written
    /// (case and all, whole words: "Red", "Grace" and "Mountain" are words; folded matching sent g0120's "on the
    /// mountain" to Mountain's dossier); a name of two or more words also matches folded ("innkeeper zheng"). A trade:
    /// a lower-case "silverash" gets no dossier.
    #[must_use]
    pub fn new_dossier(&self, q: &str) -> Option<TopicPassage> {
        let nq = norm(q);
        let as_written = |name: &str| q.match_indices(name).any(|(i, _)| {
            let before = q[..i].chars().next_back();
            let rest = &q[i + name.len()..];
            let after = rest.chars().next();
            // Not the start of a longer name: "Lin Qingyan" is not Lin (g0120, 2026-10-02).
            let longer = rest.strip_prefix(' ').and_then(|r| r.chars().next()).is_some_and(char::is_uppercase);
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) && !longer
        });
        self.new_dossiers.iter()
            .filter(|(name, _)| {
                let n = norm(name);
                // One-word names under 4 letters never match (Yu, Lin, Ace, May, Ji: "Deputy Minister Yu" is not Yu).
                (n.contains(' ') || n.chars().count() >= 4) && (as_written(name) || (n.contains(' ') && contains_words(&nq, &n)))
            })
            .max_by_key(|(name, _)| name.len())
            .map(|(name, text)| TopicPassage { topic: name.clone(), text: text.clone() })
    }

    /// Run a route's tool with its string arguments.
    ///
    /// # Errors
    /// An unknown tool, a missing or unresolvable argument, or no answer in the table.
    pub fn run(&self, r: &Route) -> ToolResult {
        match r.tool.as_str() {
            "reading_guide" => self.reading_guide(),
            "reading_event" => self.reading_event(r.arg("event")?),
            "reading_chronological" => self.reading_chronological(),
            "reading_time" => self.reading_time(r.arg("event")?),
            "reading_compare" => self.reading_compare(r.arg("event")?, r.arg("other")?),
            "first_appearance" => self.first_appearance(r.arg("character")?),
            "recap" => self.recap(r.arg("event")?, r.opt("ending").is_some_and(|v| v.eq_ignore_ascii_case("true"))),
            "timeline_all" => Ok(self.timeline_all()),
            "deaths_in_event" => {
                let who = match r.opt("who").map(str::to_lowercase).as_deref() {
                    None | Some("all") => Who::All,
                    Some("npc" | "npcs" | "non-playable") => Who::Npc,
                    Some("operators" | "operator" | "playable") => Who::Operators,
                    Some(v) => return Err(ToolError::BadArg { arg: "who", value: v.to_owned() }),
                };
                self.deaths_in_event(r.arg("event")?, who)
            }
            "death_of" => self.death_of(r.arg("character")?),
            "dead_operators" => self.dead_operators(),
            "real_name" => self.real_name(r.arg("operator")?),
            "real_names" => self.real_names(r.opt("place")),
            "operator_attribute" => self.operator_attribute(r.arg("operator")?, Field::parse(r.arg("field")?)?),
            "operator_filter" => self.operator_filter(&filter_from_args(r)?),
            "cross_ref" => self.cross_ref(r.arg("table")?),
            "appearances" => self.appearances(r.arg("character")?),
            "design_basis" => self.design_basis(r.arg("question")?)
                .ok_or_else(|| ToolError::NotFound { what: "design inspiration", name: r.arg("question").unwrap_or_default().to_owned() }),
            // The model router wrote {"run": "all"} for "any of the IS endings" (2026-09-29).
            "canon" => self.canon(r.opt("run").filter(|v| !matches!(norm(v).as_str(), "all" | "any" | "every" | "all runs"))),
            // Not an answer: `ask` adds the summary to retrieval (see `topic`); here it only reports the resolution.
            "topic" => self.topic(r.arg("topic")?).map(|p| format!("Topic passage for retrieval: {}", p.topic)),
            t => Err(ToolError::UnknownTool(t.to_owned())),
        }
    }

    // ---------------------------------------------------------------- argument resolution

    /// A main episode or event: a group id, "Episode 7" / "chapter 14" / "EP07" / "7", its name in any case,
    /// a text that contains one name (the longest wins), or a name within 1 edit (2 from 9 characters).
    ///
    /// # Errors
    /// [`ToolError::NotFound`] when nothing or more than one group matches.
    pub fn resolve_event(&self, name: &str) -> Result<&Value, ToolError> {
        let nf = || ToolError::NotFound { what: "event", name: name.to_owned() };
        if let Some(g) = self.groups.get(name.trim()) {
            return Ok(g);
        }
        let n = norm(name);
        if let Some(ch) = episode_number(&n) {
            return self.groups.values().find(|g| g["chapter"].as_u64() == Some(ch)).ok_or_else(nf);
        }
        // Operator records (story_*) are not events a player names.
        let events: Vec<(&Value, String)> = self.groups.values()
            .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
            .filter_map(|g| g["name"].as_str().map(|x| (g, norm(x)))).collect();
        let strip = |x: &str| x.strip_prefix("the ").unwrap_or(x).to_owned();
        if let Some((g, _)) = events.iter().find(|(_, gn)| *gn == n || strip(gn) == strip(&n)) {
            return Ok(g);
        }
        // Without a leading "the" and each word's trailing "s": "Masses Travel" is The Masses' Travels (the model
        // router wrote it so for Ian's question, 2026-09-30).
        let key = |x: &str| strip(x).split(' ').map(|w| if w.len() > 3 { w.trim_end_matches('s') } else { w }).collect::<Vec<_>>().join(" ");
        let nk = key(&n);
        let keyed: Vec<(&Value, String)> = events.iter().map(|(g, gn)| (*g, key(gn))).collect();
        if let Some((g, _)) = keyed.iter().find(|(_, k)| *k == nk) {
            return Ok(g);
        }
        // A longer text naming one event ("the Babel rerun").
        if let Some((g, _)) = events.iter().filter(|(_, gn)| gn.len() >= 4 && contains_words(&n, gn)).max_by_key(|(_, gn)| gn.len()) {
            return Ok(g);
        }
        // Part of one name ("Siracusano"), when only one event holds it.
        let part: Vec<&(&Value, String)> = events.iter().filter(|_| n.len() >= 5).filter(|(_, gn)| contains_words(gn, &n)).collect();
        if part.len() == 1 {
            return Ok(part[0].0);
        }
        fuzzy_unique(&n, events.iter().map(|(g, gn)| (*g, gn.as_str())))
            .or_else(|| fuzzy_unique(&nk, keyed.iter().map(|(g, k)| (*g, k.as_str())))).ok_or_else(nf)
    }

    /// A name among `cands`: exact, then case and punctuation folded, then through an identity link (Louisa is
    /// Kal'tsit), then within 1 edit (2 from 9 characters) when one candidate is closest.
    ///
    /// # Errors
    /// [`ToolError::NotFound`].
    pub fn resolve_name<'a>(&self, what: &'static str, name: &str, cands: &[&'a str]) -> Result<&'a str, ToolError> {
        let name = name.trim();
        if let Some(c) = cands.iter().find(|c| **c == name) {
            return Ok(c);
        }
        let n = norm(name);
        let by_norm = |x: &str| cands.iter().find(|c| norm(c) == x).copied();
        if let Some(c) = by_norm(&n) {
            return Ok(c);
        }
        for ident in self.identities.iter().filter(|i| i.iter().any(|x| norm(x) == n)) {
            if let Some(c) = ident.iter().find_map(|x| by_norm(&norm(x))) {
                return Ok(c);
            }
        }
        let normed: Vec<(&'a str, String)> = cands.iter().map(|c| (*c, norm(c))).collect();
        fuzzy_unique(&n, normed.iter().map(|(c, cn)| (*c, cn.as_str())))
            .ok_or_else(|| ToolError::NotFound { what, name: name.to_owned() })
    }

    fn guide_items(&self) -> Result<&Vec<Value>, ToolError> {
        self.guide.as_ref().and_then(|g| g["items"].as_array()).ok_or(ToolError::Missing("reading_guide.json"))
    }

    /// The reading-guide entry of an event or episode.
    fn guide_item(&self, event: &str) -> Result<&Value, ToolError> {
        let items = self.guide_items()?;
        let gid = self.resolve_event(event)?["groupId"].as_str().unwrap_or_default().to_owned();
        items.iter().find(|i| i["groupId"].as_str() == Some(gid.as_str()))
            .ok_or_else(|| ToolError::NotFound { what: "reading-guide entry", name: event.to_owned() })
    }

    fn attrs(&self) -> Result<&Vec<Value>, ToolError> {
        self.attributes.as_ref().ok_or(ToolError::Missing("operator_attributes.jsonl"))
    }

    /// Values of one attribute column, without the game's placeholders (the routes' dictionaries).
    #[must_use]
    pub fn attr_values(&self, k: &str) -> BTreeSet<String> {
        self.attributes.iter().flatten().map(|r| s(r, k)).filter(|v| v.len() >= 3 && !matches!(v.as_str(), "Unknown" | "Undisclosed"))
            .collect()
    }

    /// Affiliations and birthplaces, the values a place filter takes.
    #[must_use]
    pub fn places(&self) -> BTreeSet<String> {
        PLACE_KEYS.iter().flat_map(|k| self.attr_values(k)).collect()
    }

    /// Operator codenames of the attribute table.
    #[must_use]
    pub fn operator_names(&self) -> Vec<&str> {
        self.attributes.iter().flatten().filter_map(|r| r["name"].as_str()).filter(|n| !n.is_empty()).collect()
    }

    /// Whether the question names anything Trevor's tables know: an operator, a speaker, an event or episode, a topic
    /// (race, nation, concept, with its merged names), a place, or an identity-link name, as whole words after `norm`,
    /// three letters or more; or the word "arknights". `ask --lore-only` adds its off-topic sentence only when this is
    /// false (2026-10-03), so a lore question that names its subject keeps its prompt byte for byte.
    #[must_use]
    pub fn names_anything(&self, q: &str) -> bool {
        // Possessives too: `norm` drops the apostrophe, so "siege's" would read "sieges".
        let nq = norm(q);
        let np = norm(&q.replace("'s", " ").replace("\u{2019}s", " "));
        let hit = |n: &str| { let n = norm(n); n.chars().count() >= 3 && (contains_words(&nq, &n) || contains_words(&np, &n)) };
        if hit("arknights") || self.operator_names().into_iter().any(hit) || self.speaker_names().into_iter().any(hit)
            || self.places().iter().any(|p| hit(p)) || self.identities.iter().flatten().any(|n| hit(n)) {
            return true;
        }
        if self.groups.values().any(|g| g["name"].as_str().is_some_and(hit)) {
            return true;
        }
        self.topics.iter().flatten().any(|t| t["topic"].as_str().is_some_and(hit)
            || t["names"].as_array().into_iter().flatten().filter_map(Value::as_str).any(hit))
    }

    /// Named characters with a death event.
    #[must_use]
    pub fn death_names(&self) -> Vec<String> {
        self.deaths.iter().flatten().filter(|e| e["generic"].as_bool() != Some(true)).map(|e| s(e, "character"))
            .filter(|n| !n.is_empty()).collect::<BTreeSet<_>>().into_iter().collect()
    }

    /// Short names in a question that are the start of exactly one operator or identity-link name ("kal" for
    /// Kal'tsit): (the word as written, the full name). Words that are a whole name, or shorter than 3 letters, are
    /// left alone; the caller also checks that the word is rare in the corpus (a real word like "can" is not a name).
    #[must_use]
    pub fn short_names(&self, q: &str) -> Vec<(String, String)> {
        let mut names: Vec<String> = self.operator_names().into_iter().map(str::to_owned).collect();
        names.extend(self.identities.iter().flatten().cloned());
        let first = |n: &str| norm(n).split(' ').next().unwrap_or_default().to_owned();
        let firsts: HashSet<String> = names.iter().map(|n| first(n)).collect();
        let mut out = Vec::new();
        for w in q.split(|c: char| !(c.is_alphanumeric() || matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`' | '\u{b4}'))).filter(|w| !w.is_empty()) {
            let f = norm(w);
            if f.chars().count() < 3 || firsts.contains(&f) || f.chars().any(|c| c.is_ascii_digit()) {
                continue;
            }
            let hits: BTreeSet<&str> = names.iter().filter(|n| first(n).starts_with(&f)).map(String::as_str).collect();
            // One target, or several spellings of one person (an identity's names all start alike, as Kal'tsit's do).
            let targets: BTreeSet<String> = hits.iter().map(|n| first(n)).collect();
            if targets.len() == 1 {
                if let Some(full) = hits.iter().min_by_key(|n| n.len()) {
                    out.push((w.to_owned(), (*full).to_owned()));
                }
            }
        }
        out
    }

    /// Speakers with a first appearance.
    #[must_use]
    pub fn speaker_names(&self) -> Vec<&str> {
        self.first.as_ref().and_then(Value::as_object).map(|m| m.keys().map(String::as_str).collect()).unwrap_or_default()
    }

    // ---------------------------------------------------------------- listing

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
    fn story_edges(&self) -> Vec<Edge> {
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

    /// Which IS endings later stories continue from, per run, with one quote each. The game never labels an ending
    /// canon; this is inferred from later stories only (`scripts/canon_refs.py`).
    ///
    /// # Errors
    /// Missing table or a run that does not resolve.
    pub fn canon(&self, run: Option<&str>) -> ToolResult {
        let runs = self.canon.as_ref().and_then(|v| v["runs"].as_array()).ok_or(ToolError::Missing("is_endings.json"))?;
        if self.opts.is_offset && run.is_some_and(is_fungimist) {
            return Ok(FUNGIMIST.to_owned());
        }
        let chosen: Vec<&Value> = match run {
            Some(r) => vec![self.resolve_run(r, runs)?],
            None => runs.iter().collect(),
        };
        let offset = usize::from(self.opts.is_offset);
        // The per-run verdicts (scripts/canon_refs.py, 2026-09-30) when every chosen run has one; else the reference
        // counts, as before.
        if !self.opts.canon_v1 && chosen.iter().all(|r| r["verdict"].is_object()) {
            let mut out = String::from("The game never labels an ending canon; Trevor's reading of the stories:\n");
            for r in chosen {
                out.push('\n');
                out.push_str(&canon_verdict(r, offset));
            }
            return Ok(out);
        }
        let mut out = String::from("The game never labels an Integrated Strategies ending as canon. What follows is inferred from later stories that continue from or refer to an ending, with one quote each.\n");
        for r in chosen {
            out.push('\n');
            out.push_str(&canon_run(r, offset));
        }
        Ok(out)
    }

    /// "IS2", "IS #2", "Integrated Strategies 2", "2" (the game's numbers: IS #2 is rogue_1), "rogue_1" (the data's
    /// id), a run name ("Mizuki & Caerula Arbor") or a unique part of one ("Mizuki"), within the same misspelling
    /// budget as names.
    fn resolve_run<'a>(&self, r: &str, runs: &'a [Value]) -> Result<&'a Value, ToolError> {
        let nf = || ToolError::NotFound { what: "Integrated Strategies run", name: r.to_owned() };
        let n = norm(r);
        let digits: String = n.chars().filter(char::is_ascii_digit).collect();
        let numbered = n.split(' ').all(|w| matches!(w, "is" | "integrated" | "strategies" | "strategy" | "rogue" | "roguelike")
            || w.chars().all(|c| c.is_ascii_digit()) || w.strip_prefix("is").is_some_and(|x| x.chars().all(|c| c.is_ascii_digit())));
        if numbered && !digits.is_empty() {
            let id = match digits.parse::<usize>() {
                Ok(d) if self.opts.is_offset && !n.contains("rogue") => format!("rogue_{}", d.saturating_sub(1)),
                _ => format!("rogue_{digits}"),
            };
            return runs.iter().find(|x| s(x, "run") == id).ok_or_else(nf);
        }
        let names: Vec<(&Value, String)> = runs.iter().map(|x| (x, norm(&s(x, "runName")))).collect();
        if let Some((x, _)) = names.iter().find(|(_, rn)| *rn == n) {
            return Ok(x);
        }
        let part: Vec<&(&Value, String)> = names.iter().filter(|(_, rn)| n.len() >= 3 && (contains_words(rn, &n) || contains_words(&n, rn))).collect();
        if part.len() == 1 {
            return Ok(part[0].0);
        }
        fuzzy_unique(&n, names.iter().map(|(x, rn)| (*x, rn.as_str()))).ok_or_else(nf)
    }

    // ---------------------------------------------------------------- topics

    /// The summary of a race, nation or concept, its bracket citations stripped (they number passages the answer
    /// model does not see).
    ///
    /// # Errors
    /// Missing table or a topic that does not resolve.
    pub fn topic(&self, name: &str) -> Result<TopicPassage, ToolError> {
        let rows = self.topics.as_ref().ok_or(ToolError::Missing("topics.jsonl"))?;
        let nf = || ToolError::NotFound { what: "topic", name: name.to_owned() };
        let n = norm(name);
        // Topic names first, then aliases: "Beast Lord" is an alias of Feranmut and the singular of Beast Lords.
        let by_name = rows.iter().find(|r| { let t = norm(&s(r, "topic")); t == n || t == format!("{n}s") || format!("{t}s") == n });
        // `names`: forms the topics from data merged into a topic (demonyms, plurals, short forms), after every alias.
        let nn = n.as_str();
        let alias_in = |k: &'static str| move |r: &&Value| r[k].as_array().into_iter().flatten()
            .filter_map(Value::as_str).any(|a| { let a = norm(a); a == nn || format!("{a}s") == nn });
        let by_alias = || rows.iter().find(alias_in("aliases")).or_else(|| rows.iter().find(alias_in("names")));
        let names: Vec<(&Value, String)> = rows.iter().map(|r| (r, norm(&s(r, "topic")))).collect();
        let r = by_name.or_else(by_alias).or_else(|| fuzzy_unique(&n, names.iter().map(|(r, t)| (*r, t.as_str())))).ok_or_else(nf)?;
        let deep = self.deep_entry(&s(r, "topic"));
        let text = deep.unwrap_or_else(|| strip_citations(&s(r, self.topic_text_key(r))));
        // Summaries are 214 to 389 words except Pythia's 38, which is the generator asking for passages (2026-09-29
        // judge); under 100 words is treated as no summary.
        if text.split_whitespace().count() < 100 {
            return Err(ToolError::NoData(format!("no summary for {}", s(r, "topic"))));
        }
        Ok(TopicPassage { topic: s(r, "topic"), text })
    }

    // ---------------------------------------------------------------- overview

    /// The whole-story passages, citations stripped: the story overview, the world overview, then one summary per
    /// storyline (the main story first, then the game's order), each as (id, label, text).
    ///
    /// # Errors
    /// Missing table, or no story overview in it.
    pub fn overview_passages(&self) -> Result<Vec<(String, String, String)>, ToolError> {
        let rows = self.overview.as_ref().ok_or(ToolError::Missing("overview.jsonl"))?;
        let rank = |r: &Value| match (s(r, "kind").as_str(), s(r, "id").as_str()) {
            ("story", _) => 0,
            ("world", _) => 1,
            (_, "mainLine") => 2,
            _ => 3,
        };
        let mut v: Vec<&Value> = rows.iter().filter(|r| !s(r, "summary").trim().is_empty()).collect();
        v.sort_by_key(|r| rank(r));
        if v.first().is_none_or(|r| rank(r) != 0) {
            return Err(ToolError::NoData("no story overview".to_owned()));
        }
        Ok(v.into_iter().map(|r| (s(r, "id"), s(r, "name"), strip_citations(&s(r, "summary")))).collect())
    }

    // ---------------------------------------------------------------- recaps

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

    fn death_events(&self) -> Result<&Vec<Value>, ToolError> {
        self.deaths.as_ref().ok_or(ToolError::Missing("deaths.jsonl"))
    }

    fn latest(&self) -> String {
        self.groups.values().max_by_key(|g| g["releaseTime"].as_i64().unwrap_or(0))
            .and_then(|g| g["name"].as_str()).unwrap_or("?").to_owned()
    }

    /// Named deaths an event shows or reports, with quotes, role labels after, earlier deaths recalled.
    ///
    /// # Errors
    /// Missing table or an unknown event.
    pub fn deaths_in_event(&self, event: &str, who: Who) -> ToolResult {
        let events = self.death_events()?;
        let g = self.resolve_event(event)?;
        let gid = g["groupId"].as_str().unwrap_or_default();
        let name = g["name"].as_str().unwrap_or(gid);
        let (npc_only, ops_only) = (who == Who::Npc, who == Who::Operators);
        let mut seen = HashSet::new();
        let dies: Vec<&Value> = events.iter()
            .filter(|e| e["groupId"].as_str() == Some(gid) && e["kind"].as_str() == Some("dies"))
            .filter(|e| !(npc_only && e["playable"].as_bool() == Some(true)) && !(ops_only && e["playable"].as_bool() != Some(true)))
            .filter(|e| seen.insert(s(e, "character"))).collect();
        let (named, generic): (Vec<&Value>, Vec<&Value>) = dies.into_iter().partition(|e| e["generic"].as_bool() != Some(true));
        let who = if npc_only { "non-playable characters" } else if ops_only { "playable operators" } else { "characters" };
        let mut out = format!("Named {who} whose death {name} shows or reports ({}), each with the line that shows it. \
Trevor finds deaths from lines that say so; a death shown only by action can be missing.\n", named.len());
        for e in &named {
            out.push_str(&death_line(e)); out.push('\n');
        }
        if !generic.is_empty() {
            out.push_str(&format!("Also unnamed or role-labelled: {}.\n",
                generic.iter().map(|e| s(e, "character")).collect::<Vec<_>>().join(", ")));
        }
        let dying: HashSet<String> = events.iter()
            .filter(|e| e["groupId"].as_str() == Some(gid) && e["kind"].as_str() == Some("dies")).map(|e| s(e, "character")).collect();
        let earlier: Vec<String> = events.iter()
            .filter(|e| e["groupId"].as_str() == Some(gid) && e["kind"].as_str() == Some("dead") && !dying.contains(&s(e, "character")))
            .map(|e| s(e, "character")).collect::<BTreeSet<_>>().into_iter().collect();
        if !earlier.is_empty() {
            out.push_str(&format!("Mentioned as having died earlier: {}.\n", earlier.join(", ")));
        }
        Ok(out)
    }

    /// Playable operators whose death the game text confirms (`scripts/deaths.py build`, 2026-10-04: reported in 2 or more
    /// story groups and speaking in no story released after the first report; the wiki's status before, `opts.wiki_legacy`),
    /// and a count of the unconfirmed ones.
    ///
    /// # Errors
    /// Missing table.
    pub fn dead_operators(&self) -> ToolResult {
        let events = self.death_events()?;
        let latest = self.latest();
        let mut by_op: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
        for e in events.iter().filter(|e| e["playable"].as_bool() == Some(true) && e["kind"].as_str() == Some("dies")) {
            by_op.entry(s(e, "character")).or_default().push(e);
        }
        let (conf, unconf): (Vec<_>, Vec<_>) = by_op.iter().partition(|(_, v)| v.iter().any(|e| e["corroborated"].as_bool() == Some(true)));
        let how = if self.opts.wiki_legacy { "the script shows it and the Arknights wiki marks them dead" }
            else { "the script reports it in two or more stories and they speak in no story released after" };
        let mut out = format!("Playable operators whose death is confirmed ({how}), \
as of the stories in Trevor's corpus (latest: {latest}): {}. A later story can change this, and Trevor never marks anyone as alive.\n", conf.len());
        for (_, v) in &conf {
            out.push_str(&death_line(v[0])); out.push('\n');
        }
        if !unconf.is_empty() {
            // Not listed: on the full run these were rumors (W, Ines), titles (Graf Urtica) and misreadings
            // (Rosmontis "Are they here to take my life away?"), none marked dead by the wiki.
            let not = if self.opts.wiki_legacy { "that the wiki does not confirm" }
                else { "that the game text does not confirm (one story only, or they speak in a later story)" };
            out.push_str(&format!("The script also reports deaths for {} more operators {not}; most are \
rumors inside the story, titles passing between people, or misreadings. Ask about one operator (\"Does W die?\") to see the lines.\n",
                unconf.len()));
        }
        Ok(out)
    }

    /// A cast-wide cross-reference table (`scripts/crossref.py`). "bosses_playable": the boss enemies of the enemy
    /// handbook whose character is a playable operator, each with the table that made the match.
    ///
    /// # Errors
    /// An unknown table, or a table that has not been built.
    pub fn cross_ref(&self, table: &str) -> ToolResult {
        let name = CROSSREF_TABLES.iter().find(|t| norm(t) == norm(table))
            .ok_or_else(|| ToolError::BadArg { arg: "table", value: table.to_owned() })?;
        let v = self.crossref.get(*name).ok_or(ToolError::Missing("artifacts/crossref/bosses_playable.json (scripts/crossref.py)"))?;
        let rows = v["rows"].as_array().map(Vec::as_slice).unwrap_or_default();
        let mut by_op: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
        for r in rows {
            by_op.entry(s(r, "operator")).or_default().push(r);
        }
        let mut out = format!("Boss enemies you fight that are also playable operators, from the game data (the enemy handbook's {} boss \
entries matched by name to the {} obtainable operators, through the codename, the operator's real name or Trevor's identity links): \
{} operators. A match by name means the same character; a simulated or training version of an ally counts too, and a boss who \
joins later under another name is found only when a real name or identity link connects the two, so the list may miss some.\n",
            v["bosses"].as_u64().unwrap_or(0), v["operators"].as_u64().unwrap_or(0), by_op.len());
        for (op, rs) in &by_op {
            let bosses = rs.iter().map(|r| format!("\"{}\" (matched by {})", s(r, "boss"),
                s(r, "matchedBy").split(" (").next().unwrap_or_default())).collect::<Vec<_>>().join("; ");
            out.push_str(&format!("- {op}: {bosses}\n"));
        }
        Ok(out)
    }

    /// Where a character appears (2026-10-03, Ian's item 7): the story groups where they speak and, apart, those
    /// where they are only mentioned, in EN release order, from `artifacts/appearances/appearances.json`.
    ///
    /// # Errors
    /// Missing table, or a character with no speaking or mentioning chunk.
    pub fn appearances(&self, character: &str) -> ToolResult {
        let v = self.appearances.as_ref().ok_or(ToolError::Missing("artifacts/appearances/appearances.json (scripts/appearances.py)"))?;
        let chars = v["characters"].as_object().ok_or(ToolError::Missing("artifacts/appearances/appearances.json"))?;
        let keys: Vec<&str> = chars.keys().map(String::as_str).collect();
        let name = self.resolve_name("character", character, &keys)?;
        let c = &chars[name];
        let groups = &v["groups"];
        let list = |k: &str| -> Vec<(u64, String, u64)> {
            let mut rows: Vec<(u64, String, u64)> = c[k].as_object().into_iter().flatten().map(|(g, n)| {
                let gr = &groups[g.as_str()];
                (gr["order"].as_u64().unwrap_or(10_000), appearance_label(gr, g), n.as_u64().unwrap_or(0))
            }).collect();
            rows.sort();
            rows
        };
        let (speaks, mentioned) = (list("speaks"), list("mentioned"));
        if speaks.is_empty() && mentioned.is_empty() {
            return Err(ToolError::NotFound { what: "appearance", name: character.to_owned() });
        }
        let scenes = |n: u64| if n == 1 { "1 scene".to_owned() } else { format!("{n} scenes") };
        let mut out = format!("Where {name} appears in the story text Trevor has, from the scripts' speaker labels and name matches \
(game text, not a summary; a scene is a passage of about 400 tokens):\n\nSpeaks in {} stories, in EN release order:\n", speaks.len());
        for (_, l, n) in &speaks {
            out.push_str(&format!("- {l} ({})\n", scenes(*n)));
        }
        if !mentioned.is_empty() {
            out.push_str(&format!("\nMentioned by name without speaking in {}:\n", mentioned.len()));
            for (_, l, n) in &mentioned {
                out.push_str(&format!("- {l} ({})\n", scenes(*n)));
            }
        }
        let owners: BTreeSet<String> = c["namedIn"].as_array().into_iter().flatten().filter_map(Value::as_str)
            .filter_map(|id| self.source_owner(id)).filter(|o| norm(o) != norm(name)).collect();
        if !owners.is_empty() {
            out.push_str(&format!("\nAlso named in the operator files, voice lines, modules or outfits of: {}.\n",
                owners.into_iter().collect::<Vec<_>>().join(", ")));
        }
        out.push_str("\nNot counted: stories without speaker labels (Integrated Strategies, paradox simulations, which have no script \
in the game data), and appearances under a name the identity links do not connect.");
        Ok(out)
    }

    /// The operator whose file, voice lines, module or outfit a typed source id belongs to ("archive_char_4042_lumen" ->
    /// Lumen), from the attribute table's char ids; "profile_x" is Trevor's dossier of x.
    #[must_use]
    pub fn source_owner(&self, id: &str) -> Option<String> {
        if let Some(p) = id.strip_prefix("profile_") {
            return Some(p.to_owned());
        }
        let attrs = self.attributes.as_ref()?;
        attrs.iter().filter_map(|a| a["charId"].as_str().map(|c| (c, a)))
            .filter(|(c, _)| id.contains(*c) || id.split('_').next_back().is_some_and(|x| c.ends_with(&format!("_{x}")) && id.starts_with("module_")))
            .max_by_key(|(c, _)| c.len()).map(|(_, a)| s(a, "name"))
    }

    /// The character of a question that asks where someone appears ("What chapters does Elysium appear in?", "which
    /// events is Mudrock in", 2026-10-03): an appearance word and a story word, not "first" (the `first_appearance` tool's
    /// question), and one name of the appearance index (or an identity link to one) as whole words, the longest kept.
    #[must_use]
    pub fn appearance_question(&self, q: &str) -> Option<String> {
        let n = norm(q);
        let appear = ["appear", "appears", "appeared", "appearance", "appearances", "show up", "shows up", "showed up",
            "featured", "feature", "features", "present in"];
        let story = ["chapter", "chapters", "event", "events", "story", "stories", "episode", "episodes", "storyline", "storylines"];
        if !appear.iter().any(|w| contains_words(&n, w)) || !story.iter().any(|w| contains_words(&n, w)) || contains_words(&n, "first") {
            return None;
        }
        let chars = self.appearances.as_ref()?["characters"].as_object()?;
        let skip: BTreeSet<&str> = appear.iter().chain(story.iter()).copied().collect();
        let mut best: Option<&str> = None;
        let names = chars.keys().map(String::as_str).chain(self.identities.iter().flatten().map(String::as_str));
        for name in names {
            let x = norm(name);
            if x.chars().count() < 3 || skip.contains(x.as_str()) || !contains_words(&n, &x) {
                continue;
            }
            if best.is_none_or(|b| norm(b).len() < x.len()) {
                best = Some(name);
            }
        }
        best.map(str::to_owned)
    }

    /// The main episode or event a question names, for the scoped scenes (2026-10-05 night): "Episode 7" or "chapter 7"
    /// by number, else the longest event name the question holds as whole words, compared case and punctuation folded and
    /// without a leading "the", 6 characters or more ("in masses travels" is The Masses' Travels). Operator records
    /// (story_*) are not events a player names.
    #[must_use]
    pub fn event_in_question(&self, q: &str) -> Option<&Value> {
        let n = norm(q);
        let t: Vec<&str> = n.split(' ').collect();
        // An event named as a time boundary ("What happened to Mandragora after Episode 10?") is not the scope: the
        // question asks about what lies outside it (real r014 lost pairwise when scoped to Episode 10, 2026-10-05 night).
        let boundary = |w: Option<&&str>| w.is_some_and(|w| matches!(*w, "after" | "before" | "since" | "until" | "following"));
        for (i, w) in t.iter().enumerate() {
            if matches!(*w, "episode" | "chapter") && !boundary(i.checked_sub(1).and_then(|j| t.get(j))) {
                if let Some(x) = t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) {
                    if let Some(g) = self.groups.values().find(|g| g["chapter"].as_u64() == Some(x)) {
                        return Some(g);
                    }
                }
            }
        }
        self.groups.values()
            .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
            .filter_map(|g| {
                let gn = norm(g["name"].as_str()?);
                let s = gn.strip_prefix("the ").unwrap_or(&gn).to_owned();
                let at = format!(" {n} ").find(&format!(" {s} "))?;
                let before: Vec<&str> = n[..at.min(n.len())].split(' ').filter(|w| !w.is_empty() && *w != "the").collect();
                (s.len() >= 6 && !boundary(before.last())).then_some((g, s.len()))
            })
            .max_by_key(|(_, l)| *l).map(|(g, _)| g)
    }

    /// The characters a question names who speak or are named in story group `gid` by the appearance index, as (index
    /// name, every form: the index name and its identity-link names), for the scoped scenes. A form counts when the
    /// question holds it as whole words (case and punctuation folded, 3 characters or more) and it is not a word of the
    /// event's own name; whether it is a proper name is the caller's corpus check.
    #[must_use]
    pub fn event_characters(&self, q: &str, gid: &str, event_name: &str) -> Vec<(String, Vec<String>)> {
        let n = norm(q);
        let en = norm(event_name);
        let Some(chars) = self.appearances.as_ref().and_then(|v| v["characters"].as_object()) else { return Vec::new() };
        let mut out: Vec<(String, Vec<String>)> = Vec::new();
        for (name, c) in chars {
            // A race, nation, place or organization used as a speaker label ("Sarkaz", "Liberi") is not a character.
            if (c["speaks"].get(gid).is_none() && c["mentioned"].get(gid).is_none()) || self.names_group_topic(name) {
                continue;
            }
            let k = norm(name);
            let mut forms = vec![name.clone()];
            for ident in self.identities.iter().filter(|i| i.iter().any(|x| norm(x) == k)) {
                for x in ident {
                    if !forms.contains(x) {
                        forms.push(x.clone());
                    }
                }
            }
            let hit = forms.iter().any(|f| {
                let x = norm(f);
                x.chars().count() >= 3 && contains_words(&n, &x) && !contains_words(&en, &x)
            });
            if hit {
                out.push((name.clone(), forms));
            }
        }
        out.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
        out
    }

    /// A design-inspiration answer (2026-10-04, Ian's ian11), or `None` when the question is not one. It fires on a
    /// question with a design word ("inspired by", "inspiration", "motif", "modeled after", "design inspiration", or
    /// "based on/off" at the end, before "a"/"an"/"real", or with "animal" in the question) and reads it both ways: each
    /// content word of the question (4 letters or more, not a question or design word) is matched as a whole word against
    /// the inferred subjects of `design_infer.jsonl` (operators whose design draws on it) and against operator names (what
    /// their design draws on, with their game alters); an operator the question names with nothing inferred gets a line
    /// saying the game data does not show a subject. Labelled as Trevor's inference from the game data, never as a fact.
    /// With `opts.wiki_legacy` it is the wiki-trivia answer of the morning of 2026-10-04 (measurement only).
    #[must_use]
    pub fn design_basis(&self, q: &str) -> Option<String> {
        if self.opts.wiki_legacy {
            return self.design_basis_wiki(q);
        }
        let rows = self.design_basis.as_ref()?;
        let (n, keys) = design_question(q)?;
        let attrs = self.attributes.as_deref().unwrap_or_default();
        let subj_words = |sub: &Value| norm(sub["subject"].as_str().unwrap_or_default());
        let line = |r: &Value, sub: &Value| {
            let ev = sub["evidence"].as_array().into_iter().flatten().take(3)
                .map(|e| format!("{} ({}): \"{}\"", s(e, "id"), s(e, "source"), s(e, "text"))).collect::<Vec<_>>().join("; ");
            format!("- {}: likely draws on {} ({}, {} confidence). Clues: {} Evidence: {ev}", s(r, "operator"), s(sub, "subject"),
                    s(sub, "category"), s(sub, "confidence"), s(sub, "why"))
        };
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut parts: Vec<String> = Vec::new();
        for k in &keys {
            let named: Vec<String> = attrs.iter().map(|a| s(a, "name")).filter(|nm| contains_words(&norm(nm), k)).collect();
            let mut t = String::new();
            if about_char {
                for r in rows.iter().filter(|r| !named.contains(&s(r, "operator"))) {
                    for sub in r["subjects"].as_array().into_iter().flatten().filter(|sub| contains_words(&subj_words(sub), k)) {
                        t.push_str(&line(r, sub));
                        t.push('\n');
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators whose design Trevor infers draws on something named \"{k}\":\n{t}"));
            } else if about_char {
                parts.push(format!("No operator's inferred design names \"{k}\" (the game data may not show it).\n"));
            }
            let mut t = String::new();
            for nm in &named {
                let own: Vec<&Value> = rows.iter().filter(|r| s(r, "operator") == *nm).collect();
                let subs = |r: &Value| r["subjects"].as_array().cloned().unwrap_or_default();
                for r in &own {
                    if subs(r).is_empty() {
                        t.push_str(&format!("- {nm}: the game data does not point to a specific real-world subject.\n"));
                    }
                    for sub in &subs(r) {
                        t.push_str(&line(r, sub));
                        t.push('\n');
                    }
                }
                let alters: Vec<String> = own.first().map(|r| r["alters"].as_array().into_iter().flatten()
                    .filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
                for al in &alters {
                    for r in rows.iter().filter(|r| s(r, "operator") == *al) {
                        for sub in &subs(r) {
                            t.push_str(&format!("{} (the alternate operator of {nm})\n", line(r, sub)));
                        }
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators with \"{k}\" in their name, and what Trevor infers their design draws on:\n{t}"));
            }
        }
        // Only "no operator names it" lines: nothing to answer from, so the question goes on to the router as before.
        if parts.iter().all(|p| p.starts_with("No operator's")) {
            return None;
        }
        Some(format!("Trevor's inference from the game data, not stated by the game: the game never says what a design is \
based on, so these are deduced from each operator's file, skills, modules, name and the model-written descriptions of their \
art, and each was checked against the evidence it cites. Say they are inferences.\n\n{}", parts.join("\n")))
    }

    /// For `ask --design-deduce` (2026-10-05): `None` when the question is not a design question (the `design_basis`
    /// detector); else whether it asks which operator or character (the forward reading: "which operator is inspired by
    /// a jellyfish") and the evidence rows of the operators it names (the reverse reading: "what animal is Kirara based
    /// on?"). A question that asks for an operator is forward even when a word of it is an operator's name (ian11's
    /// "phantom").
    #[must_use]
    pub fn design_deduce_question(&self, q: &str) -> Option<(bool, Vec<&Value>)> {
        let rows = self.design_evidence.as_ref()?;
        let (n, _) = design_question(q)?;
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut named: Vec<&Value> = rows.iter().filter(|r| { let nm = norm(&s(r, "operator")); nm.len() >= 3 && contains_words(&n, &nm) }).collect();
        // "Texas the Omertosa" names Texas too: keep the longest names only
        let longest: Vec<String> = named.iter().map(|r| norm(&s(r, "operator"))).collect();
        named.retain(|r| { let nm = norm(&s(r, "operator")); !longest.iter().any(|o| o.len() > nm.len() && contains_words(o, &nm)) });
        Some((about_char, named))
    }

    /// Whether `phrase` is grounded in an evidence row (`--design-deduce`'s check of the matching step): it shares a word
    /// of 4 letters or more (plural folded) with the row's text that is not one of the generic words of the shortlist
    /// (in more than a quarter of the rows: "blue", "large", "body"), so a feature the model invents does not count.
    #[must_use]
    pub fn design_grounded(&self, row: &Value, phrase: &str) -> bool {
        let Some(rows) = self.design_evidence.as_ref() else { return false };
        let words = |t: &str| -> BTreeSet<String> {
            norm(t).split(' ').filter(|w| w.chars().count() >= 4)
                .map(|w| if w.len() > 4 && w.ends_with('s') && !w.ends_with("ss") { w[..w.len() - 1].to_owned() } else { w.to_owned() })
                .collect()
        };
        let have = words(&s(row, "text"));
        words(phrase).iter().filter(|w| have.contains(*w))
            .any(|w| rows.iter().filter(|r| words(&s(r, "text")).contains(w)).count() * 4 <= rows.len())
    }

    /// The `k` operators whose design evidence best matches `terms` (each a phrase with a weight: candidate names and
    /// groups 2, features 1): a phrase scores its weight times the inverse document frequency (over the evidence rows)
    /// of its rarest word the row contains, so one phrase counts once and a long row of generic words ("large black and
    /// white body") does not outscore a row naming the animal ("bear"). Words of 4 letters or more, a plural "s"
    /// folded; a word in more than a quarter of the rows never counts.
    #[must_use]
    pub fn design_shortlist(&self, terms: &[(String, f64)], k: usize) -> Vec<&Value> {
        let Some(rows) = self.design_evidence.as_ref() else { return Vec::new() };
        let words = |t: &str| -> BTreeSet<String> {
            norm(t).split(' ').filter(|w| w.chars().count() >= 4)
                .map(|w| if w.len() > 4 && w.ends_with('s') && !w.ends_with("ss") { w[..w.len() - 1].to_owned() } else { w.to_owned() })
                .collect()
        };
        let docs: Vec<BTreeSet<String>> = rows.iter().map(|r| words(&s(r, "text"))).collect();
        let mut df: HashMap<&str, usize> = HashMap::new();
        for d in &docs {
            for w in d {
                *df.entry(w.as_str()).or_default() += 1;
            }
        }
        let n = docs.len() as f64;
        let idf = |w: &str| { let f = *df.get(w).unwrap_or(&0) as f64; if f > n / 4.0 { 0.0 } else { (n / (1.0 + f)).ln().max(0.0) } };
        let phrases: Vec<(BTreeSet<String>, f64)> = terms.iter().map(|(t, wt)| (words(t), *wt)).collect();
        let mut scored: Vec<(f64, usize)> = docs.iter().enumerate().map(|(i, d)| {
            let sc: f64 = phrases.iter().map(|(ws, wt)| wt * ws.iter().filter(|w| d.contains(w.as_str())).map(|w| idf(w)).fold(0.0, f64::max)).sum();
            (sc, i)
        }).filter(|(sc, _)| *sc > 0.0).collect();
        scored.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        scored.into_iter().take(k).map(|(_, i)| &rows[i]).collect()
    }

    /// The wiki-trivia design answer of the morning of 2026-10-04 (`ask --wiki-legacy`, measurement only), unchanged:
    /// reads `design_basis_wiki`, loaded from `crate::reference` only under that flag.
    /// A design-inspiration answer (2026-10-04, Ian's ian11), or `None` when the question is not one. It fires on a
    /// question with a design word ("inspired by", "inspiration", "motif", "modeled after", "design inspiration", or
    /// "based on/off" at the end, before "a"/"an"/"real", or with "animal" in the question) and reads it both ways: each content word of the question (4 letters or more, not a question or design word) is
    /// matched as a whole word against the basis subjects of the wiki rows (operators based on it) and against
    /// operator names (what they are based on, with their game alters), and every reading that has rows is given, always
    /// labelled as the wiki's trivia, not the game text.
    #[must_use]
    fn design_basis_wiki(&self, q: &str) -> Option<String> {
        const CUES: &[&str] = &["inspired by", "inspiration", "inspirations", "motif", "motifs", "modeled after", "modelled after",
            "modeled on", "modelled on", "designed after", "design inspiration", "character design", "real life counterpart"];
        const STOP: &[&str] = &["what", "which", "whose", "where", "when", "operator", "operators", "animal", "animals", "based",
            "inspired", "inspiration", "inspirations", "motif", "motifs", "modeled", "modelled", "design", "designs", "designed",
            "name", "names", "named", "part", "with", "have", "their", "there", "they", "this", "that", "from", "about", "into",
            "upon", "does", "were", "would", "could", "should", "character", "characters", "arknights", "real", "world", "life",
            "creature", "creatures", "thing", "things", "kind", "type", "like", "looks", "also", "some", "other", "only", "most",
            "more", "many", "much", "know", "tell", "anyone", "someone", "something", "whom", "being", "been", "game", "games",
            "story", "lore", "skin", "outfit", "outfits", "plant", "object", "person", "people", "mythology", "myth"];
        let rows = self.design_basis_wiki.as_ref()?;
        let n = norm(q);
        // "based on/off" counts only as a design question's: at the end ("what is Lucilla based on"), before "a", "an",
        // "any" or "real", or with "animal" or "creature" in the question; "based on his inspection" and "Based on the
        // conversation between ..." (2 gold questions) are not.
        let words: Vec<&str> = n.split(' ').collect();
        let based = words.windows(2).enumerate().any(|(i, w)| w[0] == "based" && matches!(w[1], "on" | "off" | "upon")
            && words.get(i + 2).is_none_or(|x| matches!(*x, "a" | "an" | "any" | "real")))
            || (contains_words(&n, "based") && ["animal", "animals", "creature", "creatures"].iter().any(|w| contains_words(&n, w)));
        if !based && !CUES.iter().any(|c| contains_words(&n, c)) {
            return None;
        }
        let keys: Vec<&str> = n.split(' ').filter(|w| w.chars().count() >= 4 && !STOP.contains(w)).collect();
        let attrs = self.attributes.as_deref().unwrap_or_default();
        let line = |r: &Value| format!("- {}: based on or referencing {}. Wiki: \"{}\" ({})", s(r, "operator"),
            r["basis"].as_array().into_iter().flatten().filter_map(Value::as_str).collect::<Vec<_>>().join(", "), s(r, "sentence"), s(r, "url"));
        // The subject reading needs a question about a character ("Could Babel be inspired by the Tower of Babel?" asks
        // about an organization; 1 of 1,228 Reddit and Discord questions).
        let about_char = ["operator", "operators", "character", "characters", "who", "whose", "op", "ops"].iter().any(|w| contains_words(&n, w));
        let mut parts: Vec<String> = Vec::new();
        for k in &keys {
            let named: Vec<String> = attrs.iter().map(|a| s(a, "name")).filter(|nm| contains_words(&norm(nm), k)).collect();
            // An operator the name reading lists is not repeated under the subject reading.
            let by_subject: Vec<&Value> = rows.iter().filter(|_| about_char).filter(|r| !named.contains(&s(r, "operator")))
                .filter(|r| r["basis"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|b| contains_words(&norm(b), k))).collect();
            if !by_subject.is_empty() {
                let mut t = format!("Operators based on something named \"{k}\":\n");
                for r in by_subject {
                    t.push_str(&line(r));
                    t.push('\n');
                }
                parts.push(t);
            }
            let mut t = String::new();
            for nm in &named {
                let own: Vec<&Value> = rows.iter().filter(|r| s(r, "operator") == *nm).collect();
                let alters: Vec<String> = own.first().map(|r| r["alters"].as_array().into_iter().flatten()
                    .filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
                for r in &own {
                    t.push_str(&line(r));
                    t.push('\n');
                }
                for al in &alters {
                    for r in rows.iter().filter(|r| s(r, "operator") == *al) {
                        t.push_str(&format!("{} (the alternate operator of {nm})\n", line(r)));
                    }
                }
            }
            if !t.is_empty() {
                parts.push(format!("Operators with \"{k}\" in their name, and what the wiki says they are based on:\n{t}"));
            }
        }
        if parts.is_empty() {
            return None;
        }
        Some(format!("From the Arknights wiki's trivia pages (arknights.wiki.gg), not the game text: design inspirations are \
not stated in the story, so this is fan-written trivia, quoted as written.\n\n{}", parts.join("\n")))
    }

    /// The cross-reference table a question asks for without the router (2026-10-03, leftover d): `bosses_playable` for
    /// a question with "boss" or "bosses" and a word for joining or playing ("playable", "join", "operator", "recruit",
    /// "obtain", "ally", "side"). The router line for the table moved 34 unrelated routes; this test reads the question only.
    #[must_use]
    pub fn cross_ref_question(&self, q: &str) -> Option<&'static str> {
        let n = norm(q);
        let boss = contains_words(&n, "boss") || contains_words(&n, "bosses");
        let join = ["playable", "join", "joins", "joined", "joining", "operator", "operators", "recruit", "recruitable", "obtain",
            "obtainable", "ally", "allies", "our side", "your side", "rhodes island side"].iter().any(|w| contains_words(&n, w));
        (boss && join && self.crossref.contains_key("bosses_playable")).then_some("bosses_playable")
    }

    /// An Integrated Strategies ending a question asks about (2026-10-03, Ian's item 2): the run (an "IS" number,
    /// "Integrated Strategies" with a number, or a run's full name) and the ending (its number, "first" to "fifth", or its
    /// name), from `is_endings.json`. Returns (data run id, the run's numbered endings in the game's order as (number, name),
    /// the asked ending's number when the question names one). Endings without a number in their id are not numbered
    /// endings ("ro3_ending_c", The Fleetest of Light, has no script).
    #[must_use]
    pub fn is_ending_question(&self, q: &str) -> Option<IsEnding> {
        let n = norm(q);
        if !["ending", "endings", "endbook", "endbooks"].iter().any(|w| contains_words(&n, w)) {
            return None;
        }
        let runs = self.canon.as_ref()?["runs"].as_array()?;
        let offset = usize::from(self.opts.is_offset);
        let words: Vec<&str> = n.split(' ').collect();
        let mut run: Option<&Value> = None;
        for (i, w) in words.iter().enumerate() {
            let num = w.strip_prefix("is").filter(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit())).map(str::to_owned)
                .or_else(|| (matches!(*w, "is" | "strategies") && i + 1 < words.len()).then(|| words[i + 1].trim_start_matches('#').to_owned())
                    .filter(|x| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit()) && (*w == "strategies" || words[..i].iter().all(|p| *p != "what"))));
            if let Some(d) = num.and_then(|d| d.parse::<usize>().ok()) {
                let id = format!("rogue_{}", d.saturating_sub(offset));
                run = runs.iter().find(|r| s(r, "run") == id);
                if run.is_some() {
                    break;
                }
            }
        }
        let run = run.or_else(|| runs.iter().find(|r| { let rn = norm(&s(r, "runName")); rn.len() >= 5 && contains_words(&n, &rn) }))?;
        let mut endings: Vec<(usize, String)> = run["endings"].as_array().into_iter().flatten().filter_map(|e| {
            let k = s(e, "endingId").rsplit('_').next()?.parse::<usize>().ok()?;
            Some((k, s(e, "name")))
        }).collect();
        endings.sort();
        let ords = ["first", "second", "third", "fourth", "fifth", "sixth"];
        let asked = endings.iter().find(|(k, name)| {
            let x = norm(name);
            (x.len() >= 4 && contains_words(&n, &x))
                || [format!("ending {k}"), format!("ending #{k}"), format!("ending no {k}"), format!("{} ending", ords.get(k - 1).copied().unwrap_or("-")),
                    format!("e{k}"), format!("end {k}"), format!("ending{k}")].iter().any(|p| contains_words(&n, &norm(p)))
        }).map(|(k, _)| *k);
        Some((s(run, "run"), s(run, "runName"), endings, asked))
    }

    /// One character: every death event for them in release order, so the answer follows the story.
    ///
    /// # Errors
    /// Missing table or a character with no death event.
    pub fn death_of(&self, character: &str) -> ToolResult {
        let events = self.death_events()?;
        let names = self.death_names();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let who = self.resolve_name("character with a death event", character, &refs)?.to_owned();
        let latest = self.latest();
        let mut mine: Vec<&Value> = events.iter().filter(|e| s(e, "character") == who).collect();
        mine.sort_by_key(|e| e["releaseTime"].as_i64().unwrap_or(0));
        let mut out = format!("What the script says about {who}'s death, in release order (as of the stories in Trevor's corpus, latest: {latest}):\n");
        for e in &mine {
            let kind = if e["kind"].as_str() == Some("dies") { "dies" } else { "mentioned as dead" };
            out.push_str(&format!("- {kind} in {}: \"{}\"\n", s(e, "storyId"), s(e, "quote")));
        }
        if mine.iter().any(|e| e["playable"].as_bool() == Some(true) && e["corroborated"].as_bool() != Some(true)) {
            let why = if self.opts.wiki_legacy { "the wiki does not mark them dead" }
                else { "the game text does not confirm it (reported in one story only, or they speak in a story released after)" };
            out.push_str(&format!("This is a playable operator and {why}: treat these lines as unconfirmed. Stories often \
report deaths that turn out false (W and Ines are reported dead and are alive).\n"));
        }
        Ok(out)
    }

    // ---------------------------------------------------------------- real names

    fn names_table(&self) -> Result<&Vec<Value>, ToolError> {
        self.real_names.as_ref().ok_or(ToolError::Missing("real_names.jsonl"))
    }

    /// One operator's real name with the line that states it (`scripts/real_names.py`).
    ///
    /// # Errors
    /// Missing table, an unknown operator, or no real name in the table (retrieval may still find one).
    pub fn real_name(&self, operator: &str) -> ToolResult {
        let rows = self.names_table()?;
        let names: Vec<&str> = rows.iter().filter_map(|r| r["name"].as_str()).filter(|n| !n.is_empty()).collect();
        let who = self.resolve_name("operator", operator, &names)?;
        let r = rows.iter().rev().find(|r| r["name"].as_str() == Some(who)).ok_or_else(|| ToolError::NotFound { what: "operator", name: operator.to_owned() })?;
        r["realName"].is_string().then(|| format!("{}'s real name is {}. The line that states it: \"{}\" ({}).",
                                                  s(r, "name"), s(r, "realName"), s(r, "quote"), s(r, "storyId")))
            .ok_or_else(|| ToolError::NoData(format!("no real name for {who} in the table")))
    }

    /// Every real name in the table, optionally for operators affiliated with or born in `place`.
    ///
    /// # Errors
    /// Missing table or an unknown place.
    pub fn real_names(&self, place: Option<&str>) -> ToolResult {
        let rows = self.names_table()?;
        let named: Vec<&Value> = rows.iter().filter(|r| r["realName"].is_string()).collect();
        let line = |r: &Value| format!("- {}: {} (\"{}\", {})", s(r, "name"), s(r, "realName"), s(r, "quote"), s(r, "storyId"));
        // "Real names of every operator from Kazimierz": the place filter of the attribute table narrows the list.
        let (place, ids) = match place {
            Some(p) => {
                let p = self.resolve_place(p)?;
                let ids: HashSet<String> = self.attrs()?.iter().filter(|r| PLACE_KEYS.iter().any(|k| r[*k].as_str() == Some(p.as_str())))
                    .filter_map(|r| r["charId"].as_str().map(str::to_owned)).collect();
                (Some(p), Some(ids))
            }
            None => (None, None),
        };
        let named: Vec<&&Value> = named.iter().filter(|r| ids.as_ref().is_none_or(|ids| ids.contains(&s(r, "charId")))).collect();
        let scope = place.map_or(String::new(), |p| format!(" affiliated with or born in {p}"));
        let mut out = format!("Real names Trevor found for {} playable operators{scope} (of {} checked), each with the line that \
states it. Checked: each operator's file and up to 4 story passages that name them near a naming word, so a name revealed \
elsewhere can be missing; for the rest, those passages give no real name.\n", named.len(), rows.len());
        for r in &named {
            out.push_str(&line(r)); out.push('\n');
        }
        Ok(out)
    }

    // ---------------------------------------------------------------- game-data attributes
    // Roster questions answered from the game data (`scripts/attributes.py`): "Which operators are Sarkaz?",
    // "How many 6-star Casters are there?", "List all operators from Kazimierz", "What race is Texas?".

    fn resolve_place(&self, p: &str) -> Result<String, ToolError> {
        let places: Vec<String> = self.places().into_iter().collect();
        let refs: Vec<&str> = places.iter().map(String::as_str).collect();
        self.resolve_name("place", p, &refs).or_else(|e| contained_value(p, &refs).ok_or(e)).map(str::to_owned)
    }

    /// A value of `k` (race, branch), accepting a plural ("Sarkazs") or a value inside a longer text ("Fortress
    /// defenders": the model router passed the branch with its class on 2026-09-29).
    fn resolve_value(&self, what: &'static str, k: &str, v: &str) -> Result<String, ToolError> {
        let vals: Vec<String> = self.attr_values(k).into_iter().collect();
        let refs: Vec<&str> = vals.iter().map(String::as_str).collect();
        self.resolve_name(what, v, &refs)
            .or_else(|e| v.strip_suffix('s').map_or(Err(e.clone()), |x| self.resolve_name(what, x, &refs).map_err(|_| e)))
            .or_else(|e| contained_value(v, &refs).ok_or(e))
            .map(str::to_owned)
    }

    /// Operators matching every filter, listed with a count, or the 5 tallest or shortest of them.
    ///
    /// # Errors
    /// Missing table, no filter and no sort, or a value no operator has.
    pub fn operator_filter(&self, f: &Filter) -> ToolResult {
        let rows = self.attrs()?;
        type Pred = Box<dyn Fn(&Value) -> bool>;
        let mut filters: Vec<(String, Pred)> = Vec::new();
        // A value in the wrong field moves to the field it resolves in: the model router wrote {"place": "Soloblade"}
        // for "List all Soloblade operators" (2026-09-29). Only a place that is no place moves.
        let mut f = f.clone();
        if let Some(p) = f.place.clone().filter(|p| self.resolve_place(p).is_err()) {
            if f.branch.is_none() && self.resolve_value("branch", "branch", &p).is_ok() {
                f.branch = f.place.take();
            } else if f.race.is_none() && self.resolve_value("race", "race", &p).is_ok() {
                f.race = f.place.take();
            }
        }
        let f = &f;
        if let Some(p) = &f.place {
            let p = self.resolve_place(p)?;
            let pl = p.clone();
            filters.push((format!("affiliated with or born in {p}"), Box::new(move |r| PLACE_KEYS
                .iter().any(|k| r[*k].as_str() == Some(pl.as_str())))));
        }
        if let Some(race) = &f.race {
            let race = self.resolve_value("race", "race", race)?;
            let rl = race.clone();
            filters.push((format!("race {race}"), Box::new(move |r| r["race"].as_str().is_some_and(|x| x.split('/').any(|p| p.trim() == rl)))));
        }
        for c in &f.classes {
            let c = resolve_class(c)?;
            filters.push((format!("class {c}"), Box::new(move |r| r["class"].as_str() == Some(c))));
        }
        if let Some(b) = &f.branch {
            let b = self.resolve_value("branch", "branch", b)?;
            let bl = b.clone();
            filters.push((format!("branch {b}"), Box::new(move |r| r["branch"].as_str() == Some(bl.as_str()))));
        }
        if let Some(n) = f.rarity {
            if !(1..=6).contains(&n) {
                return Err(ToolError::BadArg { arg: "rarity", value: n.to_string() });
            }
            filters.push((format!("{n}-star"), Box::new(move |r| r["rarity"].as_i64() == Some(n))));
        }
        match f.female {
            Some(true) => filters.push(("female".into(), Box::new(|r| r["gender"].as_str() == Some("Female")))),
            Some(false) => filters.push(("male".into(), Box::new(|r| r["gender"].as_str() == Some("Male")))),
            None => {}
        }
        match f.infected {
            Some(false) => filters.push(("not infected".into(), Box::new(|r| r["infected"].as_bool() == Some(false)))),
            Some(true) => filters.push(("infected".into(), Box::new(|r| r["infected"].as_bool() == Some(true)))),
            None => {}
        }
        if filters.is_empty() && f.tallest.is_none() {
            return Err(ToolError::MissingArg("a filter or sort"));
        }
        let hit: Vec<&Value> = rows.iter().filter(|r| filters.iter().all(|(_, f)| f(r))).collect();
        let desc = filters.iter().map(|(d, _)| d.as_str()).collect::<Vec<_>>().join(", ");
        // Superlatives over height, the one ordered attribute the game data gives every operator (401 of 407).
        if let Some(tallest) = f.tallest {
            let w = if tallest { "tallest" } else { "shortest" };
            let mut h: Vec<&Value> = hit.into_iter().filter(|r| r["heightCm"].is_i64()).collect();
            h.sort_by_key(|r| r["heightCm"].as_i64().unwrap_or(0));
            if tallest {
                h.reverse();
            }
            let scope = if desc.is_empty() { "playable operators".to_owned() } else { format!("operators ({desc})") };
            let mut out = format!("The {w} {scope} by the height in their operator files ({} with a height). The files \
give the height as written, some approximate or in armor.\n", h.len());
            for r in h.iter().take(5) {
                out.push_str(&format!("- {}: {} cm\n", s(r, "name"), r["heightCm"].as_i64().unwrap_or(0)));
            }
            return Ok(out);
        }
        let mut out = format!("{} of the {} playable operators are {desc}, from the game data (character_table and each \
operator's file). The game leaves some fields Unknown or Undisclosed; those operators are not counted.\n", hit.len(), rows.len());
        for r in &hit {
            let why = if desc.contains("affiliated") {
                let aff = ["nation", "group", "team"].iter().map(|k| s(r, k)).filter(|v| !v.is_empty()).collect::<Vec<_>>().join(", ");
                format!(" (affiliation: {}; born in {})", if aff.is_empty() { "none".into() } else { aff },
                        Some(s(r, "birthplace")).filter(|b| !b.is_empty()).unwrap_or_else(|| "not given".into()))
            } else { String::new() };
            out.push_str(&format!("- {}{why}\n", s(r, "name")));
        }
        Ok(out)
    }

    /// One operator's attribute: "What race is Texas?", "When is Exusiai's birthday?".
    ///
    /// # Errors
    /// Missing table, an unknown operator, or a field the game leaves empty for them.
    pub fn operator_attribute(&self, operator: &str, field: Field) -> ToolResult {
        let rows = self.attrs()?;
        let names = self.operator_names();
        let who = self.resolve_name("operator", operator, &names)?;
        let r = rows.iter().rev().find(|r| r["name"].as_str() == Some(who)).ok_or_else(|| ToolError::NotFound { what: "operator", name: operator.to_owned() })?;
        let v = match field.key() {
            "heightCm" => r["heightCm"].as_i64().map(|h| format!("{h} cm")),
            "rarity" => r["rarity"].as_i64().map(|h| format!("{h} stars")),
            "nation" => Some(["nation", "group", "team"].iter().map(|k| s(r, k)).filter(|v| !v.is_empty()).collect::<Vec<_>>().join(", "))
                .filter(|v| !v.is_empty()),
            k => r[k].as_str().map(str::to_owned),
        }.ok_or_else(|| ToolError::NoData(format!("{who} has no {}", field.key())))?;
        let label = match field.key() { "heightCm" => "height", "infectionText" => "infection status", "nation" => "affiliation", k => k };
        Ok(format!("{}'s {label}, from their operator file in the game data: {v}.", s(r, "name")))
    }
}

/// IS #1, which the EN game data has only as the untranslated CN `roguelike_table.json` (no roguelike_topic_table
/// entry), so no ending text or later reference can be read.
const FUNGIMIST: &str = "Integrated Strategies #1, Ceobe's Fungimist, is not in the game data Trevor has: the EN data \
holds only an untranslated Chinese table for it, with no ending text, so Trevor cannot say which of its endings later \
stories continue from. The game never labels an Integrated Strategies ending as canon in any case. Trevor's IS data \
starts at #2, Phantom & Crimson Solitaire.";

fn game_order_agrees(game_first: &str, release_first: &str) -> bool {
    game_first == release_first
}

/// "IS1", "IS #1", "Integrated Strategies 1", "Fungimist", "Ceobe's Fungimist".
fn is_fungimist(r: &str) -> bool {
    let n = norm(r);
    if n.contains("fungimist") {
        return true;
    }
    let digits: String = n.chars().filter(char::is_ascii_digit).collect();
    digits == "1" && !n.contains("rogue")
        && n.split(' ').all(|w| matches!(w, "is" | "integrated" | "strategies" | "strategy" | "roguelike" | "1" | "is1"))
}

/// A run's heading with the game's number: "Mizuki & Caerula Arbor (Integrated Strategies #3)".
fn run_heading(r: &Value, offset: usize) -> String {
    let no = s(r, "run").strip_prefix("rogue_").and_then(|d| d.parse::<usize>().ok()).map_or_else(|| s(r, "run"), |d| (d + offset).to_string());
    format!("{} (Integrated Strategies #{no}):\n", s(r, "runName"))
}

/// The judge's reasoning for a reader: its passage numbers mean nothing outside the prompt, so "Passages [1], [2]
/// and [4]" becomes "The stories" (one: "One story"), other bracket numbers go, ending ids become names, and the
/// text is cut to its first 3 sentences.
fn reader_reasoning(text: &str, names: &HashMap<String, String>) -> String {
    let mut t = text.to_owned();
    for (id, name) in names {
        t = t.replace(&format!("{id} ({name})"), name).replace(id.as_str(), name);
    }
    let mut out = String::with_capacity(t.len());
    let mut rest = t.as_str();
    while let Some(i) = rest.find("assage") {
        let start = i.saturating_sub(1);
        let word_start = &rest[start..];
        let plural = word_start[1..].starts_with("assages");
        let after = &word_start[if plural { 8 } else { 7 }..];
        // A list of "[n]" joined by ", ", " and ", ", and ".
        let mut j = 0;
        let mut count = 0;
        loop {
            let tail = &after[j..];
            let sep = [", and ", " and ", ", ", " "].iter().find(|p| tail.starts_with(**p)).map_or(0, |p| p.len());
            let t2 = &tail[sep..];
            let digits = t2.chars().take_while(char::is_ascii_digit).count();
            if digits > 0 && (count == 0 || sep > 0) {
                j += sep + digits;
                count += 1;
                continue;
            }
            if t2.starts_with('[') {
                if let Some(close) = t2.find(']') {
                    if t2[1..close].chars().all(|c| c.is_ascii_digit() || c == ',' || c == ' ') && close > 1 {
                        j += sep + close + 1;
                        count += 1;
                        continue;
                    }
                }
            }
            break;
        }
        if count == 0 || !matches!(&rest[start..=start], "P" | "p") {
            out.push_str(&rest[..i + 6]);
            rest = &rest[i + 6..];
            continue;
        }
        let capital = &rest[start..=start] == "P";
        let phrase = match (count > 1 || plural, capital) {
            (true, true) => "The stories",
            (true, false) => "the stories",
            (false, true) => "One story",
            (false, false) => "one story",
        };
        out.push_str(&rest[..start]);
        out.push_str(phrase);
        rest = &after[j..];
    }
    out.push_str(rest);
    let clean = strip_citations(&out);
    // A sentence still about passage numbers ("while 3 and 4 describe local events") or about passages it set aside
    // ("Passages [3] and [8] were discounted", which reads as if every story were) is prompt bookkeeping; drop it.
    let sentences: Vec<&str> = clean.split_inclusive(". ").filter(|x| {
        let l = x.to_lowercase();
        !l.contains("passage") && !l.contains("discount")
    }).collect();
    sentences.iter().take(3).copied().collect::<String>().trim().to_owned()
}

/// One run from its verdict: the ending the stories treat as canon (or none, or undetermined) with the judge's
/// confidence and reasoning, whether the second model agrees, the stories relied on with a quote each, then every
/// other story linked to an ending. Hand grades (`grade`, `gradeNote`) are evaluation data and never printed.
fn canon_verdict(r: &Value, offset: usize) -> String {
    let v = &r["verdict"];
    let names: HashMap<String, String> = r["endings"].as_array().into_iter().flatten()
        .map(|e| (s(e, "endingId"), s(e, "name"))).filter(|(i, _)| !i.is_empty()).collect();
    let name_of = |id: &str| names.get(id).cloned().unwrap_or_else(|| id.to_owned());
    let mut out = run_heading(r, offset);
    let id = s(v, "endingId");
    let conf = s(v, "confidence");
    match id.as_str() {
        "none" => out.push_str("No ending is treated as canon by any story.\n"),
        "undetermined" | "" => out.push_str("Undetermined: the stories Trevor has do not settle which ending is canon.\n"),
        _ => out.push_str(&format!("Treated as canon: {} ({conf} confidence).\n", if s(v, "name").is_empty() { name_of(&id) } else { s(v, "name") })),
    }
    let why = reader_reasoning(&s(v, "reasoning"), &names);
    if !why.is_empty() {
        out.push_str(&format!("Why: {why}\n"));
    }
    let second = &v["second"];
    if second.is_object() {
        let sid = s(second, "endingId");
        let reading = match sid.as_str() {
            "none" => "no ending".to_owned(),
            "undetermined" | "" => "undetermined".to_owned(),
            x => format!("{} ({} confidence)", name_of(x), s(second, "confidence")),
        };
        if second["agrees"].as_bool() == Some(true) {
            out.push_str(&format!("A second model agrees: {reading}.\n"));
        } else {
            out.push_str(&format!("A second model disagrees: it reads {reading}.\n"));
        }
    } else {
        out.push_str("No second model has read this run.\n");
    }
    let evidence: Vec<&Value> = r["evidence"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    let date = |e: &Value| e["released"].as_str().unwrap_or("date unknown").to_owned();
    // A quote credited to several endings (Highmore's "Gone will be the fins" to Precious Days and Stella Caerula) is
    // printed once, under its own heading, and counts for neither ending's story lines.
    let mut endings_of: HashMap<(String, String), BTreeSet<String>> = HashMap::new();
    for e in &evidence {
        endings_of.entry((s(e, "story"), s(e, "quote"))).or_default().insert(s(e, "endingId"));
    }
    let shared = |e: &Value| endings_of.get(&(s(e, "story"), s(e, "quote"))).is_some_and(|x| x.len() > 1);
    // One line per (ending, story): the longest quote, cut at 200 characters, and how many other passages it has.
    let story_line = |items: &[&Value]| -> String {
        let quotes: BTreeSet<String> = items.iter().map(|e| s(e, "quote")).collect();
        let best = quotes.iter().max_by_key(|q| q.chars().count()).cloned().unwrap_or_default();
        let best = if best.chars().count() > 200 { format!("{}...", best.chars().take(200).collect::<String>().trim_end()) } else { best };
        let best = best.split_whitespace().collect::<Vec<_>>().join(" ");
        let more = quotes.len().saturating_sub(1);
        let more = if more > 0 { format!(" (+{more} more passage{})", if more == 1 { "" } else { "s" }) } else { String::new() };
        format!("\"{best}\"{more}")
    };
    let group = |ending: &str, story: &str| -> Vec<&Value> {
        evidence.iter().copied().filter(|e| s(e, "endingId") == ending && s(e, "story") == story && !shared(e)).collect()
    };
    let mut done: HashSet<(String, String)> = HashSet::new();
    let cites: Vec<String> = v["cites"].as_array().into_iter().flatten().filter_map(|c| c.as_str().map(str::to_owned)).collect();
    if !cites.is_empty() {
        out.push_str("Stories that treat it as canon:\n");
        for c in &cites {
            let items = group(&id, c);
            let when = items.first().map(|e| date(e))
                .or_else(|| evidence.iter().find(|e| s(e, "story") == *c).map(|e| date(e)));
            let when = when.map_or(String::new(), |d| format!(" ({d})"));
            if items.is_empty() {
                out.push_str(&format!("  - {c}{when}\n"));
            } else {
                out.push_str(&format!("  - {c}{when}: {}\n", story_line(&items)));
            }
            done.insert((id.clone(), c.clone()));
        }
    }
    // Every other (ending, story) pair, in the order the evidence lists them.
    let mut order: Vec<(String, String)> = Vec::new();
    for e in &evidence {
        let key = (s(e, "endingId"), s(e, "story"));
        if !shared(e) && !done.contains(&key) && !order.contains(&key) {
            order.push(key);
        }
    }
    if !order.is_empty() {
        out.push_str("Other stories linked to an ending (not relied on):\n");
        let mut endings: Vec<String> = Vec::new();
        for (e, _) in &order {
            if !endings.contains(e) {
                endings.push(e.clone());
            }
        }
        for ending in &endings {
            out.push_str(&format!("  {}:\n", name_of(ending)));
            for (_, story) in order.iter().filter(|(e, _)| e == ending) {
                let items = group(ending, story);
                out.push_str(&format!("    - {story} ({}): {}\n", date(items[0]), story_line(&items)));
            }
        }
    }
    let mut several: Vec<(&Value, String)> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for e in evidence.iter().filter(|e| shared(e)) {
        if seen.insert((s(e, "story"), s(e, "quote"))) {
            let names: Vec<String> = endings_of[&(s(e, "story"), s(e, "quote"))].iter().map(|x| name_of(x)).collect();
            several.push((e, names.join(", ")));
        }
    }
    if !several.is_empty() {
        out.push_str("Linked to several endings:\n");
        for (e, names) in &several {
            out.push_str(&format!("  - {} ({}), {names}: {}\n", s(e, "story"), date(e), story_line(&[*e])));
        }
    }
    out
}

/// One run's endings with the later stories that continue from each, and a strength label from the counts.
fn canon_run(r: &Value, offset: usize) -> String {
    let endings: Vec<&Value> = r["endings"].as_array().map(|a| a.iter().collect()).unwrap_or_default();
    let stories = |e: &Value| -> BTreeSet<String> {
        e["referencedBy"].as_array().into_iter().flatten().map(|x| s(x, "story")).filter(|x| !x.is_empty()).collect()
    };
    // A story credited to several endings of this run splits its evidence between them.
    let mut credited: HashMap<String, Vec<String>> = HashMap::new();
    for e in &endings {
        for st in stories(e) {
            credited.entry(st).or_default().push(s(e, "name"));
        }
    }
    let counts: Vec<(String, usize)> = endings.iter().map(|e| (s(e, "name"), stories(e).len())).collect();
    let max = counts.iter().map(|c| c.1).max().unwrap_or(0);
    let leaders: Vec<&(String, usize)> = counts.iter().filter(|c| c.1 == max).collect();
    let no = s(r, "run").strip_prefix("rogue_").and_then(|d| d.parse::<usize>().ok()).map_or_else(|| s(r, "run"), |d| (d + offset).to_string());
    let mut out = format!("{} (Integrated Strategies #{no}):\n", s(r, "runName"));
    if max > 0 && leaders.len() == 1 {
        out.push_str(&format!("Most continued: {} ({} later {}).\n", leaders[0].0, max, if max == 1 { "story" } else { "stories" }));
    } else if max > 0 {
        out.push_str("No ending is favored: the most-referenced endings are tied.\n");
    } else {
        out.push_str("No ending is favored: no later story refers to any of its endings.\n");
    }
    for e in &endings {
        let st = stories(e);
        let strength = match st.len() {
            0 => "no later story refers to it".to_owned(),
            1 => "one later story".to_owned(),
            k => format!("continued by {k} later stories"),
        };
        out.push_str(&format!("- {}: {strength}", s(e, "name")));
        let split: Vec<String> = st.iter().filter_map(|x| credited.get(x).filter(|v| v.len() > 1)
            .map(|v| format!("{x} is credited to {} as well, so its evidence is split between them",
                             v.iter().filter(|n| **n != s(e, "name")).cloned().collect::<Vec<_>>().join(" and ")))).collect();
        if !split.is_empty() {
            out.push_str(&format!("; {}", split.join("; ")));
        }
        out.push('\n');
        for st_name in &st {
            if let Some(q) = e["quotes"].as_array().into_iter().flatten().find(|q| s(q, "story") == *st_name) {
                out.push_str(&format!("  - {st_name} ({}): \"{}\"\n", s(q, "released"), s(q, "quote")));
            }
        }
    }
    out
}

/// Drop "[3]", "[2, 3, 5]" citations and the space before them.
fn strip_citations(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    let mut rest = t;
    while let Some(i) = rest.find('[') {
        let close = rest[i..].find(']').map(|j| i + j);
        match close {
            Some(j) if j > i + 1 && rest[i + 1..j].chars().all(|c| c.is_ascii_digit() || c == ',' || c == ' ') => {
                out.push_str(rest[..i].trim_end_matches(' '));
                rest = &rest[j + 1..];
            }
            _ => {
                out.push_str(&rest[..=i]);
                rest = &rest[i + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// A community placement rule agrees with a game link when it names the other story on the same side ("before
/// Episode 14" for Babel -> Episode 14).
fn rule_matches_edge(r: &Value, edges: &[Edge]) -> bool {
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

/// An appearance-index group as a reader names it: "Episode 7: The Birth of Tragedy", "operator record of Elysium
/// 'Long Journey'", or the event's name.
fn appearance_label(g: &Value, id: &str) -> String {
    let name = g["name"].as_str().unwrap_or(id);
    let kind = g["kind"].as_str().unwrap_or_default();
    match (kind, g["episode"].as_u64()) {
        ("main", Some(e)) => format!("Episode {e}: {name}"),
        (k, _) if k.starts_with("operator record") => format!("{k}, '{name}'"),
        _ => name.to_owned(),
    }
}

fn guide_label(i: &Value) -> String {
    if s(i, "kind") == "main" {
        format!("Episode {} {}", i["episode"].as_u64().unwrap_or(0), s(i, "name"))
    } else { s(i, "name") }
}

fn death_line(e: &Value) -> String {
    format!("- {}: \"{}\" ({})", s(e, "character"), s(e, "quote"), s(e, "storyId"))
}

fn resolve_class(c: &str) -> Result<&'static str, ToolError> {
    let n = norm(c);
    CLASSES.iter().find(|k| { let k = k.to_lowercase(); n == k || n == format!("{k}s") }).copied()
        .ok_or_else(|| ToolError::BadArg { arg: "class", value: c.to_owned() })
}

/// The filter of an `operator_filter` route. `class` may hold several classes separated by commas.
///
/// # Errors
/// A value that is not a number, a boolean, a gender or a sort.
pub fn filter_from_args(r: &Route) -> Result<Filter, ToolError> {
    let bool_arg = |k: &'static str| -> Result<Option<bool>, ToolError> {
        r.opt(k).map(|v| match v.to_lowercase().as_str() {
            "true" | "yes" | "infected" => Ok(true),
            "false" | "no" | "not infected" | "uninfected" => Ok(false),
            _ => Err(ToolError::BadArg { arg: k, value: v.to_owned() }),
        }).transpose()
    };
    Ok(Filter {
        place: r.opt("place").map(str::to_owned),
        race: r.opt("race").map(str::to_owned),
        classes: r.opt("class").map(|c| c.split(',').map(|x| x.trim().to_owned()).filter(|x| !x.is_empty()).collect()).unwrap_or_default(),
        branch: r.opt("branch").map(str::to_owned),
        rarity: r.opt("rarity").map(|v| v.trim_end_matches(['*', ' ']).trim_end_matches("-star").trim_end_matches(" star").parse::<i64>()
            .map_err(|_| ToolError::BadArg { arg: "rarity", value: v.to_owned() })).transpose()?,
        female: r.opt("gender").map(|v| match v.to_lowercase().as_str() {
            "female" | "women" | "woman" => Ok(true),
            "male" | "men" | "man" => Ok(false),
            _ => Err(ToolError::BadArg { arg: "gender", value: v.to_owned() }),
        }).transpose()?,
        infected: bool_arg("infected")?,
        tallest: r.opt("sort").map(|v| match v.to_lowercase().as_str() {
            "tallest" => Ok(true),
            "shortest" => Ok(false),
            _ => Err(ToolError::BadArg { arg: "sort", value: v.to_owned() }),
        }).transpose()?,
    })
}

/// Lowercase, accents folded, apostrophes dropped ("Ch'en" is "chen"), other punctuation as spaces, spaces single.
#[must_use]
pub fn norm(x: &str) -> String {
    let mut out = String::with_capacity(x.len());
    for c in x.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            'š' => 's',
            c => c,
        };
        if matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`' | '\u{b4}') {
            continue;
        }
        out.push(if c.is_alphanumeric() { c } else { ' ' });
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `hay` contains `needle` as whole words (both already normalized).
#[must_use]
pub fn contains_words(hay: &str, needle: &str) -> bool {
    !needle.is_empty() && format!(" {hay} ").contains(&format!(" {needle} "))
}

/// The longest value `text` holds as whole words (a plural "s" allowed), for an argument with extra words.
fn contained_value<'a>(text: &str, vals: &[&'a str]) -> Option<&'a str> {
    let n = norm(text);
    vals.iter().filter(|v| { let vn = norm(v); vn.len() >= 3 && (contains_words(&n, &vn) || contains_words(&n, &format!("{vn}s"))) })
        .max_by_key(|v| v.len()).copied()
}

/// "episode 7", "chapter 14", "ep 7", "ep07", "ch14", "main 7" or a bare "7" in a normalized name.
fn episode_number(n: &str) -> Option<u64> {
    let t: Vec<&str> = n.split(' ').collect();
    if t.len() == 1 {
        if let Ok(x) = t[0].parse() {
            return Some(x);
        }
    }
    for (i, w) in t.iter().enumerate() {
        if matches!(*w, "episode" | "chapter" | "ep" | "ch" | "main") {
            if let Some(x) = t.get(i + 1).and_then(|x| x.parse().ok()) {
                return Some(x);
            }
        }
        for p in ["episode", "chapter", "ep", "ch"] {
            if let Some(x) = w.strip_prefix(p).and_then(|x| x.parse().ok()) {
                return Some(x);
            }
        }
    }
    None
}

/// The one candidate within the edit budget with the smallest distance; none on a tie or past the budget.
/// Budget: 0 under 5 characters (Ines is not Inez), 1 under 9, 2 from 9.
fn fuzzy_unique<'a, T: Copy>(n: &str, cands: impl Iterator<Item = (T, &'a str)>) -> Option<T> {
    let len = n.chars().count();
    let budget = if len < 5 { 0 } else if len < 9 { 1 } else { 2 };
    if budget == 0 {
        return None;
    }
    let mut best: Option<(usize, T)> = None;
    let mut tie = false;
    for (c, cn) in cands {
        if cn.chars().count().abs_diff(len) > budget {
            continue;
        }
        let d = levenshtein(n, cn);
        if d > budget {
            continue;
        }
        match best {
            Some((bd, _)) if d > bd => {}
            Some((bd, _)) if d == bd => tie = true,
            _ => { best = Some((d, c)); tie = false; }
        }
    }
    if tie { None } else { best.map(|(_, c)| c) }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            cur[j + 1] = (prev[j] + usize::from(ca != *cb)).min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tools() -> Tools {
        let mut t = Tools::default();
        for (id, name, ch) in [("act33side", "Babel", None), ("act21side", "IL Siracusano", None), ("main_7", "The Birth of Tragedy", Some(7)),
                               ("main_14", "Absolved Will Be the Seekers", Some(14)), ("act29side", "Zwillingstürme im Herbst", None),
                               ("act42side", "The Masses' Travels", None), ("story_x", "Babel Records", None)] {
            let mut g = json!({"groupId": id, "name": name});
            if let Some(c) = ch {
                g["chapter"] = json!(c);
            }
            t.groups.insert(id.to_owned(), g);
        }
        t.identities = vec![vec!["Kal'tsit".into(), "AMa-10".into(), "Louisa".into(), "Kal'tsit".into()]];
        t.attributes = Some(vec![
            json!({"name": "Texas", "charId": "c1", "race": "Lupo", "heightCm": 161, "nation": "Lungmen", "class": "Vanguard"}),
            json!({"name": "Texas the Omertosa", "charId": "c2", "race": "Lupo", "heightCm": 161, "class": "Specialist"}),
            json!({"name": "Kal'tsit", "charId": "c3", "race": "Feline", "heightCm": 162, "class": "Medic"}),
            json!({"name": "Flamebringer", "charId": "c4", "race": "Sarkaz", "heightCm": 185, "class": "Guard", "branch": "Dreadnought"}),
        ]);
        t
    }

    fn gid(t: &Tools, n: &str) -> Option<String> {
        t.resolve_event(n).ok().map(|g| g["groupId"].as_str().unwrap_or_default().to_owned())
    }

    #[test]
    fn events_resolve_by_name_number_and_misspelling() {
        let t = tools();
        assert_eq!(gid(&t, "act33side").as_deref(), Some("act33side"));
        assert_eq!(gid(&t, "Babel").as_deref(), Some("act33side"));
        assert_eq!(gid(&t, "babel").as_deref(), Some("act33side"));
        assert_eq!(gid(&t, "Il Siracusano").as_deref(), Some("act21side"));
        assert_eq!(gid(&t, "Il Siracusanno").as_deref(), Some("act21side"));
        assert_eq!(gid(&t, "Siracusano").as_deref(), Some("act21side"));
        assert_eq!(gid(&t, "Episode 7").as_deref(), Some("main_7"));
        assert_eq!(gid(&t, "chapter 14").as_deref(), Some("main_14"));
        assert_eq!(gid(&t, "EP07").as_deref(), Some("main_7"));
        assert_eq!(gid(&t, "Birth of Tragedy").as_deref(), Some("main_7"));
        assert_eq!(gid(&t, "Zwillingsturme im Herbst").as_deref(), Some("act29side"));
        assert_eq!(gid(&t, "the Masses Travels").as_deref(), Some("act42side"));
        assert_eq!(gid(&t, "the Babel rerun").as_deref(), Some("act33side"));
        assert_eq!(gid(&t, "Masses Travel").as_deref(), Some("act42side"));
        assert_eq!(gid(&t, "masses travels").as_deref(), Some("act42side"));
        assert_eq!(gid(&t, "The Masse's Travel").as_deref(), Some("act42side"));
        assert_eq!(gid(&t, "ch 14").as_deref(), Some("main_14"));
        assert_eq!(gid(&t, "chapter 14").as_deref(), Some("main_14"));
    }

    #[test]
    fn unknown_events_are_errors_not_guesses() {
        let t = tools();
        assert!(matches!(t.resolve_event("Episode 99"), Err(ToolError::NotFound { .. })));
        assert!(matches!(t.resolve_event("Rhodes Island"), Err(ToolError::NotFound { .. })));
        assert!(t.resolve_event("Babl").is_err(), "4 letters: no fuzzy budget");
    }

    #[test]
    fn names_resolve_exactly_then_through_identities_then_fuzzily() {
        let t = tools();
        let names = t.operator_names();
        assert_eq!(t.resolve_name("operator", "Texas", &names), Ok("Texas"));
        assert_eq!(t.resolve_name("operator", "texas the omertosa", &names), Ok("Texas the Omertosa"));
        assert_eq!(t.resolve_name("operator", "Kaltsit", &names), Ok("Kal'tsit"));
        assert_eq!(t.resolve_name("operator", "Louisa", &names), Ok("Kal'tsit"));
        assert_eq!(t.resolve_name("operator", "Flamebringr", &names), Ok("Flamebringer"));
        assert!(t.resolve_name("operator", "Amiya", &names).is_err());
        assert!(t.resolve_name("operator", "Texa", &names).is_err());
    }

    #[test]
    fn tools_answer_and_fail_cleanly() {
        let t = tools();
        let r = t.run(&Route::new("operator_attribute", &[("operator", "texas"), ("field", "height")])).unwrap();
        assert!(r.contains("161 cm"), "{r}");
        assert!(matches!(t.run(&Route::new("operator_attribute", &[("operator", "Texas"), ("field", "birthday")])), Err(ToolError::NoData(_))));
        assert!(matches!(t.run(&Route::new("operator_attribute", &[("operator", "Texas"), ("field", "mood")])), Err(ToolError::BadArg { .. })));
        let r = t.run(&Route::new("operator_filter", &[("race", "Sarkaz"), ("sort", "tallest")])).unwrap();
        assert!(r.contains("Flamebringer: 185 cm"), "{r}");
        let r = t.run(&Route::new("operator_filter", &[("class", "medics")])).unwrap();
        assert!(r.starts_with("1 of the 4"), "{r}");
        assert!(t.run(&Route::new("operator_filter", &[])).is_err(), "no filter would list every operator");
        assert!(t.run(&Route::new("operator_filter", &[("race", "Elf")])).is_err());
        let r = t.run(&Route::new("operator_filter", &[("place", "Dreadnought")])).unwrap();
        assert!(r.contains("branch Dreadnought"), "{r}");
        let r = t.run(&Route::new("operator_filter", &[("branch", "Dreadnought guards")])).unwrap();
        assert!(r.contains("branch Dreadnought"), "{r}");
        assert!(matches!(t.run(&Route::new("death_of", &[("character", "W")])), Err(ToolError::Missing(_))));
        assert!(matches!(t.run(&Route::new("fly", &[])), Err(ToolError::UnknownTool(_))));
    }

    /// The shipped tables, when present (artifacts/ is git-ignored, so this passes vacuously without them).
    #[test]
    fn question_detectors_fire_on_their_questions_only() {
        let t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
        if t.appearances.is_none() || t.canon.is_none() || !t.crossref.contains_key("bosses_playable") {
            return;
        }
        assert_eq!(t.appearance_question("What chapters does Elysium appear in?").as_deref(), Some("Elysium"));
        assert_eq!(t.appearance_question("When does Elysium first appear?"), None, "the first_appearance tool's question");
        assert_eq!(t.appearance_question("Who is Elysium?"), None);
        // The scoped scenes' detectors (2026-10-05 night): the event a question names, without "the" or the apostrophe,
        // and the characters of the appearance index who appear in it.
        let q = "what is the name of the npc that mon3tr suggests assassinating in masses travels";
        let g = t.event_in_question(q).expect("The Masses' Travels");
        assert_eq!(g["groupId"].as_str(), Some("act42side"));
        assert!(t.event_characters(q, "act42side", "The Masses' Travels").iter().any(|(n, _)| n == "Mon3tr"));
        assert_eq!(t.event_in_question("Who does Mandragora fight in Episode 10?").and_then(|g| g["chapter"].as_u64()), Some(10));
        assert!(t.event_in_question("What happened to Mandragora after Episode 10?").is_none(), "a time boundary is not the scope");
        assert!(t.event_in_question("Who is the best cook?").is_none());
        let a = t.appearances("Elysium").unwrap();
        assert!(a.contains("Episode 16") && a.contains("Stultifera Navis") && a.contains("Mentioned by name without speaking"), "{a}");
        assert_eq!(t.cross_ref_question("Which bosses later join Rhodes Island as playable operators?"), Some("bosses_playable"));
        assert_eq!(t.cross_ref_question("Who is the final boss of Episode 8?"), None);
        let (run, _, endings, k) = t.is_ending_question("What happens in IS4 Ending 2 (including its endbooks)?").unwrap();
        assert_eq!((run.as_str(), k), ("rogue_3", Some(2)));
        assert_eq!(endings.iter().map(|e| e.0).collect::<Vec<_>>(), vec![1, 2, 3, 4], "ro3_ending_c is not a numbered ending");
        assert_eq!(t.is_ending_question("what is the second ending of Integrated Strategies 3").map(|x| (x.0, x.3)), Some(("rogue_2".into(), Some(2))));
        assert!(t.is_ending_question("How does Episode 8 end?").is_none());
        assert_eq!(t.named_topic("Who are the three main factions in dossoles?").map(|p| p.topic).as_deref(), Some("Dossoles"));
        assert_eq!(t.place_nation("Dossoles", false).map(|p| p.topic).as_deref(), Some("Bolívar"));
        assert!(t.place_nation("Sarkaz", false).is_none(), "a race is not a place");
        assert!(t.place_nation("Sarkaz", true).is_none(), "a race is not a place");
        if t.place_nations.is_some() {
            assert_eq!(t.place_nation("Chernobog", true).map(|p| p.topic).as_deref(), Some("Ursus"));
            assert_eq!(t.place_nation("Dossoles", true).map(|p| p.topic).as_deref(), Some("Bolívar"));
        }
    }

    #[test]
    fn real_tables_resolve_player_names() {
        let t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
        if t.groups.is_empty() || t.attributes.is_none() || t.real_names.is_none() || t.deaths.is_none() {
            return;
        }
        for (name, id) in [("Il Siracusano", "act21side"), ("Babel", "act33side"), ("Episode 7", "main_7"), ("chapter 14", "main_14"),
                           ("Zwillingsturme im Herbst", "act29side"), ("Such Is the Joy of Our Reunion", "act40side"),
                           ("Darknight Memoir", "act9d0"), ("the Masses Travels", "act42side")] {
            assert_eq!(gid(&t, name).as_deref(), Some(id), "{name}");
        }
        let run = |tool: &str, args: &[(&str, &str)]| t.run(&Route::new(tool, args));
        assert!(run("operator_attribute", &[("operator", "texas"), ("field", "height")]).unwrap().contains("161 cm"));
        assert!(run("real_name", &[("operator", "Bagpipe")]).unwrap().contains("Fiona Young"));
        assert!(matches!(run("real_name", &[("operator", "Amiya")]), Err(ToolError::NoData(_))));
        assert!(run("death_of", &[("character", "patriot")]).unwrap().contains("Patriot"));
        assert!(run("deaths_in_event", &[("event", "Babel"), ("who", "all")]).unwrap().contains("Theresa"));
        assert!(run("reading_event", &[("event", "Babel")]).unwrap().contains("Episode 13"));
        assert!(matches!(run("recap", &[("event", "Rhodes Island"), ("ending", "false")]), Err(ToolError::NotFound { .. })));
        assert!(matches!(run("operator_attribute", &[("operator", "Theresa"), ("field", "race")]), Err(ToolError::NotFound { .. })));
        if t.canon.is_some() {
            let mut t = Tools::load(Path::new(env!("CARGO_MANIFEST_DIR")));
            t.opts.canon_v1 = true;
            let run = |tool: &str, args: &[(&str, &str)]| t.run(&Route::new(tool, args));
            let r = run("canon", &[("run", "IS4")]).unwrap();
            assert!(r.contains("no later story refers to any of its endings"), "{r}");
            // The counts move with the evidence scheme (CANON_EVIDENCE in scripts/canon_refs.py); the framing does not.
            let r = run("canon", &[("run", "IS3")]).unwrap();
            assert!(r.starts_with("The game never labels") && r.contains("Stella Caerula"), "{r}");
        }
        if t.topics.is_some() {
            assert_eq!(t.topic("Sarkazs").unwrap().topic, "Sarkaz");
            assert!(!t.topic("Sami").unwrap().text.contains('['));
            assert!(t.topic("Pythia").is_err(), "Pythia's summary is the generator asking for passages");
        }
        if t.canon.as_ref().is_some_and(|c| c["runs"][0]["verdict"].is_object()) {
            for run_ in ["IS3", "IS2"] {
                let r = run("canon", &[("run", run_)]).unwrap();
                if std::env::var("TREVOR_PRINT_CANON").is_ok() {
                    println!("=== {run_}\n{r}");
                }
                assert!(r.starts_with("The game never labels an ending canon; Trevor's reading of the stories:"), "{r}");
                assert!(!r.contains("grade") && !r.contains("[1]"), "{r}");
            }
        }
        if t.storylines.is_some() {
            let r = run("reading_compare", &[("event", "Masses Travel"), ("other", "chapter 15")]).unwrap();
            assert!(r.contains("Dissociative Recombination") && r.contains("Storylines view places"), "{r}");
            assert!(t.reading_guide().unwrap().contains("The game's Storylines view places Babel before Episode 14"));
        }
    }

    #[test]
    fn canon_labels_strength_splits_and_ties_from_the_counts() {
        let mut t = Tools::default();
        let q = |st: &str| json!({"story": st, "released": "2025-01-01", "quote": format!("line from {st}")});
        let e = |name: &str, sts: &[&str]| json!({"name": name, "referencedBy": sts.iter().map(|x| json!({"story": x})).collect::<Vec<_>>(),
                                                   "quotes": sts.iter().map(|x| q(x)).collect::<Vec<_>>()});
        t.canon = Some(json!({"runs": [
            {"run": "rogue_1", "runName": "Phantom & Crimson Solitaire", "endings": [e("A", &["Act or Die"]), e("B", &["Act or Die"]), e("C", &[])]},
            {"run": "rogue_2", "runName": "Mizuki & Caerula Arbor", "endings": [e("Stella", &["module", "Episode 15"]), e("D", &[])]},
        ]}));
        let r1 = t.canon(Some("IS2")).unwrap();
        assert!(r1.starts_with("The game never labels"), "{r1}");
        assert!(r1.contains("No ending is favored") && r1.contains("A: one later story; Act or Die is credited to B as well"), "{r1}");
        assert!(r1.contains("C: no later story refers to it"), "{r1}");
        let r2 = t.canon(Some("mizuki")).unwrap();
        assert!(r2.contains("Most continued: Stella (2 later stories)") && r2.contains("Stella: continued by 2 later stories"), "{r2}");
        assert!(t.canon(Some("Integrated Strategies #3")).unwrap() == r2);
        assert!(t.canon(Some("rogue_2")).unwrap() == r2, "a data id is not shifted");
        assert!(t.canon(Some("IS1")).unwrap().contains("Ceobe's Fungimist, is not in the game data"));
        assert!(t.canon(Some("Fungimist")).unwrap().starts_with("Integrated Strategies #1"));
        assert!(matches!(t.canon(Some("IS9")), Err(ToolError::NotFound { .. })));
        let all = t.canon(None).unwrap();
        assert!(all.contains("Phantom & Crimson Solitaire (Integrated Strategies #2)") && all.contains("(Integrated Strategies #3)"), "{all}");
        t.opts.is_offset = false;
        assert!(t.canon(None).unwrap().contains("Phantom & Crimson Solitaire (Integrated Strategies #1)"), "the old numbering");
        assert!(t.canon(Some("IS1")).unwrap().contains("Phantom"));
    }

    #[test]
    fn topics_resolve_by_name_then_alias_and_lose_their_citations() {
        let t = Tools { topics: Some(vec![json!({"topic": "Beast Lords", "aliases": ["Beast Lord", "Feranmut"], "summary": format!("Old [1]. Very old [2, 3].{}", " x".repeat(100))}),
                             json!({"topic": "Feranmut", "aliases": ["Feranmut", "Sui", "Beast Lord"], "summary": format!("Sui [4].{}", " x".repeat(100))}),
                             json!({"topic": "Pythia", "aliases": ["Pythia"], "summary": "Please provide the numbered passages [1]."}),
                             json!({"topic": "Sami", "aliases": ["Sami"], "summary": format!("Cold [x] land [9].{}", " snow".repeat(100))})]), ..Tools::default() };
        assert_eq!(t.topic("beast lord").unwrap().topic, "Beast Lords");
        assert_eq!(t.topic("Sui").unwrap().topic, "Feranmut");
        assert!(t.topic("Beast Lords").unwrap().text.starts_with("Old. Very old. x"));
        assert!(matches!(t.topic("Pythia"), Err(ToolError::NoData(_))), "a summary under 100 words is no summary");
        assert!(t.topic("sami").unwrap().text.starts_with("Cold [x] land. snow"));
        assert!(t.topic("Kazimierz").is_err());
    }

    #[test]
    fn deep_entries_replace_the_summary_only_when_switched_on_and_long_enough() {
        let mut t = Tools { topics: Some(vec![json!({"topic": "Sami", "aliases": ["Sami"], "summary": format!("Cold [1].{}", " snow".repeat(100))}),
                                         json!({"topic": "Sarkaz", "aliases": ["Sarkaz"], "summary": format!("Horned [2].{}", " x".repeat(100))})]),
                            ..Tools::default() };
        t.deep_topics.insert("Sami".into(), format!("What it is\nA northern land [1, 2].{}", " ice".repeat(100)));
        t.deep_topics.insert("Sarkaz".into(), "What it is\nToo short [1].".into());
        assert!(t.topic("sami").unwrap().text.starts_with("Cold. snow"), "off: the summary, as before");
        t.topics_deep = true;
        assert!(t.topic("sami").unwrap().text.starts_with("What it is\nA northern land. ice"));
        assert!(t.topic("Sarkaz").unwrap().text.starts_with("Horned."), "a deep entry under 100 words falls back to the summary");
        assert!(t.deep_entry("Sarkaz").is_none() && t.deep_entry("Sami").is_some());
        // The 2026-10-05 gate: only a question about the topic as a whole.
        assert!(t.broad_topic_question("Who are the sarkaz?") && t.broad_topic_question("What is sami guarding?"));
        assert!(!t.broad_topic_question("Do Sarkaz, as a culture, give their horns as gifts?"));
        assert!(!t.broad_topic_question("Who is Amiya?"), "names no topic");
        let g = Tools { topics: Some(vec![json!({"topic": "Aegir", "kind": "race", "aliases": ["Ægir"]}), json!({"topic": "Arts", "kind": "concept", "aliases": []})]),
                        ..Tools::default() };
        assert!(g.names_group_topic("Are Petrams related to Aegirs?") && !g.names_group_topic("Is Kristen related to Friston?"));
        assert!(!g.names_group_topic("Are Arts related to Originium?"), "a concept topic is not a group of people");
    }

    #[test]
    fn answer_check_reads_the_category_from_the_question_and_the_names_from_the_data() {
        let a = |n: &str, nation: &str, birth: &str| json!({"name": n, "nation": nation, "birthplace": birth});
        let t = Tools { attributes: Some(vec![a("Sakiko Togawa", "", "Unknown"), a("Iris", "Victoria", "Victoria"), a("Rosmontis", "Rhodes Island", "Victoria"),
                                              a("Amiya", "Rhodes Island", "Rim Billiton")]),
                        real_names: Some(vec![json!({"name": "Rosmontis", "realName": "Rosa"})]),
                        topics: Some(vec![json!({"topic": "Rhodes Island", "kind": "organization", "aliases": []})]),
                        identity_links: vec![("Mabel".into(), "Bluishsilver".into(), "former_name".into(), String::new(), String::new(), true)],
                        ..Tools::default() };
        let q = "Which Rhodes Island Operator (not playable unit) had visited Iris' castle of dreams?";
        assert_eq!(t.answer_constraint(q), Some(AnswerConstraint::NonPlayable));
        assert_eq!(t.answer_constraint("Which operator/Rhodes Island NPC comes closest to being a Psychologist?"), None, "operator or NPC");
        assert_eq!(t.answer_constraint("Which Rhodes Island operator won the chess tournament?"), None, "an organization is no nation");
        assert_eq!(t.answer_constraint("Which operator from Victoria plays the harp?"), Some(AnswerConstraint::Nation("Victoria".into())));
        let c = AnswerConstraint::NonPlayable;
        assert_eq!(t.failing_answer_name(q, "Sakiko Togawa visited it [3].", &c).map(|x| x.0).as_deref(), Some("Sakiko Togawa"));
        assert_eq!(t.failing_answer_name(q, "Rosa visited it, then Mabel.", &c).map(|x| x.1).as_deref(), Some("Rosa is the playable operator Rosmontis"));
        assert!(t.failing_answer_name(q, "Mabel, the former Bluishsilver, visited Iris.", &c).is_none(), "an NPC passes; Iris is in the question");
        assert!(t.failing_answer_name(q, "The passages do not say.", &c).is_none());
        assert!(t.failing_answer_name(q, "The passages do not identify one. They mention Rosmontis [1].", &c).is_none(), "context, not the answer");
        let v = AnswerConstraint::Nation("Victoria".into());
        assert!(t.failing_answer_name("Which operator from Victoria?", "Rosmontis.", &v).is_none(), "birthplace Victoria");
        assert!(t.failing_answer_name("Which operator from Victoria?", "Amiya.", &v).is_some());
    }

    #[test]
    fn game_data_for_trait_questions_about_named_characters_and_families() {
        let a = |n: &str, race: &str| json!({"name": n, "race": race, "birthplace": "Kazimierz", "nation": "Kazimierz", "group": null, "heightCm": 170});
        let t = Tools { attributes: Some(vec![a("Nearl", "Kuranta"), a("Nearl the Radiant Knight", "Kuranta"), a("Angelina", "Vulpo"), a("Mountain", "Feline")]),
                        topics: Some(vec![json!({"topic": "Kuranta", "aliases": ["Kuranta"], "summary": format!("Horse-like [1].{}", " x".repeat(100)),
                                                 "summaryV2": format!("Equine.{}", " y".repeat(100))})]),
                        game_data: true, topics_v2_text: true, ..Tools::default() };
        let g = t.game_data("Do the Nearls have wings?").unwrap();
        assert_eq!(g.topic, "Nearl, Nearl the Radiant Knight");
        assert!(g.text.contains("race Kuranta") && g.text.contains("Race Kuranta (Trevor's topic summary, start): Equine."));
        assert!(g.text.contains("Races whose topic summaries mention wings: none"));
        assert_eq!(t.game_data("how old is angelina").unwrap().topic, "Angelina");
        assert!(t.game_data("What did Nearl do in Kazimierz?").is_none(), "not a trait question");
        // A trade, kept knowingly: folded matching of 6 letters or more lets a common word through on a trait question.
        assert_eq!(t.game_data("how tall is the mountain").unwrap().topic, "Mountain");
        assert!(Tools { game_data: false, ..t }.game_data("Do the Nearls have wings?").is_none());
    }

    #[test]
    fn new_dossiers_match_by_own_name_long_folded_short_as_written() {
        let t = Tools { new_dossiers: vec![("Rosa".into(), "r".into()), ("Innkeeper Zheng".into(), "z".into()), ("Pole-Carrier".into(), "p".into())],
                        ..Tools::default() };
        assert_eq!(t.new_dossier("Who is Rosa's mentor?").unwrap().topic, "Rosa");
        assert!(t.new_dossier("a rosa flower").is_none(), "a one-word name matches only as written");
        assert!(t.new_dossier("Rosalind says").is_none());
        assert_eq!(t.new_dossier("what does innkeeper zheng sell").unwrap().topic, "Innkeeper Zheng");
        let m = Tools { new_dossiers: vec![("Mountain".into(), "m".into())], ..Tools::default() };
        assert!(m.new_dossier("the tree he planted on the mountain").is_none());
        assert!(m.new_dossier("Is Mountain strong?").is_some());
        let l = Tools { new_dossiers: vec![("Lin".into(), "l".into())], ..Tools::default() };
        assert!(l.new_dossier("where Lin Qingyan was first brought").is_none(), "the start of a longer name");
        assert!(l.new_dossier("Is Lin a mob boss?").is_none(), "under 4 letters");
        assert_eq!(t.new_dossier("who is the pole carrier").unwrap().topic, "Pole-Carrier");
        assert!(Tools::default().new_dossier("Who is Red?").is_none(), "v1 loads none");
    }

    #[test]
    fn topics_from_data_resolve_by_merged_names_and_drop_under_v1() {
        let long = |w: &str| format!("{w}.{}", " x".repeat(100));
        let rows = vec![json!({"topic": "Victoria", "aliases": ["Victoria"], "names": ["Victorian", "Victorians"], "summary": long("Old")}),
                        json!({"topic": "Yan-Lungmen", "aliases": ["Yan-Lungmen", "Lungmen"], "source": "table", "summary": long("City")}),
                        json!({"topic": "Kylin", "aliases": ["Kylin"], "source": "table", "thin": "passages: 2 of 2 name it"})];
        let mut t = Tools { topics: Some(served_topics(rows)), ..Tools::default() };
        assert_eq!(t.topic("victorians").unwrap().topic, "Victoria");
        assert_eq!(t.topic("Lungmen").unwrap().topic, "Yan-Lungmen");
        assert!(matches!(t.topic("Kylin"), Err(ToolError::NotFound { .. })), "a thin topic is not served");
        t.topics_v1();
        assert_eq!(t.topics.as_ref().unwrap().len(), 1);
        assert!(t.topics.as_ref().unwrap()[0].get("names").is_none(), "v1 has no merged names");
        assert!(t.topic("Lungmen").is_err());
    }

    #[test]
    fn short_names_expand_only_unique_prefixes() {
        let mut t = tools();
        t.identities.push(vec!["Bibeak".into()]);
        assert_eq!(t.short_names("What did bibeak design for kal"), vec![("kal".to_owned(), "Kal'tsit".to_owned())]);
        assert!(t.short_names("Is Texas a Lupo?").is_empty(), "a whole name is left alone");
        // Texas and Texas the Omertosa share a first word, so "tex" is one target, the shorter name; "flame" also
        // matches Flamebringer here, which is why the caller checks corpus frequency (flame is a common word).
        assert_eq!(t.short_names("tex or flame"), vec![("tex".to_owned(), "Texas".to_owned()), ("flame".to_owned(), "Flamebringer".to_owned())]);
    }

    #[test]
    fn cross_ref_lists_bosses_by_operator_and_off_topic_questions_name_nothing() {
        let mut t = tools();
        assert!(matches!(t.cross_ref("bosses_playable"), Err(ToolError::Missing(_))));
        t.crossref.insert("bosses_playable".into(), json!({"bosses": 229, "operators": 407, "rows": [
            {"operator": "W", "boss": "W", "matchedBy": "codename"},
            {"operator": "Amiya", "boss": "'Amiya'", "matchedBy": "codename"},
            {"operator": "Amiya", "boss": "'Amiya', Furnace's Finale", "matchedBy": "codename"}]}));
        let out = t.cross_ref("bosses_playable").unwrap();
        assert!(out.contains("229 boss") && out.contains(": 2 operators") && out.contains("- Amiya: \"'Amiya'\" (matched by codename); \"'Amiya', Furnace's Finale\""));
        assert!(matches!(t.cross_ref("everything"), Err(ToolError::BadArg { .. })));
        assert!(!t.names_anything("give me instructions for baking a tray of brownies"));
        assert!(t.names_anything("how tall is texas?") && t.names_anything("what happens in babel") && t.names_anything("who is Louisa"));
        assert!(t.names_anything("is this in arknights?") && t.names_anything("what's texas's hobby?"));
    }

    #[test]
    fn design_deduce_reads_forward_and_reverse_and_shortlists_by_rare_features() {
        let mut t = tools();
        assert!(t.design_deduce_question("what animal is kirara based on").is_none());
        let row = |n: &str, text: &str| json!({"charId": n, "operator": n, "race": "", "alters": [], "text": text});
        t.design_evidence = Some(vec![
            row("Lucilla", "Art: dark ribbon-like tentacles and a translucent bell above her, blue water."),
            row("Phantom", "Art: a black cat, blue light."),
            row("Texas", "Art: wolf ears, blue sword."),
            row("Texas the Omertosa", "Art: a black wolf companion, blue."),
            row("Kirara", "Art: coral, blue screens.")]);
        // ian11 asks for an operator: forward, even though "phantom" is an operator's name.
        let (fwd, _) = t.design_deduce_question("what operator is based off an animal with phantom as part of its name").unwrap();
        assert!(fwd);
        let (fwd, named) = t.design_deduce_question("What animal is Kirara based on?").unwrap();
        assert!(!fwd && named.len() == 1 && named[0]["operator"] == "Kirara");
        // the longest name wins over the name it contains
        let (_, named) = t.design_deduce_question("What is Texas the Omertosa based on?").unwrap();
        assert_eq!(named.iter().map(|r| r["operator"].as_str().unwrap()).collect::<Vec<_>>(), vec!["Texas the Omertosa"]);
        assert!(t.design_deduce_question("who is phantom").is_none());
        // "tentacles" and "bell" are rare, "blue" is everywhere: Lucilla first, and an operator sharing only "blue" scores 0
        let short = t.design_shortlist(&[("giant phantom jelly".to_owned(), 2.0), ("long ribbon-like tentacles".to_owned(), 1.0),
                                         ("bell".to_owned(), 1.0), ("blue".to_owned(), 1.0)], 3);
        assert_eq!(short.first().map(|r| r["operator"].as_str().unwrap()), Some("Lucilla"));
        assert!(short.iter().all(|r| r["operator"] != "Kirara"));
    }

    #[test]
    fn design_basis_answers_both_readings_and_only_design_questions() {
        let mut t = tools();
        assert_eq!(t.design_basis("what operator is based off an animal with phantom as part of its name"), None);
        t.attributes = Some(vec![json!({"name": "Lucilla"}), json!({"name": "Phantom"}), json!({"name": "Tragodia"}), json!({"name": "Kirara"})]);
        let sub = |x: &str| json!({"subject": x, "category": "animal", "why": "clues.", "confidence": "medium",
                                   "evidence": [{"id": "E3", "source": "operator file, Profile", "text": "a line"}]});
        t.design_basis = Some(vec![
            json!({"operator": "Kirara", "alters": [], "subjects": [sub("phantom jellyfish")]}),
            json!({"operator": "Lucilla", "alters": [], "subjects": []}),
            json!({"operator": "Phantom", "alters": ["Tragodia"], "subjects": [sub("The Phantom of the Opera")]}),
            json!({"operator": "Tragodia", "alters": ["Phantom"], "subjects": [sub("Dionysus")]})]);
        let out = t.design_basis("what operator is based off an animal with phantom as part of its name").unwrap();
        assert!(out.starts_with("Trevor's inference from the game data") && !out.contains("wiki"));
        assert!(out.contains("- Kirara: likely draws on phantom jellyfish (animal, medium confidence)"));
        assert!(out.contains("- Phantom: likely draws on The Phantom of the Opera") && out.contains("(the alternate operator of Phantom)"));
        assert_eq!(out.matches("- Phantom:").count(), 1);
        assert_eq!(t.design_basis("who is phantom"), None);
        assert_eq!(t.design_basis("what is the kitchen of rhodes island based on"), None);
        // An operator with nothing inferred says so instead of guessing.
        assert!(t.design_basis("what animal is lucilla based on").unwrap().contains("- Lucilla: the game data does not point"));
        // The kill switch is the wiki answer, read only from the rows the legacy loader gives it.
        t.opts.wiki_legacy = true;
        assert_eq!(t.design_basis("what animal is lucilla based on"), None);
        t.design_basis_wiki = Some(vec![json!({"operator": "Lucilla", "basis": ["giant phantom jelly"], "sentence": "s.", "alters": [], "url": "u1"})]);
        assert!(t.design_basis("what animal is lucilla based on").unwrap().starts_with("From the Arknights wiki's trivia pages"));
    }

    /// The wiki is an eval reference only (2026-10-04): no serving code names its directory except `crate::reference`,
    /// whose loader only `ask --wiki-legacy` calls, and loading the tables never reads it.
    #[test]
    fn serving_code_never_opens_eval_reference() {
        let dir = concat!("eval/", "reference");
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src.clone()];
        let mut offenders = Vec::new();
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().is_some_and(|x| x == "rs") && p != src.join("reference.rs") {
                    let text = std::fs::read_to_string(&p).unwrap();
                    if text.contains(dir) || text.contains(concat!("wiki_", "cache")) || text.contains(concat!("design_basis", ".jsonl"))
                        || text.contains(concat!("wiki_", "deceased")) || text.contains(concat!("wiki_", "status")) {
                        offenders.push(p.display().to_string());
                    }
                }
            }
        }
        assert!(offenders.is_empty(), "serving code names the wiki eval references: {offenders:?}");
        let root = std::env::temp_dir().join(format!("trevor-ref-test-{}", std::process::id()));
        std::fs::create_dir_all(root.join(dir)).unwrap();
        std::fs::write(root.join(dir).join(concat!("design_basis", ".jsonl")), "{\"operator\": \"Lucilla\", \"basis\": [\"jelly\"]}\n").unwrap();
        let t = Tools::load(&root);
        assert!(t.design_basis.is_none() && t.design_basis_wiki.is_none() && !t.opts.wiki_legacy);
        assert!(crate::reference::wiki_design_basis(&root).is_some());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn norm_folds_case_accents_and_apostrophes() {
        assert_eq!(norm("Ch'en, the Holungday!"), "chen the holungday");
        assert_eq!(norm("Zwillingstürme"), "zwillingsturme");
        assert_eq!(episode_number("episode 7"), Some(7));
        assert_eq!(episode_number("ch14"), Some(14));
        assert_eq!(episode_number("babel"), None);
    }
}
