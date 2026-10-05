//! The character gallery against the real EN tree: every sprite folder, the
//! names the scripts speak it under by the lit-slot rule, and every expression
//! it offers. Skips when the EN scripts or the sprite textures are not on disk
//! (CI's `game-data` artifact carries scripts but no textures).
//!
//! Run with `cargo test --test story_sprites_real_data_test -- --nocapture`.
//! `SPRITES_DUMP=<dir>` also writes the list and three sheets as JSON there,
//! the fixture the frontend's gallery is checked against.

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use backend::app::services::story::{
    StorySpriteEntry, StorySpriteKind, build_index, build_sprite_index,
};
use backend::core::gamedata::assets::AssetIndex;
use backend::core::story::StoryAssetIndex;

fn assets_dir() -> PathBuf {
    let dir = std::env::var("ASSETS_DIR").unwrap_or_else(|_| "../assets/output/en".into());
    Path::new(env!("CARGO_MANIFEST_DIR")).join(dir)
}

fn tree_present(dir: &Path) -> bool {
    dir.join("gamedata/excel/story_review_table.json").exists()
        && dir.join("textures/avg/characters").is_dir()
}

fn top(e: &StorySpriteEntry) -> Option<&str> {
    e.names.first().map(|n| n.name.as_str())
}

fn row(e: &StorySpriteEntry) -> String {
    let names: Vec<String> = e
        .names
        .iter()
        .take(4)
        .map(|n| format!("{} {}", n.name, n.count))
        .collect();
    format!(
        "{} kind={:?} char={:?} variant={:?} lines={} stories={} variants={} thumb={:?} names=[{}]",
        e.base,
        e.kind,
        e.char_id,
        e.variant,
        e.lines,
        e.story_count,
        e.variant_count,
        e.thumb.as_ref().map(|t| (&t.key, t.uses)),
        names.join(", ")
    )
}

#[test]
fn the_gallery_names_every_folder_by_the_lit_slot_rule() {
    let dir = assets_dir();
    if !tree_present(&dir) {
        eprintln!(
            "story_sprites_real_data_test: no EN story tree with sprites at {}, skipping",
            dir.display()
        );
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let library = build_index(gd, &asset_index, &dir);
    let assets = StoryAssetIndex::for_dir(&dir, &asset_index);

    let started = Instant::now();
    let (index, details, census) = build_sprite_index(gd, &library, &assets, &dir);
    let build_ms = started.elapsed().as_millis();

    let sprites = &index.sprites;
    let operators = sprites
        .iter()
        .filter(|s| s.kind == StorySpriteKind::Operator)
        .count();
    let named = sprites.iter().filter(|s| !s.names.is_empty()).count();
    let shown = sprites.iter().filter(|s| s.story_count > 0).count();
    let shown_unnamed = sprites
        .iter()
        .filter(|s| s.story_count > 0 && s.names.is_empty())
        .count();
    let variants: usize = details.values().map(|d| d.variants.len()).sum();
    let used_variants: usize = details
        .values()
        .map(|d| d.variants.iter().filter(|v| v.uses > 0).count())
        .sum();
    println!(
        "gallery: {} folders ({} operator, {} npc); {} named, {} unnamed; {} shown by a script, \
         {} of those unnamed, {} never shown; {} expressions ({} used); build {} ms",
        sprites.len(),
        operators,
        sprites.len() - operators,
        named,
        sprites.len() - named,
        shown,
        shown_unnamed,
        sprites.len() - shown,
        variants,
        used_variants,
        build_ms
    );
    println!("census: {census:?}");

    let list = serde_json::to_vec(&index).unwrap();
    let all_details: Vec<_> = details.values().collect();
    let detail_bytes = serde_json::to_vec(&all_details).unwrap();
    let mut per: Vec<usize> = details
        .values()
        .map(|d| serde_json::to_vec(d).unwrap().len())
        .collect();
    per.sort_unstable();
    println!(
        "payload: list {} bytes; every sheet together {} bytes; one sheet median {} max {} bytes",
        list.len(),
        detail_bytes.len(),
        per[per.len() / 2],
        per[per.len() - 1]
    );

    let by: HashMap<&str, &StorySpriteEntry> =
        sprites.iter().map(|s| (s.base.as_str(), s)).collect();
    for base in [
        "avg_npc_043_1",
        "avg_npc_2125_1",
        "avg_npc_2127_1",
        "char_002_amiya_1",
        "avg_474_gladiia_1",
    ] {
        if let Some(e) = by.get(base) {
            println!("{}", row(e));
        }
    }
    let jie: Vec<&StorySpriteEntry> = sprites.iter().filter(|s| top(s) == Some("Jie")).collect();
    for e in &jie {
        println!("top name Jie: {}", row(e));
    }
    let any_jie: Vec<String> = sprites
        .iter()
        .filter(|s| s.names.iter().any(|n| n.name == "Jie"))
        .map(|s| {
            let n = s.names.iter().find(|n| n.name == "Jie").unwrap();
            format!("{} ({} of {})", s.base, n.count, s.lines)
        })
        .collect();
    println!("folders with any Jie line: {}", any_jie.join("; "));

    if let Ok(out) = std::env::var("SPRITES_DUMP") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("sprites.json"), &list).unwrap();
        for base in ["avg_npc_043_1", "avg_npc_2125_1", "char_002_amiya_1"] {
            let d = &details[base];
            std::fs::write(
                out.join(format!("{base}.json")),
                serde_json::to_vec(d).unwrap(),
            )
            .unwrap();
        }
        println!("dumped to {}", out.display());
    }

    assert!(
        sprites.len() >= 1_600,
        "{} folders; the EN extract carries 1,636",
        sprites.len()
    );

    let nine = by["avg_npc_043_1"];
    assert_eq!(top(nine), Some("Nine"), "{}", row(nine));
    // Her lit art is `#2` by the legacy hub's own order, and it is the one
    // the scripts put up most, so it is the card.
    assert_eq!(nine.thumb.as_ref().map(|t| t.key.as_str()), Some("#2$1"));

    assert_eq!(
        jie.len(),
        1,
        "Jie must be the primary name of exactly one folder"
    );
    assert_eq!(jie[0].base, "avg_npc_2125_1", "{}", row(jie[0]));
    assert_ne!(top(by["avg_npc_2127_1"]), Some("Jie"));

    let amiya = by["char_002_amiya_1"];
    assert_eq!(amiya.kind, StorySpriteKind::Operator);
    assert_eq!(amiya.char_id.as_deref(), Some("char_002_amiya"));
    assert_eq!(amiya.operator_name.as_deref(), Some("Amiya"));
    assert_eq!(amiya.variant.as_deref(), Some("1"));
    assert_eq!(top(amiya), Some("Amiya"), "{}", row(amiya));

    let gladiia = by["avg_474_gladiia_1"];
    assert_eq!(gladiia.char_id.as_deref(), Some("char_474_glady"));

    // Every expression of every sheet resolves, and a sheet lists each pair
    // of files once.
    for d in details.values() {
        let mut seen = std::collections::HashSet::new();
        for v in &d.variants {
            assert!(
                seen.insert((&v.sprite.body_url, &v.sprite.face_url)),
                "{} lists {} twice",
                d.sprite.base,
                v.key
            );
        }
    }
}

