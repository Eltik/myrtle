//! The character gallery: every sprite folder under `textures/avg/characters/`,
//! the names the scripts speak it under, and every expression it offers.
//!
//! The census is two walks:
//! - SCRIPTS, once per distinct library script in library order:
//!   [`crate::core::story::speakers`] attributes each named line to the sprite(s)
//!   LIT when spoken (the reader's speaker-colour rule) and counts every put-up.
//!   Source of `names`, `lines`, `stories`, `firstSeen` and each expression's `uses`.
//! - FOLDERS, once each: every `#N$M` the hub offers, resolved by
//!   [`StoryAssetIndex::resolve_character`] so gallery and reader never draw a key
//!   differently, deduplicated by the files it lands on.
//!
//! Built under `cpu::run` and cached whole per server in a [`ServerCache`], retired
//! by a new `GameData` or asset tree. List and detail are separate routes because
//! all details at once are several times the list (`docs/story-reader.md`,
//! "Character gallery").

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Weak};
use std::time::Instant;

use super::cache::{ServerCache, StoryTrees, cached_index};
use super::dto::{
    StorySpriteDetail, StorySpriteEntry, StorySpriteExample, StorySpriteIndex, StorySpriteKind,
    StorySpriteName, StorySpriteNameDetail, StorySpriteNameStory, StorySpriteStory,
    StorySpriteVariant,
};
use super::index::StoryIndexCache;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::core::story::assets::{parse_sprite_name, served_path, trim_index};
use crate::core::story::speakers::{PLACING_COMMANDS, attribute_speakers, is_meaningful_name};
use crate::core::story::{StoryAssetIndex, load_script, parser};

/// The built gallery, cached per server.
#[derive(Debug)]
pub struct StorySpriteCache {
    pub index: StorySpriteIndex,
    /// Lowercased folder -> its sheet.
    pub details: HashMap<String, StorySpriteDetail>,
    /// The census counts the doc quotes, kept so a test can print them.
    pub census: SpriteCensus,
    pub build_ms: u128,
    /// The asset tree the build resolved against; a re-extract retires it.
    assets: Weak<AssetIndex>,
}

/// What the script walk saw, in its own units.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpriteCensus {
    pub scripts: u32,
    /// Lines with a non-empty `[name=...]`.
    pub named_lines: u32,
    /// Of those, the ones whose name is a mask (`???`) and is never kept.
    pub masked_lines: u32,
    /// Of the kept ones, the ones no sprite was lit for.
    pub unattributed_lines: u32,
    /// Of the attributed ones, the ones split over two or more sprites.
    pub split_lines: u32,
    /// Folders left out of the gallery: no expression resolves, or every
    /// one it offers is a blank plate.
    pub hidden: Vec<String>,
    /// Names cut by the alias threshold ([`keep_aliases`]), over every folder.
    pub dropped_names: u32,
    /// Their weighted lines, summed.
    pub dropped_lines: f64,
}

/// A name is an ALIAS of a folder only with at least this many weighted lines…
pub const ALIAS_MIN_LINES: f64 = 3.0;
/// …and at least this share of the folder's named lines.
pub const ALIAS_MIN_SHARE: f64 = 0.02;

/// Cut the lit-slot rule's misattributions out of a folder's names (sorted,
/// most lines first): someone else speaking while this sprite stays lit
/// leaves a trickle of foreign names (Kal'tsit's 3,102 lines carried "Wei
/// Yenwu" 3, "Taciturn Passerby" 2, "Mon3tr" 0.5). A name stays when it has
/// [`ALIAS_MIN_LINES`] AND [`ALIAS_MIN_SHARE`] of `total`; the PRIMARY always
/// stays, so a folder with two lines keeps its name. Returns the kept names
/// and the cut ones (the STRAY names), both still most lines first.
#[must_use]
pub fn keep_aliases(
    names: Vec<StorySpriteName>,
    total: f64,
) -> (Vec<StorySpriteName>, Vec<StorySpriteName>) {
    let mut kept = Vec::with_capacity(names.len());
    let mut stray = Vec::new();
    for (i, name) in names.into_iter().enumerate() {
        if i == 0 || (name.count >= ALIAS_MIN_LINES && name.count >= ALIAS_MIN_SHARE * total) {
            kept.push(name);
        } else {
            stray.push(name);
        }
    }
    (kept, stray)
}

