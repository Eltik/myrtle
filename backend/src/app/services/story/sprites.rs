//! The CHARACTER GALLERY: every sprite folder under
//! `textures/avg/characters/`, with the names the scripts speak it under and
//! every expression it offers.
//!
//! What this module OWNS is the census, and the census is two walks:
//!
//! - Over the SCRIPTS, once per distinct library script in library order:
//!   [`crate::core::story::speakers`] attributes every named line to the
//!   sprite(s) LIT when it is spoken, the rule the reader's own speaker colour
//!   uses, and counts every time a sprite is put up. That is where `names`,
//!   `lines`, `stories`, `firstSeen` and each expression's `uses` come from.
//! - Over the FOLDERS, once each: every `#N$M` the hub offers, resolved by
//!   [`StoryAssetIndex::resolve_character`] so the gallery and the reader can
//!   never draw one key differently, deduplicated by the files it lands on.
//!
//! Nothing here is per request: the build runs under `cpu::run` and the whole
//! result (the light list AND every folder's detail) is cached per server in a
//! [`ServerCache`], retired by a new `GameData` load or a new asset tree. The
//! list and the detail are two routes because the detail carries every
//! expression and every story, which for all folders at once is several times
//! the list (measured in `docs/story-reader.md`, "Character gallery").

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::Instant;

