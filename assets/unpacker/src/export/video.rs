//! The game's cutscenes, `raw/video/**/*.usm`, turned into web-playable `.webm` and `.mp4`.
//!
//! A USM is the `CriWare` `Sofdec2` container: a flat run of chunks, each a 32-byte big-endian header
//! followed by a payload and zero padding. The signature names the stream (`CRID` the file and
//! stream directory, `@SFV` video, `@SFA` audio, `@ALP` an alpha plane, `@SBT` subtitles, `@CUE`
//! cue points) and the low two bits of byte 0x0F name the payload kind (0 stream data, 1 a header
//! page, 2 a `#... END` section marker, 3 a metadata page such as the seek index). The demuxer is
//! a port of `WannaCRI`'s `usm/chunk.py` and `usm/usm.py::_process_chunks` (MIT, Donmai-dev,
//! <https://github.com/donmai-me/WannaCRI>): the header fields, the payload bounds
//! `8 + offset .. 8 + size - padding`, and the rule that a stream is the concatenation of its
//! stream-kind payloads in file order.
//!
//! THERE IS NO FRAME REASSEMBLY TO DO. Measured on all 16 EN clips (2026-09-25): every `@SFV`
//! stream chunk holds exactly one VP9 frame, and the payloads already carry the IVF framing, the
//! 32-byte `DKIF` file header in the first chunk and the 12-byte size + timestamp header in front
//! of every frame. `01.usm` has 709 video stream chunks and its IVF header counts 709 frames.
//! `WannaCRI`'s `.ivf` is those payloads concatenated, not a wrapper it writes, and the same holds
//! for the ADX audio, whose `0x8000` header rides in the first `@SFA` payload. ffmpeg's own `.usm`
//! demuxer emits each payload as a packet with the IVF headers still inside, which is why it
//! reports "Invalid frame marker" on files that are perfectly readable.
//!
//! The clips are NOT encrypted. `VideoManager.OnInitSDK` (RVA 0x42015fc) adds the
//! `CriWareInitializer` with `useDecrypter` left false, so Sofdec2 decryption is never armed and
//! the payloads are plain VP9 and ADX. Should a future server arm it, the key is
//! `0x00D47EB533AEF7E5 XOR Convert.ToUInt64(decrypterConfig.key)` inside
//! `CriWareDecrypter.Initialize` (RVA 0x4d95cec), and `WannaCRI`'s `generate_keys` and packet
//! XOR are what to port next. Nothing here decrypts.
//!
//! Chunk kinds the 16 EN clips use: `CRID`, `@SFV` and `@SFA` (the four
//! `mixstory/bg_mainline_*` backdrops have no `@SFA`), every chunk on channel 0 with payload
//! offset 0x18. `@ALP`, `@SBT`, `@CUE` and a second channel never occur, so they are counted and
//! reported rather than written; a clip that grows an alpha plane will say so in the log instead
//! of losing it silently.
//!
//! The transcode shells out to `ffmpeg`: the `.webm` stream-copies the VP9 (a remux, not a
//! re-encode) with Opus 128k audio, the `.mp4` is libx264 crf 20 yuv420p with AAC 160k and
//! `+faststart` for Safari. crf 20 is quality-first and costs bytes: `act49side/ta02` lands at
//! 41,083,136 B at crf 20, 31,021,023 B at 23 and 23,565,582 B at 26, against a 21,004,672 B
//! source and an 18,089,803 B `.webm`. The `.webm` is what browsers take first, so the fallback is
//! where to spend less if delivery bytes start to matter.
//!
//! OUTPUT NAMES ARE LOWERCASE. Story scripts write `video/act38side/PV01.mp4` while the raw file
//! is `pv01.usm`; the backend resolves clips by lowercased relative path.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use walkdir::WalkDir;

/// Bytes in a chunk header: signature, size, then 24 bytes of fields.
pub const CHUNK_HEADER_LEN: usize = 0x20;

/// x264 quality for the `.mp4`. See the module doc for what the neighbouring values cost.
const X264_CRF: &str = "20";

