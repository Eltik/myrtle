//! Client-binary extractor for IL2CPP recovery.
//!
//! Pulls the two artifacts an IL2CPP dump needs — the native library
//! (`libil2cpp.so` on Android, the Mach-O executable on iOS) and
//! `global-metadata.dat` — out of a user-provided app package, then reports
//! whether a *static* dump is likely to succeed.
//!
//! Arknights ships anti-tamper that encrypts `global-metadata.dat` at runtime,
//! so a plaintext file inside the package is the happy path; a mismatched magic
//! means the on-disk copy is scrambled and only a Frida memory dump will work.
//!
//! Every container format handled here (APK / XAPK / APKM / OBB / IPA) is a ZIP
//! archive. XAPK/APKM additionally nest an inner base APK (and OBBs); this module
//! peels exactly one level of nesting.

use std::fs;
use std::io::{Cursor, Read, Seek};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use zip::ZipArchive;

/// `global-metadata.dat` magic, little-endian u32 at offset 0.
pub const GLOBAL_METADATA_MAGIC: u32 = 0xFAB1_1BAF;
/// IL2CPP metadata version this toolchain targets (Unity 2021.3 client).
pub const EXPECTED_METADATA_VERSION: u32 = 27;

// --- Mach-O constants ---------------------------------------------------------
const MH_MAGIC_64: u32 = 0xfeed_facf;
const MH_CIGAM_64: u32 = 0xcffa_edfe;
const MH_MAGIC_32: u32 = 0xfeed_face;
const MH_CIGAM_32: u32 = 0xcefa_edfe;
const FAT_MAGIC: u32 = 0xcafe_babe; // fat header is always big-endian on disk
const FAT_CIGAM: u32 = 0xbeba_feca;
const LC_ENCRYPTION_INFO: u32 = 0x21;
const LC_ENCRYPTION_INFO_64: u32 = 0x2C;
const CPU_TYPE_ARM64: u32 = 0x0100_000c;

/// Classification of a `global-metadata.dat` from its 8-byte header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataStatus {
    /// Magic matches and version is the expected v27 — safe to dump statically.
    Ok { version: u32 },
    /// Magic matches but a different version — dumper may still work, verify.
    WrongVersion { version: u32 },
    /// Magic does not match — the on-disk file is (almost certainly) encrypted.
    LikelyEncrypted { magic: u32 },
    /// Fewer than 8 bytes — not a metadata file.
    TooShort,
}

impl MetadataStatus {
    /// Human-readable one-liner for the report.
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Ok { version } => {
                format!("OK, plaintext metadata (v{version})")
            }
            Self::WrongVersion { version } => format!(
                "magic OK but version is v{version} (expected v{EXPECTED_METADATA_VERSION}) — \
                 check your Il2CppDumper build supports it"
            ),
            Self::LikelyEncrypted { magic } => format!(
                "LIKELY ENCRYPTED (magic {magic:#010x} != {GLOBAL_METADATA_MAGIC:#010x}) — a \
                 static dump will fail; use the Frida memory-dump runbook"
            ),
            Self::TooShort => "file too short to be global-metadata.dat".to_string(),
        }
    }
}

/// Classify a `global-metadata.dat` from its raw bytes (reads only the first 8).
#[must_use]
pub fn classify_metadata(bytes: &[u8]) -> MetadataStatus {
    if bytes.len() < 8 {
        return MetadataStatus::TooShort;
    }
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if magic != GLOBAL_METADATA_MAGIC {
        return MetadataStatus::LikelyEncrypted { magic };
    }
    if version == EXPECTED_METADATA_VERSION {
        MetadataStatus::Ok { version }
    } else {
        MetadataStatus::WrongVersion { version }
    }
}

/// FairPlay-encryption status of an iOS Mach-O executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MachOStatus {
    /// `LC_ENCRYPTION_INFO(_64)` present with `cryptid != 0`.
    FairPlayEncrypted { cryptid: u32 },
    /// Encryption command present with `cryptid == 0` (already decrypted).
    NotEncrypted,
    /// No encryption load command at all (never a FairPlay binary).
    NoEncryptionCommand,
}

