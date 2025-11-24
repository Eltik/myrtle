use anyhow::Result;
use image::RgbaImage;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Thread-safe file saver with collision prevention
pub struct SafeSaver {
    saved_count: Arc<AtomicUsize>,
}

impl SafeSaver {
    pub fn new() -> Self {
        SafeSaver {
            saved_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the number of files saved
    pub fn count(&self) -> usize {
        self.saved_count.load(Ordering::Relaxed)
    }

    /// Reset counter
    pub fn reset(&self) {
        self.saved_count.store(0, Ordering::Relaxed);
    }

    /// Save bytes to file with collision prevention
    pub fn save_bytes(
        &self,
        data: &[u8],
        destdir: &Path,
        name: &str,
        ext: &str,
    ) -> Result<PathBuf> {
        fs::create_dir_all(destdir)?;

        let dest = destdir.join(format!("{}{}", name, ext));
        let final_dest = self.no_namesake(&dest);

        // Check for duplicate content
        if self.is_duplicate(data, &final_dest)? {
            return Ok(final_dest);
        }

        let mut file = fs::File::create(&final_dest)?;
        file.write_all(data)?;

        self.saved_count.fetch_add(1, Ordering::Relaxed);
        log::debug!("Saved: {:?}", final_dest);

        Ok(final_dest)
    }

    /// Save image to file
    pub fn save_image(&self, img: &RgbaImage, destdir: &Path, name: &str) -> Result<PathBuf> {
        use std::io::Cursor;

        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png)?;

        self.save_bytes(buffer.get_ref(), destdir, name, ".png")
    }

    /// Generate unique filename to prevent overwriting
    fn no_namesake(&self, dest: &Path) -> PathBuf {
        if !dest.exists() {
            return dest.to_path_buf();
        }

        let stem = dest.file_stem().unwrap_or_default().to_string_lossy();
        let ext = dest
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let parent = dest.parent().unwrap_or(Path::new(""));

        let mut counter = 0;
        loop {
            let new_name = format!("{}${}{}", stem, counter, ext);
            let new_path = parent.join(new_name);
            if !new_path.exists() {
                return new_path;
            }
            counter += 1;
        }
    }

    /// Check if identical content already exists
    fn is_duplicate(&self, data: &[u8], dest: &Path) -> Result<bool> {
        let destdir = dest.parent().unwrap_or(Path::new(""));
        if !destdir.is_dir() {
            return Ok(false);
        }

        let stem = dest.file_stem().unwrap_or_default().to_string_lossy();
        let ext = dest
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        for entry in fs::read_dir(destdir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(file_stem) = path.file_stem() {
                let file_stem = file_stem.to_string_lossy();
                let file_ext = path
                    .extension()
                    .map(|e| e.to_string_lossy().to_string())
                    .unwrap_or_default();

                if file_stem.starts_with(stem.as_ref()) && file_ext == ext {
                    if let Ok(existing_data) = fs::read(&path) {
                        if existing_data == data {
                            log::debug!("Duplicate prevented: {:?}", path);
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

impl Default for SafeSaver {
    fn default() -> Self {
        Self::new()
    }
}