/// What a chunk carries, by its 4-byte signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkKind {
    Info,
    Video,
    Audio,
    Alpha,
    Subtitle,
    Cue,
    Other([u8; 4]),
}

impl ChunkKind {
    #[must_use]
    pub const fn from_signature(sig: [u8; 4]) -> Self {
        match &sig {
            b"CRID" => Self::Info,
            b"@SFV" => Self::Video,
            b"@SFA" => Self::Audio,
            b"@ALP" => Self::Alpha,
            b"@SBT" => Self::Subtitle,
            b"@CUE" => Self::Cue,
            _ => Self::Other(sig),
        }
    }
}

/// The low two bits of header byte 0x0F.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadType {
    Stream,
    Header,
    SectionEnd,
    Metadata,
}

impl PayloadType {
    #[must_use]
    pub const fn from_bits(b: u8) -> Self {
        match b & 0x3 {
            0 => Self::Stream,
            1 => Self::Header,
            2 => Self::SectionEnd,
            _ => Self::Metadata,
        }
    }
}

/// One parsed 32-byte chunk header. Bytes 0x08, 0x0D, 0x0E and 0x18..0x20 are reserved and
/// zero in every EN clip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkHeader {
    pub kind: ChunkKind,
    /// Bytes that follow the signature and this field: the rest of the header, the payload and
    /// the padding.
    pub size: u32,
    /// Where the payload starts, counted from byte 0x08. 0x18 in every EN clip.
    pub payload_offset: u8,
    pub padding: u16,
    pub channel: u8,
    pub payload_type: PayloadType,
    pub frame_time: u32,
    pub frame_rate: u32,
}

const fn be32(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl ChunkHeader {
    /// Parse a header and check that its payload bounds fit inside the chunk.
    pub fn parse(b: &[u8; CHUNK_HEADER_LEN]) -> io::Result<Self> {
        let h = Self {
            kind: ChunkKind::from_signature([b[0], b[1], b[2], b[3]]),
            size: be32(b, 0x04),
            payload_offset: b[0x09],
            padding: u16::from_be_bytes([b[0x0A], b[0x0B]]),
            channel: b[0x0C],
            payload_type: PayloadType::from_bits(b[0x0F]),
            frame_time: be32(b, 0x10),
            frame_rate: be32(b, 0x14),
        };
        // The payload may not start inside the fields just read, and offset + padding may not
        // exceed the chunk. WannaCRI checks neither; a violation here means a misaligned walk.
        if usize::from(h.payload_offset) < CHUNK_HEADER_LEN - 8
            || u64::from(h.payload_offset) + u64::from(h.padding) > u64::from(h.size)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "chunk {:?}: size {} cannot hold payload offset {} + padding {}",
                    h.kind, h.size, h.payload_offset, h.padding
                ),
            ));
        }
        Ok(h)
    }

    /// Payload bytes, excluding header and padding.
    #[must_use]
    pub fn payload_len(&self) -> usize {
        self.size as usize - usize::from(self.payload_offset) - usize::from(self.padding)
    }
}

/// What one demux wrote and what it passed over.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DemuxStats {
    pub chunks: usize,
    /// `@SFV` stream chunks written, one VP9 frame each.
    pub video_frames: usize,
    pub video_bytes: u64,
    pub audio_packets: usize,
    pub audio_bytes: u64,
    /// The first bytes of the audio stream, for sniffing its codec.
    pub audio_head: Vec<u8>,
    /// Stream chunks not written: alpha, subtitle, cue, unknown signatures, and any channel
    /// other than 0. Zero on every EN clip.
    pub ignored_stream_chunks: usize,
}

/// Read exactly `buf.len()` bytes, or none at a clean end of input. `Ok(false)` is EOF.
fn read_or_eof(r: &mut impl Read, buf: &mut [u8]) -> io::Result<bool> {
    let mut got = 0;
    while got < buf.len() {
        match r.read(&mut buf[got..]) {
            Ok(0) if got == 0 => return Ok(false),
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    format!("truncated chunk header ({got} of {} bytes)", buf.len()),
                ));
            }
            Ok(n) => got += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