/// A plate with no art: a PNG whose every pixel is fully transparent.
/// `char_empty` is one (24 x 24, alpha 0 everywhere), the invisible occupant
/// 410 stories put in a slot. Only a file under the cap is decoded: on EN the
/// ONLY non-patch PNG under 16 KiB in `textures/avg/characters` is
/// `char_empty.png` (192 bytes), so the 12,107 bodies are not decoded to find
/// the one that is empty.
fn is_blank_plate(path: &Path) -> bool {
    const DECODE_CAP: u64 = 16 * 1024;
    if std::fs::metadata(path).map_or(true, |m| m.len() > DECODE_CAP) {
        return false;
    }
    image::open(path).is_ok_and(|img| img.to_rgba8().pixels().all(|p| p[3] == 0))
}

/// Example lines per name on the sheet.
const EXAMPLES: usize = 3;
/// Stories listed per name on the sheet.
const NAME_STORIES: usize = 5;

/// One folder's script-side tallies while the walk runs.
#[derive(Debug, Default)]
struct Tally {
    /// Display name -> weighted lines.
    names: HashMap<String, f64>,
    /// Per name: where it is spoken and the example candidates.
    name_detail: HashMap<String, NameTally>,
    lines: f64,
    /// Story order -> what the folder does in that story.
    stories: BTreeMap<u32, StoryTally>,
    /// Raw sprite name as written -> times put up.
    uses: HashMap<String, u32>,
    first_seen: Option<i64>,
}

/// One folder in one story.
#[derive(Debug)]
struct StoryTally {
    id: String,
    /// Weighted named lines attributed to the folder.
    lines: f64,
    /// Times the story puts the folder up.
    uses: u32,
}

/// One (folder, name) pair's whereabouts while the walk runs.
#[derive(Debug, Default)]
struct NameTally {
    /// Story order -> (story id, weighted lines).
    stories: BTreeMap<u32, (String, f64)>,
    /// Up to [`EXAMPLES`] lines from each of the first [`EXAMPLES`] stories,
    /// in walk order.
    candidates: Vec<ExampleLine>,
}

/// A line that may be shown as an example of a name.
#[derive(Debug)]
struct ExampleLine {
    story_order: u32,
    story_id: String,
    line: u32,
    text: String,
}

impl NameTally {
    fn add(&mut self, order: u32, story_id: &str, line: u32, text: &str, w: f64) {
        let stories_seen = self.stories.len();
        self.stories
            .entry(order)
            .or_insert_with(|| (story_id.to_owned(), 0.0))
            .1 += w;
        // Bounded by EXAMPLES squared, so the scan per line is constant.
        let in_story = self
            .candidates
            .iter()
            .filter(|c| c.story_order == order)
            .count();
        let new_story = in_story == 0;
        if in_story < EXAMPLES && (!new_story || stories_seen < EXAMPLES) && !text.trim().is_empty()
        {
            self.candidates.push(ExampleLine {
                story_order: order,
                story_id: story_id.to_owned(),
                line,
                text: text.to_owned(),
            });
        }
    }

    /// The examples, SPREAD across stories: the first line of each of the
    /// first stories, then second lines, until [`EXAMPLES`] are taken.
    fn examples(&self) -> Vec<&ExampleLine> {
        let mut orders: Vec<u32> = self.candidates.iter().map(|c| c.story_order).collect();
        orders.dedup();
        let mut out = Vec::new();
        for round in 0..EXAMPLES {
            for &order in &orders {
                if out.len() == EXAMPLES {
                    return out;
                }
                if let Some(c) = self
                    .candidates
                    .iter()
                    .filter(|c| c.story_order == order)
                    .nth(round)
                {
                    out.push(c);
                }
            }
        }
        out
    }
}

/// What the sheet says about one story beside a name or an example.
#[derive(Debug, Clone, Default)]
struct StoryMeta {
    name: String,
    code: Option<String>,
    tag: Option<String>,
    group_id: String,
    group_name: String,
}

/// Story id -> its [`StoryMeta`], for every script the walk read.
type StoryMetas = HashMap<String, StoryMeta>;

