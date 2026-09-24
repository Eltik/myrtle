//! The story reader against the real EN tree: every library story and every
//! operator record is probed, loaded, parsed and asset-resolved, and the
//! counts the phase 1 report quotes are printed. Skips when the EN assets are
//! not on disk (CI's `game-data` artifact carries tables only, no scripts).
//!
//! Run with `cargo test --test story_real_data_test -- --nocapture`.

mod common;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use backend::app::error::ApiError;
use backend::app::services::story::{
    StoryCategory, StoryCoverKind, build_index, group_illustrations, load_and_parse,
};
use backend::core::gamedata::assets::AssetIndex;
use backend::core::gamedata::types::operator::OperatorProfession;
use backend::core::story::{self, StoryAssetIndex, parser};

fn assets_dir() -> PathBuf {
    let dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    Path::new(env!("CARGO_MANIFEST_DIR")).join(dir)
}

fn scripts_present(dir: &Path) -> bool {
    dir.join("gamedata/story/story_variables.json.json")
        .exists()
        && dir.join("gamedata/excel/story_review_table.json").exists()
}

#[test]
fn every_en_story_parses_and_the_library_resolves() {
    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!(
            "story_real_data_test: no EN story tree at {}, skipping",
            dir.display()
        );
        return;
    }
    let gd = common::load_game_data();
    assert!(
        !gd.story_reviews.is_empty(),
        "story_review_table loaded 0 groups; warnings: {:?}",
        gd.table_warnings
    );
    let asset_index = Arc::new(AssetIndex::build(&dir));

    // The library.
    let cache = build_index(gd, &asset_index, &dir);
    let index = &cache.index;
    let total_stories: usize = index.groups.iter().map(|g| g.stories.len()).sum();
    let with_script: usize = index
        .groups
        .iter()
        .flat_map(|g| g.stories.iter())
        .filter(|s| s.has_script)
        .count();
    let groups_with_any: usize = index
        .groups
        .iter()
        .filter(|g| g.stories.iter().any(|s| s.has_script))
        .count();
    println!(
        "index: {} groups, {} stories, {} with a script ({} groups with at least one); {} operator record groups, {} record stories; build {} ms of which probe {} ms",
        index.groups.len(),
        total_stories,
        with_script,
        groups_with_any,
        index.records.len(),
        index.records.iter().map(|r| r.stories.len()).sum::<usize>(),
        cache.build_ms,
        cache.probe_ms
    );
    let mut per_category: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for g in &index.groups {
        let e = per_category.entry(g.category.as_str()).or_default();
        e.0 += 1;
        e.1 += g.stories.len();
    }
    for c in StoryCategory::ALL {
        let (groups, stories) = per_category.get(c.as_str()).copied().unwrap_or((0, 0));
        println!(
            "category {:<12} {:>4} groups {:>5} stories",
            c.as_str(),
            groups,
            stories
        );
    }
    // The game's own themed shelves, and what they cover.
    {
        let known: HashSet<&str> = index.groups.iter().map(|g| g.id.as_str()).collect();
        let mut shelved: HashSet<&str> = HashSet::new();
        let mut slots = 0usize;
        assert!(
            !index.storylines.is_empty(),
            "stage_table carried no Storylines; warnings: {:?}",
            gd.table_warnings
        );
        for line in &index.storylines {
            for id in &line.group_ids {
                assert!(
                    known.contains(id.as_str()),
                    "storyline {} names {id}, which is not a group in the index",
                    line.id
                );
                shelved.insert(id.as_str());
                slots += 1;
            }
            let arc_groups: usize = line.arcs.iter().map(|a| a.group_ids.len()).sum();
            println!(
                "storyline {:>2} {:<10} {:<28} {:>3} groups {:>2} arcs ({arc_groups} grouped)",
                line.sort,
                line.id,
                line.name,
                line.group_ids.len(),
                line.arcs.len(),
            );
        }
        let non_record = index
            .groups
            .iter()
            .filter(|g| g.category != StoryCategory::Record)
            .count();
        let unshelved: Vec<&str> = index
            .groups
            .iter()
            .filter(|g| g.category != StoryCategory::Record && !shelved.contains(g.id.as_str()))
            .map(|g| g.id.as_str())
            .collect();
        println!(
            "storylines: {} shelves, {slots} slots over {} distinct groups; {non_record} non-record groups, {} on no shelf ({unshelved:?})",
            index.storylines.len(),
            shelved.len(),
            unshelved.len(),
        );
        // Every shelved group is a real story group, never an operator record set.
        for id in &shelved {
            let g = index.groups.iter().find(|g| g.id == *id).expect("shelved");
            assert_ne!(
                g.category,
                StoryCategory::Record,
                "storyline shelf holds record group {id}"
            );
        }
        // The mainline shelf is the only one the game splits into arcs, and
        // every arc it declares carries at least one group.
        let mainline = index
            .storylines
            .iter()
            .find(|l| l.id == "mainLine")
            .expect("the mainLine storyline");
        assert!(
            !mainline.arcs.is_empty(),
            "mainLine declares no arcs, so the main story cannot be sectioned"
        );
        for arc in &mainline.arcs {
            assert!(
                !arc.name.is_empty() && !arc.group_ids.is_empty(),
                "empty arc {arc:?} on mainLine"
            );
        }
        for line in index.storylines.iter().filter(|l| l.id != "mainLine") {
            assert!(
                line.arcs.is_empty(),
                "{} declares arcs; only mainLine was expected to",
                line.id
            );
        }
    }

    // The Story Collection's art: the key visual, the chapter glyph, the shelf
    // glyph, and the number a reader knows a chapter by. Every emitted URL is
    // checked on disk, because a served path that 404s is worse than none.
    {
        let on_disk = |url: &str| dir.join(url.trim_start_matches('/')).exists();
        let banners: Vec<&str> = index
            .groups
            .iter()
            .filter_map(|g| g.banner_url.as_deref())
            .collect();
        let icons: Vec<&str> = index
            .groups
            .iter()
            .filter_map(|g| g.icon_url.as_deref())
            .collect();
        let title_images: Vec<&str> = index
            .groups
            .iter()
            .filter_map(|g| g.title_image_url.as_deref())
            .collect();
        let numbered: Vec<u32> = index
            .groups
            .iter()
            .filter_map(|g| g.chapter_number)
            .collect();
        println!(
            "story collection art: {} banners, {} group icons, {} chapter numbers",
            banners.len(),
            icons.len(),
            numbered.len()
        );
        for url in banners
            .iter()
            .chain(icons.iter())
            .chain(title_images.iter())
        {
            assert!(on_disk(url), "group art {url} is not on disk");
        }
        // The title logotype rides with the key visual: same 81 sets, same
        // join, so a group carries both or neither.
        assert_eq!(
            title_images.len(),
            81,
            "title logotypes over the whole index"
        );
        for g in &index.groups {
            assert_eq!(
                g.banner_url.is_some(),
                g.title_image_url.is_some(),
                "group {} carries a key visual without its title art, or the other way round",
                g.id
            );
        }
        println!(
            "story collection art: {} title logotypes",
            title_images.len()
        );
        // The shelves list 81 sets on EN, so 81 groups carry a key visual and
        // all 17 mainline chapters are among them.
        assert_eq!(banners.len(), 81, "key visuals over the whole index");
        assert_eq!(
            icons.len(),
            17,
            "chapter glyphs; MAINLINE sets carry the only decos"
        );
        let main: Vec<&_> = index
            .groups
            .iter()
            .filter(|g| g.category == StoryCategory::Main)
            .collect();
        assert_eq!(main.len(), 17, "mainline groups");
        let mut seen: Vec<u32> = main.iter().filter_map(|g| g.chapter_number).collect();
        seen.sort_unstable();
        assert_eq!(
            seen,
            (0..=16).collect::<Vec<u32>>(),
            "every mainline chapter numbers itself 0..16"
        );
        assert_eq!(
            numbered.len(),
            17,
            "only the mainline chapters carry a chapter number"
        );
        for g in &main {
            assert!(
                g.banner_url.is_some() && g.icon_url.is_some(),
                "mainline chapter {} has no key visual or no glyph",
                g.id
            );
            println!(
                "  chapter {:>2} {:<8} banner {:?} icon {:?}",
                g.chapter_number.unwrap(),
                g.id,
                g.banner_url
                    .as_deref()
                    .map(|u| u.rsplit('/').next().unwrap()),
                g.icon_url.as_deref().map(|u| u.rsplit('/').next().unwrap()),
            );
        }
        // A banner is NOT a cover: where both exist they are different files.
        let both = index
            .groups
            .iter()
            .filter(|g| g.banner_url.is_some() && g.cover_url.is_some())
            .count();
        let same = index
            .groups
            .iter()
            .filter(|g| g.banner_url.is_some() && g.banner_url == g.cover_url)
            .count();
        println!("{both} groups carry both a cover and a banner, {same} of them the same file");
        assert_eq!(
            same, 0,
            "a banner that is only the cover again carries nothing new"
        );

        let shelf_icons = index
            .storylines
            .iter()
            .filter(|l| l.icon_url.is_some())
            .count();
        for line in &index.storylines {
            if let Some(url) = line.icon_url.as_deref() {
                assert!(on_disk(url), "storyline art {url} is not on disk");
            }
            println!(
                "  shelf {:<10} icon {:?} chapters {:?}",
                line.id,
                line.icon_url
                    .as_deref()
                    .map(|u| u.rsplit('/').next().unwrap()),
                line.chapter_range,
            );
        }
        assert_eq!(
            shelf_icons,
            index.storylines.len(),
            "every shelf draws a glyph, mainLine through its logo"
        );
        let mainline = index
            .storylines
            .iter()
            .find(|l| l.id == "mainLine")
            .expect("the mainLine storyline");
        assert_eq!(
            mainline.chapter_range.map(|r| (r.from, r.to)),
            Some((0, 16)),
            "the mainline shelf spans chapters 0 to 16"
        );
        for arc in &mainline.arcs {
            let range = arc
                .chapter_range
                .unwrap_or_else(|| panic!("mainline arc {} spans no chapter", arc.name));
            assert!(
                range.from <= range.to,
                "backwards arc range on {}",
                arc.name
            );
            let url = arc
                .icon_url
                .as_deref()
                .unwrap_or_else(|| panic!("mainline arc {} draws no icon", arc.name));
            assert!(on_disk(url), "arc icon {url} is not on disk");
            println!(
                "  arc {:<24} {:?} chapters {} to {}",
                arc.name,
                url.rsplit('/').next().unwrap(),
                range.from,
                range.to
            );
        }
        // A themed shelf may hold mainline chapters too (`ssLine_1` holds
        // `main_7`..`main_14`), so a range off `mainLine` is real, not a bug;
        // it is absent only where the shelf holds no numbered chapter.
        for line in &index.storylines {
            let has_chapter = line.group_ids.iter().any(|id| {
                index
                    .groups
                    .iter()
                    .any(|g| &g.id == id && g.chapter_number.is_some())
            });
            assert_eq!(
                line.chapter_range.is_some(),
                has_chapter,
                "{} disagrees with its own groups about holding a chapter",
                line.id
            );
        }
    }

    let covers = index
        .groups
        .iter()
        .filter(|g| g.cover_url.is_some())
        .count();
    let entry_pics = index
        .groups
        .iter()
        .filter(|g| g.cover_kind == Some(StoryCoverKind::EntryPic))
        .count();
    let derived = index
        .groups
        .iter()
        .filter(|g| g.cover_kind == Some(StoryCoverKind::Background))
        .count();
    let coverless: Vec<&str> = index
        .groups
        .iter()
        .filter(|g| g.cover_kind.is_none())
        .map(|g| g.id.as_str())
        .collect();
    println!(
        "covers by kind: {entry_pics} entryPic, {derived} background, {} none ({coverless:?})",
        coverless.len()
    );
    assert_eq!(
        entry_pics + derived + coverless.len(),
        index.groups.len(),
        "a group carries neither a cover kind nor none of one"
    );
    for g in &index.groups {
        assert_eq!(
            g.cover_url.is_some(),
            g.cover_kind.is_some(),
            "group {} has a cover url without a kind, or the other way round",
            g.id
        );
    }
    // A group with no cover is a group with no scripted story at all: there is
    // no background to derive one from.
    for id in &coverless {
        let g = index.groups.iter().find(|g| &g.id == id).unwrap();
        assert!(
            !g.stories.iter().any(|s| s.has_script),
            "group {id} has a scripted story but no cover"
        );
    }
    for g in index
        .groups
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
    {
        println!(
            "  cover {:<8} {:?} {:?}",
            g.id,
            g.cover_kind,
            g.cover_url
                .as_deref()
                .map(|u| u.rsplit('/').next().unwrap())
        );
    }
    let zones = index.groups.iter().filter(|g| g.zone.is_some()).count();
    let chapters = index
        .groups
        .iter()
        .filter(|g| g.zone.as_ref().is_some_and(|z| z.chapter_name.is_some()))
        .count();
    let display_types: BTreeMap<String, usize> = index
        .groups
        .iter()
        .filter_map(|g| g.display_type.clone())
        .fold(BTreeMap::new(), |mut m, d| {
            *m.entry(d).or_default() += 1;
            m
        });
    println!(
        "covers {covers}, main groups with a zone {zones}, with a chapter name {chapters}, display types {display_types:?}"
    );
    for g in index
        .groups
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
    {
        let z = g.zone.as_ref();
        println!(
            "  main {:<8} {:<40} zone first={:?} chapter={:?}",
            g.id,
            g.name,
            z.and_then(|z| z.name_first.as_deref()),
            z.and_then(|z| z.chapter_name.as_deref())
        );
    }
    assert_eq!(index.groups.len(), gd.story_reviews.len());
    assert!(
        with_script >= 1797,
        "expected at least 1797 stories with a script, got {with_script}"
    );
    let main_groups = index
        .groups
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
        .count();
    assert_eq!(zones, main_groups, "every main group should carry a zone");

    // Every script: probe, load, parse, resolve.
    let story_assets = StoryAssetIndex::for_dir(&dir, &asset_index);
    let (bg_n, img_n, spr_n, aud_n) = story_assets.counts();
    println!(
        "story asset index: {} backgrounds, {} images, {} sprite folders, {} audio clips, {} variables, built in {} ms",
        bg_n,
        img_n,
        spr_n,
        aud_n,
        story_assets.variables().len(),
        story_assets.build_ms
    );

    let mut ids: Vec<&String> = cache.lookup.keys().collect();
    ids.sort();
    let mut kinds: HashMap<String, usize> = HashMap::new();
    let mut parsed = 0usize;
    let mut missing = 0usize;
    let mut not_a_script = 0usize;
    let mut slowest: (u128, String, usize) = (0, String::new(), 0);
    let mut total_parse_us = 0u128;
    let mut total_lines = 0usize;
    let mut refs_bg: BTreeMap<String, bool> = BTreeMap::new();
    let mut refs_img: BTreeMap<String, bool> = BTreeMap::new();
    // name -> None when unresolved, Some(has a face patch) when resolved.
    let mut refs_chr_faces: BTreeMap<String, Option<bool>> = BTreeMap::new();
    // name -> whether the wire carries a `facePos` for it.
    let mut refs_chr_pos: BTreeMap<String, bool> = BTreeMap::new();
    // `"1280x1280"` -> how many distinct names resolve to a body that size.
    let mut body_sizes: BTreeMap<String, usize> = BTreeMap::new();
    // Names that carry a placement with no texture size to divide it by: the
    // patch would have nothing to scale against, so this must stay empty.
    let mut pos_without_size: Vec<String> = Vec::new();
    let mut bodies_sized = 0usize;
    let mut bodies_unsized: Vec<String> = Vec::new();
    // The character's own prefab plate, which beats the 1024 slot template.
    let mut plates_sized = 0usize;
    let mut plateless: Vec<String> = Vec::new();
    // `"1090x1090"` -> how many distinct names are drawn at that plate.
    let mut plate_sizes: BTreeMap<String, usize> = BTreeMap::new();
    // Every referenced background/CG name -> true when its `imageSizes` entry
    // assumed the pixels-per-unit (read off the PNG header) rather than read
    // it from a `sprites.json`.
    let mut image_size_assumed: BTreeMap<String, bool> = BTreeMap::new();
    // Referenced names the wire carries a URL for and NO size, which must
    // stay empty: the frontend cannot draw a plate it cannot measure.
    let mut sizeless: BTreeMap<String, &'static str> = BTreeMap::new();
    let mut ppu_values: BTreeMap<String, usize> = BTreeMap::new();
    let mut bg_cher_1: Option<backend::core::story::ImageSize> = None;
    // Sprite folders whose face patch carries no placement, for the report.
    let mut patchless_folders: BTreeMap<String, usize> = BTreeMap::new();
    // Parser parity, measured over the same corpus.
    let mut continued_lines = 0u64;
    let mut decoded_values = 0u64;
    let mut dropped_arg_blocks = 0u64;
    let mut typo_lines = 0u64;
    let mut escape_stories: BTreeMap<String, u32> = BTreeMap::new();
    let mut refs_mus: BTreeMap<String, bool> = BTreeMap::new();
    let mut refs_snd: BTreeMap<String, bool> = BTreeMap::new();
    let mut refs_avt: BTreeMap<String, bool> = BTreeMap::new();
    let mut ambiguous: HashSet<String> = HashSet::new();
    let mut words_total = 0u64;
    // Every distinct URL the wire `StoryAssets` maps would emit, over every
    // story, with the kind that emitted it. Checked once each against the
    // assets root: a miss here is a request the reader would fire and fail.
    let mut emitted_urls: BTreeMap<String, &'static str> = BTreeMap::new();

    // The index's per-story count, to check every one of them against the
    // parse below rather than only their sums.
    let index_words: HashMap<&str, u32> = index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .chain(index.records.iter().flat_map(|r| &r.stories))
        .map(|s| (s.id.as_str(), s.word_count))
        .collect();

    for id in ids {
        let story_ref = &cache.lookup[id];
        let script = match story::load_script(&dir, &story_ref.story_txt) {
            Ok(s) => s,
            Err(story::ScriptError::Missing { .. }) => {
                missing += 1;
                continue;
            }
            Err(story::ScriptError::NotAScript { .. }) => {
                not_a_script += 1;
                continue;
            }
            Err(e) => panic!("{id}: {e}"),
        };
        let t = Instant::now();
        let parsed_story = story::parse_story(
            id,
            &story_ref.name,
            &story_ref.group_id,
            &script,
            &story_assets,
        );
        let us = t.elapsed().as_micros();
        let stats = parser::parse_with_stats(&script).1;
        continued_lines += u64::from(stats.continued_lines);
        decoded_values += u64::from(stats.decoded_values);
        dropped_arg_blocks += u64::from(stats.dropped_arg_blocks);
        let invalid = stats
            .diagnostics
            .iter()
            .filter(|d| d.deviation == parser::Deviation::InvalidEscape)
            .count();
        typo_lines += stats
            .diagnostics
            .iter()
            .filter(|d| d.deviation == parser::Deviation::TypoFolded)
            .count() as u64;
        if invalid > 0 {
            escape_stories.insert(id.clone(), invalid as u32);
        }
        total_parse_us += us;
        total_lines += parsed_story.commands.len();
        if us > slowest.0 {
            slowest = (us, id.clone(), parsed_story.commands.len());
        }
        parsed += 1;
        words_total += u64::from(parsed_story.word_count);
        assert_eq!(
            index_words.get(id.as_str()).copied(),
            Some(parsed_story.word_count),
            "{id}: the index's word count is not the parse's"
        );
        for c in &parsed_story.commands {
            *kinds.entry(c.kind.clone()).or_default() += 1;
        }
        // The wire's own maps: every background and CG name it carries a URL
        // for must carry a size under the SAME key.
        for (name, kind) in parsed_story
            .assets
            .backgrounds
            .keys()
            .map(|k| (k, "background"))
            .chain(parsed_story.assets.images.keys().map(|k| (k, "image")))
        {
            match parsed_story.assets.image_sizes.get(name) {
                Some(size) => {
                    if image_size_assumed
                        .insert(name.clone(), size.ppu_assumed.unwrap_or(false))
                        .is_none()
                    {
                        *ppu_values.entry(format!("{:.4}", size.ppu)).or_default() += 1;
                    }
                    if name == "bg_cher_1" {
                        bg_cher_1 = Some(*size);
                    }
                }
                None => {
                    sizeless.insert(name.clone(), kind);
                }
            }
        }
        let refs = story::resolve_refs(&parsed_story.commands, &story_assets);
        for (k, v) in refs.backgrounds {
            if story_assets.image_is_ambiguous(&k) {
                ambiguous.insert(k.clone());
            }
            if let Some(url) = v.as_ref() {
                emitted_urls.insert(url.clone(), "background");
            }
            refs_bg.entry(k).or_insert(v.is_some());
        }
        for (k, v) in refs.images {
            if story_assets.image_is_ambiguous(&k) {
                ambiguous.insert(k.clone());
            }
            if let Some(url) = v.as_ref() {
                emitted_urls.insert(url.clone(), "image");
            }
            refs_img.entry(k).or_insert(v.is_some());
        }
        for (k, v) in refs.characters {
            if let Some(sprite) = v.as_ref() {
                emitted_urls.insert(sprite.body_url.clone(), "body");
                if let Some(face) = sprite.face_url.as_ref() {
                    emitted_urls.insert(face.clone(), "face");
                }
            }
            if let Some(sprite) = v.as_ref() {
                if refs_chr_pos
                    .insert(k.clone(), sprite.face_pos.is_some())
                    .is_none()
                {
                    match sprite.body_size {
                        Some(b) => {
                            bodies_sized += 1;
                            *body_sizes.entry(format!("{}x{}", b.w, b.h)).or_default() += 1;
                        }
                        None => bodies_unsized.push(k.clone()),
                    }
                    if sprite.face_pos.is_some() && sprite.body_size.is_none() {
                        pos_without_size.push(k.clone());
                    }
                    match sprite.plate {
                        Some(p) => {
                            plates_sized += 1;
                            *plate_sizes.entry(format!("{}x{}", p.w, p.h)).or_default() += 1;
                        }
                        None => plateless.push(k.clone()),
                    }
                }
                if sprite.face_url.is_some() && sprite.face_pos.is_none() {
                    let folder = sprite.body_url.rsplit('/').nth(1).unwrap_or("?").to_owned();
                    *patchless_folders.entry(folder).or_default() += 1;
                }
            }
            refs_chr_faces
                .entry(k)
                .or_insert(v.map(|s| s.face_url.is_some()));
        }
        for (k, v) in refs.music {
            if let Some(url) = v.as_ref() {
                emitted_urls.insert(url.clone(), "music");
            }
            refs_mus.entry(k).or_insert(v.is_some());
        }
        for (k, v) in refs.sounds {
            if let Some(url) = v.as_ref() {
                emitted_urls.insert(url.clone(), "sound");
            }
            refs_snd.entry(k).or_insert(v.is_some());
        }
        for (k, v) in refs.avatars {
            if let Some(url) = v.as_ref() {
                emitted_urls.insert(url.clone(), "avatar");
            }
            refs_avt.entry(k).or_insert(v.is_some());
        }
    }

    println!(
        "scripts: {} parsed, {} missing, {} summary-only, {} lookup ids; {} commands, {} words; parse total {} ms, mean {} us, slowest {} us ({}, {} commands)",
        parsed,
        missing,
        not_a_script,
        cache.lookup.len(),
        total_lines,
        words_total,
        total_parse_us / 1000,
        if parsed == 0 {
            0
        } else {
            total_parse_us / parsed as u128
        },
        slowest.0,
        slowest.1,
        slowest.2
    );
    assert!(parsed >= 1797, "parsed {parsed}");

    // The word counts the index computes ONCE at build, against the parse
    // just done story by story. `words_total` is summed over `cache.lookup`,
    // the distinct story ids, which is what `totals` counts.
    // Every scripted story counts words but ONE: `main_14-20_beg` is a
    // 296-byte cutscene, `[Video(res="video/02.mp4")]` between two `[Dialog]`
    // hides, with no prose line in it at all. 1,796 of 1,797 carry words.
    let zero_worded: Vec<&str> = index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .chain(index.records.iter().flat_map(|r| &r.stories))
        .filter(|s| s.has_script && s.word_count == 0)
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        zero_worded,
        ["main_14_level_main_14-20_beg"],
        "the scripted stories counting 0 words changed"
    );
    let unscripted_worded = index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .chain(index.records.iter().flat_map(|r| &r.stories))
        .filter(|s| !s.has_script && s.word_count != 0)
        .count();
    assert_eq!(unscripted_worded, 0, "a story with no script counts words");
    for g in &index.groups {
        assert_eq!(
            g.word_count,
            g.stories.iter().map(|s| s.word_count).sum::<u32>(),
            "group {} word count is not its stories' sum",
            g.id
        );
    }
    for r in &index.records {
        assert_eq!(
            r.word_count,
            r.stories.iter().map(|s| s.word_count).sum::<u32>(),
            "record group {} word count is not its stories' sum",
            r.char_id
        );
    }
    println!(
        "totals: {} stories, {} with a script, {} words (group sum {}, record sum {}); build {} ms of which the read+parse pass {} ms and the story asset index {} ms",
        index.totals.stories,
        index.totals.with_script,
        index.totals.words,
        index
            .groups
            .iter()
            .map(|g| u64::from(g.word_count))
            .sum::<u64>(),
        index
            .records
            .iter()
            .map(|r| u64::from(r.word_count))
            .sum::<u64>(),
        cache.build_ms,
        cache.probe_ms,
        cache.story_assets_ms
    );
    assert_eq!(
        index.totals.stories as usize,
        cache.lookup.len(),
        "totals.stories is not the distinct story count"
    );
    assert_eq!(
        index.totals.with_script as usize, parsed,
        "totals.withScript is not the number of scripts that parse"
    );
    assert_eq!(
        u64::from(index.totals.words),
        words_total,
        "totals.words is not the per-story sum"
    );

    // The record card's fields: every operator record carries a rarity, a
    // profession and an avatar the frontend can render under `/api/assets`.
    let no_avatar: Vec<&str> = index
        .records
        .iter()
        .filter(|r| r.avatar_url.is_empty())
        .map(|r| r.char_id.as_str())
        .collect();
    let no_rarity: Vec<&str> = index
        .records
        .iter()
        .filter(|r| r.rarity == 0)
        .map(|r| r.char_id.as_str())
        .collect();
    let unknown_profession: Vec<&str> = index
        .records
        .iter()
        .filter(|r| r.profession == OperatorProfession::Unknown)
        .map(|r| r.char_id.as_str())
        .collect();
    let mut by_rarity: BTreeMap<u8, usize> = BTreeMap::new();
    let mut by_profession: BTreeMap<String, usize> = BTreeMap::new();
    for r in &index.records {
        *by_rarity.entry(r.rarity).or_default() += 1;
        *by_profession
            .entry(r.profession.to_raw_str().to_owned())
            .or_default() += 1;
    }
    println!(
        "records: {} groups, {} with an avatar, rarities {:?}, professions {:?}",
        index.records.len(),
        index.records.len() - no_avatar.len(),
        by_rarity,
        by_profession
    );
    assert!(
        no_avatar.is_empty(),
        "records with no avatar: {no_avatar:?}"
    );
    assert!(
        no_rarity.is_empty(),
        "records with no rarity: {no_rarity:?}"
    );
    assert!(
        unknown_profession.is_empty(),
        "records with no profession: {unknown_profession:?}"
    );
    assert!(
        index.records.iter().all(|r| r.avatar_url.starts_with('/')),
        "an avatar url is not an asset path"
    );

    // `[Video]` stories, which the library marks.
    let video_ids: Vec<&str> = index
        .groups
        .iter()
        .flat_map(|g| &g.stories)
        .filter(|s| s.has_video)
        .map(|s| s.id.as_str())
        .collect();
    println!(
        "videos: {} stories carry a `video` command: {:?}",
        video_ids.len(),
        &video_ids[..video_ids.len().min(20)]
    );
    assert!(
        video_ids.contains(&"main_14_level_main_14-20_beg"),
        "the one wordless cutscene is not marked as a video"
    );
    assert!(
        index
            .groups
            .iter()
            .flat_map(|g| &g.stories)
            .all(|s| !s.has_video || s.has_script),
        "a story with no script claims a video"
    );

    let mut kind_rows: Vec<(&String, &usize)> = kinds.iter().collect();
    kind_rows.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let kinds_line: Vec<String> = kind_rows.iter().map(|(k, n)| format!("{k} {n}")).collect();
    println!("kinds ({}): {}", kind_rows.len(), kinds_line.join(", "));
    for (typo, _) in parser::typo_table() {
        assert!(
            !kinds.contains_key(*typo),
            "typo `{typo}` survived normalisation"
        );
    }

    fn report(label: &str, refs: &BTreeMap<String, bool>) -> (usize, usize) {
        let total = refs.len();
        let hit = refs.values().filter(|v| **v).count();
        let mut misses: Vec<&String> = refs.iter().filter(|(_, v)| !**v).map(|(k, _)| k).collect();
        misses.truncate(20);
        println!("{label}: {hit} of {total} distinct names resolved; first unresolved: {misses:?}");
        (hit, total)
    }
    let (bg_hit, bg_total) = report("backgrounds", &refs_bg);
    let (img_hit, img_total) = report("images", &refs_img);
    let refs_chr: BTreeMap<String, bool> = refs_chr_faces
        .iter()
        .map(|(k, v)| (k.clone(), v.is_some()))
        .collect();
    let (chr_hit, chr_total) = report("characters", &refs_chr);
    let faces_total = refs_chr_faces
        .iter()
        .filter(|(k, v)| k.contains('#') && v.is_some())
        .count();
    let faces_with_patch = refs_chr_faces
        .values()
        .filter(|v| **v == Some(true))
        .count();
    let (mus_hit, mus_total) = report("music", &refs_mus);
    let (snd_hit, snd_total) = report("sounds", &refs_snd);
    let (avt_hit, avt_total) = report("popupdialog avatars", &refs_avt);
    assert_eq!(avt_hit, avt_total, "every dialoghead should resolve");
    let faces_with_pos = refs_chr_pos.values().filter(|v| **v).count();
    let patch_without_pos = refs_chr_faces
        .iter()
        .filter(|(k, v)| **v == Some(true) && refs_chr_pos.get(*k) == Some(&false))
        .count();
    println!(
        "image names present in both bg/ and imgs/ trees: {}; distinct character names with a `#face` that resolved: {faces_total}; distinct names resolved with a face patch file: {faces_with_patch}",
        ambiguous.len()
    );
    let mut patchless: Vec<(&String, &usize)> = patchless_folders.iter().collect();
    patchless.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    println!(
        "face patches with no placement sit in {} folders; the busiest: {:?}",
        patchless.len(),
        &patchless[..patchless.len().min(8)]
    );
    println!(
        "faces on the wire: {faces_with_patch} names carry a face patch, {faces_with_pos} of them carry a facePos; {patch_without_pos} have a patch and NO facePos (sentinel hub, whole-body sprite or a folder with no hub.json)"
    );
    println!(
        "parser parity over the same {parsed} stories: {continued_lines} raw lines joined by a trailing backslash, {decoded_values} argument values decoded an escape, {dropped_arg_blocks} argument blocks the client would DROP on an invalid escape (in {} stories), {typo_lines} command names folded by the typo table the client does not have",
        escape_stories.len()
    );
    let mut sizes: Vec<(&String, &usize)> = body_sizes.iter().collect();
    sizes.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    println!(
        "body textures on the wire: {bodies_sized} of {} resolved names carry a bodySize; by size: {sizes:?}",
        refs_chr_pos.len()
    );
    let mut plates: Vec<(&String, &usize)> = plate_sizes.iter().collect();
    plates.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    println!(
        "character plates on the wire: {plates_sized} of {} resolved names carry one, {} carry none; the commonest: {:?}",
        refs_chr_pos.len(),
        plateless.len(),
        &plates[..plates.len().min(8)]
    );
    let assumed = image_size_assumed.values().filter(|a| **a).count();
    let mut ppus: Vec<(&String, &usize)> = ppu_values.iter().collect();
    ppus.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    println!(
        "image sizes on the wire: {} referenced names carry one, {} read from sprites.json and {assumed} from the PNG header at an assumed ppu of 100; {} names carry a URL and no size; ppu census {:?}",
        image_size_assumed.len(),
        image_size_assumed.len() - assumed,
        sizeless.len(),
        &ppus[..ppus.len().min(8)]
    );
    assert!(
        sizeless.is_empty(),
        "{} referenced image names carry no size: {:?}",
        sizeless.len(),
        sizeless.iter().take(8).collect::<Vec<_>>()
    );
    // The capture's own number: `bg_cher_1` is 1024x576 at ppu 68.2464, which
    // draws 1500.4 x 844.0 canvas px against the game's measured 1500.7 x 843.5.
    let cher = bg_cher_1.expect("bg_cher_1 is referenced by main_00-01_beg");
    assert_eq!((cher.w, cher.h), (1024.0, 576.0), "bg_cher_1 rect");
    assert!(
        (cher.ppu - 68.2464).abs() < 0.001 && cher.ppu_assumed.is_none(),
        "bg_cher_1 ppu {:?}",
        cher
    );
    assert!(
        plates_sized * 100 >= refs_chr_pos.len() * 95,
        "only {plates_sized} of {} resolved names carry a plate; first without: {:?}",
        refs_chr_pos.len(),
        &plateless[..plateless.len().min(8)]
    );
    assert!(
        faces_with_pos > 0,
        "no facePos reached the wire; has `hub.json` been exported?"
    );
    // `facePos` is measured in BODY TEXTURE pixels, so a placement with no
    // size beside it cannot be turned into a fraction of the plate.
    assert!(
        pos_without_size.is_empty(),
        "{} names carry a facePos and NO bodySize: {:?}",
        pos_without_size.len(),
        &pos_without_size[..pos_without_size.len().min(8)]
    );
    assert!(
        bodies_unsized.is_empty(),
        "{} resolved bodies carry no bodySize: {:?}",
        bodies_unsized.len(),
        &bodies_unsized[..bodies_unsized.len().min(8)]
    );
    assert!(
        bg_hit * 100 >= bg_total * 95,
        "backgrounds {bg_hit}/{bg_total}"
    );
    assert!(
        img_hit * 100 >= img_total * 90,
        "images {img_hit}/{img_total}"
    );
    assert!(
        chr_hit * 100 >= chr_total * 99,
        "characters {chr_hit}/{chr_total}"
    );
    assert!(
        mus_hit * 100 >= mus_total * 99,
        "music {mus_hit}/{mus_total}"
    );
    assert!(
        snd_hit * 100 >= snd_total * 99,
        "sounds {snd_hit}/{snd_total}"
    );

    // GOAL: no request the reader can fire may 404. Every distinct URL the
    // wire `StoryAssets` emits over all 1,797 resolvable stories is stat'd
    // against the assets root, in the same form `GET /api/assets/{path}`
    // serves (`%23` back to `#`). A miss is a resolver defect, never a test
    // to relax.
    let mut per_kind: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    let mut url_misses: Vec<(&'static str, String)> = Vec::new();
    for (url, kind) in &emitted_urls {
        let entry = per_kind.entry(kind).or_default();
        entry.0 += 1;
        let on_disk = dir.join(url.trim_start_matches('/').replace("%23", "#"));
        if on_disk.exists() {
            entry.1 += 1;
        } else {
            url_misses.push((kind, url.clone()));
        }
    }
    println!(
        "emitted URLs checked on disk: {} distinct, {} missing; per kind {:?}",
        emitted_urls.len(),
        url_misses.len(),
        per_kind
    );
    let unresolved: BTreeMap<&str, usize> = BTreeMap::from([
        ("background", bg_total - bg_hit),
        ("image", img_total - img_hit),
        ("character", chr_total - chr_hit),
        ("music", mus_total - mus_hit),
        ("sound", snd_total - snd_hit),
        ("avatar", avt_total - avt_hit),
    ]);
    println!(
        "distinct referenced names with NO map entry (the frontend's skip path): {unresolved:?}"
    );
    assert!(
        url_misses.is_empty(),
        "{} emitted URLs have no file: {:?}",
        url_misses.len(),
        &url_misses[..url_misses.len().min(20)]
    );

    // The service's synchronous half, as the route would call it. GT-1
    // (`1stact`) is summary-only in the served tree, so the smoke uses the
    // first main story with a script.
    let known = index
        .groups
        .iter()
        .filter(|g| g.id == "main_0")
        .flat_map(|g| g.stories.iter())
        .find(|s| s.has_script)
        .expect("main_0 has a story with a script");
    let got = load_and_parse(&dir, &asset_index, &known.id, &cache.lookup[&known.id])
        .expect("loads")
        .expect("known id");
    assert_eq!(got.id, known.id);
    assert_eq!(got.name, known.name);
    assert_eq!(got.group_id, "main_0");
    assert!(got.commands.len() > 50, "{}", got.commands.len());
    assert!(got.word_count > 100);
    assert!(!got.assets.backgrounds.is_empty());
    assert!(!got.assets.characters.is_empty());
    println!(
        "smoke {} ({}): {} commands, {} words, {} backgrounds, {} images, {} characters, {} music cues, {} sounds; first background {:?}",
        got.id,
        got.name,
        got.commands.len(),
        got.word_count,
        got.assets.backgrounds.len(),
        got.assets.images.len(),
        got.assets.characters.len(),
        got.assets.music.len(),
        got.assets.sounds.len(),
        got.assets.backgrounds.iter().next()
    );
    for url in got
        .assets
        .backgrounds
        .values()
        .chain(got.assets.images.values())
        .chain(got.assets.sounds.values())
        .chain(got.assets.characters.values().map(|c| &c.body_url))
    {
        let on_disk = dir.join(url.trim_start_matches('/').replace("%23", "#"));
        assert!(on_disk.exists(), "resolved URL has no file: {url}");
    }

    // The ILLUSTRATIONS tab: every distinct background, CG and sprite folder
    // a group's stories reference, off the cached facts.
    let mut all_bg: HashSet<String> = HashSet::new();
    let mut all_img: HashSet<String> = HashSet::new();
    let mut all_spr: HashSet<String> = HashSet::new();
    let mut ill_urls: HashSet<String> = HashSet::new();
    let mut ill_unresolved = (0usize, 0usize, 0usize);
    let mut faces_total = 0usize;
    let mut largest = (0usize, String::new(), 0usize);
    for g in &index.groups {
        let ill = group_illustrations(&cache, &story_assets, &g.id)
            .unwrap_or_else(|| panic!("group {} has no illustrations entry", g.id));
        assert_eq!(ill.group_id, g.id);
        assert_eq!(
            u32::try_from(ill.backgrounds.len() + ill.images.len()).unwrap(),
            g.illustration_count,
            "group {} illustrationCount",
            g.id
        );
        assert_eq!(
            u32::try_from(ill.sprites.len()).unwrap(),
            g.sprite_count,
            "group {} spriteCount",
            g.id
        );
        for item in ill.backgrounds.iter().chain(ill.images.iter()) {
            assert!(
                !item.story_ids.is_empty(),
                "group {} lists {} with no story",
                g.id,
                item.name
            );
        }
        for item in &ill.backgrounds {
            all_bg.insert(item.name.clone());
            match &item.url {
                Some(u) => {
                    ill_urls.insert(u.clone());
                }
                None => ill_unresolved.0 += 1,
            }
        }
        for item in &ill.images {
            all_img.insert(item.name.clone());
            match &item.url {
                Some(u) => {
                    ill_urls.insert(u.clone());
                }
                None => ill_unresolved.1 += 1,
            }
        }
        for item in &ill.sprites {
            all_spr.insert(item.base.clone());
            faces_total += item.faces as usize;
            match &item.body_url {
                Some(u) => {
                    ill_urls.insert(u.clone());
                }
                None => ill_unresolved.2 += 1,
            }
        }
        let bytes = serde_json::to_vec(&ill).unwrap().len();
        if bytes > largest.0 {
            largest = (
                bytes,
                g.id.clone(),
                ill.backgrounds.len() + ill.images.len(),
            );
        }
    }
    println!(
        "illustrations over the library: {} distinct backgrounds, {} CGs, {} sprite folders ({faces_total} face variants); unresolved names bg {} img {} sprite {}",
        all_bg.len(),
        all_img.len(),
        all_spr.len(),
        ill_unresolved.0,
        ill_unresolved.1,
        ill_unresolved.2
    );
    println!(
        "largest group response: {} is {} bytes of JSON over {} illustrations",
        largest.1, largest.0, largest.2
    );
    let missing: Vec<&String> = ill_urls
        .iter()
        .filter(|u| {
            !dir.join(u.trim_start_matches('/').replace("%23", "#"))
                .exists()
        })
        .collect();
    assert!(
        missing.is_empty(),
        "{} illustration URLs have no file: {:?}",
        missing.len(),
        &missing[..missing.len().min(8)]
    );
    assert!(
        all_bg.len() >= 900 && all_img.len() >= 1200 && all_spr.len() >= 1500,
        "corpus totals moved: {} backgrounds, {} images, {} sprites",
        all_bg.len(),
        all_img.len(),
        all_spr.len()
    );

    // The Prologue, against the reference site's own Illustrations tab.
    let prologue = group_illustrations(&cache, &story_assets, "main_0").expect("main_0");
    println!(
        "main_0: {} backgrounds {:?}, {} CGs {:?}, {} sprites",
        prologue.backgrounds.len(),
        prologue
            .backgrounds
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>(),
        prologue.images.len(),
        prologue
            .images
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>(),
        prologue.sprites.len()
    );
    // The reference site's Prologue Illustrations tab shows 13, which is this
    // `images` list exactly: `bg_0_babel` is a `bg_`-NAMED plate written by an
    // `[Image]` command, so it is a CG on the wire and not one of the 4
    // backgrounds. The reference counts CGs alone; `illustrationCount` counts
    // both, so the tab is 17 rows over two shelves for this group.
    assert_eq!(prologue.images.len(), 13, "main_0 CG count");
    assert_eq!(prologue.backgrounds.len(), 4, "main_0 background count");
    assert!(
        prologue.images.iter().any(|i| i.name == "bg_0_babel"),
        "main_0 does not list bg_0_babel"
    );
    assert!(
        prologue.images.iter().any(|i| i.name == "avg_0_1"),
        "main_0 does not list avg_0_1"
    );
    assert!(
        prologue.backgrounds.iter().all(|b| b.url.is_some())
            && prologue.images.iter().all(|i| i.url.is_some()),
        "a Prologue illustration did not resolve"
    );
    assert!(
        !prologue.sprites.is_empty() && prologue.sprites.iter().all(|s| s.faces >= 1),
        "a Prologue sprite carries no face"
    );

    // An operator `charId` answers with that operator's records.
    let kroos = group_illustrations(&cache, &story_assets, "char_124_kroos").expect("kroos");
    println!(
        "char_124_kroos: {} backgrounds, {} CGs, {} sprites",
        kroos.backgrounds.len(),
        kroos.images.len(),
        kroos.sprites.len()
    );
    assert!(group_illustrations(&cache, &story_assets, "no_such_group").is_none());

    // Mainline release dates: the group rows are all -1, the ZONE carries the
    // date for the five chapters that have one.
    let dated: Vec<(&str, i64)> = index
        .groups
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
        .map(|g| (g.id.as_str(), g.start_time))
        .collect();
    println!("mainline startTime: {dated:?}");
    assert_eq!(
        dated.iter().filter(|(_, t)| *t > 0).count(),
        5,
        "expected 5 dated mainline chapters"
    );
    assert!(dated.contains(&("main_10", 1_666_180_800)));
    assert!(dated.contains(&("main_14", 1_730_394_000)));
    // Nothing else moved: every other group still carries its table value.
    for g in index
        .groups
        .iter()
        .filter(|g| g.category != StoryCategory::Main)
    {
        assert_eq!(
            g.start_time, gd.story_reviews[&g.id].start_time,
            "group {} startTime moved",
            g.id
        );
    }

    // A library id with no script answers 404 with a message.
    if let Some(unbacked) = index
        .groups
        .iter()
        .flat_map(|g| g.stories.iter())
        .find(|s| !s.has_script)
    {
        match load_and_parse(
            &dir,
            &asset_index,
            &unbacked.id,
            &cache.lookup[&unbacked.id],
        ) {
            Err(ApiError::NotFoundMessage(m)) => println!("unbacked {}: {m}", unbacked.id),
            other => panic!(
                "expected NotFoundMessage for {}, got {other:?}",
                unbacked.id
            ),
        }
    }
}

