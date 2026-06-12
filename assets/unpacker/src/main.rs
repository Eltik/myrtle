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

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

use cli::{Cli, Command};
use export::alpha_merge;
use export::audio::export_audio;
use export::portrait;
use export::spine;
use export::stage_preview::{self, StageAspectMap};
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
    }
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
            match export::gamedata::export_gamedata(&args.input, &idx_path, &args.output) {
                Ok(count) => println!("Exported {count} gamedata files"),
                Err(e) => eprintln!("error: {e}"),
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
}

/// Search input dir and its parent for a .idx manifest file
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

/// Class IDs needed for spine `MonoBehaviour` reference chain:
/// 1=GameObject (front/back classification), 114=MonoBehaviour, 49=TextAsset,
/// 21=Material, 28=Texture2D
const SPINE_CLASS_IDS: &[i32] = &[1, 114, 49, 21, 28];

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
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
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

    // Phase 1: Spine extraction (only read spine-relevant class_ids)
    let spine_claimed_pids: HashSet<i64> = if is_spine_bundle {
        let mut spine_objects = HashMap::new();

        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                continue;
            }
            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };
            for obj in &sf.objects {
                // Only deserialize objects relevant to the spine reference chain
                if !SPINE_CLASS_IDS.contains(&obj.class_id) {
                    continue;
                }
                let val = match read_object(&sf, obj) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                spine_objects.insert(obj.path_id, (obj.class_id, val));
            }
        }

        let (spine_assets, claimed) = if is_enemy_spine_bundle {
            spine::collect_enemy_spine_assets(&spine_objects)
        } else {
            spine::collect_spine_assets(&spine_objects)
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
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
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

        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                continue;
            }

            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };

            // Audio pass: export AudioClips, disambiguating shared bundles via
            // the AssetBundle container map. Only built when this file holds audio.
            if extract_audio && sf.objects.iter().any(|o| o.class_id == 83) {
                let container_dirs = build_container_dirs(&sf);
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

        // Export buffered textures (with or without alpha merging)
        if !decoded_textures.is_empty() {
            let dir = output_dir.join("textures").join(&bundle_subdir);
            std::fs::create_dir_all(&dir).ok();

            // Unsquash stage mappreview thumbnails to their natural aspect.
            // Arknights packs these as 512×512 squares; we resize back using
            // the level's tile grid dims (with a 16:9 fallback).
            if stage_preview::detect_mappreview_bundle(&bundle_subdir) {
                for tex in decoded_textures.values_mut() {
                    stage_preview::unsquash_mappreview_texture(tex, stage_aspects);
                }
            }

            if merge_alpha {
                exported += alpha_merge::merge_and_export(decoded_textures, &dir);
            } else {
                for tex in decoded_textures.values() {
                    match save_decoded_texture(tex, &dir) {
                        Ok(()) => exported += 1,
                        Err(e) => eprintln!("  error saving {}: {e}", tex.name),
                    }
                }
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

        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
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
