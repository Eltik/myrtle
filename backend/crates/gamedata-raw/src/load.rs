use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read};
use std::path::PathBuf;

use myrtle_types::Server;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, from_slice};
use thiserror::Error;
use walkdir::WalkDir;

use crate::integrity::SkippedTable;
use crate::sanitize::SanitizingReader;
use crate::tables::{
    activity::{ActivityTables, FARM_ARCHIVE, FarmArchive, scan_boss_stages, scan_skin_refs},
    audio::AudioTables,
    autochess::AutochessTables,
    building::BuildingTables,
    character::CharacterTables,
    enemy::{EnemyDatabaseFile, EnemyTables},
    event_shop::{EVENT_SHOP_REL_PATH, EventShopFile},
    gacha::GachaTables,
    gacha_detail::{POOL_DETAIL_FILE_VERSION, POOL_DETAIL_REL_PATH, PoolDetailFile},
    item::ItemTables,
    level::{LEVELS_DIR, LevelFile, LevelTables, is_level_file},
    medal::MedalTables,
    roguelike::RoguelikeTables,
    sandbox::SandboxTables,
    stage::StageTables,
    story::{AVG_CHARACTERS, PNG_HEAD_LEN, StorySpriteFiles, StoryTables},
};

/// Where table bytes come from
pub trait TableSource {
    /// Bytes of `gamedata/{rel_path}`, sanitized for serde, `None` when absent
    fn read_gamedata(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error>;

    /// Bytes of `excel/{name}.json`, `None` when absent
    fn read_table(&self, name: &str) -> Result<Option<Vec<u8>>, Error> {
        self.read_gamedata(&format!("excel/{name}.json"))
    }

    /// Relative paths of file under asset root
    fn list_assets(&self) -> Result<Vec<String>, Error>;

    /// Bytes of `{rel_path}` under the server root (`derived/` sidecars), `None` when absent
    fn read_derived(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        let _ = rel_path;
        Ok(None)
    }

    /// Paths of files under `gamedata/{rel_dir}`, relative to `gamedata/`, sorted
    fn list_gamedata(&self, rel_dir: &str) -> Result<Vec<String>, Error> {
        let _ = rel_dir;
        Ok(Vec::new())
    }

    /// Bytes of `{rel_path}` under the server root, unsanitized, `None` when absent
    fn read_asset(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        let _ = rel_path;
        Ok(None)
    }

    /// First `len` bytes of `{rel_path}` under the server root, `None` when absent or shorter
    fn read_asset_head(&self, rel_path: &str, len: usize) -> Result<Option<Vec<u8>>, Error> {
        let _ = (rel_path, len);
        Ok(None)
    }
}

/// Local extract directory, `{ASSETS_DIR}/{server}`
#[derive(Debug, Clone)]
pub struct DirSource {
    pub root: PathBuf,
}

impl TableSource for DirSource {
    fn read_gamedata(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        let path = self.root.join("gamedata").join(rel_path);
        let file = match File::open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut bytes = Vec::new();
        SanitizingReader::new(BufReader::new(file)).read_to_end(&mut bytes)?;
        Ok(Some(bytes))
    }

