//! Topic summaries (races, nations, places, organizations, concepts), deep topic entries, new dossiers and the
//! game-data passage of trait questions.

use std::path::Path;

use serde_json::Value;

use super::text::{contains_words, fuzzy_unique, norm, strip_citations};
use super::{ToolError, Tools, TopicPassage, s};
use crate::util::load_jsonl_opt;

/// Words of a question about a body, a race feature, an ability or an age (`Tools::game_data`).
pub(super) const TRAIT_WORDS: &[&str] = &[
    "wing",
    "wings",
    "horn",
    "horns",
    "halo",
    "halos",
    "ear",
    "ears",
    "tail",
    "tails",
    "fur",
    "furry",
    "scales",
    "scale",
    "claws",
    "fangs",
    "antlers",
    "feathers",
    "hooves",
    "mane",
    "eyes",
    "pupils",
    "age",
    "aged",
    "old",
    "older",
    "oldest",
    "younger",
    "youngest",
    "years",
    "height",
    "tall",
    "taller",
    "race",
    "species",
    "fly",
    "flying",
    "swim",
    "lifespan",
    "immortal",
    "ageless",
    "ability",
    "abilities",
    "power",
    "powers",
    "body",
    "blood",
    "physiology",
    "anatomy",
    "biology",
];

/// Body words of a trait question for which the game-data passage lists the races described with them.
pub(super) const BODY_WORDS: &[&str] = &[
    "wing", "wings", "horn", "horns", "halo", "halos", "ear", "ears", "tail", "tails", "fur", "scales", "claws",
    "fangs", "antlers", "feathers", "hooves", "mane",
];

/// The topic rows the tool serves: every row not marked thin.
pub(super) fn served_topics(rows: Vec<Value>) -> Vec<Value> {
    rows.into_iter().filter(|r| r.get("thin").is_none()).collect()
}

impl Tools {
    /// The group topics a question names ("Evaluate the political influence of Karlan Trade Group and Blacksteel"):
    /// served nation, place, race and organization topics whose name or alias of 4 or more characters
    /// the question holds as whole words, case folded, a name inside a longer matched one not counted; each with its
    /// forms, in the order the question names them.
    #[must_use]
    pub fn named_group_topics(&self, q: &str) -> Vec<(String, Vec<String>)> {
        let n = norm(q);
        let Some(rows) = self.topics.as_ref() else {
            return Vec::new();
        };
        let mut hits: Vec<(usize, String, &Value)> = Vec::new();
        for r in rows
            .iter()
            .filter(|r| matches!(s(r, "kind").as_str(), "nation" | "place" | "race" | "organization"))
        {
            for f in std::iter::once(s(r, "topic")).chain(
                r["aliases"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|a| a.as_str().map(str::to_owned)),
            ) {
                let x = norm(&f);
                if x.chars().count() >= 4 && contains_words(&n, &x) {
                    hits.push((n.find(&x).unwrap_or(usize::MAX), x, r));
                }
            }
        }
        let outer: Vec<&(usize, String, &Value)> = hits
            .iter()
            .filter(|(_, x, r)| {
                !hits
                    .iter()
                    .any(|(_, y, u)| s(u, "topic") != s(r, "topic") && y.len() > x.len() && contains_words(y, x))
            })
            .collect();
        let mut out: Vec<(usize, String, Vec<String>)> = Vec::new();
        for (at, _, r) in outer {
            let topic = s(r, "topic");
            if let Some(o) = out.iter_mut().find(|o| o.1 == topic) {
                o.0 = o.0.min(*at);
                continue;
            }
            let forms = std::iter::once(topic.clone())
                .chain(
                    r["aliases"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|a| a.as_str().map(str::to_owned)),
                )
                .filter(|f| f.chars().count() >= 4)
                .collect();
            out.push((*at, topic, forms));
        }
        out.sort_by_key(|o| o.0);
        out.into_iter().map(|(_, t, f)| (t, f)).collect()
    }

