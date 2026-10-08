//! Game-data lists from the public backend, cached in memory.
//!
//! The backend has no name-search endpoint, so `/collection` and the birthday announcer fetch
//! whole lists (operators, enemies, stages, story groups) and match locally. Each list lives in
//! its own [`Slot`]: fresh for [`LIST_TTL`], then served stale while one background refresh runs,
//! so a lookup only ever waits on the network when the bot has never fetched that list.
//!
//! Field names follow the real responses (checked against `api.myrtle.moe`), and every field the
//! bot doesn't print is left out so serde skips it.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer};
use tokio::sync::{Mutex, RwLock};

use crate::api::operator_detail::{
    OperatorDetail, ParadoxEntry, ParadoxStage, Range, SkinData, TableSkill, VoiceData,
};
use crate::types::Error;
use crate::utils::non_blank;

/// How long a fetched list counts as fresh. Game data changes on patch days, not by the minute.
pub const LIST_TTL: Duration = Duration::from_mins(30);
/// Per-request timeout. The enemy and stage lists run to a few MB, so the 5 s status-check
/// timeout in `api::CONFIG_TIMEOUT` is too tight for them.
const LIST_TIMEOUT: Duration = Duration::from_secs(30);
/// After a failed refresh, how long the stale list is served before the next attempt. Without
/// it every autocomplete keystroke would retry a backend that is down.
const RETRY_AFTER: Duration = Duration::from_secs(60);

/// One operator from `GET /api/operators/index`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operator {
    pub id: String,
    pub name: String,
    /// The codename in another script (Rosa's is "Роса"). A single space for nearly everyone.
    #[serde(default)]
    pub appellation: String,
    pub rarity: u8,
    /// Class and branch names. `Option` on the backend: null when the server's own tables
    /// don't name the id, and one null must not fail the whole index.
    pub profession_name: Option<String>,
    pub sub_profession_name: Option<String>,
    pub position: String,
    #[serde(default)]
    pub tag_list: Vec<String>,
    pub nation_name: Option<String>,
    pub group_name: Option<String>,
    pub team_name: Option<String>,
    #[serde(default)]
    pub artists: Vec<String>,
    #[serde(default)]
    pub voice_actors: Vec<String>,
    /// Verbatim profile text: "Dec. 23", "Sept 17th", "Unknown", or empty.
    #[serde(default)]
    pub date_of_birth: String,
    #[serde(default)]
    pub is_not_obtainable: bool,
}

impl Operator {
    /// The appellation when it says something the name doesn't.
    #[must_use]
    pub fn appellation(&self) -> Option<&str> {
        let a = self.appellation.trim();
        (!a.is_empty() && a != self.name).then_some(a)
    }

    /// The class name ("Caster"), when the backend has one.
    #[must_use]
    pub fn class(&self) -> Option<&str> {
        non_blank(self.profession_name.as_deref())
    }

    /// The branch name ("Core Caster"), when the backend has one.
    #[must_use]
    pub fn branch(&self) -> Option<&str> {
        non_blank(self.sub_profession_name.as_deref())
    }

    /// Nation, group and team, whichever the operator has, most general first.
    #[must_use]
    pub fn factions(&self) -> Vec<&str> {
        [&self.nation_name, &self.group_name, &self.team_name]
            .into_iter()
            .filter_map(|f| f.as_deref().map(str::trim).filter(|f| !f.is_empty()))
            .collect()
    }
}

/// One enemy, flattened out of `GET /api/static/enemies` (`enemyData`, keyed by id).
#[derive(Debug, Clone)]
pub struct Enemy {
    pub id: String,
    /// Handbook index such as "B1"; absent for a few summons.
    pub index: Option<String>,
    pub name: String,
    /// `NORMAL`, `ELITE` or `BOSS`.
    pub level: Option<String>,
    pub description: Option<String>,
    /// `(text, textFormat)`; the format is `NORMAL`, `TITLE` or `SILENCE`.
    pub abilities: Vec<(String, String)>,
    /// `PHYSIC`, `MAGIC`, `HEAL`, `NO_DAMAGE`.
    pub damage_types: Vec<String>,
    /// Level-0 `applyWay`: `MELEE`, `RANGED`, `ALL` or `NONE`. The top-level `attackType` is
    /// null on every enemy in the real data, so this is the only attack-type signal.
    pub apply_way: Option<String>,
    /// Level-0 `motion`: `WALK` or `FLY`.
    pub motion: Option<String>,
    pub stats: Option<EnemyStats>,
    pub hide_in_handbook: bool,
    sort_id: i64,
}

