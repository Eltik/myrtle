use std::{io, path::Path};

use super::manifest::ResourceManifest;
use super::text_asset::export_text_asset;
use crate::unity::bundle::BundleFile;
use crate::unity::object_reader::read_object;
use crate::unity::serialized_file::SerializedFile;
use walkdir::WalkDir;

pub fn export_gamedata(
    bundle_dir: &Path,
    idx_path: &Path,
    output_dir: &Path,
) -> Result<usize, io::Error> {
    let manifest = ResourceManifest::load(idx_path)?;
    let mut exported = 0;

    for entry in WalkDir::new(bundle_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        // Only process .bin files
        if path.extension().is_none_or(|ext| ext != "bin") {
            continue;
        }

        let data = std::fs::read(path)?;
        let bundle = match BundleFile::parse(data) {
            Ok(b) => b,
            Err(_) => continue,
        };

        if bundle.files.is_empty() {
            continue;
        }

        let sf = match SerializedFile::parse(bundle.files[0].data.clone()) {
            Ok(sf) => sf,
            Err(_) => continue,
        };

        for obj in &sf.objects {
            if obj.class_id != 49 {
                continue;
            } // TextAsset only

            let val = match read_object(&sf, obj) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let name = match val["m_Name"].as_str() {
                Some(n) => n,
                None => continue,
            };

            // Look up real path from manifest
            let real_path = match manifest.get_output_path(name) {
                Some(p) => p,
                None => continue,
            };

            // Only extract gamedata files
            if !real_path.starts_with("gamedata/") {
                continue;
            }

            // Build output directory from the real path
            let out_path = output_dir.join(real_path);
            let out_parent = out_path.parent().unwrap_or(output_dir);
            std::fs::create_dir_all(out_parent)?;

            // Export using text_asset (handles AES decrypt + JSON detection)
            let file_stem = Path::new(real_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(name);
            match export_text_asset(&val, out_parent, Some(file_stem)) {
                Ok(()) => exported += 1,
                Err(e) => {
                    eprintln!("  error exporting {name}: {e}");
                }
            }
        }
    }

    Ok(exported)
}
