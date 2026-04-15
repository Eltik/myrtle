use unpacker::unity::bundle::BundleFile;
use unpacker::unity::serialized_file::SerializedFile;

#[test]
fn parse_serialized() {
    let path = format!(
        "{}/tests/assets/ui_skin_groups.ab",
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
    let entry = &bundle.files[0];

    let sf = SerializedFile::parse(entry.data.clone()).unwrap();
    assert!(!sf.unity_version.is_empty());
    assert!(sf.enable_type_tree);
    assert!(!sf.types.is_empty());
    assert!(!sf.objects.is_empty());
}