/// Prices the one choice the lit-slot rule leaves open: whether a shown
/// cut-in or interlude window is the WHOLE answer for a line, or joins the lit
/// slots as one more candidate. Each policy is scored by how much of its
/// plate-time weight lands on a folder whose primary name IS the line's
/// speaker, the primaries taken from plate-free lines only so neither policy
/// grades itself.
#[test]
fn plates_over_the_stage_priced_both_ways() {
    use backend::core::story::speakers::{PlateRule, SpeakerRules, attribute_speakers_with};
    use backend::core::story::{load_script, parser};

    let dir = assets_dir();
    if !tree_present(&dir) {
        eprintln!("story_sprites_real_data_test: no EN tree, skipping");
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let library = build_index(gd, &asset_index, &dir);
    let assets = StoryAssetIndex::for_dir(&dir, &asset_index);

    let mut scripts = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for g in &library.index.groups {
        for s in &g.stories {
            if let Some(r) = library.lookup.get(&s.id)
                && seen.insert(r.story_txt.clone())
                && let Ok(text) = load_script(&dir, &r.story_txt)
            {
                scripts.push(parser::parse(&text));
            }
        }
    }
    let folder_of = |raw: &str| assets.sprite_folder_name(raw).map(str::to_owned);
    let walk_all = |rules: SpeakerRules| {
        scripts
            .iter()
            .map(|c| attribute_speakers_with(c, rules, folder_of))
            .collect::<Vec<_>>()
    };
    let exclusive = walk_all(SpeakerRules {
        plates: PlateRule::Exclusive,
    });
    let joined = walk_all(SpeakerRules {
        plates: PlateRule::Join,
    });
    let fallback = walk_all(SpeakerRules {
        plates: PlateRule::Fallback,
    });

    let mut votes: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for w in &exclusive {
        for l in w.lines.iter().filter(|l| l.plate == 0) {
            for (f, x) in &l.sprites {
                *votes
                    .entry(f.clone())
                    .or_default()
                    .entry(l.speaker.clone())
                    .or_default() += x;
            }
        }
    }
    let primary: HashMap<String, String> = votes
        .into_iter()
        .filter_map(|(f, names)| {
            names
                .into_iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(n, _)| (f, n))
        })
        .collect();
    let score = |walks: &[backend::core::story::speakers::SpeakerWalk], mask: u8| {
        let (mut lines, mut hit, mut total, mut unattributed) = (0_u32, 0.0_f64, 0.0_f64, 0_u32);
        for w in walks {
            for l in w.lines.iter().filter(|l| l.plate & mask != 0) {
                lines += 1;
                if l.sprites.is_empty() {
                    unattributed += 1;
                }
                for (f, x) in &l.sprites {
                    total += x;
                    if primary.get(f) == Some(&l.speaker) {
                        hit += x;
                    }
                }
            }
        }
        (lines, unattributed, hit, total)
    };
    let mut n = 0;
    for (label, mask) in [("any plate", 3_u8), ("cut-in", 1), ("window", 2)] {
        let (lines, u1, h1, t1) = score(&exclusive, mask);
        let (_, u2, h2, t2) = score(&joined, mask);
        let (_, u3, h3, t3) = score(&fallback, mask);
        if mask == 3 {
            n = lines;
        }
        println!(
            "{label}: {lines} lines; exclusive {h1:.1}/{t1:.1} = {:.4} ({u1} unattributed); \
             joined {h2:.1}/{t2:.1} = {:.4} ({u2}); fallback {h3:.1}/{t3:.1} = {:.4} ({u3})",
            h1 / t1,
            h2 / t2,
            h3 / t3
        );
    }
    assert!(n > 0);
}