use super::cache::{ServerCache, cached_index};
use super::dto::{
    StorySpriteDetail, StorySpriteEntry, StorySpriteIndex, StorySpriteKind, StorySpriteName,
    StorySpriteStory, StorySpriteVariant,
};
use super::index::StoryIndexCache;
use crate::app::{cpu, error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::hypergryph::constants::Server;
use crate::core::story::assets::{parse_sprite_name, trim_index};
use crate::core::story::speakers::{attribute_speakers, is_meaningful_name};
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

/// One folder's script-side tallies while the walk runs.
#[derive(Debug, Default)]
struct Tally {
    names: HashMap<String, f64>,
    lines: f64,
    /// Story order -> (story id, lines, uses).
    stories: BTreeMap<u32, (String, f64, u32)>,
    /// Raw sprite name as written -> times put up.
    uses: HashMap<String, u32>,
    first_seen: Option<i64>,
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
    t.stories
        .entry(order)
        .or_insert_with(|| (story_id.to_owned(), 0.0, 0));
    t
}

/// Round a weighted count for the wire: two decimals carry every split the
/// corpus makes (halves and thirds) without a float tail.
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
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
    let mut at: HashMap<(String, Option<String>), usize> = HashMap::new();
    let mut add = |out: &mut Vec<StorySpriteVariant>, key: String, uses: u32| {
        let Some(sprite) = assets.resolve_character(&format!("{folder}{key}")) else {
            return;
        };
        let id = (sprite.body_url.clone(), sprite.face_url.clone());
        if let Some(&i) = at.get(&id) {
            out[i].uses += uses;
            return;
        }
        at.insert(id, out.len());
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

/// Build the gallery from the cached library index. Pure over its inputs
/// plus one read of every distinct library script and the PNG headers the
/// resolver reads for bodies whose hub carries no size.
#[must_use]
pub fn build_sprite_index(
    gd: &GameData,
    library: &StoryIndexCache,
    assets: &StoryAssetIndex,
    assets_dir: &Path,
) -> (
    StorySpriteIndex,
    HashMap<String, StorySpriteDetail>,
    SpriteCensus,
) {
    let mut census = SpriteCensus::default();
    let mut tallies: HashMap<String, Tally> = HashMap::new();
    // Story id -> (name, group id, group name), for the sheet's story list.
    let mut story_meta: HashMap<String, (String, Option<String>, String, String)> = HashMap::new();
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
            story_meta.insert(
                story.id.clone(),
                (
                    story.name.clone(),
                    story.avg_tag.clone(),
                    group.id.clone(),
                    group.name.clone(),
                ),
            );
            let commands = parser::parse(&text);
            // Every distinct name is resolved once up front, so the walk's
            // resolver is a pure lookup that borrows nothing mutably.
            for c in &commands {
                if matches!(
                    c.kind.as_str(),
                    "character" | "charslot" | "charactercutin" | "interlude"
                ) {
                    for (k, v) in &c.args {
                        if k.starts_with("name") && !folder_memo.contains_key(v.trim()) {
                            let f = assets.sprite_folder_name(v.trim()).map(str::to_owned);
                            folder_memo.insert(v.trim().to_owned(), f);
                        }
                    }
                }
            }
            let walk = attribute_speakers(&commands, |raw| {
                folder_memo.get(raw.trim()).cloned().flatten()
            });

            for (folder, raw) in &walk.placements {
                let t = touch(&mut tallies, folder, dated, order, &story.id);
                *t.uses.entry(raw.clone()).or_default() += 1;
                if let Some(s) = t.stories.get_mut(&order) {
                    s.2 += 1;
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
                    let t = touch(&mut tallies, folder, dated, order, &story.id);
                    *t.names.entry(line.speaker.clone()).or_default() += w;
                    t.lines += w;
                    if let Some(s) = t.stories.get_mut(&order) {
                        s.1 += w;
                    }
                }
            }
        }
    }

    let operators = OperatorIds::new(gd);
    let mut sprites = Vec::new();
    let mut details = HashMap::new();
    for folder in assets.sprite_folder_names() {
        let tally = tallies.remove(folder).unwrap_or_default();
        let mut variants = variants_of(assets, folder, &tally.uses);
        variants.retain(|v| {
            !is_blank_plate(
                &assets_dir.join(
                    v.sprite
                        .body_url
                        .trim_start_matches('/')
                        .replace("%23", "#"),
                ),
            )
        });
        if variants.is_empty() {
            census.hidden.push(folder.to_owned());
            continue;
        }
        let thumb = variants
            .iter()
            .filter(|v| v.uses > 0)
            .max_by(|a, b| a.uses.cmp(&b.uses).then_with(|| b.key.cmp(&a.key)))
            .or_else(|| variants.first())
            .cloned();
        let mut names: Vec<StorySpriteName> = tally
            .names
            .into_iter()
            .map(|(name, count)| StorySpriteName {
                name,
                count: round2(count),
            })
            .collect();
        names.sort_by(|a, b| {
            b.count
                .total_cmp(&a.count)
                .then_with(|| a.name.cmp(&b.name))
        });
        let op = operators.operator_for(folder);
        let first = tally
            .stories
            .iter()
            .next()
            .map(|(o, (id, _, _))| (*o, id.clone()));
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
            lines: round2(tally.lines),
            story_count: u32::try_from(tally.stories.len()).unwrap_or(u32::MAX),
            first_seen: tally.first_seen,
            first_order: first.as_ref().map(|(o, _)| *o),
            first_story: first.map(|(_, id)| id),
            variant_count: u32::try_from(variants.len()).unwrap_or(u32::MAX),
            thumb,
        };
        let stories = tally
            .stories
            .into_values()
            .map(|(id, lines, uses)| {
                let (name, tag, group_id, group_name) =
                    story_meta.get(&id).cloned().unwrap_or_default();
                StorySpriteStory {
                    id,
                    name,
                    tag,
                    group_id,
                    group_name,
                    lines: round2(lines),
                    uses,
                }
            })
            .collect();
        details.insert(
            folder.to_ascii_lowercase(),
            StorySpriteDetail {
                sprite: entry.clone(),
                variants,
                stories,
            },
        );
        sprites.push(entry);
    }
    (StorySpriteIndex { sprites }, details, census)
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
    let live = server_data.asset_index.load_full();
    let assets_dir = PathBuf::from(&server_data.assets_dir);
    let build_gd = Arc::clone(&gd);
    let fresh_live = Arc::clone(&live);
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
                    let assets = StoryAssetIndex::for_dir(&assets_dir, &live);
                    let (index, details, census) =
                        build_sprite_index(&build_gd, &library, &assets, &assets_dir);
                    Arc::new(StorySpriteCache {
                        index,
                        details,
                        census,
                        build_ms: started.elapsed().as_millis(),
                        assets: Arc::downgrade(&live),
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
    fn weighted_counts_round_to_two_decimals() {
        assert!((round2(1.0 / 3.0 + 1.0 / 3.0) - 0.67).abs() < 1e-9);
        assert!((round2(342.5) - 342.5).abs() < 1e-9);
    }
}

/// Where one listed expression's thumb is served from: the server's assets
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
    let assets_dir = server_data.assets_dir.clone();
    let rel = super::sprite_thumbs::ensure(&assets_dir, &detail.sprite.base, variant).await?;
    Ok((assets_dir, rel))
}