/// Cutscene clips: which `[Video]` references have a file under `video/`.
///
/// EN writes 26 `[Video]` commands over 26 scripts naming 25 distinct `res`
/// values, and 12 of those 25 have a transcode on disk. The other 13 are clips
/// EN retired, and the reader skips them rather than halting on a black
/// rectangle. Only 5 of the 26 scripts are LIBRARY stories; the other 21 are
/// zone and activity entry scripts the reader has no route to, so the halt
/// fires on 5 stories in practice and all 5 of those resolve.
#[test]
fn cutscene_videos_resolve_where_en_still_ships_the_clip() {
    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!("story_real_data_test: no EN story tree, skipping video probe");
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let cache = build_index(gd, &asset_index, &dir);
    let story_assets = StoryAssetIndex::for_dir(&dir, &asset_index);

    // Every script on disk, not only the ones the library lists, because the
    // entry scripts carry most of the references and none of them is a story.
    let mut refs: BTreeMap<String, bool> = BTreeMap::new();
    let mut commands = 0usize;
    let mut scripts = 0usize;
    let mut main_10: Option<backend::core::story::StoryScript> = None;
    let mut act15: Option<backend::core::story::StoryScript> = None;
    for entry in walkdir::WalkDir::new(dir.join("gamedata/story")).min_depth(1) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.is_file() || !path.to_string_lossy().ends_with(".txt.txt") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if !content
            .lines()
            .any(|l| l.trim_start().to_ascii_lowercase().starts_with("[video"))
        {
            continue;
        }
        scripts += 1;
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let script = story::parse_story(&name, &name, "", &content, &story_assets);
        for c in script.commands.iter().filter(|c| c.kind == "video") {
            let Some(res) = c.args.get("res") else {
                continue;
            };
            commands += 1;
            refs.insert(res.clone(), script.assets.videos.contains_key(res));
        }
        if name.starts_with("main_10_zone_enter") {
            main_10 = Some(script);
        } else if name.starts_with("level_act15side_entry") {
            act15 = Some(script);
        }
    }
    let resolved = refs.values().filter(|v| **v).count();
    println!(
        "video: {} clips on disk, {scripts} scripts, {commands} [Video] commands, {} distinct res, {resolved} resolved",
        story_assets.video_count(),
        refs.len()
    );
    for (res, ok) in &refs {
        println!("  {} {res}", if *ok { "HAS " } else { "MISS" });
    }
    assert_eq!(scripts, 26, "scripts carrying a [Video] moved");
    assert_eq!(commands, 26, "[Video] command count moved");
    assert_eq!(refs.len(), 25, "distinct [Video] res values moved");
    assert_eq!(resolved, 12, "resolved cutscene count moved");
    assert_eq!(story_assets.video_count(), 16, "clips on disk moved");

    // How many of those references the reader can actually reach: a `[Video]`
    // in a script the library never lists is unreachable by construction.
    let in_library = cache
        .index
        .groups
        .iter()
        .flat_map(|g| g.stories.iter())
        .filter(|s| s.has_video)
        .count();
    println!("video: {in_library} of {scripts} scripts are library stories");
    assert_eq!(in_library, 5, "library stories carrying a [Video] moved");

    // The chapter 10 opener carries both transcodes.
    let script = main_10.expect("main_10_zone_enter is on disk");
    let sources = script
        .assets
        .videos
        .get("video/main_10/main_10_enter.mp4")
        .expect("main_10_enter is on the wire");
    assert_eq!(
        sources.webm_url.as_deref(),
        Some("/video/main_10/main_10_enter.webm")
    );
    assert_eq!(
        sources.mp4_url.as_deref(),
        Some("/video/main_10/main_10_enter.mp4")
    );

    // The act15side entry names a clip EN no longer ships: nothing on the wire.
    let script = act15.expect("level_act15side_entry is on disk");
    assert!(
        script.assets.videos.is_empty(),
        "act15side should carry no videos, got {:?}",
        script.assets.videos
    );
}

