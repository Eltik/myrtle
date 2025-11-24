use anyhow::{Context, Result};
use hound::{SampleFormat, WavReader, WavWriter};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Voice file metadata extracted from filename
#[derive(Debug, Clone)]
struct VoiceMetadata {
    char_id: String,
    _char_name: String,
    language: String,
    sequence: u32,
    full_path: PathBuf,
}

/// Extract metadata from voice filename
/// Pattern: char_123_name_lang_001.wav
fn extract_voice_metadata(path: &Path) -> Option<VoiceMetadata> {
    let filename = path.file_stem()?.to_str()?;

    // Try pattern: char_XXX_name_lang_NNN
    let pattern = Regex::new(r"^(char_\d+)_([a-z_]+)_([a-z]+)_(\d+)$").ok()?;

    if let Some(caps) = pattern.captures(filename) {
        return Some(VoiceMetadata {
            char_id: caps.get(1)?.as_str().to_string(),
            _char_name: caps.get(2)?.as_str().to_string(),
            language: caps.get(3)?.as_str().to_string(),
            sequence: caps.get(4)?.as_str().parse().ok()?,
            full_path: path.to_path_buf(),
        });
    }

    // Fallback: use parent directory as category
    let parent = path.parent()?.file_name()?.to_str()?;
    Some(VoiceMetadata {
        char_id: parent.to_string(),
        _char_name: parent.to_string(),
        language: "unknown".to_string(),
        sequence: 0,
        full_path: path.to_path_buf(),
    })
}

/// Merge multiple WAV files into one with silence gaps
fn merge_wav_files(files: &[PathBuf], output: &Path, silence_ms: u32) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    // Read first file to get spec
    let first_reader =
        WavReader::open(&files[0]).with_context(|| format!("Failed to open {:?}", files[0]))?;
    let spec = first_reader.spec();
    drop(first_reader);

    // Create output writer
    let mut writer = WavWriter::create(output, spec)
        .with_context(|| format!("Failed to create {:?}", output))?;

    // Calculate silence samples
    let silence_samples = (spec.sample_rate * silence_ms / 1000) as usize;

    for (i, file) in files.iter().enumerate() {
        // Read and write samples
        let mut reader =
            WavReader::open(file).with_context(|| format!("Failed to open {:?}", file))?;

        // Check if spec matches
        let file_spec = reader.spec();
        if file_spec.channels != spec.channels || file_spec.sample_rate != spec.sample_rate {
            log::warn!(
                "Skipping {:?}: incompatible format ({}ch {}Hz vs {}ch {}Hz)",
                file,
                file_spec.channels,
                file_spec.sample_rate,
                spec.channels,
                spec.sample_rate
            );
            continue;
        }

        // Write samples based on format
        match spec.sample_format {
            SampleFormat::Int => {
                match spec.bits_per_sample {
                    16 => {
                        for sample in reader.samples::<i16>() {
                            writer.write_sample(sample?)?;
                        }
                    }
                    32 => {
                        for sample in reader.samples::<i32>() {
                            writer.write_sample(sample?)?;
                        }
                    }
                    _ => {
                        // Default to i16
                        for sample in reader.samples::<i16>() {
                            writer.write_sample(sample?)?;
                        }
                    }
                }
            }
            SampleFormat::Float => {
                for sample in reader.samples::<f32>() {
                    writer.write_sample(sample?)?;
                }
            }
        }

        // Add silence between clips (not after last)
        if i < files.len() - 1 {
            match spec.sample_format {
                SampleFormat::Int => {
                    for _ in 0..(silence_samples * spec.channels as usize) {
                        writer.write_sample(0i16)?;
                    }
                }
                SampleFormat::Float => {
                    for _ in 0..(silence_samples * spec.channels as usize) {
                        writer.write_sample(0.0f32)?;
                    }
                }
            }
        }
    }

    writer.finalize()?;
    Ok(())
}

/// Collect and organize voice files
pub fn main(srcdir: &Path, destdir: &Path, merge: bool) -> Result<()> {
    log::info!("Collecting voice files from {:?}", srcdir);

    // Group WAV files by character
    let mut groups: HashMap<String, Vec<VoiceMetadata>> = HashMap::new();

    for entry in WalkDir::new(srcdir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "wav" {
            continue;
        }

        if let Some(metadata) = extract_voice_metadata(path) {
            let key = format!("{}_{}", metadata.char_id, metadata.language);
            groups.entry(key).or_default().push(metadata);
        }
    }

    if groups.is_empty() {
        println!("No voice files found");
        return Ok(());
    }

    std::fs::create_dir_all(destdir)?;

    // Sort each group by sequence
    for voices in groups.values_mut() {
        voices.sort_by_key(|v| v.sequence);
    }

    // Progress bar
    let pb = ProgressBar::new(groups.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut total_copied = 0;
    let mut total_merged = 0;

    for (key, voices) in &groups {
        let char_dir = destdir.join(key);
        std::fs::create_dir_all(&char_dir)?;

        if merge && voices.len() > 1 {
            // Merge all voice files for this character
            let paths: Vec<PathBuf> = voices.iter().map(|v| v.full_path.clone()).collect();
            let merged_path = char_dir.join(format!("{}_merged.wav", key));

            match merge_wav_files(&paths, &merged_path, 500) {
                Ok(_) => {
                    total_merged += 1;
                    log::debug!("Merged {} files into {:?}", paths.len(), merged_path);
                }
                Err(e) => {
                    log::error!("Failed to merge {}: {}", key, e);
                }
            }
        }

        // Also copy individual files
        for voice in voices {
            let filename = voice.full_path.file_name().unwrap_or_default();
            let dest_path = char_dir.join(filename);
            std::fs::copy(&voice.full_path, &dest_path)?;
            total_copied += 1;
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done!");

    println!("\nVoice collection complete!");
    println!("  Collected {} voice files", total_copied);
    println!("  Characters: {}", groups.len());
    if merge {
        println!("  Merged packs: {}", total_merged);
    }

    Ok(())
}
