use std::io::Cursor;
use std::path::Path;

pub fn extract_zip(data: &[u8], savedir: &Path) -> anyhow::Result<()> {
    let reader = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(reader)?;
    archive.extract(savedir)?;
    Ok(())
}