/// Walk a USM and write channel 0's video stream to `video` and its audio stream to `audio`,
/// each the concatenation of its stream payloads in file order.
pub fn demux(
    reader: impl Read,
    video: &mut impl Write,
    audio: &mut impl Write,
) -> io::Result<DemuxStats> {
    let mut r = BufReader::with_capacity(1 << 20, reader);
    let mut stats = DemuxStats::default();
    let mut head = [0u8; CHUNK_HEADER_LEN];
    let mut body = Vec::new();
    while read_or_eof(&mut r, &mut head[..8])? {
        let size = be32(&head, 4) as usize;
        if size < CHUNK_HEADER_LEN - 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("chunk {} declares size {size}", stats.chunks),
            ));
        }
        body.resize(size, 0);
        r.read_exact(&mut body)?;
        head[8..].copy_from_slice(&body[..CHUNK_HEADER_LEN - 8]);
        let h = ChunkHeader::parse(&head)?;
        if stats.chunks == 0 && h.kind != ChunkKind::Info {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "not a USM: the first chunk is not CRID",
            ));
        }
        stats.chunks += 1;
        if h.payload_type != PayloadType::Stream {
            continue;
        }
        let start = usize::from(h.payload_offset);
        let payload = &body[start..start + h.payload_len()];
        match (h.kind, h.channel) {
            (ChunkKind::Video, 0) => {
                video.write_all(payload)?;
                stats.video_frames += 1;
                stats.video_bytes += payload.len() as u64;
            }
            (ChunkKind::Audio, 0) => {
                if stats.audio_head.len() < 4 {
                    let need = 4 - stats.audio_head.len();
                    stats
                        .audio_head
                        .extend_from_slice(&payload[..need.min(payload.len())]);
                }
                audio.write_all(payload)?;
                stats.audio_packets += 1;
                stats.audio_bytes += payload.len() as u64;
            }
            (ChunkKind::Info, _) => {}
            _ => stats.ignored_stream_chunks += 1,
        }
    }
    Ok(stats)
}

/// The extension ffmpeg needs to pick the audio demuxer: ADX opens with `0x80 0x00`, HCA with
/// `HCA\0` (bit 7 set on each letter when masked).
#[must_use]
pub fn audio_extension(head: &[u8]) -> &'static str {
    match head {
        [0x80, 0x00, ..] => "adx",
        [h, c, a, ..] if h & 0x7F == b'H' && c & 0x7F == b'C' && a & 0x7F == b'A' => "hca",
        _ => "sfa",
    }
}

/// How one clip ended.
#[derive(Debug)]
pub enum ClipStatus {
    Transcoded,
    Skipped,
    Failed(String),
}

/// One row of the per-clip report.
#[derive(Debug)]
pub struct ClipReport {
    /// Lowercased path under `video/`, without extension.
    pub stem: String,
    pub input_bytes: u64,
    pub webm_bytes: u64,
    pub mp4_bytes: u64,
    /// IVF frames demuxed; 0 on a skip, where nothing was demuxed.
    pub frames: usize,
    /// Whether an `@SFA` stream was demuxed; `None` on a skip, where nothing was read.
    pub has_audio: Option<bool>,
    pub wall: Duration,
    pub status: ClipStatus,
}

/// Totals for one pass over a server's `raw/video`.
#[derive(Debug, Default)]
pub struct TreeReport {
    pub transcoded: usize,
    pub skipped: usize,
    pub failed: usize,
}

