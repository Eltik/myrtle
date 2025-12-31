//! Post-processing reorganization module for extracted assets.
//!
//! This module handles moving extracted files from their raw extraction location
//! (e.g., `anon/[hash]/`) to proper game paths (e.g., `gamedata/story/obt/main/`)
//! within the same directory structure (in-place reorganization).

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::generated_fbs::resource_manifest_generated::*;

/// Mapping from bundle hash to list of (asset_name, game_path) pairs
pub struct BundlePathMapping {
    /// Maps bundle hash (e.g., "d4d578962b25496976daab70b0cc7801")
    /// to list of (filename, output_path) tuples
    pub bundle_to_assets: HashMap<String, Vec<AssetPathInfo>>,

    /// Maps asset name (lowercase) directly to output path (for quick lookup)
    /// Prefers non-[uc]info paths over [uc]info paths
    pub asset_to_path: HashMap<String, String>,

    /// Maps asset name (lowercase, with hash suffix) to output path
    pub asset_to_path_with_hash: HashMap<String, String>,
}

/// Information about an asset's path mapping
#[derive(Debug, Clone)]
pub struct AssetPathInfo {
    /// The asset's filename (without hash suffix)
    pub filename: String,
    /// The asset's filename (with hash suffix, as stored in bundle)
    pub filename_with_hash: String,
    /// The proper game path (e.g., "gamedata/story/obt/main/level_main_00-01_beg.txt")
    pub game_path: String,
}

impl BundlePathMapping {
    /// Load bundle-to-path mapping from resource manifest (.idx file)
    pub fn load(idx_path: &Path) -> Result<Self> {
        let data = std::fs::read(idx_path)
            .with_context(|| format!("Failed to read manifest file: {:?}", idx_path))?;

        // Skip first 128 bytes (RSA signature)
        if data.len() < 128 {
            anyhow::bail!("Manifest file too small");
        }
        let fb_data = &data[128..];

        // Decode FlatBuffer
        let manifest = unsafe { root_as_clz_torappu_resource_resource_manifest_unchecked(fb_data) };

        let mut bundle_to_assets: HashMap<String, Vec<AssetPathInfo>> = HashMap::new();
        let mut asset_to_path: HashMap<String, String> = HashMap::new();
        let mut asset_to_path_with_hash: HashMap<String, String> = HashMap::new();

        // Get bundle list for index lookup
        let bundles = manifest.bundles();

        if let (Some(assets), Some(bundles)) = (manifest.assetToBundleList(), bundles) {
            for i in 0..assets.len() {
                let asset = assets.get(i);

                // Get the path (e.g., "dyn/gamedata/story/obt/main/level_main_00-01_beg.txt.bytes")
                let path = match asset.path() {
                    Some(p) => p,
                    None => continue,
                };

                // Get the filename (with hash suffix)
                let name_with_hash = match asset.name() {
                    Some(n) => n,
                    None => continue,
                };

                // Get the bundle index
                let bundle_idx = asset.bundleIndex() as usize;
                if bundle_idx >= bundles.len() {
                    continue;
                }

                // Get the bundle hash name
                let bundle = bundles.get(bundle_idx);
                let bundle_name = match bundle.name() {
                    Some(n) => n,
                    None => continue,
                };

                // Clean up the path:
                // 1. Remove "dyn/" prefix
                // 2. Remove ".bytes" suffix
                let game_path = path
                    .strip_prefix("dyn/")
                    .unwrap_or(path)
                    .strip_suffix(".bytes")
                    .unwrap_or(path)
                    .to_string();

                // Extract clean filename (without hash suffix)
                let filename = strip_hash_suffix(name_with_hash);

                let info = AssetPathInfo {
                    filename: filename.to_string(),
                    filename_with_hash: name_with_hash.to_string(),
                    game_path: game_path.clone(),
                };

                // Add to bundle mapping
                bundle_to_assets
                    .entry(bundle_name.to_string())
                    .or_default()
                    .push(info);

                // Check if this is a [uc]info path (summary) vs non-[uc] path (full content)
                let is_uc_info = game_path.contains("[uc]info");

                // Use lowercase keys for case-insensitive matching
                let key_lower = filename.to_lowercase();
                let key_with_hash_lower = name_with_hash.to_lowercase();

                // Only insert if:
                // 1. Key doesn't exist yet, OR
                // 2. Current path is not [uc]info and existing path is [uc]info
                //    (prefer full content paths over summary paths)
                if let Some(existing) = asset_to_path.get(&key_lower) {
                    let existing_is_uc_info = existing.contains("[uc]info");
                    if !is_uc_info && existing_is_uc_info {
                        // Replace [uc]info path with non-[uc] path
                        asset_to_path.insert(key_lower.clone(), game_path.clone());
                    }
                    // Otherwise keep the existing (prefer first non-[uc] path found)
                } else {
                    asset_to_path.insert(key_lower.clone(), game_path.clone());
                }

                // Same logic for with-hash lookup
                if let Some(existing) = asset_to_path_with_hash.get(&key_with_hash_lower) {
                    let existing_is_uc_info = existing.contains("[uc]info");
                    if !is_uc_info && existing_is_uc_info {
                        asset_to_path_with_hash.insert(key_with_hash_lower, game_path);
                    }
                } else {
                    asset_to_path_with_hash.insert(key_with_hash_lower, game_path);
                }
            }
        }

        log::info!(
            "Loaded bundle mapping: {} bundles, {} assets",
            bundle_to_assets.len(),
            asset_to_path.len()
        );

        Ok(BundlePathMapping {
            bundle_to_assets,
            asset_to_path,
            asset_to_path_with_hash,
        })
    }

