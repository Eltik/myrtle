//! Trevor's table tools: answers produced from precomputed tables, with structured arguments.
//!
//! `ask` used to answer some questions from tables with one function per route that both parsed the
//! question with keyword lists and wrote the answer. The writing half lives here, unchanged in its
//! text; the parsing half is now a router's job (`ask --router model|keywords`, `crate::router`).
//! Every tool resolves its own arguments: an event given by name ("Babel", "Episode 7", "chapter 14",
//! a misspelling within one or two letters) resolves to a group id, a character or operator name
//! through the tables and the identity links. A failed resolution is a [`ToolError`], which the
//! caller treats as "answer by retrieval"; a tool never answers for a name it did not resolve.

mod answer_check;
mod appearances;
mod canon;
mod deaths;
mod operators;
mod reading;
mod resolve;
#[cfg(test)]
mod tests;
mod text;
mod topics;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use self::operators::filter_from_args;
pub use self::reading::RecapPart;
pub use self::text::{contains_words, norm, whole_word_matches};
use self::topics::served_topics;
use crate::util::{load_json_opt, load_jsonl_opt};

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
    /// Trevor has), so rogue_1 is IS #2. Off (`ask --is-numbering-v1`) is the old rogue_N = IS N.
    pub is_offset: bool,
    /// The canon answer without the per-run verdicts: strength labels from reference counts only
    /// (`ask --canon-v1`). Off, a run whose data carries a `verdict` is answered from it.
    pub canon_v1: bool,
    /// The keyword router's recap rule takes a summary request ("Give me a summary of The Masses Travels") whose event
    /// only the event detector finds ([`Tools::event_in_question`]: case and punctuation folded), takes a bare
    /// "summary", and reads the event's name with its punctuation folded as the question's is. Off
    /// (`ask --no-summary-route`) is the rule before.
    pub summary_route: bool,
    /// The keyword router's recap rule also takes "tell me about <event>" and "what is <event>" when nothing but the
    /// event is asked and the event's name is not also a character, operator or served topic ("What is Babel?" asks
    /// about the organization). Off (`ask --no-event-about`) is the rule before.
    pub event_about: bool,
}

impl ToolOptions {
    pub const KEYWORDS: Self = Self {
        storylines: false,
        exact_recap: false,
        is_offset: true,
        canon_v1: false,
        summary_route: true,
        event_about: true,
    };
}

impl Default for ToolOptions {
    fn default() -> Self {
        Self {
            storylines: true,
            exact_recap: true,
            is_offset: true,
            canon_v1: false,
            summary_route: true,
            event_about: true,
        }
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
        Self {
            tool: tool.to_owned(),
            args: args.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect(),
        }
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
        self.args
            .get(k)
            .map(String::as_str)
            .filter(|v| !v.trim().is_empty())
            .ok_or(ToolError::MissingArg(k))
    }

