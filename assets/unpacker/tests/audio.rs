use std::collections::HashMap;
use std::path::PathBuf;

use unpacker::export::audio::export_audio;
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

#[test]
fn test_export_audio() {
    let path = format!(
        "{}/tests/assets/audio_char_002_amiya.ab",
        env!("CARGO_MANIFEST_DIR")
    );
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("skip: {e}");
            return;
        }
    };
    let bundle = BundleFile::parse(data).unwrap();

    let mut resources = HashMap::new();
    for entry in &bundle.files {
        if entry.path.ends_with(".resS") || entry.path.ends_with(".resource") {
            let filename = entry.path.rsplit('/').next().unwrap_or(&entry.path);
            resources.insert(filename.to_string(), entry.data.clone());
        }
    }

    let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();
    let output_dir = PathBuf::from(format!("{}/test_output/audio", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut exported = 0;
    for obj in &sf.objects {
        if obj.class_id != 83 {
            continue;
        }
        let val = read_object(&sf, obj).unwrap();
        if export_audio(&val, &output_dir, &resources).is_ok() {
            exported += 1;
        }
    }
    assert!(exported > 0, "should export at least one audio clip");
}