impl MachOStatus {
    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::FairPlayEncrypted { cryptid } => format!(
                "FairPlay-encrypted (cryptid={cryptid}) — needs a decrypted dump from a \
                 jailbroken device (e.g. frida-ios-dump / flexdecrypt)"
            ),
            Self::NotEncrypted => {
                "not encrypted (cryptid=0) — Mach-O is already decrypted".to_string()
            }
            Self::NoEncryptionCommand => {
                "no LC_ENCRYPTION_INFO command — treat as unencrypted".to_string()
            }
        }
    }
}

fn read_u32(data: &[u8], off: usize, big_endian: bool) -> Result<u32> {
    let b = data
        .get(off..off + 4)
        .ok_or_else(|| anyhow!("Mach-O truncated: wanted 4 bytes at {off:#x}"))?;
    let arr = [b[0], b[1], b[2], b[3]];
    Ok(if big_endian {
        u32::from_be_bytes(arr)
    } else {
        u32::from_le_bytes(arr)
    })
}

/// Analyze a Mach-O (thin or fat/universal) for FairPlay encryption.
///
/// Fat binaries are resolved to their arm64 slice first. Endianness is derived
/// from the header magic.
///
/// # Errors
/// Returns an error if the bytes are not a recognizable Mach-O, are truncated,
/// or a fat binary contains no arm64 slice.
pub fn analyze_macho(data: &[u8]) -> Result<MachOStatus> {
    // Fat header is stored big-endian regardless of slice endianness.
    let leading = read_u32(data, 0, true)?;
    if leading == FAT_MAGIC || leading == FAT_CIGAM {
        let nfat = read_u32(data, 4, true)?;
        for i in 0..nfat as usize {
            // fat_arch: cputype, cpusubtype, offset, size, align (5 x u32)
            let entry = 8 + i * 20;
            let cputype = read_u32(data, entry, true)?;
            let offset = read_u32(data, entry + 8, true)? as usize;
            let size = read_u32(data, entry + 12, true)? as usize;
            if cputype == CPU_TYPE_ARM64 {
                let slice = data
                    .get(offset..offset + size)
                    .ok_or_else(|| anyhow!("fat arm64 slice out of bounds"))?;
                return analyze_thin_macho(slice);
            }
        }
        bail!("fat/universal binary has no arm64 slice");
    }
    analyze_thin_macho(data)
}

fn analyze_thin_macho(data: &[u8]) -> Result<MachOStatus> {
    let raw_magic = read_u32(data, 0, false)?;
    let (is64, big_endian) = match raw_magic {
        MH_MAGIC_64 => (true, false),
        MH_CIGAM_64 => (true, true),
        MH_MAGIC_32 => (false, false),
        MH_CIGAM_32 => (false, true),
        other => bail!("not a Mach-O (bad magic {other:#010x})"),
    };

    // mach_header(_64): magic, cputype, cpusubtype, filetype, ncmds,
    // sizeofcmds, flags[, reserved]. ncmds is field index 4 → byte offset 16.
    let ncmds = read_u32(data, 16, big_endian)?;
    let header_size = if is64 { 32 } else { 28 };

    let mut off = header_size;
    for _ in 0..ncmds {
        let cmd = read_u32(data, off, big_endian)?;
        let cmdsize = read_u32(data, off + 4, big_endian)? as usize;
        if cmd == LC_ENCRYPTION_INFO || cmd == LC_ENCRYPTION_INFO_64 {
            // encryption_info_command: cmd, cmdsize, cryptoff, cryptsize, cryptid
            let cryptid = read_u32(data, off + 16, big_endian)?;
            return Ok(if cryptid != 0 {
                MachOStatus::FairPlayEncrypted { cryptid }
            } else {
                MachOStatus::NotEncrypted
            });
        }
        if cmdsize == 0 {
            break; // malformed; avoid infinite loop
        }
        off += cmdsize;
    }
    Ok(MachOStatus::NoEncryptionCommand)
}

