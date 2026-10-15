//! An AAC `.m4a` beside every in-game music `.ogg`, for the browsers that cannot decode Vorbis.
//!
//! The music player and the story reader decode the game's BGM with `decodeAudioData` and
//! schedule the intro and the loop on the audio clock, which is what keeps the join gapless.
//! The clips are Ogg Vorbis rebuilt from FSB5 (`fsb5.rs`); where a browser has no Vorbis
//! decoder (iOS Safari is the case this exists for, unverified on a device as of 2026-10-13)
//! the frontend fetches the `.m4a` sibling instead (`frontend/src/lib/story/audioBuffers.ts`).
//!
//! WHICH FILES: every `.ogg` under `audio/audio/sound_beta_2/music/`, plus the `m_*.ogg` music
//! clips under `audio/anon/` (the title theme, `m_sys_title_{intro,loop}`, ships in an
//! anonymous bundle). Voices and SFX are not transcoded. Measured on the EN tree 2026-10-13:
//! 863 music clips, 799 at 44.1 kHz stereo and 64 at 48 kHz stereo. The sample rate is
//! KEPT, not forced to 44.1 kHz: a resample filters each file on its own, so an intro and a
//! loop resampled apart would no longer meet sample-exactly at the join.
//!
//! THE PRIMING IS CARRIED, NOT LEAKED. ffmpeg's `aac` encoder emits 1024 priming samples and
//! pads the last frame out to 1024, and the mp4 muxer writes an `elst` edit list that names
//! the valid range. Measured 2026-10-13 on `m_bat2_vectorbreak2` (ffmpeg 9.0.1): the intro is
//! 1,591,680 source samples and 1556 AAC packets (1024 priming + 640 remainder), the loop
//! 6,048,000 samples and 5908 packets (1024 + 768). macOS AudioToolbox (`afinfo`, `afconvert`)
//! reads the edit list and decodes exactly 1,591,680 and 6,048,000 frames at lag 0 against
//! the Vorbis decode (cross-correlation 0.99999 and 0.9995). WebKit decodes through the same
//! framework, which is why no trim is applied client side; iOS itself was not measured.
//!
//! FRESHNESS IS BY CONTENT. The extract is non-incremental and rewrites every `.ogg` on every
//! run, so an mtime test alone would re-encode all 863 clips each time. A clip is up to date
//! when its `.m4a` is newer than its `.ogg` (made since the last write) OR when the manifest
//! `audio/m4a_sources.json` records the `.ogg`'s current fingerprint for it. A skipped `.m4a`
//! is touched, the way `video.rs` touches a skipped clip, so the orphan sweep in
//! `assets/orphans.mjs` (every file in a written subtree older than the run is deleted)
//! keeps it, and the manifest is rewritten on every pass for the same reason.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, SystemTime};

use rayon::prelude::*;
use walkdir::WalkDir;

pub use super::video::TreeReport;
use super::video::{ffmpeg_available, file_len, newer_than, run_ffmpeg, touch};

/// AAC-LC bitrate. The Vorbis sources run about 105 to 120 kbps, so this is headroom for a
/// lossy-to-lossy pass, not a match: `m_bat2_vectorbreak2_loop` goes 2,000,271 B -> 3,502,748 B.
const AAC_BITRATE: &str = "192k";

/// The manifest, relative to the output tree. Under `audio/` so a pass that writes it lands in
/// a subtree the extract writes anyway.
const MANIFEST: &str = "audio/m4a_sources.json";

/// Most ffmpeg processes at once. The encoder is close to single threaded (113% CPU measured
/// on one clip, 137 s of audio in 1.6 s wall) and the VPS has 3 cores that also serve the site.
const MAX_JOBS: usize = 4;

/// Why a pass did no work at all.
#[derive(Debug)]
pub enum TreeSkip {
    /// Neither music root exists under the output tree.
    NoMusicDir,
    /// `ffmpeg` could not be spawned.
    NoFfmpeg,
}

