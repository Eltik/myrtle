//! TypeTreeGenerator - Generates type tree structures from Unity assemblies
//!
//! This module provides functionality for extracting type information from
//! Unity .NET assemblies (DLLs) or IL2CPP binaries to generate TypeTreeNodes.
//!
//! This is a PyO3 implementation that bridges to Python's TypeTreeGeneratorAPI

use super::type_tree_node::TypeTreeNode;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Base trait for TypeTreeGenerator implementations
///
/// This mimics the external `TypeTreeGeneratorAPI` package used by Python.
///
/// Python equivalent: TypeTreeGeneratorAPI.TypeTreeGenerator (external package)
trait TypeTreeGeneratorBase {
    /// Initialize the generator with a Unity version
    fn new(unity_version: &str) -> Result<Self, io::Error>
    where
        Self: Sized;

    /// Load a .NET assembly DLL
    fn load_dll(&mut self, dll: Vec<u8>) -> Result<(), io::Error>;

    /// Load IL2CPP binaries (GameAssembly.dll + global-metadata.dat)
    fn load_il2cpp(&mut self, il2cpp: Vec<u8>, metadata: Vec<u8>) -> Result<(), io::Error>;

    /// Get type tree nodes as JSON string
    fn get_nodes_as_json(&self, assembly: &str, fullname: &str) -> Result<String, io::Error>;

    /// Get type tree nodes as TypeTreeNode list
    fn get_nodes(&self, assembly: &str, fullname: &str) -> Result<Vec<TypeTreeNode>, io::Error>;
}

/// PyO3 bridge to Python's TypeTreeGeneratorAPI
#[derive(Debug)]
struct PyTypeTreeGeneratorBase {
    /// Python object: TypeTreeGeneratorAPI.TypeTreeGenerator instance
    py_generator: Py<PyAny>,
}

impl TypeTreeGeneratorBase for PyTypeTreeGeneratorBase {
    fn new(unity_version: &str) -> Result<Self, io::Error> {
        Python::with_gil(|py| {
            // Import TypeTreeGeneratorAPI module
            let module = PyModule::import(py, "TypeTreeGeneratorAPI").map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to import TypeTreeGeneratorAPI: {}", e),
                )
            })?;

            // Get TypeTreeGenerator class
            let generator_class = module.getattr("TypeTreeGenerator").map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("TypeTreeGenerator class not found: {}", e),
                )
            })?;

            // Call TypeTreeGenerator(unity_version)
            let py_generator = generator_class.call1((unity_version,)).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to create TypeTreeGenerator: {}", e),
                )
            })?;

            Ok(PyTypeTreeGeneratorBase {
                py_generator: py_generator.unbind(),
            })
        })
    }

    fn load_dll(&mut self, dll: Vec<u8>) -> Result<(), io::Error> {
        Python::with_gil(|py| {
            // Convert Rust Vec<u8> to Python bytes
            let py_bytes = PyBytes::new(py, &dll);

            // Call self.py_generator.load_dll(dll)
            self.py_generator
                .bind(py)
                .call_method1("load_dll", (py_bytes,))
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to load DLL: {}", e),
                    )
                })?;

            Ok(())
        })
    }

    fn load_il2cpp(&mut self, il2cpp: Vec<u8>, metadata: Vec<u8>) -> Result<(), io::Error> {
        Python::attach(|py| {
            let il2cpp_bytes = PyBytes::new(py, &il2cpp);
            let metadata_bytes = PyBytes::new(py, &metadata);

            self.py_generator
                .bind(py)
                .call_method1("load_il2cpp", (il2cpp_bytes, metadata_bytes))
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to load IL2CPP: {}", e),
                    )
                })?;

            Ok(())
        })
    }

    fn get_nodes_as_json(&self, assembly: &str, fullname: &str) -> Result<String, io::Error> {
        Python::attach(|py| {
            let json_str: String = self
                .py_generator
                .bind(py)
                .call_method1("get_nodes_as_json", (assembly, fullname))?
                .extract()
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to get nodes as JSON: {}", e),
                    )
                })?;

            Ok(json_str)
        })
    }

    fn get_nodes(&self, assembly: &str, fullname: &str) -> Result<Vec<TypeTreeNode>, io::Error> {
        Python::with_gil(|py| {
            // Call Python: generator.get_nodes_as_json(assembly, fullname)
            let json_str: String = self
                .py_generator
                .bind(py)
                .call_method1("get_nodes_as_json", (assembly, fullname))?
                .extract()
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to extract JSON: {}", e),
                    )
                })?;

            // Parse JSON into Vec<TypeTreeNode>
            let nodes: Vec<TypeTreeNode> = serde_json::from_str(&json_str).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse nodes JSON: {}", e),
                )
            })?;

            Ok(nodes)
        })
    }
}