/// A single artifact written to the output directory.
#[derive(Debug)]
pub struct Extracted {
    /// Path inside the source container the bytes came from.
    pub source_path: String,
    /// Where it was written on disk.
    pub written_to: PathBuf,
    /// Size in bytes.
    pub size: usize,
}

/// Full result of a `client-extract` run.
#[derive(Debug, Default)]
pub struct ExtractReport {
    pub native_lib: Option<Extracted>,
    pub metadata: Option<Extracted>,
    pub metadata_status: Option<MetadataStatus>,
    pub macho_status: Option<MachOStatus>,
    /// Non-fatal notes (e.g. "metadata not found in inner APK, searched OBBs").
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Platform {
    /// Plain Android APK or standalone OBB.
    AndroidZip,
    /// XAPK / APKM — outer zip nesting an inner base APK (+ OBBs).
    AndroidBundle,
    /// iOS IPA.
    Ios,
}

fn detect_platform(path: &Path) -> Result<Platform> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match ext.as_str() {
        "apk" | "obb" => Ok(Platform::AndroidZip),
        "xapk" | "apkm" => Ok(Platform::AndroidBundle),
        "ipa" => Ok(Platform::Ios),
        other => bail!("unrecognized package extension {other:?}; expected apk/obb/xapk/apkm/ipa"),
    }
}

/// A zip archive read fully into memory: (entry name, entry bytes) pairs.
type ZipEntries = Vec<(String, Vec<u8>)>;

/// Read a zip archive fully into memory as (name, bytes) pairs.
fn read_all_entries<R: Read + Seek>(mut ar: ZipArchive<R>) -> Result<ZipEntries> {
    let mut out = Vec::new();
    for i in 0..ar.len() {
        let mut f = ar.by_index(i)?;
        if !f.is_file() {
            continue;
        }
        let name = f.name().to_string();
        let mut buf = Vec::with_capacity(usize::try_from(f.size()).unwrap_or(0));
        f.read_to_end(&mut buf)?;
        out.push((name, buf));
    }
    Ok(out)
}

/// Normalize a zip entry name to forward slashes for suffix matching.
fn norm(name: &str) -> String {
    name.replace('\\', "/")
}

fn is_native_lib(name: &str) -> bool {
    let n = norm(name);
    n.ends_with("arm64-v8a/libil2cpp.so")
}

fn is_metadata(name: &str) -> bool {
    let n = norm(name);
    n.ends_with("global-metadata.dat")
}

/// Entrypoint: extract native lib + metadata from `input` into `output_dir`.
///
/// # Errors
/// Returns an error if the package can't be opened/parsed or the output
/// directory can't be written. Missing artifacts are reported (not errors).
pub fn run(input: &Path, output_dir: &Path) -> Result<ExtractReport> {
    let platform = detect_platform(input)?;
    fs::create_dir_all(output_dir)
        .with_context(|| format!("creating output dir {}", output_dir.display()))?;

    match platform {
        Platform::AndroidZip => extract_android(input, output_dir, /*nested=*/ false),
        Platform::AndroidBundle => extract_android(input, output_dir, /*nested=*/ true),
        Platform::Ios => extract_ios(input, output_dir),
    }
}

fn open_zip(path: &Path) -> Result<ZipEntries> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let ar = ZipArchive::new(Cursor::new(bytes))
        .with_context(|| format!("opening {} as zip", path.display()))?;
    read_all_entries(ar)
}

