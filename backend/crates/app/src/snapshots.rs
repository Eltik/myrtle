use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use arc_swap::ArcSwapOption;
use myrtle_db::repos::data_generations::Generation;
use myrtle_engine_base::BaseData;
use myrtle_engine_grade::GradeData;
use myrtle_engine_story::{StoryAssetIndex, StoryLibrary, build_index};
use myrtle_gamedata::compile::{CompileOptions, compile};
use myrtle_gamedata::enrich::reference::align;
use myrtle_gamedata::model::{GameModel, GameVersion};
use myrtle_gamedata_raw::tables::event_shop::EVENT_SHOP_REL_PATH;
use myrtle_gamedata_raw::tables::gacha_detail::POOL_DETAIL_REL_PATH;
use myrtle_gamedata_raw::tables::sandbox_universe::SandboxUniverse;
use myrtle_gamedata_raw::{DirSource, TableSource, load};
use myrtle_types::Server;

use crate::AppError;
use crate::cache::key_hash;
use crate::config::Config;

pub const RELOAD_RETRY: Duration = Duration::from_mins(10);
pub const STORY_RETRY: Duration = Duration::from_secs(30);
const RELEASE_ART: &str = "derived/release-art";

pub struct Snapshot {
    pub model: GameModel,
    pub grade: GradeData,
    pub base: BaseData,
    pub generation: i64,
    pub res_version: Option<String>,
    pub pipeline_seq: i64,
    story: StoryCell,
}

pub struct Story {
    pub library: StoryLibrary,
    pub assets: StoryAssetIndex,
    pub(crate) derived: crate::services::story::Derived,
}

#[derive(Default)]
struct StoryCell {
    dirs: Option<(PathBuf, PathBuf)>,
    built: OnceLock<Story>,
    failed: Mutex<Option<(Instant, String)>>,
    gate: tokio::sync::Mutex<()>,
}

struct NoFiles;

impl TableSource for NoFiles {
    fn read_gamedata(&self, _: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        Ok(None)
    }

    fn list_assets(&self) -> Result<Vec<String>, std::io::Error> {
        Ok(Vec::new())
    }
}

struct ArtSplit {
    text: DirSource,
    art: DirSource,
}

impl ArtSplit {
    fn tree(&self, rel_path: &str) -> &DirSource {
        if rel_path.starts_with("gamedata/") {
            &self.text
        } else {
            &self.art
        }
    }
}

impl TableSource for ArtSplit {
    fn read_gamedata(&self, rel_path: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        self.text.read_gamedata(rel_path)
    }

    fn list_assets(&self) -> Result<Vec<String>, std::io::Error> {
        self.art.list_assets()
    }

    fn read_derived(&self, rel_path: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        self.text.read_derived(rel_path)
    }

    fn list_gamedata(&self, rel_dir: &str) -> Result<Vec<String>, std::io::Error> {
        self.text.list_gamedata(rel_dir)
    }

    fn read_asset(&self, rel_path: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        self.tree(rel_path).read_asset(rel_path)
    }

    fn read_asset_head(
        &self,
        rel_path: &str,
        len: usize,
    ) -> Result<Option<Vec<u8>>, std::io::Error> {
        self.tree(rel_path).read_asset_head(rel_path, len)
    }
}

impl Snapshot {
    #[must_use]
    pub fn key(&self, text: &str) -> String {
        format!("{text}:{}", self.model.meta.game_version.res)
    }

    pub fn load(root: &Path, options: &CompileOptions) -> Result<Self, AppError> {
        Self::load_with_art(root, root, options, &[])
    }

    pub fn load_with_art(
        root: &Path,
        art: &Path,
        options: &CompileOptions,
        references: &[Arc<Self>],
    ) -> Result<Self, AppError> {
        let server = options.server;
        let failed = |e: &dyn std::fmt::Display| {
            AppError::Internal(format!("game data for {}: {e}", server.as_str()))
        };
        let source = ArtSplit {
            text: DirSource {
                root: root.to_path_buf(),
            },
            art: DirSource {
                root: art.to_path_buf(),
            },
        };
        let raw = load(&source, server).map_err(|e| failed(&e))?;
        let sandbox_universe = SandboxUniverse::build(&raw.sandbox.perm);
        let compiled = compile(raw, options).map_err(|e| failed(&e))?;
        for warning in &compiled.report.warnings {
            tracing::warn!(server = server.as_str(), warning, "game data compile");
        }
        for file in &compiled.report.derived_files {
            let path = root.join(file.rel_path);
            if let Err(e) = write_atomic(&path, &file.bytes) {
                tracing::warn!(path = %path.display(), error = %e, "derived file not written");
            }
        }
        let mut model = compiled.model;
        if !references.is_empty() {
            let references: Vec<&GameModel> = references.iter().map(|r| &r.model).collect();
            let facts = align(&mut model, &references, options);
            tracing::info!(
                server = server.as_str(),
                ?facts,
                "aligned on the other servers"
            );
        }
        let mut snapshot = Self::from_model(model, sandbox_universe);
        snapshot.story.dirs = Some((art.to_path_buf(), root.to_path_buf()));
        Ok(snapshot)
    }

