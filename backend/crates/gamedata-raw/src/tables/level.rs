use serde::Deserialize;

use super::enemy::MaybeValue;

/// `gamedata/levels/`
pub const LEVELS_DIR: &str = "levels";

#[derive(Debug, Default)]
pub struct LevelTables {
    /// Every file under `gamedata/levels/`, relative to it
    pub files: Vec<String>,
    /// The level files, relative to `gamedata/levels/` and in path order, see
    /// [`is_level_file`]; `None` where one failed to read or parse
    pub levels: Vec<(String, Option<LevelFile>)>,
}

/// A `*.json` under `gamedata/levels/`, outside the `enemydata/` stats
/// directory and `levelreplacers/` (IS layout variants that duplicate a base
/// node). `rel_path` is relative to `gamedata/levels/`.
pub fn is_level_file(rel_path: &str) -> bool {
    let mut segments: Vec<&str> = rel_path.split('/').collect();
    let Some(file) = segments.pop() else {
        return false;
    };
    !segments
        .iter()
        .any(|dir| *dir == "enemydata" || *dir == "levelreplacers")
        && file
            .rsplit_once('.')
            .is_some_and(|(stem, ext)| !stem.is_empty() && ext == "json")
}

/// One level file, the enemies it declares and the ones its waves spawn
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LevelFile {
    #[serde(default)]
    pub enemy_db_refs: Vec<DbRef>,
    #[serde(default)]
    pub waves: Vec<Wave>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DbRef {
    pub id: String,
    /// Index into the enemy's `enemy_database.json` levels: the stage fields
    /// this level, not level 0
    #[serde(default)]
    pub level: i32,
    #[serde(default)]
    pub overwritten_data: Option<Overwritten>,
}

/// The per-stage stat overrides an `EnemyDbRefs` entry may carry, the two
/// stats the damage engine reads only
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Overwritten {
    #[serde(default)]
    pub attributes: Option<OverwrittenAttributes>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OverwrittenAttributes {
    #[serde(default)]
    pub def: Option<MaybeValue<f64>>,
    #[serde(default)]
    pub magic_resistance: Option<MaybeValue<f64>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Wave {
    #[serde(default)]
    pub fragments: Vec<Fragment>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Fragment {
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Action {
    #[serde(default)]
    pub action_type: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub count: i64,
}

#[cfg(test)]
mod tests {
    use super::is_level_file;

    #[test]
    fn level_files_skip_the_stats_and_replacer_directories() {
        assert!(is_level_file("obt/main/level_main_01-07.json"));
        assert!(is_level_file("levels_meta.json.json"));
        assert!(!is_level_file("enemydata/enemy_database.json"));
        assert!(!is_level_file(
            "obt/roguelike/ro4/levelreplacers/level_x.json"
        ));
        assert!(!is_level_file("obt/main/level_main_01-07.txt"));
        assert!(!is_level_file("obt/main/.json"));
    }
}