/// A story's meta, empty for an id the walk never read.
fn meta_of(metas: &StoryMetas, id: &str) -> StoryMeta {
    metas.get(id).cloned().unwrap_or_default()
}

/// The folder's tally, with this story registered on it.
fn touch<'a>(
    tallies: &'a mut HashMap<String, Tally>,
    folder: &str,
    dated: Option<i64>,
    order: u32,
    story_id: &str,
) -> &'a mut Tally {
    let t = tallies.entry(folder.to_owned()).or_default();
    if let Some(d) = dated {
        t.first_seen = Some(t.first_seen.map_or(d, |f| f.min(d)));
    }
    t.stories.entry(order).or_insert_with(|| StoryTally {
        id: story_id.to_owned(),
        lines: 0.0,
        uses: 0,
    });
    t
}

/// Round a weighted count for the wire: two decimals carry every split the
/// corpus makes (halves and thirds) without a float tail.
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// A collection length as a wire count.
fn count_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The character table's operators by the NUMBER in their id: `002` ->
/// `char_002_amiya`. The number is the join, not the stem, because a story
/// set can spell the stem differently from the table: Gladiia's sets are
/// `avg_474_gladiia_*` and her id is `char_474_glady`.
#[derive(Debug, Default)]
pub struct OperatorIds<'a> {
    by_number: HashMap<&'a str, Vec<(&'a str, &'a str)>>,
}

impl<'a> OperatorIds<'a> {
    #[must_use]
    pub fn new(gd: &'a GameData) -> Self {
        let mut by_number: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
        for (id, op) in &gd.operators {
            let Some(rest) = id.strip_prefix("char_") else {
                continue;
            };
            let Some((num, _)) = rest.split_once('_') else {
                continue;
            };
            if num.bytes().all(|b| b.is_ascii_digit()) {
                by_number
                    .entry(num)
                    .or_default()
                    .push((id.as_str(), op.name.as_str()));
            }
        }
        for ids in by_number.values_mut() {
            ids.sort_unstable();
        }
        Self { by_number }
    }

    /// The operator a folder draws, and what the folder adds after the id.
    ///
    /// `char_002_amiya_1` -> (`char_002_amiya`, Amiya, `1`);
    /// `avg_474_gladiia_1` -> (`char_474_glady`, Gladiia, `1`);
    /// `char_130_doberm_ex` -> (`char_130_doberm`, Dobermann, `ex`). The
    /// prefixes are the ones the tier list's story-sprite kind reads
    /// (`avgnew_`, `avg_char_`, `avg_`, `char_`). When two ids share a number
    /// the one whose stem the folder spells wins, and with no such id the
    /// number names nobody.
    #[must_use]
    pub fn operator_for(&self, folder: &str) -> Option<(String, String, Option<String>)> {
        let lower = folder.to_ascii_lowercase();
        let rest = ["avgnew_", "avg_char_", "avg_", "char_"]
            .iter()
            .find_map(|p| lower.strip_prefix(p))?;
        let parts: Vec<&str> = rest.split('_').collect();
        let num = *parts.first()?;
        if num.is_empty() || !num.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let ids = self.by_number.get(num)?;
        let stem = parts.get(1).copied().unwrap_or_default();
        let (id, name) = ids
            .iter()
            .find(|(id, _)| id.rsplit_once('_').is_some_and(|(_, s)| s == stem))
            .or_else(|| (ids.len() == 1).then(|| &ids[0]))?;
        let tail = parts.get(2..).map(|t| t.join("_")).unwrap_or_default();
        Some((
            (*id).to_owned(),
            (*name).to_owned(),
            (!tail.is_empty()).then_some(tail),
        ))
    }
}

/// A raw script name -> the `#N$M` key it addresses, read the way the
/// resolver reads it (a missing index is 1, leading zeros cut).
fn key_of(raw: &str) -> String {
    let p = parse_sprite_name(raw);
    if let Some(alias) = p.alias {
        return format!("@{alias}");
    }
    format!(
        "#{}${}",
        trim_index(p.face.unwrap_or("1")),
        trim_index(p.body.unwrap_or("1"))
    )
}