    /// Get the proper output path for an asset (case-insensitive matching)
    pub fn get_output_path(&self, asset_name: &str) -> Option<&String> {
        // Convert to lowercase for case-insensitive matching
        let name_lower = asset_name.to_lowercase();

        // Try exact match first (already lowercase in our map)
        if let Some(path) = self.asset_to_path.get(&name_lower) {
            return Some(path);
        }

        // Try with hash suffix map
        if let Some(path) = self.asset_to_path_with_hash.get(&name_lower) {
            return Some(path);
        }

        // Try without common extensions
        let name_no_ext = name_lower
            .strip_suffix(".txt")
            .or_else(|| name_lower.strip_suffix(".json"))
            .or_else(|| name_lower.strip_suffix(".bytes"))
            .unwrap_or(&name_lower);

        if let Some(path) = self.asset_to_path.get(name_no_ext) {
            return Some(path);
        }

        self.asset_to_path_with_hash.get(name_no_ext)
    }

    /// Get all assets for a given bundle hash
    pub fn get_bundle_assets(&self, bundle_hash: &str) -> Option<&Vec<AssetPathInfo>> {
        self.bundle_to_assets.get(bundle_hash)
    }
}

/// Strip 6-character hex hash suffix from filename
fn strip_hash_suffix(name: &str) -> &str {
    if name.len() < 7 {
        return name;
    }

    let suffix_start = name.len() - 6;
    let suffix = &name[suffix_start..];

    // Check if suffix is valid hex
    if suffix.chars().all(|c| c.is_ascii_hexdigit()) {
        &name[..suffix_start]
    } else {
        name
    }
}

/// Statistics about reorganization
#[derive(Debug, Default)]
pub struct ReorganizeStats {
    pub files_processed: usize,
    pub files_moved: usize,
    pub files_skipped: usize,
    pub directories_created: usize,
    pub directories_removed: usize,
    pub errors: usize,
}

