// Clippy: pedantic/nursery noise intentional for this asset-pipeline tool
// (byte/bit casts, long parser functions, single-char math vars, indicatif
// template strings, internal-only docs). See lib.rs for the rationale.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_long_first_doc_paragraph,
    clippy::implicit_hasher,
    clippy::option_if_let_else,
    clippy::manual_let_else,
    clippy::match_same_arms,
    clippy::items_after_statements,
    clippy::needless_pass_by_value,
    clippy::branches_sharing_code,
    clippy::or_fun_call,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::struct_excessive_bools,
    clippy::fn_params_excessive_bools,
    clippy::many_single_char_names,
    clippy::unreadable_literal,
    clippy::format_push_string,
    clippy::literal_string_with_formatting_args
)]

mod cli;

use unpacker::export;
use unpacker::unity;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use base64::Engine;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

use cli::{Cli, Command};
use export::alpha_merge;
use export::audio::export_audio;
use export::avg_hub;
use export::avg_sprites;
use export::portrait;
use export::spine;
use export::stage_preview::{self, StageAspectMap};
use export::story_art;
use export::text_asset::export_text_asset;
use export::texture::{decode_texture_object, save_decoded_texture};
use unity::bundle::BundleFile;
use unity::object_reader::read_object;
use unity::serialized_file::SerializedFile;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Extract(args) => cmd_extract(&args),
        Command::List(args) => cmd_list(&args),
        Command::Verify(args) => cmd_verify(&args),
        Command::BackfillHubs(args) => cmd_backfill_hubs(&args),
        Command::BackfillSprites(args) => cmd_backfill_sprites(&args),
        Command::BackfillStoryArt(args) => cmd_backfill_story_art(&args),
    }
}

/// Write the Story Collection's art for one server without re-extracting
/// anything else. The `spritepack/mixstory_*` bundles are plain `Texture2D` +
/// full-rect `Sprite` pairs, one texture per sprite, so this is the ordinary
/// texture pass narrowed to those bundles: decode, alpha-merge, write
/// `textures/spritepack/<bundle>/<m_Name>.png`. The report is per KIND, by
/// stem, because each stem answers a different `stage_table` field.
fn cmd_backfill_story_art(args: &cli::BackfillStoryArtArgs) {
    let started = std::time::Instant::now();
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }
    let mut bundles: Vec<PathBuf> = WalkDir::new(args.input.join("spritepack"))
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e == "ab"))
        .filter(|p| {
            let sub = p.strip_prefix(&args.input).unwrap_or(p).with_extension("");
            story_art::is_story_art_bundle(&sub)
        })
        .collect();
    bundles.sort();
    if bundles.is_empty() {
        eprintln!(
            "error: no spritepack/mixstory_* bundles under {}",
            args.input.display()
        );
        std::process::exit(1);
    }
    println!("backfilling story art from {} bundles", bundles.len());

    let rows: Vec<(PathBuf, usize, Vec<String>)> = bundles
        .par_iter()
        .map(|path| {
            let sub = path
                .strip_prefix(&args.input)
                .unwrap_or(path)
                .with_extension("");
            let Ok(data) = std::fs::read(path) else {
                return (sub, 0, Vec::new());
            };
            let Ok(bundle) = BundleFile::parse(data) else {
                return (sub, 0, Vec::new());
            };
            let mut resources: HashMap<String, Vec<u8>> = HashMap::new();
            for entry in &bundle.files {
                if is_resource_entry(&entry.path) {
                    let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
                    resources.insert(filename.to_string(), entry.data.clone());
                }
            }
            let mut textures: HashMap<String, export::texture::DecodedTexture> = HashMap::new();
            for entry in &bundle.files {
                if is_resource_entry(&entry.path) {
                    continue;
                }
                let Ok(sf) = SerializedFile::parse(entry.data.clone()) else {
                    continue;
                };
                for obj in &sf.objects {
                    if obj.class_id != 28 {
                        continue;
                    }
                    let Ok(val) = read_object(&sf, obj) else {
                        continue;
                    };
                    if let Ok(Some(tex)) = decode_texture_object(&val, &resources) {
                        textures.insert(tex.name.clone(), tex);
                    }
                }
            }
            let dir = args.output.join("textures").join(&sub);
            std::fs::create_dir_all(&dir).ok();
            // The names BEFORE the merge folds `<name>[alpha]` into `<name>`,
            // so the census counts plates, not companions.
            let mut stems: Vec<String> = textures
                .keys()
                .filter(|n| !n.ends_with("[alpha]"))
                .cloned()
                .collect();
            stems.sort();
            let written = if args.no_merge {
                let mut n = 0;
                for tex in textures.values() {
                    match save_decoded_texture(tex, &dir) {
                        Ok(()) => n += 1,
                        Err(e) => eprintln!("  error saving {}: {e}", tex.name),
                    }
                }
                n
            } else {
                alpha_merge::merge_and_export(textures, &dir)
            };
            (sub, written, stems)
        })
        .collect();

    let mut written = 0usize;
    let mut per_kind: HashMap<&'static str, usize> = HashMap::new();
    let mut empty: Vec<String> = Vec::new();
    for (sub, n, stems) in rows {
        if n == 0 {
            empty.push(sub.display().to_string());
            continue;
        }
        written += n;
        println!("  {} -> {n} files", sub.display());
        for stem in &stems {
            *per_kind.entry(story_art::art_kind(stem)).or_default() += 1;
        }
    }
    println!("story art written: {written} files");
    let mut kinds: Vec<(&str, usize)> = per_kind.into_iter().collect();
    kinds.sort_unstable_by_key(|(k, n)| (std::cmp::Reverse(*n), *k));
    for (kind, n) in kinds {
        println!("  {kind}: {n}");
    }
    if !empty.is_empty() {
        println!("bundles that yielded no texture: {empty:?}");
    }
    println!("wall time {:.1} s", started.elapsed().as_secs_f64());
}

