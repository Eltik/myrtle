use std::io::Error;
use std::path::PathBuf;

use myrtle_types::Server;
use thiserror::Error;

use crate::integrity::SkippedTable;
use crate::tables::{
    activity::ActivityTables, audio::AudioTables, autochess::AutochessTables,
    building::BuildingTables, character::CharacterTables, enemy::EnemyTables, gacha::GachaTables,
    item::ItemTables, medal::MedalTables, roguelike::RoguelikeTables, sandbox::SandboxTables,
    stage::StageTables, story::StoryTables,
};

/// Where table bytes come from
pub trait TableSource {
    /// Bytes of `excel/{name}.json`, `None` when absent
    fn read_table(&self, name: &str) -> Result<Option<Vec<u8>>, Error>;

    /// Relative paths of file under asset root
    fn list_assets(&self) -> Result<Vec<String>, Error>;
}

/// A local extract directory, `{ASSETS_DIR}/{server}`.
#[derive(Debug, Clone)]
pub struct DirSource {
    pub root: PathBuf,
}

impl TableSource for DirSource {
    fn read_table(&self, name: &str) -> Result<Option<Vec<u8>>, Error> {
        let _ = name;
        todo!()
    }

    fn list_assets(&self) -> Result<Vec<String>, Error> {
        todo!()
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("table {table}: read failed: {source}")]
    Io {
        table: String,
        #[source]
        source: Error,
    },
    #[error("table {table}: parse failed at {path}: {message}")]
    Parse {
        table: String,
        path: String,
        message: String,
    },
    #[error("table {table} is required and missing")]
    Missing { table: String },
}

#[derive(Debug, Default)]
pub struct RawGameData {
    pub server: Option<Server>,
    pub character: CharacterTables,
    pub item: ItemTables,
    pub stage: StageTables,
    pub activity: ActivityTables,
    pub gacha: GachaTables,
    pub building: BuildingTables,
    pub medal: MedalTables,
    pub roguelike: RoguelikeTables,
    pub sandbox: SandboxTables,
    pub enemy: EnemyTables,
    pub story: StoryTables,
    pub audio: AudioTables,
    pub autochess: AutochessTables,
    /// Asset paths from [`TableSource::list_assets`]
    pub asset_paths: Vec<String>,
    /// Tables unpacker kept from earlier extract (`.skipped.json`)
    pub skipped: Vec<SkippedTable>,
}

/// Load every table for `server`
pub fn load(source: &dyn TableSource, server: Server) -> Result<RawGameData, LoadError> {
    let _ = (source, server);
    todo!()
}
