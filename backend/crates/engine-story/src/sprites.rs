use std::collections::{BTreeMap, HashMap, HashSet};

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::TableSource;
use myrtle_gamedata_raw::tables::operator::Operator;

use crate::assets::{StoryAssetIndex, parse_sprite_name, served_rel, trim_index};
use crate::dto::{
    StorySpriteDetail, StorySpriteEntry, StorySpriteExample, StorySpriteIndex, StorySpriteKind,
    StorySpriteName, StorySpriteNameDetail, StorySpriteNameStory, StorySpriteStory,
    StorySpriteVariant,
};
use crate::index::StoryLibrary;
use crate::parser;
use crate::script::load_script;
use crate::speakers::{PLACING_COMMANDS, attribute_speakers, is_meaningful_name};
use crate::thumbs::is_blank;

pub const ALIAS_MIN_LINES: f64 = 3.0;
pub const ALIAS_MIN_SHARE: f64 = 0.02;
const EXAMPLES: usize = 3;
const NAME_STORIES: usize = 5;
const BLANK_DECODE_CAP: u64 = 16 * 1024;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SpriteCensus {
    pub scripts: u32,
    pub named_lines: u32,
    pub masked_lines: u32,
    pub unattributed_lines: u32,
    pub split_lines: u32,
    pub hidden: Vec<String>,
    pub dropped_names: u32,
    pub dropped_lines: f64,
}

#[derive(Debug)]
pub struct SpriteGallery {
    pub index: StorySpriteIndex,
    pub details: HashMap<String, StorySpriteDetail>,
    pub census: SpriteCensus,
}

#[must_use]
pub fn keep_aliases(
    names: Vec<StorySpriteName>,
    total: f64,
) -> (Vec<StorySpriteName>, Vec<StorySpriteName>) {
    let (mut kept, mut stray) = (Vec::with_capacity(names.len()), Vec::new());
    for (i, name) in names.into_iter().enumerate() {
        if i == 0 || (name.count >= ALIAS_MIN_LINES && name.count >= ALIAS_MIN_SHARE * total) {
            kept.push(name);
        } else {
            stray.push(name);
        }
    }
    (kept, stray)
}

#[derive(Debug, Default)]
struct Tally {
    names: HashMap<String, f64>,
    name_detail: HashMap<String, NameTally>,
    lines: f64,
    /// Story order -> (id, weighted lines, put-ups)
    stories: BTreeMap<u32, (String, f64, u32)>,
    /// Raw sprite name -> put-ups
    uses: HashMap<String, u32>,
    first_seen: Option<i64>,
}

#[derive(Debug)]
struct ExampleLine {
    order: u32,
    story_id: String,
    line: u32,
    text: String,
}

#[derive(Debug, Default)]
struct NameTally {
    stories: BTreeMap<u32, (String, f64)>,
    candidates: Vec<ExampleLine>,
}

impl NameTally {
    fn add(&mut self, order: u32, story_id: &str, line: u32, text: &str, w: f64) {
        let stories_seen = self.stories.len();
        self.stories
            .entry(order)
            .or_insert_with(|| (story_id.to_owned(), 0.0))
            .1 += w;
        let in_story = self.candidates.iter().filter(|c| c.order == order).count();
        if in_story < EXAMPLES
            && (in_story > 0 || stories_seen < EXAMPLES)
            && !text.trim().is_empty()
        {
            self.candidates.push(ExampleLine {
                order,
                story_id: story_id.to_owned(),
                line,
                text: text.to_owned(),
            });
        }
    }

