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
