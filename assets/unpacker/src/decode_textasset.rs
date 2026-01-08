use aes::Aes128;
use anyhow::{Context, Result};
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

use crate::flatbuffers_decode;
use crate::resource_manifest::ResourceManifest;
use crate::utils::{is_binary_file, is_known_asset_file, is_unity_bundle, mkdir};

type Aes128CbcDec = cbc::Decryptor<Aes128>;

/// Arknights AES-CBC mask (chat_mask v2)
const MASK_V2: &[u8; 32] = b"UITpAi82pHAWwnzqHRMCwPonJLIB3WCl";

/// Decrypt AES-CBC encrypted Arknights data
pub fn aes_cbc_decrypt(data: &[u8], mask: &[u8; 32], has_rsa: bool) -> Result<Vec<u8>> {
    // Skip RSA signature if present (128 bytes)
    let data = if has_rsa && data.len() > 128 {
        &data[128..]
    } else {
        data
    };

    if data.len() < 16 {
        anyhow::bail!("Data too short for AES decryption (need at least 16 bytes)");
    }

    // Calculate key (first 16 bytes of mask)
    let key = &mask[..16];

    // Calculate IV (XOR of first 16 data bytes with last 16 mask bytes)
    let iv: [u8; 16] = data[..16]
        .iter()
        .zip(&mask[16..32])
        .map(|(&d, &m)| d ^ m)
        .collect::<Vec<u8>>()
        .try_into()
        .map_err(|_| anyhow::anyhow!("IV calculation failed"))?;

    // Decrypt the remaining data
    let mut decrypted = data[16..].to_vec();

    let cipher = Aes128CbcDec::new_from_slices(key, &iv)
        .map_err(|e| anyhow::anyhow!("Failed to create AES cipher: {:?}", e))?;

    // Decrypt in place
    let decrypted_len = cipher
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut decrypted)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?
        .len();

    decrypted.truncate(decrypted_len);
    Ok(decrypted)
}

/// Decode an encrypted Arknights file to JSON
pub fn decode_aes_file(path: &Path) -> Result<serde_json::Value> {
    let data = std::fs::read(path).with_context(|| format!("Failed to read file: {:?}", path))?;

    let decrypted = aes_cbc_decrypt(&data, MASK_V2, true)?;

    // Try JSON first
    match serde_json::from_slice(&decrypted) {
        Ok(json) => {
            log::debug!("Decoded JSON from {:?}", path);
            Ok(json)
        }
        Err(_) => {
            // Try BSON
            let doc: bson::Document = bson::Document::from_reader(Cursor::new(&decrypted))
                .with_context(|| format!("Failed to decode as JSON or BSON: {:?}", path))?;
            log::debug!("Decoded BSON from {:?}", path);
            Ok(serde_json::to_value(doc)?)
        }
    }
}

/// Check if output file is up-to-date (newer than input)
fn is_up_to_date(input: &Path, output: &Path) -> bool {
    if !output.exists() {
        return false;
    }

    let input_time = match input.metadata().and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };

    let output_time = match output.metadata().and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return false,
    };

    output_time >= input_time
}

/// Strip hash suffix from filename (6 hex chars at the end)
fn strip_hash_suffix(filename: &str) -> String {
    let hash_suffix_re = Regex::new(r"[a-f0-9]{6}$").unwrap();
    hash_suffix_re.replace(filename, "").to_string()
}

/// Get output path for a file, using manifest if available
fn get_output_path(filename: &str, destdir: &Path, manifest: Option<&ResourceManifest>) -> PathBuf {
    if let Some(m) = manifest {
        if let Some(path) = m.get_output_path(filename) {
            // Use manifest path with .json extension
            return destdir.join(format!("{}.json", path));
        }
    }
    // Fallback: strip hash suffix and output directly to destdir
    let clean_name = strip_hash_suffix(filename);
    destdir.join(format!("{}.json", clean_name))
}