/// Why a pass did no work at all.
#[derive(Debug)]
pub enum TreeSkip {
    /// `<input>/raw/video` does not exist.
    NoVideoDir,
    /// `ffmpeg` could not be spawned.
    NoFfmpeg,
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn file_len(p: &Path) -> u64 {
    fs::metadata(p).map_or(0, |m| m.len())
}

fn newer_than(p: &Path, than: SystemTime) -> bool {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .is_ok_and(|t| t > than)
}

/// Refresh a skipped output's mtime. `run.mjs` sweeps every file in a written subtree whose
/// mtime predates the extract, so a clip skipped as up to date would otherwise be deleted as an
/// orphan the first time any other clip in `video/` is re-transcoded.
fn touch(p: &Path) -> io::Result<()> {
    File::options()
        .write(true)
        .open(p)?
        .set_modified(SystemTime::now())
}

fn run_ffmpeg(args: &[&std::ffi::OsStr]) -> Result<(), String> {
    let out = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y"])
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("spawn ffmpeg: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        let last = err
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("");
        Err(format!("ffmpeg exited {}: {last}", out.status))
    }
}

/// Demux one clip into `scratch` and write its two outputs. Each output goes to a `.partial`
/// sibling first and is renamed into place only when ffmpeg succeeds, so a killed run never
/// leaves a truncated file that the next run would take as up to date.
fn transcode_clip(
    usm: &Path,
    webm: &Path,
    mp4: &Path,
    scratch: &Path,
) -> Result<DemuxStats, String> {
    let ivf = scratch.join("video.ivf");
    let aud_raw = scratch.join("audio.bin");
    let stats = {
        let src = File::open(usm).map_err(|e| format!("open: {e}"))?;
        let mut v = BufWriter::new(File::create(&ivf).map_err(|e| format!("scratch: {e}"))?);
        let mut a = BufWriter::new(File::create(&aud_raw).map_err(|e| format!("scratch: {e}"))?);
        let stats = demux(src, &mut v, &mut a).map_err(|e| format!("demux: {e}"))?;
        v.flush().map_err(|e| format!("scratch: {e}"))?;
        a.flush().map_err(|e| format!("scratch: {e}"))?;
        stats
    };
    if stats.video_frames == 0 {
        return Err("no @SFV stream on channel 0".to_string());
    }
    let aud = if stats.audio_bytes > 0 {
        let p = scratch.join(format!("audio.{}", audio_extension(&stats.audio_head)));
        fs::rename(&aud_raw, &p).map_err(|e| format!("scratch: {e}"))?;
        Some(p)
    } else {
        None
    };

    if let Some(dir) = webm.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    }
    let partial = |p: &Path| {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        p.with_file_name(format!(".partial.{name}"))
    };
    let (webm_tmp, mp4_tmp) = (partial(webm), partial(mp4));

    let os = |s: &'static str| std::ffi::OsStr::new(s);
    let mut webm_args: Vec<&std::ffi::OsStr> = vec![os("-i"), ivf.as_os_str()];
    let mut mp4_args = webm_args.clone();
    if let Some(a) = &aud {
        for args in [&mut webm_args, &mut mp4_args] {
            args.extend([
                os("-i"),
                a.as_os_str(),
                os("-map"),
                os("0:v:0"),
                os("-map"),
                os("1:a:0"),
            ]);
        }
    } else {
        // The mixstory backdrops are silent loops with no @SFA stream.
        webm_args.extend([os("-map"), os("0:v:0")]);
        mp4_args.extend([os("-map"), os("0:v:0")]);
    }
    webm_args.extend([os("-c:v"), os("copy")]);
    mp4_args.extend([
        os("-c:v"),
        os("libx264"),
        os("-preset"),
        os("medium"),
        os("-crf"),
        os(X264_CRF),
        os("-pix_fmt"),
        os("yuv420p"),
    ]);
    if aud.is_some() {
        webm_args.extend([os("-c:a"), os("libopus"), os("-b:a"), os("128k")]);
        mp4_args.extend([os("-c:a"), os("aac"), os("-b:a"), os("160k")]);
    }
    mp4_args.extend([os("-movflags"), os("+faststart")]);
    webm_args.extend([os("-f"), os("webm"), webm_tmp.as_os_str()]);
    mp4_args.extend([os("-f"), os("mp4"), mp4_tmp.as_os_str()]);