    /// Whether a question names a served nation, place, race or organization topic (a form of 4 letters or more as whole
    /// words): "Are Petrams related to Aegirs?" asks about peoples, not people (the relation rule's gate).
    #[must_use]
    pub fn names_group_topic(&self, q: &str) -> bool {
        let n = norm(q);
        self.topics
            .iter()
            .flatten()
            .filter(|r| matches!(s(r, "kind").as_str(), "nation" | "place" | "race" | "organization"))
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
            .any(|f| f.chars().count() >= 4 && (contains_words(&n, &f) || contains_words(&n, &format!("{f}s"))))
    }

    /// Whether a question asks about a topic as a whole (the deep-entry gate): it names a served topic and, besides that
    /// topic's forms and function words, has at most one content word of 3 letters or more. "Who are the sarkaz?" (0)
    /// and "What is sami guarding?" (1, guarding) are; "Do Liberi, as a culture, give their feathers as gifts?" and "Do
    /// black Sankta halos adn energy wings mean something?", which ask one detail, are not.
    #[must_use]
    pub fn broad_topic_question(&self, q: &str) -> bool {
        const STOP: &[&str] = &[
            "what",
            "who",
            "whom",
            "which",
            "where",
            "when",
            "why",
            "how",
            "are",
            "is",
            "was",
            "were",
            "the",
            "do",
            "does",
            "did",
            "about",
            "tell",
            "know",
            "explain",
            "mean",
            "means",
            "they",
            "them",
            "their",
            "there",
            "this",
            "that",
            "these",
            "those",
            "and",
            "for",
            "with",
            "really",
            "exactly",
            "actually",
            "lore",
            "arknights",
            "can",
            "you",
            "someone",
            "anyone",
            "please",
            "all",
            "any",
            "much",
            "more",
            "exist",
        ];
        let Some(rows) = self.topics.as_ref() else { return false };
        let n = norm(q);
        let mut rest = format!(" {n} ");
        let mut named = false;
        let mut forms: Vec<String> = rows
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
            .filter(|f| f.chars().count() >= 4)
            .collect();
        forms.sort_by_key(|f| std::cmp::Reverse(f.len()));
        for f in &forms {
            let pad = format!(" {f} ");
            if rest.contains(&pad) {
                named = true;
                rest = rest.replace(&pad, " ");
            }
        }
        named
            && rest
                .split_whitespace()
                .filter(|w| w.chars().count() >= 3 && !STOP.contains(w))
                .count()
                <= 1
    }

    /// The deep entry of a topic (`ask --deep-topics`), its citations stripped, when one of at least 100 words exists.
    #[must_use]
    pub fn deep_entry(&self, topic: &str) -> Option<String> {
        let t = self.deep_topics.get(topic).filter(|_| self.topics_deep)?;
        Some(strip_citations(t)).filter(|x| x.split_whitespace().count() >= 100)
    }