    fn examples(&self) -> Vec<&ExampleLine> {
        let mut orders: Vec<u32> = self.candidates.iter().map(|c| c.order).collect();
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
                    .filter(|c| c.order == order)
                    .nth(round)
                {
                    out.push(c);
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, Default)]
struct StoryMeta {
    name: String,
    code: Option<String>,
    tag: Option<String>,
    group_id: String,
    group_name: String,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn len_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[derive(Debug, Default)]
pub struct OperatorIds<'a> {
    by_number: HashMap<&'a str, Vec<(&'a str, &'a str)>>,
}

impl<'a> OperatorIds<'a> {
    #[must_use]
    pub fn new(m: &'a GameModel) -> Self {
        let mut by_number: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
        for (id, op) in &m.operators {
            if let Some((num, _)) = id.strip_prefix("char_").and_then(|r| r.split_once('_'))
                && num.bytes().all(|b| b.is_ascii_digit())
            {
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

    #[must_use]
    pub fn operator_for(&self, folder: &str) -> Option<(String, String, Option<String>)> {
        let lower = folder.to_ascii_lowercase();
        let rest = ["avgnew_", "avg_char_", "avg_", "char_"]
            .iter()
            .find_map(|p| lower.strip_prefix(p))?;
        let parts: Vec<&str> = rest.split('_').collect();
        let num = parts[0];
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

#[must_use]
pub fn sprite_operator<'a>(m: &'a GameModel, id: &str) -> Option<(String, &'a Operator)> {
    let rest = ["avgnew_", "avg_char_", "avg_", "char_"]
        .iter()
        .find_map(|p| id.strip_prefix(p))?;
    if !rest.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let char_id = format!("char_{rest}");
    let op = m.operators.get(&char_id)?;
    Some((char_id, op))
}

#[must_use]
pub fn key_of(raw: &str) -> String {
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

fn variants_of(
    assets: &StoryAssetIndex,
    folder: &str,
    uses: &HashMap<String, u32>,
) -> Vec<StorySpriteVariant> {
    let mut out: Vec<StorySpriteVariant> = Vec::new();
    let mut at: HashMap<(String, Option<String>), usize> = HashMap::new();
    let mut used: Vec<(&String, &u32)> = uses.iter().collect();
    used.sort();
    let keys = assets
        .sprite_variant_keys(folder)
        .into_iter()
        .map(|k| (k, 0))
        .chain(used.into_iter().map(|(raw, n)| (key_of(raw), *n)));
    for (key, n) in keys {
        let Some(sprite) = assets.resolve_character(&format!("{folder}{key}")) else {
            continue;
        };
        let files = (sprite.body_url.clone(), sprite.face_url.clone());
        if let Some(&i) = at.get(&files) {
            out[i].uses += n;
            continue;
        }
        at.insert(files, out.len());
        out.push(StorySpriteVariant {
            key,
            whole_body: sprite.face_url.is_none(),
            sprite,
            uses: n,
        });
    }
    out
}

fn touch<'t>(
    tallies: &'t mut HashMap<String, Tally>,
    folder: &str,
    dated: Option<i64>,
    order: u32,
    story_id: &str,
) -> &'t mut (String, f64, u32) {
    let t = tallies.entry(folder.to_owned()).or_default();
    if let Some(d) = dated {
        t.first_seen = Some(t.first_seen.map_or(d, |f| f.min(d)));
    }
    t.stories
        .entry(order)
        .or_insert_with(|| (story_id.to_owned(), 0.0, 0))
}

fn walk_scripts(
    library: &StoryLibrary,
    assets: &StoryAssetIndex,
    text: &dyn TableSource,
    census: &mut SpriteCensus,
) -> (HashMap<String, Tally>, HashMap<String, StoryMeta>) {
    let mut tallies: HashMap<String, Tally> = HashMap::new();
    let mut metas = HashMap::new();
    let mut seen_txt: HashSet<&str> = HashSet::new();
    let mut order: u32 = 0;
    let mut folder_of: HashMap<String, Option<String>> = HashMap::new();
    for group in &library.index.groups {
        let dated = (group.start_time > 0).then_some(group.start_time);
        for story in group.stories.iter().filter(|s| s.has_script) {
            let Some(sref) = library.lookup.get(&story.id) else {
                continue;
            };
            if !seen_txt.insert(sref.story_txt.as_str()) {
                continue;
            }
            let Ok(script) = load_script(text, &sref.story_txt) else {
                continue;
            };
            census.scripts += 1;
            order += 1;
            metas.insert(
                story.id.clone(),
                StoryMeta {
                    name: story.name.clone(),
                    code: story.code.clone(),
                    tag: story.avg_tag.clone(),
                    group_id: group.id.clone(),
                    group_name: group.name.clone(),
                },
            );
            let commands = parser::parse(&script);
            for name in commands
                .iter()
                .filter(|c| PLACING_COMMANDS.contains(&c.kind.as_str()))
                .flat_map(|c| &c.args)
                .filter(|(k, _)| k.starts_with("name"))
                .map(|(_, v)| v.trim())
            {
                if !folder_of.contains_key(name) {
                    folder_of.insert(
                        name.to_owned(),
                        assets.sprite_folder_name(name).map(str::to_owned),
                    );
                }
            }
            let walk = attribute_speakers(&commands, |raw| {
                folder_of.get(raw.trim()).cloned().flatten()
            });

            for (folder, raw) in &walk.placements {
                touch(&mut tallies, folder, dated, order, &story.id).2 += 1;
                if let Some(t) = tallies.get_mut(folder) {
                    *t.uses.entry(raw.clone()).or_default() += 1;
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
                    touch(&mut tallies, folder, dated, order, &story.id).1 += w;
                    if let Some(t) = tallies.get_mut(folder) {
                        *t.names.entry(line.speaker.clone()).or_default() += w;
                        t.name_detail
                            .entry(line.speaker.clone())
                            .or_default()
                            .add(order, &story.id, line.line, &line.text, *w);
                        t.lines += w;
                    }
                }
            }
        }
    }
    (tallies, metas)
}

fn card_variant(variants: &[StorySpriteVariant]) -> Option<StorySpriteVariant> {
    variants
        .iter()
        .filter(|v| v.uses > 0)
        .max_by(|a, b| a.uses.cmp(&b.uses).then_with(|| b.key.cmp(&a.key)))
        .or_else(|| variants.first())
        .cloned()
}

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

fn name_details(
    names: &[StorySpriteName],
    detail: &HashMap<String, NameTally>,
    metas: &HashMap<String, StoryMeta>,
) -> Vec<StorySpriteNameDetail> {
    let meta = |id: &str| metas.get(id).cloned().unwrap_or_default();
    names
        .iter()
        .map(|n| {
            let mut out = StorySpriteNameDetail {
                name: n.name.clone(),
                count: n.count,
                stories: Vec::new(),
                more_stories: 0,
                examples: Vec::new(),
            };
            let Some(d) = detail.get(&n.name) else {
                return out;
            };
            let mut stories: Vec<(u32, &(String, f64))> =
                d.stories.iter().map(|(o, s)| (*o, s)).collect();
            stories.sort_by(|a, b| b.1.1.total_cmp(&a.1.1).then(a.0.cmp(&b.0)));
            out.more_stories = len_u32(stories.len().saturating_sub(NAME_STORIES));
            out.stories = stories
                .into_iter()
                .take(NAME_STORIES)
                .map(|(_, (id, lines))| {
                    let m = meta(id);
                    StorySpriteNameStory {
                        id: id.clone(),
                        name: m.name,
                        code: m.code,
                        tag: m.tag,
                        lines: round2(*lines),
                    }
                })
                .collect();
            out.examples = d
                .examples()
                .into_iter()
                .map(|e| {
                    let m = meta(&e.story_id);
                    StorySpriteExample {
                        story_id: e.story_id.clone(),
                        story_name: m.name,
                        code: m.code,
                        tag: m.tag,
                        line: e.line,
                        text: e.text.clone(),
                    }
                })
                .collect();
            out
        })
        .collect()
}

#[must_use]
pub fn build_sprite_index(
    m: &GameModel,
    library: &StoryLibrary,
    assets: &StoryAssetIndex,
    text: &dyn TableSource,
    art: &dyn TableSource,
    file_len: &dyn Fn(&str) -> Option<u64>,
) -> SpriteGallery {
    let mut census = SpriteCensus::default();
    let (mut tallies, metas) = walk_scripts(library, assets, text, &mut census);
    let blank = |url: &str| {
        file_len(url).is_some_and(|n| n <= BLANK_DECODE_CAP)
            && art
                .read_asset(&served_rel(url))
                .ok()
                .flatten()
                .is_some_and(|b| is_blank(&b))
    };
    let operators = OperatorIds::new(m);
    let mut sprites = Vec::new();
    let mut details = HashMap::new();
    for folder in assets.sprite_folder_names() {
        let tally = tallies.remove(folder).unwrap_or_default();
        let mut variants = variants_of(assets, folder, &tally.uses);
        variants.retain(|v| !blank(&v.sprite.body_url));
        if variants.is_empty() {
            census.hidden.push(folder.to_owned());
            continue;
        }
        let (names, stray) = keep_aliases(ranked_names(tally.names), tally.lines);
        let noise: f64 = stray.iter().map(|n| n.count).sum();
        census.dropped_names += len_u32(stray.len());
        census.dropped_lines += noise;
        let op = operators.operator_for(folder);
        let first = tally.stories.iter().next().map(|(o, s)| (*o, s.0.clone()));
        let (char_id, operator_name, variant) = match op {
            Some((id, name, v)) => (Some(id), Some(name), v),
            None => (None, None, None),
        };
        let entry = StorySpriteEntry {
            base: folder.to_owned(),
            kind: if char_id.is_some() {
                StorySpriteKind::Operator
            } else {
                StorySpriteKind::Npc
            },
            char_id,
            operator_name,
            variant,
            names,
            noise: round2(noise),
            lines: round2(tally.lines),
            story_count: len_u32(tally.stories.len()),
            first_seen: tally.first_seen,
            first_order: first.as_ref().map(|(o, _)| *o),
            first_story: first.map(|(_, id)| id),
            variant_count: len_u32(variants.len()),
            thumb: card_variant(&variants),
        };
        let stories = tally
            .stories
            .into_values()
            .map(|(id, lines, uses)| {
                let m = metas.get(&id).cloned().unwrap_or_default();
                StorySpriteStory {
                    id,
                    name: m.name,
                    tag: m.tag,
                    group_id: m.group_id,
                    group_name: m.group_name,
                    lines: round2(lines),
                    uses,
                }
            })
            .collect();
        details.insert(
            folder.to_ascii_lowercase(),
            StorySpriteDetail {
                names: name_details(&entry.names, &tally.name_detail, &metas),
                stray_names: name_details(&stray, &tally.name_detail, &metas),
                sprite: entry.clone(),
                variants,
                stories,
            },
        );
        sprites.push(entry);
    }
    SpriteGallery {
        index: StorySpriteIndex { sprites },
        details,
        census,
    }
}