/// Every expression `folder` offers, in hub order, deduplicated by the files
/// each key resolves to, plus any expression a script reaches that the hub
/// enumeration did not (a file-name rule pick), so a USED expression is never
/// missing from its sheet.
fn variants_of(
    assets: &StoryAssetIndex,
    folder: &str,
    uses: &HashMap<String, u32>,
) -> Vec<StorySpriteVariant> {
    let mut out: Vec<StorySpriteVariant> = Vec::new();
    // (body url, face url) -> its index in `out`.
    let mut at: HashMap<(String, Option<String>), usize> = HashMap::new();
    let mut add = |out: &mut Vec<StorySpriteVariant>, key: String, uses: u32| {
        let Some(sprite) = assets.resolve_character(&format!("{folder}{key}")) else {
            return;
        };
        let files = (sprite.body_url.clone(), sprite.face_url.clone());
        if let Some(&i) = at.get(&files) {
            out[i].uses += uses;
            return;
        }
        at.insert(files, out.len());
        out.push(StorySpriteVariant {
            key,
            whole_body: sprite.face_url.is_none(),
            sprite,
            uses,
        });
    };
    for key in assets.sprite_variant_keys(folder) {
        add(&mut out, key, 0);
    }
    // In a fixed order so the key an extra expression is listed under does
    // not depend on hash order.
    let mut used: Vec<(&String, &u32)> = uses.iter().collect();
    used.sort();
    for (raw, n) in used {
        add(&mut out, key_of(raw), *n);
    }
    out
}

/// What the script walk leaves for the folder pass.
#[derive(Debug, Default)]
struct ScriptTallies {
    /// Folder as the resolver spells it -> its tally.
    folders: HashMap<String, Tally>,
    stories: StoryMetas,
}

/// Walk every distinct library script once, in library order, and tally
/// every named line and every placement onto its folder.
fn walk_scripts(
    library: &StoryIndexCache,
    assets: &StoryAssetIndex,
    assets_dir: &Path,
    census: &mut SpriteCensus,
) -> ScriptTallies {
    let mut out = ScriptTallies::default();
    let mut seen_txt: HashSet<&str> = HashSet::new();
    let mut order: u32 = 0;
    // A raw name -> its folder, memoised: the corpus writes the same few
    // thousand names hundreds of thousands of times.
    let mut folder_memo: HashMap<String, Option<String>> = HashMap::new();

    for group in &library.index.groups {
        let dated = (group.start_time > 0).then_some(group.start_time);
        for story in &group.stories {
            if !story.has_script {
                continue;
            }
            let Some(sref) = library.lookup.get(&story.id) else {
                continue;
            };
            if !seen_txt.insert(sref.story_txt.as_str()) {
                continue;
            }
            let Ok(text) = load_script(assets_dir, &sref.story_txt) else {
                continue;
            };
            census.scripts += 1;
            order += 1;
            out.stories.insert(
                story.id.clone(),
                StoryMeta {
                    name: story.name.clone(),
                    code: story.code.clone(),
                    tag: story.avg_tag.clone(),
                    group_id: group.id.clone(),
                    group_name: group.name.clone(),
                },
            );
            let commands = parser::parse(&text);
            resolve_sprite_names(&commands, assets, &mut folder_memo);
            let walk = attribute_speakers(&commands, |raw| {
                folder_memo.get(raw.trim()).cloned().flatten()
            });

            for (folder, raw) in &walk.placements {
                let t = touch(&mut out.folders, folder, dated, order, &story.id);
                *t.uses.entry(raw.clone()).or_default() += 1;
                if let Some(s) = t.stories.get_mut(&order) {
                    s.uses += 1;
                }
            }
            for line in &walk.lines {
                census.named_lines += 1;
                if !is_meaningful_name(&line.speaker) {
                    census.masked_lines += 1;
                    continue;
                }
                if line.sprites.is_empty() {
                    census.unattributed_lines += 1;
                    continue;
                }
                if line.sprites.len() > 1 {
                    census.split_lines += 1;
                }
                for (folder, w) in &line.sprites {
                    let t = touch(&mut out.folders, folder, dated, order, &story.id);
                    *t.names.entry(line.speaker.clone()).or_default() += w;
                    t.name_detail
                        .entry(line.speaker.clone())
                        .or_default()
                        .add(order, &story.id, line.line, &line.text, *w);
                    t.lines += w;
                    if let Some(s) = t.stories.get_mut(&order) {
                        s.lines += w;
                    }
                }
            }
        }
    }
    out
}