    pub fn story(&self) -> Result<&Story, AppError> {
        let failure = |e: &str| AppError::Internal(format!("story library: {e}"));
        if let Some(story) = self.story.built.get() {
            return Ok(story);
        }
        let mut failed = self
            .story
            .failed
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(story) = self.story.built.get() {
            return Ok(story);
        }
        if let Some((_, e)) = failed.as_ref().filter(|(at, _)| at.elapsed() < STORY_RETRY) {
            return Err(failure(e));
        }
        let built = self.build_story();
        *failed = built.as_ref().err().map(|e| (Instant::now(), e.clone()));
        let story = built
            .map(|story| self.story.built.get_or_init(|| story))
            .map_err(|e| failure(&e));
        drop(failed);
        story
    }

    #[must_use]
    pub fn story_dirs(&self) -> Option<(&Path, &Path)> {
        self.story
            .dirs
            .as_ref()
            .map(|(art, text)| (art.as_path(), text.as_path()))
    }

    #[must_use]
    pub fn story_built(&self) -> bool {
        self.story.built.get().is_some()
    }

    pub async fn story_gate(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.story.gate.lock().await
    }

    fn build_story(&self) -> Result<Story, String> {
        let started = Instant::now();
        let (assets, library) = if let Some((art, text)) = &self.story.dirs {
            let (art, text) = (
                DirSource { root: art.clone() },
                DirSource { root: text.clone() },
            );
            let assets = StoryAssetIndex::build(&art, &text, &|rel| {
                crate::services::story::list_story_tree(&art.root, rel)
            })
            .map_err(|e| e.to_string())?;
            let library = build_index(&self.model, &self.model.assets, &assets, &text);
            (assets, library)
        } else {
            let assets = StoryAssetIndex::default();
            let library = build_index(&self.model, &self.model.assets, &assets, &NoFiles);
            (assets, library)
        };
        tracing::info!(
            groups = library.index.groups.len(),
            stories = library.lookup.len(),
            build_ms = started.elapsed().as_millis(),
            "story library built"
        );
        Ok(Story {
            library,
            assets,
            derived: crate::services::story::Derived::default(),
        })
    }

    #[must_use]
    pub fn from_model(model: GameModel, sandbox_universe: SandboxUniverse) -> Self {
        let grade = GradeData {
            operators: model.operators.clone(),
            favor: model.favor.clone(),
            roguelike: model.roguelike.clone(),
            medals: model.medals.clone(),
            stage_universe: model.stage_universe.clone(),
            sandbox_universe,
        };
        let base = BaseData::new(
            model.operators.clone(),
            model.building.clone(),
            model.consts.clone(),
        );
        Self {
            model,
            grade,
            base,
            generation: 0,
            res_version: None,
            pipeline_seq: 0,
            story: StoryCell::default(),
        }
    }
}

fn references_hash(references: &[Arc<Snapshot>]) -> String {
    let mut text = String::new();
    for reference in references {
        let meta = &reference.model.meta;
        let _ = writeln!(text, "{}\0{}", meta.server.as_str(), meta.game_version.res);
    }
    key_hash(text.as_bytes())
}

#[must_use]
pub fn game_version(root: &Path) -> GameVersion {
    let excel = root.join("gamedata/excel");
    let client = ["data_version.txt.txt", "data_version.txt"]
        .iter()
        .find_map(|name| std::fs::read_to_string(excel.join(name)).ok())
        .and_then(|text| {
            text.lines()
                .find_map(|l| l.strip_prefix("VersionControl:"))
                .map(|v| v.trim().to_owned())
        })
        .unwrap_or_default();
    GameVersion {
        client,
        res: fingerprint(root).unwrap_or_default(),
    }
}

