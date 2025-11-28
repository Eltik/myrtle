use aes::Aes128;
use anyhow::{Context, Result};
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

use crate::flatbuffers_decode;
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
            let doc: bson::Document = bson::from_slice(&decrypted)
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

/// Process a single text asset file
fn text_asset_resolve(fp: &Path, destdir: &Path) -> Result<bool> {
    // Skip if not a binary file
    if !fp.is_file() || !is_binary_file(fp) {
        return Ok(false);
    }

    let filename = fp.file_name().unwrap_or_default().to_string_lossy();
    let output_path = destdir.join(format!("{}.json", filename));

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
                    let output_path = destdir.join(format!("{}.json", filename));

                    mkdir(destdir)?;
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
            let output_path = destdir.join(format!("{}.json", filename));

            mkdir(destdir)?;
            std::fs::write(&output_path, json_str)?;

            log::debug!("Decoded AES: {:?} -> {:?}", fp, output_path);
            return Ok(true);
        }
        Err(e) => {
            log::debug!("AES decryption failed for {:?}: {}", fp, e);
        }
    }

    Ok(false)
}

/// Main entry point for TextAsset decoding
pub fn main(rootdir: &Path, destdir: &Path, do_del: bool) -> Result<()> {
    log::info!("Retrieving file paths...");

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

    // Process files in parallel
    files.par_iter().for_each(|file| {
        let rel_path = file.strip_prefix(rootdir).unwrap_or(file);
        let subdestdir = destdir.join(rel_path.parent().unwrap_or(Path::new("")));

        match text_asset_resolve(file, &subdestdir) {
            Ok(true) => {
                decoded_count.fetch_add(1, Ordering::Relaxed);
            }
            Ok(false) => {
                // Check if it was skipped due to being up-to-date
                let filename = file.file_name().unwrap_or_default().to_string_lossy();
                let output_path = subdestdir.join(format!("{}.json", filename));
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