/// Resolve every sprite name `commands` can put up into `memo`, once per
/// distinct name, so the walk's resolver is a pure lookup that borrows
/// nothing mutably.
fn resolve_sprite_names(
    commands: &[parser::StoryCommand],
    assets: &StoryAssetIndex,
    memo: &mut HashMap<String, Option<String>>,
) {
    let names = commands
        .iter()
        .filter(|c| PLACING_COMMANDS.contains(&c.kind.as_str()))
        .flat_map(|c| &c.args)
        .filter(|(k, _)| k.starts_with("name"))
        .map(|(_, v)| v.trim());
    for name in names {
        if !memo.contains_key(name) {
            let folder = assets.sprite_folder_name(name).map(str::to_owned);
            memo.insert(name.to_owned(), folder);
        }
    }
}

/// The expression a folder's card shows: the most used (the lexically smaller
/// key on a tie), else the first the folder offers.
fn card_variant(variants: &[StorySpriteVariant]) -> Option<StorySpriteVariant> {
    variants
        .iter()
        .filter(|v| v.uses > 0)
        .max_by(|a, b| a.uses.cmp(&b.uses).then_with(|| b.key.cmp(&a.key)))
        .or_else(|| variants.first())
        .cloned()
}

/// A folder's names, most lines first, the name breaking a tie.
fn ranked_names(names: HashMap<String, f64>) -> Vec<StorySpriteName> {
    let mut ranked: Vec<StorySpriteName> = names
        .into_iter()
        .map(|(name, count)| StorySpriteName {
            name,
            count: round2(count),
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.count
            .total_cmp(&a.count)
            .then_with(|| a.name.cmp(&b.name))
    });
    ranked
}

/// Build the gallery from the cached library index. Pure over its inputs
/// plus one read of every distinct library script and the PNG headers the
/// resolver reads for bodies whose hub carries no size. The scripts are read
/// under `assets_dir` and the plates under `art_dir`, the same tree except on
/// a text-only server, whose art is the default server's.
#[must_use]
pub fn build_sprite_index(
    gd: &GameData,
    library: &StoryIndexCache,
    assets: &StoryAssetIndex,
    art_dir: &Path,
    assets_dir: &Path,
) -> (
    StorySpriteIndex,
    HashMap<String, StorySpriteDetail>,
    SpriteCensus,
) {
    let mut census = SpriteCensus::default();
    let ScriptTallies {
        folders: mut tallies,
        stories: story_meta,
    } = walk_scripts(library, assets, assets_dir, &mut census);

    let operators = OperatorIds::new(gd);
    let mut sprites = Vec::new();
    let mut details = HashMap::new();
    for folder in assets.sprite_folder_names() {
        let tally = tallies.remove(folder).unwrap_or_default();
        let mut variants = variants_of(assets, folder, &tally.uses);
        variants.retain(|v| !is_blank_plate(&served_path(art_dir, &v.sprite.body_url)));
        if variants.is_empty() {
            census.hidden.push(folder.to_owned());
            continue;
        }
        let (names, stray) = keep_aliases(ranked_names(tally.names), tally.lines);
        let noise: f64 = stray.iter().map(|n| n.count).sum();
        census.dropped_names += count_u32(stray.len());
        census.dropped_lines += noise;
        let op = operators.operator_for(folder);
        let first = tally
            .stories
            .iter()
            .next()
            .map(|(order, s)| (*order, s.id.clone()));
        let entry = StorySpriteEntry {
            base: folder.to_owned(),
            kind: if op.is_some() {
                StorySpriteKind::Operator
            } else {
                StorySpriteKind::Npc
            },
            char_id: op.as_ref().map(|(id, _, _)| id.clone()),
            operator_name: op.as_ref().map(|(_, name, _)| name.clone()),
            variant: op.and_then(|(_, _, v)| v),
            names,
            noise: round2(noise),
            lines: round2(tally.lines),
            story_count: count_u32(tally.stories.len()),
            first_seen: tally.first_seen,
            first_order: first.as_ref().map(|(o, _)| *o),
            first_story: first.map(|(_, id)| id),
            variant_count: count_u32(variants.len()),
            thumb: card_variant(&variants),
        };
        let stories = tally
            .stories
            .into_values()
            .map(|s| {
                let m = meta_of(&story_meta, &s.id);
                StorySpriteStory {
                    id: s.id,
                    name: m.name,
                    tag: m.tag,
                    group_id: m.group_id,
                    group_name: m.group_name,
                    lines: round2(s.lines),
                    uses: s.uses,
                }
            })
            .collect();
        details.insert(
            folder.to_ascii_lowercase(),
            StorySpriteDetail {
                names: name_details(&entry.names, &tally.name_detail, &story_meta),
                stray_names: name_details(&stray, &tally.name_detail, &story_meta),
                sprite: entry.clone(),
                variants,
                stories,
            },
        );
        sprites.push(entry);
    }
    (StorySpriteIndex { sprites }, details, census)
}