/// Dumps the wire-shape `StoryScript` JSON the frontend engine tests run on.
/// Inert unless `STORY_FIXTURE_OUT=<dir>` is set; then it writes one file per
/// story id below into that directory, so the engine's vitest suite runs on
/// real parser output instead of a hand-written imitation of it.
///
/// `cd backend && STORY_FIXTURE_OUT=../frontend/src/lib/story/__fixtures__ cargo test --test story_real_data_test fixtures`
#[test]
fn dump_engine_fixtures_when_asked() {
    let Ok(out) = std::env::var("STORY_FIXTURE_OUT") else {
        return;
    };
    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!("story_real_data_test: no EN story tree, not dumping fixtures");
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let cache = build_index(gd, &asset_index, &dir);
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join(out);
    std::fs::create_dir_all(&out).expect("fixture dir");
    // The tutorial opener, and the chapter 15 epilogue: 11 decisions, a
    // `references="1"` / `"2"` pair at lines 53 and 57 of the file, and
    // charslot throughout.
    for id in ["main_0_0_welcome_to_guide", "main_15_level_main_15-08_end"] {
        let story_ref = cache
            .lookup
            .get(id)
            .unwrap_or_else(|| panic!("{id} is not in the index"));
        let script = load_and_parse(&dir, &asset_index, id, story_ref)
            .expect("loads")
            .expect("known id");
        let json = serde_json::to_string(&script).expect("serialises");
        let path = out.join(format!("{id}.json"));
        std::fs::write(&path, &json).expect("writes fixture");
        println!(
            "fixture {}: {} commands, {} bytes -> {}",
            id,
            script.commands.len(),
            json.len(),
            path.display()
        );
    }
}