/// Prices the per-expression thumb on the gallery's real first page: the 60
/// cards of the default sort (most stories, then lines, then name), each
/// card's own expression composed at 320 px high. Prints the bytes the page
/// fetched before (full body plus face PNGs) and after (one thumb each), the
/// cold build time, and lossless WebP against PNG on the first 20.
#[test]
fn first_page_thumbs_priced() {
    use backend::app::services::story::sprite_thumbs::{THUMB_H, ThumbFormat, compose, encode};

    let dir = assets_dir();
    if !tree_present(&dir) {
        eprintln!("story_sprites_real_data_test: no EN tree, skipping");
        return;
    }
    let gd = common::load_game_data();
    let asset_index = Arc::new(AssetIndex::build(&dir));
    let library = build_index(gd, &asset_index, &dir);
    let assets = StoryAssetIndex::for_dir(&dir, &asset_index);
    let (index, _, census) = build_sprite_index(gd, &library, &assets, &dir);
    println!(
        "hidden folders: {} {:?}",
        census.hidden.len(),
        census.hidden
    );
    assert!(census.hidden.iter().any(|h| h == "char_empty"));
    assert!(index.sprites.iter().all(|s| s.base != "char_empty"));

    let mut page: Vec<&StorySpriteEntry> = index.sprites.iter().collect();
    let name =
        |e: &StorySpriteEntry| top(e).map_or_else(|| e.base.to_lowercase(), str::to_lowercase);
    page.sort_by(|a, b| {
        b.story_count
            .cmp(&a.story_count)
            .then(b.lines.total_cmp(&a.lines))
            .then_with(|| name(a).cmp(&name(b)))
    });
    page.truncate(60);
    let path = |url: &str| dir.join(url.trim_start_matches('/').replace("%23", "#"));
    let mut before_files = std::collections::HashSet::new();
    let (mut webp_bytes, mut webp20, mut png20) = (0_usize, 0_usize, 0_usize);
    let mut compose_ms = 0_u128;
    let mut webp_ms = 0_u128;
    let started = Instant::now();
    for (i, e) in page.iter().enumerate() {
        let v = e
            .thumb
            .as_ref()
            .expect("a listed folder has a card expression");
        before_files.insert(path(&v.sprite.body_url));
        let face = v.sprite.face_url.as_deref().map(path);
        if let Some(f) = &face {
            before_files.insert(f.clone());
        }
        let t = Instant::now();
        let body = std::fs::read(path(&v.sprite.body_url)).unwrap();
        let face_png = face.map(|f| std::fs::read(f).unwrap());
        let img = compose(&body, face_png.as_deref(), v, THUMB_H).unwrap();
        compose_ms += t.elapsed().as_millis();
        assert_eq!(img.height(), THUMB_H);
        let t = Instant::now();
        let w = encode(&img, ThumbFormat::Webp).unwrap().len();
        webp_ms += t.elapsed().as_millis();
        webp_bytes += w;
        if i < 4
            && let Ok(out) = std::env::var("SPRITE_THUMB_OUT")
        {
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(
                Path::new(&out).join(format!("{}.png", e.base)),
                encode(&img, ThumbFormat::Png).unwrap(),
            )
            .unwrap();
        }
        if i < 20 {
            webp20 += w;
            png20 += encode(&img, ThumbFormat::Png).unwrap().len();
        }
    }
    let total_ms = started.elapsed().as_millis();
    let before: u64 = before_files
        .iter()
        .map(|f| std::fs::metadata(f).unwrap().len())
        .sum();
    println!(
        "first page of {}: before {} files {} bytes; after {} webp thumbs {} bytes; \
         cold build {} ms (compose {} ms, webp encode {} ms, the rest PNG encodes of the first 20)",
        page.len(),
        before_files.len(),
        before,
        page.len(),
        webp_bytes,
        total_ms,
        compose_ms,
        webp_ms
    );
    println!("first 20: lossless webp {webp20} bytes, png {png20} bytes");
}