fn extract_android(input: &Path, output_dir: &Path, nested: bool) -> Result<ExtractReport> {
    let mut report = ExtractReport::default();
    let outer = open_zip(input)?;

    // Build the set of archives to search. For a bundle (XAPK/APKM), the useful
    // payload lives inside a nested base APK; OBBs (also nested zips) may hold
    // the metadata. We peel exactly one level.
    let mut sources: Vec<(String, ZipEntries)> = Vec::new();

    if nested {
        // Inner base APK: prefer one literally named like a base split.
        let inner_apks: Vec<&(String, Vec<u8>)> = outer
            .iter()
            .filter(|(n, _)| norm(n).to_ascii_lowercase().ends_with(".apk"))
            .collect();
        if inner_apks.is_empty() {
            bail!("bundle has no inner .apk (looked for *.apk inside the outer zip)");
        }
        let base = pick_base_apk(&inner_apks);
        report.notes.push(format!("using inner APK: {}", base.0));
        let inner_ar = ZipArchive::new(Cursor::new(base.1.clone()))
            .with_context(|| format!("opening inner apk {}", base.0))?;
        sources.push((base.0.clone(), read_all_entries(inner_ar)?));

        // Nested OBBs may carry the metadata under assets/bin/Data.
        for (n, b) in &outer {
            if norm(n).to_ascii_lowercase().ends_with(".obb")
                && let Ok(ar) = ZipArchive::new(Cursor::new(b.clone()))
                && let Ok(entries) = read_all_entries(ar)
            {
                sources.push((n.clone(), entries));
            }
        }
    } else {
        sources.push((input.display().to_string(), outer));
    }

    // Native lib: only ever in the APK.
    for (container, entries) in &sources {
        if report.native_lib.is_some() {
            break;
        }
        if let Some((name, bytes)) = entries.iter().find(|(n, _)| is_native_lib(n)) {
            report.native_lib = Some(write_artifact(
                output_dir,
                "libil2cpp.so",
                &format!("{container}!{name}"),
                bytes,
            )?);
        }
    }

    // Metadata: APK first, then any OBBs.
    for (container, entries) in &sources {
        if report.metadata.is_some() {
            break;
        }
        if let Some((name, bytes)) = entries.iter().find(|(n, _)| is_metadata(n)) {
            report.metadata_status = Some(classify_metadata(bytes));
            report.metadata = Some(write_artifact(
                output_dir,
                "global-metadata.dat",
                &format!("{container}!{name}"),
                bytes,
            )?);
        }
    }

    if report.native_lib.is_none() {
        report
            .notes
            .push("libil2cpp.so (arm64-v8a) not found".to_string());
    }
    if report.metadata.is_none() {
        report
            .notes
            .push("global-metadata.dat not found in APK or OBBs".to_string());
    }

    Ok(report)
}

/// Choose the base APK from a bundle's inner APKs: prefer `base.apk`, else the
/// one without a `config`/`split` name, else the largest (most complete).
fn pick_base_apk<'a>(apks: &[&'a (String, Vec<u8>)]) -> &'a (String, Vec<u8>) {
    let base_name = |n: &str| norm(n).rsplit('/').next().unwrap_or(n).to_ascii_lowercase();
    if let Some(b) = apks.iter().find(|(n, _)| base_name(n) == "base.apk") {
        return b;
    }
    if let Some(b) = apks
        .iter()
        .find(|(n, _)| !base_name(n).contains("config") && !base_name(n).contains("split"))
    {
        return b;
    }
    apks.iter().max_by_key(|(_, b)| b.len()).unwrap()
}

fn extract_ios(input: &Path, output_dir: &Path) -> Result<ExtractReport> {
    let mut report = ExtractReport::default();
    let entries = open_zip(input)?;

    // Metadata: Payload/*.app/**/global-metadata.dat
    if let Some((name, bytes)) = entries
        .iter()
        .find(|(n, _)| norm(n).starts_with("Payload/") && is_metadata(n))
    {
        report.metadata_status = Some(classify_metadata(bytes));
        report.metadata = Some(write_artifact(
            output_dir,
            "global-metadata.dat",
            name,
            bytes,
        )?);
    } else {
        report
            .notes
            .push("global-metadata.dat not found under Payload/*.app".to_string());
    }

    // Executable: the Mach-O sitting directly in Payload/<Name>.app/. We locate
    // the .app root, then pick the top-level file whose bytes are a Mach-O.
    // (Avoids parsing the binary Info.plist for CFBundleExecutable.)
    if let Some(app_root) = ios_app_root(&entries) {
        let depth = app_root.matches('/').count();
        let exe = entries.iter().find(|(n, b)| {
            let nn = norm(n);
            nn.starts_with(&app_root)
                && nn.matches('/').count() == depth // directly under .app/
                && looks_like_macho(b)
        });
        if let Some((name, bytes)) = exe {
            report.macho_status = Some(analyze_macho(bytes)?);
            report.native_lib = Some(write_artifact(
                output_dir,
                Path::new(&norm(name))
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("executable"),
                name,
                bytes,
            )?);
        } else {
            report.notes.push(format!(
                "no Mach-O executable found directly under {app_root}"
            ));
        }
    } else {
        report
            .notes
            .push("no Payload/*.app directory found in IPA".to_string());
    }

    Ok(report)
}