    fn opt(&self, k: &str) -> Option<&str> {
        self.args
            .get(k)
            .map(String::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty() && !v.eq_ignore_ascii_case("none"))
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
    pub const NAMES: &[&str] = &[
        "race",
        "birthplace",
        "birthday",
        "height",
        "infection",
        "class",
        "branch",
        "rarity",
        "gender",
        "affiliation",
    ];

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
            _ => {
                return Err(ToolError::BadArg {
                    arg: "field",
                    value: s.to_owned(),
                });
            }
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

pub const CLASSES: [&str; 8] = [
    "Vanguard",
    "Guard",
    "Defender",
    "Sniper",
    "Caster",
    "Medic",
    "Supporter",
    "Specialist",
];
const PLACE_KEYS: [&str; 4] = ["nation", "group", "team", "birthplace"];

/// The tables, loaded once.
#[derive(Default)]
pub struct Tools {
    /// Chronology v1 groups by id (`artifacts/chrono/timeline_v1.json`). Ordered: lookups by name and the latest
    /// release iterate it, and a HashMap would make their ties depend on the run.
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
    /// The dossiers chosen from data, not in P3b: (name, text as P3b shows a dossier). Empty unless
    /// [`Tools::load_new_dossiers`] ran (`ask --lore v2`).
    pub new_dossiers: Vec<(String, String)>,
    /// `ask --lore v2`: answer a trait question with the named characters' game data (see [`Tools::game_data`]).
    pub game_data: bool,
    /// `ask --lore v2`: topic summaries use `summaryV2` where a trait audit regenerated one.
    pub topics_v2_text: bool,
    /// With `topics_v2_text`: `summaryV2r` where the race-trait rule (`topics.py traitrule`, no model) wrote
    /// one (`ask --race-trait-rule`).
    pub topics_race_rule: bool,
    /// Cast-wide cross-references from game data (`scripts/crossref.py`): table name to its JSON
    /// (`artifacts/crossref/<table>.json`). Read by the `cross_ref` tool, which `ask`'s cross-reference trigger routes to.
    pub crossref: BTreeMap<String, Value>,
    /// Per character, the story groups where they speak and where they are only mentioned (`scripts/appearances.py`):
    /// `{"groups": {id: {name, kind, episode, order}}, "characters": {name: {forms, speaks, mentioned, namedIn}}}`.
    /// Read by the `appearances` tool.
    pub appearances: Option<Value>,
    /// Deep topic entries (`scripts/topic_deep.py`): topic name to the judged entry's served text
    /// (a map-reduce over the story groups, operator files, game records and dossiers naming the topic).
    pub deep_topics: HashMap<String, String>,
    /// Serve the deep entry instead of the summary where one exists (set per question by `ask`'s broad-question gate).
    pub topics_deep: bool,
    /// Identity links with their evidence (`artifacts/entities/identities.v2.json`, `ENTITY_V2=1 scripts/entities.py
    /// build`): (name, other name, relation, quote, where, stated). Read by the answer category check.
    pub identity_links: Vec<(String, String, String, String, String, bool)>,
    pub opts: ToolOptions,
}

/// The category a who/which question asks its answer to belong to (`Tools::answer_constraint`).
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

fn s(v: &Value, k: &str) -> String {
    v[k].as_str().unwrap_or_default().to_owned()
}

impl Tools {
    /// Load every table under `root` (the trevor directory). A missing table is not an error here: the tools
    /// that need it answer [`ToolError::Missing`]. A missing, unreadable or unparsable table is logged, one line each,
    /// and listed by [`crate::util::load_failures`].
    #[must_use]
    pub fn load(root: &Path) -> Self {
        let mut t = Self::default();
        if let Some(v) = load_json_opt::<Value>(&root.join("artifacts/chrono/timeline_v1.json")) {
            for g in v["groups"].as_array().into_iter().flatten() {
                if let Some(id) = g["groupId"].as_str() {
                    t.groups.insert(id.to_owned(), g.clone());
                }
            }
        }
        t.guide = load_json_opt::<Value>(&root.join("artifacts/chrono/reading_guide.json"));
        t.community = load_json_opt::<Value>(&root.join("data/reading_community.json"));
        t.first = load_json_opt::<Value>(&root.join("artifacts/chrono/first_appearance.json"));
        t.bank = load_jsonl_opt::<Value>(&root.join("artifacts/bank/bank.jsonl")).unwrap_or_default();
        t.deaths = load_jsonl_opt::<Value>(&root.join("artifacts/entities/deaths.jsonl"));
        t.real_names = load_jsonl_opt::<Value>(&root.join("artifacts/entities/real_names.jsonl"));
        t.attributes = load_jsonl_opt::<Value>(&root.join("artifacts/entities/operator_attributes.jsonl"));
        if let Some(v) = load_json_opt::<Value>(&root.join("artifacts/entities/identities.json")) {
            for i in v.as_array().into_iter().flatten() {
                let mut names = vec![s(i, "label")];
                names.extend(
                    i["names"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|n| n.as_str().map(str::to_owned)),
                );
                t.identities.push(names);
            }
        }
        t.storylines = load_json_opt::<Value>(&root.join("artifacts/chrono/storylines.json"));
        t.canon = load_json_opt::<Value>(&root.join("artifacts/canon/is_endings.json"));
        // A thin topic (too few passages name it, or a short or poorly supported summary; scripts/topics.py thin) is
        // never served; only topics from data are ever marked.
        t.topics = load_jsonl_opt::<Value>(&root.join("artifacts/topics/topics.jsonl")).map(served_topics);
        if crate::env::on_unless_0("TREVOR_TOPIC_IDENTITIES") {
            t.fold_group_identities();
        }
        if crate::env::on_unless_0("TREVOR_ORG_SHORT_NAMES") {
            t.org_short_names();
        }
        for table in CROSSREF_TABLES {
            if let Some(v) = load_json_opt::<Value>(&root.join(format!("artifacts/crossref/{table}.json"))) {
                t.crossref.insert((*table).to_owned(), v);
            }
        }
        t.appearances = load_json_opt::<Value>(&root.join("artifacts/appearances/appearances.json"));
        for d in load_jsonl_opt::<Value>(&root.join("artifacts/topics/deep.jsonl")).unwrap_or_default() {
            if let (Some(t_), Some(text)) = (d["topic"].as_str(), d["served"].as_str()) {
                t.deep_topics.insert(t_.to_owned(), text.to_owned());
            }
        }
        if let Some(v) = load_json_opt::<Value>(&root.join("artifacts/entities/identities.v2.json")) {
            for i in v["identities"].as_array().into_iter().flatten() {
                for (name, ev) in i["evidence"].as_object().into_iter().flatten() {
                    for e in ev.as_array().into_iter().flatten() {
                        t.identity_links.push((
                            name.clone(),
                            s(e, "other"),
                            s(e, "relation"),
                            s(e, "quote"),
                            s(e, "where"),
                            true,
                        ));
                    }
                }
            }
            for e in v["inferred"].as_array().into_iter().flatten() {
                t.identity_links.push((
                    s(e, "name_a"),
                    s(e, "name_b"),
                    s(e, "relation"),
                    s(e, "quote"),
                    s(e, "where"),
                    false,
                ));
            }
        }
        t
    }

    /// Identity links that name an organization, nation or place, not a person: an identity group whose label is such a
    /// topic's name or alias gives its names to that topic as aliases and leaves the person lists, so a group name kept
    /// as a person name ("Blacksteel" for "Blacksteel Worldwide") still resolves to its topic. `TREVOR_TOPIC_IDENTITIES=0`
    /// keeps the person lists as written.
    pub(crate) fn fold_group_identities(&mut self) {
        let Some(rows) = self.topics.as_mut() else { return };
        let forms = |r: &Value| -> Vec<String> {
            std::iter::once(s(r, "topic"))
                .chain(
                    r["aliases"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|a| a.as_str().map(str::to_owned)),
                )
                .map(|f| norm(&f))
                .collect()
        };
        self.identities.retain(|group| {
            let Some(label) = group.first().map(|l| norm(l)) else {
                return true;
            };
            let Some(r) = rows.iter_mut().find(|r| {
                matches!(s(r, "kind").as_str(), "organization" | "nation" | "place") && forms(r).contains(&label)
            }) else {
                return true;
            };
            let have = forms(r);
            if let Some(al) = r["aliases"].as_array_mut() {
                for n in group.iter().filter(|n| !have.contains(&norm(n))) {
                    al.push(Value::String(n.clone()));
                }
            }
            false
        });
    }

    /// The short name of an organization topic named with a company suffix: "Blacksteel Worldwide" is also
    /// "Blacksteel", the name the story uses, when the head is 4 characters or more and no topic already has it as a
    /// form. "Group" is not a suffix here, so the Ursus Student Self-government Group keeps its name. It replaces, at the
    /// source, the identity link `fold_group_identities` would read, which `entities.py build` drops as naming a group.
    /// `TREVOR_ORG_SHORT_NAMES=0` leaves the topics as written.
    pub(crate) fn org_short_names(&mut self) {
        const SUFFIX: &[&str] = &[
            "worldwide",
            "company",
            "co",
            "ltd",
            "inc",
            "corporation",
            "corp",
            "llc",
            "holdings",
            "enterprises",
            "industries",
        ];
        let Some(rows) = self.topics.as_mut() else { return };
        let forms: BTreeSet<String> = rows
            .iter()
            .flat_map(|r| {
                std::iter::once(s(r, "topic")).chain(
                    r["aliases"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|a| a.as_str().map(str::to_owned)),
                )
            })
            .map(|f| norm(&f))
            .collect();
        for r in rows.iter_mut().filter(|r| s(r, "kind") == "organization") {
            let topic = s(r, "topic");
            let words: Vec<&str> = topic
                .split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '-'))
                .filter(|w| !w.is_empty())
                .collect();
            let mut end = words.len();
            while end > 0 && SUFFIX.contains(&words[end - 1].to_lowercase().as_str()) {
                end -= 1;
            }
            if end == 0 || end == words.len() {
                continue;
            }
            let head = words[..end].join(" ");
            if head.chars().count() >= 4 && !forms.contains(&norm(&head)) {
                if let Some(al) = r["aliases"].as_array_mut() {
                    al.push(Value::String(head));
                }
            }
        }
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
            "recap" => self.recap(
                r.arg("event")?,
                r.opt("ending").is_some_and(|v| v.eq_ignore_ascii_case("true")),
                RecapPart::parse(r.opt("part"))?,
            ),
            "timeline_all" => Ok(self.timeline_all()),
            "deaths_in_event" => {
                let who = match r.opt("who").map(str::to_lowercase).as_deref() {
                    None | Some("all") => Who::All,
                    Some("npc" | "npcs" | "non-playable") => Who::Npc,
                    Some("operators" | "operator" | "playable") => Who::Operators,
                    Some(v) => {
                        return Err(ToolError::BadArg {
                            arg: "who",
                            value: v.to_owned(),
                        });
                    }
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
            // The model router can write {"run": "all"} for "any of the IS endings": treated as no run.
            "canon" => self.canon(
                r.opt("run")
                    .filter(|v| !matches!(norm(v).as_str(), "all" | "any" | "every" | "all runs")),
            ),
            // Not an answer: `ask` adds the summary to retrieval (see `topic`); here it only reports the resolution.
            "topic" => self
                .topic(r.arg("topic")?)
                .map(|p| format!("Topic passage for retrieval: {}", p.topic)),
            t => Err(ToolError::UnknownTool(t.to_owned())),
        }
    }

    // ---------------------------------------------------------------- argument resolution
}
