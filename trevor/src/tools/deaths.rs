//! Death tables: who dies in an event, which operators are dead, and one character's death.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde_json::Value;

use super::{ToolError, ToolResult, Tools, Who, s};

impl Tools {
    /// Named characters with a death event.
    #[must_use]
    pub fn death_names(&self) -> Vec<String> {
        self.deaths.iter().flatten().filter(|e| e["generic"].as_bool() != Some(true)).map(|e| s(e, "character"))
            .filter(|n| !n.is_empty()).collect::<BTreeSet<_>>().into_iter().collect()
    }

    pub(super) fn death_events(&self) -> Result<&Vec<Value>, ToolError> {
        self.deaths.as_ref().ok_or(ToolError::Missing("deaths.jsonl"))
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
}

pub(super) fn death_line(e: &Value) -> String {
    format!("- {}: \"{}\" ({})", s(e, "character"), s(e, "quote"), s(e, "storyId"))
}