/// Level-0 attributes, the numbers the handbook shows first.
#[derive(Debug, Clone, Copy)]
pub struct EnemyStats {
    pub hp: f64,
    pub atk: f64,
    pub def: f64,
    pub res: f64,
}

#[derive(Deserialize)]
struct EnemyTable {
    #[serde(rename = "enemyData")]
    enemy_data: HashMap<String, RawEnemy>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEnemy {
    enemy_id: String,
    enemy_index: Option<String>,
    name: String,
    enemy_level: Option<String>,
    description: Option<String>,
    ability_list: Option<Vec<RawAbility>>,
    damage_type: Option<Vec<String>>,
    #[serde(default)]
    hide_in_handbook: bool,
    sort_id: Option<i64>,
    stats: Option<RawEnemyStats>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAbility {
    text: String,
    text_format: Option<String>,
}

#[derive(Deserialize)]
struct RawEnemyStats {
    #[serde(default)]
    levels: Vec<RawEnemyLevel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEnemyLevel {
    attributes: Option<RawAttributes>,
    apply_way: Option<String>,
    motion: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAttributes {
    max_hp: f64,
    atk: f64,
    def: f64,
    magic_resistance: f64,
}

impl From<RawEnemy> for Enemy {
    fn from(raw: RawEnemy) -> Self {
        let level0 = raw.stats.and_then(|s| s.levels.into_iter().next());
        let (apply_way, motion, stats) = match level0 {
            Some(l) => (
                l.apply_way,
                l.motion,
                l.attributes.map(|a| EnemyStats {
                    hp: a.max_hp,
                    atk: a.atk,
                    def: a.def,
                    res: a.magic_resistance,
                }),
            ),
            None => (None, None, None),
        };
        Self {
            id: raw.enemy_id,
            index: raw.enemy_index,
            name: raw.name,
            level: raw.enemy_level,
            description: raw.description,
            abilities: raw
                .ability_list
                .unwrap_or_default()
                .into_iter()
                .map(|a| (a.text, a.text_format.unwrap_or_default()))
                .collect(),
            damage_types: raw.damage_type.unwrap_or_default(),
            apply_way,
            motion,
            stats,
            hide_in_handbook: raw.hide_in_handbook,
            sort_id: raw.sort_id.unwrap_or(i64::MAX),
        }
    }
}

/// One stage from `GET /api/static/stage-index`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub stage_id: String,
    pub code: String,
    /// Null in the index for tutorials and auto-chess boards; those are dropped on load.
    #[serde(deserialize_with = "null_as_empty")]
    pub name: String,
    pub zone_name: Option<String>,
    pub ap_cost: Option<i64>,
    /// `NORMAL`, `FOUR_STAR` (Challenge Mode) or `SIX_STAR` (Extreme).
    pub difficulty: Option<String>,
    /// Map preview, an asset path without a leading slash.
    pub preview: Option<String>,
}

impl Stage {
    /// Which variant of a code this is, for stages that share one ("1-7" is also a Challenge
    /// Mode stage). The difficulty labels are the frontend's (`stages.difficulty.*`).
    ///
    /// `isHard` alone is not a variant marker: it is also set on stages that are their own
    /// thing, like "H8-4" and "RM-H1", whose codes already say so.
    #[must_use]
    pub fn mode(&self) -> Option<&'static str> {
        if self.stage_id.starts_with("easy_") {
            return Some("Story Environment");
        }
        if self.stage_id.starts_with("tough_") {
            return Some("Adverse Environment");
        }
        match self.difficulty.as_deref() {
            Some("FOUR_STAR") => Some("Challenge Mode"),
            Some("SIX_STAR") => Some("Extreme"),
            _ => None,
        }
    }
}

/// The parts of `GET /api/stages/{id}/detail` the stage embed prints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDetail {
    pub stage: StageDetailStage,
    pub zone: Option<StageDetailZone>,
    #[serde(default)]
    pub enemies: HashMap<String, Named>,
    #[serde(default)]
    pub materials: HashMap<String, Named>,
    pub level_data: Option<StageLevelData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDetailStage {
    /// Briefing text with game markup. On a Challenge Mode, Extreme or Adverse Environment
    /// variant it carries the extra rule after a "Condition:" header.
    pub description: Option<String>,
    /// "LV.10", "Elite 2 Lv. 20", or "-" when the stage has none.
    pub danger_level: Option<String>,
    pub stage_drop_info: Option<StageDropInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDetailZone {
    pub zone_name_first: Option<String>,
    pub zone_name_second: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageDropInfo {
    #[serde(default)]
    pub display_detail_rewards: Vec<StageReward>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageReward {
    /// `NORMAL`, `SPECIAL`, `ADDITIONAL`, `ONCE` or `COMPLETE`.
    pub drop_type: String,
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageLevelData {
    /// The level file is passed through raw, so this may be null as well as missing.
    pub enemy_db_refs: Option<Vec<EnemyRef>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnemyRef {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Named {
    pub name: String,
}

/// One story group from `GET /api/story/index`, with the operator an operator record belongs to.
#[derive(Debug, Clone)]
pub struct StoryGroup {
    pub id: String,
    pub name: String,
    /// `main`, `side`, `vignette` or `record`.
    pub category: String,
    /// Asset path (leading slash) of the group's cover background.
    pub cover_url: Option<String>,
    /// Asset path of the key-visual banner; only the main and side groups have one.
    pub banner_url: Option<String>,
    pub word_count: Option<u64>,
    /// Main-story chapter label, e.g. "Episode 1".
    pub chapter: Option<String>,
    /// For an operator record, the operator's name.
    pub operator: Option<String>,
    pub stories: Vec<StoryEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StoryEntry {
    pub id: String,
}

#[derive(Deserialize)]
struct StoryIndex {
    groups: Vec<RawStoryGroup>,
    #[serde(default)]
    records: Vec<RawStoryRecord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStoryGroup {
    id: String,
    name: String,
    category: String,
    cover_url: Option<String>,
    banner_url: Option<String>,
    word_count: Option<u64>,
    zone: Option<RawStoryZone>,
    #[serde(default)]
    stories: Vec<StoryEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStoryZone {
    name_first: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStoryRecord {
    name: String,
    #[serde(default)]
    stories: Vec<RawRecordStory>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRecordStory {
    group_id: String,
}

/// A cached list: the last good value with its fetch time, when the last fetch failed, and a
/// lock that lets exactly one task fetch at a time.
struct Slot<T> {
    value: RwLock<Option<(Instant, Arc<T>)>>,
    /// Set by a failed fetch, cleared by a good one. With no value to fall back on, callers
    /// arriving within [`RETRY_AFTER`] of it fail at once instead of each waiting out a fetch.
    failed_at: RwLock<Option<Instant>>,
    refresh: Mutex<()>,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            value: RwLock::new(None),
            failed_at: RwLock::new(None),
            refresh: Mutex::new(()),
        }
    }
}

impl<T: Send + Sync + 'static> Slot<T> {
    /// The cached list, fetching it through `fetch` when there is none yet.
    ///
    /// A stale list is returned at once and refreshed in the background, so only the very first
    /// lookup waits on the network. A failed refresh keeps the stale list.
    async fn get<F, Fut>(self: &Arc<Self>, label: &'static str, fetch: F) -> Result<Arc<T>, Error>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, Error>> + Send + 'static,
    {
        let cached = self.value.read().await.clone();
        if let Some((at, value)) = cached {
            if at.elapsed() >= LIST_TTL {
                self.spawn_refresh(label, fetch);
            }
            return Ok(value);
        }

        self.backing_off(label).await?;
        let _guard = self.refresh.lock().await;
        // Whoever held the lock before us may have just filled the slot, or just failed.
        if let Some((_, value)) = self.value.read().await.as_ref() {
            return Ok(Arc::clone(value));
        }
        self.backing_off(label).await?;
        self.fetch_into(label, fetch).await
    }

    /// The cached list without ever waiting on the network, for autocomplete. `None` when
    /// nothing is cached yet; a background fetch is then started, unless one is running or the
    /// last one failed within [`RETRY_AFTER`].
    async fn peek<F, Fut>(self: &Arc<Self>, label: &'static str, fetch: F) -> Option<Arc<T>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, Error>> + Send + 'static,
    {
        let cached = self.value.read().await.clone();
        if let Some((at, value)) = cached {
            if at.elapsed() >= LIST_TTL {
                self.spawn_refresh(label, fetch);
            }
            return Some(value);
        }
        if self.backing_off(label).await.is_ok() {
            self.spawn_refresh(label, fetch);
        }
        None
    }

    /// Refresh in the background. Someone else already refreshing is fine: that refresh serves
    /// this caller too.
    fn spawn_refresh<F, Fut>(self: &Arc<Self>, label: &'static str, fetch: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, Error>> + Send + 'static,
    {
        let slot = Arc::clone(self);
        tokio::spawn(async move {
            let Ok(_guard) = slot.refresh.try_lock() else {
                return;
            };
            slot.fetch_into(label, fetch).await.ok();
        });
    }

    /// An error when the last fetch failed less than [`RETRY_AFTER`] ago.
    async fn backing_off(&self, label: &str) -> Result<(), Error> {
        let failed_at = *self.failed_at.read().await;
        match failed_at {
            Some(at) if at.elapsed() < RETRY_AFTER => Err(format!(
                "Couldn't load the {label} list from the backend. Try again in a minute."
            )
            .into()),
            _ => Ok(()),
        }
    }

    /// Run `fetch` and store the result. The caller holds `refresh`.
    async fn fetch_into<F, Fut>(&self, label: &'static str, fetch: F) -> Result<Arc<T>, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, Error>>,
    {
        match fetch().await {
            Ok(fresh) => {
                let fresh = Arc::new(fresh);
                *self.value.write().await = Some((Instant::now(), Arc::clone(&fresh)));
                *self.failed_at.write().await = None;
                tracing::debug!("gamedata: refreshed {label}");
                Ok(fresh)
            }
            Err(e) => {
                tracing::warn!("gamedata: fetching {label} failed: {e}");
                *self.failed_at.write().await = Some(Instant::now());
                // Keep serving the stale list, but push the next attempt back by RETRY_AFTER.
                if let Some((at, _)) = self.value.write().await.as_mut()
                    && let Some(backdated) =
                        Instant::now().checked_sub(LIST_TTL.saturating_sub(RETRY_AFTER))
                {
                    *at = backdated;
                }
                Err(e)
            }
        }
    }

    async fn is_loaded(&self) -> bool {
        self.value.read().await.is_some()
    }
}

/// The game-data cache shared through `Data`, and by the birthday announcer.
pub struct GameData {
    client: Client,
    /// `endpoints.public_backend`, without a trailing slash. Empty when unconfigured.
    base: String,
    operators: Arc<Slot<Vec<Operator>>>,
    enemies: Arc<Slot<Vec<Enemy>>>,
    stages: Arc<Slot<Vec<Stage>>>,
    stories: Arc<Slot<Vec<StoryGroup>>>,
    /// Item id to display name, from `GET /api/static/materials`. Only the operator Costs page
    /// reads it (promotion and skill costs carry ids, not names).
    materials: Arc<Slot<HashMap<String, String>>>,
    /// Attack ranges by id, from `GET /api/static/ranges`: every Overview draws one.
    ranges: Arc<Slot<HashMap<String, Range>>>,
    /// Paradox Simulations by operator id, from `handbookStageData` in
    /// `GET /api/static/handbook`. Every operator view reads it to know whether to offer the page.
    paradox: Arc<Slot<HashMap<String, ParadoxEntry>>>,
    /// The whole skill table, from `GET /api/static/skills`. Only the Summons page reads it:
    /// a summon's skills arrive as bare ids.
    skill_table: Arc<Slot<HashMap<String, TableSkill>>>,
    /// Whether each operator has voice lines, learned from its first `/api/voices/{id}` fetch.
    /// 25 of 441 have none (Amiya's other forms, the reserve and collab placeholders), and the
    /// Voice page is offered only where there is something to show.
    voice_presence: RwLock<HashMap<String, (Instant, bool)>>,
    /// The last [`VOICE_CACHE_MAX`] operators' voice lines, for `/voiceline`: its line and
    /// language autocompletes read them on every keystroke.
    voice_cache: RwLock<HashMap<String, (Instant, Arc<VoiceData>)>>,
}

impl GameData {
    #[must_use]
    pub fn new(client: Client, public_backend: &str) -> Self {
        Self {
            client,
            base: public_backend.trim_end_matches('/').to_string(),
            operators: Arc::default(),
            enemies: Arc::default(),
            stages: Arc::default(),
            stories: Arc::default(),
            materials: Arc::default(),
            ranges: Arc::default(),
            paradox: Arc::default(),
            skill_table: Arc::default(),
            voice_presence: RwLock::default(),
            voice_cache: RwLock::default(),
        }
    }

    /// The backend base URL, or an error a command can show as-is.
    pub fn base(&self) -> Result<&str, Error> {
        if self.base.is_empty() {
            return Err("No public backend is configured (`endpoints.public_backend`).".into());
        }
        Ok(&self.base)
    }

    /// A URL under the backend's `/api`, e.g. `api_url("/avatar/char_002_amiya")`.
    #[must_use]
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/api{path}", self.base)
    }

    /// The URL of an asset-index path (`/textures/...`), the way the frontend serves them.
    #[must_use]
    pub fn asset_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        let encoded: Vec<String> = path.split('/').map(encode_path_segment).collect();
        format!("{}/api/assets/{}", self.base, encoded.join("/"))
    }

    /// The `(client, url)` a list fetch needs, or the no-backend error.
    fn request(&self, path: &str) -> Result<(Client, String), Error> {
        Ok((self.client.clone(), format!("{}{path}", self.base()?)))
    }

    /// The list at `path`, from `slot` or fetched through `load`.
    async fn list<T, F, Fut>(
        &self,
        slot: &Arc<Slot<T>>,
        path: &str,
        label: &'static str,
        load: F,
    ) -> Result<Arc<T>, Error>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Client, String) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, Error>> + Send + 'static,
    {
        let (client, url) = self.request(path)?;
        slot.get(label, move || load(client, url)).await
    }

    /// The list at `path` if it is cached, for autocomplete: never waits on the network.
    async fn cached_list<T, F, Fut>(
        &self,
        slot: &Arc<Slot<T>>,
        path: &str,
        label: &'static str,
        load: F,
    ) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Client, String) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, Error>> + Send + 'static,
    {
        let (client, url) = self.request(path).ok()?;
        slot.peek(label, move || load(client, url)).await
    }

    /// `/api/<route>/<id><suffix>`, fetched on demand and not cached.
    async fn fetch_one<T: DeserializeOwned>(
        &self,
        route: &str,
        id: &str,
        suffix: &str,
    ) -> Result<T, Error> {
        let url = format!(
            "{}/api/{route}/{}{suffix}",
            self.base()?,
            encode_path_segment(id)
        );
        fetch_json(self.client.clone(), url).await
    }

    pub async fn operators(&self) -> Result<Arc<Vec<Operator>>, Error> {
        self.list(&self.operators, OPERATORS_PATH, "operator", fetch_json)
            .await
    }

    pub async fn enemies(&self) -> Result<Arc<Vec<Enemy>>, Error> {
        self.list(&self.enemies, ENEMIES_PATH, "enemy", load_enemies)
            .await
    }

    pub async fn stages(&self) -> Result<Arc<Vec<Stage>>, Error> {
        self.list(&self.stages, STAGES_PATH, "stage", load_stages)
            .await
    }

    pub async fn stories(&self) -> Result<Arc<Vec<StoryGroup>>, Error> {
        self.list(&self.stories, STORIES_PATH, "story", load_stories)
            .await
    }

    /// Item names by id. Not part of [`GameData::warm`]: only the operator Costs and Paradox
    /// Simulation pages need it.
    pub async fn materials(&self) -> Result<Arc<HashMap<String, String>>, Error> {
        self.list(&self.materials, MATERIALS_PATH, "item", load_materials)
            .await
    }

    /// Attack ranges by id.
    pub async fn ranges(&self) -> Result<Arc<HashMap<String, Range>>, Error> {
        self.list(&self.ranges, RANGES_PATH, "range", fetch_json)
            .await
    }

    /// Paradox Simulations by operator id.
    pub async fn paradox(&self) -> Result<Arc<HashMap<String, ParadoxEntry>>, Error> {
        self.list(
            &self.paradox,
            HANDBOOK_PATH,
            "Paradox Simulation",
            load_paradox,
        )
        .await
    }

    /// Every skill by id. Not part of [`GameData::warm`]: only the Summons page needs it.
    pub async fn skill_table(&self) -> Result<Arc<HashMap<String, TableSkill>>, Error> {
        self.list(&self.skill_table, SKILLS_PATH, "skill", fetch_json)
            .await
    }

    /// One operator's full record, fetched on demand and not cached.
    pub async fn operator_detail(&self, id: &str) -> Result<OperatorDetail, Error> {
        self.fetch_one("operators", id, "").await
    }

    /// One operator's outfits, fetched on demand and not cached.
    pub async fn skins(&self, id: &str) -> Result<SkinData, Error> {
        self.fetch_one("skins", id, "").await
    }

    /// One operator's voice lines, fetched on demand and not cached. Remembers whether there
    /// were any, for [`GameData::known_voice_presence`].
    pub async fn voices(&self, id: &str) -> Result<VoiceData, Error> {
        let data: VoiceData = self.fetch_one("voices", id, "").await?;
        self.voice_presence.write().await.insert(
            id.to_string(),
            (Instant::now(), !data.char_words.is_empty()),
        );
        Ok(data)
    }

    /// One operator's voice lines, cached for [`LIST_TTL`] across the last [`VOICE_CACHE_MAX`]
    /// operators asked for.
    pub async fn voice_lines(&self, id: &str) -> Result<Arc<VoiceData>, Error> {
        if let Some((at, data)) = self.voice_cache.read().await.get(id)
            && at.elapsed() < LIST_TTL
        {
            return Ok(Arc::clone(data));
        }
        let data = Arc::new(self.voices(id).await?);
        let mut cache = self.voice_cache.write().await;
        cache.insert(id.to_string(), (Instant::now(), Arc::clone(&data)));
        while cache.len() > VOICE_CACHE_MAX {
            let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, (at, _))| *at)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            cache.remove(&oldest);
        }
        drop(cache);
        Ok(data)
    }

    /// The audio of one voice-line recording, by its `voiceUrl`. Refuses anything over
    /// [`CLIP_MAX_BYTES`] or not Ogg.
    pub async fn voice_clip(&self, voice_url: &str) -> Result<Vec<u8>, Error> {
        let url = clip_url(self.base()?, voice_url);
        let response = self
            .client
            .get(&url)
            .timeout(CLIP_TIMEOUT)
            .send()
            .await
            .map_err(|e| format!("GET {url}: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("GET {url}: HTTP {status}").into());
        }
        if response
            .content_length()
            .is_some_and(|n| n > CLIP_MAX_BYTES as u64)
        {
            return Err(format!("GET {url}: over {CLIP_MAX_BYTES} B").into());
        }
        let body = response
            .bytes()
            .await
            .map_err(|e| format!("GET {url}: body: {e}"))?;
        if body.len() > CLIP_MAX_BYTES || !body.starts_with(b"OggS") {
            return Err(format!("GET {url}: not an Ogg file ({} B)", body.len()).into());
        }
        Ok(body.to_vec())
    }

    /// Whether the operator has voice lines, if a fetch in the last [`LIST_TTL`] said.
    pub async fn known_voice_presence(&self, id: &str) -> Option<bool> {
        self.voice_presence
            .read()
            .await
            .get(id)
            .filter(|(at, _)| at.elapsed() < LIST_TTL)
            .map(|(_, has)| *has)
    }

    /// A Paradox Simulation stage's enemies and waves, fetched on demand and not cached.
    pub async fn paradox_stage(&self, stage_id: &str) -> Result<ParadoxStage, Error> {
        self.fetch_one("stages", stage_id, "/detail").await
    }

    /// One stage's detail, fetched on demand and not cached: it is only read when an embed for
    /// that stage is being built.
    pub async fn stage_detail(&self, stage_id: &str) -> Result<StageDetail, Error> {
        self.fetch_one("stages", stage_id, "/detail").await
    }

    /// The operator list if it is cached, for autocomplete: never waits on the network.
    pub async fn cached_operators(&self) -> Option<Arc<Vec<Operator>>> {
        self.cached_list(&self.operators, OPERATORS_PATH, "operator", fetch_json)
            .await
    }

    /// The enemy list if it is cached, for autocomplete: never waits on the network.
    pub async fn cached_enemies(&self) -> Option<Arc<Vec<Enemy>>> {
        self.cached_list(&self.enemies, ENEMIES_PATH, "enemy", load_enemies)
            .await
    }

    /// The stage list if it is cached, for autocomplete: never waits on the network.
    pub async fn cached_stages(&self) -> Option<Arc<Vec<Stage>>> {
        self.cached_list(&self.stages, STAGES_PATH, "stage", load_stages)
            .await
    }

    /// The story list if it is cached, for autocomplete: never waits on the network.
    pub async fn cached_stories(&self) -> Option<Arc<Vec<StoryGroup>>> {
        self.cached_list(&self.stories, STORIES_PATH, "story", load_stories)
            .await
    }

    /// Whether every list has been fetched at least once, so a lookup won't wait on the network.
    pub async fn is_warm(&self) -> bool {
        self.operators.is_loaded().await
            && self.enemies.is_loaded().await
            && self.stages.is_loaded().await
            && self.stories.is_loaded().await
    }

    /// Fetch every list once, so the first lookups after startup answer from memory.
    pub async fn warm(&self) {
        if self.base.is_empty() {
            return;
        }
        let (ops, enemies, stages, stories, ranges, paradox) = tokio::join!(
            self.operators(),
            self.enemies(),
            self.stages(),
            self.stories(),
            self.ranges(),
            self.paradox()
        );
        let failed = [
            ops.is_err(),
            enemies.is_err(),
            stages.is_err(),
            stories.is_err(),
            ranges.is_err(),
            paradox.is_err(),
        ]
        .iter()
        .filter(|f| **f)
        .count();
        if failed == 0 {
            tracing::info!("gamedata: all lists cached");
        } else {
            tracing::warn!("gamedata: {failed} list(s) failed to load at startup; retrying on use");
        }
    }
}

/// Operators whose voice lines [`GameData::voice_lines`] keeps. One response parses to a few
/// hundred KB at most (Ling: 114 lines in three sets, 124 KB of JSON).
const VOICE_CACHE_MAX: usize = 32;
/// The largest voice clip fetched: under Discord's 10 MiB upload limit, and far over any line
/// (Amiya's idle line is 43 to 59 KB in each language).
pub const CLIP_MAX_BYTES: usize = 8 * 1024 * 1024;
const CLIP_TIMEOUT: Duration = Duration::from_secs(15);

/// A recording's URL. `voice_url` is already percent-encoded (`nian%2312`), so it is joined
/// as it is, never encoded again.
fn clip_url(base: &str, voice_url: &str) -> String {
    let path = voice_url.trim_start_matches('/');
    format!("{base}/api/assets/audio/{path}")
}

const OPERATORS_PATH: &str = "/api/operators/index";
const ENEMIES_PATH: &str = "/api/static/enemies";
const STAGES_PATH: &str = "/api/static/stage-index";
const STORIES_PATH: &str = "/api/story/index";
const MATERIALS_PATH: &str = "/api/static/materials";
const RANGES_PATH: &str = "/api/static/ranges";
const HANDBOOK_PATH: &str = "/api/static/handbook";
const SKILLS_PATH: &str = "/api/static/skills";

/// Only `handbookStageData` is decoded; serde skips the rest of the 4.7 MB table.
#[derive(Deserialize)]
struct HandbookTable {
    #[serde(rename = "handbookStageData", default)]
    stage_data: HashMap<String, ParadoxEntry>,
}

async fn load_paradox(client: Client, url: String) -> Result<HashMap<String, ParadoxEntry>, Error> {
    let table: HandbookTable = fetch_json(client, url).await?;
    Ok(table.stage_data)
}

#[derive(Deserialize)]
struct MaterialsTable {
    #[serde(default)]
    items: HashMap<String, MaterialItem>,
}

#[derive(Deserialize)]
struct MaterialItem {
    name: Option<String>,
}

async fn load_materials(client: Client, url: String) -> Result<HashMap<String, String>, Error> {
    let table: MaterialsTable = fetch_json(client, url).await?;
    Ok(table
        .items
        .into_iter()
        .filter_map(|(id, item)| item.name.map(|name| (id, name)))
        .collect())
}

async fn load_enemies(client: Client, url: String) -> Result<Vec<Enemy>, Error> {
    let table: EnemyTable = fetch_json(client, url).await?;
    let mut enemies: Vec<Enemy> = table.enemy_data.into_values().map(Enemy::from).collect();
    enemies.sort_by(|a, b| a.sort_id.cmp(&b.sort_id).then(a.id.cmp(&b.id)));
    Ok(enemies)
}

async fn load_stages(client: Client, url: String) -> Result<Vec<Stage>, Error> {
    let mut stages: Vec<Stage> = fetch_json(client, url).await?;
    stages.retain(|s| !s.name.is_empty());
    Ok(stages)
}

async fn load_stories(client: Client, url: String) -> Result<Vec<StoryGroup>, Error> {
    let index: StoryIndex = fetch_json(client, url).await?;
    let owners: HashMap<String, String> = index
        .records
        .into_iter()
        .flat_map(|r| {
            r.stories
                .into_iter()
                .map(move |s| (s.group_id, r.name.clone()))
        })
        .collect();
    Ok(index
        .groups
        .into_iter()
        .map(|g| StoryGroup {
            operator: owners.get(&g.id).cloned(),
            id: g.id,
            name: g.name,
            category: g.category,
            cover_url: g.cover_url,
            banner_url: g.banner_url,
            word_count: g.word_count,
            chapter: g.zone.and_then(|z| z.name_first),
            stories: g.stories,
        })
        .collect())
}

async fn fetch_json<T: DeserializeOwned>(client: Client, url: String) -> Result<T, Error> {
    let response = client
        .get(&url)
        .timeout(LIST_TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("GET {url}: {e}"))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("GET {url}: HTTP {status}").into());
    }
    Ok(response
        .json::<T>()
        .await
        .map_err(|e| format!("GET {url}: undecodable body: {e}"))?)
}

fn null_as_empty<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Option::<String>::deserialize(d).map(Option::unwrap_or_default)
}

/// Percent-encode one URL path segment. Stage ids carry `#` ("main_01-07#f#"), which would
/// otherwise start a fragment.
#[must_use]
pub fn encode_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_stage_ids_and_asset_paths() {
        assert_eq!(encode_path_segment("main_01-07#f#"), "main_01-07%23f%23");
        let data = GameData::new(Client::new(), "https://api.example.com/");
        assert_eq!(
            data.asset_url("/textures/avg/bg/bg indoor.png"),
            "https://api.example.com/api/assets/textures/avg/bg/bg%20indoor.png"
        );
    }

    /// The URL `/api/voices` gives is joined verbatim: the live outfit path is already
    /// encoded, and the live fetch of this exact shape answered 200 `audio/ogg`.
    #[test]
    fn clip_urls_keep_the_backends_encoding() {
        assert_eq!(
            clip_url(
                "https://api.myrtle.moe",
                "/audio/sound_beta_2/voice_en/char_2023_ling_nian%2312/CN_001.ogg"
            ),
            "https://api.myrtle.moe/api/assets/audio/audio/sound_beta_2/voice_en/\
             char_2023_ling_nian%2312/CN_001.ogg"
        );
    }

    #[test]
    fn index_fields_decode_from_null() {
        // Every field the backend types as Option, sent as null. Any one of them failing to
        // decode would fail the whole operator index.
        let ops: Vec<Operator> = serde_json::from_str(
            r#"[{"id":"char_x","name":"X","appellation":" ","rarity":4,
                "profession":"CASTER","position":"RANGED","tagList":[],
                "professionName":null,"subProfessionName":null,"nationName":null,
                "groupName":null,"teamName":null,"artists":[],"voiceActors":[],
                "dateOfBirth":"","isNotObtainable":false}]"#,
        )
        .unwrap();
        assert_eq!(ops[0].class(), None);
        assert_eq!(ops[0].branch(), None);
        assert!(ops[0].factions().is_empty());

        let stages: Vec<Stage> = serde_json::from_str(
            r#"[{"stageId":"guide_01","code":"TR-1","name":null,"zoneName":null,
                "apCost":0,"difficulty":"NORMAL","preview":null}]"#,
        )
        .unwrap();
        assert!(stages[0].name.is_empty());

        let level: StageLevelData = serde_json::from_str(r#"{"enemyDbRefs":null}"#).unwrap();
        assert!(level.enemy_db_refs.is_none());

        let enemy: RawEnemy = serde_json::from_str(
            r#"{"enemyId":"e","enemyIndex":null,"name":"E","enemyLevel":"NORMAL",
                "description":null,"abilityList":null,"damageType":null,"sortId":null,
                "stats":null}"#,
        )
        .unwrap();
        let enemy = Enemy::from(enemy);
        assert!(enemy.stats.is_none() && enemy.abilities.is_empty());
    }

    #[tokio::test]
    async fn failed_cold_fetch_backs_off_and_peek_never_waits() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let calls = Arc::new(AtomicUsize::new(0));
        let slot: Arc<Slot<Vec<u8>>> = Arc::default();
        let failing = |calls: Arc<AtomicUsize>| {
            move || async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err::<Vec<u8>, Error>("down".into())
            }
        };

