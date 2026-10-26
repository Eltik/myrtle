#[derive(Debug, Default)]
pub struct StoryTables {}

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Error;
use serde_json::Value;
use serde_json::from_slice;

/// `gamedata/story/story_variables.json.json`, `$name` -> value table scripts
/// reference (e.g. `[PlayMusic(intro="$escape_intro", key="$escape_loop")]`).
/// Values are audio paths, avatar ids or numbers, all kept as strings
#[derive(Debug, Default, Clone)]
pub struct StoryVariables {
    map: HashMap<String, String>,
}

impl StoryVariables {
    #[must_use]
    pub fn path(server_assets_dir: &Path) -> PathBuf {
        server_assets_dir.join("gamedata/story/story_variables.json.json")
    }

    pub fn parse(raw: &[u8]) -> Result<Self, Error> {
        let parsed: HashMap<String, Value> = from_slice(raw)?;
        let map = parsed
            .into_iter()
            .map(|(k, v)| {
                let s = match v {
                    Value::String(s) => s,
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

    /// `$key` reads the table (`None` if absent), a bare value is returned as itself
    #[must_use]
    pub fn resolve<'a>(&'a self, reference: &'a str) -> Option<&'a str> {
        let r = reference.trim();
        match r.strip_prefix('$') {
            Some(key) => self.map.get(key).map(String::as_str),
            None => Some(r),
        }
    }
}
