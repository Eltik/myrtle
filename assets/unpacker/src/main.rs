mod cli;

use unpacker::export;
use unpacker::unity;

use std::collections::{HashMap, HashSet};
use std::path::Path;
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
        if !args.extract_all() && !args.image && !args.text && !args.audio && !args.spine {
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
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
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

/// Class IDs needed for spine MonoBehaviour reference chain:
/// 114=MonoBehaviour, 49=TextAsset, 21=Material, 28=Texture2D
const SPINE_CLASS_IDS: &[i32] = &[114, 49, 21, 28];

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
    let is_spine_bundle = extract_spine && spine::detect_spine_bundle(&bundle_subdir, input_dir);
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

        let (spine_assets, claimed) = spine::collect_spine_assets(&spine_objects);
        if !spine_assets.is_empty() {
            let char_name = spine::char_name_from_bundle(&bundle_subdir);
            let count =
                spine::export_spine_assets(&spine_assets, output_dir, &char_name, &resources);
            exported += count;
        }
        // Drop spine_objects before Phase 2 to free memory
        drop(spine_objects);
        claimed
    } else {
        HashSet::new()
    };

    // Phase 1.5: Portrait extraction from SpritePacker atlases
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
                if obj.class_id == 114 || obj.class_id == 28 {
                    let val = match read_object(&sf, obj) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    all_objects.insert(obj.path_id, (obj.class_id, val));
                }
            }
        }

        // Parse SpritePacker MonoBehaviours
        let mut packers = Vec::new();
        let mut claimed = HashSet::new();
        for (pid, (class_id, val)) in &all_objects {
            if *class_id == 114
                && let Some(packer) = portrait::parse_sprite_packer(val)
            {
                claimed.insert(*pid);
                claimed.insert(packer.texture_pid);
                claimed.insert(packer.alpha_pid);
                packers.push(packer);
            }
        }

        if !packers.is_empty() {
            // Decode all textures referenced by packers (RGB + alpha)
            let mut decoded: HashMap<i64, export::texture::DecodedTexture> = HashMap::new();
            for (pid, (class_id, val)) in &all_objects {
                if *class_id == 28 && claimed.contains(pid) {
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
            exported += portrait::extract_portraits(&packers, &decoded, &dir);
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

        for entry in &bundle.files {
            if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
                continue;
            }

            let sf = match SerializedFile::parse(entry.data.clone()) {
                Ok(sf) => sf,
                Err(_) => continue,
            };

            for obj in &sf.objects {
                // Skip assets claimed by spine/portrait extraction
                if !claimed_pids.is_empty() && claimed_pids.contains(&obj.path_id) {
                    continue;
                }

                // Only deserialize objects we'll actually export
                match obj.class_id {
                    28 if extract_image => {}
                    49 if extract_text => {}
                    83 if extract_audio => {}
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
                    83 => {
                        let dir = output_dir.join("audio").join(&bundle_subdir);
                        std::fs::create_dir_all(&dir).ok();
                        match export_audio(&val, &dir, &resources) {
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

fn class_id_name(id: i32) -> &'static str {
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