        assert!(slot.get("test", failing(calls.clone())).await.is_err());
        // Within RETRY_AFTER of the failure: fail at once, without another fetch.
        assert!(slot.get("test", failing(calls.clone())).await.is_err());
        assert!(slot.peek("test", failing(calls.clone())).await.is_none());
        tokio::task::yield_now().await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // Out of backoff, peek still answers at once and fetches in the background.
        *slot.failed_at.write().await = None;
        let ok = || async { Ok::<Vec<u8>, Error>(vec![7]) };
        assert!(slot.peek("test", ok).await.is_none());
        for _ in 0..100 {
            if slot.is_loaded().await {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(slot.peek("test", ok).await.as_deref(), Some(&vec![7_u8]));
    }

    #[test]
    fn stage_modes() {
        let stage = |id: &str, difficulty: &str| Stage {
            stage_id: id.to_string(),
            code: "1-7".to_string(),
            name: "The Tyrant".to_string(),
            zone_name: None,
            ap_cost: Some(6),
            difficulty: Some(difficulty.to_string()),
            preview: None,
        };
        assert_eq!(stage("main_01-07", "NORMAL").mode(), None);
        assert_eq!(stage("hard_08-04", "NORMAL").mode(), None);
        assert_eq!(
            stage("main_01-07#f#", "FOUR_STAR").mode(),
            Some("Challenge Mode")
        );
        assert_eq!(stage("main_16-04#s", "SIX_STAR").mode(), Some("Extreme"));
        assert_eq!(
            stage("tough_10-12", "NORMAL").mode(),
            Some("Adverse Environment")
        );
        assert_eq!(
            stage("easy_10-12", "NORMAL").mode(),
            Some("Story Environment")
        );
    }
}
