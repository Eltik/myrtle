//! Atlas Slicer - Auto-detect and extract sprites from texture atlases
//!
//! This module provides functionality to automatically detect sprite boundaries
//! in texture atlases that don't have metadata. It uses connected component
//! analysis to find regions of non-transparent pixels.

use anyhow::Result;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::collections::VecDeque;
use std::path::Path;

/// Represents a detected sprite region in the atlas
#[derive(Debug, Clone)]
pub struct SpriteRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl SpriteRegion {
    /// Calculate the area of the region
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

/// Merge an RGB image with a separate alpha channel image
///
/// The alpha image's red channel is used as the alpha value
pub fn merge_alpha(rgb: &RgbaImage, alpha: &RgbaImage) -> RgbaImage {
    let (width, height) = rgb.dimensions();
    let (alpha_width, alpha_height) = alpha.dimensions();

    // Use the smaller dimensions if they don't match
    let out_width = width.min(alpha_width);
    let out_height = height.min(alpha_height);

    let mut result = ImageBuffer::new(out_width, out_height);

    for y in 0..out_height {
        for x in 0..out_width {
            let rgb_pixel = rgb.get_pixel(x, y);
            let alpha_pixel = alpha.get_pixel(x, y);

            // Take RGB from the main image, alpha from the alpha image's red channel
            result.put_pixel(
                x,
                y,
                Rgba([rgb_pixel[0], rgb_pixel[1], rgb_pixel[2], alpha_pixel[0]]),
            );
        }
    }

    result
}

/// Load an atlas image with automatic alpha merging
///
/// If an alpha file exists (same name with 'a' suffix before extension),
/// it will be automatically merged.
pub fn load_atlas_with_alpha(path: &Path) -> Result<RgbaImage> {
    let img = image::open(path)?.to_rgba8();

    // Try to find alpha file
    // Pattern: name#0.png -> name#0a.png
    let path_str = path.to_string_lossy();
    let alpha_path = if path_str.contains("#0.png") {
        path_str.replace("#0.png", "#0a.png")
    } else if path_str.ends_with(".png") {
        path_str.replace(".png", "a.png")
    } else {
        return Ok(img);
    };

    let alpha_path = Path::new(&alpha_path);
    if alpha_path.exists() {
        let alpha_img = image::open(alpha_path)?.to_rgba8();
        Ok(merge_alpha(&img, &alpha_img))
    } else {
        Ok(img)
    }
}

/// Detect sprite boundaries using connected component analysis
///
/// This function finds all connected regions of non-transparent pixels
/// and returns their bounding boxes.
///
/// # Arguments
/// * `image` - The atlas image to analyze
/// * `alpha_threshold` - Pixels with alpha below this value are considered transparent
/// * `min_size` - Minimum width and height for a region to be included
///
/// # Returns
/// Vector of detected sprite regions, sorted by position (top-left to bottom-right)
pub fn detect_sprites(image: &RgbaImage, alpha_threshold: u8, min_size: u32) -> Vec<SpriteRegion> {
    let (width, height) = image.dimensions();
    let mut visited = vec![false; (width * height) as usize];
    let mut regions = Vec::new();

    // Helper to check if a pixel is opaque enough
    let is_opaque = |x: u32, y: u32| -> bool {
        if x >= width || y >= height {
            return false;
        }
        image.get_pixel(x, y)[3] >= alpha_threshold
    };

    // Helper to get visited index
    let idx = |x: u32, y: u32| -> usize { (y * width + x) as usize };

    // Scan for unvisited opaque pixels
    for start_y in 0..height {
        for start_x in 0..width {
            if visited[idx(start_x, start_y)] || !is_opaque(start_x, start_y) {
                continue;
            }

            // BFS to find connected region
            let mut queue = VecDeque::new();
            queue.push_back((start_x, start_y));
            visited[idx(start_x, start_y)] = true;

            let mut min_x = start_x;
            let mut max_x = start_x;
            let mut min_y = start_y;
            let mut max_y = start_y;

            while let Some((x, y)) = queue.pop_front() {
                // Update bounding box
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);

                // Check 4-connected neighbors
                let neighbors = [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ];

                for (nx, ny) in neighbors {
                    if nx < width && ny < height && !visited[idx(nx, ny)] && is_opaque(nx, ny) {
                        visited[idx(nx, ny)] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }

            // Calculate region dimensions
            let region_width = max_x - min_x + 1;
            let region_height = max_y - min_y + 1;

            // Only include regions above minimum size
            if region_width >= min_size && region_height >= min_size {
                regions.push(SpriteRegion {
                    x: min_x,
                    y: min_y,
                    width: region_width,
                    height: region_height,
                });
            }
        }
    }

    // Sort by position (top to bottom, left to right)
    regions.sort_by(|a, b| {
        let y_cmp = a.y.cmp(&b.y);
        if y_cmp == std::cmp::Ordering::Equal {
            a.x.cmp(&b.x)
        } else {
            y_cmp
        }
    });

    regions
}

/// Extract a sprite region from the atlas
///
/// Returns a new image containing just the specified region
pub fn extract_region(image: &RgbaImage, region: &SpriteRegion) -> RgbaImage {
    let mut sprite = ImageBuffer::new(region.width, region.height);

    for dy in 0..region.height {
        for dx in 0..region.width {
            let src_x = region.x + dx;
            let src_y = region.y + dy;
            let pixel = image.get_pixel(src_x, src_y);
            sprite.put_pixel(dx, dy, *pixel);
        }
    }

    sprite
}

/// Process an atlas file and extract all sprites
///
/// # Arguments
/// * `input_path` - Path to the atlas PNG file
/// * `output_dir` - Directory to save extracted sprites
/// * `alpha_threshold` - Pixels with alpha below this are transparent
/// * `min_size` - Minimum sprite size in pixels
/// * `prefix` - Optional prefix for output filenames
///
/// # Returns
/// Number of sprites extracted
pub fn slice_atlas(
    input_path: &Path,
    output_dir: &Path,
    alpha_threshold: u8,
    min_size: u32,
    prefix: Option<&str>,
) -> Result<usize> {
    // Load image with alpha
    let image = load_atlas_with_alpha(input_path)?;
    let (width, height) = image.dimensions();
    println!("Loaded atlas: {}x{}", width, height);

    // Detect sprite regions
    let regions = detect_sprites(&image, alpha_threshold, min_size);
    println!("Detected {} sprite regions", regions.len());

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Extract and save each sprite
    let file_prefix = prefix.unwrap_or("sprite");
    for (i, region) in regions.iter().enumerate() {
        let sprite = extract_region(&image, region);
        let output_path = output_dir.join(format!("{}_{}.png", file_prefix, i));
        sprite.save(&output_path)?;
        println!(
            "  Saved: {} ({}x{} at {}, {})",
            output_path.display(),
            region.width,
            region.height,
            region.x,
            region.y
        );
    }

    Ok(regions.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_region_area() {
        let region = SpriteRegion {
            x: 10,
            y: 20,
            width: 100,
            height: 50,
        };
        assert_eq!(region.area(), 5000);
    }

    #[test]
    fn test_detect_sprites_empty_image() {
        let image = ImageBuffer::from_fn(100, 100, |_, _| Rgba([0, 0, 0, 0]));
        let regions = detect_sprites(&image, 1, 4);
        assert!(regions.is_empty());
    }

    #[test]
    fn test_detect_sprites_single_rect() {
        let mut image = ImageBuffer::from_fn(100, 100, |_, _| Rgba([0, 0, 0, 0]));

        // Draw a 20x20 white square at (10, 10)
        for y in 10..30 {
            for x in 10..30 {
                image.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }

        let regions = detect_sprites(&image, 1, 4);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].x, 10);
        assert_eq!(regions[0].y, 10);
        assert_eq!(regions[0].width, 20);
        assert_eq!(regions[0].height, 20);
    }
}