/// Write the story image trees' `sprites.json` files for one server without
/// re-extracting a single texture. Reports folders written, sprite entries and
/// the pixels-per-unit census, because the PPU is what sizes a plate: an
/// absent `screenadapt` is `SetNativeSize` at the sprite's own PPU against the
/// canvas scaler's reference 100.
fn cmd_backfill_sprites(args: &cli::BackfillSpritesArgs) {
    let started = std::time::Instant::now();
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }
    let mut bundles: Vec<PathBuf> = ["avg", "spritepack"]
        .iter()
        .flat_map(|root| WalkDir::new(args.input.join(root)).min_depth(1))
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e == "ab"))
        .filter(|p| {
            let sub = p.strip_prefix(&args.input).unwrap_or(p).with_extension("");
            avg_sprites::is_avg_image_bundle(&sub)
        })
        .collect();
    bundles.sort();
    if bundles.is_empty() {
        eprintln!(
            "error: no story image bundles under {}",
            args.input.display()
        );
        std::process::exit(1);
    }
    println!("backfilling sprites.json for {} bundles", bundles.len());

    let rows: Vec<(PathBuf, avg_sprites::SpriteMetaMap, bool, bool)> = bundles
        .par_iter()
        .map(|path| {
            let sub = path
                .strip_prefix(&args.input)
                .unwrap_or(path)
                .with_extension("");
            let dir = args.output.join("textures").join(&sub);
            let exists = dir.is_dir();
            let sprites = std::fs::read(path)
                .ok()
                .and_then(|d| BundleFile::parse(d).ok())
                .as_ref()
                .map(avg_sprites::sprites_from_bundle)
                .unwrap_or_default();
            let ok = if !sprites.is_empty()
                && (exists || args.create_missing)
                && let Err(e) = avg_sprites::write_sprites_json(&dir, &sprites)
            {
                eprintln!("  error writing {}: {e}", dir.display());
                false
            } else {
                true
            };
            (sub, sprites, exists, ok)
        })
        .collect();

    let mut written = 0usize;
    let mut entries = 0usize;
    let mut empty = 0usize;
    let mut no_folder: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    // The PPU value as written, keyed to 4 decimals so 68.2464 is one bucket.
    let mut ppu: HashMap<String, usize> = HashMap::new();
    let mut off_pivot = 0usize;
    for (sub, sprites, exists, ok) in rows {
        let label = sub.display().to_string();
        if sprites.is_empty() {
            empty += 1;
            continue;
        }
        if !exists && !args.create_missing {
            no_folder.push(label);
            continue;
        }
        if !ok {
            failed.push(label);
            continue;
        }
        written += 1;
        entries += sprites.len();
        for meta in sprites.values() {
            *ppu.entry(format!("{:.4}", meta.ppu)).or_default() += 1;
            if (meta.pivot.x - 0.5).abs() > f32::EPSILON
                || (meta.pivot.y - 0.5).abs() > f32::EPSILON
            {
                off_pivot += 1;
            }
        }
    }
    println!(
        "sprites.json written to {written} folders, {entries} sprite entries; {empty} bundles carry no sprite, {} have no output folder, {} failed to write",
        no_folder.len(),
        failed.len()
    );
    let mut rows: Vec<(String, usize)> = ppu.into_iter().collect();
    rows.sort_unstable_by_key(|(v, n)| (std::cmp::Reverse(*n), v.clone()));
    let top: Vec<String> = rows
        .iter()
        .take(8)
        .map(|(v, n)| format!("{v} x {n}"))
        .collect();
    let shown: usize = rows.iter().take(8).map(|(_, n)| n).sum();
    println!(
        "pixels per unit: {} distinct values; {}; the other {} values cover {} entries",
        rows.len(),
        top.join(", "),
        rows.len().saturating_sub(8),
        entries - shown
    );
    println!("sprite entries whose pivot is not (0.5,0.5): {off_pivot}");
    if !no_folder.is_empty() {
        let head: Vec<&String> = no_folder.iter().take(10).collect();
        println!("first folders with no PNGs on disk: {head:?}");
    }
    println!("wall time {:.1} s", started.elapsed().as_secs_f64());
}