/// The sheet's per-name detail for `names`, in the same order: the stories
/// that speak this sprite under the name (most lines first, [`NAME_STORIES`]
/// of them, and how many more) and up to [`EXAMPLES`] lines.
fn name_details(
    names: &[StorySpriteName],
    detail: &HashMap<String, NameTally>,
    meta: &StoryMetas,
) -> Vec<StorySpriteNameDetail> {
    names
        .iter()
        .map(|n| {
            let Some(d) = detail.get(&n.name) else {
                return StorySpriteNameDetail {
                    name: n.name.clone(),
                    count: n.count,
                    stories: Vec::new(),
                    more_stories: 0,
                    examples: Vec::new(),
                };
            };
            let mut stories: Vec<(u32, &(String, f64))> =
                d.stories.iter().map(|(o, s)| (*o, s)).collect();
            stories.sort_by(|a, b| b.1.1.total_cmp(&a.1.1).then(a.0.cmp(&b.0)));
            let more = stories.len().saturating_sub(NAME_STORIES);
            StorySpriteNameDetail {
                name: n.name.clone(),
                count: n.count,
                stories: stories
                    .into_iter()
                    .take(NAME_STORIES)
                    .map(|(_, (id, lines))| {
                        let m = meta_of(meta, id);
                        StorySpriteNameStory {
                            id: id.clone(),
                            name: m.name,
                            code: m.code,
                            tag: m.tag,
                            lines: round2(*lines),
                        }
                    })
                    .collect(),
                more_stories: count_u32(more),
                examples: d
                    .examples()
                    .into_iter()
                    .map(|e| {
                        let m = meta_of(meta, &e.story_id);
                        StorySpriteExample {
                            story_id: e.story_id.clone(),
                            story_name: m.name,
                            code: m.code,
                            tag: m.tag,
                            line: e.line,
                            text: e.text.clone(),
                        }
                    })
                    .collect(),
            }
        })
        .collect()
}

static SPRITES: ServerCache<StorySpriteCache> = ServerCache::new();

/// How many times the gallery has actually been BUILT in this process.
#[must_use]
pub fn sprite_index_builds() -> u64 {
    SPRITES.builds()
}

/// The cached gallery for a server, built on the first call after each game
/// data load or asset reload. The library index is fetched FIRST, outside the
/// gallery's slot, so a gallery build never holds its slot while waiting on
/// the index's.
pub async fn cached_sprites(
    state: &AppState,
    server: Server,
) -> Result<Arc<StorySpriteCache>, ApiError> {
    let library = cached_index(state, server).await?;
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let gd = server_data.game_data.load_full();
    let trees = StoryTrees::of(&server_data);
    let build_gd = Arc::clone(&gd);
    let fresh_live = Arc::clone(&trees.live);
    SPRITES
        .get_or_build(
            server,
            gd,
            move |hit| {
                hit.assets
                    .upgrade()
                    .is_some_and(|seen| Arc::ptr_eq(&seen, &fresh_live))
            },
            || async move {
                let built = cpu::run("story_sprites", move || {
                    let started = Instant::now();
                    let assets = trees.story_assets();
                    let (index, details, census) = build_sprite_index(
                        &build_gd,
                        &library,
                        &assets,
                        &trees.art_dir,
                        &trees.assets_dir,
                    );
                    Arc::new(StorySpriteCache {
                        index,
                        details,
                        census,
                        build_ms: started.elapsed().as_millis(),
                        assets: Arc::downgrade(&trees.live),
                    })
                })
                .await?;
                tracing::info!(
                    server = ?server,
                    folders = built.index.sprites.len(),
                    scripts = built.census.scripts,
                    build_ms = built.build_ms,
                    "story sprite gallery built"
                );
                Ok(built)
            },
        )
        .await
}