/// Reorganize extracted files from hash-based folders to proper game paths
pub fn reorganize_inplace(
    source_dir: &Path,
    output_dir: &Path,
    mapping: &BundlePathMapping,
) -> Result<ReorganizeStats> {
    let mut stats = ReorganizeStats::default();
    let mut empty_dirs: Vec<PathBuf> = Vec::new();
    let in_place = source_dir == output_dir;

    // Look for anon directories containing hash-based folders
    // Check multiple possible locations (direct, nested upk, etc.)
    let possible_anon_dirs = [source_dir.join("anon"), source_dir.join("upk").join("anon")];

    // Collect hash folders from all found anon directories
    let mut hash_folders: Vec<walkdir::DirEntry> = Vec::new();

    for anon_dir in possible_anon_dirs.iter().filter(|d| d.exists()) {
        log::info!("Searching anon directory: {:?}", anon_dir);
        let folders: Vec<_> = WalkDir::new(anon_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                is_hash_folder(&name) || name.starts_with("[unpack]")
            })
            .collect();
        hash_folders.extend(folders);
    }

    // Also check source_dir itself for hash folders
    if hash_folders.is_empty() {
        hash_folders = WalkDir::new(source_dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                is_hash_folder(&name) || name.starts_with("[unpack]")
            })
            .collect();
    }

    println!("Found {} hash folders to process", hash_folders.len());

    // Process each hash folder
    for entry in hash_folders {
        let folder_name = entry.file_name().to_string_lossy();
        let bundle_hash = if folder_name.starts_with("[unpack]") {
            folder_name.trim_start_matches("[unpack]").to_string()
        } else {
            folder_name.to_string()
        };

        // Build lookup maps for this bundle (using lowercase keys for case-insensitive matching)
        let bundle_assets = mapping.get_bundle_assets(&bundle_hash);

        // Create lookups with lowercase keys, preferring non-[uc]info paths
        let mut asset_lookup: HashMap<String, &AssetPathInfo> = HashMap::new();
        let mut asset_lookup_with_hash: HashMap<String, &AssetPathInfo> = HashMap::new();

        if let Some(assets) = bundle_assets {
            for a in assets {
                let key = a.filename.to_lowercase();
                let key_with_hash = a.filename_with_hash.to_lowercase();
                let is_uc_info = a.game_path.contains("[uc]info");

                // Prefer non-[uc]info paths over [uc]info paths
                if let Some(existing) = asset_lookup.get(&key) {
                    if !is_uc_info && existing.game_path.contains("[uc]info") {
                        asset_lookup.insert(key, a);
                    }
                } else {
                    asset_lookup.insert(key, a);
                }

                if let Some(existing) = asset_lookup_with_hash.get(&key_with_hash) {
                    if !is_uc_info && existing.game_path.contains("[uc]info") {
                        asset_lookup_with_hash.insert(key_with_hash, a);
                    }
                } else {
                    asset_lookup_with_hash.insert(key_with_hash, a);
                }
            }
        }

        let mut folder_emptied = true;

        // Process files in this folder
        for file_entry in WalkDir::new(entry.path())
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            stats.files_processed += 1;
            let file_name = file_entry.file_name().to_string_lossy();
            let file_name_lower = file_name.to_lowercase();
            let source_path = file_entry.path();

            // Try to find game path from manifest (using lowercase for case-insensitive matching)
            let game_path = asset_lookup
                .get(&file_name_lower)
                .or_else(|| asset_lookup_with_hash.get(&file_name_lower))
                .map(|info| info.game_path.clone())
                .or_else(|| mapping.get_output_path(&file_name).cloned())
                .or_else(|| infer_path_from_filename(&file_name));

            if let Some(rel_path) = game_path {
                // Strip "dyn/" prefix if present (decoded folder shouldn't have it)
                let clean_path = rel_path.strip_prefix("dyn/").unwrap_or(&rel_path);
                let dest_path = output_dir.join(clean_path);

                // Skip if source and dest are the same
                if source_path == dest_path {
                    stats.files_skipped += 1;
                    folder_emptied = false;
                    continue;
                }

                // Skip raw encrypted/binary files (should be decoded first)
                // These have hash suffixes and contain non-text data
                if is_raw_encrypted_file(source_path) {
                    log::debug!("Skipping raw encrypted file: {}", source_path.display());
                    stats.files_skipped += 1;
                    folder_emptied = false;
                    continue;
                }

                // If destination exists, prefer files with actual dialogue content
                // (full dialogue scripts vs short summaries)
                if dest_path.exists() {
                    if !should_overwrite(source_path, &dest_path) {
                        stats.files_skipped += 1;
                        folder_emptied = false;
                        continue;
                    }
                }

                // Create parent directory
                if let Some(parent) = dest_path.parent() {
                    if !parent.exists() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            log::error!("Failed to create directory {:?}: {}", parent, e);
                            stats.errors += 1;
                            folder_emptied = false;
                            continue;
                        }
                        stats.directories_created += 1;
                    }
                }

                // Move or copy file depending on mode
                let result = if in_place {
                    // In-place: try rename first, fallback to copy+delete
                    std::fs::rename(source_path, &dest_path).or_else(|_| {
                        std::fs::copy(source_path, &dest_path).map(|_| {
                            let _ = std::fs::remove_file(source_path);
                        })
                    })
                } else {
                    // Cross-directory: copy file (keep source)
                    std::fs::copy(source_path, &dest_path).map(|_| ())
                };

                match result {
                    Ok(_) => {
                        stats.files_moved += 1;
                        log::debug!("Moved {} -> {}", source_path.display(), dest_path.display());
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to move {} -> {}: {}",
                            source_path.display(),
                            dest_path.display(),
                            e
                        );
                        stats.errors += 1;
                        folder_emptied = false;
                    }
                }
            } else {
                stats.files_skipped += 1;
                folder_emptied = false;
                log::debug!("No mapping found for: {}", file_name);
            }
        }

        // Mark folder for removal if emptied (only in in-place mode)
        if in_place && folder_emptied {
            empty_dirs.push(entry.path().to_path_buf());
        }
    }

    // Clean up empty hash folders (only in in-place mode)
    for dir in empty_dirs {
        // Double-check it's empty (no files, maybe just .DS_Store)
        let is_empty = WalkDir::new(&dir)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.file_name().to_string_lossy() != ".DS_Store")
            .next()
            .is_none();

        if is_empty {
            if let Err(e) = std::fs::remove_dir_all(&dir) {
                log::warn!("Failed to remove empty directory {:?}: {}", dir, e);
            } else {
                stats.directories_removed += 1;
                log::debug!("Removed empty directory: {}", dir.display());
            }
        }
    }

    Ok(stats)
}

