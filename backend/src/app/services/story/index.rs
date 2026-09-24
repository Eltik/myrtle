//! The BUILD: loaded game data plus one pass over the scripts on disk, turned
//! into the whole library index.
//!
//! What this module OWNS is the derivation, and its cost. The walk is linear in
//! the library and the constants are the reason the result is cached rather
//! than recomputed:
//!
//! - 451 EN groups and 315 operator records over 2,254 story slots, which name
//!   1,887 DISTINCT script paths. The probe memoises on the path, so 367 record
//!   stories that a `record` group already listed are read once, not twice.
//! - One read plus one parse per distinct path: 5,793 ms of the 7,279 ms debug
//!   build. The word count and the illustration names come out of that same
//!   read, so no later request re-opens a script to count anything.
//! - Everything after the probe is a walk over what is already in memory: the
//!   totals pass, the storyline shelves and the sort are each one pass over the
//!   2,254 entries.
//!
//! Which is why [`super::cache`] holds the result per server and lets exactly
//! one caller build it. The census behind the numbers above lives in
//! `docs/story-reader.md`, section "1. What is true about the data".

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::archive::build_archives;
use super::art::{avatar_for, cover_for, storyline_art_for};
use super::dto::{
    ChapterRange, OperatorRecordGroup, StoryArchive, StoryCategory, StoryCoverKind, StoryEntry,
    StoryGroup, StoryIndex, StoryTotals, StoryZone, Storyline, StorylineArc,
};
use super::illustrations::{GroupRefs, GroupRefsBuilder};
use super::music::group_music;
use crate::app::services::operators::rarity_to_stars;
use crate::core::gamedata::assets::AssetIndex;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::OperatorProfession;
use crate::core::gamedata::types::stage::StorylineStorySet;
use crate::core::gamedata::types::story_review::StoryRequiredStage;
use crate::core::story::{self, ScriptFacts, StoryAssetIndex};

/// What `GET /story/{id}` needs to find and label a script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryRef {
    pub story_txt: String,
    pub name: String,
    pub group_id: String,
}

/// The built index plus the id -> script lookup, cached per game data.
#[derive(Debug)]
pub struct StoryIndexCache {
    pub index: StoryIndex,
    pub lookup: HashMap<String, StoryRef>,
    /// The inverse of `lookup`: script path (`StoryTxt`) -> story id.
    ///
    /// The game's own "this story was played" record is a set of script
    /// PATHS (`user.status.flags`), not story ids, so the refresh needs this
    /// direction. It is built here, once per game-data load, because
    /// inverting 1,887 entries on every account refresh is work the cache
    /// already owns. First writer wins, which is the library group rather
    /// than the operator record when a path is listed under both.
    pub by_txt: HashMap<String, String>,
    /// Story id -> the stage gates the review table puts on it
    /// (`RequiredStages`). The refresh reads the account's `dungeon.stages`
    /// against these to tell a story the game PLAYED for the player (its
    /// stage was entered or cleared) from one the Archive merely unlocked.
    pub gates: HashMap<String, Vec<StoryRequiredStage>>,
    /// Wall time of the build, probe included.
    pub build_ms: u128,
    /// Wall time of the script probe alone.
    pub probe_ms: u128,
    /// Wall time of the story asset index the derived covers resolve
    /// through, 0 when an earlier build already cached it.
    pub story_assets_ms: u128,
    /// Per group AND per operator `charId`, the distinct illustration names
    /// its stories reference. The two id spaces cannot collide: every
    /// operator id starts with `char_` and 0 of the 451 EN group ids do.
    pub illustrations: HashMap<String, GroupRefs>,
    /// The event archives, keyed by library group id and built ALREADY
    /// RESOLVED. It is cached rather than rebuilt per request because the
    /// whole thing is 5 groups and 300 items on EN, and because its art is
    /// resolved by the same rule `coverUrl` and `bannerUrl` are: at build
    /// time, off game data the cache is keyed by.
    pub archives: HashMap<String, StoryArchive>,
    /// Wall time of the archive build, the art walk included.
    pub archive_ms: u128,
}