/// Process a single text asset file
fn text_asset_resolve(
    fp: &Path,
    destdir: &Path,
    manifest: Option<&ResourceManifest>,
) -> Result<bool> {
    // Skip if not a binary file
    if !fp.is_file() || !is_binary_file(fp) {
        return Ok(false);
    }

    let filename = fp.file_name().unwrap_or_default().to_string_lossy();
    let output_path = get_output_path(&filename, destdir, manifest);

    // Skip if output is already up-to-date
    if is_up_to_date(fp, &output_path) {
        log::debug!("Skipping {:?} (up-to-date)", fp);
        return Ok(false);
    }

    let data = std::fs::read(fp)?;

    // Try FlatBuffer decoding first (skip 128-byte RSA signature)
    if data.len() > 128 {
        let fbo_data = &data[128..];
        if flatbuffers_decode::is_flatbuffer(fbo_data) {
            match flatbuffers_decode::decode_flatbuffer(fbo_data, &filename) {
                Ok(json) => {
                    let json_str = serde_json::to_string_pretty(&json)?;

                    // Create parent directories
                    if let Some(parent) = output_path.parent() {
                        mkdir(parent)?;
                    }
                    std::fs::write(&output_path, json_str)?;

                    log::debug!("Decoded FlatBuffer: {:?} -> {:?}", fp, output_path);
                    return Ok(true);
                }
                Err(e) => {
                    log::debug!("FlatBuffer decode failed for {:?}: {}", fp, e);
                }
            }
        }
    }

    // Try AES decryption
    match decode_aes_file(fp) {
        Ok(json) => {
            let json_str = serde_json::to_string_pretty(&json)?;

            // Create parent directories
            if let Some(parent) = output_path.parent() {
                mkdir(parent)?;
            }
            std::fs::write(&output_path, json_str)?;

            log::debug!("Decoded AES: {:?} -> {:?}", fp, output_path);
            return Ok(true);
        }
        Err(e) => {
            log::debug!("AES decryption failed for {:?}: {}", fp, e);
        }
    }

    // Final fallback: If it looks like a FlatBuffer but schema failed, extract strings
    if data.len() > 128 {
        let fbo_data = &data[128..];
        if flatbuffers_decode::is_flatbuffer(fbo_data) {
            let strings = flatbuffers_decode::extract_strings(fbo_data);
            if !strings.is_empty() {
                let json = serde_json::json!({
                    "type": "unknown_flatbuffer",
                    "note": "Schema mismatch - raw string extraction",
                    "strings": strings
                });
                let json_str = serde_json::to_string_pretty(&json)?;

                if let Some(parent) = output_path.parent() {
                    mkdir(parent)?;
                }
                std::fs::write(&output_path, json_str)?;

                log::debug!(
                    "Extracted strings from FlatBuffer: {:?} -> {:?}",
                    fp,
                    output_path
                );
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Try to find the .idx manifest file automatically
fn find_manifest(rootdir: &Path) -> Option<PathBuf> {
    // Look for .idx files in parent directories (up to 3 levels)
    let mut current = rootdir.to_path_buf();
    for _ in 0..5 {
        // Check for any .idx file in this directory
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "idx") {
                    log::info!("Found manifest: {:?}", path);
                    return Some(path);
                }
            }
        }

        // Move to parent
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

/// Main entry point for TextAsset decoding
pub fn main(
    rootdir: &Path,
    destdir: &Path,
    do_del: bool,
    manifest_path: Option<&Path>,
) -> Result<()> {
    log::info!("Retrieving file paths...");

    // Try to load manifest
    let manifest = if let Some(path) = manifest_path {
        match ResourceManifest::load(path) {
            Ok(m) => {
                println!(
                    "Loaded manifest from {:?} with {} entries",
                    path,
                    m.filename_to_path.len()
                );
                Some(Arc::new(m))
            }
            Err(e) => {
                log::warn!("Failed to load manifest from {:?}: {}", path, e);
                None
            }
        }
    } else if let Some(found_path) = find_manifest(rootdir) {
        match ResourceManifest::load(&found_path) {
            Ok(m) => {
                println!(
                    "Auto-detected manifest from {:?} with {} entries",
                    found_path,
                    m.filename_to_path.len()
                );
                Some(Arc::new(m))
            }
            Err(e) => {
                log::warn!("Failed to load auto-detected manifest: {}", e);
                None
            }
        }
    } else {
        log::info!("No manifest found, using fallback filename stripping");
        None
    };

    // Collect files to process (exclude known assets and AB files)
    let files: Vec<PathBuf> = WalkDir::new(rootdir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.is_file())
        .filter(|p| !is_known_asset_file(p))
        .filter(|p| !is_unity_bundle(p))
        .collect();

    if files.is_empty() {
        println!("No files to process");
        return Ok(());
    }

    if do_del && destdir.exists() {
        log::info!("Cleaning destination directory...");
        std::fs::remove_dir_all(destdir)?;
    }

    std::fs::create_dir_all(destdir)?;

    // Progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let decoded_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);
    let manifest_ref = manifest.as_deref();

    // Suppress panic output during parallel processing
    // (FlatBuffer decodes may panic on schema mismatches - this is expected and handled)
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // Silent panic hook

    // Process files in parallel
    files.par_iter().for_each(|file| {
        match text_asset_resolve(file, destdir, manifest_ref) {
            Ok(true) => {
                decoded_count.fetch_add(1, Ordering::Relaxed);
            }
            Ok(false) => {
                // Check if it was skipped due to being up-to-date
                let filename = file.file_name().unwrap_or_default().to_string_lossy();
                let output_path = get_output_path(&filename, destdir, manifest_ref);
                if output_path.exists() {
                    skipped_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err(e) => {
                log::error!("Error processing {:?}: {}", file, e);
            }
        }

        pb.inc(1);
    });

    pb.finish_with_message("Done!");

    // Restore original panic hook
    std::panic::set_hook(prev_hook);

    let decoded = decoded_count.load(Ordering::Relaxed);
    let skipped = skipped_count.load(Ordering::Relaxed);

    println!("\nTextAsset decoding complete!");
    println!("  Searched {} files", files.len());
    println!("  Decoded {} files", decoded);
    if skipped > 0 {
        println!("  Skipped {} files (up-to-date)", skipped);
    }

    Ok(())
}
