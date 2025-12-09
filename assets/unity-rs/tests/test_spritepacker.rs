use std::cell::RefCell;
use std::rc::Rc;
use unity_rs::helpers::import_helper::FileSource;
use unity_rs::Environment;

fn print_typetree_node(node: &unity_rs::helpers::type_tree_node::TypeTreeNode, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}{}[{}] : {}", prefix, node.m_name, node.m_type, node.m_byte_size);
    for child in &node.m_children {
        print_typetree_node(child, indent + 1);
    }
}

#[test]
fn test_spritepacker_typetree() {
    // Load portrait_hub.ab
    let file_path = "/Users/eltik/Documents/Coding/myrtle.moe/assets/downloader/ArkAssets/arts/charportraits/portrait_hub.ab";

    let mut env = Environment::new();
    env.load_file(FileSource::Path(file_path.to_string()), None, false);

    let env_rc = Rc::new(RefCell::new(env));
    Environment::set_environment_references(&env_rc).unwrap();

    let env = env_rc.borrow();

    println!("\n=== Analyzing portrait_hub.ab MonoBehaviour TypeTree ===\n");

    for mut obj in env.objects() {
        if obj.obj_type == unity_rs::ClassIDType::MonoBehaviour {
            println!("MonoBehaviour PathID: {}", obj.path_id);

            // Check if serialized_type has node
            if let Some(ref st) = obj.serialized_type {
                println!("  serialized_type class_id: {}", st.class_id);
                println!("  script_type_index: {}", st.script_type_index);
                if let Some(ref class_name) = st.m_class_name {
                    println!("  class_name: {}", class_name);
                }
                if let Some(ref node) = st.node {
                    println!("\n=== TypeTree Structure ===");
                    print_typetree_node(node, 0);
                } else {
                    println!("  NO TypeTree node available in serialized_type");
                }
            } else {
                println!("  NO serialized_type available");
            }

            // Try to read the raw data with parse_as_dict
            println!("\n=== Trying parse_as_dict ===");
            match obj.clone().parse_as_object(None, false) {
                Ok(json) => {
                    println!("JSON:\n{}", serde_json::to_string_pretty(&json).unwrap());
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            break; // Just the first MonoBehaviour
        }
    }
}

#[test]
fn test_pack_spritepacker_typetree() {
    // Load pack0.ab - this has the actual SpritePacker MonoBehaviours
    let file_path = "/Users/eltik/Documents/Coding/myrtle.moe/assets/downloader/ArkAssets/arts/charportraits/pack0.ab";

    let mut env = Environment::new();
    env.load_file(FileSource::Path(file_path.to_string()), None, false);

    let env_rc = Rc::new(RefCell::new(env));
    Environment::set_environment_references(&env_rc).unwrap();

    let env = env_rc.borrow();

    println!("\n=== Analyzing pack0.ab SpritePacker MonoBehaviour TypeTree ===\n");

    let mut mono_count = 0;
    for mut obj in env.objects() {
        if obj.obj_type == unity_rs::ClassIDType::MonoBehaviour {
            mono_count += 1;
            if mono_count > 1 {
                continue; // Just show first one
            }

            println!("MonoBehaviour PathID: {}", obj.path_id);

            // Check if serialized_type has node
            if let Some(ref st) = obj.serialized_type {
                println!("  serialized_type class_id: {}", st.class_id);
                println!("  script_type_index: {}", st.script_type_index);
                if let Some(ref class_name) = st.m_class_name {
                    println!("  class_name: {}", class_name);
                }
                if let Some(ref ns) = st.m_name_space {
                    println!("  namespace: {}", ns);
                }
                if let Some(ref asm) = st.m_assembly_name {
                    println!("  assembly: {}", asm);
                }
                if let Some(ref node) = st.node {
                    println!("\n=== TypeTree Structure ===");
                    print_typetree_node(node, 0);
                } else {
                    println!("  NO TypeTree node available in serialized_type");
                }
            } else {
                println!("  NO serialized_type available");
            }

            // Clone the node first to avoid borrow issues
            let node_clone = obj.serialized_type.as_ref().and_then(|st| st.node.clone());

            // Try to read the raw data using read_typetree directly
            println!("\n=== Trying read_typetree with serialized_type node ===");
            if let Some(node) = node_clone {
                // Reset reader position
                obj.reset();
                match obj.read_typetree(Some(node), false, false) {
                    Ok(json) => {
                        // Truncate large output
                        let json_str = serde_json::to_string_pretty(&json).unwrap();
                        if json_str.len() > 10000 {
                            println!("JSON (truncated to 10000 chars):\n{}...", &json_str[..10000]);
                        } else {
                            println!("JSON:\n{}", json_str);
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    println!("\nTotal MonoBehaviours in pack0.ab: {}", mono_count);
}
