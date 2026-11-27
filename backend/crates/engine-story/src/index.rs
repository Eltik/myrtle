use std::collections::{HashMap, HashSet};

use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::TableSource;
use myrtle_gamedata_raw::tables::operator::{OperatorProfession, OperatorRarity};
use myrtle_gamedata_raw::tables::stage::StorylineStorySet;
use myrtle_gamedata_raw::tables::story::{StoryRequiredStage, StoryReviewGroup};

use crate::archive::build_archives;
use crate::assets::StoryAssetIndex;
use crate::dto::{
    ChapterRange, IllustrationItem, OperatorRecordGroup, SpriteItem, StoryArchive, StoryCategory,
    StoryCoverKind, StoryEntry, StoryGroup, StoryIllustrations, StoryIndex, StoryMusic,
    StoryTotals, StoryZone, Storyline, StorylineArc,
};
use crate::opt;
use crate::script::{
    ScriptError, ScriptFacts, StoryScript, load_script, load_synopsis, parse_story, script_facts,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryRef {
    pub story_txt: String,
    pub name: String,
    pub group_id: String,
    pub story_info: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NameRefs {
    pub name: String,
    pub story_ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SpriteRefs {
    pub base: String,
    pub story_ids: Vec<String>,
    pub faces: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct GroupRefs {
    pub backgrounds: Vec<NameRefs>,
    pub images: Vec<NameRefs>,
    pub sprites: Vec<SpriteRefs>,
}

#[derive(Debug, Default)]
pub struct StoryLibrary {
    pub index: StoryIndex,
    pub lookup: HashMap<String, StoryRef>,
    pub by_txt: HashMap<String, String>,
    pub gates: HashMap<String, Vec<StoryRequiredStage>>,
    pub illustrations: HashMap<String, GroupRefs>,
    pub archives: HashMap<String, StoryArchive>,
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[derive(Debug, Default)]
struct RefsBuilder {
    refs: GroupRefs,
    at: HashMap<(u8, String), usize>,
}

fn push_story(ids: &mut Vec<String>, story: &str) {
    if ids.last().map(String::as_str) != Some(story) {
        ids.push(story.to_owned());
    }
}

impl RefsBuilder {
    fn name(&mut self, kind: u8, name: &str, story: &str) {
        let list = if kind == 0 {
            &mut self.refs.backgrounds
        } else {
            &mut self.refs.images
        };
        let i = *self.at.entry((kind, name.to_owned())).or_insert_with(|| {
            list.push(NameRefs {
                name: name.to_owned(),
                story_ids: Vec::new(),
            });
            list.len() - 1
        });
        push_story(&mut list[i].story_ids, story);
    }

    fn add(&mut self, story: &str, facts: &ScriptFacts) {
        for name in &facts.backgrounds {
            self.name(0, name, story);
        }
        for name in &facts.images {
            self.name(1, name, story);
        }
        for sprite in &facts.sprites {
            let sprites = &mut self.refs.sprites;
            let i = *self.at.entry((2, sprite.base.clone())).or_insert_with(|| {
                sprites.push(SpriteRefs {
                    base: sprite.base.clone(),
                    ..SpriteRefs::default()
                });
                sprites.len() - 1
            });
            let row = &mut sprites[i];
            push_story(&mut row.story_ids, story);
            for face in &sprite.faces {
                if !row.faces.contains(face) {
                    row.faces.push(face.clone());
                }
            }
        }
    }
}

fn category_for(entry_type: &str, act_type: &str, first_txt: &str) -> StoryCategory {
    if first_txt.starts_with("obt/rogue") {
        return StoryCategory::Is;
    }
    if first_txt.starts_with("obt/sandbox") {
        return StoryCategory::Reclamation;
    }
    match (entry_type, act_type) {
        ("MAINLINE", _) => StoryCategory::Main,
        ("ACTIVITY", "ACTIVITY_STORY") => StoryCategory::Side,
        ("MINI_ACTIVITY", _) | (_, "MINI_STORY") => StoryCategory::Vignette,
        ("NONE", _) if first_txt.starts_with("obt/memory") => StoryCategory::Record,
        _ => StoryCategory::SideContent,
    }
}

#[must_use]
pub fn group_category(g: &StoryReviewGroup) -> StoryCategory {
    let first_txt = g
        .info_unlock_datas
        .iter()
        .map(|s| s.story_txt.as_str())
        .find(|t| !t.is_empty())
        .unwrap_or("");
    category_for(&g.entry_type, &g.act_type, first_txt)
}

fn zone_id_for(m: &GameModel, group: &StoryReviewGroup) -> Option<String> {
    if m.zones.contains_key(&group.id) {
        return Some(group.id.clone());
    }
    group
        .info_unlock_datas
        .iter()
        .flat_map(|s| &s.required_stages)
        .find_map(|rs| m.stages.get(&rs.stage_id).map(|st| st.zone_id.clone()))
}

fn chapter_id_for(m: &GameModel, zone_id: &str) -> Option<String> {
    m.zone_chapters.get(zone_id).cloned().or_else(|| {
        let n: i32 = zone_id.strip_prefix("main_")?.parse().ok()?;
        m.chapters.values().find_map(|c| {
            let lo: i32 = c.start_zone_id.strip_prefix("main_")?.parse().ok()?;
            let hi: i32 = c.end_zone_id.strip_prefix("main_")?.parse().ok()?;
            (lo <= n && n <= hi).then(|| c.chapter_id.clone())
        })
    })
}

fn zone_for(m: &GameModel, zone_id: &str) -> Option<StoryZone> {
    let zone = m.zones.get(zone_id)?;
    Some(StoryZone {
        chapter_name: chapter_id_for(m, zone_id)
            .and_then(|c| m.chapters.get(&c))
            .map(|c| c.chapter_name.clone()),
        name_first: zone.zone_name_first.clone(),
        name_second: zone.zone_name_second.clone(),
        name_third: zone.zone_name_third.clone(),
    })
}

fn chapter_number_for(m: &GameModel, group_id: &str, zone_id: Option<&str>) -> Option<u32> {
    zone_id
        .and_then(|z| m.zones.get(z))
        .and_then(|z| z.zone_name_title_current.as_deref())
        .and_then(|t| t.trim().parse::<u32>().ok())
        .or_else(|| group_id.strip_prefix("main_")?.parse().ok())
}

fn main_ordinal(id: &str) -> i32 {
    id.strip_prefix("main_")
        .and_then(|n| n.parse().ok())
        .unwrap_or(i32::MAX)
}

const fn stars(rarity: &OperatorRarity) -> u8 {
    match rarity {
        OperatorRarity::SixStar => 6,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::FourStar => 4,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::OneStar => 1,
    }
}

fn avatar_for(assets: &AssetIndex, char_id: &str) -> Option<String> {
    ["", "_2", "_1"].into_iter().find_map(|suffix| {
        assets
            .path(AssetKind::Avatar, &format!("{char_id}{suffix}"))
            .map(str::to_owned)
    })
}

fn storyline_art(assets: &AssetIndex, id: Option<&str>) -> Option<String> {
    assets
        .path(AssetKind::StorylineArt, id?.trim())
        .map(str::to_owned)
}

fn cover_for(assets: &AssetIndex, pic_id: Option<&str>) -> Option<String> {
    let pic = pic_id?;
    let key = pic.strip_prefix("storyEntryPic_").unwrap_or(pic);
    assets
        .path(AssetKind::StoryEntryPic, key)
        .map(str::to_owned)
}

fn group_music(
    m: &GameModel,
    assets: &StoryAssetIndex,
    group_id: &str,
    category: StoryCategory,
) -> Option<StoryMusic> {
    let banks = &m.music;
    let exact = format!("sys.ON_ACTIVITY_LOADED.{group_id}");
    let music = if category == StoryCategory::Main {
        banks.music(&format!(
            "music_3in1bg_main{}",
            group_id.strip_prefix("main_")?
        ))
    } else {
        banks
            .music_for_bank(&exact)
            .or_else(|| banks.music_by_bank_prefix(&format!("{exact}.")))
            .or_else(|| banks.music(&format!("music_3in1bg_{group_id}")))
    };
    let bank = banks.bank(music.map_or(&exact, |m| &m.bank))?;
    Some(StoryMusic {
        title: music.and_then(|m| opt(&m.name)),
        intro_url: bank.intro.as_deref().and_then(|i| assets.resolve_audio(i)),
        loop_url: assets.resolve_audio(bank.loop_.as_deref()?)?,
    })
}

fn zone_owners(m: &GameModel) -> HashMap<&str, &str> {
    let mut out = HashMap::new();
    for g in m.story_reviews.values() {
        for rs in g.info_unlock_datas.iter().flat_map(|s| &s.required_stages) {
            if let Some(stage) = m.stages.get(&rs.stage_id) {
                out.entry(stage.zone_id.as_str()).or_insert(g.id.as_str());
            }
        }
    }
    out
}

fn resolve_group(m: &GameModel, owners: &HashMap<&str, &str>, id: &str) -> Option<String> {
    if m.story_reviews.contains_key(id) {
        return Some(id.to_owned());
    }
    let prefix = format!("{id}_");
    owners
        .iter()
        .filter(|(zone, _)| zone.starts_with(&prefix))
        .map(|(_, group)| (*group).to_owned())
        .min()
}

fn story_sets_by_group<'a>(
    m: &'a GameModel,
    owners: &HashMap<&str, &str>,
) -> HashMap<String, &'a StorylineStorySet> {
    let mut sets: Vec<&StorylineStorySet> = m.storyline_story_sets.values().collect();
    sets.sort_by(|a, b| a.story_set_id.cmp(&b.story_set_id));
    let mut out = HashMap::new();
    for set in sets {
        if let Some(group) = set.group_id().and_then(|id| resolve_group(m, owners, id)) {
            out.entry(group).or_insert(set);
        }
    }
    out
}

fn chapter_range(group_ids: &[String], numbers: &HashMap<String, u32>) -> Option<ChapterRange> {
    let mut it = group_ids.iter().filter_map(|g| numbers.get(g).copied());
    let first = it.next()?;
    let (from, to) = it.fold((first, first), |(lo, hi), n| (lo.min(n), hi.max(n)));
    Some(ChapterRange { from, to })
}

fn build_storylines(
    m: &GameModel,
    assets: &AssetIndex,
    owners: &HashMap<&str, &str>,
    numbers: &HashMap<String, u32>,
) -> Vec<Storyline> {
    let mut lines: Vec<_> = m.storylines.iter().collect();
    lines.sort_by(|a, b| {
        a.sort_id
            .cmp(&b.sort_id)
            .then_with(|| a.storyline_id.cmp(&b.storyline_id))
    });
    lines
        .into_iter()
        .map(|line| {
            let mut locations: Vec<_> = line.locations.iter().collect();
            locations.sort_by_key(|l| l.sort_id);
            let (mut group_ids, mut own_group_ids, mut arcs) = (Vec::new(), Vec::new(), Vec::new());
            for location in locations {
                if let Some(split) = &location.mainline_split_data {
                    if let Some(name) = split.sub_name.clone().filter(|n| !n.is_empty()) {
                        arcs.push(StorylineArc {
                            name,
                            sort: location.sort_id,
                            icon_url: storyline_art(assets, split.icon_id.as_deref()),
                            group_ids: Vec::new(),
                            chapter_range: None,
                        });
                    }
                    continue;
                }
                let Some(group_id) = location
                    .relevant_story_set_id
                    .as_deref()
                    .and_then(|id| m.storyline_story_sets.get(id))
                    .and_then(StorylineStorySet::group_id)
                    .and_then(|id| resolve_group(m, owners, id))
                else {
                    continue;
                };
                if let Some(arc) = arcs.last_mut() {
                    arc.group_ids.push(group_id.clone());
                }
                if location.location_type == "STORY_SET" && !own_group_ids.contains(&group_id) {
                    own_group_ids.push(group_id.clone());
                }
                if !group_ids.contains(&group_id) {
                    group_ids.push(group_id);
                }
            }
            for arc in &mut arcs {
                arc.chapter_range = chapter_range(&arc.group_ids, numbers);
            }
            Storyline {
                id: line.storyline_id.clone(),
                name: line.storyline_name.clone(),
                sort: line.sort_id,
                icon_url: storyline_art(assets, line.storyline_icon_id.as_deref())
                    .or_else(|| storyline_art(assets, line.storyline_logo_id.as_deref())),
                logo_url: storyline_art(assets, line.storyline_logo_id.as_deref()),
                chapter_range: chapter_range(&group_ids, numbers),
                group_ids,
                own_group_ids,
                arcs,
            }
        })
        .collect()
}

fn sort_stories(stories: &mut [StoryEntry]) {
    stories.sort_by(|a, b| a.sort.cmp(&b.sort).then_with(|| a.id.cmp(&b.id)));
}

#[must_use]
pub fn build_index(
    m: &GameModel,
    art: &AssetIndex,
    assets: &StoryAssetIndex,
    text: &dyn TableSource,
) -> StoryLibrary {
    let mut lib = StoryLibrary::default();
    let mut measured: HashMap<String, Option<ScriptFacts>> = HashMap::new();
    let mut probe = |txt: &str| -> Option<ScriptFacts> {
        measured
            .entry(txt.to_owned())
            .or_insert_with(|| script_facts(text, txt, assets))
            .clone()
    };
    let owners = zone_owners(m);
    let story_sets = story_sets_by_group(m, &owners);
    let mut numbers: HashMap<String, u32> = HashMap::new();
    let mut groups = Vec::with_capacity(m.story_reviews.len());
    for g in m.story_reviews.values() {
        let category = group_category(g);
        let mut first_bgs: HashMap<&str, String> = HashMap::new();
        let mut refs = RefsBuilder::default();
        let mut stories: Vec<StoryEntry> = g
            .info_unlock_datas
            .iter()
            .map(|s| {
                lib.lookup
                    .entry(s.story_id.clone())
                    .or_insert_with(|| StoryRef {
                        story_txt: s.story_txt.clone(),
                        name: s.story_name.clone(),
                        group_id: g.id.clone(),
                        story_info: s.story_info.as_deref().and_then(opt),
                    });
                if !s.story_txt.is_empty() {
                    lib.by_txt
                        .entry(s.story_txt.clone())
                        .or_insert_with(|| s.story_id.clone());
                }
                lib.gates
                    .entry(s.story_id.clone())
                    .or_insert_with(|| s.required_stages.clone());
                let facts = probe(&s.story_txt);
                if let Some(f) = &facts {
                    if let Some(bg) = f.first_background() {
                        first_bgs.insert(s.story_id.as_str(), bg.to_owned());
                    }
                    refs.add(&s.story_id, f);
                }
                StoryEntry {
                    id: s.story_id.clone(),
                    name: s.story_name.clone(),
                    code: s.story_code.clone().filter(|c| !c.is_empty()),
                    sort: s.story_sort,
                    avg_tag: s.avg_tag.clone().filter(|t| !t.is_empty()),
                    group_id: g.id.clone(),
                    has_script: facts.is_some(),
                    word_count: facts.as_ref().map_or(0, |f| f.word_count),
                    has_video: facts.as_ref().is_some_and(|f| f.has_video),
                    required_stages: s
                        .required_stages
                        .iter()
                        .map(|r| r.stage_id.clone())
                        .collect(),
                }
            })
            .collect();
        sort_stories(&mut stories);
        let zone_id = (category == StoryCategory::Main)
            .then(|| zone_id_for(m, g))
            .flatten();
        let start_time = zone_id
            .as_ref()
            .and_then(|z| m.zone_open_times.get(z).copied())
            .filter(|t| *t > 0)
            .unwrap_or(g.start_time);
        let (cover_url, cover_kind) = cover_for(art, g.story_entry_pic_id.as_deref()).map_or_else(
            || {
                let derived = stories
                    .iter()
                    .filter(|s| s.has_script)
                    .find_map(|s| first_bgs.get(s.id.as_str()))
                    .and_then(|name| assets.resolve_background(name))
                    .map(|(url, _)| url);
                let kind = derived.as_ref().map(|_| StoryCoverKind::Background);
                (derived, kind)
            },
            |url| (Some(url), Some(StoryCoverKind::EntryPic)),
        );
        let set = story_sets.get(&g.id).copied();
        let chapter_number = (category == StoryCategory::Main)
            .then(|| chapter_number_for(m, &g.id, zone_id.as_deref()))
            .flatten();
        if let Some(n) = chapter_number {
            numbers.insert(g.id.clone(), n);
        }
        let refs = refs.refs;
        groups.push(StoryGroup {
            id: g.id.clone(),
            name: g.name.clone(),
            category,
            entry_type: g.entry_type.clone(),
            act_type: g.act_type.clone(),
            display_type: m
                .activities
                .get(&g.id)
                .map(|a| a.display_type.clone())
                .filter(|d| !d.is_empty()),
            cover_url,
            cover_kind,
            banner_url: storyline_art(art, set.and_then(|s| s.kv_image_id.as_deref())),
            title_image_url: storyline_art(art, set.and_then(|s| s.title_image_id.as_deref())),
            icon_url: storyline_art(
                art,
                set.and_then(|s| s.mainline_data.as_ref())
                    .and_then(|d| d.deco_image_id.as_deref()),
            ),
            chapter_number,
            start_time,
            zone: zone_id.as_deref().and_then(|z| zone_for(m, z)),
            word_count: stories.iter().map(|s| s.word_count).sum(),
            illustration_count: count(refs.backgrounds.len() + refs.images.len()),
            sprite_count: count(refs.sprites.len()),
            music: group_music(m, assets, &g.id, category),
            stories,
        });
        lib.illustrations.insert(g.id.clone(), refs);
    }
    groups.sort_by(|a, b| {
        let rank = |c: StoryCategory| StoryCategory::ALL.iter().position(|x| *x == c);
        rank(a.category)
            .cmp(&rank(b.category))
            .then_with(|| main_ordinal(&a.id).cmp(&main_ordinal(&b.id)))
            .then_with(|| a.start_time.cmp(&b.start_time))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut records = Vec::new();
    let mut handbook: Vec<_> = m.handbook.handbook_dict.iter().collect();
    handbook.sort_by(|a, b| a.0.cmp(b.0));
    for (char_id, item) in handbook {
        let mut stories = Vec::new();
        let mut refs = RefsBuilder::default();
        for set in &item.handbook_avg_list {
            for avg in &set.avg_list {
                let name = lib
                    .lookup
                    .get(&avg.story_id)
                    .map_or_else(|| set.story_set_name.clone(), |r| r.name.clone());
                lib.lookup
                    .entry(avg.story_id.clone())
                    .or_insert_with(|| StoryRef {
                        story_txt: avg.story_txt.clone(),
                        name: name.clone(),
                        group_id: set.story_set_id.clone(),
                        story_info: opt(&avg.story_info),
                    });
                if !avg.story_txt.is_empty() {
                    lib.by_txt
                        .entry(avg.story_txt.clone())
                        .or_insert_with(|| avg.story_id.clone());
                }
                let facts = probe(&avg.story_txt);
                if let Some(f) = &facts {
                    refs.add(&avg.story_id, f);
                }
                stories.push(StoryEntry {
                    id: avg.story_id.clone(),
                    name,
                    code: None,
                    sort: set.sort_id * 100 + avg.story_sort,
                    avg_tag: None,
                    group_id: set.story_set_id.clone(),
                    has_script: facts.is_some(),
                    word_count: facts.as_ref().map_or(0, |f| f.word_count),
                    has_video: facts.as_ref().is_some_and(|f| f.has_video),
                    required_stages: Vec::new(),
                });
            }
        }
        if stories.is_empty() {
            continue;
        }
        sort_stories(&mut stories);
        let op = m.operators.get(char_id);
        let refs = refs.refs;
        records.push(OperatorRecordGroup {
            char_id: char_id.clone(),
            name: op.map_or_else(|| char_id.clone(), |o| o.name.clone()),
            rarity: op.map_or(0, |o| stars(&o.rarity)),
            profession: op.map_or(OperatorProfession::Unknown, |o| o.profession.clone()),
            profession_name: op.and_then(|o| o.profession_name.clone()),
            avatar_url: avatar_for(art, char_id).unwrap_or_default(),
            word_count: stories.iter().map(|s| s.word_count).sum(),
            illustration_count: count(refs.backgrounds.len() + refs.images.len()),
            sprite_count: count(refs.sprites.len()),
            stories,
        });
        lib.illustrations.insert(char_id.clone(), refs);
    }

    let mut seen = HashSet::new();
    let mut totals = StoryTotals::default();
    for e in groups
        .iter()
        .flat_map(|g| &g.stories)
        .chain(records.iter().flat_map(|r| &r.stories))
    {
        if seen.insert(e.id.as_str()) {
            totals.stories += 1;
            if e.has_script {
                totals.with_script += 1;
                totals.words += e.word_count;
            }
        }
    }

    lib.archives = build_archives(m, assets);
    lib.index = StoryIndex {
        storylines: build_storylines(m, art, &owners, &numbers),
        groups,
        records,
        totals,
    };
    lib
}

impl StoryLibrary {
    #[must_use]
    pub fn illustrations(
        &self,
        assets: &StoryAssetIndex,
        group_id: &str,
    ) -> Option<StoryIllustrations> {
        let refs = self.illustrations.get(group_id)?;
        let items = |names: &[NameRefs], resolve: &dyn Fn(&str) -> Option<String>| {
            names
                .iter()
                .map(|r| IllustrationItem {
                    name: r.name.clone(),
                    url: resolve(&r.name),
                    story_ids: r.story_ids.clone(),
                })
                .collect()
        };
        Some(StoryIllustrations {
            group_id: group_id.to_owned(),
            backgrounds: items(&refs.backgrounds, &|n| {
                assets.resolve_background(n).map(|(u, _)| u)
            }),
            images: items(&refs.images, &|n| assets.resolve_image(n).map(|(u, _)| u)),
            sprites: refs
                .sprites
                .iter()
                .map(|r| SpriteItem {
                    base: r.base.clone(),
                    body_url: assets.resolve_character(&r.base).map(|c| c.body_url),
                    story_ids: r.story_ids.clone(),
                    faces: count(r.faces.len()),
                })
                .collect(),
        })
    }

    pub fn load_story(
        &self,
        text: &dyn TableSource,
        assets: &StoryAssetIndex,
        story_id: &str,
    ) -> Result<Option<StoryScript>, ScriptError> {
        let Some(r) = self.lookup.get(story_id) else {
            return Ok(None);
        };
        let script = load_script(text, &r.story_txt)?;
        let mut parsed = parse_story(story_id, &r.name, &r.group_id, &script, assets);
        parsed.synopsis = r
            .story_info
            .as_deref()
            .and_then(|info| load_synopsis(text, info));
        Ok(Some(parsed))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainlineChapter {
    pub group_id: String,
    pub name: String,
    pub number: Option<u32>,
    /// `chapter_table` index and name
    pub act: Option<(i32, String)>,
    pub banner_url: Option<String>,
}

#[must_use]
pub fn mainline_chapters(m: &GameModel, art: &AssetIndex) -> Vec<MainlineChapter> {
    let owners = zone_owners(m);
    let story_sets = story_sets_by_group(m, &owners);
    let mut out: Vec<MainlineChapter> = m
        .story_reviews
        .values()
        .filter(|g| group_category(g) == StoryCategory::Main)
        .map(|g| {
            let zone_id = zone_id_for(m, g);
            MainlineChapter {
                group_id: g.id.clone(),
                name: g.name.clone(),
                number: chapter_number_for(m, &g.id, zone_id.as_deref()),
                act: zone_id
                    .as_deref()
                    .and_then(|z| chapter_id_for(m, z))
                    .and_then(|c| m.chapters.get(&c))
                    .map(|c| (c.chapter_index, c.chapter_name.clone())),
                banner_url: storyline_art(
                    art,
                    story_sets.get(&g.id).and_then(|s| s.kv_image_id.as_deref()),
                ),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        (
            a.number.unwrap_or(u32::MAX),
            main_ordinal(&a.group_id),
            &a.group_id,
        )
            .cmp(&(
                b.number.unwrap_or(u32::MAX),
                main_ordinal(&b.group_id),
                &b.group_id,
            ))
    });
    out
}