#[must_use]
pub fn fingerprint(root: &Path) -> Option<String> {
    if !root.join("gamedata").is_dir() {
        return None;
    }
    let mut files = Vec::new();
    for dir in ["gamedata", RELEASE_ART] {
        walk(root, &root.join(dir), &mut files);
    }
    for file in [EVENT_SHOP_REL_PATH, POOL_DETAIL_REL_PATH] {
        if let Ok(meta) = std::fs::metadata(root.join(file)) {
            files.push((file.to_owned(), meta.len(), mtime(&meta)));
        }
    }
    files.sort_unstable();
    let mut text = String::new();
    for (rel, len, modified) in &files {
        let _ = writeln!(text, "{rel}\0{len}\0{}", nanos(*modified));
    }
    Some(key_hash(text.as_bytes()))
}

fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, u64, SystemTime)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        let path = entry.path();
        if meta.is_dir() {
            walk(root, &path, files);
        } else if let (true, Some(rel)) = (
            meta.is_file(),
            path.strip_prefix(root).ok().and_then(Path::to_str),
        ) {
            files.push((rel.replace('\\', "/"), meta.len(), mtime(&meta)));
        }
    }
}

fn mtime(meta: &std::fs::Metadata) -> SystemTime {
    meta.modified().unwrap_or(UNIX_EPOCH)
}

fn nanos(t: SystemTime) -> u128 {
    t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos())
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = dir.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let written = std::fs::File::create(&tmp).and_then(|mut f| {
        f.write_all(bytes)?;
        f.sync_all()
    });
    if let Err(e) = written.and_then(|()| std::fs::rename(&tmp, path)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::File::open(dir)?.sync_all()
}

#[derive(Debug, Clone, Copy, Default)]
struct Progress {
    done: Option<i64>,
    failed: Option<(i64, Instant)>,
}

pub struct SnapshotRegistry {
    assets_dir: PathBuf,
    default_server: Server,
    profile_align: bool,
    obtain_channel_align: bool,
    art_fallback: bool,
    retry: Duration,
    cells: HashMap<Server, ArcSwapOption<Snapshot>>,
    progress: HashMap<Server, Mutex<Progress>>,
    build: tokio::sync::Mutex<()>,
    reloading: tokio::sync::Mutex<()>,
}