/// Check if a folder name looks like a hash (32 hex chars)
fn is_hash_folder(name: &str) -> bool {
    name.len() == 32 && name.chars().all(|c| c.is_ascii_hexdigit())
}

/// Infer game path from filename patterns
fn infer_path_from_filename(filename: &str) -> Option<String> {
    // Story files: level_main_*, level_sub_*
    if filename.starts_with("level_main_") || filename.starts_with("level_sub_") {
        let proper_name = ensure_txt_extension(filename);
        return Some(format!("gamedata/story/obt/main/{}", proper_name));
    }

    // Activity story files
    if filename.starts_with("level_act") {
        let proper_name = ensure_txt_extension(filename);
        // Try to extract activity ID for subfolder
        if let Some(act_id) = extract_activity_id(filename) {
            return Some(format!(
                "gamedata/story/activities/{}/{}",
                act_id, proper_name
            ));
        }
        return Some(format!("gamedata/story/activities/{}", proper_name));
    }

    // Guide/tutorial files
    if filename.starts_with("guide_") {
        let proper_name = ensure_txt_extension(filename);
        return Some(format!("gamedata/story/obt/guide/{}", proper_name));
    }

    // Memory story files
    if filename.starts_with("mem_") {
        let proper_name = ensure_txt_extension(filename);
        return Some(format!("gamedata/story/obt/memory/{}", proper_name));
    }

    // Roguelike story files
    if filename.starts_with("ro_") || filename.starts_with("rogue_") {
        let proper_name = ensure_txt_extension(filename);
        return Some(format!("gamedata/story/obt/roguelike/{}", proper_name));
    }

    // Character/game data tables (JSON files)
    if filename.ends_with("_table") || filename.ends_with("_data") {
        let proper_name = ensure_json_extension(filename);
        return Some(format!("gamedata/excel/{}", proper_name));
    }

    None
}

/// Extract activity ID from filename (e.g., "level_act13side_01_beg" -> "act13side")
fn extract_activity_id(filename: &str) -> Option<String> {
    let name = filename.strip_prefix("level_")?;
    // Find the activity ID (e.g., act13side, act1d0, etc.)
    let parts: Vec<&str> = name.splitn(2, '_').collect();
    if parts.is_empty() {
        return None;
    }
    let act_part = parts[0];
    if act_part.starts_with("act") {
        Some(act_part.to_string())
    } else {
        None
    }
}

/// Ensure filename has .txt extension
fn ensure_txt_extension(filename: &str) -> String {
    if filename.ends_with(".txt") {
        filename.to_string()
    } else if filename.contains('.') {
        filename.to_string()
    } else {
        format!("{}.txt", filename)
    }
}

