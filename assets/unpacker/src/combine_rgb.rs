use anyhow::{Context, Result};
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Patterns to identify alpha images
static ALPHA_PATTERNS: once_cell::sync::Lazy<Vec<Regex>> = once_cell::sync::Lazy::new(|| {
    vec![
        Regex::new(r"(.+)\[alpha\](\$[0-9]+)?").unwrap(),
        Regex::new(r"(.+)_alpha(\$[0-9]+)?").unwrap(),
        Regex::new(r"(.+)alpha(\$[0-9]+)?").unwrap(),
        // Single character suffix patterns for alpha/mask images
        Regex::new(r"(.+)a(\$[0-9]+)?$").unwrap(),
        Regex::new(r"(.+)b(\$[0-9]+)?$").unwrap(),
    ]
});

/// Combiner for RGB and Alpha images
pub struct AlphaRGBCombiner {
    alpha: DynamicImage,
}

impl AlphaRGBCombiner {
    pub fn new(alpha_path: &Path) -> Result<Self> {
        let alpha = image::open(alpha_path)
            .with_context(|| format!("Failed to open alpha image: {:?}", alpha_path))?;
        Ok(Self { alpha })
    }

    pub fn from_image(alpha: DynamicImage) -> Self {
        Self { alpha }
    }

    /// Combine RGB image with this alpha channel
    pub fn combine_with(&self, rgb: &DynamicImage, remove_bleeding: bool) -> RgbaImage {
        let rgb_rgba = rgb.to_rgba8();
        let alpha_gray = self.alpha.to_luma8();

        let (width, height) = rgb_rgba.dimensions();

        // Resize alpha if needed
        let alpha_resized = if alpha_gray.dimensions() != (width, height) {
            image::imageops::resize(
                &alpha_gray,
                width,
                height,
                image::imageops::FilterType::Triangle,
            )
        } else {
            alpha_gray
        };

        // Combine RGB with Alpha
        let mut result = ImageBuffer::new(width, height);
        for (x, y, pixel) in result.enumerate_pixels_mut() {
            let rgb_pixel = rgb_rgba.get_pixel(x, y);
            let alpha_value = alpha_resized.get_pixel(x, y).0[0];

            if remove_bleeding && alpha_value == 0 {
                *pixel = Rgba([0, 0, 0, 0]);
            } else {
                *pixel = Rgba([rgb_pixel[0], rgb_pixel[1], rgb_pixel[2], alpha_value]);
            }
        }

        result
    }

    /// Apply premultiplied alpha to an RGBA image
    pub fn apply_premultiplied_alpha(rgba: &DynamicImage) -> RgbaImage {
        let rgba_img = rgba.to_rgba8();
        let (width, height) = rgba_img.dimensions();

        let mut result = ImageBuffer::new(width, height);
        for (x, y, pixel) in result.enumerate_pixels_mut() {
            let src = rgba_img.get_pixel(x, y);
            let alpha = src[3] as f32 / 255.0;

            *pixel = Rgba([
                (src[0] as f32 * alpha) as u8,
                (src[1] as f32 * alpha) as u8,
                (src[2] as f32 * alpha) as u8,
                src[3],
            ]);
        }

        result
    }
}

/// Extract the base name from an alpha image filename
pub fn calc_real_name(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;

    for pattern in ALPHA_PATTERNS.iter() {
        if let Some(captures) = pattern.captures(stem) {
            if let Some(base) = captures.get(1) {
                return Some(base.as_str().to_string());
            }
        }
    }

    None
}

