//! `gamedata/story/story_variables.json.json`: the `$name` -> value table the
//! scripts reference (`[PlayMusic(intro="$escape_intro", key="$escape_loop")]`).
//! EN carries 2,611 keys. Values are mostly audio paths such as
//! `Sound_Beta_2/Music/act25side/m_avg_DecisiveBattle_loop`; some are avatar
//! ids (`avatar_amiya` -> `char_002_amiya`) and a few are numbers, so the file
//! is read as loose JSON and every value is kept as its string form.

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct StoryVariables {
    map: HashMap<String, String>,
}

impl StoryVariables {
    /// The file's path under a server's assets root.
    #[must_use]
    pub fn path(server_assets_dir: &Path) -> std::path::PathBuf {
        server_assets_dir.join("gamedata/story/story_variables.json.json")
    }

    /// Load the table; a missing or malformed file yields an empty table and
    /// a warning, never a failure, so a server without stories still boots.
    #[must_use]
    pub fn load(server_assets_dir: &Path) -> Self {
        let path = Self::path(server_assets_dir);
        let raw = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "story variables unreadable");
                return Self::default();
            }
        };
        let parsed: HashMap<String, serde_json::Value> = match serde_json::from_slice(&raw) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "story variables malformed");
                return Self::default();
            }
        };
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
        Self { map }
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