/// Write the sprite-hub `hub.json` files for one server without re-extracting
/// a single texture. Reports the same census the phase-4 report quotes:
/// folders written, groups per folder, how many hubs are legacy sentinels.
fn cmd_backfill_hubs(args: &cli::BackfillHubsArgs) {
    let started = std::time::Instant::now();
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }
    let root = args.input.join("avg/characters");
    let mut bundles: Vec<PathBuf> = WalkDir::new(&root)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e == "ab"))
        .collect();
    bundles.sort();
    if bundles.is_empty() {
        eprintln!("error: no .ab bundles under {}", root.display());
        std::process::exit(1);
    }
    println!("backfilling hubs for {} bundles", bundles.len());

    #[derive(Default)]
    struct Census {
        written: usize,
        no_hub: usize,
        no_folder: Vec<String>,
        failed: Vec<String>,
        legacy: usize,
        sentinel: usize,
        groups: HashMap<usize, usize>,
        sprites: usize,
        aliased: usize,
        whole_body: usize,
        unnamed: usize,
        /// Texture px -> how many `isWholeBody` entries are that square.
        body_px: HashMap<i64, usize>,
        /// Texture px -> how many face patches are that square.
        face_px: HashMap<i64, usize>,
        sized: usize,
        /// Every written hub's root-rect WIDTH in canvas px, for the census.
        root_w: Vec<f32>,
        /// Hubs whose bundle carries no root `RectTransform` to read.
        no_root: usize,
        /// Root rects that are not square, which the slot template never is.
        root_oblong: usize,
    }

    let rows: Vec<(String, Option<export::avg_hub::Hub>, bool, bool)> = bundles
        .par_iter()
        .map(|path| {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_owned();
            let dir = args.output.join("textures/avg/characters").join(&stem);
            let exists = dir.is_dir();
            let hub = std::fs::read(path)
                .ok()
                .and_then(|d| BundleFile::parse(d).ok())
                .as_ref()
                .and_then(export::avg_hub::hub_from_bundle);
            let ok = if let Some(h) = hub.as_ref()
                && (exists || args.create_missing)
                && let Err(e) = export::avg_hub::write_hub_json(&dir, h)
            {
                eprintln!("  error writing {}: {e}", dir.display());
                false
            } else {
                true
            };
            (stem, hub, exists, ok)
        })
        .collect();

    let mut c = Census::default();
    for (stem, hub, exists, ok) in rows {
        let Some(hub) = hub else {
            c.no_hub += 1;
            continue;
        };
        if !exists && !args.create_missing {
            c.no_folder.push(stem);
            continue;
        }
        if !ok {
            c.failed.push(stem);
            continue;
        }
        c.written += 1;
        let stem_lc = stem.to_ascii_lowercase();
        if hub.legacy {
            c.legacy += 1;
        }
        if hub
            .groups
            .iter()
            .all(export::avg_hub::HubGroup::is_sentinel)
        {
            c.sentinel += 1;
        }
        match hub.root {
            Some(r) => {
                c.root_w.push(r.w);
                if (r.w - r.h).abs() > f32::EPSILON {
                    c.root_oblong += 1;
                }
            }
            None => c.no_root += 1,
        }
        *c.groups.entry(hub.groups.len()).or_default() += 1;
        for g in &hub.groups {
            c.sprites += g.sprites.len();
            c.aliased += g.sprites.iter().filter(|s| !s.alias.is_empty()).count();
            c.whole_body += g.sprites.iter().filter(|s| s.is_whole_body).count();
            c.unnamed += g.sprites.iter().filter(|s| s.name.is_empty()).count();
            // A BODY is the entry named after the bundle itself
            // (`avg_1037_amiya3_1$1`); everything else in the group is a face
            // patch. The `isWholeBody` flag is NOT that split: only 460 of
            // 12,114 entries carry it, while 2,130 entries are bodies.
            for sprite in &g.sprites {
                let Some(size) = sprite.size else { continue };
                c.sized += 1;
                let px = size.w.max(size.h) as i64;
                let bucket = if sprite.name.to_ascii_lowercase().starts_with(&stem_lc) {
                    &mut c.body_px
                } else {
                    &mut c.face_px
                };
                *bucket.entry(px).or_default() += 1;
            }
        }
    }
    let mut groups: Vec<(usize, usize)> = c.groups.into_iter().collect();
    groups.sort_unstable();
    let dist: Vec<String> = groups
        .iter()
        .map(|(n, k)| format!("{n} group(s) x {k}"))
        .collect();
    println!(
        "hub.json written to {} folders; {} bundles carry no hub; {} have no output folder; {} failed to write",
        c.written,
        c.no_hub,
        c.no_folder.len(),
        c.failed.len()
    );
    println!("groups per folder: {}", dist.join(", "));
    println!(
        "legacy `AVGCharacterSpriteHub` class {}; hubs whose every group is the (-1,-1)/(0,0) sentinel {}",
        c.legacy, c.sentinel
    );
    println!(
        "sprite entries {}, of which {} carry an alias, {} are flagged isWholeBody, {} have no in-bundle Sprite",
        c.sprites, c.aliased, c.whole_body, c.unnamed
    );
    let census = |rows: &Vec<(i64, usize)>| {
        rows.iter()
            .map(|(px, n)| format!("{px}px x {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let head = |m: &HashMap<i64, usize>, n: usize| {
        let mut rows: Vec<(i64, usize)> = m.iter().map(|(k, v)| (*k, *v)).collect();
        rows.sort_unstable_by_key(|(px, n)| (std::cmp::Reverse(*n), *px));
        let total: usize = rows.iter().map(|(_, n)| n).sum();
        let shown: usize = rows.iter().take(n).map(|(_, n)| n).sum();
        format!(
            "{total} over {} distinct sizes; {}, the other {} sizes {}",
            rows.len(),
            census(&rows.iter().take(n).copied().collect()),
            rows.len().saturating_sub(n),
            total - shown
        )
    };
    println!(
        "sprite entries carrying an m_Rect size {} of {}; bodies: {}",
        c.sized,
        c.sprites,
        head(&c.body_px, 6)
    );
    println!("face patches: {}", head(&c.face_px, 6));
    // The prefab's own root rect, which BEATS the 1024 slot template: the
    // spread is what the character-size fix is worth.
    let mut widths = c.root_w;
    widths.sort_unstable_by(f32::total_cmp);
    if widths.is_empty() {
        println!("root rects: none read");
    } else {
        let at_1024 = widths.iter().filter(|w| (**w - 1024.0).abs() < 0.5).count();
        let median = widths[widths.len() / 2];
        println!(
            "root rects: {} read, {} carry none; {at_1024} are the 1024 slot template, min {}, max {}, median {median}; {} not square",
            widths.len(),
            c.no_root,
            widths[0],
            widths[widths.len() - 1],
            c.root_oblong
        );
    }
    if !c.no_folder.is_empty() {
        let head: Vec<&String> = c.no_folder.iter().take(10).collect();
        println!("first folders with no PNGs on disk: {head:?}");
    }
    println!("wall time {:.1} s", started.elapsed().as_secs_f64());
}

fn cmd_extract(args: &cli::ExtractArgs) {
    let should_extract_gamedata = args.gamedata || args.extract_all();

    if should_extract_gamedata {
        let idx_path = match &args.idx {
            Some(p) => p.clone(),
            None => match find_idx_file(&args.input) {
                Some(p) => {
                    println!("Auto-detected manifest: {}", p.display());
                    p
                }
                None => {
                    if args.gamedata {
                        // Only error if user explicitly requested gamedata
                        eprintln!("error: no .idx manifest found; use --idx <manifest.idx>");
                        std::process::exit(1);
                    } else {
                        // Silent skip when auto-extracting everything
                        println!("No .idx manifest found, skipping gamedata extraction");
                        std::path::PathBuf::new()
                    }
                }
            },
        };
        if !idx_path.as_os_str().is_empty() {
            std::fs::create_dir_all(&args.output).unwrap();
            // Exit status is load-bearing here. run.mjs treats exit 0 as a clean
            // extract and then runs sweepOrphans, which deletes every file in a
            // touched subtree that this run did not rewrite. Reporting success
            // after failing to write N tables is therefore an instruction to
            // delete those N tables, having just proved the disk cannot be
            // written. Fail instead: the watcher keeps the old output, records no
            // new stamp, and arms its backoff.
            match export::gamedata::export_gamedata(&args.input, &idx_path, &args.output) {
                Ok((count, 0)) => println!("Exported {count} gamedata files"),
                Ok((count, failed)) => {
                    eprintln!(
                        "error: exported {count} gamedata files but {failed} failed to write; \
                         refusing to report success (the output tree still holds the previous copies)"
                    );
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        if !args.extract_all()
            && !args.image
            && !args.text
            && !args.audio
            && !args.spine
            && !args.portrait
        {
            return;
        }
    }

    // Configure rayon thread pool
    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }

    // Collect all files from input directory
    let files: Vec<_> = WalkDir::new(&args.input)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .collect();

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{bar:40}] {pos}/{len} ({per_sec}) {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    let extract_image = args.extract_all() || args.image;
    let extract_text = args.extract_all() || args.text;
    let extract_audio = args.extract_all() || args.audio;
    let extract_spine = args.extract_all() || args.spine;
    let extract_portrait = args.extract_all() || args.portrait;
    let merge_alpha = !args.no_merge && extract_image;

    // Pre-pass: build the stage_id → tile-grid-dims map needed to unsquash
    // `stage_mappreview_h2_*` thumbnails back to their natural aspect. Skipped
    // entirely when image extraction is off or when the input has no
    // mappreview bundles.
    let stage_aspects: StageAspectMap =
        if extract_image && stage_preview::input_has_mappreview_bundles(&files) {
            println!("Scanning level data for mappreview aspect ratios...");
            let map = stage_preview::build_stage_aspect_map(&files);
            println!("  found {} stage(s) with grid metadata", map.len());
            map
        } else {
            std::sync::Arc::new(std::collections::HashMap::new())
        };

    // Pre-pass: resolve shader names from any shader bundles in the input, so
    // dynchar particle materials (whose shader is an external ref into
    // `[uc]shaders.ab`) can be classified — notably the procedural
    // `VertexDisturb(CustomData)` targeting-ring reticles. Empty (feature off)
    // when no shader bundles are present.
    let shader_map = if extract_spine {
        let map = export::shader_map::build_shader_map(&files);
        if !map.is_empty() {
            println!(
                "Resolved {} shader name(s) for particle classification",
                map.len()
            );
        }
        // A shader's DECLARED property block — the only evidence for which colour
        // property a material is actually drawn with. Materials keep residue from
        // whatever shader they were authored against, so `m_SavedProperties` alone
        // mis-attributes 630 of them corpus-wide (`legacy_tint_scale`).
        let (props, ranges) = export::shader_map::build_shader_props(&files);
        if !props.is_empty() {
            println!(
                "Read declared properties for {} shader(s), {} Range limits",
                props.len(),
                ranges.len()
            );
        }
        export::shader_map::set_shader_props(props);
        export::shader_map::set_shader_ranges(ranges);
        map
    } else {
        std::collections::HashMap::new()
    };

    // Pre-pass: load the shared FX texture bundles (`refs/fx/*`) so dynchar
    // particle emitters that reference them externally (sparkles, streaks,
    // smoke, flow — currently `tex: null` and invisible) render their real
    // sprite. Empty (feature off) when the FX bundles aren't in the input.
    let fx_textures = if extract_spine {
        let fx = export::fx_textures::FxTextures::build(&files);
        if !fx.is_empty() {
            println!("Loaded shared FX texture bundles for external-texture resolution");
        }
        fx
    } else {
        export::fx_textures::FxTextures::default()
    };

    let exported = AtomicUsize::new(0);

    std::fs::create_dir_all(&args.output).unwrap();

    let input_dir = &args.input;
    files.par_iter().for_each(|file_path| {
        let count = process_bundle(
            file_path,
            input_dir,
            &args.output,
            extract_image,
            extract_text,
            extract_audio,
            extract_spine,
            extract_portrait,
            merge_alpha,
            &stage_aspects,
            &shader_map,
            &fx_textures,
        );
        if count > 0 {
            let prev = exported.fetch_add(count, Ordering::Relaxed);
            let new_total = prev + count;
            // Print periodic progress to stdout for external tools (e.g., run.mjs)
            if new_total / 500 > prev / 500 {
                pb.suspend(|| println!("progress: {new_total} assets"));
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("done");
    let total = exported.load(Ordering::Relaxed);
    println!("Exported {total} assets");
    let duplicates = spine::duplicate_count();
    if duplicates > 0 {
        println!("{duplicates} skeleton path(s) written twice with identical bytes");
    }
    if extract_spine {
        // Regenerate the derived card placement fields into the fresh scene JSONs so an
        // install of this export cannot silently revert fielded cards to square.
        //
        // BEFORE the collision check, not after. This ran after the `exit(1)` until
        // 2026-09-10, so every collision-bearing export left all 72 scene JSONs with no
        // `backdropScale`/`backdropOffsetPx` at all: the abort skipped the merge. Keeping it
        // first means the fatal arm changes only the exit code, never the tree.
        export::cardfields::merge(&args.output);
    }
    let deduped = spine::deduped_count();
    if deduped > 0 {
        // Two assets in ONE bundle resolved to one output path and the collector kept the
        // larger. Reported, never fatal: it is upstream repeating a name, and the CN corpus
        // does it on seven enemies, five of which are the same rig exported twice.
        println!("{deduped} duplicate spine asset(s) dropped within a bundle");
        for line in spine::dedup_report() {
            println!("  {line}");
        }
    }
    let folded = export::texture::texture_collision_report();
    if !folded.is_empty() {
        // Two textures whose names differ only in case, on a filesystem that folds case: one
        // survives (the larger, then the byte-smaller name) and the other is named here. A
        // case-sensitive filesystem writes both and never reaches this.
        println!(
            "{} texture(s) dropped because the output filesystem folds their names onto another's",
            folded.len()
        );
        for line in folded {
            println!("  {line}");
        }
    }
    let collisions = spine::collision_count();
    if collisions > 0 {
        // Two assets resolved to one skeleton path with different bytes. The winner is the
        // lexicographically smaller source bundle, so the tree is reproducible, and the
        // loser is named here rather than only at the moment it happened: `run.mjs` reports
        // just the last 10 lines of output, so a line printed thousands of lines earlier
        // never reaches the operator.
        //
        // This is reported, NOT fatal. Every case in the CN corpus is upstream shipping one
        // asset name twice (an enemy repeated inside one `enm_art` pack, a skin's battle
        // skeleton under the base operator's name), and a duplicated upstream name must not
        // abort a whole region update. `SPINE_COLLISIONS_FATAL=1` restores the exit(1).
        eprintln!("{collisions} skeleton path collision(s): a second asset lost its path");
        for line in spine::collision_report() {
            eprintln!("  {line}");
        }
        if std::env::var("SPINE_COLLISIONS_FATAL").is_ok() {
            std::process::exit(1);
        }
    }
}

/// Search input dir and its parent for a .idx manifest file
/// One schema type's aggregated verification verdict across every gamedata
/// file that resolved to it.
#[derive(Default)]
struct VerifyRow {
    files: usize,
    cn_fail: usize,
    cn_err: Option<String>,
    /// `None` until a file with a Yostar variant is seen.
    yostar: Option<(usize, Option<String>)>,
    chosen: std::collections::BTreeMap<&'static str, usize>,
}

/// Report which schema every gamedata table in a bundle dir verifies against.
///
/// Verification only: nothing is decoded and nothing is written. The `chosen`
/// column comes from `flatbuffers_decode::verify_table`, which asks
/// `select_schema_by_verification` itself, so this cannot drift from what
/// `extract` does. Rows are aggregated per schema type because `level_data`
/// alone is ~2900 files. Always exits 0 — it is a report, not a gate.
fn cmd_verify(args: &cli::VerifyArgs) {
    let idx_path = if let Some(p) = args.idx.clone().or_else(|| find_idx_file(&args.input)) {
        p
    } else {
        eprintln!("error: no .idx manifest found; use --idx <manifest.idx>");
        return;
    };
    let manifest = match export::manifest::ResourceManifest::load(&idx_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: cannot load {}: {e}", idx_path.display());
            return;
        }
    };

    let mut rows: std::collections::BTreeMap<&'static str, VerifyRow> =
        std::collections::BTreeMap::new();
    let mut skipped = 0usize;

    for entry in WalkDir::new(&args.input)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let Ok(data) = std::fs::read(entry.path()) else {
            continue;
        };
        let Ok(bundle) = BundleFile::parse(data) else {
            continue;
        };
        if bundle.files.is_empty() {
            continue;
        }
        let Ok(sf) = SerializedFile::parse(bundle.files[0].data.clone()) else {
            continue;
        };
        for obj in &sf.objects {
            if obj.class_id != 49 {
                continue;
            }
            let Ok(val) = read_object(&sf, obj) else {
                continue;
            };
            let Some(name) = val["m_Name"].as_str() else {
                continue;
            };
            let Some(real) = manifest.get_output_path(name) else {
                continue;
            };
            if !real.starts_with("gamedata/") {
                continue;
            }
            let Some(fb) = export::text_asset::flatbuffer_payload(&val) else {
                continue;
            };
            let file_name = Path::new(real)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(name);
            let verdict = unpacker::flatbuffers_decode::verify_table(&fb, file_name);
            if verdict.table == "unknown" {
                skipped += 1;
                continue;
            }
            let row = rows.entry(verdict.table).or_default();
            row.files += 1;
            if let Some(err) = verdict.cn {
                row.cn_fail += 1;
                row.cn_err.get_or_insert(err);
            }
            match verdict.yostar {
                unpacker::flatbuffers_decode::YostarVerdict::NotRun => {}
                unpacker::flatbuffers_decode::YostarVerdict::Verified => {
                    row.yostar.get_or_insert((0, None));
                }
                unpacker::flatbuffers_decode::YostarVerdict::Failed(err) => {
                    let y = row.yostar.get_or_insert((0, None));
                    y.0 += 1;
                    y.1.get_or_insert(err);
                }
            }
            *row.chosen.entry(verdict.chosen).or_default() += 1;
        }
    }

    for (table, r) in &rows {
        let label = if r.files == 1 {
            (*table).to_string()
        } else {
            format!("{table} x{}", r.files)
        };
        let cn = match (r.cn_fail, r.cn_err.as_deref()) {
            (0, _) => "PASS".to_string(),
            (n, err) if n == r.files => format!("FAIL {}", err.unwrap_or_default()),
            (n, err) => format!("FAIL {n}/{} {}", r.files, err.unwrap_or_default()),
        };
        let yostar = match &r.yostar {
            None => "n/a".to_string(),
            Some((0, _)) => "PASS".to_string(),
            Some((n, err)) if *n == r.files => {
                format!("FAIL {}", err.as_deref().unwrap_or_default())
            }
            Some((n, err)) => format!(
                "FAIL {n}/{} {}",
                r.files,
                err.as_deref().unwrap_or_default()
            ),
        };
        let chosen = r
            .chosen
            .iter()
            .map(|(k, v)| {
                if *v == r.files {
                    (*k).to_string()
                } else {
                    format!("{k} {v}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        println!("{label:<30} | CN: {cn:<58} | Yostar: {yostar:<50} | chosen: {chosen}");
    }
    println!(
        "{} schema types, {} gamedata buffers, {skipped} unrecognised",
        rows.len(),
        rows.values().map(|r| r.files).sum::<usize>()
    );
}

fn find_idx_file(input_dir: &Path) -> Option<std::path::PathBuf> {
    // Search input dir first, then parent
    for dir in [Some(input_dir), input_dir.parent()] {
        let Some(dir) = dir else { continue };
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "idx") {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// `.resS` / `.resource` bundle entries hold streamed payload bytes, not a
/// `SerializedFile`.
fn is_resource_entry(path: &str) -> bool {
    path.ends_with(".resS") || path.ends_with(".resource")
}

/// The `(m_FileID, m_PathID)` pair of a `PPtr` JSON object; `None` when either
/// field is missing.
fn pptr_ids(v: &serde_json::Value) -> Option<(i64, i64)> {
    Some((
        v.get("m_FileID").and_then(serde_json::Value::as_i64)?,
        v.get("m_PathID").and_then(serde_json::Value::as_i64)?,
    ))
}

/// Class IDs needed for spine `MonoBehaviour` reference chain:
/// 1=GameObject (front/back classification), 114=MonoBehaviour, 49=TextAsset,
/// 21=Material, 28=Texture2D
const SPINE_CLASS_IDS: &[i32] = &[1, 114, 49, 21, 28];

/// Dynchars additionally need Transform (4), `MeshRenderer` (23), `MeshFilter` (33)
/// and Mesh (43) to locate, place and rasterize the background scene quads, plus
/// `AnimationClip` (74) to evaluate the idle pose the quads settle into, and
/// `ParticleSystem` (198) + `ParticleSystemRenderer` (199) + Camera (20) for the
/// `[particles]` export.
///
/// Animator (95) + `AnimatorController` (91) carry the clip→rig linkage: a clip's
/// binding path hashes are relative to the `GameObject` holding the Animator that
/// plays it (`m_Controller` → `m_AnimationClips`). Without them a hash that
/// collides across identical sibling rigs — Mlynar "Fields of Ruination" ships six
/// `static_offset/fixed/scale_01/scale02/glow_01` blade quads — cannot be scoped to
/// the one rig its clip actually drives (see `anim::build_clip_animator_gos`).
const DYNCHAR_SPINE_CLASS_IDS: &[i32] =
    &[1, 4, 20, 21, 23, 28, 33, 43, 49, 74, 91, 95, 114, 198, 199];

/// Map local (`m_FileID == 0`) object `path_ids` to their intended output directory,
/// taken from the bundle's `AssetBundle` (class 142) `m_Container`.
///
/// Some bundles co-pack many assets that share the same `m_Name` — e.g. the
/// seasonal voice packs `voice/extra_*.ab` hold a dozen operators' `CN_038`/
/// `CN_044` clips, where the per-operator identity lives only in the container
/// map (`dyn/audio/sound_beta_2/voice/char_002_amiya/cn_044.ogg → path_id`).
/// Naming output by `m_Name` alone would collide (last-write-wins) and land at
/// the bundle's name instead of the operator's path. We strip the `dyn/`
/// addressable prefix and keep the parent directory; callers pair it with the
/// object's `m_Name` for the filename (preserving its casing, e.g. `CN_044`).
fn build_container_dirs(sf: &SerializedFile) -> HashMap<i64, PathBuf> {
    let mut map = HashMap::new();
    for obj in &sf.objects {
        if obj.class_id != 142 {
            continue;
        }
        let Ok(val) = read_object(sf, obj) else {
            continue;
        };
        let Some(container) = val["m_Container"].as_object() else {
            continue;
        };
        for (asset_path, info) in container {
            let asset = &info["asset"];
            // Skip external references that point into dependency bundles.
            if asset["m_FileID"].as_i64().unwrap_or(0) != 0 {
                continue;
            }
            let Some(pid) = asset["m_PathID"].as_i64() else {
                continue;
            };
            let rel = asset_path.strip_prefix("dyn/").unwrap_or(asset_path);
            if let Some(parent) = Path::new(rel).parent()
                && !parent.as_os_str().is_empty()
            {
                map.insert(pid, parent.to_path_buf());
            }
        }
    }
    map
}

#[allow(clippy::too_many_arguments)]
fn process_bundle(
    file_path: &Path,
    input_dir: &Path,
    output_dir: &Path,
    extract_image: bool,
    extract_text: bool,
    extract_audio: bool,
    extract_spine: bool,
    extract_portrait: bool,
    merge_alpha: bool,
    stage_aspects: &StageAspectMap,
    shader_map: &export::shader_map::ShaderMap,
    fx_textures: &export::fx_textures::FxTextures,
) -> usize {
    let data = match std::fs::read(file_path) {
        Ok(d) => d,
        Err(_) => return 0,
    };

    let bundle = match BundleFile::parse(data) {
        Ok(b) => b,
        Err(_) => return 0,
    };

    if bundle.files.is_empty() {
        return 0;
    }

    // Derive subdirectory from bundle's relative path (e.g., "chararts/char_002_amiya")
    let bundle_subdir = file_path
        .strip_prefix(input_dir)
        .unwrap_or(file_path)
        .with_extension("");

    // Build resource map from .resS / .resource entries
    let mut resources = HashMap::new();
    for entry in &bundle.files {
        if is_resource_entry(&entry.path) {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }

    let mut exported = 0;
    let is_enemy_spine_bundle =
        extract_spine && spine::detect_enemy_spine_bundle(&bundle_subdir, input_dir);
    let is_spine_bundle = (extract_spine && spine::detect_spine_bundle(&bundle_subdir, input_dir))
        || is_enemy_spine_bundle;
    let is_portrait_bundle =
        extract_portrait && portrait::detect_portrait_bundle(&bundle_subdir, input_dir);
    let needs_phase2 = extract_image || extract_text || extract_audio;

    // Phase 1: Spine extraction (only read spine-relevant class_ids).
    // Dynchars bundles also need the scene graph (Transform=4, MeshRenderer=23,
    // MeshFilter=33) to reach and place the standalone background layer.
    let is_dynchar_bundle =
        is_spine_bundle && spine::detect_dynchar_bundle(&bundle_subdir, input_dir);
    let spine_class_ids: &[i32] = if is_dynchar_bundle {
        DYNCHAR_SPINE_CLASS_IDS
    } else {
        SPINE_CLASS_IDS
    };
    let spine_claimed_pids: HashSet<i64> = if is_spine_bundle {
        let mut spine_objects = HashMap::new();

        for entry in &bundle.files {
            if is_resource_entry(&entry.path) {
                continue;
            }
            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };
            for obj in &sf.objects {
                // Only deserialize objects relevant to the spine reference chain
                if !spine_class_ids.contains(&obj.class_id) {
                    continue;
                }
                let mut val = match read_object(&sf, obj) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                // Materials reference their shader externally; resolve the name
                // here (where this SerializedFile's `externals` are in scope) and
                // stash it so particle classification can read it downstream.
                if obj.class_id == 21 {
                    if let Some((fid, pid)) = val.get("m_Shader").and_then(pptr_ids)
                        && let Some(name) =
                            export::shader_map::resolve_shader(&sf.externals, fid, pid, shader_map)
                    {
                        val["_shaderName"] = serde_json::Value::String(name.to_string());
                    }
                    // Resolve EXTERNAL texture slots (shared FX sprites / data maps)
                    // into synthetic in-bundle Texture2Ds so the emitter renders
                    // them. The pixels stream from the FX bundle's resS, so decode
                    // here (where the FX map is available) and inline the RGBA.
                    //
                    // The ordinary single-`_MainTex` path skips grab-pass /
                    // distortion shaders (they warp the framebuffer behind them — a
                    // normal/flow map, not a sprite — so the raw texture stamps an
                    // iridescent blob) and gates opaque textures out.
                    //
                    // The `Ram/` shader family (`Ram/Disturb`, `Ram/VertexDisturb`)
                    // is NOT a framebuffer-grab distortion: it composites ramp +
                    // dissolve + UV-disturb from FOUR-to-SIX of its own textures,
                    // several of them legitimately opaque. So for Ram materials we
                    // resolve every sampled slot and bypass BOTH gates (the
                    // Disturb-name gate and the opaque gate).
                    let shader = val
                        .get("_shaderName")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("");
                    let is_ram = shader.contains("Ram/");
                    let is_distortion = shader.contains("GrabPass")
                        || (shader.contains("Disturb") && !shader.contains("VertexDisturb"));
                    let base_slots: &[&str] = if is_ram {
                        &[
                            "_MainTex",
                            "_RamTex",
                            "_DisturbTex",
                            "_DissolveTex",
                            "_VertexDisturbTex",
                            "_VertexDisturbWeightTex",
                        ]
                    } else if is_distortion {
                        &[]
                    } else {
                        &["_MainTex"]
                    };
                    // The dissolve/disturb MASK pair is not a `Ram/` peculiarity — every
                    // sub-namespaced `Particles-L2D` compositor binds the same two data
                    // maps and cuts its quad down to their silhouette. Resolving them
                    // only for `Ram/` starved the rest of the family: a
                    // `Disturb/Disturb(CustomData)` material falls in the `is_distortion`
                    // branch above, which resolves NOTHING, so Mlynar's entrance wind
                    // sheets reached the renderer with no mask at all and covered their
                    // whole bounding rectangle. Add the pair (never `_MainTex`, which
                    // keeps its ungated `_meshExtResolved` path below) and let the slot
                    // lookup itself decide: a material that doesn't bind them skips.
                    let mut slots: Vec<&str> = base_slots.to_vec();
                    if export::spine::is_l2d_compositor(shader) {
                        // TWO-MAP SPELLINGS. The `Dissolve/` family binds `_DissolveTex_01` /
                        // `_DissolveTex_02` (its single-name `_DissolveTex` is inert residue),
                        // and this list only ever carried the single names — so a two-map
                        // material's masks were never resolved across bundles and the layer drew
                        // its full bounding rectangle. fugue's `_01` resolved only by ACCIDENT:
                        // another material in the same bundle binds the same flow.ab texture
                        // under the single name, so it was already in `spine_objects` by path_id.
                        // `_02` (mask.ab, `mask_15`) had no such twin and stayed null.
                        // The slot lookup below skips any name the material doesn't bind, so
                        // adding the numbered pair is inert for every single-map material.
                        for mask in [
                            "_DissolveTex",
                            "_DisturbTex",
                            "_DissolveTex_01",
                            "_DissolveTex_02",
                            // The disturb WEIGHT map (see `SceneRam::weight_*`). Same reasoning
                            // as the numbered dissolve pair: a material that does not bind it
                            // skips, so this is inert for every other family.
                            "_WeightTex",
                        ] {
                            if !slots.contains(&mask) {
                                slots.push(mask);
                            }
                        }
                    }
                    for slot in &slots {
                        let Some((fid, pid)) = val
                            .get("m_SavedProperties")
                            .and_then(|sp| sp.get("m_TexEnvs"))
                            .and_then(|te| te.get(*slot))
                            .and_then(|m| m.get("m_Texture"))
                            .and_then(pptr_ids)
                        else {
                            continue;
                        };
                        if fid == 0 {
                            continue; // already in-bundle
                        }
                        // Masks are DATA maps, not sprites: a dissolve/disturb texture is
                        // legitimately opaque and would be thrown out by the sprite
                        // opacity gate, so it takes the raw path like the Ram slots do.
                        let decoded = if is_ram
                            || matches!(
                                *slot,
                                "_DissolveTex"
                                    | "_DisturbTex"
                                    | "_DissolveTex_01"
                                    | "_DissolveTex_02"
                                    | "_WeightTex"
                            ) {
                            fx_textures.resolve_decode_ram(&sf.externals, fid, pid)
                        } else {
                            fx_textures.resolve_decode(&sf.externals, fid, pid)
                        };
                        if let Some(decoded) = decoded {
                            let synth = serde_json::json!({
                                "m_Name": decoded.name,
                                "m_Width": decoded.width,
                                "m_Height": decoded.height,
                                "_decodedRGBA": base64::engine::general_purpose::STANDARD.encode(&decoded.rgba),
                            });
                            spine_objects.insert(pid, (28, synth));
                            val["m_SavedProperties"]["m_TexEnvs"][*slot]["m_Texture"] =
                                serde_json::json!({ "m_FileID": 0, "m_PathID": pid });
                        }
                    }
                    // A `_MainTex` the baseline particle gates SKIP — a distortion-
                    // shader sprite (slots = [] above) or an opaque fill rejected by
                    // `resolve_decode`'s opacity gate — can still be the real texture
                    // of an entrance-sequenced scene QUAD: Mlynar's white transition
                    // flash (`mask_09`, opaque) and its wind sheets (`smoke_46`, on a
                    // Disturb-shader material). Resolve it UNGATED and mark the
                    // material `_meshExtResolved`; the scene exporter renders such
                    // materials ONLY on entrance-windowed layers, so an always-on
                    // frozen fx quad can't pollute the idle scene (see spine.rs).
                    if let Some((fid, pid)) = val
                        .get("m_SavedProperties")
                        .and_then(|sp| sp.get("m_TexEnvs"))
                        .and_then(|te| te.get("_MainTex"))
                        .and_then(|m| m.get("m_Texture"))
                        .and_then(pptr_ids)
                        && fid != 0
                        && let Some(decoded) =
                            fx_textures.resolve_decode_ram(&sf.externals, fid, pid)
                    {
                        let synth = serde_json::json!({
                            "m_Name": decoded.name,
                            "m_Width": decoded.width,
                            "m_Height": decoded.height,
                            "_decodedRGBA": base64::engine::general_purpose::STANDARD.encode(&decoded.rgba),
                        });
                        spine_objects.insert(pid, (28, synth));
                        val["m_SavedProperties"]["m_TexEnvs"]["_MainTex"]["m_Texture"] =
                            serde_json::json!({ "m_FileID": 0, "m_PathID": pid });
                        val["_meshExtResolved"] = serde_json::Value::Bool(true);
                    }
                }
                // A ParticleSystemRenderer whose MATERIAL lives in another bundle
                // (dynchar `effect.ab`) resolves to `tex: null` and renders nothing —
                // the emitter has neither a blend class nor a sprite (Virtuosa's
                // falling gold-mist + standing rain). Resolve the external material
                // here (externals in scope) + its own `_MainTex` (in that same FX
                // bundle) and INLINE both as synthetic in-bundle objects, rewriting the
                // renderer's ref to in-bundle, so downstream `resolve_material` finds
                // the blend (`_DstBlend`) and the decoded sprite. Scalable — any skin's
                // external-material emitters resolve, no per-skin constant.
                if obj.class_id == 199
                    && let Some(mats) = val.get("m_Materials").and_then(|m| m.as_array()).cloned()
                {
                    for (i, mref) in mats.iter().enumerate() {
                        let Some((mfid, mpid)) = pptr_ids(mref) else {
                            continue;
                        };
                        if mfid == 0 {
                            continue; // material already in-bundle
                        }
                        let Some((mat_cab, mut mat_json, mat_ext_cabs)) =
                            fx_textures.resolve_material_ref(&sf.externals, mfid, mpid)
                        else {
                            continue;
                        };
                        // Resolve the material's own `_MainTex` — in the SAME FX bundle
                        // (`file_id==0`) or a FURTHER one it references (`file_id>0`).
                        if let Some((tfid, tpid)) = mat_json
                            .get("m_SavedProperties")
                            .and_then(|sp| sp.get("m_TexEnvs"))
                            .and_then(|te| te.get("_MainTex"))
                            .and_then(|m| m.get("m_Texture"))
                            .and_then(pptr_ids)
                            && let Some(decoded) =
                                fx_textures.material_texture(&mat_cab, &mat_ext_cabs, tfid, tpid)
                        {
                            let synth = serde_json::json!({
                                "m_Name": decoded.name,
                                "m_Width": decoded.width,
                                "m_Height": decoded.height,
                                "_decodedRGBA": base64::engine::general_purpose::STANDARD.encode(&decoded.rgba),
                            });
                            spine_objects.insert(tpid, (28, synth));
                            mat_json["m_SavedProperties"]["m_TexEnvs"]["_MainTex"]["m_Texture"] =
                                serde_json::json!({ "m_FileID": 0, "m_PathID": tpid });
                        }
                        spine_objects.insert(mpid, (21, mat_json));
                        val["m_Materials"][i] =
                            serde_json::json!({ "m_FileID": 0, "m_PathID": mpid });
                    }
                }
                spine_objects.insert(obj.path_id, (obj.class_id, val));
            }
        }

        let (spine_assets, claimed) = if is_enemy_spine_bundle {
            spine::collect_enemy_spine_assets(&spine_objects)
        } else {
            spine::collect_spine_assets(&spine_objects, &resources)
        };
        if !spine_assets.is_empty() {
            // Enemy pack bundles hold many enemies; each asset derives its own
            // directory from its skel name instead of the bundle name.
            let char_name =
                (!is_enemy_spine_bundle).then(|| spine::char_name_from_bundle(&bundle_subdir));
            let count = spine::export_spine_assets(
                &spine_assets,
                output_dir,
                char_name.as_deref(),
                &resources,
                &bundle_subdir.to_string_lossy(),
            );
            exported += count;
        }
        // Drop spine_objects before Phase 2 to free memory
        drop(spine_objects);
        claimed
    } else {
        HashSet::new()
    };

    // Phase 1.5: Portrait extraction.
    //
    // Supports two layouts:
    //   - Unity Sprite (class 213) atlases — current hot-update bundles
    //     (`spritepack/char_portrait_*.ab`). Each Sprite references an RGB
    //     Texture2D (and optionally a separate alpha Texture2D) by path_id.
    //   - Legacy SpritePacker MonoBehaviour (class 114) with `_sprites` +
    //     `_atlas` — older `charportraits/portraits_hub.ab` style bundles.
    let portrait_claimed_pids: HashSet<i64> = if is_portrait_bundle {
        let mut all_objects: HashMap<i64, (i32, serde_json::Value)> = HashMap::new();

        for entry in &bundle.files {
            if is_resource_entry(&entry.path) {
                continue;
            }
            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };
            for obj in &sf.objects {
                // 114 = MonoBehaviour (legacy SpritePacker), 213 = Sprite, 28 = Texture2D
                if obj.class_id == 114 || obj.class_id == 213 || obj.class_id == 28 {
                    let val = match read_object(&sf, obj) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    all_objects.insert(obj.path_id, (obj.class_id, val));
                }
            }
        }

        let mut claimed = HashSet::new();

        // Preferred path: Unity Sprite (class 213) objects.
        let mut sprites = Vec::new();
        for (pid, (class_id, val)) in &all_objects {
            if *class_id == 213
                && let Some(sprite) = portrait::parse_sprite(val)
            {
                claimed.insert(*pid);
                claimed.insert(sprite.texture_pid);
                if sprite.alpha_pid != 0 {
                    claimed.insert(sprite.alpha_pid);
                }
                sprites.push(sprite);
            }
        }

        // Fallback / additional path: legacy SpritePacker MonoBehaviours.
        let mut packers = Vec::new();
        for (pid, (class_id, val)) in &all_objects {
            if *class_id == 114
                && let Some(packer) = portrait::parse_sprite_packer(val)
            {
                claimed.insert(*pid);
                claimed.insert(packer.texture_pid);
                if packer.alpha_pid != 0 {
                    claimed.insert(packer.alpha_pid);
                }
                packers.push(packer);
            }
        }

        if !sprites.is_empty() || !packers.is_empty() {
            // Decode every Texture2D claimed by either path. Also decode all
            // Texture2Ds so the `{name}a` alpha-fallback lookup can resolve
            // alpha atlases that aren't referenced by explicit path_id.
            let mut decoded: HashMap<i64, export::texture::DecodedTexture> = HashMap::new();
            for (pid, (class_id, val)) in &all_objects {
                if *class_id == 28 {
                    match decode_texture_object(val, &resources) {
                        Ok(Some(tex)) => {
                            decoded.insert(*pid, tex);
                        }
                        Ok(None) => {}
                        Err(e) => eprintln!("  error decoding portrait texture: {e}"),
                    }
                }
            }

            let dir = output_dir.join("portraits");
            std::fs::create_dir_all(&dir).ok();
            if !sprites.is_empty() {
                exported += portrait::extract_sprites(&sprites, &decoded, &dir);
            }
            if !packers.is_empty() {
                exported += portrait::extract_portraits(&packers, &decoded, &dir);
            }

            // Claim every Texture2D we decoded so Phase 2 doesn't also emit
            // them as raw atlas dumps.
            for pid in decoded.keys() {
                claimed.insert(*pid);
            }
        }

        drop(all_objects);
        claimed
    } else {
        HashSet::new()
    };

    // Combine claimed path_ids from spine + portrait phases
    let claimed_pids: HashSet<i64> = spine_claimed_pids
        .iter()
        .chain(portrait_claimed_pids.iter())
        .copied()
        .collect();

    // Phase 2: Normal per-object export (skip claimed assets by path_id)
    if needs_phase2 {
        // Buffer decoded textures for alpha merging
        let mut decoded_textures: HashMap<String, export::texture::DecodedTexture> = HashMap::new();

        // Voice bundles are resolved by exact path (VoiceAsset → file path), so
        // co-packed clips must land at their real per-operator path from the
        // container map. SFX resolve by stem, so they stay under the bundle name.
        let is_voice_bundle = bundle_subdir.components().any(|c| {
            c.as_os_str()
                .to_str()
                .is_some_and(|s| s == "voice" || s.starts_with("voice_"))
        });

        // Opaque `anon/<hash>` bundles carry no meaningful path in their
        // filename, so textures would otherwise land under `textures/anon/<hash>/`.
        // Recover a friendly directory from each asset's container address
        // (e.g. `dyn/mouse/glow` -> `mouse`). Named bundles already encode their
        // path and are left untouched.
        let bundle_is_anon = bundle_subdir.starts_with("anon");
        // Per-texture output subdir (relative to `textures/`), keyed by m_Name.
        // Only populated for anon bundles; absent entries fall back to bundle_subdir.
        let mut tex_dirs: HashMap<String, PathBuf> = HashMap::new();

        for entry in &bundle.files {
            if is_resource_entry(&entry.path) {
                continue;
            }

            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };

            let has_audio = extract_audio && sf.objects.iter().any(|o| o.class_id == 83);
            // The AssetBundle container map disambiguates co-packed audio and
            // recovers friendly directories for opaque anon texture bundles.
            // Only built when one of those consumers needs it.
            let container_dirs = if has_audio || (extract_image && bundle_is_anon) {
                build_container_dirs(&sf)
            } else {
                HashMap::new()
            };

            // An anon bundle usually represents a single addressable (e.g. the
            // prefab `dyn/mouse/pc_mouse_mgr`); its leaf textures/sprites aren't
            // individually registered in m_Container. Mirror AssetStudio by
            // propagating that lone container directory to every co-packed
            // texture — but only when the bundle maps to exactly one directory,
            // so ambiguous multi-asset bundles fall back to the hashed name.
            let anon_dir: Option<PathBuf> = if bundle_is_anon {
                let dirs: std::collections::HashSet<&PathBuf> = container_dirs.values().collect();
                (dirs.len() == 1)
                    .then(|| dirs.into_iter().next().cloned())
                    .flatten()
            } else {
                None
            };

            // Audio pass: export AudioClips, disambiguating shared bundles via
            // the AssetBundle container map.
            if has_audio {
                let audio: Vec<(i64, serde_json::Value)> = sf
                    .objects
                    .iter()
                    .filter(|o| o.class_id == 83 && !claimed_pids.contains(&o.path_id))
                    .filter_map(|o| read_object(&sf, o).ok().map(|v| (o.path_id, v)))
                    .collect();

                // Count output filenames to detect intra-bundle m_Name collisions.
                let mut name_counts: HashMap<&str, u32> = HashMap::new();
                for (_, v) in &audio {
                    *name_counts
                        .entry(v["m_Name"].as_str().unwrap_or("unnamed"))
                        .or_default() += 1;
                }

                for (pid, val) in &audio {
                    let name = val["m_Name"].as_str().unwrap_or("unnamed");
                    let collides = name_counts.get(name).copied().unwrap_or(0) > 1;
                    // Use the container path when co-packed (collision) or in a
                    // voice bundle; for per-character voice bundles the container
                    // dir equals bundle_subdir, so this is a no-op there.
                    let dir = match container_dirs.get(pid) {
                        Some(cdir) if collides || is_voice_bundle => {
                            output_dir.join("audio").join(cdir)
                        }
                        _ => output_dir.join("audio").join(&bundle_subdir),
                    };
                    std::fs::create_dir_all(&dir).ok();
                    match export_audio(val, &dir, &resources) {
                        Ok(()) => exported += 1,
                        Err(e) => eprintln!("  error exporting {name}: {e}"),
                    }
                }
            }

            // Texture/text pass.
            for obj in &sf.objects {
                // Skip assets claimed by spine/portrait extraction
                if !claimed_pids.is_empty() && claimed_pids.contains(&obj.path_id) {
                    continue;
                }

                // Only deserialize objects we'll actually export
                match obj.class_id {
                    28 if extract_image => {}
                    49 if extract_text => {}
                    _ => continue,
                }

                let val = match read_object(&sf, obj) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let name = val["m_Name"].as_str().unwrap_or("unnamed");

                match obj.class_id {
                    28 => {
                        // Buffer textures instead of saving immediately
                        match decode_texture_object(&val, &resources) {
                            Ok(Some(tex)) => {
                                // For opaque anon bundles, route each texture to
                                // its container directory (e.g. `dyn/mouse/glow`
                                // -> `mouse`) instead of the hashed bundle name,
                                // preferring a per-asset entry and falling back
                                // to the bundle's single addressable directory.
                                if let Some(cdir) = container_dirs
                                    .get(&obj.path_id)
                                    .cloned()
                                    .or_else(|| anon_dir.clone())
                                {
                                    tex_dirs.insert(tex.name.clone(), cdir);
                                }
                                decoded_textures.insert(tex.name.clone(), tex);
                            }
                            Ok(None) => {}
                            Err(e) => eprintln!("  error decoding {name}: {e}"),
                        }
                    }
                    49 => {
                        let dir = output_dir.join("text").join(&bundle_subdir);
                        std::fs::create_dir_all(&dir).ok();
                        match export_text_asset(&val, &dir, None) {
                            Ok(()) => exported += 1,
                            Err(e) => eprintln!("  error exporting {name}: {e}"),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        // Export buffered textures (with or without alpha merging).
        if !decoded_textures.is_empty() {
            // Group textures by their resolved output subdir. Named bundles yield
            // a single group (`bundle_subdir`), preserving prior behavior; anon
            // bundles split assets into their container directories. Alpha pairs
            // share a container, so grouping keeps `foo`/`foo[alpha]` together.
            let mut groups: HashMap<PathBuf, HashMap<String, export::texture::DecodedTexture>> =
                HashMap::new();
            for (name, tex) in decoded_textures {
                let sub = tex_dirs
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| bundle_subdir.clone());
                groups.entry(sub).or_default().insert(name, tex);
            }

            // Unsquash stage mappreview thumbnails to their natural aspect.
            // Arknights packs these as 512×512 squares; we resize back using
            // the level's tile grid dims (with a 16:9 fallback).
            let is_mappreview = stage_preview::detect_mappreview_bundle(&bundle_subdir);

            for (sub, mut texs) in groups {
                let dir = output_dir.join("textures").join(&sub);
                std::fs::create_dir_all(&dir).ok();

                if is_mappreview {
                    for tex in texs.values_mut() {
                        stage_preview::unsquash_mappreview_texture(tex, stage_aspects);
                    }
                }

                if merge_alpha {
                    exported += alpha_merge::merge_and_export(texs, &dir);
                } else {
                    for tex in texs.values() {
                        match save_decoded_texture(tex, &dir) {
                            Ok(()) => exported += 1,
                            Err(e) => eprintln!("  error saving {}: {e}", tex.name),
                        }
                    }
                }
            }
        }
    }

    // The sprite hub: where a face patch sits on its body. A first-class
    // output of the texture pass, rewritten on every extract so the orphan
    // sweep never reaps it, and the only thing under
    // `textures/avg/characters/<folder>/` that is not a PNG.
    if extract_image
        && avg_hub::is_avg_character_bundle(&bundle_subdir)
        && let Some(hub) = avg_hub::hub_from_bundle(&bundle)
    {
        let dir = output_dir.join("textures").join(&bundle_subdir);
        match avg_hub::write_hub_json(&dir, &hub) {
            Ok(()) => exported += 1,
            Err(e) => eprintln!(
                "  error writing hub.json for {}: {e}",
                bundle_subdir.display()
            ),
        }
    }

    // The image sprites: a plate's own rect, pixels-per-unit and pivot. Same
    // deal as the hub, for the story's background and CG trees, because an
    // absent `screenadapt` sizes a plate at its OWN PPU and the PNG alone does
    // not carry it.
    if extract_image && avg_sprites::is_avg_image_bundle(&bundle_subdir) {
        let sprites = avg_sprites::sprites_from_bundle(&bundle);
        if !sprites.is_empty() {
            let dir = output_dir.join("textures").join(&bundle_subdir);
            match avg_sprites::write_sprites_json(&dir, &sprites) {
                Ok(()) => exported += 1,
                Err(e) => eprintln!(
                    "  error writing sprites.json for {}: {e}",
                    bundle_subdir.display()
                ),
            }
        }
    }

    exported
}

fn cmd_list(args: &cli::ListArgs) {
    let data = match std::fs::read(&args.input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error reading {}: {e}", args.input.display());
            std::process::exit(1);
        }
    };

    let bundle = match BundleFile::parse(data) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error parsing bundle: {e}");
            std::process::exit(1);
        }
    };

    println!("Bundle: {} file(s)", bundle.files.len());

    for (i, entry) in bundle.files.iter().enumerate() {
        println!(
            "\n--- File {i}: {} ({} bytes) ---",
            entry.path,
            entry.data.len()
        );

        if is_resource_entry(&entry.path) {
            println!("  (resource data)");
            continue;
        }

        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(sf) => sf,
            Err(e) => {
                println!("  (parse error: {e})");
                continue;
            }
        };

        println!(
            "  Unity {}, platform {}, {} objects",
            sf.unity_version,
            sf.target_platform,
            sf.objects.len()
        );

        for obj in &sf.objects {
            let class_name = class_id_name(obj.class_id);
            let name = match read_object(&sf, obj) {
                Ok(val) => val["m_Name"].as_str().unwrap_or("").to_string(),
                Err(_) => String::new(),
            };

            println!(
                "  [{:>4}] {:<20} path_id={:<12} size={:>8}  {}",
                obj.class_id, class_name, obj.path_id, obj.byte_size, name
            );
        }
    }
}

const fn class_id_name(id: i32) -> &'static str {
    match id {
        1 => "GameObject",
        4 => "Transform",
        21 => "Material",
        23 => "MeshRenderer",
        28 => "Texture2D",
        33 => "MeshFilter",
        43 => "Mesh",
        48 => "Shader",
        49 => "TextAsset",
        54 => "Rigidbody2D",
        65 => "BoxCollider2D",
        74 => "AnimationClip",
        83 => "AudioClip",
        91 => "AnimatorController",
        95 => "Animator",
        114 => "MonoBehaviour",
        115 => "MonoScript",
        128 => "Font",
        142 => "AssetBundle",
        150 => "PreloadData",
        152 => "MovieTexture",
        156 => "TerrainData",
        184 => "AudioMixerGroup",
        186 => "AudioMixer",
        198 => "ParticleSystem",
        199 => "ParticleSystemRenderer",
        213 => "Sprite",
        224 => "RectTransform",
        225 => "CanvasGroup",
        226 => "Canvas",
        228 => "CanvasRenderer",
        258 => "TextMeshPro",
        _ => "Unknown",
    }
}