/// Calculate similarity between RGB and Alpha images
/// Returns a score where lower is better (more similar)
pub fn calc_similarity(rgb_path: &Path, alpha_path: &Path, precision: u32) -> Result<f64> {
    let rgb = image::open(rgb_path)?.to_luma8();
    let alpha = image::open(alpha_path)?.to_luma8();

    // Resize both to small size for quick comparison
    let rgb_small = image::imageops::resize(&rgb, precision, precision, FilterType::Triangle);
    let alpha_small = image::imageops::resize(&alpha, precision, precision, FilterType::Triangle);

    // Calculate mean absolute difference
    let mut total_diff: u64 = 0;
    for (rgb_pixel, alpha_pixel) in rgb_small.pixels().zip(alpha_small.pixels()) {
        let diff = (rgb_pixel.0[0] as i32 - alpha_pixel.0[0] as i32).unsigned_abs() as u64;
        total_diff += diff;
    }

    let num_pixels = (precision * precision) as f64;
    Ok(total_diff as f64 / num_pixels)
}

/// Search for matching RGB image
pub fn search_rgb(alpha_path: &Path) -> Result<PathBuf> {
    let real_name = calc_real_name(alpha_path).context("Not a recognized alpha image name")?;

    let ext = alpha_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png");

    let dir = alpha_path
        .parent()
        .context("Alpha image has no parent directory")?;

    let candidates: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                let p_ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
                (stem == real_name || (stem.starts_with(&real_name) && stem.contains('$')))
                    && p_ext == ext
                    && p != alpha_path
            } else {
                false
            }
        })
        .collect();

    match candidates.len() {
        0 => anyhow::bail!("No RGB image found for {:?}", alpha_path),
        1 => Ok(candidates[0].clone()),
        _ => {
            // Find most similar image
            let mut best_candidate = candidates[0].clone();
            let mut best_score = f64::MAX;

            for candidate in &candidates {
                if let Ok(score) = calc_similarity(candidate, alpha_path, 150) {
                    if score < best_score {
                        best_score = score;
                        best_candidate = candidate.clone();
                    }
                }
            }

            log::debug!(
                "Multiple candidates for {:?}, chose {:?} (score: {:.2})",
                alpha_path,
                best_candidate,
                best_score
            );
            Ok(best_candidate)
        }
    }
}

/// Main entry point for image combination
pub fn main(rootdir: &Path, destdir: &Path, do_del: bool) -> Result<()> {
    log::info!("Retrieving image paths...");

    // Collect all alpha images
    let alpha_files: Vec<PathBuf> = WalkDir::new(rootdir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .map(|e| {
                    let ext = e.to_string_lossy().to_lowercase();
                    ext == "png" || ext == "jpg" || ext == "jpeg"
                })
                .unwrap_or(false)
        })
        .filter(|p| calc_real_name(p).is_some())
        .collect();

    if alpha_files.is_empty() {
        println!("No alpha images found");
        return Ok(());
    }

    if do_del && destdir.exists() {
        std::fs::remove_dir_all(destdir)?;
    }

    std::fs::create_dir_all(destdir)?;

    // Progress bar
    let pb = ProgressBar::new(alpha_files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let combined_count = AtomicUsize::new(0);
    let rootdir_clone = rootdir.to_path_buf();
    let destdir_clone = destdir.to_path_buf();

    // Process in parallel
    alpha_files.par_iter().for_each(|alpha_path| {
        if let Ok(rgb_path) = search_rgb(alpha_path) {
            if let Ok(combiner) = AlphaRGBCombiner::new(alpha_path) {
                if let Ok(rgb) = image::open(&rgb_path) {
                    let result = combiner.combine_with(&rgb, true);

                    if let Some(real_name) = calc_real_name(alpha_path) {
                        let rel_path = alpha_path
                            .strip_prefix(&rootdir_clone)
                            .unwrap_or(alpha_path);
                        let output_dir =
                            destdir_clone.join(rel_path.parent().unwrap_or(Path::new("")));

                        if std::fs::create_dir_all(&output_dir).is_ok() {
                            let output_path = output_dir.join(format!("{}.png", real_name));
                            if result.save(&output_path).is_ok() {
                                combined_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        }
        pb.inc(1);
    });

    pb.finish_with_message("Done!");

    println!("\nImage combination complete!");
    println!("  Processed {} alpha images", alpha_files.len());
    println!(
        "  Combined {} images",
        combined_count.load(Ordering::Relaxed)
    );

    Ok(())
}