/// The gallery's light list.
pub async fn get_story_sprites(
    state: &AppState,
    server: Server,
) -> Result<StorySpriteIndex, ApiError> {
    Ok(cached_sprites(state, server).await?.index.clone())
}

/// One folder's expression sheet, by folder name (case-insensitive).
pub async fn get_story_sprite(
    state: &AppState,
    server: Server,
    base: &str,
) -> Result<StorySpriteDetail, ApiError> {
    cached_sprites(state, server)
        .await?
        .details
        .get(&base.to_ascii_lowercase())
        .cloned()
        .ok_or_else(|| ApiError::NotFoundMessage(format!("no story sprite folder `{base}`")))
}

/// Where one listed expression's thumb is served from: the server's art
/// root and the cached file under it, rendered first when missing or stale.
/// `key` is the expression's `#N$M` (or `@alias`) exactly as the sheet lists
/// it; an expression the gallery does not list is a 404.
pub async fn get_variant_thumb(
    state: &AppState,
    server: Server,
    base: &str,
    key: &str,
) -> Result<(String, String), ApiError> {
    let gallery = cached_sprites(state, server).await?;
    let detail = gallery
        .details
        .get(&base.to_ascii_lowercase())
        .ok_or(ApiError::NotFound)?;
    let variant = detail
        .variants
        .iter()
        .find(|v| v.key == key)
        .ok_or(ApiError::NotFound)?;
    let server_data = state.try_server_data(server).ok_or(ApiError::NotFound)?;
    let art_dir = server_data.art_dir.clone();
    let rel = super::sprite_thumbs::ensure(&art_dir, &detail.sprite.base, variant).await?;
    Ok((art_dir, rel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_name_reads_as_the_key_the_resolver_reads() {
        assert_eq!(key_of("avg_npc_043_1"), "#1$1");
        assert_eq!(key_of("avg_npc_043_1#2"), "#2$1");
        assert_eq!(key_of("avg_225_haak_1#03$2"), "#3$2");
        assert_eq!(key_of("avg_1_a_1@smile"), "@smile");
    }

    #[test]
    fn an_alias_needs_three_lines_and_two_percent() {
        let n = |name: &str, count: f64| StorySpriteName {
            name: name.to_owned(),
            count,
        };
        let names = |v: &[StorySpriteName]| v.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
        // Kal'tsit: every foreign name is under 2% of 3,102.
        let (kept, stray) = keep_aliases(
            vec![n("Kal'tsit", 3102.0), n("Wei Yenwu", 3.0), n("Mon3tr", 0.5)],
            3108.0,
        );
        assert_eq!(names(&kept), ["Kal'tsit"]);
        assert_eq!(names(&stray), ["Wei Yenwu", "Mon3tr"]);
        // Jie: 14 of 269 is 5.2% and stays; 3 of 269 is 1.1% and goes.
        let (kept, _) = keep_aliases(
            vec![
                n("Jie", 252.0),
                n("Minister of Works", 14.0),
                n("Chun", 3.0),
            ],
            269.0,
        );
        assert_eq!(names(&kept), ["Jie", "Minister of Works"]);
        // A tiny folder keeps its primary below 3 lines; 2.0 >= 2% but < 3 is cut.
        let (kept, stray) = keep_aliases(vec![n("Guard", 2.0), n("Soldier", 2.0)], 4.0);
        assert_eq!(names(&kept), ["Guard"]);
        assert_eq!(names(&stray), ["Soldier"]);
    }

    #[test]
    fn weighted_counts_round_to_two_decimals() {
        assert!((round2(1.0 / 3.0 + 1.0 / 3.0) - 0.67).abs() < 1e-9);
        assert!((round2(342.5) - 342.5).abs() < 1e-9);
    }
}
