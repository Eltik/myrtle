use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::core::local::gamedata::operators::enrich_all_operators;
use crate::core::local::gamedata::skills::enrich_all_skills;
use crate::core::local::types::handbook::Handbook;
use crate::core::local::types::material::Materials;
use crate::core::local::types::module::{BattleEquip, RawModules};
use crate::core::local::types::skill::RawSkill;
use crate::core::local::types::skin::SkinData;
use crate::core::local::types::{GameData, operator::CharacterTable};

#[derive(Debug)]
pub enum DataError {
    Io(std::io::Error),
    Parse {
        table: String,
        error: serde_json::Error,
    },
    Missing {
        table: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Io(e) => write!(f, "IO error: {}", e),
            DataError::Parse { table, error } => write!(f, "Failed to parse {}: {}", table, error),
            DataError::Missing { table } => write!(f, "Missing table file: {}", table),
        }
    }
}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

impl std::error::Error for DataError {}

pub struct DataHandler {
    data_dir: PathBuf,
}

impl DataHandler {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn load_table<T: DeserializeOwned>(&self, table_name: &str) -> Result<T, DataError> {
        let path = self.data_dir.join(format!("{}.json", table_name));
        let file = std::fs::File::open(&path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DataError::Missing {
                    table: table_name.to_string(),
                }
            } else {
                DataError::Io(e)
            }
        })?;
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| DataError::Parse {
            table: table_name.to_string(),
            error: e,
        })
    }

    pub fn load_table_or_empty<K, V>(&self, table_name: &str) -> HashMap<K, V>
    where
        K: std::hash::Hash + Eq,
        V: DeserializeOwned,
        HashMap<K, V>: DeserializeOwned,
    {
        self.load_table(table_name).unwrap_or_default()
    }
}

pub fn init_game_data(data_dir: &Path) -> Result<GameData, DataError> {
    let handler = DataHandler::new(data_dir);

    // Load character_table with wrapper struct
    let character_table = handler.load_table::<CharacterTable>("character_table")?;
    let raw_operators = character_table.characters;

    // TODO: Load other tables once they're decoded by the unpacker
    // For now, use empty defaults since only character_table is decoded
    let raw_skills: HashMap<String, RawSkill> = HashMap::new();
    let raw_modules = RawModules::default();
    let battle_equip = BattleEquip::default();
    let handbook = Handbook::default();
    let skins = SkinData::default();
    let materials = Materials::default();

    let skills = enrich_all_skills(raw_skills);

    let operators = enrich_all_operators(
        &raw_operators,
        &skills,
        &raw_modules,
        &battle_equip,
        &handbook,
        &skins,
    );

    Ok(GameData {
        operators,
        skills,
        materials,
    })
}

pub fn init_game_data_or_default(data_dir: &Path) -> GameData {
    match init_game_data(data_dir) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Warning: Failed to load game data: {}", e);
            eprintln!("Starting with empty game data");
            GameData::default()
        }
    }
}