impl SnapshotRegistry {
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let servers: Vec<Server> = config
            .server
            .servers
            .iter()
            .map(|s| s.data_server())
            .collect();
        Self {
            assets_dir: PathBuf::from(&config.assets.dir),
            default_server: servers.first().copied().unwrap_or(Server::EN),
            profile_align: config.switches.profile_align.enabled,
            obtain_channel_align: config.switches.obtain_channel_align.enabled,
            art_fallback: config.switches.art_fallback.enabled,
            retry: RELOAD_RETRY,
            cells: servers
                .iter()
                .map(|s| (*s, ArcSwapOption::empty()))
                .collect(),
            progress: servers.iter().map(|s| (*s, Mutex::default())).collect(),
            build: tokio::sync::Mutex::new(()),
            reloading: tokio::sync::Mutex::new(()),
        }
    }

    #[must_use]
    pub const fn with_retry(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    #[must_use]
    pub const fn default_server(&self) -> Server {
        self.default_server
    }

    #[must_use]
    pub fn servers(&self) -> Vec<Server> {
        let mut servers: Vec<Server> = self.cells.keys().copied().collect();
        servers.sort_by_key(|s| (*s != self.default_server, s.as_str()));
        servers
    }

    #[must_use]
    pub fn server_dir(&self, server: Server) -> PathBuf {
        self.assets_dir.join(server.data_server().as_str())
    }

    #[must_use]
    pub fn has_own_art(&self, server: Server) -> bool {
        server.data_server() == self.default_server
            || !self.art_fallback
            || self.server_dir(server).join("portraits").is_dir()
    }

    #[must_use]
    pub fn art_dir(&self, server: Server) -> PathBuf {
        self.server_dir(if self.has_own_art(server) {
            server
        } else {
            self.default_server
        })
    }

    #[must_use]
    pub fn get(&self, server: Server) -> Option<Arc<Snapshot>> {
        self.cells.get(&server.data_server())?.load_full()
    }

    pub fn require(&self, server: Server) -> Result<Arc<Snapshot>, AppError> {
        self.get(server).ok_or_else(|| {
            AppError::NotFound(Some(format!("no game data for {}", server.as_str())))
        })
    }

    pub fn or_default(&self, server: Server) -> Result<Arc<Snapshot>, AppError> {
        self.get(server)
            .or_else(|| self.get(self.default_server))
            .ok_or(AppError::Unavailable)
    }

    #[must_use]
    pub fn loaded(&self) -> Vec<Server> {
        std::iter::once(self.default_server)
            .chain(
                Server::ALL
                    .into_iter()
                    .filter(|s| *s != self.default_server),
            )
            .filter(|s| s.data_server() == *s && self.get(*s).is_some())
            .collect()
    }

    pub async fn activate_at(&self, server: Server, stored: &Generation) -> Result<(), AppError> {
        let loaded = self.load_at(server, stored).await;
        match &loaded {
            Ok(_) => self.done(server, stored.generation),
            Err(_) => self.failed(server, stored.generation),
        }
        loaded.map(drop)
    }

    pub(crate) async fn load_at(
        &self,
        server: Server,
        stored: &Generation,
    ) -> Result<Arc<Snapshot>, AppError> {
        let server = server.data_server();
        let generation = stored.generation;
        self.progress(server)?;
        let _build = self.build.lock().await;
        let (root, art) = (self.server_dir(server), self.art_dir(server));
        let (profile_align, obtain_channel_align) = (self.profile_align, self.obtain_channel_align);
        let references: Vec<Arc<Snapshot>> = if server == self.default_server {
            Vec::new()
        } else {
            let others = self.loaded().into_iter().filter(|s| *s != server);
            others.filter_map(|s| self.get(s)).collect()
        };
        let aligned_on = (server != self.default_server).then(|| references_hash(&references));
        let snapshot = tokio::task::spawn_blocking(move || {
            let mut game_version = game_version(&root);
            game_version.res = match aligned_on {
                Some(references) => format!("{generation}-{}-{references}", game_version.res),
                None => format!("{generation}-{}", game_version.res),
            };
            let options = CompileOptions {
                game_version,
                profile_align,
                obtain_channel_align,
                ..CompileOptions::new(server)
            };
            Snapshot::load_with_art(&root, &art, &options, &references)
        })
        .await
        .map_err(AppError::internal)??;
        let snapshot = Arc::new(Snapshot {
            generation,
            res_version: stored.res_version.clone(),
            pipeline_seq: stored.pipeline_seq,
            ..snapshot
        });
        self.install_shared(server, Arc::clone(&snapshot));
        Ok(snapshot)
    }

    pub(crate) async fn reloading(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.reloading.lock().await
    }

    pub(crate) fn due(&self, stored: &HashMap<Server, Generation>) -> Vec<(Server, Generation)> {
        let mut due: Vec<(Server, Generation)> = self
            .progress
            .iter()
            .filter_map(|(server, progress)| {
                let want = stored.get(server).cloned().unwrap_or_default();
                let progress = *progress.lock().unwrap_or_else(PoisonError::into_inner);
                let waiting = progress.failed.is_some_and(|(failed, at)| {
                    failed == want.generation && at.elapsed() < self.retry
                });
                (progress.done != Some(want.generation) && !waiting).then_some((*server, want))
            })
            .collect();
        due.sort_by_key(|(s, _)| (*s != self.default_server, s.as_str()));
        due
    }

    pub(crate) fn done(&self, server: Server, generation: i64) {
        self.update(server, |p| {
            *p = Progress {
                done: Some(generation),
                failed: None,
            };
        });
    }

    pub(crate) fn failed(&self, server: Server, generation: i64) {
        self.update(server, |p| p.failed = Some((generation, Instant::now())));
    }

    fn update(&self, server: Server, f: impl FnOnce(&mut Progress)) {
        if let Some(progress) = self.progress.get(&server.data_server()) {
            f(&mut progress.lock().unwrap_or_else(PoisonError::into_inner));
        }
    }

    fn progress(&self, server: Server) -> Result<Progress, AppError> {
        self.progress
            .get(&server.data_server())
            .map(|p| *p.lock().unwrap_or_else(PoisonError::into_inner))
            .ok_or_else(|| {
                AppError::NotFound(Some(format!(
                    "{} is not a configured server",
                    server.as_str()
                )))
            })
    }

    pub fn install(&self, server: Server, snapshot: Snapshot) {
        self.install_shared(server, Arc::new(snapshot));
    }

    pub fn install_shared(&self, server: Server, snapshot: Arc<Snapshot>) {
        if let Some(cell) = self.cells.get(&server.data_server()) {
            cell.store(Some(snapshot));
        }
    }
}
