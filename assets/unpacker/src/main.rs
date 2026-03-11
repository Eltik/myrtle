mod cli;
mod export;
mod unity;

#[macro_use]
pub mod fb_json_macros;
pub mod fb_json_auto;
pub mod fb_json_auto_yostar;
pub mod flatbuffers_decode;
pub mod generated_fbs;
pub mod generated_fbs_yostar;

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;

use cli::{Cli, Command};
use export::audio::export_audio;
use export::text_asset::export_text_asset;
use export::texture::export_texture;
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
    if args.gamedata {
        let idx_path = match &args.idx {
            Some(p) => p.clone(),
            None => match find_idx_file(&args.input) {
                Some(p) => {
                    println!("Auto-detected manifest: {}", p.display());
                    p
                }
                None => {
                    eprintln!("error: no .idx manifest found; use --idx <manifest.idx>");
                    std::process::exit(1);
                }
            },
        };
        std::fs::create_dir_all(&args.output).unwrap();
        match export::gamedata::export_gamedata(&args.input, &idx_path, &args.output) {
            Ok(count) => println!("Exported {count} gamedata files"),
            Err(e) => eprintln!("error: {e}"),
        }
        if !args.extract_all() && !args.image && !args.text && !args.audio {
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

    let exported = AtomicUsize::new(0);

    std::fs::create_dir_all(&args.output).unwrap();

    files.par_iter().for_each(|file_path| {
        let count = process_bundle(
            file_path,
            &args.output,
            extract_image,
            extract_text,
            extract_audio,
        );
        exported.fetch_add(count, Ordering::Relaxed);
        pb.inc(1);
    });

    pb.finish_with_message("done");
    println!("Exported {} assets", exported.load(Ordering::Relaxed));
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

fn process_bundle(
    file_path: &Path,
    output_dir: &Path,
    extract_image: bool,
    extract_text: bool,
    extract_audio: bool,
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

    // Build resource map from .resS / .resource entries
    let mut resources = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }

    let mut exported = 0;

    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            continue;
        }

        let sf = match SerializedFile::parse(entry.data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };

        for obj in &sf.objects {
            let val = match read_object(&sf, obj) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let name = val["m_Name"].as_str().unwrap_or("unnamed");

            let result = match obj.class_id {
                28 if extract_image => {
                    let dir = output_dir.join("textures");
                    std::fs::create_dir_all(&dir).ok();
                    export_texture(&val, &dir, &resources)
                }
                49 if extract_text => {
                    let dir = output_dir.join("text");
                    std::fs::create_dir_all(&dir).ok();
                    export_text_asset(&val, &dir, None)
                }
                83 if extract_audio => {
                    let dir = output_dir.join("audio");
                    std::fs::create_dir_all(&dir).ok();
                    export_audio(&val, &dir, &resources)
                }
                _ => continue,
            };

            match result {
                Ok(()) => exported += 1,
                Err(e) => eprintln!("  error exporting {name}: {e}"),
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
