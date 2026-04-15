use std::path::PathBuf;

use unpacker::export::text_asset::export_text_asset;
use unpacker::unity::bundle::BundleFile;
use unpacker::unity::object_reader::read_object;
use unpacker::unity::serialized_file::SerializedFile;

#[test]
fn test_export_text_asset() {
    let path = format!(
        "{}/tests/assets/anon_23ff0ecb.bin",
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
    let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();

    let output_dir = PathBuf::from(format!("{}/test_output/text", env!("CARGO_MANIFEST_DIR")));
    std::fs::create_dir_all(&output_dir).unwrap();

    let mut exported = 0;
    for obj in &sf.objects {
        if obj.class_id != 49 {
            continue;
        }
        let val = read_object(&sf, obj).unwrap();
        if export_text_asset(&val, &output_dir, None).is_ok() {
            exported += 1;
        }
    }
    assert!(exported > 0, "should export at least one text asset");
}
