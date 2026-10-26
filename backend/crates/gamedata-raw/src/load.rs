use std::path::PathBuf;

use myrtle_types::Server;

use crate::tables;

/// Where table bytes come from
pub trait TableSource {
    /// Bytes of `excel/{name}.json`, `None` when absent
    fn read_table(&self, name: &str) -> Result<Option<Vec<u8>>, std::io::Error>;

    /// Relative paths of file under asset root
    fn list_assets(&self) -> Result<Vec<String>, std::io::Error>;
}

/// A local extract directory, `{ASSETS_DIR}/{server}`.
#[derive(Debug, Clone)]
pub struct DirSource {
    pub root: PathBuf,
}

impl TableSource for DirSource {
    fn read_table(&self, name: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        let _ = name;
        todo!()
    }

    fn list_assets(&self) -> Result<Vec<String>, std::io::Error> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("table {table}: read failed: {source}")]
    Io {
        table: String,
        #[source]
        source: std::io::Error,
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
    pub character: tables::character::CharacterTables,
    pub item: tables::item::ItemTables,
    pub stage: tables::stage::StageTables,
    pub activity: tables::activity::ActivityTables,
    pub gacha: tables::gacha::GachaTables,
    pub building: tables::building::BuildingTables,
    pub medal: tables::medal::MedalTables,
    pub roguelike: tables::roguelike::RoguelikeTables,
    pub sandbox: tables::sandbox::SandboxTables,
    pub enemy: tables::enemy::EnemyTables,
    pub story: tables::story::StoryTables,
    pub audio: tables::audio::AudioTables,
    pub autochess: tables::autochess::AutochessTables,
    /// Asset paths from [`TableSource::list_assets`]
    pub asset_paths: Vec<String>,
    /// Tables unpacker kept from earlier extract (`.skipped.json`)
    pub skipped: Vec<crate::integrity::SkippedTable>,
}

/// Load every table for `server`
pub fn load(source: &dyn TableSource, server: Server) -> Result<RawGameData, LoadError> {
    let _ = (source, server);
    todo!()
}
