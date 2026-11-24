//! Game data compiled from a local extract, once per test binary.

use std::io::Error;
use std::path::PathBuf;
use std::sync::OnceLock;

use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::{DirSource, TableSource, load};
use myrtle_types::Server;

type Models = [OnceLock<Option<GameModel>>; Server::ALL.len()];

const CHARACTER_TABLES: [&str; 12] = [
    "character_table",
    "char_patch_table",
    "skill_table",
    "uniequip_table",
    "battle_equip_table",
    "handbook_info_table",
    "handbook_team_table",
    "favor_table",
    "gamedata_const",
    "skin_table",
    "range_table",
    "building_data",
];

/// A [`DirSource`] that holds [`CHARACTER_TABLES`] and nothing else
struct CharacterTables(DirSource);

impl TableSource for CharacterTables {
    fn read_gamedata(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        let table = rel_path
            .strip_prefix("excel/")
            .and_then(|name| name.strip_suffix(".json"));
        if table.is_some_and(|name| CHARACTER_TABLES.contains(&name)) {
            self.0.read_gamedata(rel_path)
        } else {
            Ok(None)
        }
    }

    fn list_assets(&self) -> Result<Vec<String>, Error> {
        Ok(Vec::new())
    }
}

/// `MYRTLE_TEST_ASSETS`, or the repo's `assets/output`, whatever directory the tests run from
#[must_use]
pub fn assets_dir() -> PathBuf {
    std::env::var_os("MYRTLE_TEST_ASSETS").map_or_else(
        || {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/output"
            ))
        },
        PathBuf::from,
    )
}

/// The extract root of `server`, present or not
#[must_use]
pub fn root(server: Server) -> PathBuf {
    assets_dir().join(server.as_str())
}

fn compiled(models: &'static Models, server: Server, narrow: bool) -> Option<&'static GameModel> {
    models[server.index() as usize]
        .get_or_init(|| {
            let dir = DirSource { root: root(server) };
            if !dir
                .root
                .join("gamedata/excel/character_table.json")
                .is_file()
            {
                return None;
            }
            let raw = if narrow {
                load(&CharacterTables(dir), server)
            } else {
                load(&dir, server)
            };
            let raw = raw.expect("load the extract");
            let compiled = compile(raw, &CompileOptions::new(server)).expect("compile the extract");
            Some(compiled.model)
        })
        .as_ref()
}

/// `None` without an extract so the test skips, a panic on one that does not compile
#[must_use]
pub fn model(server: Server) -> Option<&'static GameModel> {
    static MODELS: Models = [const { OnceLock::new() }; Server::ALL.len()];
    compiled(&MODELS, server, false)
}

/// [`model`] over the character and building tables alone, every other table and the asset index empty
#[must_use]
pub fn character_model(server: Server) -> Option<&'static GameModel> {
    static MODELS: Models = [const { OnceLock::new() }; Server::ALL.len()];
    compiled(&MODELS, server, true)
}