/// What fraction of the EN corpus the FRONTEND engine models, over every
/// script file on disk rather than only the 1,797 the library lists. The
/// handled set mirrors `HANDLED_KINDS` in `frontend/src/lib/story/engine.ts`;
/// the two are kept in step by hand and this test is what re-derives the
/// percentage the docs quote.
#[test]
fn engine_coverage_over_every_script_file() {
    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!("story_real_data_test: no EN story tree, skipping coverage");
        return;
    }
    const HANDLED: &[&str] = &[
        "name",
        "text",
        "narration",
        "multiline",
        "dialog",
        "voicewithin",
        "sticker",
        "stickerclear",
        "subtitle",
        "decision",
        "predicate",
        "character",
        "charslot",
        "image",
        "background",
        "blocker",
        "delay",
        "playmusic",
        "stopmusic",
        "musicvolume",
        "playsound",
        "stopsound",
        "soundvolume",
        "camerashake",
        "cameraeffect",
        "header",
        "imagetween",
        "backgroundtween",
        "largebgtween",
        "characteraction",
        "charactercutin",
        "curtain",
        "focusout",
        "largebg",
        "gridbg",
        "popupdialog",
        "interlude",
    ];
    let handled: HashSet<&str> = HANDLED.iter().copied().collect();
    let root = dir.join("gamedata/story");
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    let mut files = 0usize;
    /// The two parser deviations from the client, over the WHOLE tree: the
    /// 1,797 readable stories carry none of either, and every one lives in a
    /// tutorial or guide script the library does not list.
    #[derive(Default)]
    struct WholeTree {
        continued: u64,
        decoded: u64,
        dropped: u64,
        typos: u64,
        files_with_continuation: usize,
        files_with_invalid_escape: usize,
    }
    let mut whole_tree = WholeTree::default();
    for entry in walkdir::WalkDir::new(&root).min_depth(1) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".txt.txt") {
            continue;
        }
        let Ok(raw) = std::fs::read(path) else {
            continue;
        };
        let content = String::from_utf8_lossy(&raw);
        if !content.trim_start().starts_with('[') {
            continue;
        }
        files += 1;
        let (commands, stats) = parser::parse_with_stats(&content);
        whole_tree.continued += u64::from(stats.continued_lines);
        whole_tree.decoded += u64::from(stats.decoded_values);
        whole_tree.dropped += u64::from(stats.dropped_arg_blocks);
        if stats.continued_lines > 0 {
            whole_tree.files_with_continuation += 1;
        }
        if stats.dropped_arg_blocks > 0 {
            whole_tree.files_with_invalid_escape += 1;
        }
        for d in &stats.diagnostics {
            if d.deviation == parser::Deviation::TypoFolded {
                whole_tree.typos += 1;
            }
        }
        for c in commands {
            *kinds.entry(c.kind).or_default() += 1;
        }
    }
    println!(
        "parser parity over all {files} script files: {} raw lines joined by a trailing backslash (in {} files), {} argument values decoded an escape, {} argument blocks the client DROPS on an invalid escape (in {} files), {} command names folded by a typo table the client does not have",
        whole_tree.continued,
        whole_tree.files_with_continuation,
        whole_tree.decoded,
        whole_tree.dropped,
        whole_tree.files_with_invalid_escape,
        whole_tree.typos
    );
    let total: usize = kinds.values().sum();
    let modelled: usize = kinds
        .iter()
        .filter(|(k, _)| handled.contains(k.as_str()))
        .map(|(_, n)| *n)
        .sum();
    let mut rest: Vec<(&String, &usize)> = kinds
        .iter()
        .filter(|(k, _)| !handled.contains(k.as_str()))
        .collect();
    rest.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    // The phase 2 set, so the before -> after is on ONE basis.
    const PHASE_2: &[&str] = &[
        "name",
        "text",
        "narration",
        "multiline",
        "dialog",
        "voicewithin",
        "sticker",
        "stickerclear",
        "subtitle",
        "decision",
        "predicate",
        "character",
        "charslot",
        "image",
        "background",
        "blocker",
        "delay",
        "playmusic",
        "stopmusic",
        "musicvolume",
        "playsound",
        "stopsound",
        "soundvolume",
        "camerashake",
        "cameraeffect",
        "header",
    ];
    let before: HashSet<&str> = PHASE_2.iter().copied().collect();
    let modelled_before: usize = kinds
        .iter()
        .filter(|(k, _)| before.contains(k.as_str()))
        .map(|(_, n)| *n)
        .sum();
    #[allow(clippy::cast_precision_loss)]
    let pct_before = modelled_before as f64 * 100.0 / total as f64;
    #[allow(clippy::cast_precision_loss)]
    let pct = modelled as f64 * 100.0 / total as f64;
    println!("engine coverage before phase 3: {modelled_before} of {total} ({pct_before:.3}%)");
    println!(
        "engine coverage: {files} script files, {total} commands, {modelled} modelled ({pct:.3}%), {} kinds unmodelled: {}",
        rest.len(),
        rest.iter()
            .map(|(k, n)| format!("{k} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(pct > 99.0, "engine coverage fell to {pct:.3}%");
}

/// The library index is built ONCE per game-data load, however many readers
/// arrive while it is building.
///
/// Measured on the 2026-09-24 boot, before this: `/metrics` carried
/// `myrtle_cpu_task_total{kind="story_index",outcome="started"} 3` and
/// `myrtle_cpu_task_duration_seconds_sum{kind="story_index"} 14.580557` for a
/// single EN load, and the log showed "story index warmed server=EN" followed
/// a second later by "story index built build_ms=6332" on the first
/// `GET /api/story/index`. The warm and the request did NOT key on different
/// `GameData` arcs: `cached_index` dropped its `std::sync::Mutex` before the
/// 6.3 s build and took it again afterwards, so every caller inside that
/// window missed and built its own copy. The slot lock now spans the build.
#[test]
fn the_story_index_builds_once_per_game_data_load() {
    use backend::app::services::story::{cached_index, index_builds};
    use backend::core::hypergryph::constants::Server;

    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!(
            "story_real_data_test: no EN story tree at {}, skipping",
            dir.display()
        );
        return;
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");

    runtime.block_on(async move {
        // Inside the runtime: a lazy sqlx pool registers a reaper and needs a
        // Tokio context to exist at all, even though this test never connects.
        let state = test_state(&dir);
        let before = index_builds();
        let readers: Vec<_> = (0..8)
            .map(|_| {
                let state = state.clone();
                tokio::spawn(async move { cached_index(&state, Server::EN).await })
            })
            .collect();
        let mut built = Vec::new();
        let mut refused = 0_usize;
        for reader in readers {
            match reader.await.expect("reader task") {
                Ok(index) => built.push(index),
                Err(e) => {
                    refused += 1;
                    eprintln!("reader refused: {e:?}");
                }
            }
        }
        let first_round = index_builds() - before;
        assert_eq!(
            refused, 0,
            "{refused} of 8 readers were refused; without the single flight each one takes \
             its own `cpu::run` permit and the eighth is shed at the 2.5 s admission wait"
        );
        assert_eq!(
            first_round, 1,
            "8 concurrent readers on one game-data load triggered {first_round} builds; \
             the shipped defect was 3 for a warm plus one request, and this harness \
             measured 7 builds plus one 503 before the fix"
        );
        for other in &built[1..] {
            assert!(
                Arc::ptr_eq(&built[0], other),
                "every reader must get the same index allocation"
            );
        }

        // And a reader arriving after the build takes no build at all.
        let again = cached_index(&state, Server::EN).await.expect("the index");
        assert!(Arc::ptr_eq(&built[0], &again));
        assert_eq!(
            index_builds() - before,
            1,
            "a post-build reader rebuilt the index"
        );
    });
}

/// An `AppState` with nothing in it but what the story index reads: the EN
/// game data (the binary's single load, shared) and its asset index. The pool
/// is lazy and never connected, because `cached_index` touches no database.
fn test_state(dir: &Path) -> backend::app::state::AppState {
    test_state_on(dir, "postgres://postgres:password@127.0.0.1:5432/postgres")
}

/// The same, against a named database, for the one test that actually reads
/// rows. The pool is still lazy, so a wrong URL fails at the first query
/// rather than at construction.
fn test_state_on(dir: &Path, database_url: &str) -> backend::app::state::AppState {
    use backend::app::cache::store::CacheStore;
    use backend::app::state::{AppConfig, AppState, ServerData};
    use backend::core::auth::credentials::CredentialKey;
    use backend::core::hypergryph::constants::Server;
    use std::sync::atomic::AtomicBool;

    let server_data = Arc::new(ServerData {
        game_data: arc_swap::ArcSwap::new(common::shared_game_data()),
        asset_index: arc_swap::ArcSwap::from_pointee(AssetIndex::build(dir)),
        game_data_dir: dir.join("gamedata/excel").display().to_string(),
        assets_dir: dir.display().to_string(),
        loaded: AtomicBool::new(true),
    });
    let config = AppConfig {
        jwt_secret: "test-jwt-secret-not-used-by-this-test".into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "test-service-key".into(),
        assets_base_dir: dir
            .parent()
            .map_or_else(|| dir.display().to_string(), |p| p.display().to_string()),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
    };
    AppState::new(
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy(database_url)
            .expect("lazy pool"),
        CacheStore::new_memory(),
        HashMap::from([(Server::EN, server_data)]),
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    )
}

/// The event ARCHIVE against the real EN tree: the counts the "from the
/// archive" pass quotes, and the rule that every picture and every clip it
/// puts on the wire is a file that exists.
///
/// The library-group half is small and the whole point of the numbers: 5 of
/// the 15 archive components belong to a group `story_review_table` lists
/// (`act13side`, `act17side`, `act25side`, `act29side`, `act42side`), plus
/// `main_14`'s recordings, which come from `activity_table` and not from the
/// archive table at all.
#[test]
fn the_archive_resolves_every_picture_and_every_track() {
    use backend::app::services::story::{
        ArchiveNewsLine, StoryArchiveSection, build_archives, get_group_archive,
    };
    use backend::core::hypergryph::constants::Server;

    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!(
            "story_real_data_test: no EN story tree at {}, skipping",
            dir.display()
        );
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let story_assets = StoryAssetIndex::for_dir(&dir, &asset_index);

    let started = Instant::now();
    let archives = build_archives(gd, &story_assets, &dir);
    let archive_ms = started.elapsed().as_millis();
    println!("archives: {} groups in {archive_ms} ms", archives.len());

    let section = |group: &str, want: &str| -> Option<&StoryArchiveSection> {
        archives.get(group)?.sections.iter().find(|s| {
            matches!(
                (s, want),
                (StoryArchiveSection::Logs { .. }, "logs")
                    | (StoryArchiveSection::Landmarks { .. }, "landmarks")
                    | (StoryArchiveSection::News { .. }, "news")
                    | (StoryArchiveSection::Files { .. }, "files")
                    | (StoryArchiveSection::Gallery { .. }, "gallery")
                    | (StoryArchiveSection::Music { .. }, "music")
                    | (StoryArchiveSection::Recordings { .. }, "recordings")
            )
        })
    };

    // The counts the audit fixed. `act13side` also carries a 15-entry
    // `Timeline` slot, which no section exposes: it is DROPPED, and its news,
    // pictures and files are all served on their own.
    let Some(StoryArchiveSection::Landmarks { count, landmarks }) =
        section("act17side", "landmarks")
    else {
        panic!("act17side has no landmarks section");
    };
    assert_eq!(*count, 26);
    assert_eq!(landmarks.len(), 26);
    let Some(StoryArchiveSection::Logs { count, chapters }) = section("act17side", "logs") else {
        panic!("act17side has no logs section");
    };
    assert_eq!(*count, 172, "act17side logs");
    assert_eq!(chapters.len(), 22, "act17side log chapters");
    let Some(StoryArchiveSection::News { count, news }) = section("act13side", "news") else {
        panic!("act13side has no news section");
    };
    assert_eq!(*count, 18);
    assert_eq!(news.len(), 18);
    let Some(StoryArchiveSection::Files { count, .. }) = section("act25side", "files") else {
        panic!("act25side has no files section");
    };
    assert_eq!(*count, 12, "act25side files");
    let Some(StoryArchiveSection::Recordings {
        count,
        nodes,
        hidden,
    }) = section("main_14", "recordings")
    else {
        panic!("main_14 has no recordings section");
    };
    assert_eq!(nodes.len(), 5, "main_14 recording nodes");
    assert_eq!(hidden.len(), 8, "main_14 hidden clips");
    assert_eq!(*count, 43, "main_14 clips, hidden included");

    // Every picture on the wire, by kind: resolved over listed.
    let mut landmark_pics = (0_usize, 0_usize);
    let mut file_pics = (0_usize, 0_usize);
    let mut file_titles = (0_usize, 0_usize);
    let mut news_images = (0_usize, 0_usize);
    let mut gallery = (0_usize, 0_usize);
    let mut formats: BTreeMap<String, bool> = BTreeMap::new();
    let mut tracks = 0_usize;
    let mut clips = (0_usize, 0_usize);
    for archive in archives.values() {
        for s in &archive.sections {
            match s {
                StoryArchiveSection::Landmarks { landmarks, .. } => {
                    for l in landmarks {
                        landmark_pics.1 += 1;
                        landmark_pics.0 += usize::from(l.picture_url.is_some());
                    }
                }
                StoryArchiveSection::Files { files, .. } => {
                    for f in files {
                        file_pics.1 += 1;
                        file_pics.0 += usize::from(f.picture_url.is_some());
                        if let Some(raw) = gd
                            .story_archives
                            .act_archive_res_data
                            .stories
                            .get(&f.id)
                            .filter(|s| !s.title_pic.trim().is_empty())
                        {
                            let _ = raw;
                            file_titles.1 += 1;
                            file_titles.0 += usize::from(f.title_picture_url.is_some());
                        }
                    }
                }
                StoryArchiveSection::News { news, .. } => {
                    for n in news {
                        if let Some(f) = n.format.as_ref() {
                            formats.insert(
                                f.type_id.clone(),
                                f.logo_url.is_some() && f.main_logo_url.is_some(),
                            );
                        }
                        for line in &n.lines {
                            if let ArchiveNewsLine::Image { url, .. } = line {
                                news_images.1 += 1;
                                news_images.0 += usize::from(url.is_some());
                            }
                        }
                    }
                }
                StoryArchiveSection::Gallery { pictures, .. } => {
                    for p in pictures {
                        gallery.1 += 1;
                        gallery.0 += usize::from(p.url.is_some());
                    }
                }
                StoryArchiveSection::Music { tracks: t, .. } => tracks += t.len(),
                StoryArchiveSection::Recordings { nodes, hidden, .. } => {
                    for c in nodes.iter().flat_map(|n| &n.clips).chain(hidden) {
                        clips.1 += 1;
                        clips.0 += usize::from(!c.tracks.is_empty());
                    }
                }
                StoryArchiveSection::Logs { .. } => {}
            }
        }
    }
    let formats_ok = formats.values().filter(|ok| **ok).count();
    println!(
        "archive art: landmarks {}/{}, file plates {}/{}, file titles {}/{}, news mastheads {}/{}, news pictures {}/{}, gallery {}/{}; {tracks} tracks, clips with audio {}/{}",
        landmark_pics.0,
        landmark_pics.1,
        file_pics.0,
        file_pics.1,
        file_titles.0,
        file_titles.1,
        formats_ok,
        formats.len(),
        news_images.0,
        news_images.1,
        gallery.0,
        gallery.1,
        clips.0,
        clips.1
    );
    assert_eq!(landmark_pics, (26, 26), "landmark pictures");
    assert_eq!(file_pics, (16, 16), "file plates");
    assert_eq!(file_titles, (4, 4), "drawn file titles");
    assert_eq!((formats_ok, formats.len()), (3, 3), "news mastheads");
    assert_eq!(news_images, (45, 45), "news pictures");
    assert_eq!(gallery, (57, 57), "gallery pictures");
    assert_eq!(clips, (43, 43), "recorded clips with at least one language");

    // Every archived track, over the whole content half and not just the
    // groups the library can reach: the join is `Audios` -> `Musics.Bank` ->
    // `BgmBanks` (+`BankAlias`) -> the loop clip on disk.
    let audios = &gd.story_archives.act_archive_res_data.audios;
    let joined = audios
        .keys()
        .filter(|id| {
            gd.music
                .music(id)
                .and_then(|m| gd.music.bank(&m.bank))
                .and_then(|b| b.loop_.as_deref())
                .and_then(|l| story_assets.resolve_audio(l))
                .is_some()
        })
        .count();
    println!(
        "archive audio: {joined}/{} rows join to a loop clip; {tracks} of them sit on a library group",
        audios.len()
    );
    assert_eq!(joined, audios.len(), "archived tracks with a loop clip");

    // The two acts whose only slot is a `ChallengeBook` are listed and EMPTY:
    // no section carries a challenge book, so they answer 200 with nothing.
    for id in ["act29side", "act42side"] {
        assert!(
            archives[id].sections.is_empty(),
            "{id} should carry no section"
        );
    }
    assert_eq!(archives.len(), 6, "groups with an archive entry");

    // The 5 `rogue_*` and 1 `sandbox_1` components name acts the Archives do
    // not list, so nothing answers for them.
    for id in ["rogue_1", "rogue_5", "sandbox_1", "act13sre"] {
        assert!(!archives.contains_key(id), "{id} must not be a group");
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let state = test_state(&dir);
        let empty = get_group_archive(&state, Server::EN, "main_0")
            .await
            .expect("a group with no archive is 200");
        assert_eq!(empty.group_id, "main_0");
        assert!(empty.sections.is_empty(), "main_0 keeps no archive");
        let full = get_group_archive(&state, Server::EN, "act17side")
            .await
            .expect("act17side");
        assert_eq!(
            full.sections.len(),
            4,
            "logs, landmarks, a gallery and 5 tracks"
        );
        let sample = serde_json::to_string(&archives["main_14"]).expect("json");
        println!(
            "main_14 archive: {} bytes, head {}",
            sample.len(),
            &sample[..sample.len().min(420)]
        );
        let err = get_group_archive(&state, Server::EN, "not_a_group")
            .await
            .expect_err("an unknown id is a 404");
        assert!(matches!(err, ApiError::NotFoundMessage(_)), "{err:?}");
    });
}

/// The group THEME on the index: how many of the 87 EN library groups carry
/// one, how many carry a title, and that every loop clip is a file.
#[test]
fn every_group_theme_resolves_to_a_clip() {
    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!(
            "story_real_data_test: no EN story tree at {}, skipping",
            dir.display()
        );
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let cache = backend::app::services::story::build_index(gd, &asset_index, &dir);

    let library: Vec<_> = cache
        .index
        .groups
        .iter()
        .filter(|g| g.category != StoryCategory::Record)
        .collect();
    let with_music: Vec<_> = library.iter().filter(|g| g.music.is_some()).collect();
    let titled = with_music
        .iter()
        .filter(|g| g.music.as_ref().and_then(|m| m.title.as_ref()).is_some())
        .count();
    let with_intro = with_music
        .iter()
        .filter(|g| {
            g.music
                .as_ref()
                .and_then(|m| m.intro_url.as_ref())
                .is_some()
        })
        .count();
    let missing: Vec<&str> = library
        .iter()
        .filter(|g| g.music.is_none())
        .map(|g| g.id.as_str())
        .collect();
    println!(
        "themes: {}/{} library groups, {titled} titled, {with_intro} with an intro; none for {missing:?}",
        with_music.len(),
        library.len()
    );
    assert_eq!(library.len(), 87, "EN library groups");
    assert_eq!(with_music.len(), 86, "groups with a theme");
    assert_eq!(titled, 6, "themes with a title");
    assert_eq!(
        missing,
        ["act24side"],
        "the only theme without a clip is act24side, whose bank names \
         Audio/Sound_Beta_2/Music/act24side/m_sys_act24side_loop and the tree holds no \
         act24side music directory at all"
    );
    // Every mainline chapter has a theme and not one of them has a title:
    // all 17 `music_3in1bg_main{N}` rows write a single space for `Name`.
    let mainline: Vec<_> = library
        .iter()
        .filter(|g| g.category == StoryCategory::Main)
        .collect();
    assert_eq!(mainline.len(), 17);
    assert!(mainline.iter().all(|g| g.music.is_some()));
    assert!(
        mainline
            .iter()
            .all(|g| g.music.as_ref().is_some_and(|m| m.title.is_none()))
    );
    for g in &with_music {
        let m = g.music.as_ref().expect("checked");
        assert!(
            dir.join(m.loop_url.trim_start_matches('/')).is_file(),
            "{}: loop clip {} is not on disk",
            g.id,
            m.loop_url
        );
        if let Some(intro) = m.intro_url.as_deref() {
            assert!(
                dir.join(intro.trim_start_matches('/')).is_file(),
                "{}: intro clip {intro} is not on disk",
                g.id
            );
        }
    }
    println!(
        "story index build {} ms, archives {} ms",
        cache.build_ms, cache.archive_ms
    );
}

/// The community reading aggregate against the REAL local database.
///
/// Ignored by default because it needs both the EN tree and a populated
/// Postgres; run it with
/// `DATABASE_URL=postgres://... cargo test --test story_real_data_test -- --ignored the_community_aggregate --nocapture`.
/// Everything it prints is a measurement, not an assertion: the assertions are
/// the invariants that must hold whatever the data says.
#[test]
#[ignore = "needs a populated local database"]
fn the_community_aggregate_over_the_real_database() {
    use backend::app::services::story_community::compute;
    use backend::core::hypergryph::constants::Server;

    let dir = assets_dir();
    if !scripts_present(&dir) {
        eprintln!(
            "story_real_data_test: no EN story tree at {}, skipping",
            dir.display()
        );
        return;
    }
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("story_real_data_test: DATABASE_URL is unset, skipping");
        return;
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("runtime");

    runtime.block_on(async move {
        let state = test_state_on(&dir, &url);
        let index = backend::app::services::story::cached_index(&state, Server::EN)
            .await
            .expect("the story index");
        let started = Instant::now();
        let (out, census, compute_ms) = compute(&state, Server::EN).await.expect("the aggregate");
        let wall = started.elapsed();

        let payload = serde_json::to_vec(&out).expect("serializes");
        println!(
            "compute: {} ms inside the service, {:.3} s wall, {} bytes of JSON ({:.1} KB)",
            compute_ms,
            wall.as_secs_f64(),
            payload.len(),
            payload.len() as f64 / 1024.0
        );
        println!(
            "accounts: {} scanned, {} players, {} from verdict rows, {} gate-derived, {} with a reader document, {} with an empty read set; {} reads in all",
            census.scanned,
            census.players,
            census.from_verdict,
            census.from_gates,
            census.from_document,
            census.empty,
            census.reads
        );
        println!(
            "library: {} stories, {} groups ({} library groups + {} operator record groups)",
            out.stories.len(),
            out.groups.len(),
            index.index.groups.len(),
            index.index.records.len()
        );

        let players = f64::from(out.players.max(1));
        let pct = |n: u32| f64::from(n) * 100.0 / players;
        let story_name: HashMap<&str, (&str, &str)> = index
            .index
            .groups
            .iter()
            .flat_map(|g| {
                g.stories
                    .iter()
                    .map(move |s| (s.id.as_str(), (s.name.as_str(), g.id.as_str())))
            })
            .collect();
        let group_name: HashMap<&str, &str> = index
            .index
            .groups
            .iter()
            .map(|g| (g.id.as_str(), g.name.as_str()))
            .chain(
                index
                    .index
                    .records
                    .iter()
                    .map(|r| (r.char_id.as_str(), r.name.as_str())),
            )
            .collect();

        let mut top = out.stories.clone();
        top.sort_by(|a, b| b.readers.cmp(&a.readers).then_with(|| a.id.cmp(&b.id)));
        println!("top 10 stories by readers:");
        for s in top.iter().take(10) {
            let (name, group) = story_name.get(s.id.as_str()).copied().unwrap_or(("", ""));
            println!(
                "  {:>6} ({:>5.1}%)  {:<40} {:<28} {}",
                s.readers,
                pct(s.readers),
                s.id,
                name,
                group
            );
        }

        let mut top_groups = out.groups.clone();
        top_groups.sort_by(|a, b| b.readers.cmp(&a.readers).then_with(|| a.id.cmp(&b.id)));
        println!("top 10 groups by readers:");
        for g in top_groups.iter().take(10) {
            println!(
                "  {:>6} ({:>5.1}%) readers, {:>6} finished ({:>5.1}%)  {:<20} {}",
                g.readers,
                pct(g.readers),
                g.finished,
                pct(g.finished),
                g.id,
                group_name.get(g.id.as_str()).copied().unwrap_or("")
            );
        }

        // The least-read mainline: scripted stories in the MAIN category only,
        // so an unscripted interlude cannot take a place it never had readers
        // for.
        let mainline: HashSet<&str> = index
            .index
            .groups
            .iter()
            .filter(|g| g.category == StoryCategory::Main)
            .flat_map(|g| g.stories.iter().filter(|s| s.has_script).map(|s| s.id.as_str()))
            .collect();
        let mut least: Vec<_> = out
            .stories
            .iter()
            .filter(|s| mainline.contains(s.id.as_str()))
            .collect();
        least.sort_by(|a, b| a.readers.cmp(&b.readers).then_with(|| a.id.cmp(&b.id)));
        println!("least-read 5 scripted mainline stories ({} in all):", mainline.len());
        for s in least.iter().take(5) {
            let (name, group) = story_name.get(s.id.as_str()).copied().unwrap_or(("", ""));
            println!(
                "  {:>6} ({:>5.1}%)  {:<40} {:<28} {}",
                s.readers,
                pct(s.readers),
                s.id,
                name,
                group
            );
        }

        // The depth curve: a story deeper into a chapter cannot have more
        // readers than the one before it, except where the game lets a reader
        // in by another door.
        for group_id in ["main_8", "main_0", "main_14"] {
            let Some(g) = out.groups.iter().find(|g| g.id == group_id) else {
                continue;
            };
            let slots: &[backend::app::services::story::StoryEntry] = index
                .index
                .groups
                .iter()
                .find(|x| x.id == group_id)
                .map_or(&[], |x| x.stories.as_slice());
            let curve = g.depth.clone().unwrap_or_default();
            let exceptions: Vec<String> = curve
                .windows(2)
                .enumerate()
                .filter(|(_, w)| w[1] > w[0])
                .map(|(i, w)| {
                    format!(
                        "slot {} {} rises {} -> {} ({})",
                        i + 1,
                        slots.get(i + 1).map_or("", |s| s.id.as_str()),
                        w[0],
                        w[1],
                        slots.get(i + 1).map_or("", |s| s.name.as_str())
                    )
                })
                .collect();
            println!("{group_id} depth ({} stories): {curve:?}", curve.len());
            println!(
                "  monotone non-increasing: {} ({} exceptions{})",
                exceptions.is_empty(),
                exceptions.len(),
                if exceptions.is_empty() {
                    String::new()
                } else {
                    format!(": {}", exceptions.join(", "))
                }
            );
        }

        // The invariants. Depth is read off the same counters the story rows
        // carry, so the two can never disagree; readers of a group can never
        // exceed the players counted, nor be under its finishers.
        let by_id: HashMap<&str, u32> = out
            .stories
            .iter()
            .map(|s| (s.id.as_str(), s.readers))
            .collect();
        for g in &out.groups {
            assert!(
                g.readers <= out.players,
                "{} has more readers than there are players",
                g.id
            );
            assert!(
                g.finished <= g.readers,
                "{} finished by more accounts than read it",
                g.id
            );
            let stories: &[backend::app::services::story::StoryEntry] = index
                .index
                .groups
                .iter()
                .find(|x| x.id == g.id)
                .map(|x| x.stories.as_slice())
                .or_else(|| {
                    index
                        .index
                        .records
                        .iter()
                        .find(|x| x.char_id == g.id)
                        .map(|x| x.stories.as_slice())
                })
                .expect("every group in the aggregate is a group in the index");
            // A record group carries no curve, by the payload trim; every
            // other group's curve must be exactly its story rows.
            let records = index.index.records.iter().any(|x| x.char_id == g.id);
            if records {
                assert!(g.depth.is_none(), "{} carries a curve it should not", g.id);
            } else {
                let curve = g.depth.as_ref().expect("a library group carries a curve");
                assert_eq!(curve.len(), stories.len(), "{} depth is the wrong length", g.id);
                for (slot, story) in stories.iter().enumerate() {
                    assert_eq!(
                        curve[slot], by_id[story.id.as_str()],
                        "{} slot {} disagrees with the story row",
                        g.id, slot
                    );
                }
            }
        }
        assert_eq!(
            out.stories.len(),
            index.lookup.len(),
            "the aggregate must carry every story the library knows"
        );
        assert!(
            payload.len() < 150 * 1024,
            "the document is {} bytes, over the 150 KB the wire contract allows",
            payload.len()
        );
    });
}