/// TypeTreeGenerator - Wrapper around TypeTreeGeneratorBase with caching and convenience methods
///
/// This provides the same API as Python's UnityPy.helpers.TypeTreeGenerator.TypeTreeGenerator.
/// Fully functional via PyO3 bridge to Python's TypeTreeGeneratorAPI.
///
/// Python equivalent: TypeTreeGenerator.py lines 20-90
#[derive(Debug)]
pub struct TypeTreeGenerator {
    base: PyTypeTreeGeneratorBase,
    cache: HashMap<(String, String), TypeTreeNode>,
    unity_version: String,
}

impl TypeTreeGenerator {
    /// Creates a new TypeTreeGenerator for a specific Unity version
    ///
    /// # Arguments
    ///
    /// * `unity_version` - Unity version string (e.g., "2019.4.1f1")
    ///
    /// # Returns
    ///
    /// TypeTreeGenerator instance or error if TypeTreeGeneratorAPI is not installed
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: __init__ (lines 23-25)
    pub fn new(unity_version: &str) -> Result<Self, io::Error> {
        // Create PyO3 bridge to Python's TypeTreeGeneratorAPI
        let base = PyTypeTreeGeneratorBase::new(unity_version)?;

        Ok(TypeTreeGenerator {
            base,
            cache: HashMap::new(),
            unity_version: unity_version.to_string(),
        })
    }