fn category_for(entry_type: &str, act_type: &str, first_txt: &str) -> StoryCategory {
    if first_txt.starts_with("obt/rogue") || first_txt.starts_with("obt/roguelike") {
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

/// `main_15` has no zone of its own: its stages sit in `act2mainss_zone1`,
/// so the zone is read off the group's first required stage when the id
/// itself is not a zone.
fn zone_id_for(
    gd: &GameData,
    group: &crate::core::gamedata::types::story_review::StoryReviewGroup,
) -> Option<String> {
    if gd.zones.contains_key(&group.id) {
        return Some(group.id.clone());
    }
    group
        .info_unlock_datas
        .iter()
        .flat_map(|s| s.required_stages.iter())
        .find_map(|rs| gd.stages.get(&rs.stage_id).map(|st| st.zone_id.clone()))
}

fn zone_for(gd: &GameData, zone_id: &str) -> Option<StoryZone> {
    let zone_id = zone_id.to_owned();
    let zone = gd.zones.get(&zone_id)?;
    let chapter_id = gd.zone_chapters.get(&zone_id).cloned().or_else(|| {
        // Fallback: the numeric `main_N` range of `chapter_table`.
        let n: i32 = zone_id.strip_prefix("main_")?.parse().ok()?;
        gd.chapters.values().find_map(|c| {
            let lo: i32 = c.start_zone_id.strip_prefix("main_")?.parse().ok()?;
            let hi: i32 = c.end_zone_id.strip_prefix("main_")?.parse().ok()?;
            (lo <= n && n <= hi).then(|| c.chapter_id.clone())
        })
    });
    Some(StoryZone {
        chapter_name: chapter_id
            .and_then(|c| gd.chapters.get(&c))
            .map(|c| c.chapter_name.clone()),
        name_first: zone.zone_name_first.clone(),
        name_second: zone.zone_name_second.clone(),
        name_third: zone.zone_name_third.clone(),
    })
}

/// The chapter number a reader knows the group by: `zone_table`'s
/// `ZoneNameTitleCurrent`, the "00".."16" the client prints under the word
/// EPISODE, falling back to the numeric tail of a `main_N` id. `main_15` and
/// `main_16` have no zone of their own, so the fallback is what answers for
/// them when the zone lookup misses; their zones (`act2mainss_zone1`,
/// `act3mainss_zone1`) do carry "15" and "16" and are preferred.
fn chapter_number_for(gd: &GameData, group_id: &str, zone_id: Option<&str>) -> Option<u32> {
    zone_id
        .and_then(|z| gd.zones.get(z))
        .and_then(|z| z.zone_name_title_current.as_deref())
        .and_then(|t| t.trim().parse::<u32>().ok())
        .or_else(|| group_id.strip_prefix("main_")?.parse().ok())
}

/// Build the index from loaded game data. Pure over its inputs plus the
/// script probe on `assets_dir`; the route caches the result.
#[must_use]
pub fn build_index(
    gd: &GameData,
    assets: &Arc<AssetIndex>,
    assets_dir: &std::path::Path,
) -> StoryIndexCache {
    let started = Instant::now();
    // The same index `GET /story/{id}` resolves through, built here so the
    // warm pass pays for it once instead of the first reader paying for it.
    let assets_started = Instant::now();
    let story_assets = StoryAssetIndex::for_dir(assets_dir, assets);
    let story_assets_ms = assets_started.elapsed().as_millis();
    let mut lookup: HashMap<String, StoryRef> = HashMap::new();
    let mut by_txt: HashMap<String, String> = HashMap::new();
    let mut gates: HashMap<String, Vec<StoryRequiredStage>> = HashMap::new();
    let mut probe_time = std::time::Duration::ZERO;
    // One read per DISTINCT `StoryTxt`. The read is now the whole file plus a
    // parse rather than a 16-byte head, because the word count is computed
    // here, ONCE, instead of on every `GET /story/{id}`: on EN that is 1,887
    // distinct paths over 2,254 calls (1,887 library stories plus 367 record
    // stories, which the 364 `record` groups already list), so the memo saves
    // 367 loads and the pass costs 5,793 ms where the probe cost 53 ms, both
    // measured in a debug build.
    let mut measured: HashMap<String, Option<ScriptFacts>> = HashMap::new();
    let mut probe = |story_txt: &str| -> Option<ScriptFacts> {
        if let Some(hit) = measured.get(story_txt) {
            return hit.clone();
        }
        let t = Instant::now();
        let facts = story::script_facts(assets_dir, story_txt, &story_assets);
        probe_time += t.elapsed();
        measured.insert(story_txt.to_owned(), facts.clone());
        facts
    };

    let mut illustrations: HashMap<String, GroupRefs> = HashMap::new();
    // The game's own Story Collection art, joined the same way the shelves
    // are: through `StorylineStorySets`, never by group id.
    let story_sets = story_sets_by_group(gd);
    let mut chapter_numbers: HashMap<String, u32> = HashMap::new();
    let mut groups: Vec<StoryGroup> = Vec::with_capacity(gd.story_reviews.len());
    for g in gd.story_reviews.values() {
        let first_txt = g
            .info_unlock_datas
            .iter()
            .map(|s| s.story_txt.as_str())
            .find(|t| !t.is_empty())
            .unwrap_or("");
        let category = category_for(&g.entry_type, &g.act_type, first_txt);
        // Story id -> the name of its first background, for the cover below.
        let mut first_bgs: HashMap<String, String> = HashMap::new();
        let mut refs = GroupRefsBuilder::default();
        let mut stories: Vec<StoryEntry> = g
            .info_unlock_datas
            .iter()
            .map(|s| {
                lookup
                    .entry(s.story_id.clone())
                    .or_insert_with(|| StoryRef {
                        story_txt: s.story_txt.clone(),
                        name: s.story_name.clone(),
                        group_id: g.id.clone(),
                    });
                if !s.story_txt.is_empty() {
                    by_txt
                        .entry(s.story_txt.clone())
                        .or_insert_with(|| s.story_id.clone());
                }
                gates
                    .entry(s.story_id.clone())
                    .or_insert_with(|| s.required_stages.clone());
                let facts = probe(&s.story_txt);
                if let Some(bg) = facts.as_ref().and_then(|f| f.first_background.clone()) {
                    first_bgs.insert(s.story_id.clone(), bg);
                }
                if let Some(f) = facts.as_ref() {
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
        stories.sort_by(|a, b| a.sort.cmp(&b.sort).then_with(|| a.id.cmp(&b.id)));
        let word_count = stories.iter().map(|s| s.word_count).sum();
        // A mainline group's own `StartTime` is -1 for all 17 EN chapters, so
        // the release date comes off its ZONE when the zone carries one.
        let zone_id = (category == StoryCategory::Main)
            .then(|| zone_id_for(gd, g))
            .flatten();
        let start_time = zone_id
            .as_ref()
            .and_then(|z| gd.zone_open_times.get(z).copied())
            .filter(|t| *t > 0)
            .unwrap_or(g.start_time);
        let group_refs = refs.finish();
        let illustration_count = group_refs.illustration_count();
        let sprite_count = group_refs.sprite_count();
        illustrations.insert(g.id.clone(), group_refs);
        // The game's own Archives picture, else the first background of the
        // first story that HAS a script, in the order the group lists them.
        let (cover_url, cover_kind) = cover_for(assets, g.story_entry_pic_id.as_deref())
            .map_or_else(
                || {
                    let derived = stories
                        .iter()
                        .filter(|s| s.has_script)
                        .find_map(|s| first_bgs.get(&s.id))
                        .and_then(|name| story_assets.resolve_background(name))
                        .map(|(url, _)| url);
                    (derived.clone(), derived.map(|_| StoryCoverKind::Background))
                },
                |url| (Some(url), Some(StoryCoverKind::EntryPic)),
            );
        let set = story_sets.get(&g.id).copied();
        let banner_url = storyline_art_for(assets, set.and_then(|s| s.kv_image_id.as_deref()));
        let title_image_url =
            storyline_art_for(assets, set.and_then(|s| s.title_image_id.as_deref()));
        let icon_url = storyline_art_for(
            assets,
            set.and_then(|s| s.mainline_data.as_ref())
                .and_then(|m| m.deco_image_id.as_deref()),
        );
        let chapter_number = (category == StoryCategory::Main)
            .then(|| chapter_number_for(gd, &g.id, zone_id.as_deref()))
            .flatten();
        if let Some(n) = chapter_number {
            chapter_numbers.insert(g.id.clone(), n);
        }
        groups.push(StoryGroup {
            id: g.id.clone(),
            name: g.name.clone(),
            category,
            entry_type: g.entry_type.clone(),
            act_type: g.act_type.clone(),
            display_type: gd
                .activities
                .get(&g.id)
                .map(|a| a.display_type.clone())
                .filter(|d| !d.is_empty()),
            cover_url,
            cover_kind,
            banner_url,
            title_image_url,
            icon_url,
            chapter_number,
            start_time,
            zone: zone_id.as_deref().and_then(|z| zone_for(gd, z)),
            word_count,
            illustration_count,
            sprite_count,
            music: group_music(gd, &story_assets, &g.id, category),
            stories,
        });
    }
    // Main chapters in zone order, everything else by release then id.
    groups.sort_by(|a, b| {
        let ka = StoryCategory::ALL.iter().position(|c| *c == a.category);
        let kb = StoryCategory::ALL.iter().position(|c| *c == b.category);
        ka.cmp(&kb)
            .then_with(|| main_ordinal(&a.id).cmp(&main_ordinal(&b.id)))
            .then_with(|| a.start_time.cmp(&b.start_time))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut records: Vec<OperatorRecordGroup> = Vec::new();
    let mut handbook: Vec<_> = gd.handbook.handbook_dict.iter().collect();
    handbook.sort_by(|a, b| a.0.cmp(b.0));
    for (char_id, item) in handbook {
        let mut stories: Vec<StoryEntry> = Vec::new();
        let mut refs = GroupRefsBuilder::default();
        for set in &item.handbook_avg_list {
            for avg in &set.avg_list {
                let name = lookup
                    .get(&avg.story_id)
                    .map_or_else(|| set.story_set_name.clone(), |r| r.name.clone());
                lookup
                    .entry(avg.story_id.clone())
                    .or_insert_with(|| StoryRef {
                        story_txt: avg.story_txt.clone(),
                        name: name.clone(),
                        group_id: set.story_set_id.clone(),
                    });
                if !avg.story_txt.is_empty() {
                    by_txt
                        .entry(avg.story_txt.clone())
                        .or_insert_with(|| avg.story_id.clone());
                }
                let facts = probe(&avg.story_txt);
                if let Some(f) = facts.as_ref() {
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
        stories.sort_by(|a, b| a.sort.cmp(&b.sort).then_with(|| a.id.cmp(&b.id)));
        let op = gd.operators.get(char_id);
        let record_refs = refs.finish();
        let illustration_count = record_refs.illustration_count();
        let sprite_count = record_refs.sprite_count();
        illustrations.insert(char_id.clone(), record_refs);
        records.push(OperatorRecordGroup {
            char_id: char_id.clone(),
            name: op.map_or_else(|| char_id.clone(), |o| o.name.clone()),
            rarity: op.map_or(0, |o| rarity_to_stars(&o.rarity)),
            profession: op.map_or(OperatorProfession::Unknown, |o| o.profession.clone()),
            avatar_url: avatar_for(assets, char_id).unwrap_or_default(),
            word_count: stories.iter().map(|s| s.word_count).sum(),
            illustration_count,
            sprite_count,
            stories,
        });
    }

    // Over DISTINCT ids: a record story is listed both in its `record` group
    // and under its operator.
    let totals = {
        let mut seen: HashSet<&str> = HashSet::new();
        let mut totals = StoryTotals::default();
        for e in groups
            .iter()
            .flat_map(|g| &g.stories)
            .chain(records.iter().flat_map(|r| &r.stories))
        {
            if !seen.insert(e.id.as_str()) {
                continue;
            }
            totals.stories += 1;
            if e.has_script {
                totals.with_script += 1;
                totals.words += e.word_count;
            }
        }
        totals
    };

    let archive_started = Instant::now();
    let archives = build_archives(gd, &story_assets, assets_dir);
    let archive_ms = archive_started.elapsed().as_millis();

    StoryIndexCache {
        index: StoryIndex {
            groups,
            records,
            storylines: build_storylines(gd, assets, &chapter_numbers),
            totals,
        },
        lookup,
        by_txt,
        gates,
        illustrations,
        archives,
        archive_ms,
        build_ms: started.elapsed().as_millis(),
        probe_ms: probe_time.as_millis(),
        story_assets_ms,
    }
}

/// Zone id -> the `story_review_table` group whose stages sit in it. Built
/// for the two mainline story sets whose `RelevantActivityId` is an ACTIVITY
/// id rather than a group id (`act2mainss` -> `main_15`, `act3mainss` ->
/// `main_16`, whose zones are `act2mainss_zone1` and `act3mainss_zone1`).
fn zone_owner_index(gd: &GameData) -> HashMap<&str, &str> {
    let mut out: HashMap<&str, &str> = HashMap::new();
    for g in gd.story_reviews.values() {
        for stage_id in g
            .info_unlock_datas
            .iter()
            .flat_map(|s| s.required_stages.iter())
            .map(|r| r.stage_id.as_str())
        {
            if let Some(stage) = gd.stages.get(stage_id) {
                out.entry(stage.zone_id.as_str()).or_insert(g.id.as_str());
            }
        }
    }
    out
}

/// The inclusive run of mainline chapter numbers a list of groups covers, or
/// `None` when not one of them is a numbered chapter.
fn chapter_range(
    group_ids: &[String],
    chapter_numbers: &HashMap<String, u32>,
) -> Option<ChapterRange> {
    let mut it = group_ids
        .iter()
        .filter_map(|g| chapter_numbers.get(g).copied());
    let first = it.next()?;
    let (from, to) = it.fold((first, first), |(lo, hi), n| (lo.min(n), hi.max(n)));
    Some(ChapterRange { from, to })
}

/// The `story_review_table` group a storyline story set names: the set's own
/// id when the Archives carry it, else the group that owns the `{id}_zone*`
/// zone (`act2mainss` -> `main_15`, `act3mainss` -> `main_16`).
fn resolve_group(gd: &GameData, zone_owner: &HashMap<&str, &str>, id: &str) -> Option<String> {
    if gd.story_reviews.contains_key(id) {
        return Some(id.to_owned());
    }
    let prefix = format!("{id}_");
    zone_owner
        .iter()
        .filter(|(zone, _)| zone.starts_with(&prefix))
        .map(|(_, group)| (*group).to_owned())
        .min()
}

/// Group id -> the storyline story set that names it, which is where the
/// group's key visual (`KvImageId`) and, on a mainline chapter, its glyph
/// (`MainlineData.DecoImageId`) live. 81 EN sets over 81 distinct groups;
/// the sets are walked in `StorySetId` order so a tie would resolve the same
/// way on every build.
fn story_sets_by_group(gd: &GameData) -> HashMap<String, &StorylineStorySet> {
    let zone_owner = zone_owner_index(gd);
    let mut sets: Vec<&StorylineStorySet> = gd.storyline_story_sets.values().collect();
    sets.sort_by(|a, b| a.story_set_id.cmp(&b.story_set_id));
    let mut out: HashMap<String, &StorylineStorySet> = HashMap::new();
    for set in sets {
        if let Some(group_id) = set
            .group_id()
            .and_then(|id| resolve_group(gd, &zone_owner, id))
        {
            out.entry(group_id).or_insert(set);
        }
    }
    out
}

/// The storylines as the wire carries them: shelves in `SortId` order,
/// locations in theirs, every location that joins to a group the Archives
/// list, deduplicated WITHIN a shelf.
fn build_storylines(
    gd: &GameData,
    assets: &AssetIndex,
    chapter_numbers: &HashMap<String, u32>,
) -> Vec<Storyline> {
    let zone_owner = zone_owner_index(gd);
    let resolve = |id: &str| resolve_group(gd, &zone_owner, id);
    let mut lines: Vec<&crate::core::gamedata::types::stage::Storyline> =
        gd.storylines.iter().collect();
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
            let mut group_ids: Vec<String> = Vec::new();
            let mut arcs: Vec<StorylineArc> = Vec::new();
            for location in locations {
                if let Some(split) = location.mainline_split_data.as_ref() {
                    let name = split.sub_name.clone().unwrap_or_default();
                    if !name.is_empty() {
                        arcs.push(StorylineArc {
                            name,
                            sort: location.sort_id,
                            icon_url: storyline_art_for(assets, split.icon_id.as_deref()),
                            group_ids: Vec::new(),
                            chapter_range: None,
                        });
                    }
                    continue;
                }
                let Some(group_id) = location
                    .relevant_story_set_id
                    .as_deref()
                    .and_then(|set_id| gd.storyline_story_sets.get(set_id))
                    .and_then(|set| set.group_id())
                    .and_then(&resolve)
                else {
                    continue;
                };
                if let Some(arc) = arcs.last_mut() {
                    arc.group_ids.push(group_id.clone());
                }
                if !group_ids.contains(&group_id) {
                    group_ids.push(group_id);
                }
            }
            for arc in &mut arcs {
                arc.chapter_range = chapter_range(&arc.group_ids, chapter_numbers);
            }
            Storyline {
                id: line.storyline_id.clone(),
                name: line.storyline_name.clone(),
                sort: line.sort_id,
                icon_url: storyline_art_for(assets, line.storyline_icon_id.as_deref())
                    .or_else(|| storyline_art_for(assets, line.storyline_logo_id.as_deref())),
                logo_url: storyline_art_for(assets, line.storyline_logo_id.as_deref()),
                chapter_range: chapter_range(&group_ids, chapter_numbers),
                group_ids,
                arcs,
            }
        })
        .collect()
}

fn main_ordinal(id: &str) -> i32 {
    id.strip_prefix("main_")
        .and_then(|n| n.parse().ok())
        .unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_derive_from_entry_and_act_type_then_prefix() {
        assert_eq!(
            category_for("MAINLINE", "MAIN_STORY", "obt/main/x"),
            StoryCategory::Main
        );
        assert_eq!(
            category_for("ACTIVITY", "ACTIVITY_STORY", "activities/a/x"),
            StoryCategory::Side
        );
        assert_eq!(
            category_for("MINI_ACTIVITY", "MINI_STORY", "activities/a/x"),
            StoryCategory::Vignette
        );
        assert_eq!(
            category_for("NONE", "NONE", "obt/memory/x"),
            StoryCategory::Record
        );
        assert_eq!(
            category_for("NONE", "NONE", "obt/rogue/x"),
            StoryCategory::Is
        );
        assert_eq!(
            category_for("NONE", "NONE", "obt/roguelike/x"),
            StoryCategory::Is
        );
        assert_eq!(
            category_for("NONE", "NONE", "obt/sandboxperm/x"),
            StoryCategory::Reclamation
        );
        assert_eq!(
            category_for("NONE", "NONE", "obt/legion/x"),
            StoryCategory::SideContent
        );
        assert_eq!(StoryCategory::SideContent.as_str(), "sideContent");
        assert_eq!(
            serde_json::to_string(&StoryCategory::SideContent).unwrap(),
            "\"sideContent\""
        );
    }
}
