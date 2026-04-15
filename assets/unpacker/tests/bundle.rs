use unpacker::unity::bundle::BundleFile;

#[test]
fn parse_bundle() {
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
    assert!(!bundle.files.is_empty());
}
