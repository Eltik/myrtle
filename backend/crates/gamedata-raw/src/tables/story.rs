//! `story_review_table`, `story_review_meta_table`, `storyline` tables.
//!
//! Moves in: `backend/src/core/gamedata/types/{story_review.rs (99),
//! story_review_meta.rs (343)}`. Script text is not a table: `engine-story`
//! parses scripts handed to it as strings.

#[derive(Debug, Default)]
pub struct StoryTables {}

// ---- Moved down by lift-2 (Oct 25 2026), the music decision of the pilot (F12):
// `StoryVariables`, whole, from crates/engine-story/src/variables.rs (the pilot's
// lift of backend/src/core/story/variables.rs). `story_variables.json` is a table,
// and both the story and the music engines read it, so it sits below both;
// `engine_story::variables` re-exports it. Its module doc is the struct doc now.
use std::collections::HashMap;
use std::path::Path;

/// `gamedata/story/story_variables.json.json`: the `$name` -> value table the
/// scripts reference (`[PlayMusic(intro="$escape_intro", key="$escape_loop")]`).
/// EN carries 2,611 keys. Values are mostly audio paths such as
/// `Sound_Beta_2/Music/act25side/m_avg_DecisiveBattle_loop`; some are avatar
/// ids (`avatar_amiya` -> `char_002_amiya`) and a few are numbers, so the file
/// is read as loose JSON and every value is kept as its string form.
#[derive(Debug, Default, Clone)]
pub struct StoryVariables {
    map: HashMap<String, String>,
}

impl StoryVariables {
    #[must_use]
    pub fn path(server_assets_dir: &Path) -> std::path::PathBuf {
        server_assets_dir.join("gamedata/story/story_variables.json.json")
    }

    /// Parse the table's bytes (read by the caller from [`Self::path`]). The
    /// caller keeps today's rule: a missing or malformed file yields an empty
    /// table and a warning, never a failure, so a server without stories
    /// still boots. Was `load(server_assets_dir)`, which read and warned here.
    pub fn parse(raw: &[u8]) -> Result<Self, serde_json::Error> {
        let parsed: HashMap<String, serde_json::Value> = serde_json::from_slice(raw)?;
        let map = parsed
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                (k, s)
            })
            .collect();
        Ok(Self { map })
    }

    #[must_use]
    pub const fn from_map(map: HashMap<String, String>) -> Self {
        Self { map }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Resolve a script reference. `$key` reads the table (`None` when the
    /// key is absent); a bare value is returned as itself.
    #[must_use]
    pub fn resolve<'a>(&'a self, reference: &'a str) -> Option<&'a str> {
        let r = reference.trim();
        match r.strip_prefix('$') {
            Some(key) => self.map.get(key).map(String::as_str),
            None => Some(r),
        }
    }
}
