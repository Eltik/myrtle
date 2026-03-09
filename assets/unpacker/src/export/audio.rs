use std::{io, path::Path};

use serde_json::Value;

pub fn export_audio(_obj: &Value, _output_dir: &Path) -> Result<(), io::Error> {
    Ok(())
}