    fn list_assets(&self) -> Result<Vec<String>, Error> {
        let mut paths = Vec::new();
        let walk = WalkDir::new(&self.root)
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && (e.depth() != 1 || name != "gamedata")
            });
        for entry in walk {
            let entry = entry.map_err(Error::from)?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(rel) = entry
                .path()
                .strip_prefix(&self.root)
                .ok()
                .and_then(|p| p.to_str())
            else {
                continue;
            };
            paths.push(rel.replace('\\', "/"));
        }
        paths.sort_unstable();
        Ok(paths)
    }

    fn read_derived(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        self.read_asset(rel_path)
    }

    fn list_gamedata(&self, rel_dir: &str) -> Result<Vec<String>, Error> {
        let gamedata = self.root.join("gamedata");
        let dir = gamedata.join(rel_dir);
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut paths = Vec::new();
        for entry in WalkDir::new(&dir).min_depth(1) {
            let entry = entry.map_err(Error::from)?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(rel) = entry
                .path()
                .strip_prefix(&gamedata)
                .ok()
                .and_then(|p| p.to_str())
            else {
                continue;
            };
            paths.push(rel.replace('\\', "/"));
        }
        paths.sort_unstable();
        Ok(paths)
    }

    fn read_asset(&self, rel_path: &str) -> Result<Option<Vec<u8>>, Error> {
        match std::fs::read(self.root.join(rel_path)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn read_asset_head(&self, rel_path: &str, len: usize) -> Result<Option<Vec<u8>>, Error> {
        let mut file = match File::open(self.root.join(rel_path)) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut head = vec![0; len];
        match file.read_exact(&mut head) {
            Ok(()) => Ok(Some(head)),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
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
    pub level: LevelTables,
    /// Asset paths from [`TableSource::list_assets`]
    pub asset_paths: Vec<String>,
    /// Tables unpacker kept from earlier extract (`.skipped.json`)
    pub skipped: Vec<SkippedTable>,
    /// Optional tables that were missing or failed to parse and loaded as `Default`
    pub warnings: Vec<String>,
}

/// Load every table for `server`, only `character_table` is required
pub fn load(source: &dyn TableSource, server: Server) -> Result<RawGameData, LoadError> {
    let mut l = Loader {
        source,
        warnings: Vec::new(),
    };

    let character = CharacterTables {
        characters: l.required("character_table")?,
        patch: l.table("char_patch_table"),
        skills: l.table("skill_table"),
        uniequip: l.table("uniequip_table"),
        battle_equip: l.table("battle_equip_table"),
        handbook: l.table("handbook_info_table"),
        handbook_team: l.table("handbook_team_table"),
        favor: l.table("favor_table"),
        consts: l.table("gamedata_const"),
        skins: l.table("skin_table"),
        ranges: l.table("range_table"),
    };
    let item = ItemTables {
        items: l.table("item_table"),
        shop: l.table("shop_client_table"),
        display_meta: l.table("display_meta_table"),
    };
    let stage = StageTables {
        stages: l.table("stage_table"),
        zones: l.table("zone_table"),
        campaigns: l.table("campaign_table"),
        chapters: l.table("chapter_table"),
        retro: l.table("retro_table"),
        climb_tower: l.table("climb_tower_table"),
        crisis_v2: l.table("crisis_v2_table"),
    };
    let activity_buf = l.bytes("activity_table", "excel/activity_table.json");
    let activity_bytes = activity_buf.as_deref();
    let activity = ActivityTables {
        activities: l.parse_or_default("activity_table", activity_bytes),
        mission_archives: l.parse_or_default("activity_table#MissionArchives", activity_bytes),
        skin_refs: activity_bytes.map(scan_skin_refs).unwrap_or_default(),
        boss_stages: activity_bytes.map(scan_boss_stages).unwrap_or_default(),
        missions: l.table("mission_table"),
        farm_archive: l.farm_archive(),
        event_shops: l
            .derived::<Option<EventShopFile>>("event-shops", EVENT_SHOP_REL_PATH)
            .map(|f| f.shops)
            .unwrap_or_default(),
    };
    let autochess = AutochessTables {
        autochess: l.parse_or_default("activity_table#AutoChessData", activity_bytes),
    };
    let gacha = GachaTables {
        gacha: l.table("gacha_table"),
        open_server: l.table("open_server_table"),
        pool_details: l.pool_details(),
    };
    let building = BuildingTables {
        building: l.table("building_data"),
    };
    let medal = MedalTables {
        medals: l.table("medal_table"),
    };
    let roguelike = RoguelikeTables {
        topics: l.table("roguelike_topic_table"),
    };
    let sandbox = SandboxTables {
        perm: l.table("sandbox_perm_table"),
        table: l.table("sandbox_table"),
    };
    let enemy = EnemyTables {
        handbook: l.table("enemy_handbook_table"),
        database: l.file::<Option<EnemyDatabaseFile>>(
            "enemy_database",
            "levels/enemydata/enemy_database.json",
        ),
    };
    let audio = AudioTables {
        audio: l.table("audio_data"),
        voices: l.table("charword_table"),
    };
    let level = l.levels();

    let skipped = l
        .file::<Option<SkippedFile>>("skipped", ".skipped.json")
        .map(|f| f.tables)
        .unwrap_or_default();
    let asset_paths = match source.list_assets() {
        Ok(paths) => paths,
        Err(e) => {
            l.warnings.push(format!("asset listing failed: {e}"));
            Vec::new()
        }
    };
    let story = StoryTables {
        review: l.table("story_review_table"),
        review_meta: l.table("story_review_meta_table"),
        scripts: l.story_scripts(),
        sprites: l.story_sprites(&asset_paths),
    };

    Ok(RawGameData {
        server: Some(server),
        character,
        item,
        stage,
        activity,
        gacha,
        building,
        medal,
        roguelike,
        sandbox,
        enemy,
        story,
        audio,
        autochess,
        level,
        asset_paths,
        skipped,
        warnings: l.warnings,
    })
}

/// `gamedata/.skipped.json`, written by the unpacker
#[derive(Deserialize)]
struct SkippedFile {
    tables: Vec<SkippedTable>,
}

struct Loader<'a> {
    source: &'a dyn TableSource,
    warnings: Vec<String>,
}

impl Loader<'_> {
    /// `excel/{name}.json`; any failure is an error
    fn required<T: DeserializeOwned>(&self, name: &str) -> Result<T, LoadError> {
        let bytes = self
            .source
            .read_table(name)
            .map_err(|source| LoadError::Io {
                table: name.to_owned(),
                source,
            })?
            .ok_or_else(|| LoadError::Missing {
                table: name.to_owned(),
            })?;
        parse(name, &bytes)
    }

    /// `excel/{name}.json`; any failure is a warning and `T::default()`
    fn table<T: DeserializeOwned + Default>(&mut self, name: &str) -> T {
        self.file(name, &format!("excel/{name}.json"))
    }

    /// `gamedata/{rel_path}`; any failure is a warning and `T::default()`
    fn file<T: DeserializeOwned + Default>(&mut self, name: &str, rel_path: &str) -> T {
        let bytes = self.bytes(name, rel_path);
        self.parse_or_default(name, bytes.as_deref())
    }

    /// Bytes of `gamedata/{rel_path}`; a missing or unreadable file is a warning and `None`
    fn bytes(&mut self, name: &str, rel_path: &str) -> Option<Vec<u8>> {
        let read = self.source.read_gamedata(rel_path);
        self.take(name, read)
    }

    /// Bytes of a read; a missing or unreadable file is a warning and `None`
    fn take(&mut self, name: &str, read: Result<Option<Vec<u8>>, Error>) -> Option<Vec<u8>> {
        match read {
            Ok(Some(bytes)) => Some(bytes),
            Ok(None) => {
                if name != "skipped" {
                    self.warnings.push(format!("table {name}: missing"));
                }
                None
            }
            Err(source) => {
                let e = LoadError::Io {
                    table: name.to_owned(),
                    source,
                };
                self.warnings.push(e.to_string());
                None
            }
        }
    }

    /// `{rel_path}` under the server root; any failure is a warning and `T::default()`
    fn derived<T: DeserializeOwned + Default>(&mut self, name: &str, rel_path: &str) -> T {
        let bytes = self.derived_bytes(name, rel_path);
        self.parse_or_default(name, bytes.as_deref())
    }

    /// Bytes of `{rel_path}` under the server root; a missing or unreadable file is a warning and `None`
    fn derived_bytes(&mut self, name: &str, rel_path: &str) -> Option<Vec<u8>> {
        let read = self.source.read_derived(rel_path);
        self.take(name, read)
    }

    /// Files under `gamedata/{rel_dir}`; a failed listing is a warning and empty
    fn list(&mut self, name: &str, rel_dir: &str) -> Vec<String> {
        self.source.list_gamedata(rel_dir).unwrap_or_else(|e| {
            self.warnings
                .push(format!("table {name}: listing failed: {e}"));
            Vec::new()
        })
    }

    /// Bytes of a read; a missing file is `None`, an unreadable one a warning and `None`
    fn optional(&mut self, name: &str, read: Result<Option<Vec<u8>>, Error>) -> Option<Vec<u8>> {
        match read {
            Ok(bytes) => bytes,
            Err(source) => {
                let e = LoadError::Io {
                    table: name.to_owned(),
                    source,
                };
                self.warnings.push(e.to_string());
                None
            }
        }
    }

    /// `gamedata/levels/`: every file, and each level file parsed; a level that
    /// fails to read or parse is a warning and `None`
    fn levels(&mut self) -> LevelTables {
        let prefix = format!("{LEVELS_DIR}/");
        let files: Vec<String> = self
            .list("levels", LEVELS_DIR)
            .into_iter()
            .filter_map(|p| p.strip_prefix(&prefix).map(str::to_owned))
            .collect();
        let mut levels = Vec::new();
        for rel in files.iter().filter(|rel| is_level_file(rel)) {
            let name = format!("{prefix}{rel}");
            let read = self.source.read_asset(&format!("gamedata/{name}"));
            let level =
                self.take(&name, read)
                    .and_then(|bytes| match parse::<LevelFile>(&name, &bytes) {
                        Ok(level) => Some(level),
                        Err(e) => {
                            self.warnings.push(e.to_string());
                            None
                        }
                    });
            levels.push((rel.clone(), level));
        }
        LevelTables { files, levels }
    }

    /// `gamedata/story/**/*.txt`; a script that is not UTF-8 is skipped
    fn story_scripts(&mut self) -> Vec<String> {
        let mut scripts = Vec::new();
        for rel in self.list("story", "story") {
            if rel
                .rsplit_once('.')
                .is_none_or(|(stem, ext)| stem.is_empty() || ext != "txt")
            {
                continue;
            }
            let read = self.source.read_asset(&format!("gamedata/{rel}"));
            if let Some(text) = self
                .optional(&rel, read)
                .and_then(|bytes| String::from_utf8(bytes).ok())
            {
                scripts.push(text);
            }
        }
        scripts
    }

    /// Each `textures/avg/characters/{folder}/`'s `hub.json` and PNG headers;
    /// a hub that does not parse is a warning
    fn story_sprites(&mut self, asset_paths: &[String]) -> StorySpriteFiles {
        let mut files = StorySpriteFiles::default();
        for rel in asset_paths {
            let Some((folder, file)) = rel
                .strip_prefix(AVG_CHARACTERS)
                .and_then(|r| r.split_once('/'))
                .filter(|(_, file)| !file.contains('/'))
            else {
                continue;
            };
            if file == "hub.json" {
                let read = self.source.read_asset(rel);
                let Some(bytes) = self.optional(rel, read) else {
                    continue;
                };
                match parse::<Value>(rel, &bytes) {
                    Ok(hub) => {
                        files.hubs.insert(folder.to_owned(), hub);
                    }
                    Err(e) => self.warnings.push(e.to_string()),
                }
            } else if file
                .rsplit_once('.')
                .is_some_and(|(stem, ext)| !stem.is_empty() && ext == "png")
            {
                let read = self.source.read_asset_head(rel, PNG_HEAD_LEN);
                if let Some(head) = self.optional(rel, read) {
                    files.png_heads.insert(rel.clone(), head);
                }
            }
        }
        files
    }

    /// `derived/gacha_pool_details.json`; another version is a warning and `None`
    fn pool_details(&mut self) -> Option<PoolDetailFile> {
        let file =
            self.derived::<Option<PoolDetailFile>>("gacha_pool_details", POOL_DETAIL_REL_PATH)?;
        if file.version == POOL_DETAIL_FILE_VERSION {
            Some(file)
        } else {
            self.warnings.push(format!(
                "table gacha_pool_details: version {} != expected {POOL_DETAIL_FILE_VERSION}, ignoring",
                file.version
            ));
            None
        }
    }

    /// `derived/farm-stages.json`; missing and unreadable are warnings, kept apart
    fn farm_archive(&mut self) -> FarmArchive {
        let error = match self.source.read_derived(FARM_ARCHIVE) {
            Ok(None) => {
                self.warnings.push("table farm-stages: missing".to_owned());
                return FarmArchive::Absent;
            }
            Ok(Some(bytes)) => match parse("farm-stages", &bytes) {
                Ok(archive) => return FarmArchive::Loaded(archive),
                Err(e) => e,
            },
            Err(source) => LoadError::Io {
                table: "farm-stages".to_owned(),
                source,
            },
        };
        self.warnings.push(error.to_string());
        FarmArchive::Unreadable
    }

    /// `T::default()` for `None` (already warned) or a parse failure (warned here)
    fn parse_or_default<T: DeserializeOwned + Default>(
        &mut self,
        name: &str,
        bytes: Option<&[u8]>,
    ) -> T {
        let Some(bytes) = bytes else {
            return T::default();
        };
        parse(name, bytes).unwrap_or_else(|e| {
            self.warnings.push(e.to_string());
            T::default()
        })
    }
}

fn parse<T: DeserializeOwned>(table: &str, bytes: &[u8]) -> Result<T, LoadError> {
    from_slice(bytes).map_err(|e| LoadError::Parse {
        table: table.to_owned(),
        path: format!("line {} column {}", e.line(), e.column()),
        message: e.to_string(),
    })
}