    /// Loads assemblies from a Unity game installation directory
    ///
    /// Auto-detects whether the game uses IL2CPP or Mono and loads accordingly.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - Path to game root directory (contains .exe and GameName_Data folder)
    ///
    /// # Expected Structure
    ///
    /// **IL2CPP games:**
    /// ```text
    /// root_dir/
    ///   ├── GameAssembly.dll
    ///   └── GameName_Data/
    ///       └── il2cpp_data/
    ///           └── Metadata/
    ///               └── global-metadata.dat
    /// ```
    ///
    /// **Mono games:**
    /// ```text
    /// root_dir/
    ///   └── GameName_Data/
    ///       └── Managed/
    ///           ├── Assembly-CSharp.dll
    ///           └── ... other DLLs
    /// ```
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: load_local_game() (lines 27-41)
    pub fn load_local_game(&mut self, root_dir: &str) -> Result<(), io::Error> {
        let root_path = Path::new(root_dir);

        // List files in root directory
        let root_files: Vec<String> = fs::read_dir(root_path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();

        // Find the *_Data directory
        let data_dir_name = root_files
            .iter()
            .find(|name| name.ends_with("_Data"))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Could not find *_Data directory in game root",
                )
            })?;

        let data_dir = root_path.join(data_dir_name);

        // Check if IL2CPP or Mono
        if root_files.contains(&"GameAssembly.dll".to_string()) {
            // IL2CPP game
            let ga_path = root_path.join("GameAssembly.dll");
            let gm_path = data_dir
                .join("il2cpp_data")
                .join("Metadata")
                .join("global-metadata.dat");

            let ga_raw = fs::read(ga_path)?;
            let gm_raw = fs::read(gm_path)?;

            self.load_il2cpp(ga_raw, gm_raw)?;
        } else {
            // Mono game
            let managed_dir = data_dir.join("Managed");
            self.load_local_dll_folder(managed_dir.to_str().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 in path")
            })?)?;
        }

        Ok(())
    }

    /// Loads all DLL files from a directory
    ///
    /// Typically used with the Managed/ folder in Mono Unity games.
    ///
    /// # Arguments
    ///
    /// * `dll_dir` - Path to directory containing .dll files
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: load_local_dll_folder() (lines 43-48)
    pub fn load_local_dll_folder(&mut self, dll_dir: &str) -> Result<(), io::Error> {
        let dll_path = Path::new(dll_dir);

        for entry in fs::read_dir(dll_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "dll" {
                        let data = fs::read(&path)?;
                        self.load_dll(data)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Loads a single .NET assembly DLL
    ///
    /// # Arguments
    ///
    /// * `dll` - DLL file contents as bytes
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: inherited from base class (line 14)
    pub fn load_dll(&mut self, dll: Vec<u8>) -> Result<(), io::Error> {
        self.base.load_dll(dll)
    }

    /// Loads IL2CPP binaries
    ///
    /// # Arguments
    ///
    /// * `il2cpp` - GameAssembly.dll contents
    /// * `metadata` - global-metadata.dat contents
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: inherited from base class (line 15)
    pub fn load_il2cpp(&mut self, il2cpp: Vec<u8>, metadata: Vec<u8>) -> Result<(), io::Error> {
        self.base.load_il2cpp(il2cpp, metadata)
    }

    /// Gets type tree nodes and builds hierarchical structure with caching
    ///
    /// This is the main method for retrieving type information. It:
    /// 1. Checks cache for previously generated nodes
    /// 2. Calls get_nodes() to get flat node list from base
    /// 3. Builds hierarchical structure using stack-based algorithm
    /// 4. Caches the result
    ///
    /// # Arguments
    ///
    /// * `assembly` - Assembly name (without .dll extension)
    /// * `fullname` - Fully qualified type name (e.g., "UnityEngine.GameObject")
    ///
    /// # Returns
    ///
    /// Root TypeTreeNode with all children populated
    ///
    /// # Python equivalent
    /// TypeTreeGenerator.py: get_nodes_up() (lines 50-90)
    pub fn get_nodes_up(
        &mut self,
        assembly: &str,
        fullname: &str,
    ) -> Result<TypeTreeNode, io::Error> {
        // Check cache
        let cache_key = (assembly.to_string(), fullname.to_string());
        if let Some(cached_root) = self.cache.get(&cache_key) {
            return Ok(cached_root.clone());
        }

        // Get flat node list from base
        let base_nodes = self
            .base
            .get_nodes(&format!("{}.dll", assembly), fullname)?;

        if base_nodes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No nodes returned from base API",
            ));
        }

        // Build hierarchical structure
        // Python lines 57-87: Creates root node, then iterates through remaining nodes
        // building parent-child relationships using a stack
        let base_root = &base_nodes[0];
        let mut root = TypeTreeNode::new(
            base_root.m_level,
            base_root.m_type.clone(),
            base_root.m_name.clone(),
            0, // Python sets byte_size to 0
            0, // Python sets version to 0
        );
        root.m_meta_flag = base_root.m_meta_flag;

        let mut stack: Vec<*mut TypeTreeNode> = Vec::new();
        let mut parent: *mut TypeTreeNode = &mut root;
        let mut prev: *mut TypeTreeNode = &mut root;

        // Process remaining nodes
        for base_node in &base_nodes[1..] {
            let mut node = TypeTreeNode::new(
                base_node.m_level,
                base_node.m_type.clone(),
                base_node.m_name.clone(),
                0,
                0,
            );
            node.m_meta_flag = base_node.m_meta_flag;

            unsafe {
                // Adjust parent based on level changes
                if node.m_level > (*prev).m_level {
                    // Going deeper - push current parent, previous becomes new parent
                    stack.push(parent);
                    parent = prev;
                } else if node.m_level < (*prev).m_level {
                    // Going up - pop stack until we find correct parent level
                    while node.m_level <= (*parent).m_level {
                        parent = stack.pop().ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Stack underflow in node tree",
                            )
                        })?;
                    }
                }

                // Add node as child of current parent
                (*parent).m_children.push(node);

                // Update prev to point to the node we just added
                let children_len = (*parent).m_children.len();
                prev = &mut (*parent).m_children[children_len - 1] as *mut TypeTreeNode;
            }
        }

        // Cache and return
        self.cache.insert(cache_key, root.clone());
        Ok(root)
    }
}
