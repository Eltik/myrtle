use unpacker::unity::bundle::BundleFile;
use unpacker::unity::endian_reader::EndianReader;
use unpacker::unity::object_reader::{read_object, read_value};
use unpacker::unity::serialized_file::SerializedFile;
use unpacker::unity::type_tree::TypeTreeNode;

#[test]
fn read_array_corrupted_size_no_panic() {
    let element_node = TypeTreeNode {
        type_name: "int".to_string(),
        name: "data".to_string(),
        byte_size: 4,
        version: 0,
        level: 2,
        type_flags: 0,
        meta_flag: 0,
        children: vec![],
    };
    let size_node = TypeTreeNode {
        type_name: "int".to_string(),
        name: "size".to_string(),
        byte_size: 4,
        version: 0,
        level: 2,
        type_flags: 0,
        meta_flag: 0,
        children: vec![],
    };
    let array_node = TypeTreeNode {
        type_name: "Array".to_string(),
        name: "Array".to_string(),
        byte_size: -1,
        version: 0,
        level: 1,
        type_flags: 0,
        meta_flag: 0x4000,
        children: vec![size_node, element_node],
    };
    let root = TypeTreeNode {
        type_name: "vector".to_string(),
        name: "m_Container".to_string(),
        byte_size: -1,
        version: 0,
        level: 0,
        type_flags: 0,
        meta_flag: 0,
        children: vec![array_node],
    };

    // Data contains array size = -1 (0xFFFFFFFF) — wraps to usize::MAX, should not panic
    let mut data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // i32 = -1 in LE
    data.extend_from_slice(&[0u8; 64]);

    let mut r = EndianReader::new(&data, false);
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read_value(&mut r, &root)));
    assert!(
        result.is_ok(),
        "read_value should not panic on huge array size"
    );
    assert!(
        result.unwrap().is_err(),
        "should return Err for corrupted array size"
    );
}

#[test]
fn read_object_from_bundle() {
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
    let sf = SerializedFile::parse(bundle.files[0].data.clone()).unwrap();

    let targets = [28, 49, 83, 142];
    let mut read_count = 0;

    for obj in sf.objects.iter().filter(|o| targets.contains(&o.class_id)) {
        if let Ok(val) = read_object(&sf, obj) {
            // Verify it produces valid JSON
            assert!(serde_json::to_string(&val).is_ok());
            read_count += 1;
        }
    }

    assert!(
        read_count > 0,
        "should successfully read at least one object"
    );
}
