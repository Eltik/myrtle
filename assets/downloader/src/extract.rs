use std::fs::File;
use std::path::Path;

/// # Errors
///
/// Returns an error if the zip file cannot be opened, is not a valid archive,
/// or its contents cannot be extracted to `savedir`.
pub fn extract_zip(zip_path: &Path, savedir: &Path) -> anyhow::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(savedir)?;
    Ok(())
}
