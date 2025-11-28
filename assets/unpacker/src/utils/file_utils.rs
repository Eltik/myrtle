use std::fs;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

/// Image file extensions
const EXT_IMAGE: &[&str] = &[".png", ".jpg", ".jpeg", ".bmp", ".gif", ".tiff"];

/// Known asset file extensions
const EXT_KNOWN: &[&str] = &[
    ".atlas", ".skel", ".wav", ".mp3", ".m4a", ".mp4", ".avi", ".mov", ".mkv", ".flv",
];

/// Asset bundle extensions
const EXT_AB: &[&str] = &[".ab", ".bin"];

/// Check if path is an image file
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let ext_lower = format!(".{}", ext.to_lowercase());
            EXT_IMAGE.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// Check if path is a known asset file
pub fn is_known_asset_file(path: &Path) -> bool {
    if is_image_file(path) {
        return true;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let ext_lower = format!(".{}", ext.to_lowercase());
            EXT_KNOWN.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// Check if file is an asset bundle by extension
pub fn is_ab_file_by_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let ext_lower = format!(".{}", ext.to_lowercase());
            EXT_AB.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// Check if file is a Unity AssetBundle (by signature or extension)
pub fn is_ab_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    // Check extension first
    if is_ab_file_by_ext(path) {
        return true;
    }

    // Check file signature
    is_unity_bundle(path)
}

/// Check if file is a Unity bundle by signature only (not extension)
pub fn is_unity_bundle(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    if let Ok(mut file) = fs::File::open(path) {
        let mut header = [0u8; 8];
        if file.read_exact(&mut header).is_ok() {
            // UnityFS signature
            if &header[..7] == b"UnityFS" {
                return true;
            }
            // UnityWeb signature
            if &header[..8] == b"UnityWeb" {
                return true;
            }
            // UnityRaw signature
            if &header[..8] == b"UnityRaw" {
                return true;
            }
        }
    }

    false
}

/// Check if file is binary (not valid UTF-8 text)
pub fn is_binary_file(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = vec![0u8; 8192];
        if let Ok(n) = file.read(&mut buffer) {
            buffer.truncate(n);
            return std::str::from_utf8(&buffer).is_err();
        }
    }
    true
}

/// Get all files in directory recursively
pub fn get_file_list(path: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Get directory size in bytes
pub fn get_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Remove directory if it exists
pub fn rmdir(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Create directory if it doesn't exist
pub fn mkdir(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