    let result = run_ffmpeg(&webm_args)
        .and_then(|()| run_ffmpeg(&mp4_args))
        .and_then(|()| {
            fs::rename(&webm_tmp, webm)
                .and_then(|()| fs::rename(&mp4_tmp, mp4))
                .map_err(|e| format!("rename into place: {e}"))
        });
    let _ = fs::remove_file(&webm_tmp);
    let _ = fs::remove_file(&mp4_tmp);
    result.map(|()| stats)
}

fn print_row(r: &ClipReport) {
    let status = match &r.status {
        ClipStatus::Transcoded => format!("{:.1}s", r.wall.as_secs_f64()),
        ClipStatus::Skipped => "skip".to_string(),
        ClipStatus::Failed(e) => format!("FAILED: {e}"),
    };
    let frames = if r.frames > 0 {
        r.frames.to_string()
    } else {
        "-".to_string()
    };
    println!(
        "video: {:<28} in {:>11} webm {:>11} mp4 {:>11} frames {:>6} audio {:<3} {status}",
        r.stem,
        r.input_bytes,
        r.webm_bytes,
        r.mp4_bytes,
        frames,
        match r.has_audio {
            Some(true) => "yes",
            Some(false) => "no",
            None => "-",
        },
    );
}

/// Transcode every `<input>/raw/video/**/*.usm` into `<output>/video/<rel>.{webm,mp4}`, one clip
/// at a time (ffmpeg threads itself; the VPS is 3 cores). A clip whose two outputs are both newer
/// than its `.usm` is skipped unless `force`, and its outputs are touched so the orphan sweep
/// keeps them. Prints one row per clip and a summary line.
pub fn transcode_tree(
    input: &Path,
    output: &Path,
    force: bool,
) -> Result<(TreeReport, Vec<ClipReport>), TreeSkip> {
    let raw = input.join("raw").join("video");
    if !raw.is_dir() {
        return Err(TreeSkip::NoVideoDir);
    }
    let mut clips: Vec<PathBuf> = WalkDir::new(&raw)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("usm")))
        .collect();
    clips.sort();
    if !ffmpeg_available() {
        eprintln!(
            "video: warning: ffmpeg is not on PATH; skipping the video transcode ({} clip(s) under {} left untouched)",
            clips.len(),
            raw.display()
        );
        return Err(TreeSkip::NoFfmpeg);
    }

    let started = Instant::now();
    let scratch = std::env::temp_dir().join(format!("myrtle-usm-{}", std::process::id()));
    let mut report = TreeReport::default();
    let mut rows = Vec::with_capacity(clips.len());
    for usm in &clips {
        let t0 = Instant::now();
        let rel = usm.strip_prefix(&raw).unwrap_or(usm).with_extension("");
        let stem = rel.to_string_lossy().replace('\\', "/").to_lowercase();
        let webm = output.join("video").join(format!("{stem}.webm"));
        let mp4 = output.join("video").join(format!("{stem}.mp4"));
        let input_bytes = file_len(usm);
        let usm_mtime = fs::metadata(usm)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let up_to_date = !force && newer_than(&webm, usm_mtime) && newer_than(&mp4, usm_mtime);
        let (status, frames, has_audio) = if up_to_date {
            match touch(&webm).and_then(|()| touch(&mp4)) {
                Ok(()) => (ClipStatus::Skipped, 0, None),
                Err(e) => (ClipStatus::Failed(format!("touch: {e}")), 0, None),
            }
        } else {
            let res = fs::create_dir_all(&scratch)
                .map_err(|e| format!("scratch {}: {e}", scratch.display()))
                .and_then(|()| transcode_clip(usm, &webm, &mp4, &scratch));
            match res {
                Ok(s) => (
                    ClipStatus::Transcoded,
                    s.video_frames,
                    Some(s.audio_bytes > 0),
                ),
                Err(e) => (ClipStatus::Failed(e), 0, None),
            }
        };
        match status {
            ClipStatus::Transcoded => report.transcoded += 1,
            ClipStatus::Skipped => report.skipped += 1,
            ClipStatus::Failed(_) => report.failed += 1,
        }
        let row = ClipReport {
            stem,
            input_bytes,
            webm_bytes: file_len(&webm),
            mp4_bytes: file_len(&mp4),
            frames,
            has_audio,
            wall: t0.elapsed(),
            status,
        };
        print_row(&row);
        rows.push(row);
    }
    let _ = fs::remove_dir_all(&scratch);
    println!(
        "video: {} transcoded, {} skipped, {} failed of {} clip(s) in {:.1}s -> {}",
        report.transcoded,
        report.skipped,
        report.failed,
        clips.len(),
        started.elapsed().as_secs_f64(),
        output.join("video").display()
    );
    Ok((report, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One chunk as the game's muxer lays it out: offset 0x18, `padding` zero bytes after.
    fn chunk(sig: &[u8; 4], channel: u8, ptype: u8, payload: &[u8], padding: u16) -> Vec<u8> {
        let size = 0x18 + payload.len() as u32 + u32::from(padding);
        let mut c = Vec::new();
        c.extend_from_slice(sig);
        c.extend_from_slice(&size.to_be_bytes());
        c.push(0);
        c.push(0x18);
        c.extend_from_slice(&padding.to_be_bytes());
        c.push(channel);
        c.extend_from_slice(&[0, 0, ptype]);
        c.extend_from_slice(&1001u32.to_be_bytes());
        c.extend_from_slice(&2997u32.to_be_bytes());
        c.extend_from_slice(&[0; 8]);
        c.extend_from_slice(payload);
        c.extend(std::iter::repeat_n(0, usize::from(padding)));
        c
    }

    #[test]
    fn header_parse_reads_every_field() {
        let c = chunk(b"@SFV", 3, 0x02, b"#HEADER END", 5);
        let h = ChunkHeader::parse(c[..CHUNK_HEADER_LEN].try_into().unwrap()).unwrap();
        assert_eq!(h.kind, ChunkKind::Video);
        assert_eq!(h.size, 0x18 + 11 + 5);
        assert_eq!(h.payload_offset, 0x18);
        assert_eq!(h.padding, 5);
        assert_eq!(h.channel, 3);
        assert_eq!(h.payload_type, PayloadType::SectionEnd);
        assert_eq!(h.frame_time, 1001);
        assert_eq!(h.frame_rate, 2997);
        assert_eq!(h.payload_len(), 11);
        assert_eq!(c.len(), 8 + h.size as usize);
    }

    #[test]
    fn header_parse_rejects_padding_past_the_chunk() {
        let mut c = chunk(b"@SFA", 0, 0, b"abc", 0);
        c[0x0A..0x0C].copy_from_slice(&100u16.to_be_bytes());
        assert!(ChunkHeader::parse(c[..CHUNK_HEADER_LEN].try_into().unwrap()).is_err());
    }

    #[test]
    fn signatures_and_payload_bits() {
        assert_eq!(ChunkKind::from_signature(*b"CRID"), ChunkKind::Info);
        assert_eq!(ChunkKind::from_signature(*b"@ALP"), ChunkKind::Alpha);
        assert_eq!(ChunkKind::from_signature(*b"@CUE"), ChunkKind::Cue);
        assert_eq!(
            ChunkKind::from_signature(*b"XXXX"),
            ChunkKind::Other(*b"XXXX")
        );
        // Only the low two bits carry the type.
        assert_eq!(PayloadType::from_bits(0xFC), PayloadType::Stream);
        assert_eq!(PayloadType::from_bits(0x03), PayloadType::Metadata);
    }

    #[test]
    fn demux_concatenates_stream_payloads_in_file_order() {
        let mut usm = Vec::new();
        usm.extend(chunk(b"CRID", 0, 1, b"@UTF directory page", 0x6A));
        usm.extend(chunk(b"@SFV", 0, 1, b"video header page", 0x18));
        usm.extend(chunk(b"@SFA", 0, 1, b"audio header page", 0x08));
        usm.extend(chunk(
            b"@SFV",
            0,
            2,
            b"#HEADER END     ===============\0",
            0,
        ));
        usm.extend(chunk(b"@SFV", 0, 3, b"seek index", 4));
        usm.extend(chunk(b"@SFV", 0, 0, b"DKIF+frame0", 1));
        usm.extend(chunk(b"@SFA", 0, 0, &[0x80, 0x00, 0x01], 0));
        usm.extend(chunk(b"@SFV", 0, 0, b"frame1", 7));
        usm.extend(chunk(b"@ALP", 0, 0, b"alpha plane", 0));
        usm.extend(chunk(b"@SFV", 1, 0, b"second channel", 0));
        usm.extend(chunk(b"@SFA", 0, 0, b"tail", 3));
        usm.extend(chunk(b"@SFV", 0, 0, b"frame2", 0));
        usm.extend(chunk(
            b"@SFV",
            0,
            2,
            b"#CONTENTS END   ===============\0",
            0,
        ));

        let (mut v, mut a) = (Vec::new(), Vec::new());
        let s = demux(usm.as_slice(), &mut v, &mut a).unwrap();
        assert_eq!(v, b"DKIF+frame0frame1frame2");
        assert_eq!(a, b"\x80\x00\x01tail");
        assert_eq!(s.chunks, 13);
        assert_eq!(s.video_frames, 3);
        assert_eq!(s.video_bytes, 23);
        assert_eq!(s.audio_packets, 2);
        assert_eq!(s.audio_bytes, 7);
        assert_eq!(s.ignored_stream_chunks, 2);
        assert_eq!(audio_extension(&s.audio_head), "adx");
    }

    #[test]
    fn demux_rejects_a_non_usm_and_a_truncated_chunk() {
        let (mut v, mut a) = (Vec::new(), Vec::new());
        let not_usm = chunk(b"@SFV", 0, 0, b"x", 0);
        assert!(demux(not_usm.as_slice(), &mut v, &mut a).is_err());

        let mut cut = chunk(b"CRID", 0, 1, b"page", 0);
        cut.extend(chunk(b"@SFV", 0, 0, b"frame", 0));
        cut.truncate(cut.len() - 2);
        assert!(demux(cut.as_slice(), &mut v, &mut a).is_err());

        let mut short_head = chunk(b"CRID", 0, 1, b"page", 0);
        short_head.extend_from_slice(b"@SF");
        assert!(demux(short_head.as_slice(), &mut v, &mut a).is_err());
    }

    #[test]
    fn audio_codec_sniff() {
        assert_eq!(audio_extension(&[0x80, 0x00, 0x01, 0x1C]), "adx");
        assert_eq!(audio_extension(b"HCA\0"), "hca");
        assert_eq!(
            audio_extension(&[b'H' | 0x80, b'C' | 0x80, b'A' | 0x80, 0]),
            "hca"
        );
        assert_eq!(audio_extension(b"RIFF"), "sfa");
    }

    /// The real `01.usm`, against the sizes `WannaCRI` 0.3.3 demuxes from it (byte-identical
    /// streams, checked with `cmp` on 2026-09-25).
    #[test]
    #[ignore = "needs assets/ArkAssets/en/raw/video/01.usm"]
    fn demux_real_01_usm() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ArkAssets/en/raw/video/01.usm");
        let (mut v, mut a) = (Vec::new(), Vec::new());
        let s = demux(File::open(p).unwrap(), &mut v, &mut a).unwrap();
        assert_eq!(s.video_frames, 709);
        assert_eq!(v.len(), 16_579_934);
        assert_eq!(a.len(), 1_598_130);
        assert_eq!(&v[..4], b"DKIF");
        // The IVF header's own frame count agrees with the chunk count.
        assert_eq!(u32::from_le_bytes([v[24], v[25], v[26], v[27]]), 709);
        assert_eq!(s.ignored_stream_chunks, 0);
        assert_eq!(audio_extension(&s.audio_head), "adx");
    }
}