/// Return the `Payload/<Name>.app/` prefix (with trailing slash) if present.
fn ios_app_root(entries: &[(String, Vec<u8>)]) -> Option<String> {
    for (n, _) in entries {
        let nn = norm(n);
        if let Some(idx) = nn.find(".app/") {
            // Keep everything up to and including "<Name>.app/".
            let end = idx + ".app/".len();
            let prefix = &nn[..end];
            if prefix.starts_with("Payload/") {
                return Some(prefix.to_string());
            }
        }
    }
    None
}

fn looks_like_macho(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let le = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let be = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    matches!(le, MH_MAGIC_64 | MH_CIGAM_64 | MH_MAGIC_32 | MH_CIGAM_32)
        || matches!(be, FAT_MAGIC | FAT_CIGAM)
}

fn write_artifact(
    output_dir: &Path,
    out_name: &str,
    source_path: &str,
    bytes: &[u8],
) -> Result<Extracted> {
    let dest = output_dir.join(out_name);
    fs::write(&dest, bytes).with_context(|| format!("writing {}", dest.display()))?;
    Ok(Extracted {
        source_path: source_path.to_string(),
        written_to: dest,
        size: bytes.len(),
    })
}

/// Print a human-readable summary of an [`ExtractReport`] to stdout.
pub fn print_report(report: &ExtractReport) {
    println!("\n=== client-extract report ===");
    match &report.native_lib {
        Some(e) => println!(
            "native binary : {} ({} bytes)\n                from {}",
            e.written_to.display(),
            e.size,
            e.source_path
        ),
        None => println!("native binary : NOT FOUND"),
    }
    if let Some(status) = &report.macho_status {
        println!("  Mach-O      : {}", status.summary());
    }
    match &report.metadata {
        Some(e) => println!(
            "metadata      : {} ({} bytes)\n                from {}",
            e.written_to.display(),
            e.size,
            e.source_path
        ),
        None => println!("metadata      : NOT FOUND"),
    }
    if let Some(status) = &report.metadata_status {
        println!("  status      : {}", status.summary());
    }
    for note in &report.notes {
        println!("note          : {note}");
    }

    // Next-step guidance keyed off what we found.
    println!("\nnext steps:");
    match &report.metadata_status {
        Some(MetadataStatus::Ok { .. }) => {
            println!(
                "  - metadata looks plaintext; run Il2CppDumper against libil2cpp.so + \
                 global-metadata.dat, then feed dump.cs to `il2cpp-locate`."
            );
        }
        Some(MetadataStatus::WrongVersion { .. }) => {
            println!(
                "  - version differs from v27; try Il2CppDumper anyway, but confirm the \
                 metadata reader supports this version."
            );
        }
        Some(MetadataStatus::LikelyEncrypted { .. }) | Some(MetadataStatus::TooShort) | None => {
            println!(
                "  - static metadata is unusable; use the Frida memory-dump runbook \
                 (dump libil2cpp.so + global-metadata.dat from a running process on a \
                 rooted/jailbroken device), then run Il2CppDumper on the dumped pair."
            );
        }
    }
    if matches!(
        report.macho_status,
        Some(MachOStatus::FairPlayEncrypted { .. })
    ) {
        println!(
            "  - iOS binary is FairPlay-encrypted; obtain a decrypted copy \
             (frida-ios-dump) before dumping."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    fn metadata_bytes(magic: u32, version: u32) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&magic.to_le_bytes());
        v.extend_from_slice(&version.to_le_bytes());
        v.extend_from_slice(&[0u8; 64]); // padding
        v
    }

    #[test]
    fn classifies_plaintext_v27() {
        let bytes = metadata_bytes(GLOBAL_METADATA_MAGIC, 27);
        assert_eq!(
            classify_metadata(&bytes),
            MetadataStatus::Ok { version: 27 }
        );
    }

    #[test]
    fn classifies_wrong_magic_as_encrypted() {
        let bytes = metadata_bytes(0xDEAD_BEEF, 27);
        assert_eq!(
            classify_metadata(&bytes),
            MetadataStatus::LikelyEncrypted { magic: 0xDEAD_BEEF }
        );
    }

    #[test]
    fn classifies_wrong_version() {
        let bytes = metadata_bytes(GLOBAL_METADATA_MAGIC, 24);
        assert_eq!(
            classify_metadata(&bytes),
            MetadataStatus::WrongVersion { version: 24 }
        );
    }

    /// Build a minimal 64-bit arm64 Mach-O with a single
    /// LC_ENCRYPTION_INFO_64 command and the given cryptid.
    fn minimal_macho_arm64(cryptid: u32) -> Vec<u8> {
        let mut v = Vec::new();
        // mach_header_64
        v.extend_from_slice(&MH_MAGIC_64.to_le_bytes()); // magic
        v.extend_from_slice(&CPU_TYPE_ARM64.to_le_bytes()); // cputype
        v.extend_from_slice(&0u32.to_le_bytes()); // cpusubtype
        v.extend_from_slice(&2u32.to_le_bytes()); // filetype = MH_EXECUTE
        v.extend_from_slice(&1u32.to_le_bytes()); // ncmds
        v.extend_from_slice(&24u32.to_le_bytes()); // sizeofcmds
        v.extend_from_slice(&0u32.to_le_bytes()); // flags
        v.extend_from_slice(&0u32.to_le_bytes()); // reserved (64-bit)
        // encryption_info_command_64: cmd, cmdsize, cryptoff, cryptsize, cryptid, pad
        v.extend_from_slice(&LC_ENCRYPTION_INFO_64.to_le_bytes());
        v.extend_from_slice(&24u32.to_le_bytes()); // cmdsize
        v.extend_from_slice(&0x4000u32.to_le_bytes()); // cryptoff
        v.extend_from_slice(&0x1000u32.to_le_bytes()); // cryptsize
        v.extend_from_slice(&cryptid.to_le_bytes()); // cryptid
        v.extend_from_slice(&0u32.to_le_bytes()); // pad
        v
    }

    #[test]
    fn detects_fairplay_encryption() {
        let macho = minimal_macho_arm64(1);
        assert_eq!(
            analyze_macho(&macho).unwrap(),
            MachOStatus::FairPlayEncrypted { cryptid: 1 }
        );
    }

    #[test]
    fn detects_decrypted_binary() {
        let macho = minimal_macho_arm64(0);
        assert_eq!(analyze_macho(&macho).unwrap(), MachOStatus::NotEncrypted);
    }

    /// Wrap a thin arm64 Mach-O inside a fat/universal container.
    fn fat_wrap(thin: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&FAT_MAGIC.to_be_bytes()); // magic (big-endian)
        v.extend_from_slice(&1u32.to_be_bytes()); // nfat_arch
        // fat_arch
        v.extend_from_slice(&CPU_TYPE_ARM64.to_be_bytes()); // cputype
        v.extend_from_slice(&0u32.to_be_bytes()); // cpusubtype
        let offset: u32 = 8 + 20; // header + one fat_arch
        v.extend_from_slice(&offset.to_be_bytes()); // offset
        v.extend_from_slice(&(thin.len() as u32).to_be_bytes()); // size
        v.extend_from_slice(&0u32.to_be_bytes()); // align
        // pad to offset
        while v.len() < offset as usize {
            v.push(0);
        }
        v.extend_from_slice(thin);
        v
    }

    #[test]
    fn detects_fairplay_in_fat_binary() {
        let fat = fat_wrap(&minimal_macho_arm64(1));
        assert_eq!(
            analyze_macho(&fat).unwrap(),
            MachOStatus::FairPlayEncrypted { cryptid: 1 }
        );
    }

    fn make_apk_zip(metadata: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            let opts = SimpleFileOptions::default();
            w.start_file("lib/arm64-v8a/libil2cpp.so", opts).unwrap();
            w.write_all(b"\x7fELF fake native lib").unwrap();
            w.start_file("assets/bin/Data/Managed/Metadata/global-metadata.dat", opts)
                .unwrap();
            w.write_all(metadata).unwrap();
            w.finish().unwrap();
        }
        buf
    }

    #[test]
    fn extracts_apk_and_classifies_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let apk_path = dir.path().join("app.apk");
        fs::write(
            &apk_path,
            make_apk_zip(&metadata_bytes(GLOBAL_METADATA_MAGIC, 27)),
        )
        .unwrap();

        let out = dir.path().join("out");
        let report = run(&apk_path, &out).unwrap();

        assert!(
            report.native_lib.is_some(),
            "native lib should be extracted"
        );
        assert!(report.metadata.is_some(), "metadata should be extracted");
        assert_eq!(
            report.metadata_status,
            Some(MetadataStatus::Ok { version: 27 })
        );
        assert!(out.join("libil2cpp.so").exists());
        assert!(out.join("global-metadata.dat").exists());
    }

    #[test]
    fn extracts_apk_and_flags_encrypted() {
        let dir = tempfile::tempdir().unwrap();
        let apk_path = dir.path().join("app.apk");
        fs::write(&apk_path, make_apk_zip(&metadata_bytes(0x1234_5678, 27))).unwrap();

        let out = dir.path().join("out");
        let report = run(&apk_path, &out).unwrap();
        assert_eq!(
            report.metadata_status,
            Some(MetadataStatus::LikelyEncrypted { magic: 0x1234_5678 })
        );
    }

    #[test]
    fn extracts_nested_xapk() {
        // outer zip -> base.apk -> lib + metadata
        let inner = make_apk_zip(&metadata_bytes(GLOBAL_METADATA_MAGIC, 27));
        let mut outer = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut outer));
            let opts = SimpleFileOptions::default();
            w.start_file("base.apk", opts).unwrap();
            w.write_all(&inner).unwrap();
            w.start_file("manifest.json", opts).unwrap();
            w.write_all(b"{}").unwrap();
            w.finish().unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        let xapk_path = dir.path().join("app.xapk");
        fs::write(&xapk_path, &outer).unwrap();

        let out = dir.path().join("out");
        let report = run(&xapk_path, &out).unwrap();
        assert!(report.native_lib.is_some());
        assert_eq!(
            report.metadata_status,
            Some(MetadataStatus::Ok { version: 27 })
        );
    }

    #[test]
    fn extracts_ipa_and_detects_fairplay() {
        let macho = minimal_macho_arm64(1);
        let mut ipa = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut ipa));
            let opts = SimpleFileOptions::default();
            w.start_file("Payload/Arknights.app/Arknights", opts)
                .unwrap();
            w.write_all(&macho).unwrap();
            w.start_file(
                "Payload/Arknights.app/Data/Managed/Metadata/global-metadata.dat",
                opts,
            )
            .unwrap();
            w.write_all(&metadata_bytes(GLOBAL_METADATA_MAGIC, 27))
                .unwrap();
            w.finish().unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        let ipa_path = dir.path().join("app.ipa");
        fs::write(&ipa_path, &ipa).unwrap();

        let out = dir.path().join("out");
        let report = run(&ipa_path, &out).unwrap();
        assert_eq!(
            report.macho_status,
            Some(MachOStatus::FairPlayEncrypted { cryptid: 1 })
        );
        assert_eq!(
            report.metadata_status,
            Some(MetadataStatus::Ok { version: 27 })
        );
        assert!(out.join("Arknights").exists());
    }
}