    /// Which summary field a topic row serves: `summary` (v1), `summaryV2` (`--lore v2`, where the trait audit
    /// regenerated one), `summaryV2r` (with the race-trait rule, where `topics.py traitrule` wrote one).
    pub(super) fn topic_text_key(&self, r: &Value) -> &'static str {
        if self.topics_v2_text && self.topics_race_rule && r.get("summaryV2r").is_some() {
            "summaryV2r"
        } else if self.topics_v2_text && r.get("summaryV2").is_some() {
            "summaryV2"
        } else {
            "summary"
        }
    }

    /// The 46 topics of lore v1 (`ask --lore v1`): the rows without a `source`, without the merged `names`
    /// the topics from data added to them.
    pub fn topics_v1(&mut self) {
        if let Some(rows) = self.topics.as_mut() {
            rows.retain(|r| r.get("source").is_none());
            for r in rows.iter_mut() {
                if let Some(o) = r.as_object_mut() {
                    // Only the fields a v1 row had: the trait audit added summaryV2, passagesV2 and their keys to old rows.
                    o.retain(|k, _| {
                        matches!(
                            k.as_str(),
                            "topic" | "kind" | "aliases" | "asked" | "passages" | "summary" | "inputSha" | "promptSha"
                        )
                    });
                }
            }
        }
    }

    /// Load the dossiers of `artifacts/dossiers/dossiers.jsonl` that are not among the v1 ones
    /// (`dossiers.v1.jsonl`, which P3b holds), with their text as `build-profiles` writes a profile.
    pub fn load_new_dossiers(&mut self, root: &Path) {
        let v1: std::collections::HashSet<String> =
            load_jsonl_opt::<Value>(&root.join("artifacts/dossiers/dossiers.v1.jsonl"))
                .unwrap_or_default()
                .iter()
                .map(|r| s(r, "name"))
                .collect();
        for d in load_jsonl_opt::<Value>(&root.join("artifacts/dossiers/dossiers.jsonl")).unwrap_or_default() {
            let name = s(&d, "name");
            if v1.contains(&name) {
                continue;
            }
            let aka: Vec<&str> = d["names"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|x| *x != name)
                .collect();
            let head = if aka.is_empty() {
                format!("Character profile: {name}")
            } else {
                format!("Character profile: {name} (also known as {})", aka.join(", "))
            };
            let text = format!("{head}\n{}", s(&d, "dossier"));
            self.new_dossiers.push((name, text));
        }
    }

    /// A game-data passage for a question about a named character's (or family's) body, race features, abilities or
    /// age: each named operator's race, birthplace, faction and height from the operator files, then the topic summary
    /// of each race ("Do the Nearls have wings?" is answered from the files listing every Nearl as Kuranta and the
    /// Kuranta summary, which no story passage states). Names match as written (4 letters or more), folded from 6
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
        let written = |name: &str| {
            q.match_indices(name).any(|(i, _)| {
                !q[..i].chars().next_back().is_some_and(char::is_alphanumeric)
                    && !q[i + name.len()..].chars().next().is_some_and(char::is_alphanumeric)
            })
        };
        let mut hits: Vec<&Value> = attrs
            .iter()
            .filter(|r| {
                let name = s(r, "name");
                let n = norm(&name);
                let first = n.split(' ').next().unwrap_or_default().to_owned();
                (name.chars().count() >= 4 && written(&name))
                    || (n.chars().count() >= 6 && contains_words(&nq, &n))
                    || (first.chars().count() >= 4 && words.contains(&format!("{first}s").as_str()))
            })
            .collect();
        // A name inside a longer matched name ("Nearl" in "Nearl the Radiant Knight") is kept: both are the family.
        hits.truncate(6);
        if hits.is_empty() {
            return None;
        }
        let field = |r: &Value, k: &str| {
            r[k].as_str()
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
                .or_else(|| r[k].as_u64().map(|v| v.to_string()))
        };
        let mut text = String::from("Game data (the operator files' fields):");
        let mut races: Vec<String> = Vec::new();
        for r in &hits {
            let mut parts = Vec::new();
            for (k, label) in [
                ("race", "race"),
                ("birthplace", "birthplace"),
                ("nation", "nation"),
                ("group", "faction"),
                ("heightCm", "height cm"),
            ] {
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
                text.push_str(&format!(
                    "\nRace {race} (Trevor's topic summary, start): {}",
                    short.join(" ")
                ));
            }
        }
        // For each body word the question asks about, the races whose summaries describe it, with that sentence: "Do the
        // Nearls have wings?" needs to know that wings belong to the Sankta, not only what the Kuranta summary says.
        for w in words.iter().filter(|w| BODY_WORDS.contains(w)) {
            let stem = w.trim_end_matches('s');
            let mut with: Vec<String> = Vec::new();
            for r in self.topics.iter().flatten().filter(|r| s(r, "kind") == "race") {
                let body = s(r, self.topic_text_key(r));
                let sent = strip_citations(&body)
                    .split_inclusive(['.', '!', '?'])
                    .find(|x| norm(x).split(' ').any(|t| t.trim_end_matches('s') == stem))
                    .map(|x| x.trim().to_owned());
                if let Some(x) = sent {
                    with.push(format!("{} (\"{x}\")", s(r, "topic")));
                }
            }
            text.push_str(&format!(
                "\nRaces whose topic summaries mention {w}: {}",
                if with.is_empty() {
                    "none".to_owned()
                } else {
                    with.join("; ")
                }
            ));
        }
        let names: Vec<String> = hits.iter().map(|r| s(r, "name")).collect();
        Some(TopicPassage {
            topic: names.join(", "),
            text,
        })
    }

    /// The new dossier of the character the question names, if one does (the longest name wins). A dossier is matched
    /// by its own name only, never by identity-link names (those can join different characters, Estelle to Perfumer),
    /// and as written (case and all, whole words: "Red", "Grace" and "Mountain" are words, and folded matching would
    /// send "on the mountain" to Mountain's dossier); a name of two or more words also matches folded ("innkeeper
    /// zheng"). A trade: a lower-case "silverash" gets no dossier.
    #[must_use]
    pub fn new_dossier(&self, q: &str) -> Option<TopicPassage> {
        let nq = norm(q);
        let as_written = |name: &str| {
            q.match_indices(name).any(|(i, _)| {
                let before = q[..i].chars().next_back();
                let rest = &q[i + name.len()..];
                let after = rest.chars().next();
                // Not the start of a longer name: "Lin Qingyan" is not Lin.
                let longer = rest
                    .strip_prefix(' ')
                    .and_then(|r| r.chars().next())
                    .is_some_and(char::is_uppercase);
                !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric) && !longer
            })
        };
        self.new_dossiers
            .iter()
            .filter(|(name, _)| {
                let n = norm(name);
                // One-word names under 4 letters never match (Yu, Lin, Ace, May, Ji: "Deputy Minister Yu" is not Yu).
                (n.contains(' ') || n.chars().count() >= 4)
                    && (as_written(name) || (n.contains(' ') && contains_words(&nq, &n)))
            })
            .max_by_key(|(name, _)| name.len())
            .map(|(name, text)| TopicPassage {
                topic: name.clone(),
                text: text.clone(),
            })
    }

    /// The summary of a race, nation or concept, its bracket citations stripped (they number passages the answer
    /// model does not see).
    ///
    /// # Errors
    /// Missing table or a topic that does not resolve.
    pub fn topic(&self, name: &str) -> Result<TopicPassage, ToolError> {
        let rows = self.topics.as_ref().ok_or(ToolError::Missing("topics.jsonl"))?;
        let nf = || ToolError::NotFound {
            what: "topic",
            name: name.to_owned(),
        };
        let n = norm(name);
        // Topic names first, then aliases: "Beast Lord" is an alias of Feranmut and the singular of Beast Lords.
        let by_name = rows.iter().find(|r| {
            let t = norm(&s(r, "topic"));
            t == n || t == format!("{n}s") || format!("{t}s") == n
        });
        // `names`: forms the topics from data merged into a topic (demonyms, plurals, short forms), after every alias.
        let nn = n.as_str();
        let alias_in = |k: &'static str| {
            move |r: &&Value| {
                r[k].as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|a| {
                        let a = norm(a);
                        a == nn || format!("{a}s") == nn
                    })
            }
        };
        let by_alias = || {
            rows.iter()
                .find(alias_in("aliases"))
                .or_else(|| rows.iter().find(alias_in("names")))
        };
        let names: Vec<(&Value, String)> = rows.iter().map(|r| (r, norm(&s(r, "topic")))).collect();
        let r = by_name
            .or_else(by_alias)
            .or_else(|| fuzzy_unique(&n, names.iter().map(|(r, t)| (*r, t.as_str()))))
            .ok_or_else(nf)?;
        let deep = self.deep_entry(&s(r, "topic"));
        let text = deep.unwrap_or_else(|| strip_citations(&s(r, self.topic_text_key(r))));
        // A summary under 100 words is treated as no summary: the generator asking for passages instead of
        // summarizing (Pythia's) is that short, real summaries are far longer.
        if text.split_whitespace().count() < 100 {
            return Err(ToolError::NoData(format!("no summary for {}", s(r, "topic"))));
        }
        Ok(TopicPassage {
            topic: s(r, "topic"),
            text,
        })
    }

    // ---------------------------------------------------------------- recaps
}
