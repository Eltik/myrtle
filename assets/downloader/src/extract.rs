use std::fs::File;
use std::path::Path;

pub fn extract_zip(zip_path: &Path, savedir: &Path) -> anyhow::Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive.extract(savedir)?;
    Ok(())
}