/// Ensure filename has .json extension
fn ensure_json_extension(filename: &str) -> String {
    if filename.ends_with(".json") {
        filename.to_string()
    } else if filename.contains('.') {
        filename.to_string()
    } else {
        format!("{}.json", filename)
    }
}

/// Check if a file is a raw encrypted/binary file that should be decoded first.
/// These are typically FlatBuffer-encoded game data tables with hash suffixes.
fn is_raw_encrypted_file(path: &Path) -> bool {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Files with proper extensions are likely already decoded
    if filename.ends_with(".json") || filename.ends_with(".txt") {
        return false;
    }

    // Check if filename looks like it has a hash suffix (6 hex chars at end)
    // e.g., "activity_table0a200c" vs "activity_table"
    if filename.len() > 6 {
        let suffix = &filename[filename.len() - 6..];
        let has_hash_suffix = suffix.chars().all(|c| c.is_ascii_hexdigit());

        // If it has a hash suffix and contains _table or _data, it's likely raw encrypted
        if has_hash_suffix
            && (filename.contains("_table")
                || filename.contains("_data")
                || filename.contains("_database"))
        {
            // Verify by checking first few bytes for binary content
            if let Ok(data) = std::fs::read(path) {
                if data.len() > 4 {
                    // JSON starts with '{' or '[', text files are UTF-8
                    // Encrypted files start with random-looking bytes
                    let first_byte = data[0];
                    if first_byte != b'{' && first_byte != b'[' && first_byte > 127 {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Determine if source file should overwrite destination file.
/// Prefers files with actual dialogue content over summaries.
/// For story files, full dialogue scripts contain [Dialog] or [name= markers.
fn should_overwrite(source: &Path, dest: &Path) -> bool {
    // For non-text files, always overwrite
    let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "txt"
        && !source.to_string_lossy().ends_with("_beg")
        && !source.to_string_lossy().ends_with("_end")
        && !source.to_string_lossy().ends_with("_BEG")
        && !source.to_string_lossy().ends_with("_END")
    {
        return true;
    }

    // Read first 1KB of both files to check for dialogue markers
    let source_content = std::fs::read(source)
        .ok()
        .map(|d| String::from_utf8_lossy(&d[..d.len().min(4096)]).to_string());
    let dest_content = std::fs::read(dest)
        .ok()
        .map(|d| String::from_utf8_lossy(&d[..d.len().min(4096)]).to_string());

    fn has_dialogue_markers(content: &str) -> bool {
        content.contains("[Dialog]")
            || content.contains("[name=")
            || content.contains("[Character(")
    }

    fn is_summary_only(content: &str) -> bool {
        // Summaries are typically short (< 500 chars) and don't have dialogue markers
        content.len() < 500 && !has_dialogue_markers(content)
    }

    match (source_content, dest_content) {
        (Some(src), Some(dst)) => {
            let src_has_dialogue = has_dialogue_markers(&src);
            let dst_has_dialogue = has_dialogue_markers(&dst);
            let src_is_summary = is_summary_only(&src);
            let dst_is_summary = is_summary_only(&dst);

            // Prefer source if it has dialogue and dest doesn't
            if src_has_dialogue && !dst_has_dialogue {
                log::debug!("Overwriting summary with full dialogue: {}", dest.display());
                return true;
            }
            // Don't overwrite if dest has dialogue and source doesn't
            if dst_has_dialogue && !src_has_dialogue {
                log::debug!(
                    "Keeping full dialogue, skipping summary: {}",
                    source.display()
                );
                return false;
            }
            // Prefer larger file if both have or both lack dialogue
            if src_is_summary && !dst_is_summary {
                return false;
            }
            if !src_is_summary && dst_is_summary {
                return true;
            }
            // Fall back to size comparison
            let src_size = std::fs::metadata(source).map(|m| m.len()).unwrap_or(0);
            let dst_size = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
            src_size > dst_size
        }
        (Some(_), None) => true,
        (None, _) => false,
    }
}

/// Main entry point for the reorganize command
pub fn main(
    source_dir: &Path,
    dest_dir: Option<&Path>,
    manifest_path: Option<&Path>,
) -> Result<()> {
    let output_dir = dest_dir.unwrap_or(source_dir);

    if dest_dir.is_some() {
        println!("Reorganizing assets...");
        println!("  Source: {}", source_dir.display());
        println!("  Dest:   {}", output_dir.display());
    } else {
        println!("Reorganizing assets in-place...");
        println!("  Directory: {}", source_dir.display());
    }

    // Create output directory if different from source
    if dest_dir.is_some() {
        std::fs::create_dir_all(output_dir)?;
    }

    // Try to find manifest if not specified
    let manifest_path = if let Some(p) = manifest_path {
        Some(p.to_path_buf())
    } else {
        find_manifest(source_dir)
    };

    // Load bundle mapping
    let mapping = if let Some(ref mp) = manifest_path {
        println!("Loading manifest from: {}", mp.display());
        match BundlePathMapping::load(mp) {
            Ok(m) => {
                println!(
                    "  Loaded {} bundles, {} assets",
                    m.bundle_to_assets.len(),
                    m.asset_to_path.len()
                );
                m
            }
            Err(e) => {
                log::warn!("Failed to load manifest: {}", e);
                println!("  Warning: Could not load manifest, using inference only");
                BundlePathMapping {
                    bundle_to_assets: HashMap::new(),
                    asset_to_path: HashMap::new(),
                    asset_to_path_with_hash: HashMap::new(),
                }
            }
        }
    } else {
        println!("  No manifest found, using inference only");
        BundlePathMapping {
            bundle_to_assets: HashMap::new(),
            asset_to_path: HashMap::new(),
            asset_to_path_with_hash: HashMap::new(),
        }
    };

    // Perform reorganization
    println!("\nProcessing files...");
    let stats = reorganize_inplace(source_dir, output_dir, &mapping)?;

    // Print summary
    println!("\nReorganization complete:");
    println!("  Files processed:    {}", stats.files_processed);
    println!("  Files moved:        {}", stats.files_moved);
    println!("  Files skipped:      {}", stats.files_skipped);
    println!("  Dirs created:       {}", stats.directories_created);
    println!("  Empty dirs removed: {}", stats.directories_removed);
    if stats.errors > 0 {
        println!("  Errors:             {}", stats.errors);
    }

    Ok(())
}

/// Find manifest file in common locations
fn find_manifest(base_dir: &Path) -> Option<PathBuf> {
    let mut current = base_dir;
    for _ in 0..5 {
        // Look for .idx files directly
        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map(|e| e == "idx").unwrap_or(false) {
                    return Some(path);
                }
            }
        }

        // Look in common subfolders
        for subfolder in &["ArkAssets", "downloader/ArkAssets"] {
            let check_dir = current.join(subfolder);
            if check_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&check_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().map(|e| e == "idx").unwrap_or(false) {
                            return Some(path);
                        }
                    }
                }
            }
        }

        current = current.parent()?;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_hash_suffix() {
        assert_eq!(
            strip_hash_suffix("character_tabled88efb"),
            "character_table"
        );
        assert_eq!(strip_hash_suffix("enemy_database3cc1f2"), "enemy_database");
        assert_eq!(strip_hash_suffix("short"), "short");
        assert_eq!(strip_hash_suffix("no_hash_here"), "no_hash_here");
    }

    #[test]
    fn test_is_hash_folder() {
        assert!(is_hash_folder("d4d578962b25496976daab70b0cc7801"));
        assert!(!is_hash_folder("chararts"));
        assert!(!is_hash_folder("too_short"));
        assert!(!is_hash_folder("[unpack]abc"));
    }

    #[test]
    fn test_infer_path() {
        assert_eq!(
            infer_path_from_filename("level_main_00-01_beg"),
            Some("gamedata/story/obt/main/level_main_00-01_beg.txt".to_string())
        );
        assert_eq!(
            infer_path_from_filename("level_main_15-12_end"),
            Some("gamedata/story/obt/main/level_main_15-12_end.txt".to_string())
        );
        assert_eq!(
            infer_path_from_filename("level_act13side_03_beg"),
            Some("gamedata/story/activities/act13side/level_act13side_03_beg.txt".to_string())
        );
    }

    #[test]
    fn test_extract_activity_id() {
        assert_eq!(
            extract_activity_id("level_act13side_03_beg"),
            Some("act13side".to_string())
        );
        assert_eq!(
            extract_activity_id("level_act1d0_01_beg"),
            Some("act1d0".to_string())
        );
        assert_eq!(extract_activity_id("level_main_01_beg"), None);
    }
}
