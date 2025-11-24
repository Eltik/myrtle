use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Spine model with all its components
#[derive(Debug)]
struct SpineModel {
    name: String,
    skel_path: Option<PathBuf>,
    atlas_path: Option<PathBuf>,
    textures: Vec<PathBuf>,
    alpha_textures: Vec<PathBuf>,
    required_textures: Vec<String>,
    missing_textures: Vec<String>,
}

impl SpineModel {
    fn new(name: String) -> Self {
        Self {
            name,
            skel_path: None,
            atlas_path: None,
            textures: Vec::new(),
            alpha_textures: Vec::new(),
            required_textures: Vec::new(),
            missing_textures: Vec::new(),
        }
    }

    fn is_complete(&self) -> bool {
        self.skel_path.is_some() && self.atlas_path.is_some()
    }
}

/// Parse atlas file to extract required texture names
fn parse_atlas_textures(atlas_path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(atlas_path)?;
    let mut textures = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with(".png") {
            textures.push(trimmed.to_string());
        }
    }

    Ok(textures)
}

/// Extract base name from texture filename
fn get_texture_base_name(filename: &str) -> String {
    filename
        .trim_end_matches(".png")
        .trim_end_matches("[alpha]")
        .trim_end_matches("_alpha")
        .to_string()
}

/// Collect and organize Spine model files
pub fn main(src_dirs: &[&Path], dest_dirs: &[&Path]) -> Result<()> {
    for (src, dest) in src_dirs.iter().zip(dest_dirs.iter()) {
        collect_models_from_dir(src, dest)?;
    }
    Ok(())
}

fn collect_models_from_dir(src: &Path, dest: &Path) -> Result<()> {
    log::info!("Collecting models from {:?}", src);

    // First pass: collect all files
    let mut skel_files: HashMap<String, PathBuf> = HashMap::new();
    let mut atlas_files: HashMap<String, PathBuf> = HashMap::new();
    let mut texture_files: HashMap<String, PathBuf> = HashMap::new();
    let mut alpha_files: HashMap<String, PathBuf> = HashMap::new();

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "skel" => {
                let base = filename.trim_end_matches(".skel").to_string();
                skel_files.insert(base, path.to_path_buf());
            }
            "atlas" => {
                let base = filename.trim_end_matches(".atlas").to_string();
                atlas_files.insert(base, path.to_path_buf());
            }
            "png" => {
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem.contains("[alpha]") || stem.ends_with("_alpha") {
                    let _base = get_texture_base_name(stem);
                    alpha_files.insert(stem.to_string(), path.to_path_buf());
                } else {
                    texture_files.insert(stem.to_string(), path.to_path_buf());
                }
            }
            _ => {}
        }
    }

    // Second pass: build spine models
    let mut models: HashMap<String, SpineModel> = HashMap::new();

    // Add skel files
    for (name, path) in &skel_files {
        models
            .entry(name.clone())
            .or_insert_with(|| SpineModel::new(name.clone()))
            .skel_path = Some(path.clone());
    }

    // Add atlas files and parse required textures
    for (name, path) in &atlas_files {
        let model = models
            .entry(name.clone())
            .or_insert_with(|| SpineModel::new(name.clone()));
        model.atlas_path = Some(path.clone());

        // Parse atlas for required textures
        if let Ok(required) = parse_atlas_textures(path) {
            model.required_textures = required;
        }
    }

    // Match textures to models
    for (name, model) in &mut models {
        // Find textures that match this model
        for req_tex in &model.required_textures.clone() {
            let base = req_tex.trim_end_matches(".png");

            // Look for main texture
            if let Some(tex_path) = texture_files.get(base) {
                model.textures.push(tex_path.clone());
            } else {
                model.missing_textures.push(req_tex.clone());
            }

            // Look for alpha texture
            let alpha_key = format!("{}[alpha]", base);
            if let Some(alpha_path) = alpha_files.get(&alpha_key) {
                model.alpha_textures.push(alpha_path.clone());
            }
        }

        // If no required textures parsed, try to find by name pattern
        if model.required_textures.is_empty() {
            for (tex_name, tex_path) in &texture_files {
                if tex_name.starts_with(name) {
                    model.textures.push(tex_path.clone());
                }
            }
            for (alpha_name, alpha_path) in &alpha_files {
                if alpha_name.starts_with(name) {
                    model.alpha_textures.push(alpha_path.clone());
                }
            }
        }
    }

    // Filter to complete models only
    let complete_models: Vec<_> = models.values().filter(|m| m.is_complete()).collect();

    if complete_models.is_empty() {
        println!("No complete spine models found");
        return Ok(());
    }

    // Progress bar
    let pb = ProgressBar::new(complete_models.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    // Copy models to destination
    std::fs::create_dir_all(dest)?;

    let mut total_copied = 0;
    let mut models_with_missing = 0;

    for model in &complete_models {
        let model_dir = dest.join(&model.name);
        std::fs::create_dir_all(&model_dir)?;

        // Copy skel
        if let Some(ref skel_path) = model.skel_path {
            let dest_path = model_dir.join(format!("{}.skel", model.name));
            std::fs::copy(skel_path, &dest_path)?;
            total_copied += 1;
        }

        // Copy atlas
        if let Some(ref atlas_path) = model.atlas_path {
            let dest_path = model_dir.join(format!("{}.atlas", model.name));
            std::fs::copy(atlas_path, &dest_path)?;
            total_copied += 1;
        }

        // Copy textures
        for tex_path in &model.textures {
            let filename = tex_path.file_name().unwrap_or_default();
            let dest_path = model_dir.join(filename);
            std::fs::copy(tex_path, &dest_path)?;
            total_copied += 1;
        }

        // Copy alpha textures
        for alpha_path in &model.alpha_textures {
            let filename = alpha_path.file_name().unwrap_or_default();
            let dest_path = model_dir.join(filename);
            std::fs::copy(alpha_path, &dest_path)?;
            total_copied += 1;
        }

        // Log warnings for missing textures
        if !model.missing_textures.is_empty() {
            log::warn!(
                "Model '{}' missing textures: {:?}",
                model.name,
                model.missing_textures
            );
            models_with_missing += 1;
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done!");

    println!("\nModel collection complete!");
    println!("  Collected {} files", total_copied);
    println!("  Models: {}", models.len());
    println!("  Complete models: {}", complete_models.len());
    if models_with_missing > 0 {
        println!("  Models with missing textures: {}", models_with_missing);
    }

    Ok(())
}
