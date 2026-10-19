//! Where characters appear (the appearance index), the event a question names and its characters, source
//! owners and the cast-wide cross-reference tables.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::text::{contains_words, norm};
use super::{CROSSREF_TABLES, ToolError, ToolResult, Tools, s};

impl Tools {
    /// A cast-wide cross-reference table (`scripts/crossref.py`). "bosses_playable": the boss enemies of the enemy
    /// handbook whose character is a playable operator, each with the table that made the match.
    ///
    /// # Errors
    /// An unknown table, or a table that has not been built.
    pub fn cross_ref(&self, table: &str) -> ToolResult {
        let name = CROSSREF_TABLES
            .iter()
            .find(|t| norm(t) == norm(table))
            .ok_or_else(|| ToolError::BadArg {
                arg: "table",
                value: table.to_owned(),
            })?;
        let v = self.crossref.get(*name).ok_or(ToolError::Missing(
            "artifacts/crossref/bosses_playable.json (scripts/crossref.py)",
        ))?;
        let rows = v["rows"].as_array().map(Vec::as_slice).unwrap_or_default();
        let mut by_op: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
        for r in rows {
            by_op.entry(s(r, "operator")).or_default().push(r);
        }
        let mut out = format!(
            "Boss enemies you fight that are also playable operators, from the game data \
            (the enemy handbook's {} boss entries matched by name to the {} obtainable operators, \
            through the codename, the operator's real name or Trevor's identity links): {} operators. \
            A match by name means the same character; a simulated or training version of an ally counts too, \
            and a boss who joins later under another name is found only when a real name or identity link \
            connects the two, so the list may miss some.\n",
            v["bosses"].as_u64().unwrap_or(0),
            v["operators"].as_u64().unwrap_or(0),
            by_op.len()
        );
        for (op, rs) in &by_op {
            let bosses = rs
                .iter()
                .map(|r| {
                    format!(
                        "\"{}\" (matched by {})",
                        s(r, "boss"),
                        s(r, "matchedBy").split(" (").next().unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            out.push_str(&format!("- {op}: {bosses}\n"));
        }
        Ok(out)
    }

    /// Where a character appears: the story groups where they speak and, apart, those
    /// where they are only mentioned, in EN release order, from `artifacts/appearances/appearances.json`.
    ///
    /// # Errors
    /// Missing table, or a character with no speaking or mentioning chunk.
    pub fn appearances(&self, character: &str) -> ToolResult {
        let v = self.appearances.as_ref().ok_or(ToolError::Missing(
            "artifacts/appearances/appearances.json (scripts/appearances.py)",
        ))?;
        let chars = v["characters"]
            .as_object()
            .ok_or(ToolError::Missing("artifacts/appearances/appearances.json"))?;
        let keys: Vec<&str> = chars.keys().map(String::as_str).collect();
        let name = self.resolve_name("character", character, &keys)?;
        let c = &chars[name];
        let groups = &v["groups"];
        let list = |k: &str| -> Vec<(u64, String, u64)> {
            let mut rows: Vec<(u64, String, u64)> = c[k]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(g, n)| {
                    let gr = &groups[g.as_str()];
                    (
                        gr["order"].as_u64().unwrap_or(10_000),
                        appearance_label(gr, g),
                        n.as_u64().unwrap_or(0),
                    )
                })
                .collect();
            rows.sort();
            rows
        };
        let (speaks, mentioned) = (list("speaks"), list("mentioned"));
        if speaks.is_empty() && mentioned.is_empty() {
            return Err(ToolError::NotFound {
                what: "appearance",
                name: character.to_owned(),
            });
        }
        let scenes = |n: u64| {
            if n == 1 {
                "1 scene".to_owned()
            } else {
                format!("{n} scenes")
            }
        };
        let mut out = format!(
            "Where {name} appears in the story text Trevor has, \
            from the scripts' speaker labels and name matches \
            (game text, not a summary; a scene is a passage of about 400 tokens):\n\n\
            Speaks in {} stories, in EN release order:\n",
            speaks.len()
        );
        for (_, l, n) in &speaks {
            out.push_str(&format!("- {l} ({})\n", scenes(*n)));
        }
        if !mentioned.is_empty() {
            out.push_str(&format!(
                "\nMentioned by name without speaking in {}:\n",
                mentioned.len()
            ));
            for (_, l, n) in &mentioned {
                out.push_str(&format!("- {l} ({})\n", scenes(*n)));
            }
        }
        let owners: BTreeSet<String> = c["namedIn"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(|id| self.source_owner(id))
            .filter(|o| norm(o) != norm(name))
            .collect();
        if !owners.is_empty() {
            out.push_str(&format!(
                "\nAlso named in the operator files, voice lines, modules or outfits of: {}.\n",
                owners.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        out.push_str(
            "\nNot counted: stories without speaker labels \
            (Integrated Strategies, paradox simulations, which have no script in the game data), \
            and appearances under a name the identity links do not connect.",
        );
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
        attrs
            .iter()
            .filter_map(|a| a["charId"].as_str().map(|c| (c, a)))
            .filter(|(c, _)| {
                id.contains(*c)
                    || id
                        .split('_')
                        .next_back()
                        .is_some_and(|x| c.ends_with(&format!("_{x}")) && id.starts_with("module_"))
            })
            .max_by_key(|(c, _)| c.len())
            .map(|(_, a)| s(a, "name"))
    }

    /// The character of a question that asks where someone appears ("What chapters does Elysium appear in?", "which
    /// events is Mudrock in"): an appearance word and a story word, not "first" (the `first_appearance` tool's
    /// question), and one name of the appearance index (or an identity link to one) as whole words, the longest kept.
    #[must_use]
    pub fn appearance_question(&self, q: &str) -> Option<String> {
        let n = norm(q);
        let appear = [
            "appear",
            "appears",
            "appeared",
            "appearance",
            "appearances",
            "show up",
            "shows up",
            "showed up",
            "featured",
            "feature",
            "features",
            "present in",
        ];
        let story = [
            "chapter",
            "chapters",
            "event",
            "events",
            "story",
            "stories",
            "episode",
            "episodes",
            "storyline",
            "storylines",
        ];
        if !appear.iter().any(|w| contains_words(&n, w))
            || !story.iter().any(|w| contains_words(&n, w))
            || contains_words(&n, "first")
        {
            return None;
        }
        let chars = self.appearances.as_ref()?["characters"].as_object()?;
        let skip: BTreeSet<&str> = appear.iter().chain(story.iter()).copied().collect();
        let mut best: Option<&str> = None;
        let names = chars
            .keys()
            .map(String::as_str)
            .chain(self.identities.iter().flatten().map(String::as_str));
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

    /// "Was Talulah mentioned in People?": a yes/no question with a mention word, one character of the appearance index
    /// (or an identity form of one) and one story group named as written (case kept, so "people" the word is not the
    /// event People), answered from the index: speaks in N scenes, named in N scenes, or not named (2026-10-19). Before,
    /// retrieval answered it from Roaring Flare, another event.
    #[must_use]
    pub fn event_mention(&self, q: &str) -> Option<String> {
        let n = norm(q);
        let first = n.split(' ').next().unwrap_or_default();
        if !matches!(first, "was" | "is" | "does" | "did" | "has" | "have" | "are" | "were") {
            return None;
        }
        let mention = [
            "mentioned",
            "mention",
            "mentions",
            "named",
            "appear",
            "appears",
            "appeared",
            "show up",
            "shows up",
            "showed up",
            "featured",
            "present",
        ];
        if !mention.iter().any(|w| contains_words(&n, w)) {
            return None;
        }
        let v = self.appearances.as_ref()?;
        let groups = v["groups"].as_object()?;
        // "Episode 17" or "chapter 17" by number, else a group name as written.
        let words: Vec<&str> = n.split(' ').collect();
        let numbered = words.windows(2).find_map(|w| {
            matches!(w[0], "episode" | "chapter" | "ep")
                .then(|| w[1].parse::<u64>().ok())
                .flatten()
        });
        let (gid, group) = numbered
            .and_then(|k| groups.iter().find(|(_, g)| g["episode"].as_u64() == Some(k)))
            .or_else(|| {
                groups
                    .iter()
                    .filter(|(_, g)| {
                        g["name"]
                            .as_str()
                            .is_some_and(|x| x.chars().count() >= 4 && regex_free_word(q, x))
                    })
                    .max_by_key(|(_, g)| g["name"].as_str().map_or(0, str::len))
            })?;
        let gname = norm(group["name"].as_str().unwrap_or_default());
        let chars = v["characters"].as_object()?;
        let mut best: Option<(&str, usize)> = None;
        for (key, _) in chars {
            let forms = std::iter::once(key.as_str()).chain(
                self.identities
                    .iter()
                    .filter(|i| i.iter().any(|x| norm(x) == norm(key)))
                    .flatten()
                    .map(String::as_str),
            );
            for f in forms {
                let x = norm(f);
                if x.chars().count() < 3 || contains_words(&gname, &x) || !contains_words(&n, &x) {
                    continue;
                }
                if best.is_none_or(|(_, l)| l < x.len()) {
                    best = Some((key.as_str(), x.len()));
                }
            }
        }
        let (who, _) = best?;
        let label = appearance_label(group, gid);
        let scenes = |k: u64| {
            if k == 1 {
                "1 scene".to_owned()
            } else {
                format!("{k} scenes")
            }
        };
        let c = &chars[who];
        Some(if let Some(k) = c["speaks"][gid.as_str()].as_u64() {
            format!(
                "Yes. In the story text Trevor has, {who} speaks in {} of {label}.",
                scenes(k)
            )
        } else if let Some(k) = c["mentioned"][gid.as_str()].as_u64() {
            format!(
                "Yes, by name: {who} is named in {} of {label}, without speaking there.",
                scenes(k)
            )
        } else {
            format!(
                "Not by name: {who} neither speaks nor is named in {label} in the story text Trevor has (the \
                     scripts' speaker labels and name matches; a reference by a title or a description is not \
                     counted). Ask where {who} appears for the stories that do name them."
            )
        })
    }

    /// The main episode or event a question names, for the scoped scenes: "Episode 7" or "chapter 7"
    /// by number, else the longest event name the question holds as whole words, compared case and punctuation folded and
    /// without a leading "the", 6 characters or more ("in masses travels" is The Masses' Travels). Operator records
    /// (story_*) are not events a player names.
    #[must_use]
    pub fn event_in_question(&self, q: &str) -> Option<&Value> {
        let n = norm(q);
        let t: Vec<&str> = n.split(' ').collect();
        // An event named as a time boundary ("What happened to Mandragora after Episode 10?") is not the scope: the
        // question asks about what lies outside it.
        let boundary =
            |w: Option<&&str>| w.is_some_and(|w| matches!(*w, "after" | "before" | "since" | "until" | "following"));
        for (i, w) in t.iter().enumerate() {
            if matches!(*w, "episode" | "chapter") && !boundary(i.checked_sub(1).and_then(|j| t.get(j))) {
                if let Some(x) = t.get(i + 1).and_then(|x| x.parse::<u64>().ok()) {
                    if let Some(g) = self.groups.values().find(|g| g["chapter"].as_u64() == Some(x)) {
                        return Some(g);
                    }
                }
            }
        }
        self.groups
            .values()
            .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
            .filter_map(|g| {
                let gn = norm(g["name"].as_str()?);
                let s = gn.strip_prefix("the ").unwrap_or(&gn).to_owned();
                let at = format!(" {n} ").find(&format!(" {s} "))?;
                let before: Vec<&str> = n[..at.min(n.len())]
                    .split(' ')
                    .filter(|w| !w.is_empty() && *w != "the")
                    .collect();
                (s.len() >= 6 && !boundary(before.last())).then_some((g, s.len()))
            })
            .max_by_key(|(_, l)| *l)
            .map(|(g, _)| g)
    }

    /// Every main episode and event a question names, in the order it names them, for the time-gap evidence ("the time
    /// skip from chapter 8 to chapter 9", "between Episodes 8 and 9", "between Chernobog... and Roaring Flare"). Numbers
    /// follow "episode", "chapter" or "ch" (singular or plural), and a number joined to one by "and", "to", "through",
    /// "or" or a comma counts too; event names are matched as in `event_in_question`, without its time-boundary rule
    /// (a gap question names its events as boundaries). Each group once.
    #[must_use]
    pub fn events_in_question(&self, q: &str) -> Vec<&Value> {
        let n = norm(q);
        let t: Vec<&str> = n.split(' ').collect();
        let mut out: Vec<(usize, &Value)> = Vec::new();
        let mut i = 0;
        while i < t.len() {
            if matches!(t[i], "episode" | "episodes" | "chapter" | "chapters" | "ch") {
                let mut j = i + 1;
                while let Some(x) = t.get(j).and_then(|x| x.parse::<u64>().ok()) {
                    if let Some(g) = self.groups.values().find(|g| g["chapter"].as_u64() == Some(x)) {
                        if !out.iter().any(|(_, o)| o["groupId"] == g["groupId"]) {
                            out.push((j, g));
                        }
                    }
                    // "8 and 9", "8 to 9", "8, 9": the joined number is a chapter too.
                    if t.get(j + 1)
                        .is_some_and(|w| matches!(*w, "and" | "to" | "through" | "or"))
                        && t.get(j + 2).is_some_and(|w| w.parse::<u64>().is_ok())
                    {
                        j += 2;
                    } else if t.get(j + 1).is_some_and(|w| w.parse::<u64>().is_ok()) {
                        j += 1;
                    } else {
                        break;
                    }
                }
            }
            i += 1;
        }
        let padded = format!(" {n} ");
        for g in self
            .groups
            .values()
            .filter(|g| !g["groupId"].as_str().unwrap_or_default().starts_with("story_"))
        {
            let Some(name) = g["name"].as_str() else { continue };
            let gn = norm(name);
            let s = gn.strip_prefix("the ").unwrap_or(&gn);
            if s.len() < 6 || out.iter().any(|(_, o)| o["groupId"] == g["groupId"]) {
                continue;
            }
            if let Some(at) = padded.find(&format!(" {s} ")) {
                // A shorter name inside a longer one already found ("Near Light" in "Near Light: Part II") is not a
                // second event.
                let inside = out.iter().any(|(_, o)| {
                    o["name"].as_str().is_some_and(|on| {
                        let x = norm(on);
                        x.len() > s.len() && x.contains(s) && padded.contains(&x)
                    })
                });
                if !inside {
                    out.push((padded[..at].split(' ').filter(|w| !w.is_empty()).count(), g));
                }
            }
        }
        out.sort_by_key(|(at, _)| *at);
        out.into_iter().map(|(_, g)| g).collect()
    }

    /// The characters a question names who speak or are named in story group `gid` by the appearance index, as (index
    /// name, every form: the index name and its identity-link names), for the scoped scenes. A form counts when the
    /// question holds it as whole words (case and punctuation folded, 3 characters or more) and it is not a word of the
    /// event's own name; whether it is a proper name is the caller's corpus check.
    #[must_use]
    pub fn event_characters(&self, q: &str, gid: &str, event_name: &str) -> Vec<(String, Vec<String>)> {
        let n = norm(q);
        let en = norm(event_name);
        let Some(chars) = self.appearances.as_ref().and_then(|v| v["characters"].as_object()) else {
            return Vec::new();
        };
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

    /// The cross-reference table a question asks for without the router: `bosses_playable` for a question with "boss" or
    /// "bosses" and a word for joining or playing ("playable", "join", "operator", "recruit", "obtain", "ally", "side").
    /// This test reads the question only, so the table needs no router line, which would move unrelated routes.
    #[must_use]
    pub fn cross_ref_question(&self, q: &str) -> Option<&'static str> {
        let n = norm(q);
        let boss = contains_words(&n, "boss") || contains_words(&n, "bosses");
        let join = [
            "playable",
            "join",
            "joins",
            "joined",
            "joining",
            "operator",
            "operators",
            "recruit",
            "recruitable",
            "obtain",
            "obtainable",
            "ally",
            "allies",
            "our side",
            "your side",
            "rhodes island side",
        ]
        .iter()
        .any(|w| contains_words(&n, w));
        (boss && join && self.crossref.contains_key("bosses_playable")).then_some("bosses_playable")
    }
}

/// An appearance-index group as a reader names it: "Episode 7: The Birth of Tragedy", "operator record of Elysium
/// 'Long Journey'", or the event's name.
pub(super) fn appearance_label(g: &Value, id: &str) -> String {
    let name = g["name"].as_str().unwrap_or(id);
    let kind = g["kind"].as_str().unwrap_or_default();
    match (kind, g["episode"].as_u64()) {
        ("main", Some(e)) => format!("Episode {e}: {name}"),
        (k, _) if k.starts_with("operator record") => format!("{k}, '{name}'"),
        _ => name.to_owned(),
    }
}

/// Whether `name` appears in `q` as written (case kept), as whole words.
fn regex_free_word(q: &str, name: &str) -> bool {
    q.match_indices(name).any(|(i, m)| {
        let before = q[..i].chars().next_back();
        let after = q[i + m.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}