/// Every music clip under `output`, as (path, path relative to `output` with `/` separators).
fn music_clips(output: &Path) -> Vec<(PathBuf, String)> {
    let roots = [
        (output.join("audio/audio/sound_beta_2/music"), false),
        (output.join("audio/anon"), true),
    ];
    let mut out = Vec::new();
    for (root, only_m) in roots {
        if !root.is_dir() {
            continue;
        }
        for e in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            let p = e.path();
            if !e.file_type().is_file() || !p.extension().is_some_and(|x| x == "ogg") {
                continue;
            }
            if only_m
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("m_"))
            {
                continue;
            }
            let rel = p
                .strip_prefix(output)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((p.to_path_buf(), rel));
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// Length plus FNV-1a 64 of the bytes: enough to tell a rewritten-identical `.ogg` from a
/// changed one, with no hash crate.
fn fingerprint(p: &Path) -> std::io::Result<String> {
    let bytes = fs::read(p)?;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in &bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    Ok(format!("{}:{h:016x}", bytes.len()))
}

fn read_manifest(output: &Path) -> BTreeMap<String, String> {
    fs::read(output.join(MANIFEST))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Encode one clip to a `.partial` sibling and rename it into place only on success, so a
/// killed run never leaves a truncated `.m4a` that the next pass would take as current.
fn transcode_clip(ogg: &Path, m4a: &Path) -> Result<(), String> {
    let name = m4a.file_name().unwrap_or_default().to_string_lossy();
    let tmp = m4a.with_file_name(format!(".partial.{name}"));
    let os = std::ffi::OsStr::new;
    let args = [
        os("-i"),
        ogg.as_os_str(),
        os("-map"),
        os("0:a:0"),
        os("-vn"),
        os("-c:a"),
        os("aac"),
        os("-b:a"),
        os(AAC_BITRATE),
        os("-ac"),
        os("2"),
        os("-movflags"),
        os("+faststart"),
        os("-f"),
        os("mp4"),
        tmp.as_os_str(),
    ];
    let result = run_ffmpeg(&args)
        .and_then(|()| fs::rename(&tmp, m4a).map_err(|e| format!("rename into place: {e}")));
    let _ = fs::remove_file(&tmp);
    result
}

/// What one clip's pass did. A kept clip carries its `.ogg` fingerprint for the manifest.
enum ClipOutcome {
    Transcoded(String),
    Skipped(String),
    Failed,
}

/// Skip (and touch) or transcode one clip, printing its row unless it was skipped.
fn pass_clip(ogg: &Path, rel: &str, force: bool, known: &BTreeMap<String, String>) -> ClipOutcome {
    let failed = |what: std::fmt::Arguments| {
        println!("music: {rel:<72} FAILED: {what}");
        ClipOutcome::Failed
    };
    let m4a = ogg.with_extension("m4a");
    let fp = match fingerprint(ogg) {
        Ok(fp) => fp,
        Err(e) => return failed(format_args!("read: {e}")),
    };
    let ogg_mtime = fs::metadata(ogg)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let up_to_date =
        !force && m4a.is_file() && (newer_than(&m4a, ogg_mtime) || known.get(rel) == Some(&fp));
    if up_to_date {
        return match touch(&m4a) {
            Ok(()) => ClipOutcome::Skipped(fp),
            Err(e) => failed(format_args!("touch: {e}")),
        };
    }
    let t0 = Instant::now();
    match transcode_clip(ogg, &m4a) {
        Ok(()) => {
            println!(
                "music: {rel:<72} ogg {:>9} m4a {:>9} {:.1}s",
                file_len(ogg),
                file_len(&m4a),
                t0.elapsed().as_secs_f64()
            );
            ClipOutcome::Transcoded(fp)
        }
        // The old `.m4a`, if any, is left untouched, so the sweep removes it and the frontend
        // falls back to the `.ogg` rather than a stale encode.
        Err(e) => failed(format_args!("{e}")),
    }
}

/// Write `<clip>.m4a` beside every music `.ogg` under `output`, in parallel up to `MAX_JOBS`.
/// Prints a row per transcoded or failed clip (a skip is silent; 863 skip rows would bury the
/// extract's own tail) and a summary line.
pub fn transcode_tree(output: &Path, force: bool) -> Result<TreeReport, TreeSkip> {
    let clips = music_clips(output);
    if clips.is_empty() {
        return Err(TreeSkip::NoMusicDir);
    }
    if !ffmpeg_available() {
        eprintln!(
            "music: warning: ffmpeg is not on PATH; skipping the .m4a transcode ({} clip(s) under {} left untouched)",
            clips.len(),
            output.join("audio").display()
        );
        return Err(TreeSkip::NoFfmpeg);
    }
    let started = Instant::now();
    let known = read_manifest(output);
    let jobs = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .saturating_sub(1)
        .clamp(1, MAX_JOBS);
    let (transcoded, skipped, failed) = (
        AtomicUsize::new(0),
        AtomicUsize::new(0),
        AtomicUsize::new(0),
    );
    let run = || -> BTreeMap<String, String> {
        clips
            .par_iter()
            .filter_map(|(ogg, rel)| {
                let (counter, fp) = match pass_clip(ogg, rel, force, &known) {
                    ClipOutcome::Transcoded(fp) => (&transcoded, Some(fp)),
                    ClipOutcome::Skipped(fp) => (&skipped, Some(fp)),
                    ClipOutcome::Failed => (&failed, None),
                };
                counter.fetch_add(1, Ordering::Relaxed);
                fp.map(|fp| (rel.clone(), fp))
            })
            .collect()
    };
    // A pool that cannot be built (it never has been) falls back to rayon's global one.
    let fresh = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .map_or_else(|_| run(), |pool| pool.install(run));
    let manifest = output.join(MANIFEST);
    match serde_json::to_vec_pretty(&fresh) {
        Ok(bytes) => {
            if let Err(e) = fs::write(&manifest, bytes) {
                eprintln!("music: warning: write {}: {e}", manifest.display());
            }
        }
        Err(e) => eprintln!("music: warning: manifest: {e}"),
    }
    let report = TreeReport {
        transcoded: transcoded.into_inner(),
        skipped: skipped.into_inner(),
        failed: failed.into_inner(),
    };
    println!(
        "music: {} transcoded, {} skipped, {} failed of {} clip(s) in {:.1}s ({jobs} job(s)) -> .m4a beside each .ogg",
        report.transcoded,
        report.skipped,
        report.failed,
        clips.len(),
        started.elapsed().as_secs_f64(),
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn music_clips_takes_the_music_tree_and_only_m_clips_from_anon() {
        let dir = std::env::temp_dir().join(format!("myrtle-m4a-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let music = dir.join("audio/audio/sound_beta_2/music/act1/m_a");
        let anon = dir.join("audio/anon/abc");
        let voice = dir.join("audio/audio/sound_beta_2/voice/char_002");
        for d in [&music, &anon, &voice] {
            fs::create_dir_all(d).unwrap();
        }
        for f in [
            music.join("m_a_intro.ogg"),
            music.join("m_a_loop.ogg"),
            music.join("m_a_loop.m4a"),
            anon.join("m_sys_title_loop.ogg"),
            anon.join("g_ui_back.ogg"),
            voice.join("cn_001.ogg"),
        ] {
            fs::write(f, b"x").unwrap();
        }
        let rels: Vec<String> = music_clips(&dir).into_iter().map(|(_, r)| r).collect();
        assert_eq!(
            rels,
            [
                "audio/anon/abc/m_sys_title_loop.ogg",
                "audio/audio/sound_beta_2/music/act1/m_a/m_a_intro.ogg",
                "audio/audio/sound_beta_2/music/act1/m_a/m_a_loop.ogg",
            ]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fingerprint_tells_content_apart() {
        let dir = std::env::temp_dir().join(format!("myrtle-m4a-fp-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (a, b) = (dir.join("a"), dir.join("b"));
        fs::write(&a, b"OggS one").unwrap();
        fs::write(&b, b"OggS two").unwrap();
        let fa = fingerprint(&a).unwrap();
        assert_ne!(fa, fingerprint(&b).unwrap());
        fs::write(&b, b"OggS one").unwrap();
        assert_eq!(fa, fingerprint(&b).unwrap());
        assert!(fa.starts_with("8:"));
        let _ = fs::remove_dir_all(&dir);
    }
}
