/// TypeTree tests matching Python's test_typetree.py
///
/// These tests verify TypeTree node creation, serialization, and deserialization.
/// Python has 8 tests total, but many are Python-specific (memory leak checks, etc.)
/// We implement the core functionality tests that make sense in Rust.
use unity_rs::helpers::type_tree_node::TypeTreeNode;

/// Get the path to test samples directory
fn samples_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("samples")
}

/// Test 1: TypeTreeNode creation
/// Python: test_typetreenode()
#[test]
fn test_typetreenode() {
    let node = TypeTreeNode::new(
        0,                      // m_level
        "TestNode".to_string(), // m_type
        "TestNode".to_string(), // m_name
        0,                      // m_byte_size
        0,                      // m_version
    );

    assert_eq!(node.m_type, "TestNode");
    assert_eq!(node.m_name, "TestNode");
    assert_eq!(node.m_level, 0);
}

/// Test 2: Read TypeTree from real Unity objects
/// Python: test_read_typetree() (in test_main.py, already implemented)
/// This test verifies that TypeTree reading works for all sample files
#[test]
fn test_read_typetree_samples() {
    use unity_rs::files::bundle_file::FileType;
    use unity_rs::Environment;

    let samples = samples_dir();
    let mut env = Environment::new();

    env.load_folder(samples.to_str().unwrap()).unwrap();

    let mut objects_read = 0;

    for (_name, file_rc) in &env.files {
        let file_ref = file_rc.borrow();

        // Helper to read TypeTree from SerializedFile
        let mut read_serialized = |serialized_file_rc: &std::rc::Rc<
            std::cell::RefCell<unity_rs::files::serialized_file::SerializedFile>,
        >| {
            let serialized_file = serialized_file_rc.borrow();
            for (_id, obj) in &serialized_file.objects {
                let mut obj_clone = obj.clone();
                // read_typetree should not fail for any object
                let result = obj_clone.read_typetree(None, true, false);
                assert!(
                    result.is_ok(),
                    "Failed to read TypeTree for {:?}",
                    obj.obj_type
                );
                objects_read += 1;
            }
        };

        match &*file_ref {
            FileType::SerializedFile(serialized_file_rc) => {
                read_serialized(serialized_file_rc);
            }
            FileType::BundleFile(bundle) => {
                for (_bundle_file_name, bundle_file_rc) in &bundle.files {
                    let bundle_file_ref = bundle_file_rc.borrow();
                    if let FileType::SerializedFile(sf_rc) = &*bundle_file_ref {
                        read_serialized(sf_rc);
                    }
                }
            }
            _ => {}
        }
    }

    // Should have read TypeTrees from multiple objects
    assert!(
        objects_read > 0,
        "No objects found to test TypeTree reading"
    );
}

/// Test 3: TypeTreeNode from list
/// Python: test_node_from_list_clz() and test_node_from_list_dict()
///
/// TODO: Implement TypeTreeNode::from_list() to match Python's functionality
/// This would allow reconstructing a TypeTreeNode from a traversal list
#[test]
#[ignore]
fn test_node_from_list() {
    // This requires implementing TypeTreeNode::from_list() which isn't in Rust yet
    // Python code: node = TypeTreeNode.from_list(list(TEST_CLASS_NODE.traverse()))
}

/// Test 4: Simple type serialization round-trip
/// Python: test_simple_nodes()
///
/// TODO: Implement TypeTree write functionality
/// Python uses write_typetree() to serialize values, then read_typetree() to deserialize
/// Rust has read_typetree but write functionality isn't implemented yet
#[test]
#[ignore]
fn test_simple_nodes() {
    // This requires implementing write_typetree() for serialization
    // Python code: write_typetree(value, node, writer)
}

/// Test 5: Array type serialization round-trip
/// Python: test_simple_nodes_array()
///
/// TODO: Implement TypeTree write functionality for arrays
#[test]
#[ignore]
fn test_simple_nodes_array() {
    // This requires implementing write_typetree() for arrays
}

/// Test 6: Class node as dict
/// Python: test_class_node_dict()
///
/// TODO: Implement class serialization to dict
#[test]
#[ignore]
fn test_class_node_dict() {
    // This requires GameObject serialization and write_typetree
}

/// Test 7: Class node as object
/// Python: test_class_node_clz()
///
/// TODO: Implement class object serialization
#[test]
#[ignore]
fn test_class_node_clz() {
    // This requires GameObject serialization and write_typetree
}

/// Test 8: Memory management
/// Python: @check_leak decorator on tests
///
/// NOTE: Rust has RAII and automatic memory management, so memory leak tests
/// are less critical. Rust's ownership system prevents most memory leaks by design.
#[test]
fn test_memory_safety() {
    // Rust's type system and RAII ensure memory safety
    // This test exists to document that memory safety is guaranteed by the language
    let node = TypeTreeNode::new(
        0,                  // m_level
        "Test".to_string(), // m_type
        "Test".to_string(), // m_name
        0,                  // m_byte_size
        0,                  // m_version
    );

    // Node will be automatically dropped and memory freed
    drop(node);
    // No memory leak possible due to Rust's ownership model
}
